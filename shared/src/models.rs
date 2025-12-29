//! Data models for the Fitness Assistant application

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export unit types from units module for backward compatibility
pub use crate::units::{DistanceUnit, EnergyUnit, WeightUnit};

/// Data source for health entries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataSource {
    Manual,
    AppleHealth,
    GoogleFit,
    Garmin,
    Oura,
    Whoop,
    Fitbit,
}

/// User account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User settings and preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub user_id: Uuid,
    pub weight_unit: WeightUnit,
    pub distance_unit: DistanceUnit,
    pub energy_unit: EnergyUnit,
    pub timezone: String,
    pub daily_calorie_goal: Option<i32>,
    pub daily_water_goal_ml: Option<i32>,
    pub daily_step_goal: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            user_id: Uuid::nil(),
            weight_unit: WeightUnit::default(),
            distance_unit: DistanceUnit::default(),
            energy_unit: EnergyUnit::default(),
            timezone: "UTC".to_string(),
            daily_calorie_goal: None,
            daily_water_goal_ml: None,
            daily_step_goal: None,
            updated_at: Utc::now(),
        }
    }
}

/// Goal status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GoalStatus {
    Active,
    Completed,
    Abandoned,
}

/// Goal type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoalType {
    Weight,
    BodyFat,
    DailySteps,
    WeeklyWorkouts,
    DailyCalories,
    DailyProtein,
    DailyWater,
    SleepDuration,
    Custom(String),
}

/// Health goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: Uuid,
    pub user_id: Uuid,
    pub goal_type: GoalType,
    pub target_value: f64,
    pub current_value: f64,
    pub start_date: NaiveDate,
    pub target_date: Option<NaiveDate>,
    pub status: GoalStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
