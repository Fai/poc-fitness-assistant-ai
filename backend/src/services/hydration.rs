//! Hydration tracking service
//!
//! Provides business logic for hydration tracking including:
//! - Water intake logging
//! - Daily progress calculation
//! - Personalized goal calculation based on weight
//! - Goal completion detection

use crate::error::ApiError;
use crate::repositories::{
    CreateHydrationLog, HydrationGoalRepository, HydrationLogRepository, UpsertHydrationGoal,
    WeightRepository,
};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Default hydration goal in ml (2500ml = ~10 cups)
const DEFAULT_HYDRATION_GOAL_ML: i32 = 2500;

/// Hydration multiplier: ml per kg of body weight
/// Standard recommendation is 30-35ml per kg
const HYDRATION_ML_PER_KG: f64 = 33.0;

/// Activity level multipliers for hydration
const ACTIVITY_MULTIPLIERS: &[(&str, f64)] = &[
    ("sedentary", 1.0),
    ("lightly_active", 1.1),
    ("moderately_active", 1.2),
    ("very_active", 1.3),
    ("extra_active", 1.4),
];

/// Hydration log entry
#[derive(Debug, Clone)]
pub struct HydrationLog {
    pub id: Uuid,
    pub amount_ml: i32,
    pub beverage_type: String,
    pub consumed_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
}

/// Input for logging water intake
#[derive(Debug, Clone)]
pub struct LogHydrationInput {
    pub amount_ml: i32,
    pub beverage_type: Option<String>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

/// Daily hydration summary
#[derive(Debug, Clone)]
pub struct DailyHydrationSummary {
    pub date: NaiveDate,
    pub total_ml: i64,
    pub goal_ml: i32,
    pub progress_percent: f64,
    pub goal_met: bool,
    pub entry_count: i64,
    pub entries: Vec<HydrationLog>,
}

/// Hydration goal
#[derive(Debug, Clone)]
pub struct HydrationGoal {
    pub daily_goal_ml: i32,
    pub is_auto_calculated: bool,
    pub reminders_enabled: bool,
    pub reminder_interval_minutes: Option<i32>,
    pub reminder_start_time: Option<NaiveTime>,
    pub reminder_end_time: Option<NaiveTime>,
}

/// Input for setting hydration goal
#[derive(Debug, Clone)]
pub struct SetHydrationGoalInput {
    pub daily_goal_ml: Option<i32>,
    pub auto_calculate: bool,
    pub reminders_enabled: Option<bool>,
    pub reminder_interval_minutes: Option<i32>,
    pub reminder_start_time: Option<NaiveTime>,
    pub reminder_end_time: Option<NaiveTime>,
}

/// Hydration service for business logic
pub struct HydrationService;

impl HydrationService {
    /// Log water intake
    pub async fn log_hydration(
        pool: &PgPool,
        user_id: Uuid,
        input: LogHydrationInput,
    ) -> Result<HydrationLog, ApiError> {
        // Validate amount
        if input.amount_ml <= 0 {
            return Err(ApiError::Validation(
                "Amount must be greater than 0".to_string(),
            ));
        }
        if input.amount_ml > 10000 {
            return Err(ApiError::Validation(
                "Amount cannot exceed 10000ml".to_string(),
            ));
        }

        let create_input = CreateHydrationLog {
            user_id,
            amount_ml: input.amount_ml,
            beverage_type: input.beverage_type.unwrap_or_else(|| "water".to_string()),
            consumed_at: input.consumed_at.unwrap_or_else(Utc::now),
            source: input.source.unwrap_or_else(|| "manual".to_string()),
            notes: input.notes,
        };

        let record = HydrationLogRepository::create(pool, create_input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(HydrationLog {
            id: record.id,
            amount_ml: record.amount_ml,
            beverage_type: record.beverage_type,
            consumed_at: record.consumed_at,
            source: record.source,
            notes: record.notes,
        })
    }

    /// Get daily hydration summary with progress
    ///
    /// # Property 11: Hydration Progress Calculation
    /// progress = (consumed / goal) * 100
    pub async fn get_daily_summary(
        pool: &PgPool,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<DailyHydrationSummary, ApiError> {
        // Get the user's goal
        let goal_ml = Self::get_effective_goal(pool, user_id).await?;

        // Get daily summary from repository
        let summary = HydrationLogRepository::get_daily_summary(pool, user_id, date)
            .await
            .map_err(ApiError::Internal)?;

        // Get individual entries
        let entries = HydrationLogRepository::get_by_date(pool, user_id, date)
            .await
            .map_err(ApiError::Internal)?
            .into_iter()
            .map(|r| HydrationLog {
                id: r.id,
                amount_ml: r.amount_ml,
                beverage_type: r.beverage_type,
                consumed_at: r.consumed_at,
                source: r.source,
                notes: r.notes,
            })
            .collect();

        // Calculate progress
        let progress_percent = Self::calculate_progress(summary.total_ml, goal_ml);
        let goal_met = Self::is_goal_met(summary.total_ml, goal_ml);

        Ok(DailyHydrationSummary {
            date: summary.date,
            total_ml: summary.total_ml,
            goal_ml,
            progress_percent,
            goal_met,
            entry_count: summary.entry_count,
            entries,
        })
    }

    /// Calculate progress percentage
    ///
    /// # Property 11: Hydration Progress Calculation
    /// progress = (consumed / goal) * 100
    pub fn calculate_progress(consumed_ml: i64, goal_ml: i32) -> f64 {
        if goal_ml <= 0 {
            return 0.0;
        }
        (consumed_ml as f64 / goal_ml as f64) * 100.0
    }

    /// Check if goal is met (>=100%)
    ///
    /// # Property 13: Hydration Goal Completion Detection
    pub fn is_goal_met(consumed_ml: i64, goal_ml: i32) -> bool {
        if goal_ml <= 0 {
            return false;
        }
        consumed_ml >= goal_ml as i64
    }

    /// Get user's hydration goal
    pub async fn get_goal(pool: &PgPool, user_id: Uuid) -> Result<HydrationGoal, ApiError> {
        let goal_record = HydrationGoalRepository::get_by_user(pool, user_id)
            .await
            .map_err(ApiError::Internal)?;

        match goal_record {
            Some(record) => Ok(HydrationGoal {
                daily_goal_ml: record.daily_goal_ml,
                is_auto_calculated: record.is_auto_calculated,
                reminders_enabled: record.reminders_enabled,
                reminder_interval_minutes: record.reminder_interval_minutes,
                reminder_start_time: record.reminder_start_time,
                reminder_end_time: record.reminder_end_time,
            }),
            None => {
                // Return default goal if none set
                let auto_goal = Self::calculate_personalized_goal(pool, user_id).await?;
                Ok(HydrationGoal {
                    daily_goal_ml: auto_goal,
                    is_auto_calculated: true,
                    reminders_enabled: false,
                    reminder_interval_minutes: None,
                    reminder_start_time: None,
                    reminder_end_time: None,
                })
            }
        }
    }

    /// Set user's hydration goal
    pub async fn set_goal(
        pool: &PgPool,
        user_id: Uuid,
        input: SetHydrationGoalInput,
    ) -> Result<HydrationGoal, ApiError> {
        let daily_goal_ml = if input.auto_calculate {
            Self::calculate_personalized_goal(pool, user_id).await?
        } else {
            input.daily_goal_ml.unwrap_or(DEFAULT_HYDRATION_GOAL_ML)
        };

        // Validate goal
        if daily_goal_ml <= 0 || daily_goal_ml > 20000 {
            return Err(ApiError::Validation(
                "Daily goal must be between 1 and 20000ml".to_string(),
            ));
        }

        let upsert_input = UpsertHydrationGoal {
            user_id,
            daily_goal_ml,
            is_auto_calculated: input.auto_calculate,
            reminders_enabled: input.reminders_enabled.unwrap_or(false),
            reminder_interval_minutes: input.reminder_interval_minutes,
            reminder_start_time: input.reminder_start_time,
            reminder_end_time: input.reminder_end_time,
        };

        let record = HydrationGoalRepository::upsert(pool, upsert_input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(HydrationGoal {
            daily_goal_ml: record.daily_goal_ml,
            is_auto_calculated: record.is_auto_calculated,
            reminders_enabled: record.reminders_enabled,
            reminder_interval_minutes: record.reminder_interval_minutes,
            reminder_start_time: record.reminder_start_time,
            reminder_end_time: record.reminder_end_time,
        })
    }

    /// Calculate personalized hydration goal based on weight and activity level
    ///
    /// # Property 12: Personalized Hydration Goal
    /// goal = weight_kg * 33ml * activity_multiplier
    pub async fn calculate_personalized_goal(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<i32, ApiError> {
        // Get user's latest weight
        let latest_weight = WeightRepository::get_latest(pool, user_id)
            .await
            .map_err(ApiError::Internal)?;

        // Get user's activity level from settings
        let settings = crate::repositories::UserRepository::get_settings(pool, user_id)
            .await
            .map_err(ApiError::Internal)?;

        let weight_kg = latest_weight
            .map(|w| w.weight_kg.to_string().parse::<f64>().unwrap_or(70.0))
            .unwrap_or(70.0); // Default to 70kg if no weight recorded

        let activity_level = settings
            .map(|s| s.activity_level)
            .unwrap_or_else(|| "moderately_active".to_string());

        Ok(Self::calculate_goal_from_weight(weight_kg, &activity_level))
    }

    /// Calculate hydration goal from weight and activity level
    ///
    /// # Property 12: Personalized Hydration Goal
    /// goal = weight_kg * 33ml * activity_multiplier
    pub fn calculate_goal_from_weight(weight_kg: f64, activity_level: &str) -> i32 {
        let activity_multiplier = ACTIVITY_MULTIPLIERS
            .iter()
            .find(|(level, _)| *level == activity_level)
            .map(|(_, mult)| *mult)
            .unwrap_or(1.2); // Default to moderately active

        let goal = weight_kg * HYDRATION_ML_PER_KG * activity_multiplier;
        
        // Round to nearest 100ml for cleaner goals
        ((goal / 100.0).round() * 100.0) as i32
    }

    /// Get effective goal (from settings or calculated)
    async fn get_effective_goal(pool: &PgPool, user_id: Uuid) -> Result<i32, ApiError> {
        let goal_record = HydrationGoalRepository::get_by_user(pool, user_id)
            .await
            .map_err(ApiError::Internal)?;

        match goal_record {
            Some(record) if !record.is_auto_calculated => Ok(record.daily_goal_ml),
            _ => Self::calculate_personalized_goal(pool, user_id).await,
        }
    }

    /// Delete a hydration log entry
    pub async fn delete_log(
        pool: &PgPool,
        user_id: Uuid,
        log_id: Uuid,
    ) -> Result<bool, ApiError> {
        HydrationLogRepository::delete(pool, log_id, user_id)
            .await
            .map_err(ApiError::Internal)
    }

    /// Get hydration history for a date range
    pub async fn get_history(
        pool: &PgPool,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DailyHydrationSummary>, ApiError> {
        let goal_ml = Self::get_effective_goal(pool, user_id).await?;

        let summaries = HydrationLogRepository::get_daily_summaries(pool, user_id, start_date, end_date)
            .await
            .map_err(ApiError::Internal)?;

        Ok(summaries
            .into_iter()
            .map(|s| {
                let progress_percent = Self::calculate_progress(s.total_ml, goal_ml);
                let goal_met = Self::is_goal_met(s.total_ml, goal_ml);
                DailyHydrationSummary {
                    date: s.date,
                    total_ml: s.total_ml,
                    goal_ml,
                    progress_percent,
                    goal_met,
                    entry_count: s.entry_count,
                    entries: vec![], // Don't include entries in history view
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: fitness-assistant-ai, Property 11: Hydration Progress Calculation
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_progress_calculation(
            consumed_ml in 0i64..20000,
            goal_ml in 1i32..10000
        ) {
            let progress = HydrationService::calculate_progress(consumed_ml, goal_ml);
            let expected = (consumed_ml as f64 / goal_ml as f64) * 100.0;

            prop_assert!((progress - expected).abs() < 0.0001,
                "Progress {} != expected {} for consumed={}, goal={}",
                progress, expected, consumed_ml, goal_ml);
        }

        #[test]
        fn test_progress_zero_goal_returns_zero(consumed_ml in 0i64..20000) {
            let progress = HydrationService::calculate_progress(consumed_ml, 0);
            prop_assert_eq!(progress, 0.0);
        }

        #[test]
        fn test_progress_negative_goal_returns_zero(consumed_ml in 0i64..20000) {
            let progress = HydrationService::calculate_progress(consumed_ml, -100);
            prop_assert_eq!(progress, 0.0);
        }
    }

    // Feature: fitness-assistant-ai, Property 12: Personalized Hydration Goal
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_personalized_goal_formula(
            weight_kg in 40.0f64..200.0
        ) {
            let goal = HydrationService::calculate_goal_from_weight(weight_kg, "moderately_active");
            
            // Expected: weight * 33 * 1.2 (moderately active multiplier), rounded to 100
            let expected_raw = weight_kg * 33.0 * 1.2;
            let expected = ((expected_raw / 100.0).round() * 100.0) as i32;

            prop_assert_eq!(goal, expected,
                "Goal {} != expected {} for weight={}kg",
                goal, expected, weight_kg);
        }

        #[test]
        fn test_goal_increases_with_activity(weight_kg in 50.0f64..100.0) {
            let sedentary = HydrationService::calculate_goal_from_weight(weight_kg, "sedentary");
            let light = HydrationService::calculate_goal_from_weight(weight_kg, "lightly_active");
            let moderate = HydrationService::calculate_goal_from_weight(weight_kg, "moderately_active");
            let very = HydrationService::calculate_goal_from_weight(weight_kg, "very_active");
            let extra = HydrationService::calculate_goal_from_weight(weight_kg, "extra_active");

            prop_assert!(sedentary <= light, "sedentary {} > light {}", sedentary, light);
            prop_assert!(light <= moderate, "light {} > moderate {}", light, moderate);
            prop_assert!(moderate <= very, "moderate {} > very {}", moderate, very);
            prop_assert!(very <= extra, "very {} > extra {}", very, extra);
        }

        #[test]
        fn test_goal_increases_with_weight(activity in prop::sample::select(vec![
            "sedentary", "lightly_active", "moderately_active", "very_active", "extra_active"
        ])) {
            let goal_50 = HydrationService::calculate_goal_from_weight(50.0, activity);
            let goal_100 = HydrationService::calculate_goal_from_weight(100.0, activity);

            prop_assert!(goal_50 < goal_100,
                "Goal for 50kg ({}) >= goal for 100kg ({}) at activity {}",
                goal_50, goal_100, activity);
        }
    }

    // Feature: fitness-assistant-ai, Property 13: Hydration Goal Completion Detection
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_goal_met_at_100_percent(goal_ml in 1i32..10000) {
            let consumed = goal_ml as i64;
            prop_assert!(HydrationService::is_goal_met(consumed, goal_ml),
                "Goal not met at exactly 100% for goal={}", goal_ml);
        }

        #[test]
        fn test_goal_met_above_100_percent(
            goal_ml in 1i32..10000,
            extra in 1i64..5000
        ) {
            let consumed = goal_ml as i64 + extra;
            prop_assert!(HydrationService::is_goal_met(consumed, goal_ml),
                "Goal not met above 100% for consumed={}, goal={}", consumed, goal_ml);
        }

        #[test]
        fn test_goal_not_met_below_100_percent(goal_ml in 2i32..10000) {
            let consumed = (goal_ml - 1) as i64;
            prop_assert!(!HydrationService::is_goal_met(consumed, goal_ml),
                "Goal incorrectly met below 100% for consumed={}, goal={}", consumed, goal_ml);
        }

        #[test]
        fn test_goal_not_met_zero_goal(consumed_ml in 0i64..20000) {
            prop_assert!(!HydrationService::is_goal_met(consumed_ml, 0),
                "Goal incorrectly met with zero goal");
        }
    }

    #[test]
    fn test_default_activity_multiplier() {
        // Unknown activity level should default to moderately_active (1.2)
        let goal = HydrationService::calculate_goal_from_weight(70.0, "unknown");
        let expected = HydrationService::calculate_goal_from_weight(70.0, "moderately_active");
        assert_eq!(goal, expected);
    }

    #[test]
    fn test_goal_rounded_to_100() {
        // 70kg * 33 * 1.0 = 2310, should round to 2300
        let goal = HydrationService::calculate_goal_from_weight(70.0, "sedentary");
        assert_eq!(goal % 100, 0, "Goal {} not rounded to 100ml", goal);
    }
}
