//! Exercise and workout API routes

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::services::exercise::{
    ExerciseService, LogExerciseSetInput, LogWorkoutExerciseInput, LogWorkoutInput,
};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use chrono::NaiveDate;
use fitness_assistant_shared::types::{
    CreateExerciseRequest, DailyWorkoutSummaryResponse, ExerciseLibraryQuery, ExerciseResponse,
    ExerciseSetInput, ExerciseSetResponse, LogWorkoutRequest, WorkoutDetailResponse,
    WorkoutExerciseInput, WorkoutExerciseResponse, WorkoutHistoryQuery, WorkoutHistoryResponse,
    WorkoutResponse, WorkoutTypeSummaryResponse, WeeklyExerciseSummaryResponse,
};
use uuid::Uuid;

/// Create exercise routes
pub fn exercise_routes() -> Router<AppState> {
    Router::new()
        .route("/library", get(get_exercise_library))
        .route("/custom", post(create_custom_exercise).get(get_custom_exercises))
        .route("/workout", post(log_workout))
        .route("/workout/:id", get(get_workout).delete(delete_workout))
        .route("/history", get(get_workout_history))
        .route("/weekly/:date", get(get_weekly_summary))
}

/// GET /api/v1/exercise/library - Get exercise library
async fn get_exercise_library(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ExerciseLibraryQuery>,
) -> Result<Json<Vec<ExerciseResponse>>, ApiError> {
    let mut exercises = Vec::new();

    // Get exercises based on filters
    if let Some(ref search) = query.search {
        let results = ExerciseService::search_exercises(state.db(), search, query.limit).await?;
        exercises.extend(results);
    } else if let Some(ref category) = query.category {
        let results = ExerciseService::get_exercises_by_category(state.db(), category).await?;
        exercises.extend(results);
    } else if let Some(ref muscle_group) = query.muscle_group {
        let results =
            ExerciseService::get_exercises_by_muscle_group(state.db(), muscle_group).await?;
        exercises.extend(results);
    } else {
        let results = ExerciseService::get_exercise_library(state.db()).await?;
        exercises.extend(results);
    }

    // Include custom exercises if requested
    if query.include_custom {
        let custom = ExerciseService::get_custom_exercises(state.db(), auth.user_id).await?;
        exercises.extend(custom);
    }

    let response: Vec<ExerciseResponse> = exercises
        .into_iter()
        .map(|e| ExerciseResponse {
            id: e.id.to_string(),
            name: e.name,
            category: e.category,
            muscle_groups: e.muscle_groups,
            equipment: e.equipment,
            calories_per_minute: e.calories_per_minute,
            description: e.description,
            instructions: e.instructions,
            is_custom: e.is_custom,
        })
        .collect();

    Ok(Json(response))
}

/// POST /api/v1/exercise/custom - Create custom exercise
async fn create_custom_exercise(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateExerciseRequest>,
) -> Result<Json<ExerciseResponse>, ApiError> {
    let exercise = ExerciseService::create_custom_exercise(
        state.db(),
        auth.user_id,
        req.name,
        req.category,
        req.muscle_groups,
        req.equipment,
        req.calories_per_minute,
        req.description,
        req.instructions,
    )
    .await?;

    Ok(Json(ExerciseResponse {
        id: exercise.id.to_string(),
        name: exercise.name,
        category: exercise.category,
        muscle_groups: exercise.muscle_groups,
        equipment: exercise.equipment,
        calories_per_minute: exercise.calories_per_minute,
        description: exercise.description,
        instructions: exercise.instructions,
        is_custom: exercise.is_custom,
    }))
}

/// GET /api/v1/exercise/custom - Get user's custom exercises
async fn get_custom_exercises(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<ExerciseResponse>>, ApiError> {
    let exercises = ExerciseService::get_custom_exercises(state.db(), auth.user_id).await?;

    let response: Vec<ExerciseResponse> = exercises
        .into_iter()
        .map(|e| ExerciseResponse {
            id: e.id.to_string(),
            name: e.name,
            category: e.category,
            muscle_groups: e.muscle_groups,
            equipment: e.equipment,
            calories_per_minute: e.calories_per_minute,
            description: e.description,
            instructions: e.instructions,
            is_custom: e.is_custom,
        })
        .collect();

    Ok(Json(response))
}

/// POST /api/v1/exercise/workout - Log a workout
async fn log_workout(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LogWorkoutRequest>,
) -> Result<Json<WorkoutDetailResponse>, ApiError> {
    let input = LogWorkoutInput {
        name: req.name,
        workout_type: req.workout_type,
        started_at: req.started_at,
        ended_at: req.ended_at,
        duration_minutes: req.duration_minutes,
        calories_burned: req.calories_burned,
        avg_heart_rate: req.avg_heart_rate,
        max_heart_rate: req.max_heart_rate,
        distance_meters: req.distance_meters,
        elevation_gain_meters: req.elevation_gain_meters,
        source: req.source,
        notes: req.notes,
        exercises: req
            .exercises
            .into_iter()
            .map(|e| convert_exercise_input(e))
            .collect::<Result<Vec<_>, _>>()?,
    };

    let detail = ExerciseService::log_workout(state.db(), auth.user_id, input).await?;

    Ok(Json(convert_workout_detail(detail)))
}

/// GET /api/v1/exercise/workout/:id - Get workout details
async fn get_workout(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<WorkoutDetailResponse>, ApiError> {
    let workout_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid workout ID".to_string()))?;

    let detail = ExerciseService::get_workout(state.db(), auth.user_id, workout_id).await?;

    Ok(Json(convert_workout_detail(detail)))
}

/// DELETE /api/v1/exercise/workout/:id - Delete a workout
async fn delete_workout(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let workout_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::Validation("Invalid workout ID".to_string()))?;

    let deleted = ExerciseService::delete_workout(state.db(), auth.user_id, workout_id).await?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err(ApiError::NotFound("Workout not found".to_string()))
    }
}

/// GET /api/v1/exercise/history - Get workout history
async fn get_workout_history(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<WorkoutHistoryQuery>,
) -> Result<Json<WorkoutHistoryResponse>, ApiError> {
    let query = query.normalize();

    let (workouts, total_count) = ExerciseService::get_workout_history(
        state.db(),
        auth.user_id,
        query.start,
        query.end,
        query.limit,
        query.offset,
    )
    .await?;

    let items: Vec<WorkoutResponse> = workouts.into_iter().map(convert_workout).collect();
    let has_more = query.offset + (items.len() as i64) < total_count;

    Ok(Json(WorkoutHistoryResponse {
        items,
        total_count,
        limit: query.limit,
        offset: query.offset,
        has_more,
    }))
}

/// GET /api/v1/exercise/weekly/:date - Get weekly exercise summary
async fn get_weekly_summary(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(date): Path<String>,
) -> Result<Json<WeeklyExerciseSummaryResponse>, ApiError> {
    let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|_| ApiError::Validation("Invalid date format. Use YYYY-MM-DD".to_string()))?;

    let summary = ExerciseService::get_weekly_summary(state.db(), auth.user_id, date).await?;

    Ok(Json(WeeklyExerciseSummaryResponse {
        week_start: summary.week_start,
        week_end: summary.week_end,
        total_workouts: summary.total_workouts,
        total_duration_minutes: summary.total_duration_minutes,
        total_calories_burned: summary.total_calories_burned,
        workouts_by_type: summary
            .workouts_by_type
            .into_iter()
            .map(|t| WorkoutTypeSummaryResponse {
                workout_type: t.workout_type,
                count: t.count,
                total_duration_minutes: t.total_duration_minutes,
                total_calories: t.total_calories,
            })
            .collect(),
        daily_breakdown: summary
            .daily_breakdown
            .into_iter()
            .map(|d| DailyWorkoutSummaryResponse {
                date: d.date,
                workouts: d.workouts,
                duration_minutes: d.duration_minutes,
                calories_burned: d.calories_burned,
            })
            .collect(),
    }))
}

// Helper functions for type conversion

fn convert_exercise_input(input: WorkoutExerciseInput) -> Result<LogWorkoutExerciseInput, ApiError> {
    let exercise_id = Uuid::parse_str(&input.exercise_id)
        .map_err(|_| ApiError::Validation("Invalid exercise ID".to_string()))?;

    Ok(LogWorkoutExerciseInput {
        exercise_id,
        notes: input.notes,
        sets: input.sets.into_iter().map(convert_set_input).collect(),
    })
}

fn convert_set_input(input: ExerciseSetInput) -> LogExerciseSetInput {
    LogExerciseSetInput {
        reps: input.reps,
        weight_kg: input.weight_kg,
        duration_seconds: input.duration_seconds,
        distance_meters: input.distance_meters,
        rest_seconds: input.rest_seconds,
        rpe: input.rpe,
        is_warmup: input.is_warmup,
        is_dropset: input.is_dropset,
        notes: input.notes,
    }
}

fn convert_workout(workout: crate::services::exercise::Workout) -> WorkoutResponse {
    WorkoutResponse {
        id: workout.id.to_string(),
        name: workout.name,
        workout_type: workout.workout_type,
        started_at: workout.started_at,
        ended_at: workout.ended_at,
        duration_minutes: workout.duration_minutes,
        calories_burned: workout.calories_burned,
        avg_heart_rate: workout.avg_heart_rate,
        max_heart_rate: workout.max_heart_rate,
        distance_meters: workout.distance_meters,
        pace_seconds_per_km: workout.pace_seconds_per_km,
        elevation_gain_meters: workout.elevation_gain_meters,
        source: workout.source,
        notes: workout.notes,
    }
}

fn convert_workout_detail(
    detail: crate::services::exercise::WorkoutDetail,
) -> WorkoutDetailResponse {
    WorkoutDetailResponse {
        workout: convert_workout(detail.workout),
        exercises: detail
            .exercises
            .into_iter()
            .map(|e| WorkoutExerciseResponse {
                id: e.id.to_string(),
                exercise: ExerciseResponse {
                    id: e.exercise.id.to_string(),
                    name: e.exercise.name,
                    category: e.exercise.category,
                    muscle_groups: e.exercise.muscle_groups,
                    equipment: e.exercise.equipment,
                    calories_per_minute: e.exercise.calories_per_minute,
                    description: e.exercise.description,
                    instructions: e.exercise.instructions,
                    is_custom: e.exercise.is_custom,
                },
                sort_order: e.sort_order,
                notes: e.notes,
                sets: e
                    .sets
                    .into_iter()
                    .map(|s| ExerciseSetResponse {
                        id: s.id.to_string(),
                        set_number: s.set_number,
                        reps: s.reps,
                        weight_kg: s.weight_kg,
                        duration_seconds: s.duration_seconds,
                        distance_meters: s.distance_meters,
                        rest_seconds: s.rest_seconds,
                        rpe: s.rpe,
                        is_warmup: s.is_warmup,
                        is_dropset: s.is_dropset,
                        notes: s.notes,
                    })
                    .collect(),
            })
            .collect(),
    }
}
