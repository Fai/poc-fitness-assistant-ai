//! Goals repository for database operations

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// Goals
// ============================================================================

/// Goal record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GoalRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub goal_type: String,
    pub metric: String,
    pub target_value: Decimal,
    pub start_value: Option<Decimal>,
    pub current_value: Option<Decimal>,
    pub direction: String,
    pub start_date: NaiveDate,
    pub target_date: Option<NaiveDate>,
    pub status: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a goal
#[derive(Debug, Clone)]
pub struct CreateGoal {
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub goal_type: String,
    pub metric: String,
    pub target_value: Decimal,
    pub start_value: Option<Decimal>,
    pub direction: String,
    pub start_date: NaiveDate,
    pub target_date: Option<NaiveDate>,
}

/// Input for updating a goal
#[derive(Debug, Clone)]
pub struct UpdateGoal {
    pub name: Option<String>,
    pub description: Option<String>,
    pub target_value: Option<Decimal>,
    pub current_value: Option<Decimal>,
    pub target_date: Option<NaiveDate>,
    pub status: Option<String>,
}

/// Goal repository
pub struct GoalRepository;

impl GoalRepository {
    /// Create a new goal
    pub async fn create(pool: &PgPool, input: CreateGoal) -> Result<GoalRecord> {
        let record = sqlx::query_as::<_, GoalRecord>(
            r#"
            INSERT INTO goals (
                user_id, name, description, goal_type, metric,
                target_value, start_value, current_value, direction,
                start_date, target_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7, $8, $9, $10)
            RETURNING id, user_id, name, description, goal_type, metric,
                      target_value, start_value, current_value, direction,
                      start_date, target_date, status, completed_at,
                      created_at, updated_at
            "#,
        )
        .bind(input.user_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.goal_type)
        .bind(&input.metric)
        .bind(input.target_value)
        .bind(input.start_value)
        .bind(&input.direction)
        .bind(input.start_date)
        .bind(input.target_date)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get a goal by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<Option<GoalRecord>> {
        let record = sqlx::query_as::<_, GoalRecord>(
            r#"
            SELECT id, user_id, name, description, goal_type, metric,
                   target_value, start_value, current_value, direction,
                   start_date, target_date, status, completed_at,
                   created_at, updated_at
            FROM goals
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// Get all goals for a user
    pub async fn get_by_user(
        pool: &PgPool,
        user_id: Uuid,
        status: Option<&str>,
        goal_type: Option<&str>,
    ) -> Result<Vec<GoalRecord>> {
        let records = match (status, goal_type) {
            (Some(s), Some(t)) => {
                sqlx::query_as::<_, GoalRecord>(
                    r#"
                    SELECT id, user_id, name, description, goal_type, metric,
                           target_value, start_value, current_value, direction,
                           start_date, target_date, status, completed_at,
                           created_at, updated_at
                    FROM goals
                    WHERE user_id = $1 AND status = $2 AND goal_type = $3
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(user_id)
                .bind(s)
                .bind(t)
                .fetch_all(pool)
                .await?
            }
            (Some(s), None) => {
                sqlx::query_as::<_, GoalRecord>(
                    r#"
                    SELECT id, user_id, name, description, goal_type, metric,
                           target_value, start_value, current_value, direction,
                           start_date, target_date, status, completed_at,
                           created_at, updated_at
                    FROM goals
                    WHERE user_id = $1 AND status = $2
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(user_id)
                .bind(s)
                .fetch_all(pool)
                .await?
            }
            (None, Some(t)) => {
                sqlx::query_as::<_, GoalRecord>(
                    r#"
                    SELECT id, user_id, name, description, goal_type, metric,
                           target_value, start_value, current_value, direction,
                           start_date, target_date, status, completed_at,
                           created_at, updated_at
                    FROM goals
                    WHERE user_id = $1 AND goal_type = $2
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(user_id)
                .bind(t)
                .fetch_all(pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, GoalRecord>(
                    r#"
                    SELECT id, user_id, name, description, goal_type, metric,
                           target_value, start_value, current_value, direction,
                           start_date, target_date, status, completed_at,
                           created_at, updated_at
                    FROM goals
                    WHERE user_id = $1
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(user_id)
                .fetch_all(pool)
                .await?
            }
        };

        Ok(records)
    }

    /// Update a goal
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        updates: UpdateGoal,
    ) -> Result<Option<GoalRecord>> {
        let completed_at = if updates.status.as_deref() == Some("completed") {
            Some(Utc::now())
        } else {
            None
        };

        let record = sqlx::query_as::<_, GoalRecord>(
            r#"
            UPDATE goals SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                target_value = COALESCE($5, target_value),
                current_value = COALESCE($6, current_value),
                target_date = COALESCE($7, target_date),
                status = COALESCE($8, status),
                completed_at = COALESCE($9, completed_at)
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, name, description, goal_type, metric,
                      target_value, start_value, current_value, direction,
                      start_date, target_date, status, completed_at,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(&updates.name)
        .bind(&updates.description)
        .bind(updates.target_value)
        .bind(updates.current_value)
        .bind(updates.target_date)
        .bind(&updates.status)
        .bind(completed_at)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// Delete a goal
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM goals WHERE id = $1 AND user_id = $2"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// Goal Milestones
// ============================================================================

/// Milestone record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MilestoneRecord {
    pub id: Uuid,
    pub goal_id: Uuid,
    pub name: String,
    pub target_value: Decimal,
    pub percentage: i32,
    pub achieved_at: Option<DateTime<Utc>>,
    pub actual_value: Option<Decimal>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a milestone
#[derive(Debug, Clone)]
pub struct CreateMilestone {
    pub goal_id: Uuid,
    pub name: String,
    pub target_value: Decimal,
    pub percentage: i32,
}

/// Milestone repository
pub struct MilestoneRepository;

impl MilestoneRepository {
    /// Create a milestone
    pub async fn create(pool: &PgPool, input: CreateMilestone) -> Result<MilestoneRecord> {
        let record = sqlx::query_as::<_, MilestoneRecord>(
            r#"
            INSERT INTO goal_milestones (goal_id, name, target_value, percentage)
            VALUES ($1, $2, $3, $4)
            RETURNING id, goal_id, name, target_value, percentage, achieved_at, actual_value, created_at
            "#,
        )
        .bind(input.goal_id)
        .bind(&input.name)
        .bind(input.target_value)
        .bind(input.percentage)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get milestones for a goal
    pub async fn get_by_goal(pool: &PgPool, goal_id: Uuid) -> Result<Vec<MilestoneRecord>> {
        let records = sqlx::query_as::<_, MilestoneRecord>(
            r#"
            SELECT id, goal_id, name, target_value, percentage, achieved_at, actual_value, created_at
            FROM goal_milestones
            WHERE goal_id = $1
            ORDER BY percentage ASC
            "#,
        )
        .bind(goal_id)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Mark a milestone as achieved
    pub async fn achieve(
        pool: &PgPool,
        id: Uuid,
        actual_value: Decimal,
    ) -> Result<Option<MilestoneRecord>> {
        let record = sqlx::query_as::<_, MilestoneRecord>(
            r#"
            UPDATE goal_milestones SET
                achieved_at = NOW(),
                actual_value = $2
            WHERE id = $1 AND achieved_at IS NULL
            RETURNING id, goal_id, name, target_value, percentage, achieved_at, actual_value, created_at
            "#,
        )
        .bind(id)
        .bind(actual_value)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }
}
