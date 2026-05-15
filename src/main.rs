use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::{signal, sync::Mutex};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing::info_span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(test)]
mod test;

#[derive(Clone)]
struct AppState {
    users: Arc<Mutex<Vec<CreateUser>>>,
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct CreateUser {
    pub(crate) name: String,
    pub(crate) email: String,
}

enum ApiResponse {
    Json(Vec<CreateUser>),
    Message(StatusCode, String),
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Json(data) => (StatusCode::OK, Json(data)).into_response(),
            Self::Message(status, msg) => (status, msg).into_response(),
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let app = app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

pub(crate) fn app() -> Router {
    let whatever = Arc::new(Mutex::new(Vec::new()));

    let state = AppState { users: whatever };

    let user_routes = Router::new()
        .route("/", get(about))
        .route("/list", get(list_users))
        .route("/create", post(create_user))
        .route("/greet/{name}", get(greet_user));

    Router::new()
        .route("/", get(index))
        .route("/pages", get(list_items))
        .nest("/users", user_routes)
        .layer((
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                )
            }),
            TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(10)),
        ))
        .with_state(state)
}

async fn index() -> (StatusCode, &'static str) {
    (StatusCode::ACCEPTED, "Goodbye, World!")
}

async fn about() -> (StatusCode, &'static str) {
    (StatusCode::OK, "I'm the user")
}
async fn greet_user(Path(name): Path<String>) -> ApiResponse {
    ApiResponse::Message(StatusCode::OK, format!("Hello {name}"))
}

async fn create_user(State(state): State<AppState>, Json(input): Json<CreateUser>) -> ApiResponse {
    tracing::info!("Attempting to create user: {}", input.email);

    let mut users = state.users.lock().await;

    users.push(CreateUser {
        name: input.name.clone(),
        email: input.email.clone(),
    });

    ApiResponse::Message(
        StatusCode::CREATED,
        format!("Created user: {} ({})", input.name, input.email),
    )
}

async fn list_items(Query(pagination): Query<Pagination>) -> String {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20);
    format!("Page {page}, {per_page} items")
}

async fn list_users(State(state): State<AppState>) -> ApiResponse {
    tracing::info!("Attempting to fetch user data");

    let users = state.users.lock().await;

    ApiResponse::Json(users.clone())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
