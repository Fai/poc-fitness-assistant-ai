//! Exercise and workout repository for database operations

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// Exercise Library
// ============================================================================

/// Exercise record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExerciseRecord {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub muscle_groups: Vec<String>,
    pub equipment: Option<String>,
    pub calories_per_minute: Option<Decimal>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub is_custom: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating an exercise
#[derive(Debug, Clone)]
pub struct CreateExercise {
    pub name: String,
    pub category: String,
    pub muscle_groups: Vec<String>,
    pub equipment: Option<String>,
    pub calories_per_minute: Option<f64>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub is_custom: bool,
    pub created_by: Option<Uuid>,
}

/// Exercise repository
pub struct ExerciseRepository;

impl ExerciseRepository {
    /// Create a new exercise
    pub async fn create(pool: &PgPool, input: CreateExercise) -> Result<ExerciseRecord> {
        let record = sqlx::query_as::<_, ExerciseRecord>(
            r#"
            INSERT INTO exercises (name, category, muscle_groups, equipment, calories_per_minute, 
                                   description, instructions, is_custom, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, name, category, muscle_groups, equipment, calories_per_minute,
                      description, instructions, is_custom, created_by, created_at, updated_at
            "#,
        )
        .bind(&input.name)
        .bind(&input.category)
        .bind(&input.muscle_groups)
        .bind(&input.equipment)
        .bind(input.calories_per_minute)
        .bind(&input.description)
        .bind(&input.instructions)
        .bind(input.is_custom)
        .bind(input.created_by)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get all exercises (library)
    pub async fn get_all(pool: &PgPool) -> Result<Vec<ExerciseRecord>> {
        let records = sqlx::query_as::<_, ExerciseRecord>(
            r#"
            SELECT id, name, category, muscle_groups, equipment, calories_per_minute,
                   description, instructions, is_custom, created_by, created_at, updated_at
            FROM exercises
            WHERE is_custom = FALSE
            ORDER BY category, name
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Get exercises by category
    pub async fn get_by_category(pool: &PgPool, category: &str) -> Result<Vec<ExerciseRecord>> {
        let records = sqlx::query_as::<_, ExerciseRecord>(
            r#"
            SELECT id, name, category, muscle_groups, equipment, calories_per_minute,
                   description, instructions, is_custom, created_by, created_at, updated_at
            FROM exercises
            WHERE category = $1
            ORDER BY name
            "#,
        )
        .bind(category)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Get exercises by muscle group
    pub async fn get_by_muscle_group(pool: &PgPool, muscle_group: &str) -> Result<Vec<ExerciseRecord>> {
        let records = sqlx::query_as::<_, ExerciseRecord>(
            r#"
            SELECT id, name, category, muscle_groups, equipment, calories_per_minute,
                   description, instructions, is_custom, created_by, created_at, updated_at
            FROM exercises
            WHERE $1 = ANY(muscle_groups)
            ORDER BY name
            "#,
        )
        .bind(muscle_group)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Search exercises by name
    pub async fn search(pool: &PgPool, query: &str, limit: i64) -> Result<Vec<ExerciseRecord>> {
        let records = sqlx::query_as::<_, ExerciseRecord>(
            r#"
            SELECT id, name, category, muscle_groups, equipment, calories_per_minute,
                   description, instructions, is_custom, created_by, created_at, updated_at
            FROM exercises
            WHERE LOWER(name) LIKE LOWER($1)
            ORDER BY name
            LIMIT $2
            "#,
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Get exercise by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<ExerciseRecord>> {
        let record = sqlx::query_as::<_, ExerciseRecord>(
            r#"
            SELECT id, name, category, muscle_groups, equipment, calories_per_minute,
                   description, instructions, is_custom, created_by, created_at, updated_at
            FROM exercises
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// Get user's custom exercises
    pub async fn get_custom_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<ExerciseRecord>> {
        let records = sqlx::query_as::<_, ExerciseRecord>(
            r#"
            SELECT id, name, category, muscle_groups, equipment, calories_per_minute,
                   description, instructions, is_custom, created_by, created_at, updated_at
            FROM exercises
            WHERE is_custom = TRUE AND created_by = $1
            ORDER BY name
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Check if exercise exists by name (for seeding)
    pub async fn exists_by_name(pool: &PgPool, name: &str) -> Result<bool> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM exercises WHERE LOWER(name) = LOWER($1))"#,
        )
        .bind(name)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }
}

// ============================================================================
// Workouts
// ============================================================================

/// Workout record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WorkoutRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: Option<String>,
    pub workout_type: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub calories_burned: Option<i32>,
    pub avg_heart_rate: Option<i32>,
    pub max_heart_rate: Option<i32>,
    pub distance_meters: Option<Decimal>,
    pub pace_seconds_per_km: Option<i32>,
    pub elevation_gain_meters: Option<Decimal>,
    pub source: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a workout
#[derive(Debug, Clone)]
pub struct CreateWorkout {
    pub user_id: Uuid,
    pub name: Option<String>,
    pub workout_type: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub calories_burned: Option<i32>,
    pub avg_heart_rate: Option<i32>,
    pub max_heart_rate: Option<i32>,
    pub distance_meters: Option<f64>,
    pub pace_seconds_per_km: Option<i32>,
    pub elevation_gain_meters: Option<f64>,
    pub source: String,
    pub notes: Option<String>,
}

/// Workout repository
pub struct WorkoutRepository;

impl WorkoutRepository {
    /// Create a new workout
    pub async fn create(pool: &PgPool, input: CreateWorkout) -> Result<WorkoutRecord> {
        let record = sqlx::query_as::<_, WorkoutRecord>(
            r#"
            INSERT INTO workouts (user_id, name, workout_type, started_at, ended_at, duration_minutes,
                                  calories_burned, avg_heart_rate, max_heart_rate, distance_meters,
                                  pace_seconds_per_km, elevation_gain_meters, source, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, user_id, name, workout_type, started_at, ended_at, duration_minutes,
                      calories_burned, avg_heart_rate, max_heart_rate, distance_meters,
                      pace_seconds_per_km, elevation_gain_meters, source, notes, created_at, updated_at
            "#,
        )
        .bind(input.user_id)
        .bind(&input.name)
        .bind(&input.workout_type)
        .bind(input.started_at)
        .bind(input.ended_at)
        .bind(input.duration_minutes)
        .bind(input.calories_burned)
        .bind(input.avg_heart_rate)
        .bind(input.max_heart_rate)
        .bind(input.distance_meters)
        .bind(input.pace_seconds_per_km)
        .bind(input.elevation_gain_meters)
        .bind(&input.source)
        .bind(&input.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get workout by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<Option<WorkoutRecord>> {
        let record = sqlx::query_as::<_, WorkoutRecord>(
            r#"
            SELECT id, user_id, name, workout_type, started_at, ended_at, duration_minutes,
                   calories_burned, avg_heart_rate, max_heart_rate, distance_meters,
                   pace_seconds_per_km, elevation_gain_meters, source, notes, created_at, updated_at
            FROM workouts
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// Get workouts for a user within a date range
    pub async fn get_by_date_range(
        pool: &PgPool,
        user_id: Uuid,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<WorkoutRecord>, i64)> {
        let start = start.unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
        let end = end.unwrap_or_else(|| Utc::now() + chrono::Duration::days(1));

        // Get total count
        let count_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM workouts
            WHERE user_id = $1 AND started_at >= $2 AND started_at <= $3
            "#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_one(pool)
        .await?;

        let total_count = count_row.0;

        // Get paginated records
        let records = sqlx::query_as::<_, WorkoutRecord>(
            r#"
            SELECT id, user_id, name, workout_type, started_at, ended_at, duration_minutes,
                   calories_burned, avg_heart_rate, max_heart_rate, distance_meters,
                   pace_seconds_per_km, elevation_gain_meters, source, notes, created_at, updated_at
            FROM workouts
            WHERE user_id = $1 AND started_at >= $2 AND started_at <= $3
            ORDER BY started_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok((records, total_count))
    }

    /// Get workouts for a specific week
    pub async fn get_by_week(
        pool: &PgPool,
        user_id: Uuid,
        week_start: NaiveDate,
    ) -> Result<Vec<WorkoutRecord>> {
        let week_end = week_start + chrono::Duration::days(7);

        let records = sqlx::query_as::<_, WorkoutRecord>(
            r#"
            SELECT id, user_id, name, workout_type, started_at, ended_at, duration_minutes,
                   calories_burned, avg_heart_rate, max_heart_rate, distance_meters,
                   pace_seconds_per_km, elevation_gain_meters, source, notes, created_at, updated_at
            FROM workouts
            WHERE user_id = $1 AND DATE(started_at) >= $2 AND DATE(started_at) < $3
            ORDER BY started_at ASC
            "#,
        )
        .bind(user_id)
        .bind(week_start)
        .bind(week_end)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Delete a workout
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM workouts WHERE id = $1 AND user_id = $2"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// Workout Exercises
// ============================================================================

/// Workout exercise record
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WorkoutExerciseRecord {
    pub id: Uuid,
    pub workout_id: Uuid,
    pub exercise_id: Uuid,
    pub sort_order: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input for adding exercise to workout
#[derive(Debug, Clone)]
pub struct AddWorkoutExercise {
    pub workout_id: Uuid,
    pub exercise_id: Uuid,
    pub sort_order: i32,
    pub notes: Option<String>,
}

/// Workout exercise repository
pub struct WorkoutExerciseRepository;

impl WorkoutExerciseRepository {
    /// Add exercise to workout
    pub async fn create(pool: &PgPool, input: AddWorkoutExercise) -> Result<WorkoutExerciseRecord> {
        let record = sqlx::query_as::<_, WorkoutExerciseRecord>(
            r#"
            INSERT INTO workout_exercises (workout_id, exercise_id, sort_order, notes)
            VALUES ($1, $2, $3, $4)
            RETURNING id, workout_id, exercise_id, sort_order, notes, created_at
            "#,
        )
        .bind(input.workout_id)
        .bind(input.exercise_id)
        .bind(input.sort_order)
        .bind(&input.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get exercises for a workout
    pub async fn get_by_workout(pool: &PgPool, workout_id: Uuid) -> Result<Vec<WorkoutExerciseRecord>> {
        let records = sqlx::query_as::<_, WorkoutExerciseRecord>(
            r#"
            SELECT id, workout_id, exercise_id, sort_order, notes, created_at
            FROM workout_exercises
            WHERE workout_id = $1
            ORDER BY sort_order ASC
            "#,
        )
        .bind(workout_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }
}

// ============================================================================
// Exercise Sets
// ============================================================================

/// Exercise set record
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExerciseSetRecord {
    pub id: Uuid,
    pub workout_exercise_id: Uuid,
    pub set_number: i32,
    pub reps: Option<i32>,
    pub weight_kg: Option<Decimal>,
    pub duration_seconds: Option<i32>,
    pub distance_meters: Option<Decimal>,
    pub rest_seconds: Option<i32>,
    pub rpe: Option<Decimal>,
    pub is_warmup: bool,
    pub is_dropset: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating an exercise set
#[derive(Debug, Clone)]
pub struct CreateExerciseSet {
    pub workout_exercise_id: Uuid,
    pub set_number: i32,
    pub reps: Option<i32>,
    pub weight_kg: Option<f64>,
    pub duration_seconds: Option<i32>,
    pub distance_meters: Option<f64>,
    pub rest_seconds: Option<i32>,
    pub rpe: Option<f64>,
    pub is_warmup: bool,
    pub is_dropset: bool,
    pub notes: Option<String>,
}

/// Exercise set repository
pub struct ExerciseSetRepository;

impl ExerciseSetRepository {
    /// Create an exercise set
    pub async fn create(pool: &PgPool, input: CreateExerciseSet) -> Result<ExerciseSetRecord> {
        let record = sqlx::query_as::<_, ExerciseSetRecord>(
            r#"
            INSERT INTO exercise_sets (workout_exercise_id, set_number, reps, weight_kg, 
                                       duration_seconds, distance_meters, rest_seconds, rpe,
                                       is_warmup, is_dropset, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, workout_exercise_id, set_number, reps, weight_kg, duration_seconds,
                      distance_meters, rest_seconds, rpe, is_warmup, is_dropset, notes, created_at
            "#,
        )
        .bind(input.workout_exercise_id)
        .bind(input.set_number)
        .bind(input.reps)
        .bind(input.weight_kg)
        .bind(input.duration_seconds)
        .bind(input.distance_meters)
        .bind(input.rest_seconds)
        .bind(input.rpe)
        .bind(input.is_warmup)
        .bind(input.is_dropset)
        .bind(&input.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get sets for a workout exercise
    pub async fn get_by_workout_exercise(pool: &PgPool, workout_exercise_id: Uuid) -> Result<Vec<ExerciseSetRecord>> {
        let records = sqlx::query_as::<_, ExerciseSetRecord>(
            r#"
            SELECT id, workout_exercise_id, set_number, reps, weight_kg, duration_seconds,
                   distance_meters, rest_seconds, rpe, is_warmup, is_dropset, notes, created_at
            FROM exercise_sets
            WHERE workout_exercise_id = $1
            ORDER BY set_number ASC
            "#,
        )
        .bind(workout_exercise_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }
}
