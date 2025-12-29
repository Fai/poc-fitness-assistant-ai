//! Application error handling
//!
//! This module provides unified error handling for the API,
//! converting internal errors to appropriate HTTP responses.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

/// API error type that can be converted to HTTP responses
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

/// Error response body
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

/// Error detail
#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone()),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.clone()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            ApiError::Internal(err) => {
                error!("Internal error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "An internal error occurred".to_string(),
                )
            }
            ApiError::Database(err) => {
                error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "A database error occurred".to_string(),
                )
            }
        };

        let body = Json(ErrorResponse {
            error: ErrorDetail {
                code: code.to_string(),
                message,
                field: None,
            },
        });

        (status, body).into_response()
    }
}

/// Result type alias for API handlers
pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_status() {
        let error = ApiError::Validation("Invalid input".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_not_found_error_status() {
        let error = ApiError::NotFound("User not found".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_unauthorized_error_status() {
        let error = ApiError::Unauthorized("Invalid token".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
