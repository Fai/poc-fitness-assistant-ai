//! Goals API routes

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::services::goals::{CreateGoalInput, GoalsService, UpdateGoalInput};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use fitness_assistant_shared::types::{
    CreateGoalRequest, GoalProgressResponse, GoalResponse, GoalsListQuery, GoalsListResponse,
    MilestoneResponse, UpdateGoalRequest,
};

/// Create goals routes
pub fn goals_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_goal).get(list_goals))
        .route("/:id", get(get_goal).put(update_goal).delete(delete_goal))
        .route("/:id/progress", get(get_progress))
}

/// POST /api/v1/goals - Create a new goal
async fn create_goal(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateGoalRequest>,
) -> Result<Json<GoalResponse>, ApiError> {
    let input = CreateGoalInput {
        name: req.name,
        description: req.description,
        goal_type: req.goal_type,
        metric: req.metric,
        target_value: req.target_value,
        start_value: req.start_value,
        direction: req.direction,
        start_date: req.start_date,
        target_date: req.target_date,
    };

    let goal = GoalsService::create_goal(state.db(), auth.user_id, input).await?;

    Ok(Json(GoalResponse {
        id: goal.id.to_string(),
        name: goal.name,
        description: goal.description,
        goal_type: goal.goal_type,
        metric: goal.metric,
        target_value: goal.target_value,
        start_value: goal.start_value,
        current_value: goal.current_value,
        direction: goal.direction,
        start_date: goal.start_date,
        target_date: goal.target_date,
        status: goal.status,
    }))
}

/// GET /api/v1/goals - List goals
async fn list_goals(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<GoalsListQuery>,
) -> Result<Json<GoalsListResponse>, ApiError> {
    let goals = GoalsService::get_goals(
        state.db(),
        auth.user_id,
        query.status.as_deref(),
        query.goal_type.as_deref(),
    )
    .await?;

    Ok(Json(GoalsListResponse {
        goals: goals
            .into_iter()
            .map(|g| GoalResponse {
                id: g.id.to_string(),
                name: g.name,
                description: g.description,
                goal_type: g.goal_type,
                metric: g.metric,
                target_value: g.target_value,
                start_value: g.start_value,
                current_value: g.current_value,
                direction: g.direction,
                start_date: g.start_date,
                target_date: g.target_date,
                status: g.status,
            })
            .collect(),
    }))
}

/// GET /api/v1/goals/:id - Get a specific goal
async fn get_goal(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<GoalResponse>, ApiError> {
    let goal_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid goal ID".to_string()))?;

    let goal = GoalsService::get_goal(state.db(), auth.user_id, goal_id).await?;

    Ok(Json(GoalResponse {
        id: goal.id.to_string(),
        name: goal.name,
        description: goal.description,
        goal_type: goal.goal_type,
        metric: goal.metric,
        target_value: goal.target_value,
        start_value: goal.start_value,
        current_value: goal.current_value,
        direction: goal.direction,
        start_date: goal.start_date,
        target_date: goal.target_date,
        status: goal.status,
    }))
}

/// PUT /api/v1/goals/:id - Update a goal
async fn update_goal(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateGoalRequest>,
) -> Result<Json<GoalResponse>, ApiError> {
    let goal_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid goal ID".to_string()))?;

    let input = UpdateGoalInput {
        name: req.name,
        description: req.description,
        target_value: req.target_value,
        current_value: req.current_value,
        target_date: req.target_date,
        status: req.status,
    };

    let goal = GoalsService::update_goal(state.db(), auth.user_id, goal_id, input).await?;

    Ok(Json(GoalResponse {
        id: goal.id.to_string(),
        name: goal.name,
        description: goal.description,
        goal_type: goal.goal_type,
        metric: goal.metric,
        target_value: goal.target_value,
        start_value: goal.start_value,
        current_value: goal.current_value,
        direction: goal.direction,
        start_date: goal.start_date,
        target_date: goal.target_date,
        status: goal.status,
    }))
}

/// DELETE /api/v1/goals/:id - Delete a goal
async fn delete_goal(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let goal_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid goal ID".to_string()))?;

    let deleted = GoalsService::delete_goal(state.db(), auth.user_id, goal_id).await?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err(ApiError::NotFound("Goal not found".to_string()))
    }
}

/// GET /api/v1/goals/:id/progress - Get goal progress
async fn get_progress(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<GoalProgressResponse>, ApiError> {
    let goal_id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid goal ID".to_string()))?;

    let progress = GoalsService::get_progress(state.db(), auth.user_id, goal_id).await?;

    Ok(Json(GoalProgressResponse {
        goal_id: progress.goal_id.to_string(),
        progress_percent: progress.progress_percent,
        remaining: progress.remaining,
        on_track: progress.on_track,
        days_remaining: progress.days_remaining,
        projected_completion: progress.projected_completion,
        milestones: progress
            .milestones
            .into_iter()
            .map(|m| MilestoneResponse {
                id: m.id.to_string(),
                name: m.name,
                target_value: m.target_value,
                percentage: m.percentage,
                achieved: m.achieved,
                actual_value: m.actual_value,
            })
            .collect(),
    }))
}
