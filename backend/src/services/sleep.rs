//! Sleep tracking service
//!
//! Provides business logic for sleep tracking including:
//! - Sleep logging with stage breakdown
//! - Sleep efficiency calculation
//! - Sleep trend analysis
//! - Sleep goal management

use crate::error::ApiError;
use crate::repositories::{
    CreateSleepLog, SleepGoalRepository, SleepLogRepository, UpsertSleepGoal,
};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
use uuid::Uuid;

/// Default sleep goal in minutes (8 hours)
const DEFAULT_SLEEP_GOAL_MINUTES: i32 = 480;

/// Sleep log entry
#[derive(Debug, Clone)]
pub struct SleepLog {
    pub id: Uuid,
    pub sleep_start: DateTime<Utc>,
    pub sleep_end: DateTime<Utc>,
    pub total_duration_minutes: i32,
    pub awake_minutes: i32,
    pub light_minutes: i32,
    pub deep_minutes: i32,
    pub rem_minutes: i32,
    pub sleep_efficiency: Option<f64>,
    pub sleep_score: Option<i32>,
    pub times_awoken: Option<i32>,
    pub avg_heart_rate: Option<i32>,
    pub min_heart_rate: Option<i32>,
    pub hrv_average: Option<f64>,
    pub respiratory_rate: Option<f64>,
    pub source: String,
    pub notes: Option<String>,
}

/// Input for logging sleep
#[derive(Debug, Clone)]
pub struct LogSleepInput {
    pub sleep_start: DateTime<Utc>,
    pub sleep_end: DateTime<Utc>,
    pub awake_minutes: Option<i32>,
    pub light_minutes: Option<i32>,
    pub deep_minutes: Option<i32>,
    pub rem_minutes: Option<i32>,
    pub sleep_score: Option<i32>,
    pub times_awoken: Option<i32>,
    pub avg_heart_rate: Option<i32>,
    pub min_heart_rate: Option<i32>,
    pub hrv_average: Option<f64>,
    pub respiratory_rate: Option<f64>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

/// Sleep analysis result
#[derive(Debug, Clone)]
pub struct SleepAnalysis {
    pub avg_duration_minutes: f64,
    pub avg_efficiency: f64,
    pub avg_deep_percent: f64,
    pub avg_rem_percent: f64,
    pub avg_light_percent: f64,
    pub avg_awake_percent: f64,
    pub total_nights: i64,
    pub sleep_debt_minutes: i64,
    pub consistency_score: f64,
}

/// Sleep goal
#[derive(Debug, Clone)]
pub struct SleepGoal {
    pub target_duration_minutes: i32,
    pub target_bedtime: Option<NaiveTime>,
    pub target_wake_time: Option<NaiveTime>,
    pub bedtime_reminder_enabled: bool,
    pub bedtime_reminder_minutes_before: Option<i32>,
}

/// Input for setting sleep goal
#[derive(Debug, Clone)]
pub struct SetSleepGoalInput {
    pub target_duration_minutes: Option<i32>,
    pub target_bedtime: Option<NaiveTime>,
    pub target_wake_time: Option<NaiveTime>,
    pub bedtime_reminder_enabled: Option<bool>,
    pub bedtime_reminder_minutes_before: Option<i32>,
}

/// Sleep service for business logic
pub struct SleepService;

impl SleepService {
    /// Log a sleep entry
    pub async fn log_sleep(
        pool: &PgPool,
        user_id: Uuid,
        input: LogSleepInput,
    ) -> Result<SleepLog, ApiError> {
        // Validate sleep times
        if input.sleep_end <= input.sleep_start {
            return Err(ApiError::Validation(
                "Sleep end time must be after start time".to_string(),
            ));
        }

        // Calculate total duration
        let duration = input.sleep_end - input.sleep_start;
        let total_duration_minutes = duration.num_minutes() as i32;

        if total_duration_minutes <= 0 {
            return Err(ApiError::Validation(
                "Sleep duration must be greater than 0".to_string(),
            ));
        }

        if total_duration_minutes > 1440 {
            return Err(ApiError::Validation(
                "Sleep duration cannot exceed 24 hours".to_string(),
            ));
        }

        // Get stage minutes (default to 0 if not provided)
        let awake_minutes = input.awake_minutes.unwrap_or(0);
        let light_minutes = input.light_minutes.unwrap_or(0);
        let deep_minutes = input.deep_minutes.unwrap_or(0);
        let rem_minutes = input.rem_minutes.unwrap_or(0);

        // Validate stage minutes
        if awake_minutes < 0 || light_minutes < 0 || deep_minutes < 0 || rem_minutes < 0 {
            return Err(ApiError::Validation(
                "Sleep stage minutes cannot be negative".to_string(),
            ));
        }

        // If stages are provided, validate they sum correctly
        let stage_sum = awake_minutes + light_minutes + deep_minutes + rem_minutes;
        if stage_sum > 0 && stage_sum != total_duration_minutes {
            // Allow some tolerance for rounding
            let diff = (stage_sum - total_duration_minutes).abs();
            if diff > 5 {
                return Err(ApiError::Validation(format!(
                    "Sleep stages ({} min) must sum to total duration ({} min)",
                    stage_sum, total_duration_minutes
                )));
            }
        }

        // Calculate sleep efficiency
        let sleep_efficiency = Self::calculate_efficiency(total_duration_minutes, awake_minutes);

        let create_input = CreateSleepLog {
            user_id,
            sleep_start: input.sleep_start,
            sleep_end: input.sleep_end,
            total_duration_minutes,
            awake_minutes,
            light_minutes,
            deep_minutes,
            rem_minutes,
            sleep_efficiency: sleep_efficiency.map(|e| Decimal::try_from(e).unwrap_or_default()),
            sleep_score: input.sleep_score,
            times_awoken: input.times_awoken,
            avg_heart_rate: input.avg_heart_rate,
            min_heart_rate: input.min_heart_rate,
            hrv_average: input.hrv_average.map(|h| Decimal::try_from(h).unwrap_or_default()),
            respiratory_rate: input.respiratory_rate.map(|r| Decimal::try_from(r).unwrap_or_default()),
            source: input.source.unwrap_or_else(|| "manual".to_string()),
            notes: input.notes,
        };

        let record = SleepLogRepository::create(pool, create_input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(Self::record_to_sleep_log(record))
    }

    /// Calculate sleep efficiency
    ///
    /// # Property 15: Sleep Efficiency Calculation
    /// efficiency = (duration - awake) / duration * 100
    pub fn calculate_efficiency(total_duration_minutes: i32, awake_minutes: i32) -> Option<f64> {
        if total_duration_minutes <= 0 {
            return None;
        }
        
        let actual_sleep = total_duration_minutes - awake_minutes;
        if actual_sleep < 0 {
            return Some(0.0);
        }
        
        Some((actual_sleep as f64 / total_duration_minutes as f64) * 100.0)
    }

    /// Validate that sleep stages sum to total duration
    ///
    /// # Property 16: Sleep Stage Time Consistency
    /// awake + light + deep + rem = total_duration
    pub fn validate_stage_consistency(
        total_duration_minutes: i32,
        awake_minutes: i32,
        light_minutes: i32,
        deep_minutes: i32,
        rem_minutes: i32,
    ) -> bool {
        let stage_sum = awake_minutes + light_minutes + deep_minutes + rem_minutes;
        // Allow 5 minute tolerance for rounding
        (stage_sum - total_duration_minutes).abs() <= 5
    }

    /// Get sleep history for a date range
    pub async fn get_history(
        pool: &PgPool,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<SleepLog>, i64), ApiError> {
        let records = SleepLogRepository::get_history(pool, user_id, start_date, end_date, limit, offset)
            .await
            .map_err(ApiError::Internal)?;

        let total = SleepLogRepository::count_in_range(pool, user_id, start_date, end_date)
            .await
            .map_err(ApiError::Internal)?;

        let logs = records.into_iter().map(Self::record_to_sleep_log).collect();

        Ok((logs, total))
    }

    /// Get sleep analysis for a date range
    pub async fn get_analysis(
        pool: &PgPool,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<SleepAnalysis, ApiError> {
        let summary = SleepLogRepository::get_summary(pool, user_id, start_date, end_date)
            .await
            .map_err(ApiError::Internal)?;

        // Get user's sleep goal for debt calculation
        let goal = Self::get_goal(pool, user_id).await?;
        let target_minutes = goal.target_duration_minutes;

        // Calculate sleep debt
        let avg_duration = summary.avg_duration_minutes.unwrap_or(0.0);
        let days = (end_date - start_date).num_days() + 1;
        let expected_sleep = target_minutes as i64 * days;
        let actual_sleep = (avg_duration * summary.total_nights as f64) as i64;
        let sleep_debt = expected_sleep - actual_sleep;

        // Calculate stage percentages
        let avg_duration_safe = if avg_duration > 0.0 { avg_duration } else { 1.0 };
        let avg_deep_percent = (summary.avg_deep_minutes.unwrap_or(0.0) / avg_duration_safe) * 100.0;
        let avg_rem_percent = (summary.avg_rem_minutes.unwrap_or(0.0) / avg_duration_safe) * 100.0;
        let avg_light_percent = (summary.avg_light_minutes.unwrap_or(0.0) / avg_duration_safe) * 100.0;
        let avg_awake_percent = (summary.avg_awake_minutes.unwrap_or(0.0) / avg_duration_safe) * 100.0;

        // Consistency score based on how close to target (simplified)
        let consistency_score = if target_minutes > 0 {
            let ratio = avg_duration / target_minutes as f64;
            (1.0 - (ratio - 1.0).abs()).max(0.0).min(1.0) * 100.0
        } else {
            0.0
        };

        Ok(SleepAnalysis {
            avg_duration_minutes: avg_duration,
            avg_efficiency: summary.avg_efficiency.unwrap_or(0.0),
            avg_deep_percent,
            avg_rem_percent,
            avg_light_percent,
            avg_awake_percent,
            total_nights: summary.total_nights,
            sleep_debt_minutes: sleep_debt.max(0),
            consistency_score,
        })
    }

    /// Get user's sleep goal
    pub async fn get_goal(pool: &PgPool, user_id: Uuid) -> Result<SleepGoal, ApiError> {
        let goal_record = SleepGoalRepository::get_by_user(pool, user_id)
            .await
            .map_err(ApiError::Internal)?;

        match goal_record {
            Some(record) => Ok(SleepGoal {
                target_duration_minutes: record.target_duration_minutes,
                target_bedtime: record.target_bedtime,
                target_wake_time: record.target_wake_time,
                bedtime_reminder_enabled: record.bedtime_reminder_enabled,
                bedtime_reminder_minutes_before: record.bedtime_reminder_minutes_before,
            }),
            None => Ok(SleepGoal {
                target_duration_minutes: DEFAULT_SLEEP_GOAL_MINUTES,
                target_bedtime: None,
                target_wake_time: None,
                bedtime_reminder_enabled: false,
                bedtime_reminder_minutes_before: None,
            }),
        }
    }

    /// Set user's sleep goal
    pub async fn set_goal(
        pool: &PgPool,
        user_id: Uuid,
        input: SetSleepGoalInput,
    ) -> Result<SleepGoal, ApiError> {
        let target_duration = input.target_duration_minutes.unwrap_or(DEFAULT_SLEEP_GOAL_MINUTES);

        // Validate duration
        if target_duration < 60 || target_duration > 1440 {
            return Err(ApiError::Validation(
                "Target sleep duration must be between 1 and 24 hours".to_string(),
            ));
        }

        let upsert_input = UpsertSleepGoal {
            user_id,
            target_duration_minutes: target_duration,
            target_bedtime: input.target_bedtime,
            target_wake_time: input.target_wake_time,
            bedtime_reminder_enabled: input.bedtime_reminder_enabled.unwrap_or(false),
            bedtime_reminder_minutes_before: input.bedtime_reminder_minutes_before,
        };

        let record = SleepGoalRepository::upsert(pool, upsert_input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(SleepGoal {
            target_duration_minutes: record.target_duration_minutes,
            target_bedtime: record.target_bedtime,
            target_wake_time: record.target_wake_time,
            bedtime_reminder_enabled: record.bedtime_reminder_enabled,
            bedtime_reminder_minutes_before: record.bedtime_reminder_minutes_before,
        })
    }

    /// Delete a sleep log entry
    pub async fn delete_log(
        pool: &PgPool,
        user_id: Uuid,
        log_id: Uuid,
    ) -> Result<bool, ApiError> {
        SleepLogRepository::delete(pool, log_id, user_id)
            .await
            .map_err(ApiError::Internal)
    }

    /// Convert database record to domain model
    fn record_to_sleep_log(record: crate::repositories::sleep::SleepLogRecord) -> SleepLog {
        SleepLog {
            id: record.id,
            sleep_start: record.sleep_start,
            sleep_end: record.sleep_end,
            total_duration_minutes: record.total_duration_minutes,
            awake_minutes: record.awake_minutes,
            light_minutes: record.light_minutes,
            deep_minutes: record.deep_minutes,
            rem_minutes: record.rem_minutes,
            sleep_efficiency: record.sleep_efficiency.and_then(|d| d.to_f64()),
            sleep_score: record.sleep_score,
            times_awoken: record.times_awoken,
            avg_heart_rate: record.avg_heart_rate,
            min_heart_rate: record.min_heart_rate,
            hrv_average: record.hrv_average.and_then(|d| d.to_f64()),
            respiratory_rate: record.respiratory_rate.and_then(|d| d.to_f64()),
            source: record.source,
            notes: record.notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: fitness-assistant-ai, Property 15: Sleep Efficiency Calculation
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_efficiency_calculation(
            total_duration in 60i32..1440,
            awake_percent in 0u32..100
        ) {
            let awake_minutes = (total_duration as f64 * awake_percent as f64 / 100.0) as i32;
            let efficiency = SleepService::calculate_efficiency(total_duration, awake_minutes);
            
            prop_assert!(efficiency.is_some());
            let eff = efficiency.unwrap();
            
            // Expected: (duration - awake) / duration * 100
            let expected = ((total_duration - awake_minutes) as f64 / total_duration as f64) * 100.0;
            
            prop_assert!((eff - expected).abs() < 0.0001,
                "Efficiency {} != expected {} for duration={}, awake={}",
                eff, expected, total_duration, awake_minutes);
        }

        #[test]
        fn test_efficiency_bounds(
            total_duration in 60i32..1440,
            awake_minutes in 0i32..1440
        ) {
            let awake = awake_minutes.min(total_duration);
            let efficiency = SleepService::calculate_efficiency(total_duration, awake);
            
            prop_assert!(efficiency.is_some());
            let eff = efficiency.unwrap();
            
            // Efficiency should be between 0 and 100
            prop_assert!(eff >= 0.0 && eff <= 100.0,
                "Efficiency {} out of bounds for duration={}, awake={}",
                eff, total_duration, awake);
        }

        #[test]
        fn test_efficiency_zero_awake_is_100(total_duration in 60i32..1440) {
            let efficiency = SleepService::calculate_efficiency(total_duration, 0);
            
            prop_assert!(efficiency.is_some());
            prop_assert!((efficiency.unwrap() - 100.0).abs() < 0.0001,
                "Efficiency should be 100% when awake=0");
        }

        #[test]
        fn test_efficiency_all_awake_is_zero(total_duration in 60i32..1440) {
            let efficiency = SleepService::calculate_efficiency(total_duration, total_duration);
            
            prop_assert!(efficiency.is_some());
            prop_assert!(efficiency.unwrap().abs() < 0.0001,
                "Efficiency should be 0% when all time is awake");
        }

        #[test]
        fn test_efficiency_zero_duration_returns_none(awake_minutes in 0i32..100) {
            let efficiency = SleepService::calculate_efficiency(0, awake_minutes);
            prop_assert!(efficiency.is_none());
        }

        #[test]
        fn test_efficiency_negative_duration_returns_none(awake_minutes in 0i32..100) {
            let efficiency = SleepService::calculate_efficiency(-60, awake_minutes);
            prop_assert!(efficiency.is_none());
        }
    }

    // Feature: fitness-assistant-ai, Property 16: Sleep Stage Time Consistency
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_stage_consistency_valid(
            total_duration in 60i32..1440
        ) {
            // Generate stages that sum to total
            let awake = total_duration / 10;
            let light = total_duration / 2;
            let deep = total_duration / 5;
            let rem = total_duration - awake - light - deep;
            
            let is_valid = SleepService::validate_stage_consistency(
                total_duration, awake, light, deep, rem
            );
            
            prop_assert!(is_valid,
                "Stages should be valid when they sum to total: {} + {} + {} + {} = {} vs {}",
                awake, light, deep, rem, awake + light + deep + rem, total_duration);
        }

        #[test]
        fn test_stage_consistency_with_tolerance(
            total_duration in 60i32..1440,
            tolerance in 0i32..=5
        ) {
            // Generate stages that sum to total + tolerance
            let awake = total_duration / 10;
            let light = total_duration / 2;
            let deep = total_duration / 5;
            let rem = total_duration - awake - light - deep + tolerance;
            
            let is_valid = SleepService::validate_stage_consistency(
                total_duration, awake, light, deep, rem
            );
            
            prop_assert!(is_valid,
                "Stages should be valid within 5 min tolerance");
        }

        #[test]
        fn test_stage_consistency_invalid_large_diff(
            total_duration in 60i32..1440
        ) {
            // Generate stages that sum to total + 10 (outside tolerance)
            let awake = total_duration / 10;
            let light = total_duration / 2;
            let deep = total_duration / 5;
            let rem = total_duration - awake - light - deep + 10;
            
            let is_valid = SleepService::validate_stage_consistency(
                total_duration, awake, light, deep, rem
            );
            
            prop_assert!(!is_valid,
                "Stages should be invalid when diff > 5 minutes");
        }

        #[test]
        fn test_stage_sum_equals_total(
            awake in 0i32..100,
            light in 0i32..300,
            deep in 0i32..200,
            rem in 0i32..200
        ) {
            let total = awake + light + deep + rem;
            
            let is_valid = SleepService::validate_stage_consistency(
                total, awake, light, deep, rem
            );
            
            prop_assert!(is_valid,
                "Stages should always be valid when sum equals total");
        }
    }

    #[test]
    fn test_efficiency_typical_values() {
        // 8 hours sleep, 30 min awake = 93.75% efficiency
        let eff = SleepService::calculate_efficiency(480, 30).unwrap();
        assert!((eff - 93.75).abs() < 0.01);
        
        // 7 hours sleep, 1 hour awake = 85.7% efficiency
        let eff = SleepService::calculate_efficiency(420, 60).unwrap();
        assert!((eff - 85.714).abs() < 0.01);
    }

    #[test]
    fn test_stage_consistency_exact_match() {
        assert!(SleepService::validate_stage_consistency(480, 30, 240, 120, 90));
    }

    #[test]
    fn test_stage_consistency_within_tolerance() {
        // Sum is 483, total is 480, diff is 3 (within 5)
        assert!(SleepService::validate_stage_consistency(480, 30, 243, 120, 90));
    }

    #[test]
    fn test_stage_consistency_outside_tolerance() {
        // Sum is 490, total is 480, diff is 10 (outside 5)
        assert!(!SleepService::validate_stage_consistency(480, 30, 250, 120, 90));
    }
}
