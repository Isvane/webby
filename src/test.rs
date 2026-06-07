use super::*;
use crate::auth::sign_token;
use crate::models::User;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::{Service, ServiceExt};

fn get_test_token(user_id: &str) -> String {
    sign_token(user_id.to_string(), "TestCompany".to_string()).expect("Failed to sign test token")
}

async fn setup_test_app() -> axum::Router {
    unsafe {
        std::env::set_var("JWT_SECRET", "test_super_secret_key_123");
    }

    let db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");

    db.push_schema()
        .await
        .expect("Failed to sync test database schema");

    app(db)
}

#[tokio::test]
async fn test_index_handler() {
    let app = setup_test_app().await;

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
    let app = setup_test_app().await;

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
    let app = setup_test_app().await;

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
    let app = setup_test_app().await;

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

    assert_eq!(&body[..], b"Created user successfully");
}

#[tokio::test]
async fn test_delete_user_handle() {
    let mut app = setup_test_app().await;

    let response1 = app
        .call(
            Request::builder()
                .method("POST")
                .uri("/users/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "John", "email": "john@testmail.com"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);

    let token = get_test_token("1");

    let response2 = app
        .call(
            Request::builder()
                .method("DELETE")
                .uri("/users/delete/1")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();

    assert_eq!(&body[..], b"Deleted user: 1");

    let response3 = app
        .call(
            Request::builder()
                .method("DELETE")
                .uri("/users/delete/1")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response3.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_user_handle() {
    let mut app = setup_test_app().await;

    let response1 = app
        .call(
            Request::builder()
                .method("POST")
                .uri("/users/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "John", "email": "john@testmail.com"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);

    let token = get_test_token("1");

    let response2 = app
        .call(
            Request::builder()
                .method("PATCH")
                .uri("/users/update/1")
                .header("Authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "Johny", "email": "john@testmail.com"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();

    assert_eq!(&body[..], b"Updated user: 1");
}

#[tokio::test]
async fn test_list_users_handle() {
    let mut app = setup_test_app().await;

    let response1 = app
        .call(
            Request::builder()
                .method("POST")
                .uri("/users/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "John", "email": "john@testmail.com"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response1.status(), StatusCode::CREATED);

    let token = get_test_token("1");

    let response2 = app
        .call(
            Request::builder()
                .uri("/users/list")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response2.into_body(), usize::MAX)
        .await
        .unwrap();

    let users: Vec<User> = serde_json::from_slice(&body).unwrap();

    assert_eq!(users.len(), 1);
}

#[tokio::test]
async fn test_list_items() {
    let app = setup_test_app().await;

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

#[tokio::test]
async fn test_validator_name() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/users/create")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name": "", "email": "Isvane@testmail.com"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let body_string = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_string.contains("Name cannot be empty"));
}

#[tokio::test]
async fn test_validator_email() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/users/create")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name": "Isvane", "email": "not-email"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let body_string = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_string.contains("Invalid email address"));
}
