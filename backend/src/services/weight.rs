//! Weight and body composition service
//!
//! Provides business logic for weight tracking including:
//! - Weight logging with anomaly detection
//! - Moving average calculations
//! - Goal projection

use crate::error::ApiError;
use crate::repositories::{
    BodyCompositionRepository, CreateBodyCompositionLog, CreateWeightLog, WeightRepository,
};
use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// Anomaly detection threshold: 2% daily change
const ANOMALY_THRESHOLD_PERCENT: f64 = 2.0;

/// Weight entry input
#[derive(Debug, Clone)]
pub struct WeightEntryInput {
    pub weight_kg: f64,
    pub recorded_at: DateTime<Utc>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

/// Body composition entry input
#[derive(Debug, Clone)]
pub struct BodyCompositionInput {
    pub recorded_at: DateTime<Utc>,
    pub body_fat_percent: Option<f64>,
    pub muscle_mass_kg: Option<f64>,
    pub water_percent: Option<f64>,
    pub bone_mass_kg: Option<f64>,
    pub visceral_fat: Option<i32>,
    pub source: Option<String>,
}

/// Weight log response
#[derive(Debug, Clone)]
pub struct WeightLog {
    pub id: Uuid,
    pub weight_kg: f64,
    pub recorded_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
    pub is_anomaly: bool,
}

/// Body composition log response
#[derive(Debug, Clone)]
pub struct BodyCompositionLog {
    pub id: Uuid,
    pub recorded_at: DateTime<Utc>,
    pub body_fat_percent: Option<f64>,
    pub muscle_mass_kg: Option<f64>,
    pub water_percent: Option<f64>,
    pub bone_mass_kg: Option<f64>,
    pub visceral_fat: Option<i32>,
    pub source: String,
}

/// Weight trend analysis
#[derive(Debug, Clone)]
pub struct WeightTrend {
    pub current_weight: f64,
    pub start_weight: f64,
    pub total_change: f64,
    pub average_daily_change: f64,
    pub moving_average_7d: Option<f64>,
    pub moving_average_30d: Option<f64>,
    pub entries_count: usize,
}

/// Goal projection result
#[derive(Debug, Clone)]
pub struct GoalProjection {
    pub target_weight: f64,
    pub current_weight: f64,
    pub weight_to_lose: f64,
    pub average_daily_change: f64,
    pub projected_days: Option<i64>,
    pub projected_date: Option<DateTime<Utc>>,
    pub on_track: bool,
}

/// Weight service for business logic
pub struct WeightService;

impl WeightService {
    /// Log a weight entry with automatic anomaly detection
    ///
    /// # Property 5: Anomaly Detection Threshold
    /// If the absolute percentage change from the previous entry exceeds 2%,
    /// the entry is flagged as anomalous.
    pub async fn log_weight(
        pool: &PgPool,
        user_id: Uuid,
        input: WeightEntryInput,
    ) -> Result<WeightLog, ApiError> {
        // Validate weight range
        if input.weight_kg < 20.0 || input.weight_kg > 500.0 {
            return Err(ApiError::Validation(
                "Weight must be between 20 and 500 kg".to_string(),
            ));
        }

        // Check for anomaly by comparing with previous entry
        let is_anomaly = Self::detect_anomaly(pool, user_id, input.weight_kg).await?;

        let create_input = CreateWeightLog {
            user_id,
            weight_kg: input.weight_kg,
            recorded_at: input.recorded_at,
            source: input.source.unwrap_or_else(|| "manual".to_string()),
            notes: input.notes,
            is_anomaly,
        };

        let record = WeightRepository::create(pool, create_input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(WeightLog {
            id: record.id,
            weight_kg: decimal_to_f64(&record.weight_kg),
            recorded_at: record.recorded_at,
            source: record.source,
            notes: record.notes,
            is_anomaly: record.is_anomaly,
        })
    }

    /// Detect if a weight entry is anomalous (>2% change from previous)
    async fn detect_anomaly(pool: &PgPool, user_id: Uuid, new_weight: f64) -> Result<bool, ApiError> {
        let previous = WeightRepository::get_latest(pool, user_id)
            .await
            .map_err(ApiError::Internal)?;

        match previous {
            Some(prev) => {
                let prev_weight = decimal_to_f64(&prev.weight_kg);
                let percent_change = ((new_weight - prev_weight) / prev_weight).abs() * 100.0;
                Ok(percent_change > ANOMALY_THRESHOLD_PERCENT)
            }
            None => Ok(false), // First entry is never anomalous
        }
    }

    /// Get weight history for a date range
    pub async fn get_weight_history(
        pool: &PgPool,
        user_id: Uuid,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<Vec<WeightLog>, ApiError> {
        let records = WeightRepository::get_by_date_range(pool, user_id, start, end)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records
            .into_iter()
            .map(|r| WeightLog {
                id: r.id,
                weight_kg: decimal_to_f64(&r.weight_kg),
                recorded_at: r.recorded_at,
                source: r.source,
                notes: r.notes,
                is_anomaly: r.is_anomaly,
            })
            .collect())
    }

    /// Get weight history with pagination
    /// 
    /// Returns (logs, total_count) for paginated responses
    pub async fn get_weight_history_paginated(
        pool: &PgPool,
        user_id: Uuid,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<WeightLog>, i64), ApiError> {
        let (records, total_count) = WeightRepository::get_by_date_range_paginated(
            pool, user_id, start, end, limit, offset
        )
            .await
            .map_err(ApiError::Internal)?;

        let logs = records
            .into_iter()
            .map(|r| WeightLog {
                id: r.id,
                weight_kg: decimal_to_f64(&r.weight_kg),
                recorded_at: r.recorded_at,
                source: r.source,
                notes: r.notes,
                is_anomaly: r.is_anomaly,
            })
            .collect();

        Ok((logs, total_count))
    }

    /// Calculate weight trend analysis
    ///
    /// # Property 3: Moving Average Calculation
    /// The N-day moving average equals the arithmetic mean of the N most recent entries.
    pub async fn get_weight_trend(
        pool: &PgPool,
        user_id: Uuid,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<WeightTrend, ApiError> {
        let records = WeightRepository::get_by_date_range(pool, user_id, start, end)
            .await
            .map_err(ApiError::Internal)?;

        if records.is_empty() {
            return Err(ApiError::NotFound("No weight entries found".to_string()));
        }

        let weights: Vec<f64> = records
            .iter()
            .map(|r| decimal_to_f64(&r.weight_kg))
            .collect();

        // Records are ordered DESC, so first is most recent
        let current_weight = weights[0];
        let start_weight = weights[weights.len() - 1];
        let total_change = current_weight - start_weight;

        // Calculate average daily change
        let days = if records.len() > 1 {
            let first_date = records[records.len() - 1].recorded_at;
            let last_date = records[0].recorded_at;
            (last_date - first_date).num_days().max(1) as f64
        } else {
            1.0
        };
        let average_daily_change = total_change / days;

        // Calculate moving averages
        let moving_average_7d = Self::calculate_moving_average(&weights, 7);
        let moving_average_30d = Self::calculate_moving_average(&weights, 30);

        Ok(WeightTrend {
            current_weight,
            start_weight,
            total_change,
            average_daily_change,
            moving_average_7d,
            moving_average_30d,
            entries_count: records.len(),
        })
    }

    /// Calculate N-day moving average from weight entries
    ///
    /// # Property 3: Moving Average Calculation
    /// Returns the arithmetic mean of the N most recent entries.
    pub fn calculate_moving_average(weights: &[f64], n: usize) -> Option<f64> {
        if weights.is_empty() || n == 0 {
            return None;
        }

        let count = weights.len().min(n);
        let sum: f64 = weights.iter().take(count).sum();
        Some(sum / count as f64)
    }

    /// Project goal completion date
    ///
    /// # Property 4: Weight Goal Projection
    /// days_remaining = |current_weight - target_weight| / average_daily_change
    pub async fn project_goal(
        pool: &PgPool,
        user_id: Uuid,
        target_weight: f64,
    ) -> Result<GoalProjection, ApiError> {
        // Get recent weight history (at least 7 entries for meaningful projection)
        let records = WeightRepository::get_recent(pool, user_id, 30)
            .await
            .map_err(ApiError::Internal)?;

        if records.len() < 7 {
            return Err(ApiError::Validation(
                "Need at least 7 weight entries for goal projection".to_string(),
            ));
        }

        let weights: Vec<f64> = records
            .iter()
            .map(|r| decimal_to_f64(&r.weight_kg))
            .collect();

        let current_weight = weights[0];
        let oldest_weight = weights[weights.len() - 1];

        // Calculate days between oldest and newest entry
        let first_date = records[records.len() - 1].recorded_at;
        let last_date = records[0].recorded_at;
        let days = (last_date - first_date).num_days().max(1) as f64;

        let total_change = current_weight - oldest_weight;
        let average_daily_change = total_change / days;

        let weight_to_lose = current_weight - target_weight;

        // Determine if we're moving in the right direction
        let moving_toward_goal = if weight_to_lose > 0.0 {
            // Need to lose weight, so daily change should be negative
            average_daily_change < 0.0
        } else if weight_to_lose < 0.0 {
            // Need to gain weight, so daily change should be positive
            average_daily_change > 0.0
        } else {
            true // Already at goal
        };

        let (projected_days, projected_date) = if average_daily_change.abs() < 0.001 {
            // No meaningful change
            (None, None)
        } else if moving_toward_goal {
            let days_remaining = (weight_to_lose.abs() / average_daily_change.abs()).ceil() as i64;
            let projected = Utc::now() + chrono::Duration::days(days_remaining);
            (Some(days_remaining), Some(projected))
        } else {
            // Moving away from goal
            (None, None)
        };

        Ok(GoalProjection {
            target_weight,
            current_weight,
            weight_to_lose,
            average_daily_change,
            projected_days,
            projected_date,
            on_track: moving_toward_goal,
        })
    }

    /// Log body composition entry
    pub async fn log_body_composition(
        pool: &PgPool,
        user_id: Uuid,
        input: BodyCompositionInput,
    ) -> Result<BodyCompositionLog, ApiError> {
        // Validate ranges
        if let Some(bf) = input.body_fat_percent {
            if bf < 0.0 || bf > 100.0 {
                return Err(ApiError::Validation(
                    "Body fat percent must be between 0 and 100".to_string(),
                ));
            }
        }

        let create_input = CreateBodyCompositionLog {
            user_id,
            recorded_at: input.recorded_at,
            body_fat_percent: input.body_fat_percent,
            muscle_mass_kg: input.muscle_mass_kg,
            water_percent: input.water_percent,
            bone_mass_kg: input.bone_mass_kg,
            visceral_fat: input.visceral_fat,
            source: input.source.unwrap_or_else(|| "manual".to_string()),
        };

        let record = BodyCompositionRepository::create(pool, create_input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(BodyCompositionLog {
            id: record.id,
            recorded_at: record.recorded_at,
            body_fat_percent: record.body_fat_percent.map(|v| decimal_to_f64(&v)),
            muscle_mass_kg: record.muscle_mass_kg.map(|v| decimal_to_f64(&v)),
            water_percent: record.water_percent.map(|v| decimal_to_f64(&v)),
            bone_mass_kg: record.bone_mass_kg.map(|v| decimal_to_f64(&v)),
            visceral_fat: record.visceral_fat,
            source: record.source,
        })
    }

    /// Get body composition history
    pub async fn get_body_composition_history(
        pool: &PgPool,
        user_id: Uuid,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<Vec<BodyCompositionLog>, ApiError> {
        let records = BodyCompositionRepository::get_by_date_range(pool, user_id, start, end)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records
            .into_iter()
            .map(|r| BodyCompositionLog {
                id: r.id,
                recorded_at: r.recorded_at,
                body_fat_percent: r.body_fat_percent.map(|v| decimal_to_f64(&v)),
                muscle_mass_kg: r.muscle_mass_kg.map(|v| decimal_to_f64(&v)),
                water_percent: r.water_percent.map(|v| decimal_to_f64(&v)),
                bone_mass_kg: r.bone_mass_kg.map(|v| decimal_to_f64(&v)),
                visceral_fat: r.visceral_fat,
                source: r.source,
            })
            .collect())
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

    // Feature: fitness-assistant-ai, Property 3: Moving Average Calculation
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_moving_average_equals_arithmetic_mean(
            weights in prop::collection::vec(20.0f64..500.0, 1..50),
            n in 1usize..20
        ) {
            let result = WeightService::calculate_moving_average(&weights, n);

            // Moving average should exist for non-empty input
            prop_assert!(result.is_some());

            let avg = result.unwrap();
            let count = weights.len().min(n);
            let expected: f64 = weights.iter().take(count).sum::<f64>() / count as f64;

            // Allow small floating point tolerance
            prop_assert!((avg - expected).abs() < 0.0001,
                "Moving average {} != expected {} for n={}", avg, expected, n);
        }

        #[test]
        fn test_moving_average_empty_returns_none(n in 1usize..20) {
            let weights: Vec<f64> = vec![];
            let result = WeightService::calculate_moving_average(&weights, n);
            prop_assert!(result.is_none());
        }

        #[test]
        fn test_moving_average_zero_n_returns_none(
            weights in prop::collection::vec(20.0f64..500.0, 1..50)
        ) {
            let result = WeightService::calculate_moving_average(&weights, 0);
            prop_assert!(result.is_none());
        }
    }

    // Feature: fitness-assistant-ai, Property 5: Anomaly Detection Threshold
    #[test]
    fn test_anomaly_threshold_exactly_2_percent() {
        // 2% change should NOT be flagged (threshold is >2%)
        let prev_weight: f64 = 100.0;
        let new_weight: f64 = 102.0; // Exactly 2% increase
        let percent_change = ((new_weight - prev_weight) / prev_weight).abs() * 100.0;
        assert!((percent_change - 2.0).abs() < 0.0001);
        assert!(percent_change <= ANOMALY_THRESHOLD_PERCENT);
    }

    #[test]
    fn test_anomaly_threshold_above_2_percent() {
        // >2% change should be flagged
        let prev_weight: f64 = 100.0;
        let new_weight: f64 = 102.1; // 2.1% increase
        let percent_change = ((new_weight - prev_weight) / prev_weight).abs() * 100.0;
        assert!(percent_change > ANOMALY_THRESHOLD_PERCENT);
    }

    #[test]
    fn test_anomaly_threshold_below_2_percent() {
        // <2% change should NOT be flagged
        let prev_weight: f64 = 100.0;
        let new_weight: f64 = 101.5; // 1.5% increase
        let percent_change = ((new_weight - prev_weight) / prev_weight).abs() * 100.0;
        assert!(percent_change <= ANOMALY_THRESHOLD_PERCENT);
    }

    // Feature: fitness-assistant-ai, Property 4: Weight Goal Projection
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_goal_projection_formula(
            current in 50.0f64..150.0,
            target in 50.0f64..150.0,
            daily_change in -0.5f64..0.5
        ) {
            // Skip if daily change is too small
            prop_assume!(daily_change.abs() >= 0.001);

            let weight_to_lose = current - target;

            // Only test when moving toward goal
            let moving_toward_goal = if weight_to_lose > 0.0 {
                daily_change < 0.0
            } else if weight_to_lose < 0.0 {
                daily_change > 0.0
            } else {
                true
            };

            if moving_toward_goal && weight_to_lose.abs() > 0.001 {
                let days_remaining = (weight_to_lose.abs() / daily_change.abs()).ceil() as i64;

                // Verify the formula: days = |current - target| / |daily_change|
                let expected_days = (weight_to_lose.abs() / daily_change.abs()).ceil() as i64;
                prop_assert_eq!(days_remaining, expected_days);
            }
        }
    }
}
