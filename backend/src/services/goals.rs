//! Goals service for goal setting and progress tracking
//!
//! Provides business logic for:
//! - Goal creation and management
//! - Progress calculation for increasing/decreasing goals
//! - Milestone detection and recording
//! - Goal history preservation

use crate::error::ApiError;
use crate::repositories::goals::{
    CreateGoal, CreateMilestone, GoalRepository, MilestoneRepository, UpdateGoal,
};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
use uuid::Uuid;

/// Standard milestone percentages
const MILESTONE_PERCENTAGES: &[i32] = &[25, 50, 75, 100];

/// Goal entry
#[derive(Debug, Clone)]
pub struct Goal {
    pub id: Uuid,
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
}

/// Input for creating a goal
#[derive(Debug, Clone)]
pub struct CreateGoalInput {
    pub name: String,
    pub description: Option<String>,
    pub goal_type: String,
    pub metric: String,
    pub target_value: f64,
    pub start_value: Option<f64>,
    pub direction: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub target_date: Option<NaiveDate>,
}

/// Input for updating a goal
#[derive(Debug, Clone)]
pub struct UpdateGoalInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub target_value: Option<f64>,
    pub current_value: Option<f64>,
    pub target_date: Option<NaiveDate>,
    pub status: Option<String>,
}

/// Goal progress information
#[derive(Debug, Clone)]
pub struct GoalProgress {
    pub goal_id: Uuid,
    pub progress_percent: f64,
    pub remaining: f64,
    pub on_track: bool,
    pub days_remaining: Option<i64>,
    pub projected_completion: Option<NaiveDate>,
    pub milestones: Vec<Milestone>,
}

/// Milestone entry
#[derive(Debug, Clone)]
pub struct Milestone {
    pub id: Uuid,
    pub name: String,
    pub target_value: f64,
    pub percentage: i32,
    pub achieved: bool,
    pub actual_value: Option<f64>,
}

/// Goals service for business logic
pub struct GoalsService;

impl GoalsService {
    /// Create a new goal
    pub async fn create_goal(
        pool: &PgPool,
        user_id: Uuid,
        input: CreateGoalInput,
    ) -> Result<Goal, ApiError> {
        // Validate goal type
        let valid_types = ["weight", "exercise", "nutrition", "hydration", "sleep", "custom"];
        if !valid_types.contains(&input.goal_type.as_str()) {
            return Err(ApiError::Validation(format!(
                "Invalid goal type. Must be one of: {}",
                valid_types.join(", ")
            )));
        }

        // Determine direction based on goal type if not specified
        let direction = input.direction.unwrap_or_else(|| {
            if input.goal_type == "weight" {
                "decreasing".to_string()
            } else {
                "increasing".to_string()
            }
        });

        if direction != "increasing" && direction != "decreasing" {
            return Err(ApiError::Validation(
                "Direction must be 'increasing' or 'decreasing'".to_string(),
            ));
        }

        let create_input = CreateGoal {
            user_id,
            name: input.name,
            description: input.description,
            goal_type: input.goal_type,
            metric: input.metric,
            target_value: Decimal::try_from(input.target_value).unwrap_or_default(),
            start_value: input.start_value.map(|v| Decimal::try_from(v).unwrap_or_default()),
            direction,
            start_date: input.start_date.unwrap_or_else(|| Utc::now().date_naive()),
            target_date: input.target_date,
        };

        let record = GoalRepository::create(pool, create_input)
            .await
            .map_err(ApiError::Internal)?;

        // Create default milestones
        Self::create_default_milestones(pool, &record).await?;

        Ok(Self::record_to_goal(record))
    }

    /// Create default milestones for a goal
    async fn create_default_milestones(
        pool: &PgPool,
        goal: &crate::repositories::goals::GoalRecord,
    ) -> Result<(), ApiError> {
        let start = goal.start_value.and_then(|v| v.to_f64()).unwrap_or(0.0);
        let target = goal.target_value.to_f64().unwrap_or(0.0);
        let diff = target - start;

        for &pct in MILESTONE_PERCENTAGES {
            let milestone_value = start + (diff * pct as f64 / 100.0);
            let name = format!("{}% Complete", pct);

            let input = CreateMilestone {
                goal_id: goal.id,
                name,
                target_value: Decimal::try_from(milestone_value).unwrap_or_default(),
                percentage: pct,
            };

            MilestoneRepository::create(pool, input)
                .await
                .map_err(ApiError::Internal)?;
        }

        Ok(())
    }

    /// Get all goals for a user
    pub async fn get_goals(
        pool: &PgPool,
        user_id: Uuid,
        status: Option<&str>,
        goal_type: Option<&str>,
    ) -> Result<Vec<Goal>, ApiError> {
        let records = GoalRepository::get_by_user(pool, user_id, status, goal_type)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records.into_iter().map(Self::record_to_goal).collect())
    }

    /// Get a specific goal
    pub async fn get_goal(
        pool: &PgPool,
        user_id: Uuid,
        goal_id: Uuid,
    ) -> Result<Goal, ApiError> {
        let record = GoalRepository::get_by_id(pool, goal_id, user_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("Goal not found".to_string()))?;

        Ok(Self::record_to_goal(record))
    }

    /// Update a goal
    pub async fn update_goal(
        pool: &PgPool,
        user_id: Uuid,
        goal_id: Uuid,
        input: UpdateGoalInput,
    ) -> Result<Goal, ApiError> {
        // Validate status if provided
        if let Some(ref status) = input.status {
            let valid_statuses = ["active", "completed", "abandoned", "paused"];
            if !valid_statuses.contains(&status.as_str()) {
                return Err(ApiError::Validation(format!(
                    "Invalid status. Must be one of: {}",
                    valid_statuses.join(", ")
                )));
            }
        }

        let updates = UpdateGoal {
            name: input.name,
            description: input.description,
            target_value: input.target_value.map(|v| Decimal::try_from(v).unwrap_or_default()),
            current_value: input.current_value.map(|v| Decimal::try_from(v).unwrap_or_default()),
            target_date: input.target_date,
            status: input.status,
        };

        let record = GoalRepository::update(pool, goal_id, user_id, updates)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("Goal not found".to_string()))?;

        // Check and update milestones if current_value changed
        if input.current_value.is_some() {
            Self::check_milestones(pool, &record).await?;
        }

        Ok(Self::record_to_goal(record))
    }

    /// Check and update milestones based on current progress
    async fn check_milestones(
        pool: &PgPool,
        goal: &crate::repositories::goals::GoalRecord,
    ) -> Result<(), ApiError> {
        let current = goal.current_value.and_then(|v| v.to_f64()).unwrap_or(0.0);
        let milestones = MilestoneRepository::get_by_goal(pool, goal.id)
            .await
            .map_err(ApiError::Internal)?;

        for milestone in milestones {
            if milestone.achieved_at.is_some() {
                continue; // Already achieved
            }

            let target = milestone.target_value.to_f64().unwrap_or(0.0);
            let achieved = if goal.direction == "increasing" {
                current >= target
            } else {
                current <= target
            };

            if achieved {
                MilestoneRepository::achieve(
                    pool,
                    milestone.id,
                    Decimal::try_from(current).unwrap_or_default(),
                )
                .await
                .map_err(ApiError::Internal)?;
            }
        }

        Ok(())
    }

    /// Get goal progress
    ///
    /// # Property 22: Goal Progress Calculation
    /// For increasing goals: progress = (current - start) / (target - start) * 100
    /// For decreasing goals: progress = (start - current) / (start - target) * 100
    pub async fn get_progress(
        pool: &PgPool,
        user_id: Uuid,
        goal_id: Uuid,
    ) -> Result<GoalProgress, ApiError> {
        let goal = GoalRepository::get_by_id(pool, goal_id, user_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("Goal not found".to_string()))?;

        let start = goal.start_value.and_then(|v| v.to_f64()).unwrap_or(0.0);
        let current = goal.current_value.and_then(|v| v.to_f64()).unwrap_or(start);
        let target = goal.target_value.to_f64().unwrap_or(0.0);

        let progress_percent = Self::calculate_progress(start, current, target, &goal.direction);
        let remaining = Self::calculate_remaining(current, target, &goal.direction);

        // Calculate days remaining and projected completion
        let today = Utc::now().date_naive();
        let days_remaining = goal.target_date.map(|d| (d - today).num_days());
        
        let on_track = goal.target_date.map_or(true, |target_date| {
            let days_elapsed = (today - goal.start_date).num_days();
            let total_days = (target_date - goal.start_date).num_days();
            if total_days <= 0 {
                return progress_percent >= 100.0;
            }
            let expected_progress = (days_elapsed as f64 / total_days as f64) * 100.0;
            progress_percent >= expected_progress
        });

        // Get milestones
        let milestone_records = MilestoneRepository::get_by_goal(pool, goal.id)
            .await
            .map_err(ApiError::Internal)?;

        let milestones = milestone_records
            .into_iter()
            .map(|m| Milestone {
                id: m.id,
                name: m.name,
                target_value: m.target_value.to_f64().unwrap_or(0.0),
                percentage: m.percentage,
                achieved: m.achieved_at.is_some(),
                actual_value: m.actual_value.and_then(|v| v.to_f64()),
            })
            .collect();

        Ok(GoalProgress {
            goal_id,
            progress_percent,
            remaining,
            on_track,
            days_remaining,
            projected_completion: None, // Would need rate calculation
            milestones,
        })
    }

    /// Calculate progress percentage
    ///
    /// # Property 22: Goal Progress Calculation
    pub fn calculate_progress(start: f64, current: f64, target: f64, direction: &str) -> f64 {
        let total_change = (target - start).abs();
        if total_change == 0.0 {
            return if current == target { 100.0 } else { 0.0 };
        }

        let actual_change = if direction == "increasing" {
            current - start
        } else {
            start - current
        };

        let progress = (actual_change / total_change) * 100.0;
        progress.clamp(0.0, 100.0)
    }

    /// Calculate remaining amount to goal
    pub fn calculate_remaining(current: f64, target: f64, direction: &str) -> f64 {
        if direction == "increasing" {
            (target - current).max(0.0)
        } else {
            (current - target).max(0.0)
        }
    }

    /// Delete a goal
    pub async fn delete_goal(
        pool: &PgPool,
        user_id: Uuid,
        goal_id: Uuid,
    ) -> Result<bool, ApiError> {
        GoalRepository::delete(pool, goal_id, user_id)
            .await
            .map_err(ApiError::Internal)
    }

    /// Convert database record to domain model
    fn record_to_goal(record: crate::repositories::goals::GoalRecord) -> Goal {
        Goal {
            id: record.id,
            name: record.name,
            description: record.description,
            goal_type: record.goal_type,
            metric: record.metric,
            target_value: record.target_value.to_f64().unwrap_or(0.0),
            start_value: record.start_value.and_then(|v| v.to_f64()),
            current_value: record.current_value.and_then(|v| v.to_f64()),
            direction: record.direction,
            start_date: record.start_date,
            target_date: record.target_date,
            status: record.status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: fitness-assistant-ai, Property 22: Goal Progress Calculation
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_progress_increasing_goal(
            start in 0.0f64..100.0,
            progress_pct in 0.0f64..100.0
        ) {
            let target = start + 50.0; // Always 50 units to gain
            let current = start + (50.0 * progress_pct / 100.0);
            
            let progress = GoalsService::calculate_progress(start, current, target, "increasing");
            
            prop_assert!((progress - progress_pct).abs() < 0.01,
                "Progress {} != expected {} for start={}, current={}, target={}",
                progress, progress_pct, start, current, target);
        }

        #[test]
        fn test_progress_decreasing_goal(
            start in 50.0f64..150.0,
            progress_pct in 0.0f64..100.0
        ) {
            let target = start - 30.0; // Always 30 units to lose
            let current = start - (30.0 * progress_pct / 100.0);
            
            let progress = GoalsService::calculate_progress(start, current, target, "decreasing");
            
            prop_assert!((progress - progress_pct).abs() < 0.01,
                "Progress {} != expected {} for start={}, current={}, target={}",
                progress, progress_pct, start, current, target);
        }

        #[test]
        fn test_progress_at_start_is_zero(
            start in 0.0f64..100.0,
            target in 100.0f64..200.0
        ) {
            let progress = GoalsService::calculate_progress(start, start, target, "increasing");
            prop_assert_eq!(progress, 0.0);
        }

        #[test]
        fn test_progress_at_target_is_100(
            start in 0.0f64..100.0,
            target in 100.0f64..200.0
        ) {
            let progress = GoalsService::calculate_progress(start, target, target, "increasing");
            prop_assert_eq!(progress, 100.0);
        }

        #[test]
        fn test_progress_clamped_to_100(
            start in 0.0f64..100.0,
            overshoot in 1.0f64..50.0
        ) {
            let target = start + 50.0;
            let current = target + overshoot; // Exceeded goal
            
            let progress = GoalsService::calculate_progress(start, current, target, "increasing");
            prop_assert_eq!(progress, 100.0);
        }

        #[test]
        fn test_progress_clamped_to_zero(
            start in 50.0f64..100.0,
            regression in 1.0f64..20.0
        ) {
            let target = start + 50.0;
            let current = start - regression; // Went backwards
            
            let progress = GoalsService::calculate_progress(start, current, target, "increasing");
            prop_assert_eq!(progress, 0.0);
        }
    }

    // Feature: fitness-assistant-ai, Property 23: Milestone Achievement Recording
    // (Tested via integration tests as it requires database)

    // Feature: fitness-assistant-ai, Property 24: Goal History Preservation
    // (Tested via integration tests as it requires database)

    #[test]
    fn test_remaining_increasing() {
        assert_eq!(GoalsService::calculate_remaining(50.0, 100.0, "increasing"), 50.0);
        assert_eq!(GoalsService::calculate_remaining(100.0, 100.0, "increasing"), 0.0);
        assert_eq!(GoalsService::calculate_remaining(120.0, 100.0, "increasing"), 0.0);
    }

    #[test]
    fn test_remaining_decreasing() {
        assert_eq!(GoalsService::calculate_remaining(100.0, 70.0, "decreasing"), 30.0);
        assert_eq!(GoalsService::calculate_remaining(70.0, 70.0, "decreasing"), 0.0);
        assert_eq!(GoalsService::calculate_remaining(60.0, 70.0, "decreasing"), 0.0);
    }

    #[test]
    fn test_progress_same_start_target() {
        // When start equals target, should be 100% if current equals target
        assert_eq!(GoalsService::calculate_progress(50.0, 50.0, 50.0, "increasing"), 100.0);
        assert_eq!(GoalsService::calculate_progress(50.0, 60.0, 50.0, "increasing"), 0.0);
    }
}
