//! Hydration repository for database operations

use anyhow::Result;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// Hydration Logs
// ============================================================================

/// Hydration log record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HydrationLogRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount_ml: i32,
    pub beverage_type: String,
    pub consumed_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a hydration log
#[derive(Debug, Clone)]
pub struct CreateHydrationLog {
    pub user_id: Uuid,
    pub amount_ml: i32,
    pub beverage_type: String,
    pub consumed_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
}

/// Daily hydration summary
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DailyHydrationSummary {
    pub date: NaiveDate,
    pub total_ml: i64,
    pub entry_count: i64,
    pub first_entry: Option<DateTime<Utc>>,
    pub last_entry: Option<DateTime<Utc>>,
}

/// Hydration log repository
pub struct HydrationLogRepository;

impl HydrationLogRepository {
    /// Create a new hydration log entry
    pub async fn create(pool: &PgPool, input: CreateHydrationLog) -> Result<HydrationLogRecord> {
        let record = sqlx::query_as::<_, HydrationLogRecord>(
            r#"
            INSERT INTO hydration_logs (user_id, amount_ml, beverage_type, consumed_at, source, notes)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, amount_ml, beverage_type, consumed_at, source, notes, created_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.amount_ml)
        .bind(&input.beverage_type)
        .bind(input.consumed_at)
        .bind(&input.source)
        .bind(&input.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get hydration logs for a specific date
    pub async fn get_by_date(
        pool: &PgPool,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<HydrationLogRecord>> {
        let records = sqlx::query_as::<_, HydrationLogRecord>(
            r#"
            SELECT id, user_id, amount_ml, beverage_type, consumed_at, source, notes, created_at
            FROM hydration_logs
            WHERE user_id = $1 AND DATE(consumed_at) = $2
            ORDER BY consumed_at ASC
            "#,
        )
        .bind(user_id)
        .bind(date)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Get daily summary for a specific date
    pub async fn get_daily_summary(
        pool: &PgPool,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<DailyHydrationSummary> {
        let summary = sqlx::query_as::<_, DailyHydrationSummary>(
            r#"
            SELECT 
                $2::date as date,
                COALESCE(SUM(amount_ml), 0)::bigint as total_ml,
                COUNT(*)::bigint as entry_count,
                MIN(consumed_at) as first_entry,
                MAX(consumed_at) as last_entry
            FROM hydration_logs
            WHERE user_id = $1 AND DATE(consumed_at) = $2
            "#,
        )
        .bind(user_id)
        .bind(date)
        .fetch_one(pool)
        .await?;

        Ok(summary)
    }

    /// Get daily summaries for a date range
    pub async fn get_daily_summaries(
        pool: &PgPool,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DailyHydrationSummary>> {
        let summaries = sqlx::query_as::<_, DailyHydrationSummary>(
            r#"
            SELECT 
                DATE(consumed_at) as date,
                SUM(amount_ml)::bigint as total_ml,
                COUNT(*)::bigint as entry_count,
                MIN(consumed_at) as first_entry,
                MAX(consumed_at) as last_entry
            FROM hydration_logs
            WHERE user_id = $1 AND DATE(consumed_at) >= $2 AND DATE(consumed_at) <= $3
            GROUP BY DATE(consumed_at)
            ORDER BY date DESC
            "#,
        )
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool)
        .await?;

        Ok(summaries)
    }

    /// Delete a hydration log entry
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM hydration_logs WHERE id = $1 AND user_id = $2"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// Hydration Goals
// ============================================================================

/// Hydration goal record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HydrationGoalRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub daily_goal_ml: i32,
    pub is_auto_calculated: bool,
    pub reminders_enabled: bool,
    pub reminder_interval_minutes: Option<i32>,
    pub reminder_start_time: Option<NaiveTime>,
    pub reminder_end_time: Option<NaiveTime>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating/updating a hydration goal
#[derive(Debug, Clone)]
pub struct UpsertHydrationGoal {
    pub user_id: Uuid,
    pub daily_goal_ml: i32,
    pub is_auto_calculated: bool,
    pub reminders_enabled: bool,
    pub reminder_interval_minutes: Option<i32>,
    pub reminder_start_time: Option<NaiveTime>,
    pub reminder_end_time: Option<NaiveTime>,
}

/// Hydration goal repository
pub struct HydrationGoalRepository;

impl HydrationGoalRepository {
    /// Get user's hydration goal
    pub async fn get_by_user(pool: &PgPool, user_id: Uuid) -> Result<Option<HydrationGoalRecord>> {
        let record = sqlx::query_as::<_, HydrationGoalRecord>(
            r#"
            SELECT id, user_id, daily_goal_ml, is_auto_calculated, reminders_enabled,
                   reminder_interval_minutes, reminder_start_time, reminder_end_time,
                   created_at, updated_at
            FROM hydration_goals
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// Create or update user's hydration goal
    pub async fn upsert(pool: &PgPool, input: UpsertHydrationGoal) -> Result<HydrationGoalRecord> {
        let record = sqlx::query_as::<_, HydrationGoalRecord>(
            r#"
            INSERT INTO hydration_goals (user_id, daily_goal_ml, is_auto_calculated, 
                                         reminders_enabled, reminder_interval_minutes,
                                         reminder_start_time, reminder_end_time)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (user_id) DO UPDATE SET
                daily_goal_ml = EXCLUDED.daily_goal_ml,
                is_auto_calculated = EXCLUDED.is_auto_calculated,
                reminders_enabled = EXCLUDED.reminders_enabled,
                reminder_interval_minutes = EXCLUDED.reminder_interval_minutes,
                reminder_start_time = EXCLUDED.reminder_start_time,
                reminder_end_time = EXCLUDED.reminder_end_time
            RETURNING id, user_id, daily_goal_ml, is_auto_calculated, reminders_enabled,
                      reminder_interval_minutes, reminder_start_time, reminder_end_time,
                      created_at, updated_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.daily_goal_ml)
        .bind(input.is_auto_calculated)
        .bind(input.reminders_enabled)
        .bind(input.reminder_interval_minutes)
        .bind(input.reminder_start_time)
        .bind(input.reminder_end_time)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }
}
