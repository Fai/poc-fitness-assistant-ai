//! Biometrics repository for heart rate and HRV database operations

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// Heart Rate Logs
// ============================================================================

/// Heart rate log record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HeartRateLogRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bpm: i32,
    pub context: String,
    pub recorded_at: DateTime<Utc>,
    pub workout_id: Option<Uuid>,
    pub source: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a heart rate log
#[derive(Debug, Clone)]
pub struct CreateHeartRateLog {
    pub user_id: Uuid,
    pub bpm: i32,
    pub context: String,
    pub recorded_at: DateTime<Utc>,
    pub workout_id: Option<Uuid>,
    pub source: String,
    pub notes: Option<String>,
}

/// Heart rate statistics
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HeartRateStats {
    pub avg_bpm: Option<f64>,
    pub min_bpm: Option<i32>,
    pub max_bpm: Option<i32>,
    pub count: i64,
}

/// Heart rate log repository
pub struct HeartRateLogRepository;

impl HeartRateLogRepository {
    /// Create a new heart rate log entry
    pub async fn create(pool: &PgPool, input: CreateHeartRateLog) -> Result<HeartRateLogRecord> {
        let record = sqlx::query_as::<_, HeartRateLogRecord>(
            r#"
            INSERT INTO heart_rate_logs (user_id, bpm, context, recorded_at, workout_id, source, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, bpm, context, recorded_at, workout_id, source, notes, created_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.bpm)
        .bind(&input.context)
        .bind(input.recorded_at)
        .bind(input.workout_id)
        .bind(&input.source)
        .bind(&input.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get heart rate logs for a date range
    pub async fn get_history(
        pool: &PgPool,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        context: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<HeartRateLogRecord>> {
        let records = if let Some(ctx) = context {
            sqlx::query_as::<_, HeartRateLogRecord>(
                r#"
                SELECT id, user_id, bpm, context, recorded_at, workout_id, source, notes, created_at
                FROM heart_rate_logs
                WHERE user_id = $1 
                  AND DATE(recorded_at) >= $2 
                  AND DATE(recorded_at) <= $3
                  AND context = $4
                ORDER BY recorded_at DESC
                LIMIT $5 OFFSET $6
                "#,
            )
            .bind(user_id)
            .bind(start_date)
            .bind(end_date)
            .bind(ctx)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, HeartRateLogRecord>(
                r#"
                SELECT id, user_id, bpm, context, recorded_at, workout_id, source, notes, created_at
                FROM heart_rate_logs
                WHERE user_id = $1 
                  AND DATE(recorded_at) >= $2 
                  AND DATE(recorded_at) <= $3
                ORDER BY recorded_at DESC
                LIMIT $4 OFFSET $5
                "#,
            )
            .bind(user_id)
            .bind(start_date)
            .bind(end_date)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };

        Ok(records)
    }

    /// Get resting heart rate average for a date range (7-day baseline)
    pub async fn get_resting_baseline(
        pool: &PgPool,
        user_id: Uuid,
        end_date: NaiveDate,
        days: i32,
    ) -> Result<Option<f64>> {
        let start_date = end_date - chrono::Duration::days(days as i64);
        
        let result: (Option<f64>,) = sqlx::query_as(
            r#"
            SELECT AVG(bpm)::float8
            FROM heart_rate_logs
            WHERE user_id = $1 
              AND DATE(recorded_at) >= $2 
              AND DATE(recorded_at) <= $3
              AND context = 'resting'
            "#,
        )
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    /// Get heart rate statistics for a date range
    pub async fn get_stats(
        pool: &PgPool,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        context: Option<&str>,
    ) -> Result<HeartRateStats> {
        let stats = if let Some(ctx) = context {
            sqlx::query_as::<_, HeartRateStats>(
                r#"
                SELECT 
                    AVG(bpm)::float8 as avg_bpm,
                    MIN(bpm) as min_bpm,
                    MAX(bpm) as max_bpm,
                    COUNT(*)::bigint as count
                FROM heart_rate_logs
                WHERE user_id = $1 
                  AND DATE(recorded_at) >= $2 
                  AND DATE(recorded_at) <= $3
                  AND context = $4
                "#,
            )
            .bind(user_id)
            .bind(start_date)
            .bind(end_date)
            .bind(ctx)
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query_as::<_, HeartRateStats>(
                r#"
                SELECT 
                    AVG(bpm)::float8 as avg_bpm,
                    MIN(bpm) as min_bpm,
                    MAX(bpm) as max_bpm,
                    COUNT(*)::bigint as count
                FROM heart_rate_logs
                WHERE user_id = $1 
                  AND DATE(recorded_at) >= $2 
                  AND DATE(recorded_at) <= $3
                "#,
            )
            .bind(user_id)
            .bind(start_date)
            .bind(end_date)
            .fetch_one(pool)
            .await?
        };

        Ok(stats)
    }

    /// Delete a heart rate log entry
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM heart_rate_logs WHERE id = $1 AND user_id = $2"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// HRV Logs
// ============================================================================

/// HRV log record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HrvLogRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub rmssd: Decimal,
    pub sdnn: Option<Decimal>,
    pub context: String,
    pub recorded_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating an HRV log
#[derive(Debug, Clone)]
pub struct CreateHrvLog {
    pub user_id: Uuid,
    pub rmssd: Decimal,
    pub sdnn: Option<Decimal>,
    pub context: String,
    pub recorded_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
}

/// HRV statistics
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HrvStats {
    pub avg_rmssd: Option<f64>,
    pub min_rmssd: Option<f64>,
    pub max_rmssd: Option<f64>,
    pub count: i64,
}

/// HRV log repository
pub struct HrvLogRepository;

impl HrvLogRepository {
    /// Create a new HRV log entry
    pub async fn create(pool: &PgPool, input: CreateHrvLog) -> Result<HrvLogRecord> {
        let record = sqlx::query_as::<_, HrvLogRecord>(
            r#"
            INSERT INTO hrv_logs (user_id, rmssd, sdnn, context, recorded_at, source, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, rmssd, sdnn, context, recorded_at, source, notes, created_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.rmssd)
        .bind(input.sdnn)
        .bind(&input.context)
        .bind(input.recorded_at)
        .bind(&input.source)
        .bind(&input.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get HRV baseline (7-day average of morning readings)
    pub async fn get_baseline(
        pool: &PgPool,
        user_id: Uuid,
        end_date: NaiveDate,
        days: i32,
    ) -> Result<Option<f64>> {
        let start_date = end_date - chrono::Duration::days(days as i64);
        
        let result: (Option<f64>,) = sqlx::query_as(
            r#"
            SELECT AVG(rmssd)::float8
            FROM hrv_logs
            WHERE user_id = $1 
              AND DATE(recorded_at) >= $2 
              AND DATE(recorded_at) <= $3
              AND context = 'morning'
            "#,
        )
        .bind(user_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    /// Get latest HRV reading
    pub async fn get_latest(pool: &PgPool, user_id: Uuid) -> Result<Option<HrvLogRecord>> {
        let record = sqlx::query_as::<_, HrvLogRecord>(
            r#"
            SELECT id, user_id, rmssd, sdnn, context, recorded_at, source, notes, created_at
            FROM hrv_logs
            WHERE user_id = $1
            ORDER BY recorded_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// Get HRV history for a date range
    pub async fn get_history(
        pool: &PgPool,
        user_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<HrvLogRecord>> {
        let records = sqlx::query_as::<_, HrvLogRecord>(
            r#"
            SELECT id, user_id, rmssd, sdnn, context, recorded_at, source, notes, created_at
            FROM hrv_logs
            WHERE user_id = $1 
              AND DATE(recorded_at) >= $2 
              AND DATE(recorded_at) <= $3
            ORDER BY recorded_at DESC
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

    /// Delete an HRV log entry
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM hrv_logs WHERE id = $1 AND user_id = $2"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// Heart Rate Zones
// ============================================================================

/// Heart rate zones record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HeartRateZonesRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub max_heart_rate: i32,
    pub resting_heart_rate: Option<i32>,
    pub zone1_min: i32,
    pub zone1_max: i32,
    pub zone2_min: i32,
    pub zone2_max: i32,
    pub zone3_min: i32,
    pub zone3_max: i32,
    pub zone4_min: i32,
    pub zone4_max: i32,
    pub zone5_min: i32,
    pub zone5_max: i32,
    pub calculation_method: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating/updating heart rate zones
#[derive(Debug, Clone)]
pub struct UpsertHeartRateZones {
    pub user_id: Uuid,
    pub max_heart_rate: i32,
    pub resting_heart_rate: Option<i32>,
    pub zone1_min: i32,
    pub zone1_max: i32,
    pub zone2_min: i32,
    pub zone2_max: i32,
    pub zone3_min: i32,
    pub zone3_max: i32,
    pub zone4_min: i32,
    pub zone4_max: i32,
    pub zone5_min: i32,
    pub zone5_max: i32,
    pub calculation_method: String,
}

/// Heart rate zones repository
pub struct HeartRateZonesRepository;

impl HeartRateZonesRepository {
    /// Get user's heart rate zones
    pub async fn get_by_user(pool: &PgPool, user_id: Uuid) -> Result<Option<HeartRateZonesRecord>> {
        let record = sqlx::query_as::<_, HeartRateZonesRecord>(
            r#"
            SELECT id, user_id, max_heart_rate, resting_heart_rate,
                   zone1_min, zone1_max, zone2_min, zone2_max,
                   zone3_min, zone3_max, zone4_min, zone4_max,
                   zone5_min, zone5_max, calculation_method,
                   created_at, updated_at
            FROM heart_rate_zones
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// Create or update user's heart rate zones
    pub async fn upsert(pool: &PgPool, input: UpsertHeartRateZones) -> Result<HeartRateZonesRecord> {
        let record = sqlx::query_as::<_, HeartRateZonesRecord>(
            r#"
            INSERT INTO heart_rate_zones (
                user_id, max_heart_rate, resting_heart_rate,
                zone1_min, zone1_max, zone2_min, zone2_max,
                zone3_min, zone3_max, zone4_min, zone4_max,
                zone5_min, zone5_max, calculation_method
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (user_id) DO UPDATE SET
                max_heart_rate = EXCLUDED.max_heart_rate,
                resting_heart_rate = EXCLUDED.resting_heart_rate,
                zone1_min = EXCLUDED.zone1_min,
                zone1_max = EXCLUDED.zone1_max,
                zone2_min = EXCLUDED.zone2_min,
                zone2_max = EXCLUDED.zone2_max,
                zone3_min = EXCLUDED.zone3_min,
                zone3_max = EXCLUDED.zone3_max,
                zone4_min = EXCLUDED.zone4_min,
                zone4_max = EXCLUDED.zone4_max,
                zone5_min = EXCLUDED.zone5_min,
                zone5_max = EXCLUDED.zone5_max,
                calculation_method = EXCLUDED.calculation_method
            RETURNING id, user_id, max_heart_rate, resting_heart_rate,
                      zone1_min, zone1_max, zone2_min, zone2_max,
                      zone3_min, zone3_max, zone4_min, zone4_max,
                      zone5_min, zone5_max, calculation_method,
                      created_at, updated_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.max_heart_rate)
        .bind(input.resting_heart_rate)
        .bind(input.zone1_min)
        .bind(input.zone1_max)
        .bind(input.zone2_min)
        .bind(input.zone2_max)
        .bind(input.zone3_min)
        .bind(input.zone3_max)
        .bind(input.zone4_min)
        .bind(input.zone4_max)
        .bind(input.zone5_min)
        .bind(input.zone5_max)
        .bind(&input.calculation_method)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }
}
