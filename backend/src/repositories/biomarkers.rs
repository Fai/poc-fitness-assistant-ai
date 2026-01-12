//! Biomarkers repository for database operations

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// Biomarker Ranges (Reference Data)
// ============================================================================

/// Biomarker range record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BiomarkerRangeRecord {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub category: String,
    pub unit: String,
    pub low_threshold: Option<Decimal>,
    pub optimal_min: Option<Decimal>,
    pub optimal_max: Option<Decimal>,
    pub high_threshold: Option<Decimal>,
    pub description: Option<String>,
}

/// Biomarker range repository
pub struct BiomarkerRangeRepository;

impl BiomarkerRangeRepository {
    /// Get all biomarker ranges
    pub async fn get_all(pool: &PgPool) -> Result<Vec<BiomarkerRangeRecord>> {
        let records = sqlx::query_as::<_, BiomarkerRangeRecord>(
            r#"
            SELECT id, name, display_name, category, unit,
                   low_threshold, optimal_min, optimal_max, high_threshold, description
            FROM biomarker_ranges
            ORDER BY category, display_name
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Get biomarker range by name
    pub async fn get_by_name(pool: &PgPool, name: &str) -> Result<Option<BiomarkerRangeRecord>> {
        let record = sqlx::query_as::<_, BiomarkerRangeRecord>(
            r#"
            SELECT id, name, display_name, category, unit,
                   low_threshold, optimal_min, optimal_max, high_threshold, description
            FROM biomarker_ranges
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }

    /// Get biomarker range by ID
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<BiomarkerRangeRecord>> {
        let record = sqlx::query_as::<_, BiomarkerRangeRecord>(
            r#"
            SELECT id, name, display_name, category, unit,
                   low_threshold, optimal_min, optimal_max, high_threshold, description
            FROM biomarker_ranges
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(record)
    }
}

// ============================================================================
// Biomarker Logs
// ============================================================================

/// Biomarker log record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BiomarkerLogRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub biomarker_id: Uuid,
    pub value: Decimal,
    pub classification: Option<String>,
    pub test_date: NaiveDate,
    pub lab_name: Option<String>,
    pub notes: Option<String>,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

/// Biomarker log with range info
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BiomarkerLogWithRange {
    pub id: Uuid,
    pub user_id: Uuid,
    pub biomarker_id: Uuid,
    pub value: Decimal,
    pub classification: Option<String>,
    pub test_date: NaiveDate,
    pub lab_name: Option<String>,
    pub notes: Option<String>,
    pub source: String,
    pub created_at: DateTime<Utc>,
    // Range info
    pub biomarker_name: String,
    pub display_name: String,
    pub category: String,
    pub unit: String,
}

/// Input for creating a biomarker log
#[derive(Debug, Clone)]
pub struct CreateBiomarkerLog {
    pub user_id: Uuid,
    pub biomarker_id: Uuid,
    pub value: Decimal,
    pub classification: Option<String>,
    pub test_date: NaiveDate,
    pub lab_name: Option<String>,
    pub notes: Option<String>,
    pub source: String,
}

/// Biomarker log repository
pub struct BiomarkerLogRepository;

impl BiomarkerLogRepository {
    /// Create a new biomarker log
    pub async fn create(pool: &PgPool, input: CreateBiomarkerLog) -> Result<BiomarkerLogRecord> {
        let record = sqlx::query_as::<_, BiomarkerLogRecord>(
            r#"
            INSERT INTO biomarker_logs (user_id, biomarker_id, value, classification, test_date, lab_name, notes, source)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, biomarker_id, value, classification, test_date, lab_name, notes, source, created_at
            "#,
        )
        .bind(input.user_id)
        .bind(input.biomarker_id)
        .bind(input.value)
        .bind(&input.classification)
        .bind(input.test_date)
        .bind(&input.lab_name)
        .bind(&input.notes)
        .bind(&input.source)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get biomarker logs for a user with range info
    pub async fn get_by_user(
        pool: &PgPool,
        user_id: Uuid,
        biomarker_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BiomarkerLogWithRange>> {
        let records = if let Some(name) = biomarker_name {
            sqlx::query_as::<_, BiomarkerLogWithRange>(
                r#"
                SELECT bl.id, bl.user_id, bl.biomarker_id, bl.value, bl.classification,
                       bl.test_date, bl.lab_name, bl.notes, bl.source, bl.created_at,
                       br.name as biomarker_name, br.display_name, br.category, br.unit
                FROM biomarker_logs bl
                JOIN biomarker_ranges br ON bl.biomarker_id = br.id
                WHERE bl.user_id = $1 AND br.name = $2
                ORDER BY bl.test_date DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(user_id)
            .bind(name)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, BiomarkerLogWithRange>(
                r#"
                SELECT bl.id, bl.user_id, bl.biomarker_id, bl.value, bl.classification,
                       bl.test_date, bl.lab_name, bl.notes, bl.source, bl.created_at,
                       br.name as biomarker_name, br.display_name, br.category, br.unit
                FROM biomarker_logs bl
                JOIN biomarker_ranges br ON bl.biomarker_id = br.id
                WHERE bl.user_id = $1
                ORDER BY bl.test_date DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };

        Ok(records)
    }

    /// Delete a biomarker log
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM biomarker_logs WHERE id = $1 AND user_id = $2"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// Supplements
// ============================================================================

/// Supplement record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SupplementRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub brand: Option<String>,
    pub dosage: String,
    pub frequency: String,
    pub time_of_day: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub is_active: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a supplement
#[derive(Debug, Clone)]
pub struct CreateSupplement {
    pub user_id: Uuid,
    pub name: String,
    pub brand: Option<String>,
    pub dosage: String,
    pub frequency: String,
    pub time_of_day: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

/// Supplement repository
pub struct SupplementRepository;

impl SupplementRepository {
    /// Create a new supplement
    pub async fn create(pool: &PgPool, input: CreateSupplement) -> Result<SupplementRecord> {
        let record = sqlx::query_as::<_, SupplementRecord>(
            r#"
            INSERT INTO supplements (user_id, name, brand, dosage, frequency, time_of_day, start_date, end_date, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, user_id, name, brand, dosage, frequency, time_of_day, start_date, end_date, is_active, notes, created_at, updated_at
            "#,
        )
        .bind(input.user_id)
        .bind(&input.name)
        .bind(&input.brand)
        .bind(&input.dosage)
        .bind(&input.frequency)
        .bind(&input.time_of_day)
        .bind(input.start_date)
        .bind(input.end_date)
        .bind(&input.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get supplements for a user
    pub async fn get_by_user(
        pool: &PgPool,
        user_id: Uuid,
        active_only: bool,
    ) -> Result<Vec<SupplementRecord>> {
        let records = if active_only {
            sqlx::query_as::<_, SupplementRecord>(
                r#"
                SELECT id, user_id, name, brand, dosage, frequency, time_of_day,
                       start_date, end_date, is_active, notes, created_at, updated_at
                FROM supplements
                WHERE user_id = $1 AND is_active = true
                ORDER BY name
                "#,
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, SupplementRecord>(
                r#"
                SELECT id, user_id, name, brand, dosage, frequency, time_of_day,
                       start_date, end_date, is_active, notes, created_at, updated_at
                FROM supplements
                WHERE user_id = $1
                ORDER BY is_active DESC, name
                "#,
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?
        };

        Ok(records)
    }

    /// Update supplement active status
    pub async fn set_active(pool: &PgPool, id: Uuid, user_id: Uuid, active: bool) -> Result<bool> {
        let result = sqlx::query(
            r#"UPDATE supplements SET is_active = $3 WHERE id = $1 AND user_id = $2"#,
        )
        .bind(id)
        .bind(user_id)
        .bind(active)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete a supplement
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM supplements WHERE id = $1 AND user_id = $2"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

// ============================================================================
// Supplement Logs
// ============================================================================

/// Supplement log record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SupplementLogRecord {
    pub id: Uuid,
    pub supplement_id: Uuid,
    pub user_id: Uuid,
    pub taken_at: DateTime<Utc>,
    pub skipped: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a supplement log
#[derive(Debug, Clone)]
pub struct CreateSupplementLog {
    pub supplement_id: Uuid,
    pub user_id: Uuid,
    pub taken_at: DateTime<Utc>,
    pub skipped: bool,
    pub notes: Option<String>,
}

/// Adherence stats
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdherenceStats {
    pub total_expected: i64,
    pub total_taken: i64,
    pub total_skipped: i64,
}

/// Supplement log repository
pub struct SupplementLogRepository;

impl SupplementLogRepository {
    /// Create a new supplement log
    pub async fn create(pool: &PgPool, input: CreateSupplementLog) -> Result<SupplementLogRecord> {
        let record = sqlx::query_as::<_, SupplementLogRecord>(
            r#"
            INSERT INTO supplement_logs (supplement_id, user_id, taken_at, skipped, notes)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, supplement_id, user_id, taken_at, skipped, notes, created_at
            "#,
        )
        .bind(input.supplement_id)
        .bind(input.user_id)
        .bind(input.taken_at)
        .bind(input.skipped)
        .bind(&input.notes)
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    /// Get adherence stats for a supplement over a date range
    pub async fn get_adherence(
        pool: &PgPool,
        supplement_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<AdherenceStats> {
        let stats = sqlx::query_as::<_, AdherenceStats>(
            r#"
            SELECT 
                COUNT(*)::bigint as total_expected,
                COUNT(*) FILTER (WHERE NOT skipped)::bigint as total_taken,
                COUNT(*) FILTER (WHERE skipped)::bigint as total_skipped
            FROM supplement_logs
            WHERE supplement_id = $1 
              AND DATE(taken_at) >= $2 
              AND DATE(taken_at) <= $3
            "#,
        )
        .bind(supplement_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await?;

        Ok(stats)
    }

    /// Get logs for a supplement
    pub async fn get_by_supplement(
        pool: &PgPool,
        supplement_id: Uuid,
        limit: i64,
    ) -> Result<Vec<SupplementLogRecord>> {
        let records = sqlx::query_as::<_, SupplementLogRecord>(
            r#"
            SELECT id, supplement_id, user_id, taken_at, skipped, notes, created_at
            FROM supplement_logs
            WHERE supplement_id = $1
            ORDER BY taken_at DESC
            LIMIT $2
            "#,
        )
        .bind(supplement_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(records)
    }
}
