//! Common test utilities for integration tests
//!
//! This module provides shared setup and teardown for integration tests.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use fitness_assistant_backend::{config::AppConfig, routes, state::AppState};
use sqlx::PgPool;
use tower::ServiceExt;

/// Test application wrapper
pub struct TestApp {
    pub app: Router,
    pub pool: PgPool,
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

        let state = AppState::new(pool.clone(), config);
        let app = routes::create_router(state);

        Self { app, pool }
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

    /// Clean up test data
    pub async fn cleanup(&self) {
        // Truncate all tables for clean state between tests
        sqlx::query("TRUNCATE users, user_settings CASCADE")
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
