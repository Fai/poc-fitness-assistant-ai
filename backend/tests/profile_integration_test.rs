//! Integration tests for profile endpoints

mod common;

use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_profile_requires_auth() {
    let app = common::TestApp::new().await;
    
    let (status, _) = app.get("/api/v1/profile").await;
    
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_profile_success() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let (status, response) = app.get_auth("/api/v1/profile", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["email"], user.email);
    assert!(!response["id"].as_str().unwrap().is_empty());
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_update_profile_height() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let body = json!({
        "height": 175.0,
        "height_unit": "cm"
    });
    
    let (status, response) = app.put_auth("/api/v1/profile", &body.to_string(), &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["height"], 175.0);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_update_profile_invalid_height() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    // Height too low (< 50 cm)
    let body = json!({
        "height": 30.0,
        "height_unit": "cm"
    });
    
    let (status, _) = app.put_auth("/api/v1/profile", &body.to_string(), &token).await;
    
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_update_profile_activity_level() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let body = json!({
        "activity_level": "very_active"
    });
    
    let (status, response) = app.put_auth("/api/v1/profile", &body.to_string(), &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["activity_level"], "very_active");
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_settings() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let (status, response) = app.get_auth("/api/v1/profile/settings", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    // Check default values
    assert_eq!(response["weight_unit"], "kg");
    assert_eq!(response["distance_unit"], "km");
    assert_eq!(response["energy_unit"], "kcal");
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_update_settings() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let body = json!({
        "weight_unit": "lbs",
        "daily_calorie_goal": 2000,
        "daily_step_goal": 10000
    });
    
    let (status, response) = app.put_auth("/api/v1/profile/settings", &body.to_string(), &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["weight_unit"], "lbs");
    assert_eq!(response["daily_calorie_goal"], 2000);
    assert_eq!(response["daily_step_goal"], 10000);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_health_insights_incomplete_profile() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let (status, response) = app.get_auth("/api/v1/profile/insights", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    // Should have missing fields since profile is incomplete
    let missing = response["missing_fields"].as_array().unwrap();
    assert!(!missing.is_empty());
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_health_insights_complete_profile() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    // Complete the profile
    let profile_body = json!({
        "height": 175.0,
        "height_unit": "cm",
        "date_of_birth": "1990-01-15",
        "biological_sex": "male",
        "activity_level": "moderately_active"
    });
    app.put_auth("/api/v1/profile", &profile_body.to_string(), &token).await;
    
    // Log a weight
    let weight_body = json!({ "weight": 75.0 });
    app.post_auth("/api/v1/weight", &weight_body.to_string(), &token).await;
    
    let (status, response) = app.get_auth("/api/v1/profile/insights", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    
    // Should have BMI calculated
    assert!(response["bmi"].is_object());
    let bmi = &response["bmi"];
    assert!(bmi["value"].as_f64().unwrap() > 0.0);
    
    // Should have energy info
    assert!(response["energy"].is_object());
    let energy = &response["energy"];
    assert!(energy["bmr"].as_f64().unwrap() > 0.0);
    assert!(energy["tdee"].as_f64().unwrap() > 0.0);
    
    // Should have no missing fields
    let missing = response["missing_fields"].as_array().unwrap();
    assert!(missing.is_empty());
}
