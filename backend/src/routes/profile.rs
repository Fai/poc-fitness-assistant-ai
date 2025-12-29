//! User profile and settings API routes

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::services::{HealthInsightsService, ProfileService};
use crate::state::AppState;
use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use fitness_assistant_shared::types::{
    HealthInsightsResponse, UpdateProfileRequest, UpdateSettingsRequest,
    UserProfileResponse, UserSettingsResponse,
};

/// Create profile routes
pub fn profile_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_profile).put(update_profile))
        .route("/settings", get(get_settings).put(update_settings))
        .route("/insights", get(get_health_insights))
}

/// GET /api/v1/profile - Get user profile
async fn get_profile(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let profile = ProfileService::get_profile(state.db(), auth.user_id).await?;
    Ok(Json(profile))
}

/// PUT /api/v1/profile - Update user profile
/// 
/// Validates input before updating:
/// - Height must be between 50-300 cm
/// - Date of birth cannot be in the future, age must be 1-150 years
/// - Biological sex must be "male" or "female"
/// - Activity level must be one of: sedentary, lightly_active, moderately_active, very_active, extra_active
async fn update_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let profile = ProfileService::update_profile(state.db(), auth.user_id, req).await?;
    Ok(Json(profile))
}

/// GET /api/v1/profile/settings - Get user settings
async fn get_settings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserSettingsResponse>, ApiError> {
    let settings = ProfileService::get_settings(state.db(), auth.user_id).await?;
    Ok(Json(settings))
}

/// PUT /api/v1/profile/settings - Update user settings
async fn update_settings(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<UserSettingsResponse>, ApiError> {
    let settings = ProfileService::update_settings(state.db(), auth.user_id, req).await?;
    Ok(Json(settings))
}

/// GET /api/v1/profile/insights - Get health insights
/// 
/// Returns BMI, TDEE, hydration recommendations, and ideal weight based on
/// user profile data. Missing fields are reported with user-friendly labels
/// so user can complete their profile.
async fn get_health_insights(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<HealthInsightsResponse>, ApiError> {
    let insights = HealthInsightsService::get_insights(state.db(), auth.user_id).await?;
    Ok(Json(insights))
}
