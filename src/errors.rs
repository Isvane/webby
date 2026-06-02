use crate::models::User;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub enum ApiResponse {
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

pub enum AppError {
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
