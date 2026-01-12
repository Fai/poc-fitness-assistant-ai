//! Sleep tracking API routes

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::services::sleep::{LogSleepInput, SetSleepGoalInput, SleepService};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use fitness_assistant_shared::types::{
    LogSleepRequest, SetSleepGoalRequest, SleepAnalysisQuery, SleepAnalysisResponse,
    SleepGoalResponse, SleepHistoryQuery, SleepHistoryResponse, SleepLogResponse,
};

/// Create sleep routes
pub fn sleep_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(log_sleep))
        .route("/history", get(get_history))
        .route("/analysis", get(get_analysis))
        .route("/goal", get(get_goal).post(set_goal))
        .route("/:id", axum::routing::delete(delete_log))
}

/// POST /api/v1/sleep - Log sleep entry
async fn log_sleep(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LogSleepRequest>,
) -> Result<Json<SleepLogResponse>, ApiError> {
    let input = LogSleepInput {
        sleep_start: req.sleep_start,
        sleep_end: req.sleep_end,
        awake_minutes: req.awake_minutes,
        light_minutes: req.light_minutes,
        deep_minutes: req.deep_minutes,
        rem_minutes: req.rem_minutes,
        sleep_score: req.sleep_score,
        times_awoken: req.times_awoken,
        avg_heart_rate: req.avg_heart_rate,
        min_heart_rate: req.min_heart_rate,
        hrv_average: req.hrv_average,
        respiratory_rate: req.respiratory_rate,
        source: req.source,
        notes: req.notes,
    };

    let log = SleepService::log_sleep(state.db(), auth.user_id, input).await?;

    Ok(Json(SleepLogResponse {
        id: log.id.to_string(),
        sleep_start: log.sleep_start,
        sleep_end: log.sleep_end,
        total_duration_minutes: log.total_duration_minutes,
        awake_minutes: log.awake_minutes,
        light_minutes: log.light_minutes,
        deep_minutes: log.deep_minutes,
        rem_minutes: log.rem_minutes,
        sleep_efficiency: log.sleep_efficiency,
        sleep_score: log.sleep_score,
        times_awoken: log.times_awoken,
        avg_heart_rate: log.avg_heart_rate,
        min_heart_rate: log.min_heart_rate,
        hrv_average: log.hrv_average,
        respiratory_rate: log.respiratory_rate,
        source: log.source,
        notes: log.notes,
    }))
}

/// GET /api/v1/sleep/history - Get sleep history
async fn get_history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<SleepHistoryQuery>,
) -> Result<Json<SleepHistoryResponse>, ApiError> {
    let query = query.normalize();
    
    let (logs, total) = SleepService::get_history(
        state.db(),
        auth.user_id,
        query.start_date,
        query.end_date,
        query.limit,
        query.offset,
    )
    .await?;

    let has_more = query.offset + (logs.len() as i64) < total;

    Ok(Json(SleepHistoryResponse {
        items: logs
            .into_iter()
            .map(|log| SleepLogResponse {
                id: log.id.to_string(),
                sleep_start: log.sleep_start,
                sleep_end: log.sleep_end,
                total_duration_minutes: log.total_duration_minutes,
                awake_minutes: log.awake_minutes,
                light_minutes: log.light_minutes,
                deep_minutes: log.deep_minutes,
                rem_minutes: log.rem_minutes,
                sleep_efficiency: log.sleep_efficiency,
                sleep_score: log.sleep_score,
                times_awoken: log.times_awoken,
                avg_heart_rate: log.avg_heart_rate,
                min_heart_rate: log.min_heart_rate,
                hrv_average: log.hrv_average,
                respiratory_rate: log.respiratory_rate,
                source: log.source,
                notes: log.notes,
            })
            .collect(),
        total_count: total,
        limit: query.limit,
        offset: query.offset,
        has_more,
    }))
}

/// GET /api/v1/sleep/analysis - Get sleep analysis
async fn get_analysis(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<SleepAnalysisQuery>,
) -> Result<Json<SleepAnalysisResponse>, ApiError> {
    let analysis = SleepService::get_analysis(
        state.db(),
        auth.user_id,
        query.start_date,
        query.end_date,
    )
    .await?;

    Ok(Json(SleepAnalysisResponse {
        avg_duration_minutes: analysis.avg_duration_minutes,
        avg_efficiency: analysis.avg_efficiency,
        avg_deep_percent: analysis.avg_deep_percent,
        avg_rem_percent: analysis.avg_rem_percent,
        avg_light_percent: analysis.avg_light_percent,
        avg_awake_percent: analysis.avg_awake_percent,
        total_nights: analysis.total_nights,
        sleep_debt_minutes: analysis.sleep_debt_minutes,
        consistency_score: analysis.consistency_score,
    }))
}

/// GET /api/v1/sleep/goal - Get sleep goal
async fn get_goal(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<SleepGoalResponse>, ApiError> {
    let goal = SleepService::get_goal(state.db(), auth.user_id).await?;

    Ok(Json(SleepGoalResponse {
        target_duration_minutes: goal.target_duration_minutes,
        target_bedtime: goal.target_bedtime.map(|t| t.format("%H:%M").to_string()),
        target_wake_time: goal.target_wake_time.map(|t| t.format("%H:%M").to_string()),
        bedtime_reminder_enabled: goal.bedtime_reminder_enabled,
        bedtime_reminder_minutes_before: goal.bedtime_reminder_minutes_before,
    }))
}

/// POST /api/v1/sleep/goal - Set sleep goal
async fn set_goal(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<SetSleepGoalRequest>,
) -> Result<Json<SleepGoalResponse>, ApiError> {
    // Parse time strings if provided
    let target_bedtime = req
        .target_bedtime
        .as_ref()
        .map(|s| {
            chrono::NaiveTime::parse_from_str(s, "%H:%M")
                .map_err(|_| ApiError::Validation("Invalid bedtime format. Use HH:MM".to_string()))
        })
        .transpose()?;

    let target_wake_time = req
        .target_wake_time
        .as_ref()
        .map(|s| {
            chrono::NaiveTime::parse_from_str(s, "%H:%M")
                .map_err(|_| ApiError::Validation("Invalid wake time format. Use HH:MM".to_string()))
        })
        .transpose()?;

    let input = SetSleepGoalInput {
        target_duration_minutes: req.target_duration_minutes,
        target_bedtime,
        target_wake_time,
        bedtime_reminder_enabled: req.bedtime_reminder_enabled,
        bedtime_reminder_minutes_before: req.bedtime_reminder_minutes_before,
    };

    let goal = SleepService::set_goal(state.db(), auth.user_id, input).await?;

    Ok(Json(SleepGoalResponse {
        target_duration_minutes: goal.target_duration_minutes,
        target_bedtime: goal.target_bedtime.map(|t| t.format("%H:%M").to_string()),
        target_wake_time: goal.target_wake_time.map(|t| t.format("%H:%M").to_string()),
        bedtime_reminder_enabled: goal.bedtime_reminder_enabled,
        bedtime_reminder_minutes_before: goal.bedtime_reminder_minutes_before,
    }))
}

/// DELETE /api/v1/sleep/:id - Delete a sleep log entry
async fn delete_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let log_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid log ID".to_string()))?;

    let deleted = SleepService::delete_log(state.db(), auth.user_id, log_id).await?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err(ApiError::NotFound("Sleep log not found".to_string()))
    }
}
