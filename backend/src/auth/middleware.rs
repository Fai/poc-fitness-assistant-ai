//! Authentication middleware
//!
//! Provides Axum middleware for JWT validation and user extraction.
//!
//! # Performance
//! 
//! Uses pre-computed JWT keys from AppState to avoid expensive
//! key derivation on every request.

use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::FromRef,
    http::{header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Authenticated user extracted from JWT
/// 
/// This extractor validates the JWT token and extracts the user ID.
/// It uses the pre-computed JWT keys from AppState for efficiency.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| ApiError::Unauthorized("Missing authorization header".to_string()))?;

        // Check Bearer prefix
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::Unauthorized("Invalid authorization format".to_string()))?;

        // Use pre-computed JWT service from state (no allocation!)
        let claims = app_state
            .jwt()
            .validate_access_token(token)
            .map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))?;

        // Parse user ID from claims
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| ApiError::Unauthorized("Invalid user ID in token".to_string()))?;

        Ok(AuthUser { user_id })
    }
}

/// Middleware function for authentication (alternative to extractor)
/// 
/// Use this when you need to apply auth to a group of routes via layer.
#[allow(dead_code)]
pub async fn auth_middleware(
    state: AppState,
    mut request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing authorization header".to_string()))?;

    // Check Bearer prefix
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Unauthorized("Invalid authorization format".to_string()))?;

    // Use pre-computed JWT service (no allocation!)
    let claims = state
        .jwt()
        .validate_access_token(token)
        .map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))?;

    // Parse user ID and add to request extensions
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID in token".to_string()))?;

    request.extensions_mut().insert(AuthUser { user_id });

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_user_debug() {
        let user = AuthUser {
            user_id: Uuid::new_v4(),
        };
        let debug_str = format!("{:?}", user);
        assert!(debug_str.contains("AuthUser"));
    }
}
