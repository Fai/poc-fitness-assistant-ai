//! Biometrics service for heart rate and HRV tracking
//!
//! Provides business logic for:
//! - Heart rate logging and analysis
//! - HRV tracking and recovery score calculation
//! - Heart rate zone management
//! - Resting heart rate anomaly detection

use crate::error::ApiError;
use crate::repositories::{
    biometrics::{
        CreateHeartRateLog, CreateHrvLog, HeartRateLogRepository, HeartRateZonesRepository,
        HrvLogRepository,
    },
    UserRepository,
};
use chrono::{DateTime, Datelike, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
use uuid::Uuid;

/// Default max heart rate calculation: 220 - age
const DEFAULT_MAX_HR_FORMULA_BASE: i32 = 220;

/// Anomaly threshold for resting heart rate (10% deviation)
const RESTING_HR_ANOMALY_THRESHOLD: f64 = 0.10;

/// Days for baseline calculation
const BASELINE_DAYS: i32 = 7;

/// Heart rate log entry
#[derive(Debug, Clone)]
pub struct HeartRateLog {
    pub id: Uuid,
    pub bpm: i32,
    pub context: String,
    pub recorded_at: DateTime<Utc>,
    pub workout_id: Option<Uuid>,
    pub source: String,
    pub notes: Option<String>,
}

/// Input for logging heart rate
#[derive(Debug, Clone)]
pub struct LogHeartRateInput {
    pub bpm: i32,
    pub context: Option<String>,
    pub recorded_at: Option<DateTime<Utc>>,
    pub workout_id: Option<Uuid>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

/// HRV log entry
#[derive(Debug, Clone)]
pub struct HrvLog {
    pub id: Uuid,
    pub rmssd: f64,
    pub sdnn: Option<f64>,
    pub context: String,
    pub recorded_at: DateTime<Utc>,
    pub source: String,
    pub notes: Option<String>,
}

/// Input for logging HRV
#[derive(Debug, Clone)]
pub struct LogHrvInput {
    pub rmssd: f64,
    pub sdnn: Option<f64>,
    pub context: Option<String>,
    pub recorded_at: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

/// Recovery score result
#[derive(Debug, Clone)]
pub struct RecoveryScore {
    pub score: f64,
    pub hrv_current: f64,
    pub hrv_baseline: f64,
    pub resting_hr_current: Option<i32>,
    pub resting_hr_baseline: Option<f64>,
    pub status: String,
}

/// Heart rate zone
#[derive(Debug, Clone)]
pub struct HeartRateZone {
    pub zone: i32,
    pub name: String,
    pub min_bpm: i32,
    pub max_bpm: i32,
}

/// Heart rate zones configuration
#[derive(Debug, Clone)]
pub struct HeartRateZones {
    pub max_heart_rate: i32,
    pub resting_heart_rate: Option<i32>,
    pub zones: Vec<HeartRateZone>,
    pub calculation_method: String,
}

/// Zone time distribution
#[derive(Debug, Clone)]
pub struct ZoneDistribution {
    pub zone: i32,
    pub name: String,
    pub duration_seconds: i32,
    pub percentage: f64,
}

/// Resting HR analysis result
#[derive(Debug, Clone)]
pub struct RestingHrAnalysis {
    pub current_avg: f64,
    pub baseline_avg: f64,
    pub deviation_percent: f64,
    pub is_anomaly: bool,
    pub trend: String,
}

/// Biometrics service for business logic
pub struct BiometricsService;

impl BiometricsService {
    /// Log a heart rate reading
    pub async fn log_heart_rate(
        pool: &PgPool,
        user_id: Uuid,
        input: LogHeartRateInput,
    ) -> Result<HeartRateLog, ApiError> {
        // Validate BPM
        if input.bpm <= 0 || input.bpm >= 300 {
            return Err(ApiError::Validation(
                "Heart rate must be between 1 and 299 BPM".to_string(),
            ));
        }

        let context = input.context.unwrap_or_else(|| "resting".to_string());
        let valid_contexts = ["resting", "active", "workout", "sleep", "recovery"];
        if !valid_contexts.contains(&context.as_str()) {
            return Err(ApiError::Validation(format!(
                "Invalid context. Must be one of: {}",
                valid_contexts.join(", ")
            )));
        }

        let create_input = CreateHeartRateLog {
            user_id,
            bpm: input.bpm,
            context,
            recorded_at: input.recorded_at.unwrap_or_else(Utc::now),
            workout_id: input.workout_id,
            source: input.source.unwrap_or_else(|| "manual".to_string()),
            notes: input.notes,
        };

        let record = HeartRateLogRepository::create(pool, create_input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(HeartRateLog {
            id: record.id,
            bpm: record.bpm,
            context: record.context,
            recorded_at: record.recorded_at,
            workout_id: record.workout_id,
            source: record.source,
            notes: record.notes,
        })
    }

    /// Log an HRV reading
    pub async fn log_hrv(
        pool: &PgPool,
        user_id: Uuid,
        input: LogHrvInput,
    ) -> Result<HrvLog, ApiError> {
        // Validate RMSSD
        if input.rmssd <= 0.0 || input.rmssd >= 500.0 {
            return Err(ApiError::Validation(
                "RMSSD must be between 0 and 500 ms".to_string(),
            ));
        }

        if let Some(sdnn) = input.sdnn {
            if sdnn <= 0.0 || sdnn >= 500.0 {
                return Err(ApiError::Validation(
                    "SDNN must be between 0 and 500 ms".to_string(),
                ));
            }
        }

        let context = input.context.unwrap_or_else(|| "morning".to_string());
        let valid_contexts = ["morning", "sleep", "recovery", "workout"];
        if !valid_contexts.contains(&context.as_str()) {
            return Err(ApiError::Validation(format!(
                "Invalid context. Must be one of: {}",
                valid_contexts.join(", ")
            )));
        }

        let create_input = CreateHrvLog {
            user_id,
            rmssd: Decimal::try_from(input.rmssd).unwrap_or_default(),
            sdnn: input.sdnn.map(|s| Decimal::try_from(s).unwrap_or_default()),
            context,
            recorded_at: input.recorded_at.unwrap_or_else(Utc::now),
            source: input.source.unwrap_or_else(|| "manual".to_string()),
            notes: input.notes,
        };

        let record = HrvLogRepository::create(pool, create_input)
            .await
            .map_err(ApiError::Internal)?;

        Ok(HrvLog {
            id: record.id,
            rmssd: record.rmssd.to_f64().unwrap_or(0.0),
            sdnn: record.sdnn.and_then(|d| d.to_f64()),
            context: record.context,
            recorded_at: record.recorded_at,
            source: record.source,
            notes: record.notes,
        })
    }

    /// Calculate recovery score from HRV
    ///
    /// # Property 17: Recovery Score Calculation
    /// score = normalize(hrv / baseline) * 100
    pub async fn get_recovery_score(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<RecoveryScore, ApiError> {
        let today = Utc::now().date_naive();
        
        // Get latest HRV
        let latest_hrv = HrvLogRepository::get_latest(pool, user_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("No HRV data found".to_string()))?;

        // Get HRV baseline (7-day average)
        let hrv_baseline = HrvLogRepository::get_baseline(pool, user_id, today, BASELINE_DAYS)
            .await
            .map_err(ApiError::Internal)?
            .unwrap_or(latest_hrv.rmssd.to_f64().unwrap_or(50.0));

        let hrv_current = latest_hrv.rmssd.to_f64().unwrap_or(0.0);

        // Get resting HR data
        let resting_hr_baseline = HeartRateLogRepository::get_resting_baseline(
            pool, user_id, today, BASELINE_DAYS
        )
        .await
        .map_err(ApiError::Internal)?;

        // Calculate recovery score
        let score = Self::calculate_recovery_score(hrv_current, hrv_baseline);
        let status = Self::recovery_status(score);

        Ok(RecoveryScore {
            score,
            hrv_current,
            hrv_baseline,
            resting_hr_current: None, // Would need latest resting HR
            resting_hr_baseline,
            status,
        })
    }

    /// Calculate recovery score from HRV values
    ///
    /// # Property 17: Recovery Score Calculation
    /// score = normalize(hrv / baseline) * 100, capped at 0-100
    pub fn calculate_recovery_score(hrv_current: f64, hrv_baseline: f64) -> f64 {
        if hrv_baseline <= 0.0 {
            return 50.0; // Default to neutral if no baseline
        }
        
        let ratio = hrv_current / hrv_baseline;
        // Normalize: ratio of 1.0 = 100%, cap between 0-100
        let score = ratio * 100.0;
        score.clamp(0.0, 100.0)
    }

    /// Get recovery status from score
    fn recovery_status(score: f64) -> String {
        match score {
            s if s >= 80.0 => "excellent".to_string(),
            s if s >= 60.0 => "good".to_string(),
            s if s >= 40.0 => "moderate".to_string(),
            s if s >= 20.0 => "low".to_string(),
            _ => "poor".to_string(),
        }
    }

    /// Get or calculate heart rate zones
    pub async fn get_heart_rate_zones(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<HeartRateZones, ApiError> {
        // Check if user has custom zones
        if let Some(record) = HeartRateZonesRepository::get_by_user(pool, user_id)
            .await
            .map_err(ApiError::Internal)?
        {
            return Ok(HeartRateZones {
                max_heart_rate: record.max_heart_rate,
                resting_heart_rate: record.resting_heart_rate,
                zones: vec![
                    HeartRateZone { zone: 1, name: "Recovery".to_string(), min_bpm: record.zone1_min, max_bpm: record.zone1_max },
                    HeartRateZone { zone: 2, name: "Aerobic".to_string(), min_bpm: record.zone2_min, max_bpm: record.zone2_max },
                    HeartRateZone { zone: 3, name: "Tempo".to_string(), min_bpm: record.zone3_min, max_bpm: record.zone3_max },
                    HeartRateZone { zone: 4, name: "Threshold".to_string(), min_bpm: record.zone4_min, max_bpm: record.zone4_max },
                    HeartRateZone { zone: 5, name: "VO2 Max".to_string(), min_bpm: record.zone5_min, max_bpm: record.zone5_max },
                ],
                calculation_method: record.calculation_method,
            });
        }

        // Calculate default zones based on age
        let max_hr = Self::calculate_max_heart_rate(pool, user_id).await?;
        let zones = Self::calculate_zones_percentage(max_hr);

        Ok(HeartRateZones {
            max_heart_rate: max_hr,
            resting_heart_rate: None,
            zones,
            calculation_method: "percentage".to_string(),
        })
    }

    /// Calculate max heart rate from user's age
    async fn calculate_max_heart_rate(pool: &PgPool, user_id: Uuid) -> Result<i32, ApiError> {
        // Get user settings which contains date_of_birth
        let settings = UserRepository::get_settings(pool, user_id)
            .await
            .map_err(ApiError::Internal)?;

        let age = settings
            .and_then(|s| s.date_of_birth)
            .map(|dob| {
                let today = Utc::now().date_naive();
                today.year() - dob.year()
            })
            .unwrap_or(30); // Default to 30 if no DOB

        Ok(DEFAULT_MAX_HR_FORMULA_BASE - age)
    }

    /// Calculate zones as percentage of max HR
    pub fn calculate_zones_percentage(max_hr: i32) -> Vec<HeartRateZone> {
        vec![
            HeartRateZone {
                zone: 1,
                name: "Recovery".to_string(),
                min_bpm: (max_hr as f64 * 0.50) as i32,
                max_bpm: (max_hr as f64 * 0.60) as i32,
            },
            HeartRateZone {
                zone: 2,
                name: "Aerobic".to_string(),
                min_bpm: (max_hr as f64 * 0.60) as i32,
                max_bpm: (max_hr as f64 * 0.70) as i32,
            },
            HeartRateZone {
                zone: 3,
                name: "Tempo".to_string(),
                min_bpm: (max_hr as f64 * 0.70) as i32,
                max_bpm: (max_hr as f64 * 0.80) as i32,
            },
            HeartRateZone {
                zone: 4,
                name: "Threshold".to_string(),
                min_bpm: (max_hr as f64 * 0.80) as i32,
                max_bpm: (max_hr as f64 * 0.90) as i32,
            },
            HeartRateZone {
                zone: 5,
                name: "VO2 Max".to_string(),
                min_bpm: (max_hr as f64 * 0.90) as i32,
                max_bpm: max_hr,
            },
        ]
    }

    /// Calculate time spent in each zone during a workout
    ///
    /// # Property 18: Heart Rate Zone Distribution
    /// Zone times sum to workout duration
    pub fn calculate_zone_distribution(
        heart_rates: &[(i32, i32)], // (bpm, duration_seconds)
        zones: &[HeartRateZone],
    ) -> Vec<ZoneDistribution> {
        let mut zone_times: Vec<i32> = vec![0; zones.len()];
        let mut total_time = 0;

        for (bpm, duration) in heart_rates {
            total_time += duration;
            for (i, zone) in zones.iter().enumerate() {
                if *bpm >= zone.min_bpm && *bpm <= zone.max_bpm {
                    zone_times[i] += duration;
                    break;
                }
            }
        }

        zones
            .iter()
            .enumerate()
            .map(|(i, zone)| {
                let duration = zone_times[i];
                let percentage = if total_time > 0 {
                    (duration as f64 / total_time as f64) * 100.0
                } else {
                    0.0
                };
                ZoneDistribution {
                    zone: zone.zone,
                    name: zone.name.clone(),
                    duration_seconds: duration,
                    percentage,
                }
            })
            .collect()
    }

    /// Detect resting heart rate anomalies
    ///
    /// # Property 19: Resting Heart Rate Anomaly Detection
    /// Flag readings >10% deviation from baseline
    pub async fn analyze_resting_hr(
        pool: &PgPool,
        user_id: Uuid,
        days: i32,
    ) -> Result<RestingHrAnalysis, ApiError> {
        let today = Utc::now().date_naive();
        let start_date = today - chrono::Duration::days(days as i64);

        // Get current period average
        let stats = HeartRateLogRepository::get_stats(
            pool, user_id, start_date, today, Some("resting")
        )
        .await
        .map_err(ApiError::Internal)?;

        let current_avg = stats.avg_bpm.unwrap_or(0.0);

        // Get baseline (previous period)
        let baseline_end = start_date - chrono::Duration::days(1);
        let baseline_start = baseline_end - chrono::Duration::days(days as i64);
        
        let baseline_stats = HeartRateLogRepository::get_stats(
            pool, user_id, baseline_start, baseline_end, Some("resting")
        )
        .await
        .map_err(ApiError::Internal)?;

        let baseline_avg = baseline_stats.avg_bpm.unwrap_or(current_avg);

        let (deviation_percent, is_anomaly) = 
            Self::detect_hr_anomaly(current_avg, baseline_avg);

        let trend = if current_avg > baseline_avg {
            "increasing".to_string()
        } else if current_avg < baseline_avg {
            "decreasing".to_string()
        } else {
            "stable".to_string()
        };

        Ok(RestingHrAnalysis {
            current_avg,
            baseline_avg,
            deviation_percent,
            is_anomaly,
            trend,
        })
    }

    /// Detect if heart rate deviates more than threshold from baseline
    ///
    /// # Property 19: Resting Heart Rate Anomaly Detection
    /// Returns (deviation_percent, is_anomaly)
    pub fn detect_hr_anomaly(current: f64, baseline: f64) -> (f64, bool) {
        if baseline <= 0.0 {
            return (0.0, false);
        }
        
        let deviation = ((current - baseline) / baseline).abs();
        let deviation_percent = deviation * 100.0;
        let is_anomaly = deviation > RESTING_HR_ANOMALY_THRESHOLD;
        
        (deviation_percent, is_anomaly)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: fitness-assistant-ai, Property 17: Recovery Score Calculation
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_recovery_score_formula(
            hrv_current in 10.0f64..200.0,
            hrv_baseline in 10.0f64..200.0
        ) {
            let score = BiometricsService::calculate_recovery_score(hrv_current, hrv_baseline);
            
            // Score should be clamped between 0 and 100
            prop_assert!(score >= 0.0 && score <= 100.0,
                "Score {} out of bounds for current={}, baseline={}",
                score, hrv_current, hrv_baseline);
        }

        #[test]
        fn test_recovery_score_at_baseline(hrv in 20.0f64..150.0) {
            let score = BiometricsService::calculate_recovery_score(hrv, hrv);
            
            // When current equals baseline, score should be 100
            prop_assert!((score - 100.0).abs() < 0.001,
                "Score should be 100 when current equals baseline, got {}", score);
        }

        #[test]
        fn test_recovery_score_above_baseline(
            baseline in 20.0f64..100.0,
            multiplier in 1.01f64..1.5
        ) {
            let current = baseline * multiplier;
            let score = BiometricsService::calculate_recovery_score(current, baseline);
            
            // Score should be capped at 100 even if HRV is above baseline
            prop_assert_eq!(score, 100.0,
                "Score should be capped at 100 for current={}, baseline={}", current, baseline);
        }

        #[test]
        fn test_recovery_score_below_baseline(
            baseline in 50.0f64..150.0,
            ratio in 0.1f64..0.99
        ) {
            let current = baseline * ratio;
            let score = BiometricsService::calculate_recovery_score(current, baseline);
            let expected = ratio * 100.0;
            
            prop_assert!((score - expected).abs() < 0.001,
                "Score {} != expected {} for ratio {}", score, expected, ratio);
        }

        #[test]
        fn test_recovery_score_zero_baseline(hrv_current in 0.0f64..200.0) {
            let score = BiometricsService::calculate_recovery_score(hrv_current, 0.0);
            
            // Should return default 50 when baseline is 0
            prop_assert_eq!(score, 50.0);
        }
    }

    // Feature: fitness-assistant-ai, Property 18: Heart Rate Zone Distribution
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_zone_distribution_sums_to_total(
            durations in prop::collection::vec(1i32..60, 1..10)
        ) {
            let zones = BiometricsService::calculate_zones_percentage(180);
            
            // Create heart rate readings in various zones
            let heart_rates: Vec<(i32, i32)> = durations.iter().enumerate()
                .map(|(i, &d)| {
                    let zone_idx = i % zones.len();
                    let bpm = (zones[zone_idx].min_bpm + zones[zone_idx].max_bpm) / 2;
                    (bpm, d)
                })
                .collect();

            let total_input: i32 = durations.iter().sum();
            let distribution = BiometricsService::calculate_zone_distribution(&heart_rates, &zones);
            let total_output: i32 = distribution.iter().map(|z| z.duration_seconds).sum();

            prop_assert_eq!(total_input, total_output,
                "Zone times {} should sum to total duration {}", total_output, total_input);
        }

        #[test]
        fn test_zone_percentages_sum_to_100(
            durations in prop::collection::vec(1i32..60, 1..10)
        ) {
            let zones = BiometricsService::calculate_zones_percentage(180);
            
            let heart_rates: Vec<(i32, i32)> = durations.iter().enumerate()
                .map(|(i, &d)| {
                    let zone_idx = i % zones.len();
                    let bpm = (zones[zone_idx].min_bpm + zones[zone_idx].max_bpm) / 2;
                    (bpm, d)
                })
                .collect();

            let distribution = BiometricsService::calculate_zone_distribution(&heart_rates, &zones);
            let total_percent: f64 = distribution.iter().map(|z| z.percentage).sum();

            prop_assert!((total_percent - 100.0).abs() < 0.01,
                "Zone percentages {} should sum to 100", total_percent);
        }
    }

    // Feature: fitness-assistant-ai, Property 19: Resting Heart Rate Anomaly Detection
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn test_hr_anomaly_above_threshold(
            baseline in 50.0f64..100.0,
            extra in 0.02f64..0.5
        ) {
            // More than 10% deviation (use 12%+ to avoid floating point edge cases)
            let current = baseline * (1.10 + extra);
            let (_, is_anomaly) = BiometricsService::detect_hr_anomaly(current, baseline);
            
            prop_assert!(is_anomaly,
                "Should be anomaly above 10% deviation");
        }

        #[test]
        fn test_hr_anomaly_below_threshold(
            baseline in 50.0f64..100.0,
            ratio in 0.92f64..1.08
        ) {
            let current = baseline * ratio;
            let (deviation, is_anomaly) = BiometricsService::detect_hr_anomaly(current, baseline);
            
            prop_assert!(deviation < 10.0,
                "Deviation {} should be < 10%", deviation);
            prop_assert!(!is_anomaly,
                "Should not be anomaly below 10% deviation");
        }

        #[test]
        fn test_hr_anomaly_symmetric(
            baseline in 50.0f64..100.0,
            deviation_pct in 0.0f64..0.3
        ) {
            let current_high = baseline * (1.0 + deviation_pct);
            let current_low = baseline * (1.0 - deviation_pct);
            
            let (dev_high, _) = BiometricsService::detect_hr_anomaly(current_high, baseline);
            let (dev_low, _) = BiometricsService::detect_hr_anomaly(current_low, baseline);
            
            prop_assert!((dev_high - dev_low).abs() < 0.01,
                "Deviation should be symmetric: high={}, low={}", dev_high, dev_low);
        }

        #[test]
        fn test_hr_anomaly_zero_baseline(current in 0.0f64..200.0) {
            let (deviation, is_anomaly) = BiometricsService::detect_hr_anomaly(current, 0.0);
            
            prop_assert_eq!(deviation, 0.0);
            prop_assert!(!is_anomaly);
        }
    }

    #[test]
    fn test_zones_cover_full_range() {
        let zones = BiometricsService::calculate_zones_percentage(200);
        
        // Zone 1 should start at 50% of max
        assert_eq!(zones[0].min_bpm, 100);
        // Zone 5 should end at max
        assert_eq!(zones[4].max_bpm, 200);
        
        // Zones should be contiguous
        for i in 0..zones.len() - 1 {
            assert_eq!(zones[i].max_bpm, zones[i + 1].min_bpm,
                "Zone {} max should equal zone {} min", i + 1, i + 2);
        }
    }

    #[test]
    fn test_recovery_status_categories() {
        assert_eq!(BiometricsService::recovery_status(90.0), "excellent");
        assert_eq!(BiometricsService::recovery_status(70.0), "good");
        assert_eq!(BiometricsService::recovery_status(50.0), "moderate");
        assert_eq!(BiometricsService::recovery_status(30.0), "low");
        assert_eq!(BiometricsService::recovery_status(10.0), "poor");
    }
}
