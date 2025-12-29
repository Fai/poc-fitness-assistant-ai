//! Health insights service - calculates health metrics from user data

use crate::error::ApiError;
use crate::repositories::{UserRepository, WeightRepository};
use chrono::Utc;
use fitness_assistant_shared::health_metrics::{
    calculate_bmi_result, calculate_daily_water_ml, calculate_ideal_weight, calculate_tdee_result,
    classify_body_fat, estimate_body_fat_from_bmi, ActivityLevel, BiologicalSex, HealthProfile,
};
use fitness_assistant_shared::types::{
    BmiInfo, BodyFatInfo, EnergyInfo, HealthInsightsResponse, HydrationInfo, IdealWeightInfo,
};
use fitness_assistant_shared::units::WeightUnit;
use fitness_assistant_shared::validation::get_field_display_label;
use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
use uuid::Uuid;

/// Health insights service
pub struct HealthInsightsService;

impl HealthInsightsService {
    /// Get health insights for a user
    /// 
    /// Returns BMI, TDEE, hydration recommendations, and ideal weight based on
    /// user profile data. Missing fields are reported with user-friendly labels.
    pub async fn get_insights(db: &PgPool, user_id: Uuid) -> Result<HealthInsightsResponse, ApiError> {
        let settings = UserRepository::get_settings(db, user_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("Settings not found".to_string()))?;

        let weight_unit: WeightUnit = settings.weight_unit.parse().unwrap_or_default();

        let latest_weight = WeightRepository::get_latest(db, user_id)
            .await
            .map_err(ApiError::Internal)?;

        let weight_kg = latest_weight.map(|w| w.weight_kg.to_f64().unwrap_or(0.0));
        let height_cm = settings.height_cm.map(|h| h.to_f64().unwrap_or(0.0));

        let age_years = settings.date_of_birth.map(|dob| {
            let today = Utc::now().date_naive();
            today.years_since(dob).unwrap_or(0) as i32
        });

        let sex = settings
            .biological_sex
            .as_deref()
            .and_then(|s| match s.to_lowercase().as_str() {
                "male" => Some(BiologicalSex::Male),
                "female" => Some(BiologicalSex::Female),
                _ => None,
            });

        let activity: ActivityLevel = match settings.activity_level.as_str() {
            "sedentary" => ActivityLevel::Sedentary,
            "lightly_active" => ActivityLevel::LightlyActive,
            "moderately_active" => ActivityLevel::ModeratelyActive,
            "very_active" => ActivityLevel::VeryActive,
            "extra_active" => ActivityLevel::ExtraActive,
            _ => ActivityLevel::LightlyActive,
        };

        // Track missing fields with user-friendly labels
        let mut missing_fields = Vec::new();
        if weight_kg.is_none() {
            missing_fields.push(get_field_display_label("weight").to_string());
        }
        if height_cm.is_none() {
            missing_fields.push(get_field_display_label("height").to_string());
        }
        if age_years.is_none() {
            missing_fields.push(get_field_display_label("date_of_birth").to_string());
        }
        if sex.is_none() {
            missing_fields.push(get_field_display_label("biological_sex").to_string());
        }

        let bmi = Self::calculate_bmi(weight_kg, height_cm, &weight_unit);
        let energy = Self::calculate_energy(weight_kg, height_cm, age_years, sex, activity);
        let hydration = Self::calculate_hydration(weight_kg, activity);
        let ideal_weight = Self::calculate_ideal_weight(height_cm, sex, &weight_unit);
        let body_fat = Self::calculate_body_fat(bmi.as_ref(), age_years, sex);

        Ok(HealthInsightsResponse {
            bmi,
            energy,
            hydration,
            ideal_weight,
            body_fat,
            missing_fields,
        })
    }


    fn calculate_bmi(
        weight_kg: Option<f64>,
        height_cm: Option<f64>,
        weight_unit: &WeightUnit,
    ) -> Option<BmiInfo> {
        match (weight_kg, height_cm) {
            (Some(w), Some(h)) if h > 0.0 => {
                let result = calculate_bmi_result(w, h);
                Some(BmiInfo {
                    value: (result.value * 10.0).round() / 10.0,
                    category: result.category.description().to_string(),
                    healthy_weight_min: weight_unit.from_kg(result.healthy_weight_range_kg.0),
                    healthy_weight_max: weight_unit.from_kg(result.healthy_weight_range_kg.1),
                    distance_from_healthy: weight_unit.from_kg(result.distance_from_healthy_kg),
                    unit: weight_unit.to_string(),
                })
            }
            _ => None,
        }
    }

    fn calculate_energy(
        weight_kg: Option<f64>,
        height_cm: Option<f64>,
        age_years: Option<i32>,
        sex: Option<BiologicalSex>,
        activity: ActivityLevel,
    ) -> Option<EnergyInfo> {
        match (weight_kg, height_cm, age_years, sex) {
            (Some(w), Some(h), Some(age), Some(s)) if h > 0.0 && age > 0 => {
                let profile = HealthProfile {
                    height_cm: h,
                    weight_kg: w,
                    age_years: age,
                    sex: s,
                    activity_level: activity,
                };
                let result = calculate_tdee_result(&profile);
                Some(EnergyInfo {
                    bmr: result.bmr.round(),
                    tdee: result.tdee.round(),
                    calories_for_loss: result.calories_for_loss.round(),
                    calories_for_gain: result.calories_for_gain.round(),
                    calories_for_maintenance: result.calories_for_maintenance.round(),
                    unit: "kcal".to_string(),
                })
            }
            _ => None,
        }
    }

    fn calculate_hydration(weight_kg: Option<f64>, activity: ActivityLevel) -> Option<HydrationInfo> {
        weight_kg.map(|w| {
            let ml = calculate_daily_water_ml(w, activity);
            HydrationInfo {
                recommended_ml: ml,
                recommended_cups: (ml as f64 / 250.0 * 10.0).round() / 10.0,
            }
        })
    }

    fn calculate_ideal_weight(
        height_cm: Option<f64>,
        sex: Option<BiologicalSex>,
        weight_unit: &WeightUnit,
    ) -> Option<IdealWeightInfo> {
        match (height_cm, sex) {
            (Some(h), Some(s)) if h > 0.0 => {
                let result = calculate_ideal_weight(h, s);
                let min = result.devine.min(result.robinson).min(result.miller).min(result.hamwi);
                let max = result.devine.max(result.robinson).max(result.miller).max(result.hamwi);
                Some(IdealWeightInfo {
                    average: weight_unit.from_kg(result.average),
                    range_min: weight_unit.from_kg(min),
                    range_max: weight_unit.from_kg(max),
                    unit: weight_unit.to_string(),
                })
            }
            _ => None,
        }
    }

    fn calculate_body_fat(
        bmi: Option<&BmiInfo>,
        age_years: Option<i32>,
        sex: Option<BiologicalSex>,
    ) -> Option<BodyFatInfo> {
        match (bmi, age_years, sex) {
            (Some(b), Some(age), Some(s)) => {
                let estimated = estimate_body_fat_from_bmi(b.value, age, s);
                let category = classify_body_fat(estimated, s);
                Some(BodyFatInfo {
                    estimated_percent: (estimated * 10.0).round() / 10.0,
                    category: format!("{:?}", category),
                    source: "BMI-based estimate".to_string(),
                })
            }
            _ => None,
        }
    }
}
