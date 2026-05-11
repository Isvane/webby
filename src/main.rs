use axum::{Router, extract::Path, extract::Query, routing::get};
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/about", get(about))
        .route("/greet/{name}", get(greet_user))
        .route("/pages", get(list_items));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> &'static str {
    "Goodbye, World!"
}
async fn about() -> &'static str {
    "About"
}
async fn greet_user(Path(name): Path<String>) -> String {
    format!("Hello, {name}")
}

async fn list_items(Query(pagination): Query<Pagination>) -> String {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20);
    format!("Page {page}, {per_page} items")
}
