//! Sleep repository for database operations

use anyhow::Result;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// Sleep Logs
// ============================================================================

/// Sleep log record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SleepLogRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub sleep_start: DateTime<Utc>,
    pub sleep_end: DateTime<Utc>,
    pub total_duration_minutes: i32,
    pub awake_minutes: i32,
    pub light_minutes: i32,
    pub deep_minutes: i32,
    pub rem_minutes: i32,
    pub sleep_efficiency: Option<Decimal>,
    pub sleep_score: Option<i32>,
    pub times_awoken: Option<i32>,
    pub avg_heart_rate: Option<i32>,
    pub min_heart_rate: Option<i32>,
    pub hrv_average: Option<Decimal>,
    pub respiratory_rate: Option<Decimal>,
    pub source: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a sleep log
#[derive(Debug, Clone)]
pub struct CreateSleepLog {
    pub user_id: Uuid,
    pub sleep_start: DateTime<Utc>,
    pub sleep_end: DateTime<Utc>,
    pub total_duration_minutes: i32,
    pub awake_minutes: i32,
    pub light_minutes: i32,
    pub deep_minutes: i32,
    pub rem_minutes: i32,
    pub sleep_efficiency: Option<Decimal>,
    pub sleep_score: Option<i32>,
    pub times_awoken: Option<i32>,
    pub avg_heart_rate: Option<i32>,
    pub min_heart_rate: Option<i32>,
    pub hrv_average: Option<Decimal>,
    pub respiratory_rate: Option<Decimal>,
    pub source: String,
    pub notes: Option<String>,
}

/// Sleep summary for a date range
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SleepSummary {
    pub avg_duration_minutes: Option<f64>,
    pub avg_efficiency: Option<f64>,
    pub avg_deep_minutes: Option<f64>,
    pub avg_rem_minutes: Option<f64>,
    pub avg_light_minutes: Option<f64>,
    pub avg_awake_minutes: Option<f64>,
    pub total_nights: i64,
}

/// Sleep log repository
pub struct SleepLogRepository;

impl SleepLogRepository {
    /// Create a new sleep log entry
    pub async fn create(pool: &PgPool, input: CreateSleepLog) -> Result<SleepLogRecord> {
        let record = sqlx::query_as::<_, SleepLogRecord>(
            r#"
            INSERT INTO sleep_logs (
                user_id, sleep_start, sleep_end, total_duration_minutes,
                awake_minutes, light_minutes, deep_minutes, rem_minutes,
                sleep_efficiency, sleep_score, times_awoken,
                avg_heart_rate, min_heart_rate, hrv_average, respiratory_rate,
                source, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING id, user_id, sleep_start, sleep_end, total_duration_minutes,
                      awake_minutes, light_minutes, deep_minutes, rem_minutes,
                      sleep_efficiency, sleep_score, times_awoken,
                      avg_heart_rate, min_heart_rate, hrv_average, respiratory_rate,
                      source, notes, created_at, updated_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.sleep_start)
        .bind(input.sleep_end)
        .bind(input.total_duration_minutes)
        .bind(input.awake_minutes)
        .bind(input.light_minutes)
        .bind(input.deep_minutes)
        .bind(input.rem_minutes)
        .bind(input.sleep_efficiency)
        .bind(input.sleep_score)
        .bind(input.times_awoken)
        .bind(input.avg_heart_rate)
        .bind(input.min_heart_rate)
        .bind(input.hrv_average)
        .bind(input.respiratory_rate)
        .bind(&input.source)
        .bind(&input.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get sleep logs for a specific date (by wake date)
    pub async fn get_by_date(
        pool: &PgPool,
        user_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<SleepLogRecord>> {
        let records = sqlx::query_as::<_, SleepLogRecord>(
            r#"
            SELECT id, user_id, sleep_start, sleep_end, total_duration_minutes,
                   awake_minutes, light_minutes, deep_minutes, rem_minutes,
                   sleep_efficiency, sleep_score, times_awoken,
                   avg_heart_rate, min_heart_rate, hrv_average, respiratory_rate,
                   source, notes, created_at, updated_at
            FROM sleep_logs
            WHERE user_id = $1 AND DATE(sleep_end) = $2
            ORDER BY sleep_end DESC
            "#,
        )
        .bind(user_id)
        .bind(date)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Get sleep history for a date range
    pub async fn get_history(
        pool: &PgPool,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SleepLogRecord>> {
        let records = sqlx::query_as::<_, SleepLogRecord>(
            r#"
            SELECT id, user_id, sleep_start, sleep_end, total_duration_minutes,
                   awake_minutes, light_minutes, deep_minutes, rem_minutes,
                   sleep_efficiency, sleep_score, times_awoken,
                   avg_heart_rate, min_heart_rate, hrv_average, respiratory_rate,
                   source, notes, created_at, updated_at
            FROM sleep_logs
            WHERE user_id = $1 
              AND DATE(sleep_end) >= $2 
              AND DATE(sleep_end) <= $3
            ORDER BY sleep_end DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Count sleep logs in date range
    pub async fn count_in_range(
        pool: &PgPool,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint
            FROM sleep_logs
            WHERE user_id = $1 
              AND DATE(sleep_end) >= $2 
              AND DATE(sleep_end) <= $3
            "#,
        )
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }

    /// Get sleep summary statistics for a date range
    pub async fn get_summary(
        pool: &PgPool,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<SleepSummary> {
        let summary = sqlx::query_as::<_, SleepSummary>(
            r#"
            SELECT 
                AVG(total_duration_minutes)::float8 as avg_duration_minutes,
                AVG(sleep_efficiency)::float8 as avg_efficiency,
                AVG(deep_minutes)::float8 as avg_deep_minutes,
                AVG(rem_minutes)::float8 as avg_rem_minutes,
                AVG(light_minutes)::float8 as avg_light_minutes,
                AVG(awake_minutes)::float8 as avg_awake_minutes,
                COUNT(*)::bigint as total_nights
            FROM sleep_logs
            WHERE user_id = $1 
              AND DATE(sleep_end) >= $2 
              AND DATE(sleep_end) <= $3
            "#,
        )
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await?;

        Ok(summary)
    }

    /// Get the latest sleep log for a user
    pub async fn get_latest(pool: &PgPool, user_id: Uuid) -> Result<Option<SleepLogRecord>> {
        let record = sqlx::query_as::<_, SleepLogRecord>(
            r#"
            SELECT id, user_id, sleep_start, sleep_end, total_duration_minutes,
                   awake_minutes, light_minutes, deep_minutes, rem_minutes,
                   sleep_efficiency, sleep_score, times_awoken,
                   avg_heart_rate, min_heart_rate, hrv_average, respiratory_rate,
                   source, notes, created_at, updated_at
            FROM sleep_logs
            WHERE user_id = $1
            ORDER BY sleep_end DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// Delete a sleep log entry
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM sleep_logs WHERE id = $1 AND user_id = $2"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// Sleep Goals
// ============================================================================

/// Sleep goal record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SleepGoalRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub target_duration_minutes: i32,
    pub target_bedtime: Option<NaiveTime>,
    pub target_wake_time: Option<NaiveTime>,
    pub bedtime_reminder_enabled: bool,
    pub bedtime_reminder_minutes_before: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating/updating a sleep goal
#[derive(Debug, Clone)]
pub struct UpsertSleepGoal {
    pub user_id: Uuid,
    pub target_duration_minutes: i32,
    pub target_bedtime: Option<NaiveTime>,
    pub target_wake_time: Option<NaiveTime>,
    pub bedtime_reminder_enabled: bool,
    pub bedtime_reminder_minutes_before: Option<i32>,
}

/// Sleep goal repository
pub struct SleepGoalRepository;

impl SleepGoalRepository {
    /// Get user's sleep goal
    pub async fn get_by_user(pool: &PgPool, user_id: Uuid) -> Result<Option<SleepGoalRecord>> {
        let record = sqlx::query_as::<_, SleepGoalRecord>(
            r#"
            SELECT id, user_id, target_duration_minutes, target_bedtime, target_wake_time,
                   bedtime_reminder_enabled, bedtime_reminder_minutes_before,
                   created_at, updated_at
            FROM sleep_goals
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// Create or update user's sleep goal
    pub async fn upsert(pool: &PgPool, input: UpsertSleepGoal) -> Result<SleepGoalRecord> {
        let record = sqlx::query_as::<_, SleepGoalRecord>(
            r#"
            INSERT INTO sleep_goals (
                user_id, target_duration_minutes, target_bedtime, target_wake_time,
                bedtime_reminder_enabled, bedtime_reminder_minutes_before
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id) DO UPDATE SET
                target_duration_minutes = EXCLUDED.target_duration_minutes,
                target_bedtime = EXCLUDED.target_bedtime,
                target_wake_time = EXCLUDED.target_wake_time,
                bedtime_reminder_enabled = EXCLUDED.bedtime_reminder_enabled,
                bedtime_reminder_minutes_before = EXCLUDED.bedtime_reminder_minutes_before
            RETURNING id, user_id, target_duration_minutes, target_bedtime, target_wake_time,
                      bedtime_reminder_enabled, bedtime_reminder_minutes_before,
                      created_at, updated_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.target_duration_minutes)
        .bind(input.target_bedtime)
        .bind(input.target_wake_time)
        .bind(input.bedtime_reminder_enabled)
        .bind(input.bedtime_reminder_minutes_before)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }
}
