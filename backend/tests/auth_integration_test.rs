//! Integration tests for authentication endpoints

mod common;

use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
#[ignore = "requires database"]
async fn test_register_success() {
    let app = common::TestApp::new().await;
    
    let email = format!("register_test_{}@example.com", uuid::Uuid::new_v4());
    let body = json!({
        "email": email,
        "password": "SecurePassword123!"
    });
    
    let (status, response) = app.post("/api/v1/auth/register", &body.to_string()).await;
    
    assert_eq!(status, StatusCode::CREATED);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert!(!response["access_token"].as_str().unwrap().is_empty());
    assert!(!response["refresh_token"].as_str().unwrap().is_empty());
    assert_eq!(response["token_type"], "Bearer");
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_register_duplicate_email() {
    let app = common::TestApp::new().await;
    
    let email = format!("duplicate_{}@example.com", uuid::Uuid::new_v4());
    let body = json!({
        "email": email,
        "password": "SecurePassword123!"
    });
    
    // First registration should succeed
    let (status, _) = app.post("/api/v1/auth/register", &body.to_string()).await;
    assert_eq!(status, StatusCode::CREATED);
    
    // Second registration with same email should fail
    let (status, _) = app.post("/api/v1/auth/register", &body.to_string()).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_register_invalid_email() {
    let app = common::TestApp::new().await;
    
    let body = json!({
        "email": "not-an-email",
        "password": "SecurePassword123!"
    });
    
    let (status, _) = app.post("/api/v1/auth/register", &body.to_string()).await;
    
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_register_weak_password() {
    let app = common::TestApp::new().await;
    
    let body = json!({
        "email": "weak_password@example.com",
        "password": "123"
    });
    
    let (status, _) = app.post("/api/v1/auth/register", &body.to_string()).await;
    
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_login_success() {
    let app = common::TestApp::new().await;
    
    let email = format!("login_test_{}@example.com", uuid::Uuid::new_v4());
    let password = "SecurePassword123!";
    
    // Register first
    let register_body = json!({
        "email": email,
        "password": password
    });
    app.post("/api/v1/auth/register", &register_body.to_string()).await;
    
    // Then login
    let login_body = json!({
        "email": email,
        "password": password
    });
    let (status, response) = app.post("/api/v1/auth/login", &login_body.to_string()).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert!(!response["access_token"].as_str().unwrap().is_empty());
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_login_wrong_password() {
    let app = common::TestApp::new().await;
    
    let email = format!("wrong_pass_{}@example.com", uuid::Uuid::new_v4());
    
    // Register
    let register_body = json!({
        "email": email,
        "password": "CorrectPassword123!"
    });
    app.post("/api/v1/auth/register", &register_body.to_string()).await;
    
    // Login with wrong password
    let login_body = json!({
        "email": email,
        "password": "WrongPassword123!"
    });
    let (status, _) = app.post("/api/v1/auth/login", &login_body.to_string()).await;
    
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_login_nonexistent_user() {
    let app = common::TestApp::new().await;
    
    let body = json!({
        "email": "nonexistent@example.com",
        "password": "SomePassword123!"
    });
    
    let (status, _) = app.post("/api/v1/auth/login", &body.to_string()).await;
    
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_refresh_token() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let refresh_token = user.tokens.as_ref().unwrap().refresh_token.clone();
    
    let body = json!({
        "refresh_token": refresh_token
    });
    
    let (status, response) = app.post("/api/v1/auth/refresh", &body.to_string()).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert!(!response["access_token"].as_str().unwrap().is_empty());
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_refresh_token_invalid() {
    let app = common::TestApp::new().await;
    
    let body = json!({
        "refresh_token": "invalid-token"
    });
    
    let (status, _) = app.post("/api/v1/auth/refresh", &body.to_string()).await;
    
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_protected_endpoint_with_expired_token() {
    let app = common::TestApp::new().await;
    
    // Use a clearly invalid/expired token
    let fake_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwiZXhwIjoxfQ.invalid";
    
    let (status, _) = app.get_auth("/api/v1/profile", fake_token).await;
    
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
