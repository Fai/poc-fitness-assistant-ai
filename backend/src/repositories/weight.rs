//! Weight and body composition repository for database operations

use anyhow::Result;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

/// Weight log record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WeightLogRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub weight_kg: Decimal,
    pub recorded_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
    pub is_anomaly: bool,
    pub created_at: DateTime<Utc>,
}

/// Body composition log record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BodyCompositionLogRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub recorded_at: DateTime<Utc>,
    pub body_fat_percent: Option<Decimal>,
    pub muscle_mass_kg: Option<Decimal>,
    pub water_percent: Option<Decimal>,
    pub bone_mass_kg: Option<Decimal>,
    pub visceral_fat: Option<i32>,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a weight log
#[derive(Debug, Clone)]
pub struct CreateWeightLog {
    pub user_id: Uuid,
    pub weight_kg: f64,
    pub recorded_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
    pub is_anomaly: bool,
}

/// Input for creating a body composition log
#[derive(Debug, Clone)]
pub struct CreateBodyCompositionLog {
    pub user_id: Uuid,
    pub recorded_at: DateTime<Utc>,
    pub body_fat_percent: Option<f64>,
    pub muscle_mass_kg: Option<f64>,
    pub water_percent: Option<f64>,
    pub bone_mass_kg: Option<f64>,
    pub visceral_fat: Option<i32>,
    pub source: String,
}

/// Weight repository for database operations
pub struct WeightRepository;

impl WeightRepository {
    /// Create a new weight log entry
    pub async fn create(pool: &PgPool, input: CreateWeightLog) -> Result<WeightLogRecord> {
        let record = sqlx::query_as::<_, WeightLogRecord>(
            r#"
            INSERT INTO weight_logs (user_id, weight_kg, recorded_at, source, notes, is_anomaly)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, weight_kg, recorded_at, source, notes, is_anomaly, created_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.weight_kg)
        .bind(input.recorded_at)
        .bind(&input.source)
        .bind(&input.notes)
        .bind(input.is_anomaly)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get weight logs for a user within a date range
    pub async fn get_by_date_range(
        pool: &PgPool,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<WeightLogRecord>> {
        let records = sqlx::query_as::<_, WeightLogRecord>(
            r#"
            SELECT id, user_id, weight_kg, recorded_at, source, notes, is_anomaly, created_at
            FROM weight_logs
            WHERE user_id = $1 AND recorded_at >= $2 AND recorded_at <= $3
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Get the most recent weight log for a user
    pub async fn get_latest(pool: &PgPool, user_id: Uuid) -> Result<Option<WeightLogRecord>> {
        let record = sqlx::query_as::<_, WeightLogRecord>(
            r#"
            SELECT id, user_id, weight_kg, recorded_at, source, notes, is_anomaly, created_at
            FROM weight_logs
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

    /// Get the N most recent weight logs for a user
    pub async fn get_recent(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<WeightLogRecord>> {
        let records = sqlx::query_as::<_, WeightLogRecord>(
            r#"
            SELECT id, user_id, weight_kg, recorded_at, source, notes, is_anomaly, created_at
            FROM weight_logs
            WHERE user_id = $1
            ORDER BY recorded_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Get weight log by ID
    pub async fn get_by_id(
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<WeightLogRecord>> {
        let record = sqlx::query_as::<_, WeightLogRecord>(
            r#"
            SELECT id, user_id, weight_kg, recorded_at, source, notes, is_anomaly, created_at
            FROM weight_logs
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// Delete a weight log
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM weight_logs
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// Body composition repository for database operations
pub struct BodyCompositionRepository;

impl BodyCompositionRepository {
    /// Create a new body composition log entry
    pub async fn create(
        pool: &PgPool,
        input: CreateBodyCompositionLog,
    ) -> Result<BodyCompositionLogRecord> {
        let record = sqlx::query_as::<_, BodyCompositionLogRecord>(
            r#"
            INSERT INTO body_composition_logs 
                (user_id, recorded_at, body_fat_percent, muscle_mass_kg, water_percent, bone_mass_kg, visceral_fat, source)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, recorded_at, body_fat_percent, muscle_mass_kg, water_percent, bone_mass_kg, visceral_fat, source, created_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.recorded_at)
        .bind(input.body_fat_percent)
        .bind(input.muscle_mass_kg)
        .bind(input.water_percent)
        .bind(input.bone_mass_kg)
        .bind(input.visceral_fat)
        .bind(&input.source)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get body composition logs for a user within a date range
    pub async fn get_by_date_range(
        pool: &PgPool,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<BodyCompositionLogRecord>> {
        let records = sqlx::query_as::<_, BodyCompositionLogRecord>(
            r#"
            SELECT id, user_id, recorded_at, body_fat_percent, muscle_mass_kg, water_percent, bone_mass_kg, visceral_fat, source, created_at
            FROM body_composition_logs
            WHERE user_id = $1 AND recorded_at >= $2 AND recorded_at <= $3
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Get the most recent body composition log for a user
    pub async fn get_latest(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Option<BodyCompositionLogRecord>> {
        let record = sqlx::query_as::<_, BodyCompositionLogRecord>(
            r#"
            SELECT id, user_id, recorded_at, body_fat_percent, muscle_mass_kg, water_percent, bone_mass_kg, visceral_fat, source, created_at
            FROM body_composition_logs
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
}
