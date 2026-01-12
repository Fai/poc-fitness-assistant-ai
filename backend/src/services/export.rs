//! Data export service for exporting user health data
//!
//! Supports multiple formats:
//! - JSON: Full structured export
//! - CSV: Tabular export for spreadsheets
//!
//! Property 14: Data Import/Export Round-Trip
//! Exported data can be re-imported equivalently

use crate::error::ApiError;
use crate::repositories::{
    BiomarkerLogRepository, BodyCompositionRepository, ExerciseSetRepository, GoalRepository,
    HeartRateLogRepository, HrvLogRepository, HydrationLogRepository, MilestoneRepository,
    SleepLogRepository, WeightRepository, WorkoutExerciseRepository, WorkoutRepository,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Complete user data export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataExport {
    pub export_version: String,
    pub exported_at: DateTime<Utc>,
    pub user_id: String,
    pub weight_logs: Vec<WeightLogExport>,
    pub body_composition_logs: Vec<BodyCompositionExport>,
    pub workouts: Vec<WorkoutExport>,
    pub sleep_logs: Vec<SleepLogExport>,
    pub hydration_logs: Vec<HydrationLogExport>,
    pub heart_rate_logs: Vec<HeartRateLogExport>,
    pub hrv_logs: Vec<HrvLogExport>,
    pub biomarker_logs: Vec<BiomarkerLogExport>,
    pub goals: Vec<GoalExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightLogExport {
    pub id: String,
    pub weight_kg: f64,
    pub recorded_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyCompositionExport {
    pub id: String,
    pub recorded_at: DateTime<Utc>,
    pub body_fat_percent: Option<f64>,
    pub muscle_mass_kg: Option<f64>,
    pub water_percent: Option<f64>,
    pub bone_mass_kg: Option<f64>,
    pub visceral_fat: Option<i32>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutExport {
    pub id: String,
    pub name: Option<String>,
    pub workout_type: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub calories_burned: Option<i32>,
    pub distance_meters: Option<f64>,
    pub source: String,
    pub notes: Option<String>,
    pub exercises: Vec<WorkoutExerciseExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutExerciseExport {
    pub exercise_id: String,
    pub sets: Vec<ExerciseSetExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseSetExport {
    pub set_number: i32,
    pub reps: Option<i32>,
    pub weight_kg: Option<f64>,
    pub duration_seconds: Option<i32>,
    pub distance_meters: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepLogExport {
    pub id: String,
    pub sleep_start: DateTime<Utc>,
    pub sleep_end: DateTime<Utc>,
    pub awake_minutes: i32,
    pub light_minutes: i32,
    pub deep_minutes: i32,
    pub rem_minutes: i32,
    pub sleep_score: Option<i32>,
    pub source: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrationLogExport {
    pub id: String,
    pub amount_ml: i32,
    pub beverage_type: String,
    pub consumed_at: DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRateLogExport {
    pub id: String,
    pub bpm: i32,
    pub context: String,
    pub recorded_at: DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HrvLogExport {
    pub id: String,
    pub rmssd: f64,
    pub sdnn: Option<f64>,
    pub context: String,
    pub recorded_at: DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomarkerLogExport {
    pub id: String,
    pub biomarker_name: String,
    pub value: f64,
    pub classification: Option<String>,
    pub test_date: NaiveDate,
    pub lab_name: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalExport {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub goal_type: String,
    pub metric: String,
    pub target_value: f64,
    pub start_value: Option<f64>,
    pub current_value: Option<f64>,
    pub direction: String,
    pub start_date: NaiveDate,
    pub target_date: Option<NaiveDate>,
    pub status: String,
    pub milestones: Vec<MilestoneExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneExport {
    pub name: String,
    pub target_value: f64,
    pub percentage: i32,
    pub achieved: bool,
    pub achieved_at: Option<DateTime<Utc>>,
}

/// CSV export row for weight data
#[derive(Debug, Clone, Serialize)]
pub struct WeightCsvRow {
    pub date: String,
    pub weight_kg: f64,
    pub source: String,
    pub notes: String,
}

/// CSV export row for sleep data
#[derive(Debug, Clone, Serialize)]
pub struct SleepCsvRow {
    pub date: String,
    pub sleep_start: String,
    pub sleep_end: String,
    pub duration_minutes: i32,
    pub efficiency_percent: f64,
    pub deep_minutes: i32,
    pub rem_minutes: i32,
    pub light_minutes: i32,
    pub awake_minutes: i32,
}

/// Data export service
pub struct ExportService;

impl ExportService {
    /// Export all user data as JSON
    pub async fn export_json(pool: &PgPool, user_id: Uuid) -> Result<UserDataExport, ApiError> {
        // Fetch all data in parallel
        let (weights, body_comp, workouts, sleep, hydration, hr, hrv, biomarkers, goals) = tokio::join!(
            Self::fetch_weight_logs(pool, user_id),
            Self::fetch_body_composition(pool, user_id),
            Self::fetch_workouts(pool, user_id),
            Self::fetch_sleep_logs(pool, user_id),
            Self::fetch_hydration_logs(pool, user_id),
            Self::fetch_heart_rate_logs(pool, user_id),
            Self::fetch_hrv_logs(pool, user_id),
            Self::fetch_biomarker_logs(pool, user_id),
            Self::fetch_goals(pool, user_id),
        );

        Ok(UserDataExport {
            export_version: "1.0".to_string(),
            exported_at: Utc::now(),
            user_id: user_id.to_string(),
            weight_logs: weights?,
            body_composition_logs: body_comp?,
            workouts: workouts?,
            sleep_logs: sleep?,
            hydration_logs: hydration?,
            heart_rate_logs: hr?,
            hrv_logs: hrv?,
            biomarker_logs: biomarkers?,
            goals: goals?,
        })
    }

    /// Export weight data as CSV
    pub async fn export_weight_csv(pool: &PgPool, user_id: Uuid) -> Result<String, ApiError> {
        let weights = Self::fetch_weight_logs(pool, user_id).await?;
        
        let rows: Vec<WeightCsvRow> = weights
            .into_iter()
            .map(|w| WeightCsvRow {
                date: w.recorded_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                weight_kg: w.weight_kg,
                source: w.source,
                notes: w.notes.unwrap_or_default(),
            })
            .collect();

        Self::to_csv(&rows)
    }

    /// Export sleep data as CSV
    pub async fn export_sleep_csv(pool: &PgPool, user_id: Uuid) -> Result<String, ApiError> {
        let sleep_logs = Self::fetch_sleep_logs(pool, user_id).await?;
        
        let rows: Vec<SleepCsvRow> = sleep_logs
            .into_iter()
            .map(|s| {
                let duration = (s.sleep_end - s.sleep_start).num_minutes() as i32;
                let efficiency = if duration > 0 {
                    ((duration - s.awake_minutes) as f64 / duration as f64) * 100.0
                } else {
                    0.0
                };
                SleepCsvRow {
                    date: s.sleep_start.format("%Y-%m-%d").to_string(),
                    sleep_start: s.sleep_start.format("%H:%M").to_string(),
                    sleep_end: s.sleep_end.format("%H:%M").to_string(),
                    duration_minutes: duration,
                    efficiency_percent: (efficiency * 10.0).round() / 10.0,
                    deep_minutes: s.deep_minutes,
                    rem_minutes: s.rem_minutes,
                    light_minutes: s.light_minutes,
                    awake_minutes: s.awake_minutes,
                }
            })
            .collect();

        Self::to_csv(&rows)
    }

    /// Convert data to CSV string
    fn to_csv<T: Serialize>(data: &[T]) -> Result<String, ApiError> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        for record in data {
            wtr.serialize(record)
                .map_err(|e| ApiError::Internal(anyhow::anyhow!("CSV serialization error: {}", e)))?;
        }
        let bytes = wtr
            .into_inner()
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("CSV flush error: {}", e)))?;
        String::from_utf8(bytes)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("CSV encoding error: {}", e)))
    }

    async fn fetch_weight_logs(pool: &PgPool, user_id: Uuid) -> Result<Vec<WeightLogExport>, ApiError> {
        let records = WeightRepository::get_by_date_range(pool, user_id, None, None)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records
            .into_iter()
            .map(|r| WeightLogExport {
                id: r.id.to_string(),
                weight_kg: r.weight_kg.to_f64().unwrap_or(0.0),
                recorded_at: r.recorded_at,
                source: r.source,
                notes: r.notes,
            })
            .collect())
    }

    async fn fetch_body_composition(pool: &PgPool, user_id: Uuid) -> Result<Vec<BodyCompositionExport>, ApiError> {
        let records = BodyCompositionRepository::get_by_date_range(pool, user_id, None, None)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records
            .into_iter()
            .map(|r| BodyCompositionExport {
                id: r.id.to_string(),
                recorded_at: r.recorded_at,
                body_fat_percent: r.body_fat_percent.and_then(|d| d.to_f64()),
                muscle_mass_kg: r.muscle_mass_kg.and_then(|d| d.to_f64()),
                water_percent: r.water_percent.and_then(|d| d.to_f64()),
                bone_mass_kg: r.bone_mass_kg.and_then(|d| d.to_f64()),
                visceral_fat: r.visceral_fat,
                source: r.source,
            })
            .collect())
    }

    async fn fetch_workouts(pool: &PgPool, user_id: Uuid) -> Result<Vec<WorkoutExport>, ApiError> {
        let (workouts, _) = WorkoutRepository::get_by_date_range(pool, user_id, None, None, 10000, 0)
            .await
            .map_err(ApiError::Internal)?;

        let mut exports = Vec::new();
        for w in workouts {
            let workout_exercises = WorkoutExerciseRepository::get_by_workout(pool, w.id)
                .await
                .map_err(ApiError::Internal)?;

            let mut exercises = Vec::new();
            for we in workout_exercises {
                let sets = ExerciseSetRepository::get_by_workout_exercise(pool, we.id)
                    .await
                    .map_err(ApiError::Internal)?;

                exercises.push(WorkoutExerciseExport {
                    exercise_id: we.exercise_id.to_string(),
                    sets: sets
                        .into_iter()
                        .map(|s| ExerciseSetExport {
                            set_number: s.set_number,
                            reps: s.reps,
                            weight_kg: s.weight_kg.and_then(|d| d.to_f64()),
                            duration_seconds: s.duration_seconds,
                            distance_meters: s.distance_meters.and_then(|d| d.to_f64()),
                        })
                        .collect(),
                });
            }

            exports.push(WorkoutExport {
                id: w.id.to_string(),
                name: w.name,
                workout_type: w.workout_type,
                started_at: w.started_at,
                ended_at: w.ended_at,
                duration_minutes: w.duration_minutes,
                calories_burned: w.calories_burned,
                distance_meters: w.distance_meters.and_then(|d| d.to_f64()),
                source: w.source,
                notes: w.notes,
                exercises,
            });
        }

        Ok(exports)
    }

    async fn fetch_sleep_logs(pool: &PgPool, user_id: Uuid) -> Result<Vec<SleepLogExport>, ApiError> {
        let start_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2100, 12, 31).unwrap();
        
        let records = SleepLogRepository::get_history(pool, user_id, start_date, end_date, 10000, 0)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records
            .into_iter()
            .map(|r| SleepLogExport {
                id: r.id.to_string(),
                sleep_start: r.sleep_start,
                sleep_end: r.sleep_end,
                awake_minutes: r.awake_minutes,
                light_minutes: r.light_minutes,
                deep_minutes: r.deep_minutes,
                rem_minutes: r.rem_minutes,
                sleep_score: r.sleep_score,
                source: r.source,
                notes: r.notes,
            })
            .collect())
    }

    async fn fetch_hydration_logs(pool: &PgPool, user_id: Uuid) -> Result<Vec<HydrationLogExport>, ApiError> {
        // Get all hydration logs by querying a wide date range
        let start_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2100, 12, 31).unwrap();
        
        let summaries = HydrationLogRepository::get_daily_summaries(pool, user_id, start_date, end_date)
            .await
            .map_err(ApiError::Internal)?;

        // For each day with entries, get the actual logs
        let mut all_logs = Vec::new();
        for summary in summaries {
            let logs = HydrationLogRepository::get_by_date(pool, user_id, summary.date)
                .await
                .map_err(ApiError::Internal)?;
            
            for r in logs {
                all_logs.push(HydrationLogExport {
                    id: r.id.to_string(),
                    amount_ml: r.amount_ml,
                    beverage_type: r.beverage_type,
                    consumed_at: r.consumed_at,
                    source: r.source,
                });
            }
        }

        Ok(all_logs)
    }

    async fn fetch_heart_rate_logs(pool: &PgPool, user_id: Uuid) -> Result<Vec<HeartRateLogExport>, ApiError> {
        let start_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2100, 12, 31).unwrap();
        
        let records = HeartRateLogRepository::get_history(pool, user_id, start_date, end_date, None, 10000, 0)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records
            .into_iter()
            .map(|r| HeartRateLogExport {
                id: r.id.to_string(),
                bpm: r.bpm,
                context: r.context,
                recorded_at: r.recorded_at,
                source: r.source,
            })
            .collect())
    }

    async fn fetch_hrv_logs(pool: &PgPool, user_id: Uuid) -> Result<Vec<HrvLogExport>, ApiError> {
        let start_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2100, 12, 31).unwrap();
        
        let records = HrvLogRepository::get_history(pool, user_id, start_date, end_date, 10000, 0)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records
            .into_iter()
            .map(|r| HrvLogExport {
                id: r.id.to_string(),
                rmssd: r.rmssd.to_f64().unwrap_or(0.0),
                sdnn: r.sdnn.and_then(|d| d.to_f64()),
                context: r.context,
                recorded_at: r.recorded_at,
                source: r.source,
            })
            .collect())
    }

    async fn fetch_biomarker_logs(pool: &PgPool, user_id: Uuid) -> Result<Vec<BiomarkerLogExport>, ApiError> {
        let records = BiomarkerLogRepository::get_by_user(pool, user_id, None, 10000, 0)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records
            .into_iter()
            .map(|r| BiomarkerLogExport {
                id: r.id.to_string(),
                biomarker_name: r.biomarker_name,
                value: r.value.to_f64().unwrap_or(0.0),
                classification: r.classification,
                test_date: r.test_date,
                lab_name: r.lab_name,
                notes: r.notes,
            })
            .collect())
    }

    async fn fetch_goals(pool: &PgPool, user_id: Uuid) -> Result<Vec<GoalExport>, ApiError> {
        let goals = GoalRepository::get_by_user(pool, user_id, None, None)
            .await
            .map_err(ApiError::Internal)?;

        let mut exports = Vec::new();
        for g in goals {
            let milestones = MilestoneRepository::get_by_goal(pool, g.id)
                .await
                .map_err(ApiError::Internal)?;

            exports.push(GoalExport {
                id: g.id.to_string(),
                name: g.name,
                description: g.description,
                goal_type: g.goal_type,
                metric: g.metric,
                target_value: g.target_value.to_f64().unwrap_or(0.0),
                start_value: g.start_value.and_then(|d| d.to_f64()),
                current_value: g.current_value.and_then(|d| d.to_f64()),
                direction: g.direction,
                start_date: g.start_date,
                target_date: g.target_date,
                status: g.status,
                milestones: milestones
                    .into_iter()
                    .map(|m| MilestoneExport {
                        name: m.name,
                        target_value: m.target_value.to_f64().unwrap_or(0.0),
                        percentage: m.percentage,
                        achieved: m.achieved_at.is_some(),
                        achieved_at: m.achieved_at,
                    })
                    .collect(),
            });
        }

        Ok(exports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: fitness-assistant-ai, Property 14: Data Import/Export Round-Trip
    // Test that export format is consistent and can be serialized/deserialized
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_weight_export_roundtrip(
            weight in 30.0f64..300.0,
            source in "[a-z]{3,10}",
        ) {
            let export = WeightLogExport {
                id: Uuid::new_v4().to_string(),
                weight_kg: weight,
                recorded_at: Utc::now(),
                source,
                notes: None,
            };

            let json = serde_json::to_string(&export).unwrap();
            let parsed: WeightLogExport = serde_json::from_str(&json).unwrap();

            prop_assert!((parsed.weight_kg - export.weight_kg).abs() < 0.001);
            prop_assert_eq!(parsed.source, export.source);
        }

        #[test]
        fn test_sleep_export_roundtrip(
            awake in 0i32..60,
            light in 60i32..240,
            deep in 30i32..120,
            rem in 30i32..120,
        ) {
            let export = SleepLogExport {
                id: Uuid::new_v4().to_string(),
                sleep_start: Utc::now(),
                sleep_end: Utc::now(),
                awake_minutes: awake,
                light_minutes: light,
                deep_minutes: deep,
                rem_minutes: rem,
                sleep_score: Some(80),
                source: "test".to_string(),
                notes: None,
            };

            let json = serde_json::to_string(&export).unwrap();
            let parsed: SleepLogExport = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(parsed.awake_minutes, export.awake_minutes);
            prop_assert_eq!(parsed.light_minutes, export.light_minutes);
            prop_assert_eq!(parsed.deep_minutes, export.deep_minutes);
            prop_assert_eq!(parsed.rem_minutes, export.rem_minutes);
        }

        #[test]
        fn test_goal_export_roundtrip(
            target in 50.0f64..200.0,
            start in 60.0f64..150.0,
        ) {
            let export = GoalExport {
                id: Uuid::new_v4().to_string(),
                name: "Test Goal".to_string(),
                description: None,
                goal_type: "weight".to_string(),
                metric: "weight_kg".to_string(),
                target_value: target,
                start_value: Some(start),
                current_value: Some(start),
                direction: "decreasing".to_string(),
                start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                target_date: None,
                status: "active".to_string(),
                milestones: vec![],
            };

            let json = serde_json::to_string(&export).unwrap();
            let parsed: GoalExport = serde_json::from_str(&json).unwrap();

            prop_assert!((parsed.target_value - export.target_value).abs() < 0.001);
            prop_assert!((parsed.start_value.unwrap() - export.start_value.unwrap()).abs() < 0.001);
        }
    }

    #[test]
    fn test_full_export_serialization() {
        let export = UserDataExport {
            export_version: "1.0".to_string(),
            exported_at: Utc::now(),
            user_id: Uuid::new_v4().to_string(),
            weight_logs: vec![],
            body_composition_logs: vec![],
            workouts: vec![],
            sleep_logs: vec![],
            hydration_logs: vec![],
            heart_rate_logs: vec![],
            hrv_logs: vec![],
            biomarker_logs: vec![],
            goals: vec![],
        };

        let json = serde_json::to_string(&export).unwrap();
        let parsed: UserDataExport = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.export_version, "1.0");
    }
}
