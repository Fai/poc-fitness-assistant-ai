//! Exercise and workout service
//!
//! Provides business logic for exercise tracking including:
//! - Exercise library management
//! - Workout logging with sets and exercises
//! - Pace calculation for cardio workouts
//! - Weekly exercise summaries

use crate::error::ApiError;
use crate::repositories::{
    AddWorkoutExercise, CreateExercise, CreateExerciseSet, CreateWorkout, ExerciseRecord,
    ExerciseRepository, ExerciseSetRecord, ExerciseSetRepository,
    WorkoutExerciseRepository, WorkoutRecord, WorkoutRepository,
};
use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// Exercise response for API
#[derive(Debug, Clone)]
pub struct Exercise {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub muscle_groups: Vec<String>,
    pub equipment: Option<String>,
    pub calories_per_minute: Option<f64>,
    pub description: Option<String>,
    pub instructions: Option<String>,
    pub is_custom: bool,
}

/// Workout response for API
#[derive(Debug, Clone)]
pub struct Workout {
    pub id: Uuid,
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

/// Workout with exercises and sets
#[derive(Debug, Clone)]
pub struct WorkoutDetail {
    pub workout: Workout,
    pub exercises: Vec<WorkoutExerciseDetail>,
}

/// Exercise in a workout with sets
#[derive(Debug, Clone)]
pub struct WorkoutExerciseDetail {
    pub id: Uuid,
    pub exercise: Exercise,
    pub sort_order: i32,
    pub notes: Option<String>,
    pub sets: Vec<ExerciseSet>,
}

/// Exercise set response
#[derive(Debug, Clone)]
pub struct ExerciseSet {
    pub id: Uuid,
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

/// Input for logging a workout
#[derive(Debug, Clone)]
pub struct LogWorkoutInput {
    pub name: Option<String>,
    pub workout_type: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub calories_burned: Option<i32>,
    pub avg_heart_rate: Option<i32>,
    pub max_heart_rate: Option<i32>,
    pub distance_meters: Option<f64>,
    pub elevation_gain_meters: Option<f64>,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub exercises: Vec<LogWorkoutExerciseInput>,
}

/// Input for exercise in a workout
#[derive(Debug, Clone)]
pub struct LogWorkoutExerciseInput {
    pub exercise_id: Uuid,
    pub notes: Option<String>,
    pub sets: Vec<LogExerciseSetInput>,
}

/// Input for exercise set
#[derive(Debug, Clone)]
pub struct LogExerciseSetInput {
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

/// Weekly exercise summary
#[derive(Debug, Clone)]
pub struct WeeklyExerciseSummary {
    pub week_start: NaiveDate,
    pub week_end: NaiveDate,
    pub total_workouts: usize,
    pub total_duration_minutes: i32,
    pub total_calories_burned: i32,
    pub workouts_by_type: Vec<WorkoutTypeSummary>,
    pub daily_breakdown: Vec<DailyWorkoutSummary>,
}

/// Summary by workout type
#[derive(Debug, Clone)]
pub struct WorkoutTypeSummary {
    pub workout_type: String,
    pub count: usize,
    pub total_duration_minutes: i32,
    pub total_calories: i32,
}

/// Daily workout summary
#[derive(Debug, Clone)]
pub struct DailyWorkoutSummary {
    pub date: NaiveDate,
    pub workouts: usize,
    pub duration_minutes: i32,
    pub calories_burned: i32,
}

/// Exercise service for business logic
pub struct ExerciseService;

impl ExerciseService {
    /// Get exercise library (all non-custom exercises)
    pub async fn get_exercise_library(pool: &PgPool) -> Result<Vec<Exercise>, ApiError> {
        let records = ExerciseRepository::get_all(pool)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records.into_iter().map(Self::record_to_exercise).collect())
    }

    /// Get exercises by category
    pub async fn get_exercises_by_category(
        pool: &PgPool,
        category: &str,
    ) -> Result<Vec<Exercise>, ApiError> {
        let records = ExerciseRepository::get_by_category(pool, category)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records.into_iter().map(Self::record_to_exercise).collect())
    }

    /// Get exercises by muscle group
    pub async fn get_exercises_by_muscle_group(
        pool: &PgPool,
        muscle_group: &str,
    ) -> Result<Vec<Exercise>, ApiError> {
        let records = ExerciseRepository::get_by_muscle_group(pool, muscle_group)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records.into_iter().map(Self::record_to_exercise).collect())
    }

    /// Search exercises by name
    pub async fn search_exercises(
        pool: &PgPool,
        query: &str,
        limit: i64,
    ) -> Result<Vec<Exercise>, ApiError> {
        let records = ExerciseRepository::search(pool, query, limit)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records.into_iter().map(Self::record_to_exercise).collect())
    }

    /// Get user's custom exercises
    pub async fn get_custom_exercises(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Exercise>, ApiError> {
        let records = ExerciseRepository::get_custom_by_user(pool, user_id)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records.into_iter().map(Self::record_to_exercise).collect())
    }

    /// Create a custom exercise
    pub async fn create_custom_exercise(
        pool: &PgPool,
        user_id: Uuid,
        name: String,
        category: String,
        muscle_groups: Vec<String>,
        equipment: Option<String>,
        calories_per_minute: Option<f64>,
        description: Option<String>,
        instructions: Option<String>,
    ) -> Result<Exercise, ApiError> {
        let input = CreateExercise {
            name,
            category,
            muscle_groups,
            equipment,
            calories_per_minute,
            description,
            instructions,
            is_custom: true,
            created_by: Some(user_id),
        };

        let record = ExerciseRepository::create(pool, input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(Self::record_to_exercise(record))
    }

    /// Log a workout
    ///
    /// Creates a workout with exercises and sets. Automatically calculates
    /// pace for cardio workouts if duration and distance are provided.
    pub async fn log_workout(
        pool: &PgPool,
        user_id: Uuid,
        input: LogWorkoutInput,
    ) -> Result<WorkoutDetail, ApiError> {
        // Calculate pace if this is a cardio workout with distance and duration
        let pace_seconds_per_km = Self::calculate_pace(
            input.duration_minutes,
            input.distance_meters,
        );

        let create_workout = CreateWorkout {
            user_id,
            name: input.name,
            workout_type: input.workout_type,
            started_at: input.started_at,
            ended_at: input.ended_at,
            duration_minutes: input.duration_minutes,
            calories_burned: input.calories_burned,
            avg_heart_rate: input.avg_heart_rate,
            max_heart_rate: input.max_heart_rate,
            distance_meters: input.distance_meters,
            pace_seconds_per_km,
            elevation_gain_meters: input.elevation_gain_meters,
            source: input.source.unwrap_or_else(|| "manual".to_string()),
            notes: input.notes,
        };

        let workout_record = WorkoutRepository::create(pool, create_workout)
            .await
            .map_err(ApiError::Internal)?;

        // Add exercises and sets
        let mut exercise_details = Vec::new();
        for (sort_order, exercise_input) in input.exercises.into_iter().enumerate() {
            let exercise_detail = Self::add_exercise_to_workout(
                pool,
                workout_record.id,
                exercise_input,
                sort_order as i32,
            )
            .await?;
            exercise_details.push(exercise_detail);
        }

        Ok(WorkoutDetail {
            workout: Self::record_to_workout(workout_record),
            exercises: exercise_details,
        })
    }

    /// Add exercise to workout with sets
    async fn add_exercise_to_workout(
        pool: &PgPool,
        workout_id: Uuid,
        input: LogWorkoutExerciseInput,
        sort_order: i32,
    ) -> Result<WorkoutExerciseDetail, ApiError> {
        // Get exercise details
        let exercise_record = ExerciseRepository::get_by_id(pool, input.exercise_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("Exercise not found".to_string()))?;

        // Add exercise to workout
        let add_input = AddWorkoutExercise {
            workout_id,
            exercise_id: input.exercise_id,
            sort_order,
            notes: input.notes.clone(),
        };

        let workout_exercise = WorkoutExerciseRepository::create(pool, add_input)
            .await
            .map_err(ApiError::Internal)?;

        // Add sets
        let mut sets = Vec::new();
        for (set_number, set_input) in input.sets.into_iter().enumerate() {
            let create_set = CreateExerciseSet {
                workout_exercise_id: workout_exercise.id,
                set_number: (set_number + 1) as i32,
                reps: set_input.reps,
                weight_kg: set_input.weight_kg,
                duration_seconds: set_input.duration_seconds,
                distance_meters: set_input.distance_meters,
                rest_seconds: set_input.rest_seconds,
                rpe: set_input.rpe,
                is_warmup: set_input.is_warmup,
                is_dropset: set_input.is_dropset,
                notes: set_input.notes,
            };

            let set_record = ExerciseSetRepository::create(pool, create_set)
                .await
                .map_err(ApiError::Internal)?;

            sets.push(Self::record_to_set(set_record));
        }

        Ok(WorkoutExerciseDetail {
            id: workout_exercise.id,
            exercise: Self::record_to_exercise(exercise_record),
            sort_order: workout_exercise.sort_order,
            notes: workout_exercise.notes,
            sets,
        })
    }

    /// Get workout by ID with full details
    pub async fn get_workout(
        pool: &PgPool,
        user_id: Uuid,
        workout_id: Uuid,
    ) -> Result<WorkoutDetail, ApiError> {
        let workout_record = WorkoutRepository::get_by_id(pool, workout_id, user_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("Workout not found".to_string()))?;

        let exercise_details = Self::get_workout_exercises(pool, workout_id).await?;

        Ok(WorkoutDetail {
            workout: Self::record_to_workout(workout_record),
            exercises: exercise_details,
        })
    }

    /// Get exercises for a workout
    async fn get_workout_exercises(
        pool: &PgPool,
        workout_id: Uuid,
    ) -> Result<Vec<WorkoutExerciseDetail>, ApiError> {
        let workout_exercises = WorkoutExerciseRepository::get_by_workout(pool, workout_id)
            .await
            .map_err(ApiError::Internal)?;

        let mut details = Vec::new();
        for we in workout_exercises {
            let exercise_record = ExerciseRepository::get_by_id(pool, we.exercise_id)
                .await
                .map_err(ApiError::Internal)?
                .ok_or_else(|| ApiError::NotFound("Exercise not found".to_string()))?;

            let sets = ExerciseSetRepository::get_by_workout_exercise(pool, we.id)
                .await
                .map_err(ApiError::Internal)?
                .into_iter()
                .map(Self::record_to_set)
                .collect();

            details.push(WorkoutExerciseDetail {
                id: we.id,
                exercise: Self::record_to_exercise(exercise_record),
                sort_order: we.sort_order,
                notes: we.notes,
                sets,
            });
        }

        Ok(details)
    }

    /// Get workout history with pagination
    pub async fn get_workout_history(
        pool: &PgPool,
        user_id: Uuid,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Workout>, i64), ApiError> {
        let (records, total) = WorkoutRepository::get_by_date_range(
            pool, user_id, start, end, limit, offset,
        )
        .await
        .map_err(ApiError::Internal)?;

        let workouts = records.into_iter().map(Self::record_to_workout).collect();
        Ok((workouts, total))
    }

    /// Delete a workout
    pub async fn delete_workout(
        pool: &PgPool,
        user_id: Uuid,
        workout_id: Uuid,
    ) -> Result<bool, ApiError> {
        WorkoutRepository::delete(pool, workout_id, user_id)
            .await
            .map_err(ApiError::Internal)
    }

    /// Calculate pace in seconds per kilometer
    ///
    /// # Property 9: Pace Calculation Correctness
    /// pace = (duration_minutes * 60) / (distance_meters / 1000)
    /// pace = (duration_minutes * 60 * 1000) / distance_meters
    pub fn calculate_pace(duration_minutes: Option<i32>, distance_meters: Option<f64>) -> Option<i32> {
        match (duration_minutes, distance_meters) {
            (Some(duration), Some(distance)) if distance > 0.0 => {
                let duration_seconds = duration as f64 * 60.0;
                let distance_km = distance / 1000.0;
                let pace = duration_seconds / distance_km;
                Some(pace.round() as i32)
            }
            _ => None,
        }
    }

    /// Get weekly exercise summary
    ///
    /// # Property 10: Weekly Exercise Volume
    /// Weekly total equals sum of all workouts in the week
    pub async fn get_weekly_summary(
        pool: &PgPool,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<WeeklyExerciseSummary, ApiError> {
        // Find the Monday of the week containing the given date
        let week_start = Self::get_week_start(date);
        let week_end = week_start + chrono::Duration::days(6);

        let workouts = WorkoutRepository::get_by_week(pool, user_id, week_start)
            .await
            .map_err(ApiError::Internal)?;

        // Calculate totals
        let total_workouts = workouts.len();
        let total_duration_minutes: i32 = workouts
            .iter()
            .filter_map(|w| w.duration_minutes)
            .sum();
        let total_calories_burned: i32 = workouts
            .iter()
            .filter_map(|w| w.calories_burned)
            .sum();

        // Group by workout type
        let mut type_map: std::collections::HashMap<String, WorkoutTypeSummary> =
            std::collections::HashMap::new();
        for workout in &workouts {
            let entry = type_map
                .entry(workout.workout_type.clone())
                .or_insert_with(|| WorkoutTypeSummary {
                    workout_type: workout.workout_type.clone(),
                    count: 0,
                    total_duration_minutes: 0,
                    total_calories: 0,
                });
            entry.count += 1;
            entry.total_duration_minutes += workout.duration_minutes.unwrap_or(0);
            entry.total_calories += workout.calories_burned.unwrap_or(0);
        }
        let workouts_by_type: Vec<WorkoutTypeSummary> = type_map.into_values().collect();

        // Daily breakdown
        let mut daily_map: std::collections::HashMap<NaiveDate, DailyWorkoutSummary> =
            std::collections::HashMap::new();
        for workout in &workouts {
            let date = workout.started_at.date_naive();
            let entry = daily_map.entry(date).or_insert_with(|| DailyWorkoutSummary {
                date,
                workouts: 0,
                duration_minutes: 0,
                calories_burned: 0,
            });
            entry.workouts += 1;
            entry.duration_minutes += workout.duration_minutes.unwrap_or(0);
            entry.calories_burned += workout.calories_burned.unwrap_or(0);
        }
        let mut daily_breakdown: Vec<DailyWorkoutSummary> = daily_map.into_values().collect();
        daily_breakdown.sort_by_key(|d| d.date);

        Ok(WeeklyExerciseSummary {
            week_start,
            week_end,
            total_workouts,
            total_duration_minutes,
            total_calories_burned,
            workouts_by_type,
            daily_breakdown,
        })
    }

    /// Get the Monday of the week containing the given date
    fn get_week_start(date: NaiveDate) -> NaiveDate {
        let weekday = date.weekday();
        let days_from_monday = weekday.num_days_from_monday() as i64;
        date - chrono::Duration::days(days_from_monday)
    }

    /// Convert database record to Exercise
    fn record_to_exercise(record: ExerciseRecord) -> Exercise {
        Exercise {
            id: record.id,
            name: record.name,
            category: record.category,
            muscle_groups: record.muscle_groups,
            equipment: record.equipment,
            calories_per_minute: record.calories_per_minute.map(|d| decimal_to_f64(&d)),
            description: record.description,
            instructions: record.instructions,
            is_custom: record.is_custom,
        }
    }

    /// Convert database record to Workout
    fn record_to_workout(record: WorkoutRecord) -> Workout {
        Workout {
            id: record.id,
            name: record.name,
            workout_type: record.workout_type,
            started_at: record.started_at,
            ended_at: record.ended_at,
            duration_minutes: record.duration_minutes,
            calories_burned: record.calories_burned,
            avg_heart_rate: record.avg_heart_rate,
            max_heart_rate: record.max_heart_rate,
            distance_meters: record.distance_meters.map(|d| decimal_to_f64(&d)),
            pace_seconds_per_km: record.pace_seconds_per_km,
            elevation_gain_meters: record.elevation_gain_meters.map(|d| decimal_to_f64(&d)),
            source: record.source,
            notes: record.notes,
        }
    }

    /// Convert database record to ExerciseSet
    fn record_to_set(record: ExerciseSetRecord) -> ExerciseSet {
        ExerciseSet {
            id: record.id,
            set_number: record.set_number,
            reps: record.reps,
            weight_kg: record.weight_kg.map(|d| decimal_to_f64(&d)),
            duration_seconds: record.duration_seconds,
            distance_meters: record.distance_meters.map(|d| decimal_to_f64(&d)),
            rest_seconds: record.rest_seconds,
            rpe: record.rpe.map(|d| decimal_to_f64(&d)),
            is_warmup: record.is_warmup,
            is_dropset: record.is_dropset,
            notes: record.notes,
        }
    }
}

/// Convert Decimal to f64
fn decimal_to_f64(d: &Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: fitness-assistant-ai, Property 9: Pace Calculation Correctness
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_pace_calculation_formula(
            duration_minutes in 1i32..600,  // 1 min to 10 hours
            distance_meters in 100.0f64..100000.0  // 100m to 100km
        ) {
            let pace = ExerciseService::calculate_pace(Some(duration_minutes), Some(distance_meters));

            prop_assert!(pace.is_some());
            let pace_value = pace.unwrap();

            // Verify formula: pace = (duration_seconds) / (distance_km)
            let expected_pace = (duration_minutes as f64 * 60.0) / (distance_meters / 1000.0);
            let expected_rounded = expected_pace.round() as i32;

            prop_assert_eq!(pace_value, expected_rounded,
                "Pace {} != expected {} for duration={}min, distance={}m",
                pace_value, expected_rounded, duration_minutes, distance_meters);
        }

        #[test]
        fn test_pace_none_when_no_duration(distance_meters in 100.0f64..100000.0) {
            let pace = ExerciseService::calculate_pace(None, Some(distance_meters));
            prop_assert!(pace.is_none());
        }

        #[test]
        fn test_pace_none_when_no_distance(duration_minutes in 1i32..600) {
            let pace = ExerciseService::calculate_pace(Some(duration_minutes), None);
            prop_assert!(pace.is_none());
        }

        #[test]
        fn test_pace_none_when_zero_distance(duration_minutes in 1i32..600) {
            let pace = ExerciseService::calculate_pace(Some(duration_minutes), Some(0.0));
            prop_assert!(pace.is_none());
        }
    }

    // Feature: fitness-assistant-ai, Property 10: Weekly Exercise Volume
    #[test]
    fn test_week_start_calculation() {
        // Monday should return itself
        let monday = NaiveDate::from_ymd_opt(2024, 12, 30).unwrap(); // Monday
        assert_eq!(ExerciseService::get_week_start(monday), monday);

        // Sunday should return previous Monday
        let sunday = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap(); // Sunday
        let expected_monday = NaiveDate::from_ymd_opt(2024, 12, 30).unwrap();
        assert_eq!(ExerciseService::get_week_start(sunday), expected_monday);

        // Wednesday should return Monday of same week
        let wednesday = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(); // Wednesday
        assert_eq!(ExerciseService::get_week_start(wednesday), expected_monday);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_week_start_always_monday(
            year in 2020i32..2030,
            day_of_year in 1u32..366
        ) {
            // Create a date from year and day of year
            let date = NaiveDate::from_yo_opt(year, day_of_year);
            prop_assume!(date.is_some());
            let date = date.unwrap();

            let week_start = ExerciseService::get_week_start(date);

            // Week start should always be a Monday
            prop_assert_eq!(week_start.weekday(), Weekday::Mon,
                "Week start {} is not Monday for date {}", week_start, date);

            // Week start should be <= date
            prop_assert!(week_start <= date,
                "Week start {} is after date {}", week_start, date);

            // Week start should be within 6 days of date
            let days_diff = (date - week_start).num_days();
            prop_assert!(days_diff >= 0 && days_diff <= 6,
                "Week start {} is {} days from date {}", week_start, days_diff, date);
        }
    }
}
