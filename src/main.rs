use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info_span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use validator::Validate;

#[cfg(test)]
mod test;

#[derive(Debug, toasty::Model, Serialize, Deserialize, Clone)]
pub(crate) struct User {
    #[key]
    #[auto]
    pub(crate) id: u64,
    pub(crate) name: String,

    #[unique]
    pub(crate) email: String,
}

struct AppState {
    db: toasty::db::Db,
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[derive(Deserialize, Validate)]
pub(crate) struct CreateUser {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub(crate) name: String,
    #[validate(email(message = "Invalid email address"))]
    pub(crate) email: String,
}

enum ApiResponse {
    Json(Vec<User>),
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

enum AppError {
    EmailAlreadyExist(String),
    InvalidInput(String),
    UserNotFound(String),
    InternalDbError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::EmailAlreadyExist(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            Self::InvalidInput(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg).into_response(),
            Self::UserNotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
            Self::InternalDbError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
        }
    }
}

impl From<toasty::Error> for AppError {
    fn from(err: toasty::Error) -> Self {
        AppError::InternalDbError(err.to_string())
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

    let db_path = "./src/database/app.sqlite";
    let db_dir = std::path::Path::new(db_path).parent().unwrap();

    if let Err(e) = std::fs::create_dir_all(db_dir) {
        eprintln!("Warning: Failed to create database directory: {}", e);
    }

    let db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(format!("sqlite:{}", db_path).as_str())
        .await
        .expect("Failed to connect to database");

    match db.push_schema().await {
        Ok(_) => println!("Database schema initialized"),
        Err(e) => {
            if !e.to_string().contains("already exist") {
                panic!("Failed to sync database: {}", e)
            }
            println!("Schema already exists, skipping initialization");
        }
    }

    let app = app(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

pub(crate) fn app(db: toasty::db::Db) -> Router {
    let state = Arc::new(AppState { db });

    let user_routes = Router::new()
        .route("/", get(about))
        .route("/list", get(list_users))
        .route("/create", post(create_user))
        .route("/delete/{id}", delete(delete_user))
        .route("/greet/{name}", get(greet_user))
        .layer(ConcurrencyLimitLayer::new(5));

    Router::new()
        .route("/", get(index))
        .route("/pages", get(list_items))
        .nest("/users", user_routes)
        .nest_service("/assets", ServeDir::new("public"))
        .fallback_service(
            ServeDir::new("public").not_found_service(ServeFile::new("public/index.html")),
        )
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
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    (StatusCode::OK, "I'm the user")
}
async fn greet_user(Path(name): Path<String>) -> ApiResponse {
    ApiResponse::Message(StatusCode::OK, format!("Hello {name}"))
}

async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<ApiResponse, AppError> {
    let mut db = state.db.clone();

    let user = User::get_by_id(&mut db, &id)
        .await
        .map_err(|_| AppError::UserNotFound("User with that ID not found".to_string()))?;

    user.delete().exec(&mut db).await?;

    Ok(ApiResponse::Message(
        StatusCode::OK,
        format!("Deleted user: {id}"),
    ))
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateUser>,
) -> Result<ApiResponse, AppError> {
    if let Err(errors) = input.validate() {
        return Err(AppError::InvalidInput(errors.to_string()));
    }

    tracing::info!("Attempting to create user: {}", input.email);

    let mut db = state.db.clone();

    if User::get_by_email(&mut db, &input.email).await.is_ok() {
        return Err(AppError::EmailAlreadyExist(
            "User with this email already exists".to_string(),
        ));
    }

    let CreateUser { name, email } = input;

    let _new_user = toasty::create!(User { name, email }).exec(&mut db).await?;

    let message = "Created user successfully".to_string();
    Ok(ApiResponse::Message(StatusCode::CREATED, message))
}

async fn list_items(Query(pagination): Query<Pagination>) -> String {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20);
    format!("Page {page}, {per_page} items")
}

async fn list_users(State(state): State<Arc<AppState>>) -> Result<ApiResponse, AppError> {
    tracing::info!("Attempting to fetch user data");
    let mut db = state.db.clone();

    let users = User::all().exec(&mut db).await?;

    Ok(ApiResponse::Json(users))
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
