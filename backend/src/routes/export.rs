//! Data export API routes

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::services::export::ExportService;
use crate::state::AppState;
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::get,
    Json, Router,
};

/// Create export routes
pub fn export_routes() -> Router<AppState> {
    Router::new()
        .route("/json", get(export_json))
        .route("/csv/weight", get(export_weight_csv))
        .route("/csv/sleep", get(export_sleep_csv))
}

/// GET /api/v1/export/json - Export all user data as JSON
async fn export_json(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let export = ExportService::export_json(state.db(), auth.user_id).await?;
    
    let json = serde_json::to_string_pretty(&export)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("JSON serialization error: {}", e)))?;
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"fitness-data-export.json\""),
    );
    
    Ok((headers, json))
}

/// GET /api/v1/export/csv/weight - Export weight data as CSV
async fn export_weight_csv(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let csv = ExportService::export_weight_csv(state.db(), auth.user_id).await?;
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"weight-export.csv\""),
    );
    
    Ok((headers, csv))
}

/// GET /api/v1/export/csv/sleep - Export sleep data as CSV
async fn export_sleep_csv(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<impl IntoResponse, ApiError> {
    let csv = ExportService::export_sleep_csv(state.db(), auth.user_id).await?;
    
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"sleep-export.csv\""),
    );
    
    Ok((headers, csv))
}
