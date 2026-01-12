//! Integration tests for weight tracking endpoints

mod common;

use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
#[ignore = "requires database"]
async fn test_log_weight_requires_auth() {
    let app = common::TestApp::new().await;
    
    let body = json!({
        "weight": 75.5
    });
    
    let (status, _) = app.post("/api/v1/weight", &body.to_string()).await;
    
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_log_weight_success() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let body = json!({
        "weight": 75.5
    });
    
    let (status, response) = app.post_auth("/api/v1/weight", &body.to_string(), &token).await;
    
    assert_eq!(status, StatusCode::CREATED);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["weight_kg"], 75.5);
    assert!(!response["id"].as_str().unwrap().is_empty());
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_log_weight_with_unit_conversion() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    // Log weight in pounds
    let body = json!({
        "weight": 165.0,
        "unit": "lbs"
    });
    
    let (status, response) = app.post_auth("/api/v1/weight", &body.to_string(), &token).await;
    
    assert_eq!(status, StatusCode::CREATED);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    // 165 lbs ≈ 74.84 kg
    let weight_kg = response["weight_kg"].as_f64().unwrap();
    assert!(weight_kg > 74.0 && weight_kg < 76.0);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_weight_history_empty() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let (status, response) = app.get_auth("/api/v1/weight", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["items"].as_array().unwrap().len(), 0);
    assert_eq!(response["total_count"], 0);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_weight_history_with_entries() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    // Log multiple weights
    for weight in [75.0, 74.8, 74.5] {
        let body = json!({ "weight": weight });
        app.post_auth("/api/v1/weight", &body.to_string(), &token).await;
    }
    
    let (status, response) = app.get_auth("/api/v1/weight", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["items"].as_array().unwrap().len(), 3);
    assert_eq!(response["total_count"], 3);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_weight_history_pagination() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    // Log 5 weights
    for i in 0..5 {
        let body = json!({ "weight": 75.0 + (i as f64 * 0.1) });
        app.post_auth("/api/v1/weight", &body.to_string(), &token).await;
    }
    
    // Get first page with limit 2
    let (status, response) = app.get_auth("/api/v1/weight?limit=2&offset=0", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["items"].as_array().unwrap().len(), 2);
    assert_eq!(response["total_count"], 5);
    assert_eq!(response["has_more"], true);
    
    // Get second page
    let (status, response) = app.get_auth("/api/v1/weight?limit=2&offset=2", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["items"].as_array().unwrap().len(), 2);
    assert_eq!(response["has_more"], true);
    
    // Get last page
    let (status, response) = app.get_auth("/api/v1/weight?limit=2&offset=4", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["items"].as_array().unwrap().len(), 1);
    assert_eq!(response["has_more"], false);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_weight_trend() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    // Log weights showing a downward trend
    for weight in [76.0, 75.5, 75.0, 74.5] {
        let body = json!({ "weight": weight });
        app.post_auth("/api/v1/weight", &body.to_string(), &token).await;
    }
    
    let (status, response) = app.get_auth("/api/v1/weight/trend", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["start_weight"], 76.0);
    assert_eq!(response["current_weight"], 74.5);
    assert!(response["total_change"].as_f64().unwrap() < 0.0); // Weight decreased
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_log_body_composition() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let body = json!({
        "body_fat_percent": 18.5,
        "muscle_mass_kg": 35.0,
        "water_percent": 55.0
    });
    
    let (status, response) = app.post_auth("/api/v1/body-composition", &body.to_string(), &token).await;
    
    assert_eq!(status, StatusCode::CREATED);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["body_fat_percent"], 18.5);
    assert_eq!(response["muscle_mass_kg"], 35.0);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_weight_anomaly_detection() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    // Log initial weight
    let body = json!({ "weight": 75.0 });
    app.post_auth("/api/v1/weight", &body.to_string(), &token).await;
    
    // Log anomalous weight (>2% change)
    let body = json!({ "weight": 80.0 }); // ~6.7% increase
    let (status, response) = app.post_auth("/api/v1/weight", &body.to_string(), &token).await;
    
    assert_eq!(status, StatusCode::CREATED);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["is_anomaly"], true);
}
