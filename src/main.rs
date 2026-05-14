use axum::{
    Json, Router,
    extract::Path,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::info_span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[derive(Deserialize, Serialize)]
struct CreateUser {
    name: String,
    email: String,
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
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = app();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

fn app() -> Router {
    let user_routes = Router::new()
        .route("/", get(about))
        .route("/list", get(list_users))
        .route("/create", post(create_user))
        .route("/greet/{name}", get(greet_user));

    Router::new()
        .route("/", get(index))
        .route("/pages", get(list_items))
        .nest("/users", user_routes)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                )
            }),
        )
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

async fn create_user(Json(input): Json<CreateUser>) -> ApiResponse {
    tracing::info!("Attempting to create user: {}", input.email);

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

async fn list_users() -> ApiResponse {
    tracing::info!("Attempting to fetch user data");

    ApiResponse::Json(vec![CreateUser {
        email: "alice@mail.com".into(),
        name: "Alice".into(),
    }])
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_index_handler() {
        let app = app();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"Goodbye, World!");
    }

    #[tokio::test]
    async fn test_about_handler() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_greet_handler() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/users/greet/isvane")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        assert_eq!(&body[..], b"Hello isvane")
    }

    #[tokio::test]
    async fn test_create_user_handle() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/users/create")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"name": "Isvane", "email": "isvane@testmail.com"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        assert_eq!(&body[..], b"Created user: Isvane (isvane@testmail.com)");
    }

    #[tokio::test]
    async fn test_list_users_handle() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/users/list")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let users: Vec<CreateUser> = serde_json::from_slice(&body).unwrap();

        assert_eq!(users.len(), 1);
        assert_eq!(users[0].name, "Alice");
        assert_eq!(users[0].email, "alice@mail.com");
    }

    #[tokio::test]
    async fn test_list_items() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/pages?page=2&per_page=50")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        assert_eq!(&body[..], b"Page 2, 50 items")
    }
}
