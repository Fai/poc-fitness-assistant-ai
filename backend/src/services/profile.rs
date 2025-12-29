//! Profile service - business logic for user profile management

use crate::error::ApiError;
use crate::repositories::{UpdateUserSettings, UserRepository};
use chrono::Utc;
use fitness_assistant_shared::types::{
    UpdateProfileRequest, UpdateSettingsRequest, UserProfileResponse, UserSettingsResponse,
};
use fitness_assistant_shared::units::HeightUnit;
use fitness_assistant_shared::validation::{
    get_field_display_label, validate_activity_level, validate_biological_sex,
    validate_date_of_birth, validate_height_cm,
};
use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
use uuid::Uuid;

/// Profile service for user profile operations
pub struct ProfileService;

impl ProfileService {
    /// Get user profile
    pub async fn get_profile(db: &PgPool, user_id: Uuid) -> Result<UserProfileResponse, ApiError> {
        let user = UserRepository::find_by_id(db, user_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;

        let settings = UserRepository::get_settings(db, user_id)
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

        let age_years = dob.map(|d| {
            let today = Utc::now().date_naive();
            today.years_since(d).unwrap_or(0) as i32
        });

        Ok(UserProfileResponse {
            id: user.id.to_string(),
            email: user.email,
            height,
            height_unit,
            date_of_birth: dob,
            age_years,
            biological_sex: sex,
            activity_level: activity,
            created_at: user.created_at,
        })
    }


    /// Validate profile update request
    fn validate_profile_update(req: &UpdateProfileRequest) -> Result<(), ApiError> {
        // Validate height if provided
        if let Some(height) = req.height {
            let unit: HeightUnit = req
                .height_unit
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(HeightUnit::Cm);
            let height_cm = unit.to_cm(height);
            
            if let Err(msg) = validate_height_cm(height_cm) {
                return Err(ApiError::Validation(format!(
                    "{}: {}",
                    get_field_display_label("height"),
                    msg
                )));
            }
        }

        // Validate date of birth if provided
        if let Some(dob) = req.date_of_birth {
            if let Err(msg) = validate_date_of_birth(dob) {
                return Err(ApiError::Validation(format!(
                    "{}: {}",
                    get_field_display_label("date_of_birth"),
                    msg
                )));
            }
        }

        // Validate biological sex if provided
        if let Some(ref sex) = req.biological_sex {
            if let Err(msg) = validate_biological_sex(sex) {
                return Err(ApiError::Validation(format!(
                    "{}: {}",
                    get_field_display_label("biological_sex"),
                    msg
                )));
            }
        }

        // Validate activity level if provided
        if let Some(ref level) = req.activity_level {
            if let Err(msg) = validate_activity_level(level) {
                return Err(ApiError::Validation(format!(
                    "{}: {}",
                    get_field_display_label("activity_level"),
                    msg
                )));
            }
        }

        Ok(())
    }

    /// Update user profile with validation
    pub async fn update_profile(
        db: &PgPool,
        user_id: Uuid,
        req: UpdateProfileRequest,
    ) -> Result<UserProfileResponse, ApiError> {
        // Validate input
        Self::validate_profile_update(&req)?;

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

        UserRepository::update_settings(db, user_id, updates)
            .await
            .map_err(ApiError::Internal)?;

        Self::get_profile(db, user_id).await
    }

    /// Get user settings
    pub async fn get_settings(db: &PgPool, user_id: Uuid) -> Result<UserSettingsResponse, ApiError> {
        let settings = UserRepository::get_settings(db, user_id)
            .await
            .map_err(ApiError::Internal)?
            .ok_or_else(|| ApiError::NotFound("Settings not found".to_string()))?;

        Ok(UserSettingsResponse {
            weight_unit: settings.weight_unit,
            distance_unit: settings.distance_unit,
            energy_unit: settings.energy_unit,
            height_unit: settings.height_unit,
            temperature_unit: settings.temperature_unit,
            timezone: settings.timezone,
            daily_calorie_goal: settings.daily_calorie_goal,
            daily_water_goal_ml: settings.daily_water_goal_ml,
            daily_step_goal: settings.daily_step_goal,
        })
    }

    /// Update user settings
    pub async fn update_settings(
        db: &PgPool,
        user_id: Uuid,
        req: UpdateSettingsRequest,
    ) -> Result<UserSettingsResponse, ApiError> {
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

        UserRepository::update_settings(db, user_id, updates)
            .await
            .map_err(ApiError::Internal)?;

        Self::get_settings(db, user_id).await
    }
}
