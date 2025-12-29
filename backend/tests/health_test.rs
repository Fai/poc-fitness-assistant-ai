//! Integration tests for health check endpoints

mod common;

use axum::http::StatusCode;

#[tokio::test]
#[ignore = "requires database"]
async fn test_health_endpoint() {
    let app = common::TestApp::new().await;
    
    let (status, body) = app.get("/health").await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("healthy"));
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_liveness_endpoint() {
    let app = common::TestApp::new().await;
    
    let (status, body) = app.get("/health/live").await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("alive"));
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_readiness_endpoint() {
    let app = common::TestApp::new().await;
    
    let (status, body) = app.get("/health/ready").await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("ready"));
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_api_v1_root() {
    let app = common::TestApp::new().await;
    
    let (status, body) = app.get("/api/v1/").await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Fitness Assistant API v1"));
}
