//! Authentication routes
//!
//! Provides endpoints for user registration, login, and token refresh.
//!
//! # Performance Optimizations
//! 
//! - Uses pre-computed JWT keys from AppState (no per-request allocation)
//! - Password hashing runs on blocking thread pool (doesn't block async runtime)

use crate::auth::AuthUser;
use crate::error::ApiResult;
use crate::services::UserService;
use crate::state::AppState;
use axum::{extract::State, routing::post, Json, Router};
use fitness_assistant_shared::types::{AuthTokens, LoginRequest, RegisterRequest, UserProfile};
use serde::Deserialize;

/// Create auth routes
pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/me", axum::routing::get(get_profile))
}

/// Register a new user
/// 
/// POST /api/v1/auth/register
/// 
/// # Performance
/// Password hashing is offloaded to blocking thread pool.
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> ApiResult<Json<AuthTokens>> {
    // Use pre-computed JWT service from state
    let tokens = UserService::register(&state.db, state.jwt(), &req.email, &req.password).await?;
    Ok(Json(tokens))
}

/// Login with email and password
/// 
/// POST /api/v1/auth/login
/// 
/// # Performance
/// Password verification is offloaded to blocking thread pool.
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<AuthTokens>> {
    let tokens = UserService::login(&state.db, state.jwt(), &req.email, &req.password).await?;
    Ok(Json(tokens))
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Refresh access token
/// 
/// POST /api/v1/auth/refresh
async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> ApiResult<Json<AuthTokens>> {
    let tokens = UserService::refresh_token(&state.db, state.jwt(), &req.refresh_token).await?;
    Ok(Json(tokens))
}

/// Get current user profile (requires authentication)
/// 
/// GET /api/v1/auth/me
/// 
/// # Authentication
/// Requires valid Bearer token in Authorization header.
async fn get_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> ApiResult<Json<UserProfile>> {
    let profile = UserService::get_profile(&state.db, auth_user.user_id).await?;
    Ok(Json(profile))
}

#[cfg(test)]
mod tests {
    // Route tests will be added as integration tests
}
