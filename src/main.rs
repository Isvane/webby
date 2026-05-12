use axum::{
    Json, Router,
    extract::Path,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

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
    let user_routes = Router::new()
        .route("/", get(about))
        .route("/list", get(list_users))
        .route("/create", post(create_user))
        .route("/greet/{name}", get(greet_user));

    let app = Router::new()
        .route("/", get(index))
        .route("/pages", get(list_items))
        .nest("/users", user_routes);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> (StatusCode, &'static str) {
    (StatusCode::ACCEPTED, "Goodbye, World!")
}
async fn about() -> &'static str {
    "About"
}
async fn greet_user(Path(name): Path<String>) -> ApiResponse {
    ApiResponse::Message(StatusCode::OK, format!("Hello {name}"))
}

async fn create_user(Json(input): Json<CreateUser>) -> ApiResponse {
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
    ApiResponse::Json(vec![CreateUser {
        email: "alice@mail.com".into(),
        name: "Alice".into(),
    }])
}
