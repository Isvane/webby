use serde::{Deserialize, Serialize};

#[derive(Debug, toasty::Model, Serialize, Deserialize, Clone)]
pub(crate) struct User {
    #[key]
    #[auto]
    pub(crate) id: u64,
    pub(crate) name: String,
    #[unique]
    pub(crate) email: String,
}

pub struct AppState {
    pub db: toasty::db::Db,
}

#[derive(Deserialize)]
pub struct Pagination {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Deserialize, validator::Validate)]
pub(crate) struct CreateUser {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub(crate) name: String,
    #[validate(email(message = "Invalid email address"))]
    pub(crate) email: String,
}

#[derive(Deserialize, validator::Validate)]
pub(crate) struct UpdateUser {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub(crate) name: String,
    #[validate(email(message = "Invalid email address"))]
    pub(crate) email: String,
}
