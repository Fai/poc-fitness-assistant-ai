//! API request and response types

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Date range for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

/// Error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Authentication tokens response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Login request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// User profile response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}


// ============================================================================
// Weight and Body Composition Types
// ============================================================================

/// Weight log entry request (supports multiple units)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogWeightRequest {
    /// Weight value in the specified unit (defaults to kg)
    pub weight: f64,
    /// Unit of the weight value (kg, lbs, stone)
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default = "Utc::now")]
    pub recorded_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Weight log response (returns in user's preferred unit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightLogResponse {
    pub id: String,
    /// Weight in user's preferred unit
    pub weight: f64,
    /// The unit of the weight value
    pub unit: String,
    /// Weight in kg (always included for consistency)
    pub weight_kg: f64,
    pub recorded_at: DateTime<Utc>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub is_anomaly: bool,
}

/// Weight history query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightHistoryQuery {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Weight trend response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightTrendResponse {
    pub current_weight: f64,
    pub start_weight: f64,
    pub total_change: f64,
    pub average_daily_change: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moving_average_7d: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moving_average_30d: Option<f64>,
    pub entries_count: usize,
}

/// Goal projection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProjectionRequest {
    pub target_weight: f64,
}

/// Goal projection response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProjectionResponse {
    pub target_weight: f64,
    pub current_weight: f64,
    pub weight_to_lose: f64,
    pub average_daily_change: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projected_days: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projected_date: Option<DateTime<Utc>>,
    pub on_track: bool,
}

/// Body composition log request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogBodyCompositionRequest {
    #[serde(default = "Utc::now")]
    pub recorded_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_fat_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muscle_mass_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub water_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bone_mass_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visceral_fat: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Body composition log response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyCompositionResponse {
    pub id: String,
    pub recorded_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_fat_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muscle_mass_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub water_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bone_mass_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visceral_fat: Option<i32>,
    pub source: String,
}


// ============================================================================
// User Profile and Settings Types
// ============================================================================

/// User profile update request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateProfileRequest {
    /// Height value in the specified unit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    /// Height unit (cm, m, in, ft)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_unit: Option<String>,
    /// Date of birth (YYYY-MM-DD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<NaiveDate>,
    /// Biological sex (male/female) for health calculations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biological_sex: Option<String>,
    /// Activity level (sedentary, lightly_active, moderately_active, very_active, extra_active)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_level: Option<String>,
}

/// User settings update request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSettingsRequest {
    /// Preferred weight unit (kg, lbs, stone)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_unit: Option<String>,
    /// Preferred distance unit (km, miles, m)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_unit: Option<String>,
    /// Preferred energy unit (kcal, kj)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy_unit: Option<String>,
    /// Preferred height unit (cm, m, in, ft)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_unit: Option<String>,
    /// Preferred temperature unit (celsius, fahrenheit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature_unit: Option<String>,
    /// Timezone (e.g., "America/New_York")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// Daily calorie goal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_calorie_goal: Option<i32>,
    /// Daily water goal in ml
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_water_goal_ml: Option<i32>,
    /// Daily step goal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_step_goal: Option<i32>,
}

/// User profile response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_of_birth: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_years: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biological_sex: Option<String>,
    pub activity_level: String,
    pub created_at: DateTime<Utc>,
}

/// User settings response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsResponse {
    pub weight_unit: String,
    pub distance_unit: String,
    pub energy_unit: String,
    pub height_unit: String,
    pub temperature_unit: String,
    pub timezone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_calorie_goal: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_water_goal_ml: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_step_goal: Option<i32>,
}

// ============================================================================
// Health Insights Types
// ============================================================================

/// Health insights response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInsightsResponse {
    /// BMI information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bmi: Option<BmiInfo>,
    /// TDEE and calorie information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy: Option<EnergyInfo>,
    /// Hydration recommendation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hydration: Option<HydrationInfo>,
    /// Ideal weight information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ideal_weight: Option<IdealWeightInfo>,
    /// Body fat estimation (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_fat: Option<BodyFatInfo>,
    /// Missing profile fields needed for complete insights
    pub missing_fields: Vec<String>,
}

/// BMI information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmiInfo {
    pub value: f64,
    pub category: String,
    pub healthy_weight_min: f64,
    pub healthy_weight_max: f64,
    pub distance_from_healthy: f64,
    pub unit: String,
}

/// Energy/calorie information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyInfo {
    /// Basal Metabolic Rate
    pub bmr: f64,
    /// Total Daily Energy Expenditure
    pub tdee: f64,
    /// Calories for weight loss (500 deficit)
    pub calories_for_loss: f64,
    /// Calories for weight gain (500 surplus)
    pub calories_for_gain: f64,
    /// Calories for maintenance
    pub calories_for_maintenance: f64,
    pub unit: String,
}

/// Hydration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrationInfo {
    pub recommended_ml: i32,
    pub recommended_cups: f64,
}

/// Ideal weight information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdealWeightInfo {
    pub average: f64,
    pub range_min: f64,
    pub range_max: f64,
    pub unit: String,
}

/// Body fat information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyFatInfo {
    pub estimated_percent: f64,
    pub category: String,
    pub source: String,
}


// ============================================================================
// Nutrition Types
// ============================================================================

/// Food search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodSearchQuery {
    pub q: String,
    #[serde(default)]
    pub limit: Option<i64>,
}

/// Food item response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodItemResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub barcode: Option<String>,
    pub serving_size: f64,
    pub serving_unit: String,
    pub calories: f64,
    pub protein_g: f64,
    pub carbohydrates_g: f64,
    pub fat_g: f64,
    pub fiber_g: f64,
    pub sugar_g: f64,
    pub source: String,
    pub verified: bool,
}

/// Log food request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFoodRequest {
    /// ID of the food item (required unless custom_name is provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub food_item_id: Option<String>,
    /// Number of servings consumed
    pub servings: f64,
    /// Meal type: breakfast, lunch, dinner, snack
    pub meal_type: String,
    /// When the food was consumed (defaults to now)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumed_at: Option<DateTime<Utc>>,
    /// Optional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Food log response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodLogResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub food_item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub food_name: Option<String>,
    pub servings: f64,
    pub calories: f64,
    pub protein_g: f64,
    pub carbohydrates_g: f64,
    pub fat_g: f64,
    pub fiber_g: f64,
    pub meal_type: String,
    pub consumed_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Daily nutrition summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyNutritionResponse {
    pub date: NaiveDate,
    pub total_calories: f64,
    pub total_protein_g: f64,
    pub total_carbs_g: f64,
    pub total_fat_g: f64,
    pub total_fiber_g: f64,
    pub meal_count: i64,
    pub logs: Vec<FoodLogResponse>,
}

/// Create recipe request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecipeRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Number of servings the recipe makes
    pub servings: f64,
    /// Whether the recipe is public
    #[serde(default)]
    pub is_public: bool,
    /// Initial ingredients (optional)
    #[serde(default)]
    pub ingredients: Vec<RecipeIngredientInput>,
}

/// Recipe ingredient input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredientInput {
    pub food_item_id: String,
    pub servings: f64,
    #[serde(default)]
    pub sort_order: i32,
}

/// Recipe response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub servings: f64,
    pub calories_per_serving: f64,
    pub protein_per_serving: f64,
    pub carbs_per_serving: f64,
    pub fat_per_serving: f64,
    pub fiber_per_serving: f64,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
}

/// Recipe with ingredients response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDetailResponse {
    pub recipe: RecipeResponse,
    pub ingredients: Vec<RecipeIngredientResponse>,
}

/// Recipe ingredient response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredientResponse {
    pub id: String,
    pub food_item_id: String,
    pub food_name: String,
    pub servings: f64,
    pub sort_order: i32,
}

/// Add ingredient to recipe request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddIngredientRequest {
    pub food_item_id: String,
    pub servings: f64,
    #[serde(default)]
    pub sort_order: i32,
}

/// Date query parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateQuery {
    pub date: NaiveDate,
}
