//! Health check endpoints
//!
//! Provides Kubernetes-compatible health check endpoints:
//! - /health - Basic health check
//! - /health/ready - Readiness probe (checks dependencies)
//! - /health/live - Liveness probe (always returns OK if server is running)

use crate::{db, state::AppState};
use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<HealthChecks>,
}

/// Individual health checks
#[derive(Serialize)]
pub struct HealthChecks {
    pub database: CheckStatus,
}

/// Status of an individual check
#[derive(Serialize)]
pub struct CheckStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Basic health check endpoint
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: None,
    })
}

/// Readiness probe - checks if the service is ready to accept traffic
/// Returns 503 if any dependency is unhealthy
pub async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, (StatusCode, Json<HealthResponse>)> {
    let db_check = match db::health_check(&state.db).await {
        Ok(_) => CheckStatus {
            status: "healthy".to_string(),
            message: None,
        },
        Err(e) => CheckStatus {
            status: "unhealthy".to_string(),
            message: Some(e.to_string()),
        },
    };

    let is_healthy = db_check.status == "healthy";

    let response = HealthResponse {
        status: if is_healthy { "ready" } else { "not_ready" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: Some(HealthChecks { database: db_check }),
    };

    if is_healthy {
        Ok(Json(response))
    } else {
        Err((StatusCode::SERVICE_UNAVAILABLE, Json(response)))
    }
}

/// Liveness probe - checks if the service is alive
/// Always returns OK if the server is running
pub async fn liveness_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "alive".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_healthy() {
        let response = health_check().await;
        assert_eq!(response.status, "healthy");
        assert!(!response.version.is_empty());
    }

    #[tokio::test]
    async fn test_liveness_check_returns_alive() {
        let response = liveness_check().await;
        assert_eq!(response.status, "alive");
    }
}
