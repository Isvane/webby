use crate::models::Pagination;
use axum::{extract::Query, http::StatusCode};

pub async fn index() -> (StatusCode, &'static str) {
    (StatusCode::ACCEPTED, "Goodbye, World!")
}

pub async fn list_items(Query(pagination): Query<Pagination>) -> String {
    let page = pagination.page.unwrap_or(1);
    let per_page = pagination.per_page.unwrap_or(20);
    format!("Page {page}, {per_page} items")
}
