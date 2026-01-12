//! Data management service for user data operations
//!
//! Provides:
//! - Complete data deletion (GDPR compliance)
//!
//! Property 21: Data Deletion Completeness
//! After deletion, no user data remains in the database

use crate::error::ApiError;
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

/// Data management service
pub struct DataService;

impl DataService {
    /// Delete all user data across all tables
    ///
    /// # Property 21: Data Deletion Completeness
    /// After deletion, no user data remains in the database
    ///
    /// Order matters due to foreign key constraints:
    /// 1. Delete child records first (logs, milestones)
    /// 2. Delete parent records (goals, supplements)
    /// 3. Delete user settings
    /// 4. Delete user account
    pub async fn delete_all_user_data(pool: &PgPool, user_id: Uuid) -> Result<DeletionSummary, ApiError> {
        let mut summary = DeletionSummary::default();

        // Start transaction
        let mut tx = pool.begin().await.map_err(|e| ApiError::Internal(e.into()))?;

        // Delete biomarker logs
        let result = sqlx::query("DELETE FROM biomarker_logs WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.biomarker_logs = result.rows_affected() as i64;

        // Delete supplement logs (via supplements)
        let result = sqlx::query(
            "DELETE FROM supplement_logs WHERE supplement_id IN (SELECT id FROM supplements WHERE user_id = $1)"
        )
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.supplement_logs = result.rows_affected() as i64;

        // Delete supplements
        let result = sqlx::query("DELETE FROM supplements WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.supplements = result.rows_affected() as i64;

        // Delete goal milestones (via goals)
        let result = sqlx::query(
            "DELETE FROM goal_milestones WHERE goal_id IN (SELECT id FROM goals WHERE user_id = $1)"
        )
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.goal_milestones = result.rows_affected() as i64;

        // Delete goals
        let result = sqlx::query("DELETE FROM goals WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.goals = result.rows_affected() as i64;

        // Delete HRV logs
        let result = sqlx::query("DELETE FROM hrv_logs WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.hrv_logs = result.rows_affected() as i64;

        // Delete heart rate logs
        let result = sqlx::query("DELETE FROM heart_rate_logs WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.heart_rate_logs = result.rows_affected() as i64;

        // Delete heart rate zones
        let result = sqlx::query("DELETE FROM heart_rate_zones WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.heart_rate_zones = result.rows_affected() as i64;

        // Delete sleep goals
        let result = sqlx::query("DELETE FROM sleep_goals WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.sleep_goals = result.rows_affected() as i64;

        // Delete sleep logs
        let result = sqlx::query("DELETE FROM sleep_logs WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.sleep_logs = result.rows_affected() as i64;

        // Delete hydration goals
        let result = sqlx::query("DELETE FROM hydration_goals WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.hydration_goals = result.rows_affected() as i64;

        // Delete hydration logs
        let result = sqlx::query("DELETE FROM hydration_logs WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.hydration_logs = result.rows_affected() as i64;

        // Delete exercise sets (via workout_exercises via workouts)
        let result = sqlx::query(
            "DELETE FROM exercise_sets WHERE workout_exercise_id IN (
                SELECT we.id FROM workout_exercises we
                JOIN workouts w ON we.workout_id = w.id
                WHERE w.user_id = $1
            )"
        )
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.exercise_sets = result.rows_affected() as i64;

        // Delete workout exercises (via workouts)
        let result = sqlx::query(
            "DELETE FROM workout_exercises WHERE workout_id IN (SELECT id FROM workouts WHERE user_id = $1)"
        )
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.workout_exercises = result.rows_affected() as i64;

        // Delete workouts
        let result = sqlx::query("DELETE FROM workouts WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.workouts = result.rows_affected() as i64;

        // Delete custom exercises
        let result = sqlx::query("DELETE FROM exercises WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.custom_exercises = result.rows_affected() as i64;

        // Delete recipe ingredients (via recipes)
        let result = sqlx::query(
            "DELETE FROM recipe_ingredients WHERE recipe_id IN (SELECT id FROM recipes WHERE user_id = $1)"
        )
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.recipe_ingredients = result.rows_affected() as i64;

        // Delete recipes
        let result = sqlx::query("DELETE FROM recipes WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.recipes = result.rows_affected() as i64;

        // Delete food logs
        let result = sqlx::query("DELETE FROM food_logs WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.food_logs = result.rows_affected() as i64;

        // Delete custom food items
        let result = sqlx::query("DELETE FROM food_items WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.custom_food_items = result.rows_affected() as i64;

        // Delete body composition logs
        let result = sqlx::query("DELETE FROM body_composition_logs WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.body_composition_logs = result.rows_affected() as i64;

        // Delete weight logs
        let result = sqlx::query("DELETE FROM weight_logs WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.weight_logs = result.rows_affected() as i64;

        // Delete user settings
        let result = sqlx::query("DELETE FROM user_settings WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.user_settings = result.rows_affected() as i64;

        // Delete user account
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;
        summary.users = result.rows_affected() as i64;

        // Commit transaction
        tx.commit().await.map_err(|e| ApiError::Internal(e.into()))?;

        Ok(summary)
    }

    /// Verify no data remains for a user
    ///
    /// # Property 21: Data Deletion Completeness
    /// Returns true if no data exists for the user
    pub async fn verify_deletion(pool: &PgPool, user_id: Uuid) -> Result<bool, ApiError> {
        // Check each table for remaining data
        let tables = [
            "users",
            "user_settings",
            "weight_logs",
            "body_composition_logs",
            "food_logs",
            "recipes",
            "workouts",
            "hydration_logs",
            "hydration_goals",
            "sleep_logs",
            "sleep_goals",
            "heart_rate_logs",
            "hrv_logs",
            "heart_rate_zones",
            "goals",
            "supplements",
            "biomarker_logs",
        ];

        for table in tables {
            let query = if table == "users" {
                format!("SELECT COUNT(*) as count FROM {} WHERE id = $1", table)
            } else {
                format!("SELECT COUNT(*) as count FROM {} WHERE user_id = $1", table)
            };

            let count: (i64,) = sqlx::query_as(&query)
                .bind(user_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ApiError::Internal(e.into()))?;

            if count.0 > 0 {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Summary of deleted records
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct DeletionSummary {
    pub users: i64,
    pub user_settings: i64,
    pub weight_logs: i64,
    pub body_composition_logs: i64,
    pub food_logs: i64,
    pub custom_food_items: i64,
    pub recipes: i64,
    pub recipe_ingredients: i64,
    pub workouts: i64,
    pub workout_exercises: i64,
    pub exercise_sets: i64,
    pub custom_exercises: i64,
    pub hydration_logs: i64,
    pub hydration_goals: i64,
    pub sleep_logs: i64,
    pub sleep_goals: i64,
    pub heart_rate_logs: i64,
    pub hrv_logs: i64,
    pub heart_rate_zones: i64,
    pub goals: i64,
    pub goal_milestones: i64,
    pub supplements: i64,
    pub supplement_logs: i64,
    pub biomarker_logs: i64,
}

impl DeletionSummary {
    /// Get total records deleted
    pub fn total(&self) -> i64 {
        self.users
            + self.user_settings
            + self.weight_logs
            + self.body_composition_logs
            + self.food_logs
            + self.custom_food_items
            + self.recipes
            + self.recipe_ingredients
            + self.workouts
            + self.workout_exercises
            + self.exercise_sets
            + self.custom_exercises
            + self.hydration_logs
            + self.hydration_goals
            + self.sleep_logs
            + self.sleep_goals
            + self.heart_rate_logs
            + self.hrv_logs
            + self.heart_rate_zones
            + self.goals
            + self.goal_milestones
            + self.supplements
            + self.supplement_logs
            + self.biomarker_logs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deletion_summary_total() {
        let mut summary = DeletionSummary::default();
        summary.weight_logs = 10;
        summary.sleep_logs = 5;
        summary.users = 1;
        
        assert_eq!(summary.total(), 16);
    }

    #[test]
    fn test_deletion_summary_default_is_zero() {
        let summary = DeletionSummary::default();
        assert_eq!(summary.total(), 0);
    }
}
