//! Common test utilities for integration tests
//!
//! This module provides shared setup and teardown for integration tests.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use fitness_assistant_backend::{config::AppConfig, routes, state::AppState};
use serde::Deserialize;
use sqlx::PgPool;
use tower::ServiceExt;

/// Test application wrapper
pub struct TestApp {
    pub app: Router,
    pub pool: PgPool,
    pub state: AppState,
}

/// Authentication tokens for testing
#[derive(Debug, Clone, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Test user credentials
pub struct TestUser {
    pub email: String,
    pub password: String,
    pub tokens: Option<AuthTokens>,
}

impl TestApp {
    /// Create a new test application with a real database
    pub async fn new() -> Self {
        let config = test_config();
        let pool = create_test_pool(&config.database.url).await;
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        let state = AppState::new(pool.clone(), None, config);
        let app = routes::create_router(state.clone());

        Self { app, pool, state }
    }

    /// Make a GET request
    pub async fn get(&self, path: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::empty())
            .unwrap();

        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        (status, body_str)
    }

    /// Make an authenticated GET request
    pub async fn get_auth(&self, path: &str, token: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .method("GET")
            .uri(path)
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        (status, body_str)
    }

    /// Make a POST request with JSON body
    pub async fn post(&self, path: &str, body: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .method("POST")
            .uri(path)
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        (status, body_str)
    }

    /// Make an authenticated POST request with JSON body
    pub async fn post_auth(&self, path: &str, body: &str, token: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .method("POST")
            .uri(path)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        (status, body_str)
    }

    /// Make an authenticated PUT request with JSON body
    pub async fn put_auth(&self, path: &str, body: &str, token: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .method("PUT")
            .uri(path)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        (status, body_str)
    }

    /// Register a new test user and return tokens
    pub async fn register_user(&self, email: &str, password: &str) -> Result<AuthTokens, String> {
        let body = format!(r#"{{"email":"{}","password":"{}"}}"#, email, password);
        let (status, response) = self.post("/api/v1/auth/register", &body).await;
        
        if status != StatusCode::CREATED {
            return Err(format!("Registration failed: {} - {}", status, response));
        }
        
        serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse tokens: {}", e))
    }

    /// Login and return tokens
    pub async fn login(&self, email: &str, password: &str) -> Result<AuthTokens, String> {
        let body = format!(r#"{{"email":"{}","password":"{}"}}"#, email, password);
        let (status, response) = self.post("/api/v1/auth/login", &body).await;
        
        if status != StatusCode::OK {
            return Err(format!("Login failed: {} - {}", status, response));
        }
        
        serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse tokens: {}", e))
    }

    /// Create a test user and return their tokens
    pub async fn create_test_user(&self) -> TestUser {
        let email = format!("test_{}@example.com", uuid::Uuid::new_v4());
        let password = "TestPassword123!";
        
        let tokens = self.register_user(&email, password).await.ok();
        
        TestUser {
            email,
            password: password.to_string(),
            tokens,
        }
    }

    /// Clean up test data
    pub async fn cleanup(&self) {
        // Truncate all tables for clean state between tests
        sqlx::query("TRUNCATE users, user_settings, weight_logs, body_composition_logs, food_logs, food_items, recipes, recipe_ingredients CASCADE")
            .execute(&self.pool)
            .await
            .ok();
    }
}

fn test_config() -> AppConfig {
    AppConfig {
        server: fitness_assistant_backend::config::ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            allowed_origins: vec![], // Allow any in tests
        },
        database: fitness_assistant_backend::config::DatabaseConfig {
            url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/fitness_assistant_test".to_string()),
            max_connections: 5,
        },
        redis: fitness_assistant_backend::config::RedisConfig {
            url: "redis://localhost:6379".to_string(),
        },
        jwt: fitness_assistant_backend::config::JwtConfig {
            secret: "test-secret-key-for-testing-only-32chars".to_string(),
            access_token_expiry_secs: 3600,
            refresh_token_expiry_secs: 86400,
        },
        ai: fitness_assistant_backend::config::AiConfig::default(),
    }
}

async fn create_test_pool(url: &str) -> PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .expect("Failed to create test database pool")
}
