//! Weight and body composition API routes

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::repositories::UserRepository;
use crate::services::weight::{BodyCompositionInput, WeightEntryInput, WeightService};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use fitness_assistant_shared::types::{
    BodyCompositionResponse, GoalProjectionRequest, GoalProjectionResponse,
    LogBodyCompositionRequest, LogWeightRequest, WeightHistoryQuery, WeightHistoryResponse,
    WeightLogResponse, WeightTrendResponse,
};
use fitness_assistant_shared::units::WeightUnit;

/// Create weight routes
pub fn weight_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(log_weight).get(get_weight_history))
        .route("/trend", get(get_weight_trend))
        .route("/projection", post(project_goal))
        .route("/body-composition", post(log_body_composition).get(get_body_composition_history))
}

/// Parse weight unit from string, defaulting to kg
fn parse_weight_unit(unit_str: Option<&str>) -> WeightUnit {
    unit_str
        .and_then(|s| s.parse::<WeightUnit>().ok())
        .unwrap_or(WeightUnit::Kg)
}

/// Get user's preferred weight unit from settings
async fn get_user_weight_unit(state: &AppState, user_id: uuid::Uuid) -> WeightUnit {
    UserRepository::get_settings(state.db(), user_id)
        .await
        .ok()
        .flatten()
        .and_then(|s| s.weight_unit.parse::<WeightUnit>().ok())
        .unwrap_or(WeightUnit::Kg)
}

/// POST /api/v1/weight - Log a weight entry
/// 
/// Accepts weight in any unit (kg, lbs, stone). If no unit specified,
/// defaults to kg. Stores internally in kg, returns in user's preferred unit.
async fn log_weight(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LogWeightRequest>,
) -> Result<Json<WeightLogResponse>, ApiError> {
    // Parse input unit (defaults to kg if not specified)
    let input_unit = parse_weight_unit(req.unit.as_deref());
    
    // Convert to kg for storage
    let weight_kg = input_unit.to_kg(req.weight);
    
    let input = WeightEntryInput {
        weight_kg,
        recorded_at: req.recorded_at,
        source: req.source,
        notes: req.notes,
    };

    let log = WeightService::log_weight(state.db(), auth.user_id, input).await?;

    // Get user's preferred unit for response
    let preferred_unit = get_user_weight_unit(&state, auth.user_id).await;
    let weight_in_preferred = preferred_unit.from_kg(log.weight_kg);

    Ok(Json(WeightLogResponse {
        id: log.id.to_string(),
        weight: weight_in_preferred,
        unit: preferred_unit.to_string(),
        weight_kg: log.weight_kg,
        recorded_at: log.recorded_at,
        source: log.source,
        notes: log.notes,
        is_anomaly: log.is_anomaly,
    }))
}

/// GET /api/v1/weight - Get weight history with pagination
/// 
/// Returns weight entries in user's preferred unit.
/// Supports pagination with limit (default: 50, max: 100) and offset parameters.
async fn get_weight_history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<WeightHistoryQuery>,
) -> Result<Json<WeightHistoryResponse>, ApiError> {
    // Normalize pagination parameters
    let query = query.normalize();
    
    let (logs, total_count) = WeightService::get_weight_history_paginated(
        state.db(),
        auth.user_id,
        query.start,
        query.end,
        query.limit,
        query.offset,
    )
    .await?;

    // Get user's preferred unit
    let preferred_unit = get_user_weight_unit(&state, auth.user_id).await;

    let items: Vec<WeightLogResponse> = logs
        .into_iter()
        .map(|log| {
            let weight_in_preferred = preferred_unit.from_kg(log.weight_kg);
            WeightLogResponse {
                id: log.id.to_string(),
                weight: weight_in_preferred,
                unit: preferred_unit.to_string(),
                weight_kg: log.weight_kg,
                recorded_at: log.recorded_at,
                source: log.source,
                notes: log.notes,
                is_anomaly: log.is_anomaly,
            }
        })
        .collect();

    let has_more = query.offset + (items.len() as i64) < total_count;

    Ok(Json(WeightHistoryResponse {
        items,
        total_count,
        limit: query.limit,
        offset: query.offset,
        has_more,
    }))
}

/// GET /api/v1/weight/trend - Get weight trend analysis
async fn get_weight_trend(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<WeightHistoryQuery>,
) -> Result<Json<WeightTrendResponse>, ApiError> {
    let trend =
        WeightService::get_weight_trend(state.db(), auth.user_id, query.start, query.end).await?;

    Ok(Json(WeightTrendResponse {
        current_weight: trend.current_weight,
        start_weight: trend.start_weight,
        total_change: trend.total_change,
        average_daily_change: trend.average_daily_change,
        moving_average_7d: trend.moving_average_7d,
        moving_average_30d: trend.moving_average_30d,
        entries_count: trend.entries_count,
    }))
}

/// POST /api/v1/weight/projection - Project goal completion
async fn project_goal(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<GoalProjectionRequest>,
) -> Result<Json<GoalProjectionResponse>, ApiError> {
    let projection =
        WeightService::project_goal(state.db(), auth.user_id, req.target_weight).await?;

    Ok(Json(GoalProjectionResponse {
        target_weight: projection.target_weight,
        current_weight: projection.current_weight,
        weight_to_lose: projection.weight_to_lose,
        average_daily_change: projection.average_daily_change,
        projected_days: projection.projected_days,
        projected_date: projection.projected_date,
        on_track: projection.on_track,
    }))
}

/// POST /api/v1/weight/body-composition - Log body composition
async fn log_body_composition(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LogBodyCompositionRequest>,
) -> Result<Json<BodyCompositionResponse>, ApiError> {
    let input = BodyCompositionInput {
        recorded_at: req.recorded_at,
        body_fat_percent: req.body_fat_percent,
        muscle_mass_kg: req.muscle_mass_kg,
        water_percent: req.water_percent,
        bone_mass_kg: req.bone_mass_kg,
        visceral_fat: req.visceral_fat,
        source: req.source,
    };

    let log = WeightService::log_body_composition(state.db(), auth.user_id, input).await?;

    Ok(Json(BodyCompositionResponse {
        id: log.id.to_string(),
        recorded_at: log.recorded_at,
        body_fat_percent: log.body_fat_percent,
        muscle_mass_kg: log.muscle_mass_kg,
        water_percent: log.water_percent,
        bone_mass_kg: log.bone_mass_kg,
        visceral_fat: log.visceral_fat,
        source: log.source,
    }))
}

/// GET /api/v1/weight/body-composition - Get body composition history
async fn get_body_composition_history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<WeightHistoryQuery>,
) -> Result<Json<Vec<BodyCompositionResponse>>, ApiError> {
    let logs = WeightService::get_body_composition_history(
        state.db(),
        auth.user_id,
        query.start,
        query.end,
    )
    .await?;

    let response: Vec<BodyCompositionResponse> = logs
        .into_iter()
        .map(|log| BodyCompositionResponse {
            id: log.id.to_string(),
            recorded_at: log.recorded_at,
            body_fat_percent: log.body_fat_percent,
            muscle_mass_kg: log.muscle_mass_kg,
            water_percent: log.water_percent,
            bone_mass_kg: log.bone_mass_kg,
            visceral_fat: log.visceral_fat,
            source: log.source,
        })
        .collect();

    Ok(Json(response))
}
