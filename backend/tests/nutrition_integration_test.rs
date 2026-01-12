//! Integration tests for nutrition tracking endpoints

mod common;

use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
#[ignore = "requires database"]
async fn test_search_food_requires_auth() {
    let app = common::TestApp::new().await;
    
    let (status, _) = app.get("/api/v1/nutrition/search?q=apple").await;
    
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_search_food_empty_results() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    // Search for something that doesn't exist
    let (status, response) = app.get_auth("/api/v1/nutrition/search?q=xyznonexistent", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert!(response.as_array().unwrap().is_empty());
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_barcode_lookup_not_found() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let (status, _) = app.get_auth("/api/v1/nutrition/barcode/0000000000000", &token).await;
    
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_daily_summary_empty() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let (status, response) = app.get_auth("/api/v1/nutrition/daily/2024-12-29", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["total_calories"], 0.0);
    assert_eq!(response["meal_count"], 0);
    assert!(response["logs"].as_array().unwrap().is_empty());
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_create_recipe() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let body = json!({
        "name": "Test Recipe",
        "description": "A test recipe",
        "servings": 4.0,
        "is_public": false,
        "ingredients": []
    });
    
    let (status, response) = app.post_auth("/api/v1/nutrition/recipes", &body.to_string(), &token).await;
    
    assert_eq!(status, StatusCode::CREATED);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["recipe"]["name"], "Test Recipe");
    assert_eq!(response["recipe"]["servings"], 4.0);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_recipes_empty() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let (status, response) = app.get_auth("/api/v1/nutrition/recipes", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert!(response.as_array().unwrap().is_empty());
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_recipes_with_entries() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    // Create a recipe
    let body = json!({
        "name": "My Recipe",
        "servings": 2.0,
        "ingredients": []
    });
    app.post_auth("/api/v1/nutrition/recipes", &body.to_string(), &token).await;
    
    let (status, response) = app.get_auth("/api/v1/nutrition/recipes", &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response.as_array().unwrap().len(), 1);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_recipe_by_id() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    // Create a recipe
    let body = json!({
        "name": "Detailed Recipe",
        "description": "With description",
        "servings": 3.0,
        "ingredients": []
    });
    let (_, create_response) = app.post_auth("/api/v1/nutrition/recipes", &body.to_string(), &token).await;
    let create_response: serde_json::Value = serde_json::from_str(&create_response).unwrap();
    let recipe_id = create_response["recipe"]["id"].as_str().unwrap();
    
    // Get the recipe by ID
    let (status, response) = app.get_auth(&format!("/api/v1/nutrition/recipes/{}", recipe_id), &token).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(response["recipe"]["name"], "Detailed Recipe");
    assert_eq!(response["recipe"]["servings"], 3.0);
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_recipe_not_found() {
    let app = common::TestApp::new().await;
    let user = app.create_test_user().await;
    let token = user.tokens.as_ref().unwrap().access_token.clone();
    
    let fake_id = "00000000-0000-0000-0000-000000000000";
    let (status, _) = app.get_auth(&format!("/api/v1/nutrition/recipes/{}", fake_id), &token).await;
    
    assert_eq!(status, StatusCode::NOT_FOUND);
}
