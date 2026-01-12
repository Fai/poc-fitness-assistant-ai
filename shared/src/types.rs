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

/// Pagination query parameters for API endpoints
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaginationQuery {
    /// Number of items to return (default: 50, max: 100)
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Number of items to skip (default: 0)
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

impl PaginationQuery {
    /// Normalize pagination parameters to valid ranges
    pub fn normalize(&self) -> Self {
        Self {
            limit: self.limit.clamp(1, 100),
            offset: self.offset.max(0),
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

/// Paginated list response with offset-based pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedList<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

impl<T> PaginatedList<T> {
    pub fn new(items: Vec<T>, total_count: i64, limit: i64, offset: i64) -> Self {
        let has_more = offset + (items.len() as i64) < total_count;
        Self {
            items,
            total_count,
            limit,
            offset,
            has_more,
        }
    }
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeightHistoryQuery {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    /// Number of items to return (default: 50, max: 100)
    #[serde(default = "default_weight_limit")]
    pub limit: i64,
    /// Number of items to skip (default: 0)
    #[serde(default)]
    pub offset: i64,
}

fn default_weight_limit() -> i64 {
    50
}

impl WeightHistoryQuery {
    /// Normalize query parameters to valid ranges
    pub fn normalize(&self) -> Self {
        Self {
            start: self.start,
            end: self.end,
            limit: self.limit.clamp(1, 100),
            offset: self.offset.max(0),
        }
    }
}

/// Paginated weight history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightHistoryResponse {
    pub items: Vec<WeightLogResponse>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
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


// ============================================================================
// Exercise and Workout Types
// ============================================================================

/// Exercise response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseResponse {
    pub id: String,
    pub name: String,
    pub category: String,
    pub muscle_groups: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories_per_minute: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    pub is_custom: bool,
}

/// Exercise library query parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExerciseLibraryQuery {
    /// Filter by category (strength, cardio, flexibility, hiit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Filter by muscle group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muscle_group: Option<String>,
    /// Search by name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    /// Include user's custom exercises
    #[serde(default)]
    pub include_custom: bool,
    /// Limit results
    #[serde(default = "default_exercise_limit")]
    pub limit: i64,
}

fn default_exercise_limit() -> i64 {
    50
}

/// Create custom exercise request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExerciseRequest {
    pub name: String,
    pub category: String,
    pub muscle_groups: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories_per_minute: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// Log workout request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogWorkoutRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Workout type: strength, cardio, flexibility, hiit, mixed
    pub workout_type: String,
    #[serde(default = "Utc::now")]
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_minutes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories_burned: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_heart_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_heart_rate: Option<i32>,
    /// Distance in meters (for cardio)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_meters: Option<f64>,
    /// Elevation gain in meters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_gain_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Exercises performed in this workout
    #[serde(default)]
    pub exercises: Vec<WorkoutExerciseInput>,
}

/// Exercise input for workout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutExerciseInput {
    pub exercise_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default)]
    pub sets: Vec<ExerciseSetInput>,
}

/// Exercise set input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseSetInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reps: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_seconds: Option<i32>,
    /// Rate of Perceived Exertion (1-10)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpe: Option<f64>,
    #[serde(default)]
    pub is_warmup: bool,
    #[serde(default)]
    pub is_dropset: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Workout response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutResponse {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub workout_type: String,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_minutes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calories_burned: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_heart_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_heart_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_meters: Option<f64>,
    /// Pace in seconds per kilometer (calculated for cardio)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pace_seconds_per_km: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_gain_meters: Option<f64>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Workout detail response with exercises
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutDetailResponse {
    pub workout: WorkoutResponse,
    pub exercises: Vec<WorkoutExerciseResponse>,
}

/// Workout exercise response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutExerciseResponse {
    pub id: String,
    pub exercise: ExerciseResponse,
    pub sort_order: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub sets: Vec<ExerciseSetResponse>,
}

/// Exercise set response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseSetResponse {
    pub id: String,
    pub set_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reps: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_kg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpe: Option<f64>,
    pub is_warmup: bool,
    pub is_dropset: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Workout history query parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkoutHistoryQuery {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    #[serde(default = "default_workout_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_workout_limit() -> i64 {
    50
}

impl WorkoutHistoryQuery {
    pub fn normalize(&self) -> Self {
        Self {
            start: self.start,
            end: self.end,
            limit: self.limit.clamp(1, 100),
            offset: self.offset.max(0),
        }
    }
}

/// Paginated workout history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutHistoryResponse {
    pub items: Vec<WorkoutResponse>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

/// Weekly exercise summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyExerciseSummaryResponse {
    pub week_start: NaiveDate,
    pub week_end: NaiveDate,
    pub total_workouts: usize,
    pub total_duration_minutes: i32,
    pub total_calories_burned: i32,
    pub workouts_by_type: Vec<WorkoutTypeSummaryResponse>,
    pub daily_breakdown: Vec<DailyWorkoutSummaryResponse>,
}

/// Workout type summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutTypeSummaryResponse {
    pub workout_type: String,
    pub count: usize,
    pub total_duration_minutes: i32,
    pub total_calories: i32,
}

/// Daily workout summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyWorkoutSummaryResponse {
    pub date: NaiveDate,
    pub workouts: usize,
    pub duration_minutes: i32,
    pub calories_burned: i32,
}


// ============================================================================
// Hydration Types
// ============================================================================

/// Log hydration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogHydrationRequest {
    /// Amount in milliliters
    pub amount_ml: i32,
    /// Beverage type (water, tea, coffee, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beverage_type: Option<String>,
    /// When consumed (defaults to now)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumed_at: Option<DateTime<Utc>>,
    /// Source of entry (manual, quick, device)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Optional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Hydration log response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrationLogResponse {
    pub id: String,
    pub amount_ml: i32,
    pub beverage_type: String,
    pub consumed_at: DateTime<Utc>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Daily hydration summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyHydrationResponse {
    pub date: NaiveDate,
    pub total_ml: i64,
    pub goal_ml: i32,
    pub progress_percent: f64,
    pub goal_met: bool,
    pub entry_count: i64,
    pub entries: Vec<HydrationLogResponse>,
}

/// Hydration goal response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrationGoalResponse {
    pub daily_goal_ml: i32,
    pub is_auto_calculated: bool,
    pub reminders_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reminder_interval_minutes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reminder_start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reminder_end_time: Option<String>,
}

/// Set hydration goal request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetHydrationGoalRequest {
    /// Manual daily goal in ml (ignored if auto_calculate is true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_goal_ml: Option<i32>,
    /// Whether to auto-calculate based on weight and activity
    #[serde(default)]
    pub auto_calculate: bool,
    /// Enable reminder notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reminders_enabled: Option<bool>,
    /// Reminder interval in minutes (15-480)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reminder_interval_minutes: Option<i32>,
    /// Reminder start time (HH:MM format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reminder_start_time: Option<String>,
    /// Reminder end time (HH:MM format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reminder_end_time: Option<String>,
}

/// Hydration history query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrationHistoryQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

/// Hydration history response (list of daily summaries)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrationHistoryResponse {
    pub summaries: Vec<DailyHydrationSummaryResponse>,
}

/// Daily hydration summary (without entries, for history view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyHydrationSummaryResponse {
    pub date: NaiveDate,
    pub total_ml: i64,
    pub goal_ml: i32,
    pub progress_percent: f64,
    pub goal_met: bool,
    pub entry_count: i64,
}


// ============================================================================
// Sleep Types
// ============================================================================

/// Log sleep request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSleepRequest {
    /// When sleep started
    pub sleep_start: DateTime<Utc>,
    /// When sleep ended (woke up)
    pub sleep_end: DateTime<Utc>,
    /// Minutes spent awake during sleep
    #[serde(skip_serializing_if = "Option::is_none")]
    pub awake_minutes: Option<i32>,
    /// Minutes in light sleep
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_minutes: Option<i32>,
    /// Minutes in deep sleep
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deep_minutes: Option<i32>,
    /// Minutes in REM sleep
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rem_minutes: Option<i32>,
    /// Overall sleep score (0-100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_score: Option<i32>,
    /// Number of times woken up
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times_awoken: Option<i32>,
    /// Average heart rate during sleep
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_heart_rate: Option<i32>,
    /// Minimum heart rate during sleep
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_heart_rate: Option<i32>,
    /// Average HRV during sleep
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hrv_average: Option<f64>,
    /// Respiratory rate (breaths per minute)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub respiratory_rate: Option<f64>,
    /// Source of entry (manual, device name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Optional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Sleep log response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepLogResponse {
    pub id: String,
    pub sleep_start: DateTime<Utc>,
    pub sleep_end: DateTime<Utc>,
    pub total_duration_minutes: i32,
    pub awake_minutes: i32,
    pub light_minutes: i32,
    pub deep_minutes: i32,
    pub rem_minutes: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_efficiency: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_score: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times_awoken: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_heart_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_heart_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hrv_average: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub respiratory_rate: Option<f64>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Sleep history query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepHistoryQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[serde(default = "default_sleep_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_sleep_limit() -> i64 {
    30
}

impl SleepHistoryQuery {
    pub fn normalize(&self) -> Self {
        Self {
            start_date: self.start_date,
            end_date: self.end_date,
            limit: self.limit.clamp(1, 100),
            offset: self.offset.max(0),
        }
    }
}

/// Paginated sleep history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepHistoryResponse {
    pub items: Vec<SleepLogResponse>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

/// Sleep analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepAnalysisResponse {
    /// Average sleep duration in minutes
    pub avg_duration_minutes: f64,
    /// Average sleep efficiency percentage
    pub avg_efficiency: f64,
    /// Average percentage of time in deep sleep
    pub avg_deep_percent: f64,
    /// Average percentage of time in REM sleep
    pub avg_rem_percent: f64,
    /// Average percentage of time in light sleep
    pub avg_light_percent: f64,
    /// Average percentage of time awake
    pub avg_awake_percent: f64,
    /// Total number of nights tracked
    pub total_nights: i64,
    /// Sleep debt in minutes (positive = under-slept)
    pub sleep_debt_minutes: i64,
    /// Consistency score (0-100)
    pub consistency_score: f64,
}

/// Sleep goal response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepGoalResponse {
    /// Target sleep duration in minutes
    pub target_duration_minutes: i32,
    /// Target bedtime (HH:MM format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_bedtime: Option<String>,
    /// Target wake time (HH:MM format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_wake_time: Option<String>,
    /// Whether bedtime reminder is enabled
    pub bedtime_reminder_enabled: bool,
    /// Minutes before bedtime to send reminder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bedtime_reminder_minutes_before: Option<i32>,
}

/// Set sleep goal request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSleepGoalRequest {
    /// Target sleep duration in minutes (60-1440)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_duration_minutes: Option<i32>,
    /// Target bedtime (HH:MM format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_bedtime: Option<String>,
    /// Target wake time (HH:MM format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_wake_time: Option<String>,
    /// Enable bedtime reminder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bedtime_reminder_enabled: Option<bool>,
    /// Minutes before bedtime to send reminder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bedtime_reminder_minutes_before: Option<i32>,
}

/// Sleep analysis query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepAnalysisQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}


// ============================================================================
// Biometrics Types (Heart Rate & HRV)
// ============================================================================

/// Log heart rate request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogHeartRateRequest {
    /// Heart rate in BPM
    pub bpm: i32,
    /// Context: resting, active, workout, sleep, recovery
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// When measured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<DateTime<Utc>>,
    /// Associated workout ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workout_id: Option<String>,
    /// Source of measurement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Optional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Heart rate log response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRateLogResponse {
    pub id: String,
    pub bpm: i32,
    pub context: String,
    pub recorded_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workout_id: Option<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Log HRV request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogHrvRequest {
    /// RMSSD value in milliseconds
    pub rmssd: f64,
    /// SDNN value in milliseconds (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdnn: Option<f64>,
    /// Context: morning, sleep, recovery, workout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// When measured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<DateTime<Utc>>,
    /// Source of measurement
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Optional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// HRV log response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HrvLogResponse {
    pub id: String,
    pub rmssd: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdnn: Option<f64>,
    pub context: String,
    pub recorded_at: DateTime<Utc>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Recovery score response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryScoreResponse {
    /// Recovery score (0-100)
    pub score: f64,
    /// Current HRV reading
    pub hrv_current: f64,
    /// 7-day HRV baseline
    pub hrv_baseline: f64,
    /// Current resting heart rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resting_hr_current: Option<i32>,
    /// 7-day resting HR baseline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resting_hr_baseline: Option<f64>,
    /// Status: excellent, good, moderate, low, poor
    pub status: String,
}

/// Heart rate zone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRateZoneResponse {
    pub zone: i32,
    pub name: String,
    pub min_bpm: i32,
    pub max_bpm: i32,
}

/// Heart rate zones response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRateZonesResponse {
    pub max_heart_rate: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resting_heart_rate: Option<i32>,
    pub zones: Vec<HeartRateZoneResponse>,
    pub calculation_method: String,
}

/// Zone distribution response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneDistributionResponse {
    pub zone: i32,
    pub name: String,
    pub duration_seconds: i32,
    pub percentage: f64,
}

/// Resting HR analysis response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestingHrAnalysisResponse {
    pub current_avg: f64,
    pub baseline_avg: f64,
    pub deviation_percent: f64,
    pub is_anomaly: bool,
    pub trend: String,
}

/// Biometrics history query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricsHistoryQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(default = "default_biometrics_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_biometrics_limit() -> i64 {
    50
}

impl BiometricsHistoryQuery {
    pub fn normalize(&self) -> Self {
        Self {
            start_date: self.start_date,
            end_date: self.end_date,
            context: self.context.clone(),
            limit: self.limit.clamp(1, 100),
            offset: self.offset.max(0),
        }
    }
}

/// Heart rate history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartRateHistoryResponse {
    pub items: Vec<HeartRateLogResponse>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

/// HRV history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HrvHistoryResponse {
    pub items: Vec<HrvLogResponse>,
    pub total_count: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}


// ============================================================================
// Goals Types
// ============================================================================

/// Create goal request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGoalRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Goal type: weight, exercise, nutrition, hydration, sleep, custom
    pub goal_type: String,
    /// Metric being tracked (weight_kg, workout_count, calories, etc.)
    pub metric: String,
    /// Target value to achieve
    pub target_value: f64,
    /// Starting value (defaults to current)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_value: Option<f64>,
    /// Direction: increasing or decreasing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// Start date (defaults to today)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<NaiveDate>,
    /// Target completion date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_date: Option<NaiveDate>,
}

/// Update goal request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGoalRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_date: Option<NaiveDate>,
    /// Status: active, completed, abandoned, paused
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Goal response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub goal_type: String,
    pub metric: String,
    pub target_value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_value: Option<f64>,
    pub direction: String,
    pub start_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_date: Option<NaiveDate>,
    pub status: String,
}

/// Goal progress response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProgressResponse {
    pub goal_id: String,
    pub progress_percent: f64,
    pub remaining: f64,
    pub on_track: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days_remaining: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projected_completion: Option<NaiveDate>,
    pub milestones: Vec<MilestoneResponse>,
}

/// Milestone response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneResponse {
    pub id: String,
    pub name: String,
    pub target_value: f64,
    pub percentage: i32,
    pub achieved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_value: Option<f64>,
}

/// Goals list query parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoalsListQuery {
    /// Filter by status: active, completed, abandoned, paused
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Filter by goal type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_type: Option<String>,
}

/// Goals list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalsListResponse {
    pub goals: Vec<GoalResponse>,
}


// ============================================================================
// Biomarkers Types
// ============================================================================

/// Biomarker range response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomarkerRangeResponse {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub category: String,
    pub unit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimal_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimal_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Log biomarker request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogBiomarkerRequest {
    /// Biomarker name (e.g., "vitamin_d", "glucose_fasting")
    pub biomarker_name: String,
    /// Value in the biomarker's unit
    pub value: f64,
    /// Date of the test
    pub test_date: NaiveDate,
    /// Lab name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lab_name: Option<String>,
    /// Notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Biomarker log response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomarkerLogResponse {
    pub id: String,
    pub biomarker_name: String,
    pub display_name: String,
    pub category: String,
    pub value: f64,
    pub unit: String,
    /// Classification: critical_low, low, optimal, high, critical_high
    pub classification: String,
    pub test_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lab_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Biomarker history query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BiomarkerHistoryQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biomarker_name: Option<String>,
    #[serde(default = "default_biomarker_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_biomarker_limit() -> i64 {
    50
}

/// Create supplement request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSupplementRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    pub dosage: String,
    /// Frequency: daily, twice_daily, weekly, as_needed
    pub frequency: String,
    /// Time of day: morning, afternoon, evening, with_meals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_of_day: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Supplement response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplementResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    pub dosage: String,
    pub frequency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_of_day: Option<String>,
    pub start_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<NaiveDate>,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Log supplement intake request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSupplementRequest {
    pub supplement_id: String,
    #[serde(default)]
    pub skipped: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Supplement adherence query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplementAdherenceQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

/// Supplement adherence response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplementAdherenceResponse {
    pub supplement_id: String,
    pub supplement_name: String,
    pub total_days: i64,
    pub days_taken: i64,
    pub days_skipped: i64,
    pub adherence_percent: f64,
}

/// Supplements list query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SupplementsListQuery {
    #[serde(default = "default_active_only")]
    pub active_only: bool,
}

fn default_active_only() -> bool {
    true
}
