//! User profile and settings API routes

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::repositories::{UpdateUserSettings, UserRepository};
use crate::state::AppState;
use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use chrono::Utc;
use fitness_assistant_shared::health_metrics::{
    calculate_bmi_result, calculate_daily_water_ml, calculate_ideal_weight, calculate_tdee_result,
    classify_body_fat, estimate_body_fat_from_bmi, ActivityLevel, BiologicalSex, HealthProfile,
};
use fitness_assistant_shared::types::{
    BmiInfo, BodyFatInfo, EnergyInfo, HealthInsightsResponse, HydrationInfo, IdealWeightInfo,
    UpdateProfileRequest, UpdateSettingsRequest, UserProfileResponse, UserSettingsResponse,
};
use fitness_assistant_shared::units::{HeightUnit, WeightUnit};
use rust_decimal::prelude::ToPrimitive;

/// Create profile routes
pub fn profile_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_profile).put(update_profile))
        .route("/settings", get(get_settings).put(update_settings))
        .route("/insights", get(get_health_insights))
}

/// GET /api/v1/profile - Get user profile
async fn get_profile(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let user = UserRepository::find_by_id(state.db(), auth.user_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

    let settings = UserRepository::get_settings(state.db(), auth.user_id)
        .await
        .map_err(ApiError::Internal)?;

    let (height, height_unit, dob, sex, activity) = if let Some(s) = settings {
        let height_unit: HeightUnit = s.height_unit.parse().unwrap_or_default();
        let height = s.height_cm.map(|h| height_unit.from_cm(h.to_f64().unwrap_or(0.0)));
        (
            height,
            Some(height_unit.to_string()),
            s.date_of_birth,
            s.biological_sex,
            s.activity_level,
        )
    } else {
        (None, None, None, None, "lightly_active".to_string())
    };

    // Calculate age from DOB
    let age_years = dob.map(|d| {
        let today = Utc::now().date_naive();
        let years = today.years_since(d).unwrap_or(0) as i32;
        years
    });

    Ok(Json(UserProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        height,
        height_unit,
        date_of_birth: dob,
        age_years,
        biological_sex: sex,
        activity_level: activity,
        created_at: user.created_at,
    }))
}


/// PUT /api/v1/profile - Update user profile
async fn update_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    // Convert height to cm if provided
    let height_cm = if let Some(height) = req.height {
        let unit: HeightUnit = req
            .height_unit
            .as_deref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(HeightUnit::Cm);
        Some(unit.to_cm(height))
    } else {
        None
    };

    let updates = UpdateUserSettings {
        height_cm,
        date_of_birth: req.date_of_birth,
        biological_sex: req.biological_sex,
        activity_level: req.activity_level,
        height_unit: req.height_unit,
        ..Default::default()
    };

    UserRepository::update_settings(state.db(), auth.user_id, updates)
        .await
        .map_err(ApiError::Internal)?;

    // Return updated profile
    get_profile(State(state), auth).await
}

/// GET /api/v1/profile/settings - Get user settings
async fn get_settings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserSettingsResponse>, ApiError> {
    let settings = UserRepository::get_settings(state.db(), auth.user_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("Settings not found".to_string()))?;

    Ok(Json(UserSettingsResponse {
        weight_unit: settings.weight_unit,
        distance_unit: settings.distance_unit,
        energy_unit: settings.energy_unit,
        height_unit: settings.height_unit,
        temperature_unit: settings.temperature_unit,
        timezone: settings.timezone,
        daily_calorie_goal: settings.daily_calorie_goal,
        daily_water_goal_ml: settings.daily_water_goal_ml,
        daily_step_goal: settings.daily_step_goal,
    }))
}

/// PUT /api/v1/profile/settings - Update user settings
async fn update_settings(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<UserSettingsResponse>, ApiError> {
    let updates = UpdateUserSettings {
        weight_unit: req.weight_unit,
        distance_unit: req.distance_unit,
        energy_unit: req.energy_unit,
        height_unit: req.height_unit,
        temperature_unit: req.temperature_unit,
        timezone: req.timezone,
        daily_calorie_goal: req.daily_calorie_goal,
        daily_water_goal_ml: req.daily_water_goal_ml,
        daily_step_goal: req.daily_step_goal,
        ..Default::default()
    };

    UserRepository::update_settings(state.db(), auth.user_id, updates)
        .await
        .map_err(ApiError::Internal)?;

    get_settings(State(state), auth).await
}


/// GET /api/v1/profile/insights - Get health insights
/// 
/// Returns BMI, TDEE, hydration recommendations, and ideal weight based on
/// user profile data. Missing fields are reported so user can complete profile.
async fn get_health_insights(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<HealthInsightsResponse>, ApiError> {
    let settings = UserRepository::get_settings(state.db(), auth.user_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("Settings not found".to_string()))?;

    // Get user's preferred weight unit
    let weight_unit: WeightUnit = settings.weight_unit.parse().unwrap_or_default();

    // Get latest weight
    let latest_weight = crate::repositories::WeightRepository::get_latest(state.db(), auth.user_id)
        .await
        .map_err(ApiError::Internal)?;

    let weight_kg = latest_weight.map(|w| w.weight_kg.to_f64().unwrap_or(0.0));
    let height_cm = settings.height_cm.map(|h| h.to_f64().unwrap_or(0.0));
    
    // Calculate age from DOB
    let age_years = settings.date_of_birth.map(|dob| {
        let today = Utc::now().date_naive();
        today.years_since(dob).unwrap_or(0) as i32
    });

    // Parse biological sex
    let sex = settings
        .biological_sex
        .as_deref()
        .and_then(|s| match s.to_lowercase().as_str() {
            "male" => Some(BiologicalSex::Male),
            "female" => Some(BiologicalSex::Female),
            _ => None,
        });

    // Parse activity level
    let activity: ActivityLevel = match settings.activity_level.as_str() {
        "sedentary" => ActivityLevel::Sedentary,
        "lightly_active" => ActivityLevel::LightlyActive,
        "moderately_active" => ActivityLevel::ModeratelyActive,
        "very_active" => ActivityLevel::VeryActive,
        "extra_active" => ActivityLevel::ExtraActive,
        _ => ActivityLevel::LightlyActive,
    };

    // Track missing fields
    let mut missing_fields = Vec::new();
    if weight_kg.is_none() {
        missing_fields.push("weight".to_string());
    }
    if height_cm.is_none() {
        missing_fields.push("height".to_string());
    }
    if age_years.is_none() {
        missing_fields.push("date_of_birth".to_string());
    }
    if sex.is_none() {
        missing_fields.push("biological_sex".to_string());
    }

    // Calculate BMI (requires weight and height)
    let bmi = match (weight_kg, height_cm) {
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
    };

    // Calculate TDEE (requires all profile data)
    let energy = match (weight_kg, height_cm, age_years, sex) {
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
    };

    // Calculate hydration (requires weight)
    let hydration = weight_kg.map(|w| {
        let ml = calculate_daily_water_ml(w, activity);
        HydrationInfo {
            recommended_ml: ml,
            recommended_cups: (ml as f64 / 250.0 * 10.0).round() / 10.0,
        }
    });

    // Calculate ideal weight (requires height and sex)
    let ideal_weight = match (height_cm, sex) {
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
    };

    // Estimate body fat (requires BMI, age, and sex)
    let body_fat = match (bmi.as_ref(), age_years, sex) {
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
    };

    Ok(Json(HealthInsightsResponse {
        bmi,
        energy,
        hydration,
        ideal_weight,
        body_fat,
        missing_fields,
    }))
}
