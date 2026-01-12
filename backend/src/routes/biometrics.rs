//! Biometrics (Heart Rate & HRV) API routes

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::services::biometrics::{BiometricsService, LogHeartRateInput, LogHrvInput};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use fitness_assistant_shared::types::{
    BiometricsHistoryQuery, HeartRateLogResponse, HeartRateZoneResponse,
    HeartRateZonesResponse, HrvLogResponse, LogHeartRateRequest, LogHrvRequest,
    RecoveryScoreResponse, RestingHrAnalysisResponse,
};

/// Create biometrics routes
pub fn biometrics_routes() -> Router<AppState> {
    Router::new()
        .route("/heart-rate", post(log_heart_rate))
        .route("/heart-rate/history", get(get_heart_rate_history))
        .route("/heart-rate/analysis", get(get_resting_hr_analysis))
        .route("/hrv", post(log_hrv))
        .route("/hrv/history", get(get_hrv_history))
        .route("/recovery", get(get_recovery_score))
        .route("/zones", get(get_heart_rate_zones))
        .route("/heart-rate/:id", axum::routing::delete(delete_heart_rate))
        .route("/hrv/:id", axum::routing::delete(delete_hrv))
}

/// POST /api/v1/biometrics/heart-rate - Log heart rate
async fn log_heart_rate(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LogHeartRateRequest>,
) -> Result<Json<HeartRateLogResponse>, ApiError> {
    let workout_id = req.workout_id
        .as_ref()
        .map(|id| uuid::Uuid::parse_str(id))
        .transpose()
        .map_err(|_| ApiError::Validation("Invalid workout ID".to_string()))?;

    let input = LogHeartRateInput {
        bpm: req.bpm,
        context: req.context,
        recorded_at: req.recorded_at,
        workout_id,
        source: req.source,
        notes: req.notes,
    };

    let log = BiometricsService::log_heart_rate(state.db(), auth.user_id, input).await?;

    Ok(Json(HeartRateLogResponse {
        id: log.id.to_string(),
        bpm: log.bpm,
        context: log.context,
        recorded_at: log.recorded_at,
        workout_id: log.workout_id.map(|id| id.to_string()),
        source: log.source,
        notes: log.notes,
    }))
}

/// GET /api/v1/biometrics/heart-rate/history - Get heart rate history
async fn get_heart_rate_history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<BiometricsHistoryQuery>,
) -> Result<Json<fitness_assistant_shared::types::HeartRateHistoryResponse>, ApiError> {
    let query = query.normalize();
    
    let records = crate::repositories::biometrics::HeartRateLogRepository::get_history(
        state.db(),
        auth.user_id,
        query.start_date,
        query.end_date,
        query.context.as_deref(),
        query.limit,
        query.offset,
    )
    .await
    .map_err(ApiError::Internal)?;

    let items: Vec<HeartRateLogResponse> = records
        .into_iter()
        .map(|r| HeartRateLogResponse {
            id: r.id.to_string(),
            bpm: r.bpm,
            context: r.context,
            recorded_at: r.recorded_at,
            workout_id: r.workout_id.map(|id| id.to_string()),
            source: r.source,
            notes: r.notes,
        })
        .collect();

    let total_count = items.len() as i64; // Simplified - would need count query
    let has_more = items.len() as i64 >= query.limit;

    Ok(Json(fitness_assistant_shared::types::HeartRateHistoryResponse {
        items,
        total_count,
        limit: query.limit,
        offset: query.offset,
        has_more,
    }))
}

/// GET /api/v1/biometrics/heart-rate/analysis - Get resting HR analysis
async fn get_resting_hr_analysis(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<RestingHrAnalysisResponse>, ApiError> {
    let analysis = BiometricsService::analyze_resting_hr(state.db(), auth.user_id, 7).await?;

    Ok(Json(RestingHrAnalysisResponse {
        current_avg: analysis.current_avg,
        baseline_avg: analysis.baseline_avg,
        deviation_percent: analysis.deviation_percent,
        is_anomaly: analysis.is_anomaly,
        trend: analysis.trend,
    }))
}

/// POST /api/v1/biometrics/hrv - Log HRV
async fn log_hrv(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LogHrvRequest>,
) -> Result<Json<HrvLogResponse>, ApiError> {
    let input = LogHrvInput {
        rmssd: req.rmssd,
        sdnn: req.sdnn,
        context: req.context,
        recorded_at: req.recorded_at,
        source: req.source,
        notes: req.notes,
    };

    let log = BiometricsService::log_hrv(state.db(), auth.user_id, input).await?;

    Ok(Json(HrvLogResponse {
        id: log.id.to_string(),
        rmssd: log.rmssd,
        sdnn: log.sdnn,
        context: log.context,
        recorded_at: log.recorded_at,
        source: log.source,
        notes: log.notes,
    }))
}

/// GET /api/v1/biometrics/hrv/history - Get HRV history
async fn get_hrv_history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<BiometricsHistoryQuery>,
) -> Result<Json<fitness_assistant_shared::types::HrvHistoryResponse>, ApiError> {
    let query = query.normalize();
    
    use rust_decimal::prelude::ToPrimitive;
    
    let records = crate::repositories::biometrics::HrvLogRepository::get_history(
        state.db(),
        auth.user_id,
        query.start_date,
        query.end_date,
        query.limit,
        query.offset,
    )
    .await
    .map_err(ApiError::Internal)?;

    let items: Vec<HrvLogResponse> = records
        .into_iter()
        .map(|r| HrvLogResponse {
            id: r.id.to_string(),
            rmssd: r.rmssd.to_f64().unwrap_or(0.0),
            sdnn: r.sdnn.and_then(|d| d.to_f64()),
            context: r.context,
            recorded_at: r.recorded_at,
            source: r.source,
            notes: r.notes,
        })
        .collect();

    let total_count = items.len() as i64;
    let has_more = items.len() as i64 >= query.limit;

    Ok(Json(fitness_assistant_shared::types::HrvHistoryResponse {
        items,
        total_count,
        limit: query.limit,
        offset: query.offset,
        has_more,
    }))
}

/// GET /api/v1/biometrics/recovery - Get recovery score
async fn get_recovery_score(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<RecoveryScoreResponse>, ApiError> {
    let recovery = BiometricsService::get_recovery_score(state.db(), auth.user_id).await?;

    Ok(Json(RecoveryScoreResponse {
        score: recovery.score,
        hrv_current: recovery.hrv_current,
        hrv_baseline: recovery.hrv_baseline,
        resting_hr_current: recovery.resting_hr_current,
        resting_hr_baseline: recovery.resting_hr_baseline,
        status: recovery.status,
    }))
}

/// GET /api/v1/biometrics/zones - Get heart rate zones
async fn get_heart_rate_zones(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<HeartRateZonesResponse>, ApiError> {
    let zones = BiometricsService::get_heart_rate_zones(state.db(), auth.user_id).await?;

    Ok(Json(HeartRateZonesResponse {
        max_heart_rate: zones.max_heart_rate,
        resting_heart_rate: zones.resting_heart_rate,
        zones: zones.zones.into_iter().map(|z| HeartRateZoneResponse {
            zone: z.zone,
            name: z.name,
            min_bpm: z.min_bpm,
            max_bpm: z.max_bpm,
        }).collect(),
        calculation_method: zones.calculation_method,
    }))
}

/// DELETE /api/v1/biometrics/heart-rate/:id - Delete heart rate log
async fn delete_heart_rate(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let log_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid log ID".to_string()))?;

    let deleted = crate::repositories::biometrics::HeartRateLogRepository::delete(
        state.db(), log_id, auth.user_id
    )
    .await
    .map_err(ApiError::Internal)?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err(ApiError::NotFound("Heart rate log not found".to_string()))
    }
}

/// DELETE /api/v1/biometrics/hrv/:id - Delete HRV log
async fn delete_hrv(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let log_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid log ID".to_string()))?;

    let deleted = crate::repositories::biometrics::HrvLogRepository::delete(
        state.db(), log_id, auth.user_id
    )
    .await
    .map_err(ApiError::Internal)?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err(ApiError::NotFound("HRV log not found".to_string()))
    }
}
