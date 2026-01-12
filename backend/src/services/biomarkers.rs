//! Biomarkers service for biomarker and supplement tracking
//!
//! Provides business logic for:
//! - Biomarker logging and classification
//! - Supplement tracking and adherence calculation
//! - Range classification (low/optimal/high)

use crate::error::ApiError;
use crate::repositories::biomarkers::{
    BiomarkerLogRepository, BiomarkerRangeRepository, CreateBiomarkerLog, CreateSupplement,
    CreateSupplementLog, SupplementLogRepository, SupplementRepository,
};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
use uuid::Uuid;

/// Biomarker range info
#[derive(Debug, Clone)]
pub struct BiomarkerRange {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub category: String,
    pub unit: String,
    pub low_threshold: Option<f64>,
    pub optimal_min: Option<f64>,
    pub optimal_max: Option<f64>,
    pub high_threshold: Option<f64>,
    pub description: Option<String>,
}

/// Biomarker log entry
#[derive(Debug, Clone)]
pub struct BiomarkerLog {
    pub id: Uuid,
    pub biomarker_name: String,
    pub display_name: String,
    pub category: String,
    pub value: f64,
    pub unit: String,
    pub classification: String,
    pub test_date: NaiveDate,
    pub lab_name: Option<String>,
    pub notes: Option<String>,
}

/// Input for logging a biomarker
#[derive(Debug, Clone)]
pub struct LogBiomarkerInput {
    pub biomarker_name: String,
    pub value: f64,
    pub test_date: NaiveDate,
    pub lab_name: Option<String>,
    pub notes: Option<String>,
    pub source: Option<String>,
}

/// Supplement entry
#[derive(Debug, Clone)]
pub struct Supplement {
    pub id: Uuid,
    pub name: String,
    pub brand: Option<String>,
    pub dosage: String,
    pub frequency: String,
    pub time_of_day: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub is_active: bool,
    pub notes: Option<String>,
}

/// Input for creating a supplement
#[derive(Debug, Clone)]
pub struct CreateSupplementInput {
    pub name: String,
    pub brand: Option<String>,
    pub dosage: String,
    pub frequency: String,
    pub time_of_day: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

/// Supplement adherence info
#[derive(Debug, Clone)]
pub struct SupplementAdherence {
    pub supplement_id: Uuid,
    pub supplement_name: String,
    pub total_days: i64,
    pub days_taken: i64,
    pub days_skipped: i64,
    pub adherence_percent: f64,
}

/// Biomarkers service
pub struct BiomarkersService;

impl BiomarkersService {
    /// Get all biomarker ranges
    pub async fn get_ranges(pool: &PgPool) -> Result<Vec<BiomarkerRange>, ApiError> {
        let records = BiomarkerRangeRepository::get_all(pool)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records
            .into_iter()
            .map(|r| BiomarkerRange {
                id: r.id,
                name: r.name,
                display_name: r.display_name,
                category: r.category,
                unit: r.unit,
                low_threshold: r.low_threshold.and_then(|d| d.to_f64()),
                optimal_min: r.optimal_min.and_then(|d| d.to_f64()),
                optimal_max: r.optimal_max.and_then(|d| d.to_f64()),
                high_threshold: r.high_threshold.and_then(|d| d.to_f64()),
                description: r.description,
            })
            .collect())
    }

    /// Log a biomarker value
    pub async fn log_biomarker(
        pool: &PgPool,
        user_id: Uuid,
        input: LogBiomarkerInput,
    ) -> Result<BiomarkerLog, ApiError> {
        // Get the biomarker range
        let range = BiomarkerRangeRepository::get_by_name(pool, &input.biomarker_name)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound(format!("Biomarker '{}' not found", input.biomarker_name)))?;

        // Classify the value
        let classification = Self::classify_value(
            input.value,
            range.low_threshold.and_then(|d| d.to_f64()),
            range.optimal_min.and_then(|d| d.to_f64()),
            range.optimal_max.and_then(|d| d.to_f64()),
            range.high_threshold.and_then(|d| d.to_f64()),
        );

        let create_input = CreateBiomarkerLog {
            user_id,
            biomarker_id: range.id,
            value: Decimal::try_from(input.value).unwrap_or_default(),
            classification: Some(classification.clone()),
            test_date: input.test_date,
            lab_name: input.lab_name,
            notes: input.notes,
            source: input.source.unwrap_or_else(|| "manual".to_string()),
        };

        let _record = BiomarkerLogRepository::create(pool, create_input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(BiomarkerLog {
            id: _record.id,
            biomarker_name: range.name,
            display_name: range.display_name,
            category: range.category,
            value: input.value,
            unit: range.unit,
            classification,
            test_date: input.test_date,
            lab_name: _record.lab_name,
            notes: _record.notes,
        })
    }

    /// Classify a biomarker value against ranges
    ///
    /// # Property 25: Biomarker Range Classification
    /// Returns: critical_low, low, optimal, high, critical_high
    pub fn classify_value(
        value: f64,
        low_threshold: Option<f64>,
        optimal_min: Option<f64>,
        optimal_max: Option<f64>,
        high_threshold: Option<f64>,
    ) -> String {
        // Check critical low
        if let Some(low) = low_threshold {
            if value < low {
                return "critical_low".to_string();
            }
        }

        // Check low (below optimal min but above critical)
        if let Some(opt_min) = optimal_min {
            if value < opt_min {
                return "low".to_string();
            }
        }

        // Check high (above optimal max but below critical)
        if let Some(opt_max) = optimal_max {
            if value > opt_max {
                // Check critical high
                if let Some(high) = high_threshold {
                    if value > high {
                        return "critical_high".to_string();
                    }
                }
                return "high".to_string();
            }
        }

        // Check critical high (if no optimal_max defined)
        if let Some(high) = high_threshold {
            if value > high {
                return "critical_high".to_string();
            }
        }

        "optimal".to_string()
    }

    /// Get biomarker history for a user
    pub async fn get_biomarker_history(
        pool: &PgPool,
        user_id: Uuid,
        biomarker_name: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<BiomarkerLog>, ApiError> {
        let records = BiomarkerLogRepository::get_by_user(pool, user_id, biomarker_name, limit, offset)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records
            .into_iter()
            .map(|r| BiomarkerLog {
                id: r.id,
                biomarker_name: r.biomarker_name,
                display_name: r.display_name,
                category: r.category,
                value: r.value.to_f64().unwrap_or(0.0),
                unit: r.unit,
                classification: r.classification.unwrap_or_else(|| "unknown".to_string()),
                test_date: r.test_date,
                lab_name: r.lab_name,
                notes: r.notes,
            })
            .collect())
    }

    /// Create a supplement
    pub async fn create_supplement(
        pool: &PgPool,
        user_id: Uuid,
        input: CreateSupplementInput,
    ) -> Result<Supplement, ApiError> {
        let valid_frequencies = ["daily", "twice_daily", "weekly", "as_needed"];
        if !valid_frequencies.contains(&input.frequency.as_str()) {
            return Err(ApiError::Validation(format!(
                "Invalid frequency. Must be one of: {}",
                valid_frequencies.join(", ")
            )));
        }

        let create_input = CreateSupplement {
            user_id,
            name: input.name,
            brand: input.brand,
            dosage: input.dosage,
            frequency: input.frequency,
            time_of_day: input.time_of_day,
            start_date: input.start_date.unwrap_or_else(|| Utc::now().date_naive()),
            end_date: input.end_date,
            notes: input.notes,
        };

        let record = SupplementRepository::create(pool, create_input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(Self::record_to_supplement(record))
    }

    /// Get supplements for a user
    pub async fn get_supplements(
        pool: &PgPool,
        user_id: Uuid,
        active_only: bool,
    ) -> Result<Vec<Supplement>, ApiError> {
        let records = SupplementRepository::get_by_user(pool, user_id, active_only)
            .await
            .map_err(ApiError::Internal)?;

        Ok(records.into_iter().map(Self::record_to_supplement).collect())
    }

    /// Log supplement intake
    pub async fn log_supplement(
        pool: &PgPool,
        user_id: Uuid,
        supplement_id: Uuid,
        skipped: bool,
        notes: Option<String>,
    ) -> Result<(), ApiError> {
        let input = CreateSupplementLog {
            supplement_id,
            user_id,
            taken_at: Utc::now(),
            skipped,
            notes,
        };

        SupplementLogRepository::create(pool, input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(())
    }

    /// Calculate supplement adherence
    ///
    /// # Property 26: Supplement Adherence Calculation
    /// adherence = days_taken / total_days * 100
    pub async fn get_adherence(
        pool: &PgPool,
        user_id: Uuid,
        supplement_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<SupplementAdherence, ApiError> {
        // Get supplement info
        let supplements = SupplementRepository::get_by_user(pool, user_id, false)
            .await
            .map_err(ApiError::Internal)?;

        let supplement = supplements
            .into_iter()
            .find(|s| s.id == supplement_id)
            .ok_or_else(|| ApiError::NotFound("Supplement not found".to_string()))?;

        // Get adherence stats
        let stats = SupplementLogRepository::get_adherence(pool, supplement_id, start_date, end_date)
            .await
            .map_err(ApiError::Internal)?;

        // Calculate total expected days
        let total_days = (end_date - start_date).num_days() + 1;
        let adherence_percent = Self::calculate_adherence(stats.total_taken, total_days);

        Ok(SupplementAdherence {
            supplement_id,
            supplement_name: supplement.name,
            total_days,
            days_taken: stats.total_taken,
            days_skipped: stats.total_skipped,
            adherence_percent,
        })
    }

    /// Calculate adherence percentage
    ///
    /// # Property 26: Supplement Adherence Calculation
    /// adherence = days_taken / total_days * 100
    pub fn calculate_adherence(days_taken: i64, total_days: i64) -> f64 {
        if total_days <= 0 {
            return 0.0;
        }
        (days_taken as f64 / total_days as f64) * 100.0
    }

    /// Delete a biomarker log
    pub async fn delete_biomarker_log(
        pool: &PgPool,
        user_id: Uuid,
        log_id: Uuid,
    ) -> Result<bool, ApiError> {
        BiomarkerLogRepository::delete(pool, log_id, user_id)
            .await
            .map_err(ApiError::Internal)
    }

    /// Delete a supplement
    pub async fn delete_supplement(
        pool: &PgPool,
        user_id: Uuid,
        supplement_id: Uuid,
    ) -> Result<bool, ApiError> {
        SupplementRepository::delete(pool, supplement_id, user_id)
            .await
            .map_err(ApiError::Internal)
    }

    fn record_to_supplement(record: crate::repositories::biomarkers::SupplementRecord) -> Supplement {
        Supplement {
            id: record.id,
            name: record.name,
            brand: record.brand,
            dosage: record.dosage,
            frequency: record.frequency,
            time_of_day: record.time_of_day,
            start_date: record.start_date,
            end_date: record.end_date,
            is_active: record.is_active,
            notes: record.notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: fitness-assistant-ai, Property 25: Biomarker Range Classification
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_classify_optimal(value in 50.0f64..80.0) {
            // Value within optimal range
            let classification = BiomarkersService::classify_value(
                value,
                Some(30.0),  // low threshold
                Some(40.0),  // optimal min
                Some(90.0),  // optimal max
                Some(100.0), // high threshold
            );
            prop_assert_eq!(classification, "optimal");
        }

        #[test]
        fn test_classify_low(value in 30.0f64..40.0) {
            // Value below optimal but above critical
            let classification = BiomarkersService::classify_value(
                value,
                Some(20.0),  // low threshold
                Some(40.0),  // optimal min
                Some(90.0),  // optimal max
                Some(100.0), // high threshold
            );
            prop_assert_eq!(classification, "low");
        }

        #[test]
        fn test_classify_critical_low(value in 0.0f64..20.0) {
            let classification = BiomarkersService::classify_value(
                value,
                Some(20.0),  // low threshold
                Some(40.0),  // optimal min
                Some(90.0),  // optimal max
                Some(100.0), // high threshold
            );
            prop_assert_eq!(classification, "critical_low");
        }

        #[test]
        fn test_classify_high(value in 90.0f64..100.0) {
            // Value above optimal but below critical
            let classification = BiomarkersService::classify_value(
                value,
                Some(20.0),  // low threshold
                Some(40.0),  // optimal min
                Some(90.0),  // optimal max
                Some(110.0), // high threshold
            );
            prop_assert_eq!(classification, "high");
        }

        #[test]
        fn test_classify_critical_high(value in 110.0f64..200.0) {
            let classification = BiomarkersService::classify_value(
                value,
                Some(20.0),  // low threshold
                Some(40.0),  // optimal min
                Some(90.0),  // optimal max
                Some(110.0), // high threshold
            );
            prop_assert_eq!(classification, "critical_high");
        }
    }

    // Feature: fitness-assistant-ai, Property 26: Supplement Adherence Calculation
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_adherence_formula(
            days_taken in 0i64..100,
            total_days in 1i64..100
        ) {
            let adherence = BiomarkersService::calculate_adherence(days_taken, total_days);
            let expected = (days_taken as f64 / total_days as f64) * 100.0;
            
            prop_assert!((adherence - expected).abs() < 0.001,
                "Adherence {} != expected {}", adherence, expected);
        }

        #[test]
        fn test_adherence_100_percent(total_days in 1i64..100) {
            let adherence = BiomarkersService::calculate_adherence(total_days, total_days);
            prop_assert!((adherence - 100.0).abs() < 0.001);
        }

        #[test]
        fn test_adherence_0_percent(total_days in 1i64..100) {
            let adherence = BiomarkersService::calculate_adherence(0, total_days);
            prop_assert_eq!(adherence, 0.0);
        }

        #[test]
        fn test_adherence_zero_days_returns_zero(days_taken in 0i64..100) {
            let adherence = BiomarkersService::calculate_adherence(days_taken, 0);
            prop_assert_eq!(adherence, 0.0);
        }
    }

    #[test]
    fn test_classify_with_missing_thresholds() {
        // Only optimal_max defined (like cholesterol)
        assert_eq!(
            BiomarkersService::classify_value(150.0, None, None, Some(200.0), Some(240.0)),
            "optimal"
        );
        assert_eq!(
            BiomarkersService::classify_value(220.0, None, None, Some(200.0), Some(240.0)),
            "high"
        );
        assert_eq!(
            BiomarkersService::classify_value(250.0, None, None, Some(200.0), Some(240.0)),
            "critical_high"
        );
    }

    #[test]
    fn test_classify_hdl_style() {
        // HDL: higher is better, only low threshold and optimal_min
        assert_eq!(
            BiomarkersService::classify_value(70.0, Some(40.0), Some(60.0), None, None),
            "optimal"
        );
        assert_eq!(
            BiomarkersService::classify_value(50.0, Some(40.0), Some(60.0), None, None),
            "low"
        );
        assert_eq!(
            BiomarkersService::classify_value(35.0, Some(40.0), Some(60.0), None, None),
            "critical_low"
        );
    }
}
