//! Hydration tracking API routes

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::services::hydration::{HydrationService, LogHydrationInput, SetHydrationGoalInput};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::NaiveDate;
use fitness_assistant_shared::types::{
    DailyHydrationResponse, DailyHydrationSummaryResponse, HydrationGoalResponse,
    HydrationHistoryQuery, HydrationHistoryResponse, HydrationLogResponse, LogHydrationRequest,
    SetHydrationGoalRequest,
};

/// Create hydration routes
pub fn hydration_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(log_hydration))
        .route("/daily/:date", get(get_daily_summary))
        .route("/goal", get(get_goal).post(set_goal))
        .route("/history", get(get_history))
        .route("/:id", axum::routing::delete(delete_log))
}

/// POST /api/v1/hydration - Log water intake
async fn log_hydration(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LogHydrationRequest>,
) -> Result<Json<HydrationLogResponse>, ApiError> {
    let input = LogHydrationInput {
        amount_ml: req.amount_ml,
        beverage_type: req.beverage_type,
        consumed_at: req.consumed_at,
        source: req.source,
        notes: req.notes,
    };

    let log = HydrationService::log_hydration(state.db(), auth.user_id, input).await?;

    Ok(Json(HydrationLogResponse {
        id: log.id.to_string(),
        amount_ml: log.amount_ml,
        beverage_type: log.beverage_type,
        consumed_at: log.consumed_at,
        source: log.source,
        notes: log.notes,
    }))
}

/// GET /api/v1/hydration/daily/:date - Get daily hydration summary
async fn get_daily_summary(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(date): Path<String>,
) -> Result<Json<DailyHydrationResponse>, ApiError> {
    let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|_| ApiError::Validation("Invalid date format. Use YYYY-MM-DD".to_string()))?;

    let summary = HydrationService::get_daily_summary(state.db(), auth.user_id, date).await?;

    Ok(Json(DailyHydrationResponse {
        date: summary.date,
        total_ml: summary.total_ml,
        goal_ml: summary.goal_ml,
        progress_percent: summary.progress_percent,
        goal_met: summary.goal_met,
        entry_count: summary.entry_count,
        entries: summary
            .entries
            .into_iter()
            .map(|e| HydrationLogResponse {
                id: e.id.to_string(),
                amount_ml: e.amount_ml,
                beverage_type: e.beverage_type,
                consumed_at: e.consumed_at,
                source: e.source,
                notes: e.notes,
            })
            .collect(),
    }))
}

/// GET /api/v1/hydration/goal - Get hydration goal
async fn get_goal(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<HydrationGoalResponse>, ApiError> {
    let goal = HydrationService::get_goal(state.db(), auth.user_id).await?;

    Ok(Json(HydrationGoalResponse {
        daily_goal_ml: goal.daily_goal_ml,
        is_auto_calculated: goal.is_auto_calculated,
        reminders_enabled: goal.reminders_enabled,
        reminder_interval_minutes: goal.reminder_interval_minutes,
        reminder_start_time: goal.reminder_start_time.map(|t| t.format("%H:%M").to_string()),
        reminder_end_time: goal.reminder_end_time.map(|t| t.format("%H:%M").to_string()),
    }))
}

/// POST /api/v1/hydration/goal - Set hydration goal
async fn set_goal(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<SetHydrationGoalRequest>,
) -> Result<Json<HydrationGoalResponse>, ApiError> {
    // Parse time strings if provided
    let reminder_start_time = req
        .reminder_start_time
        .as_ref()
        .map(|s| {
            chrono::NaiveTime::parse_from_str(s, "%H:%M")
                .map_err(|_| ApiError::Validation("Invalid start time format. Use HH:MM".to_string()))
        })
        .transpose()?;

    let reminder_end_time = req
        .reminder_end_time
        .as_ref()
        .map(|s| {
            chrono::NaiveTime::parse_from_str(s, "%H:%M")
                .map_err(|_| ApiError::Validation("Invalid end time format. Use HH:MM".to_string()))
        })
        .transpose()?;

    let input = SetHydrationGoalInput {
        daily_goal_ml: req.daily_goal_ml,
        auto_calculate: req.auto_calculate,
        reminders_enabled: req.reminders_enabled,
        reminder_interval_minutes: req.reminder_interval_minutes,
        reminder_start_time,
        reminder_end_time,
    };

    let goal = HydrationService::set_goal(state.db(), auth.user_id, input).await?;

    Ok(Json(HydrationGoalResponse {
        daily_goal_ml: goal.daily_goal_ml,
        is_auto_calculated: goal.is_auto_calculated,
        reminders_enabled: goal.reminders_enabled,
        reminder_interval_minutes: goal.reminder_interval_minutes,
        reminder_start_time: goal.reminder_start_time.map(|t| t.format("%H:%M").to_string()),
        reminder_end_time: goal.reminder_end_time.map(|t| t.format("%H:%M").to_string()),
    }))
}

/// GET /api/v1/hydration/history - Get hydration history
async fn get_history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<HydrationHistoryQuery>,
) -> Result<Json<HydrationHistoryResponse>, ApiError> {
    let summaries =
        HydrationService::get_history(state.db(), auth.user_id, query.start_date, query.end_date)
            .await?;

    Ok(Json(HydrationHistoryResponse {
        summaries: summaries
            .into_iter()
            .map(|s| DailyHydrationSummaryResponse {
                date: s.date,
                total_ml: s.total_ml,
                goal_ml: s.goal_ml,
                progress_percent: s.progress_percent,
                goal_met: s.goal_met,
                entry_count: s.entry_count,
            })
            .collect(),
    }))
}

/// DELETE /api/v1/hydration/:id - Delete a hydration log entry
async fn delete_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let log_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid log ID".to_string()))?;

    let deleted = HydrationService::delete_log(state.db(), auth.user_id, log_id).await?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err(ApiError::NotFound("Hydration log not found".to_string()))
    }
}
