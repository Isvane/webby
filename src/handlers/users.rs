use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use std::sync::Arc;
use validator::Validate;

use crate::errors::{ApiResponse, AppError};
use crate::models::{AppState, CreateUser, Pagination, User};

pub async fn about() -> (StatusCode, &'static str) {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    (StatusCode::OK, "I'm the user")
}

pub async fn greet_user(Path(name): Path<String>) -> ApiResponse {
    ApiResponse::Message(StatusCode::OK, format!("Hello {name}"))
}

pub async fn delete_user(
    _claims: crate::auth::Claims,
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

pub async fn create_user(
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

    Ok(ApiResponse::Message(
        StatusCode::CREATED,
        "Created user successfully".to_string(),
    ))
}

pub async fn list_users(
    _claims: crate::auth::Claims,
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Result<ApiResponse, AppError> {
    tracing::info!("Attempting to fetch user data");
    let mut db = state.db.clone();

    let per_page = pagination.per_page.unwrap_or(20);

    let users = User::all()
        .limit(per_page.try_into().unwrap())
        .exec(&mut db)
        .await?;

    Ok(ApiResponse::Json(users))
}
