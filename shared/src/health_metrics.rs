//! Health metrics calculations module
//!
//! Provides calculations for BMI, TDEE, healthy weight ranges, and other
//! health-related metrics based on user profile data.
//!
//! # Design Principles
//!
//! 1. **Pure Functions**: All calculations are pure, no side effects
//! 2. **Evidence-Based**: Formulas from peer-reviewed research
//! 3. **Configurable**: Support multiple calculation methods
//! 4. **Type Safety**: Strong typing prevents unit confusion

use serde::{Deserialize, Serialize};

// ============================================================================
// User Profile Types
// ============================================================================

/// Biological sex for health calculations
/// Note: This is used for physiological calculations only
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BiologicalSex {
    Male,
    Female,
}

/// Activity level for TDEE calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActivityLevel {
    /// Little or no exercise
    Sedentary,
    /// Light exercise 1-3 days/week
    #[default]
    LightlyActive,
    /// Moderate exercise 3-5 days/week
    ModeratelyActive,
    /// Hard exercise 6-7 days/week
    VeryActive,
    /// Very hard exercise, physical job
    ExtraActive,
}

impl ActivityLevel {
    /// Get the activity multiplier for TDEE calculation
    pub fn multiplier(&self) -> f64 {
        match self {
            ActivityLevel::Sedentary => 1.2,
            ActivityLevel::LightlyActive => 1.375,
            ActivityLevel::ModeratelyActive => 1.55,
            ActivityLevel::VeryActive => 1.725,
            ActivityLevel::ExtraActive => 1.9,
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ActivityLevel::Sedentary => "Little or no exercise",
            ActivityLevel::LightlyActive => "Light exercise 1-3 days/week",
            ActivityLevel::ModeratelyActive => "Moderate exercise 3-5 days/week",
            ActivityLevel::VeryActive => "Hard exercise 6-7 days/week",
            ActivityLevel::ExtraActive => "Very hard exercise or physical job",
        }
    }
}

/// User profile data needed for health calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthProfile {
    /// Height in centimeters (stored in SI)
    pub height_cm: f64,
    /// Current weight in kilograms (stored in SI)
    pub weight_kg: f64,
    /// Age in years
    pub age_years: i32,
    /// Biological sex for physiological calculations
    pub sex: BiologicalSex,
    /// Activity level for TDEE
    pub activity_level: ActivityLevel,
}

// ============================================================================
// BMI Calculations
// ============================================================================

/// BMI category classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BmiCategory {
    SeverelyUnderweight,
    Underweight,
    Normal,
    Overweight,
    ObeseClass1,
    ObeseClass2,
    ObeseClass3,
}

impl BmiCategory {
    /// Get the BMI range for this category
    pub fn range(&self) -> (f64, f64) {
        match self {
            BmiCategory::SeverelyUnderweight => (0.0, 16.0),
            BmiCategory::Underweight => (16.0, 18.5),
            BmiCategory::Normal => (18.5, 25.0),
            BmiCategory::Overweight => (25.0, 30.0),
            BmiCategory::ObeseClass1 => (30.0, 35.0),
            BmiCategory::ObeseClass2 => (35.0, 40.0),
            BmiCategory::ObeseClass3 => (40.0, f64::INFINITY),
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            BmiCategory::SeverelyUnderweight => "Severely Underweight",
            BmiCategory::Underweight => "Underweight",
            BmiCategory::Normal => "Normal/Healthy",
            BmiCategory::Overweight => "Overweight",
            BmiCategory::ObeseClass1 => "Obese (Class I)",
            BmiCategory::ObeseClass2 => "Obese (Class II)",
            BmiCategory::ObeseClass3 => "Obese (Class III)",
        }
    }
}

/// BMI calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BmiResult {
    /// BMI value
    pub value: f64,
    /// BMI category
    pub category: BmiCategory,
    /// Healthy weight range in kg for this height
    pub healthy_weight_range_kg: (f64, f64),
    /// Distance from healthy range (negative = under, positive = over, 0 = in range)
    pub distance_from_healthy_kg: f64,
}

/// Calculate BMI from weight and height
///
/// Formula: BMI = weight(kg) / height(m)²
pub fn calculate_bmi(weight_kg: f64, height_cm: f64) -> f64 {
    let height_m = height_cm / 100.0;
    weight_kg / (height_m * height_m)
}

/// Classify BMI into category
pub fn classify_bmi(bmi: f64) -> BmiCategory {
    if bmi < 16.0 {
        BmiCategory::SeverelyUnderweight
    } else if bmi < 18.5 {
        BmiCategory::Underweight
    } else if bmi < 25.0 {
        BmiCategory::Normal
    } else if bmi < 30.0 {
        BmiCategory::Overweight
    } else if bmi < 35.0 {
        BmiCategory::ObeseClass1
    } else if bmi < 40.0 {
        BmiCategory::ObeseClass2
    } else {
        BmiCategory::ObeseClass3
    }
}

/// Calculate healthy weight range for a given height
///
/// Based on BMI 18.5-25 (normal range)
pub fn healthy_weight_range_kg(height_cm: f64) -> (f64, f64) {
    let height_m = height_cm / 100.0;
    let height_m_sq = height_m * height_m;
    let min_weight = 18.5 * height_m_sq;
    let max_weight = 25.0 * height_m_sq;
    (min_weight, max_weight)
}

/// Calculate complete BMI result
pub fn calculate_bmi_result(weight_kg: f64, height_cm: f64) -> BmiResult {
    let bmi = calculate_bmi(weight_kg, height_cm);
    let category = classify_bmi(bmi);
    let healthy_range = healthy_weight_range_kg(height_cm);
    
    let distance = if weight_kg < healthy_range.0 {
        weight_kg - healthy_range.0 // Negative = underweight
    } else if weight_kg > healthy_range.1 {
        weight_kg - healthy_range.1 // Positive = overweight
    } else {
        0.0 // In healthy range
    };

    BmiResult {
        value: bmi,
        category,
        healthy_weight_range_kg: healthy_range,
        distance_from_healthy_kg: distance,
    }
}

// ============================================================================
// BMR and TDEE Calculations
// ============================================================================

/// BMR calculation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BmrMethod {
    /// Mifflin-St Jeor (most accurate for most people)
    #[default]
    MifflinStJeor,
    /// Harris-Benedict (original, less accurate)
    HarrisBenedict,
    /// Katch-McArdle (requires body fat %, most accurate if available)
    KatchMcArdle,
}

/// Calculate Basal Metabolic Rate using Mifflin-St Jeor equation
///
/// Men: BMR = 10 × weight(kg) + 6.25 × height(cm) - 5 × age(y) + 5
/// Women: BMR = 10 × weight(kg) + 6.25 × height(cm) - 5 × age(y) - 161
pub fn calculate_bmr_mifflin(weight_kg: f64, height_cm: f64, age_years: i32, sex: BiologicalSex) -> f64 {
    let base = 10.0 * weight_kg + 6.25 * height_cm - 5.0 * age_years as f64;
    match sex {
        BiologicalSex::Male => base + 5.0,
        BiologicalSex::Female => base - 161.0,
    }
}

/// Calculate BMR using Harris-Benedict equation (revised)
///
/// Men: BMR = 88.362 + 13.397 × weight(kg) + 4.799 × height(cm) - 5.677 × age(y)
/// Women: BMR = 447.593 + 9.247 × weight(kg) + 3.098 × height(cm) - 4.330 × age(y)
pub fn calculate_bmr_harris_benedict(weight_kg: f64, height_cm: f64, age_years: i32, sex: BiologicalSex) -> f64 {
    match sex {
        BiologicalSex::Male => {
            88.362 + 13.397 * weight_kg + 4.799 * height_cm - 5.677 * age_years as f64
        }
        BiologicalSex::Female => {
            447.593 + 9.247 * weight_kg + 3.098 * height_cm - 4.330 * age_years as f64
        }
    }
}

/// Calculate BMR using Katch-McArdle equation (requires lean body mass)
///
/// BMR = 370 + 21.6 × LBM(kg)
/// LBM = weight × (1 - body_fat_percent/100)
pub fn calculate_bmr_katch_mcardle(weight_kg: f64, body_fat_percent: f64) -> f64 {
    let lean_body_mass = weight_kg * (1.0 - body_fat_percent / 100.0);
    370.0 + 21.6 * lean_body_mass
}

/// Calculate BMR with specified method
pub fn calculate_bmr(profile: &HealthProfile, method: BmrMethod) -> f64 {
    match method {
        BmrMethod::MifflinStJeor => {
            calculate_bmr_mifflin(profile.weight_kg, profile.height_cm, profile.age_years, profile.sex)
        }
        BmrMethod::HarrisBenedict => {
            calculate_bmr_harris_benedict(profile.weight_kg, profile.height_cm, profile.age_years, profile.sex)
        }
        BmrMethod::KatchMcArdle => {
            // Default to 20% body fat if not provided
            calculate_bmr_katch_mcardle(profile.weight_kg, 20.0)
        }
    }
}

/// Calculate Total Daily Energy Expenditure
///
/// TDEE = BMR × Activity Multiplier
pub fn calculate_tdee(profile: &HealthProfile) -> f64 {
    let bmr = calculate_bmr(profile, BmrMethod::MifflinStJeor);
    bmr * profile.activity_level.multiplier()
}

/// TDEE calculation result with breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdeeResult {
    /// Basal Metabolic Rate
    pub bmr: f64,
    /// Total Daily Energy Expenditure
    pub tdee: f64,
    /// Activity multiplier used
    pub activity_multiplier: f64,
    /// Calories for weight loss (500 deficit)
    pub calories_for_loss: f64,
    /// Calories for weight gain (500 surplus)
    pub calories_for_gain: f64,
    /// Calories for maintenance
    pub calories_for_maintenance: f64,
}

/// Calculate complete TDEE result
pub fn calculate_tdee_result(profile: &HealthProfile) -> TdeeResult {
    let bmr = calculate_bmr(profile, BmrMethod::MifflinStJeor);
    let tdee = bmr * profile.activity_level.multiplier();
    
    TdeeResult {
        bmr,
        tdee,
        activity_multiplier: profile.activity_level.multiplier(),
        calories_for_loss: (tdee - 500.0).max(1200.0), // Never below 1200
        calories_for_gain: tdee + 500.0,
        calories_for_maintenance: tdee,
    }
}

// ============================================================================
// Hydration Calculations
// ============================================================================

/// Calculate recommended daily water intake
///
/// Base formula: 30-35ml per kg of body weight
/// Adjusted for activity level
pub fn calculate_daily_water_ml(weight_kg: f64, activity_level: ActivityLevel) -> i32 {
    let base_ml = weight_kg * 30.0;
    let adjusted = match activity_level {
        ActivityLevel::Sedentary => base_ml,
        ActivityLevel::LightlyActive => base_ml * 1.1,
        ActivityLevel::ModeratelyActive => base_ml * 1.2,
        ActivityLevel::VeryActive => base_ml * 1.3,
        ActivityLevel::ExtraActive => base_ml * 1.4,
    };
    adjusted.round() as i32
}

// ============================================================================
// Body Fat Estimation
// ============================================================================

/// Estimate body fat percentage from BMI (rough estimate)
///
/// This is a rough estimate - actual measurement is more accurate
/// Formula: BF% = 1.20 × BMI + 0.23 × Age - 10.8 × sex - 5.4
/// where sex = 1 for male, 0 for female
pub fn estimate_body_fat_from_bmi(bmi: f64, age_years: i32, sex: BiologicalSex) -> f64 {
    let sex_factor = match sex {
        BiologicalSex::Male => 1.0,
        BiologicalSex::Female => 0.0,
    };
    let bf = 1.20 * bmi + 0.23 * age_years as f64 - 10.8 * sex_factor - 5.4;
    bf.max(3.0).min(60.0) // Clamp to reasonable range
}

/// Body fat category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyFatCategory {
    Essential,
    Athletic,
    Fitness,
    Average,
    Obese,
}

impl BodyFatCategory {
    /// Get the body fat range for this category by sex
    pub fn range(&self, sex: BiologicalSex) -> (f64, f64) {
        match (self, sex) {
            (BodyFatCategory::Essential, BiologicalSex::Male) => (2.0, 5.0),
            (BodyFatCategory::Essential, BiologicalSex::Female) => (10.0, 13.0),
            (BodyFatCategory::Athletic, BiologicalSex::Male) => (6.0, 13.0),
            (BodyFatCategory::Athletic, BiologicalSex::Female) => (14.0, 20.0),
            (BodyFatCategory::Fitness, BiologicalSex::Male) => (14.0, 17.0),
            (BodyFatCategory::Fitness, BiologicalSex::Female) => (21.0, 24.0),
            (BodyFatCategory::Average, BiologicalSex::Male) => (18.0, 24.0),
            (BodyFatCategory::Average, BiologicalSex::Female) => (25.0, 31.0),
            (BodyFatCategory::Obese, BiologicalSex::Male) => (25.0, 100.0),
            (BodyFatCategory::Obese, BiologicalSex::Female) => (32.0, 100.0),
        }
    }
}

/// Classify body fat percentage
pub fn classify_body_fat(body_fat_percent: f64, sex: BiologicalSex) -> BodyFatCategory {
    match sex {
        BiologicalSex::Male => {
            if body_fat_percent < 6.0 {
                BodyFatCategory::Essential
            } else if body_fat_percent < 14.0 {
                BodyFatCategory::Athletic
            } else if body_fat_percent < 18.0 {
                BodyFatCategory::Fitness
            } else if body_fat_percent < 25.0 {
                BodyFatCategory::Average
            } else {
                BodyFatCategory::Obese
            }
        }
        BiologicalSex::Female => {
            if body_fat_percent < 14.0 {
                BodyFatCategory::Essential
            } else if body_fat_percent < 21.0 {
                BodyFatCategory::Athletic
            } else if body_fat_percent < 25.0 {
                BodyFatCategory::Fitness
            } else if body_fat_percent < 32.0 {
                BodyFatCategory::Average
            } else {
                BodyFatCategory::Obese
            }
        }
    }
}

// ============================================================================
// Ideal Weight Calculations
// ============================================================================

/// Calculate ideal body weight using various formulas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdealWeightResult {
    /// Devine formula result
    pub devine: f64,
    /// Robinson formula result
    pub robinson: f64,
    /// Miller formula result
    pub miller: f64,
    /// Hamwi formula result
    pub hamwi: f64,
    /// Average of all formulas
    pub average: f64,
}

/// Calculate ideal body weight using multiple formulas
pub fn calculate_ideal_weight(height_cm: f64, sex: BiologicalSex) -> IdealWeightResult {
    let height_inches = height_cm / 2.54;
    let inches_over_5ft = (height_inches - 60.0).max(0.0);

    let (devine, robinson, miller, hamwi) = match sex {
        BiologicalSex::Male => {
            let devine = 50.0 + 2.3 * inches_over_5ft;
            let robinson = 52.0 + 1.9 * inches_over_5ft;
            let miller = 56.2 + 1.41 * inches_over_5ft;
            let hamwi = 48.0 + 2.7 * inches_over_5ft;
            (devine, robinson, miller, hamwi)
        }
        BiologicalSex::Female => {
            let devine = 45.5 + 2.3 * inches_over_5ft;
            let robinson = 49.0 + 1.7 * inches_over_5ft;
            let miller = 53.1 + 1.36 * inches_over_5ft;
            let hamwi = 45.5 + 2.2 * inches_over_5ft;
            (devine, robinson, miller, hamwi)
        }
    };

    let average = (devine + robinson + miller + hamwi) / 4.0;

    IdealWeightResult {
        devine,
        robinson,
        miller,
        hamwi,
        average,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // =========================================================================
    // BMI Tests
    // =========================================================================

    #[test]
    fn test_bmi_calculation() {
        // 70kg, 175cm -> BMI ~22.86
        let bmi = calculate_bmi(70.0, 175.0);
        assert!((bmi - 22.86).abs() < 0.1);
    }

    #[test]
    fn test_bmi_categories() {
        assert_eq!(classify_bmi(15.0), BmiCategory::SeverelyUnderweight);
        assert_eq!(classify_bmi(17.0), BmiCategory::Underweight);
        assert_eq!(classify_bmi(22.0), BmiCategory::Normal);
        assert_eq!(classify_bmi(27.0), BmiCategory::Overweight);
        assert_eq!(classify_bmi(32.0), BmiCategory::ObeseClass1);
        assert_eq!(classify_bmi(37.0), BmiCategory::ObeseClass2);
        assert_eq!(classify_bmi(42.0), BmiCategory::ObeseClass3);
    }

    #[test]
    fn test_healthy_weight_range() {
        // For 175cm, healthy range should be ~56.7-76.6 kg
        let (min, max) = healthy_weight_range_kg(175.0);
        assert!((min - 56.7).abs() < 0.5);
        assert!((max - 76.6).abs() < 0.5);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: BMI is always positive for valid inputs
        #[test]
        fn prop_bmi_positive(weight in 20.0f64..500.0, height in 100.0f64..250.0) {
            let bmi = calculate_bmi(weight, height);
            prop_assert!(bmi > 0.0);
        }

        /// Property: Heavier weight = higher BMI (same height)
        #[test]
        fn prop_bmi_increases_with_weight(
            weight1 in 50.0f64..100.0,
            weight2 in 100.0f64..150.0,
            height in 150.0f64..200.0
        ) {
            let bmi1 = calculate_bmi(weight1, height);
            let bmi2 = calculate_bmi(weight2, height);
            prop_assert!(bmi2 > bmi1);
        }

        /// Property: Taller height = lower BMI (same weight)
        #[test]
        fn prop_bmi_decreases_with_height(
            weight in 60.0f64..100.0,
            height1 in 150.0f64..170.0,
            height2 in 180.0f64..200.0
        ) {
            let bmi1 = calculate_bmi(weight, height1);
            let bmi2 = calculate_bmi(weight, height2);
            prop_assert!(bmi1 > bmi2);
        }

        /// Property: Healthy weight range contains weights that produce normal BMI
        #[test]
        fn prop_healthy_range_produces_normal_bmi(height in 150.0f64..200.0) {
            let (min, max) = healthy_weight_range_kg(height);
            let mid_weight = (min + max) / 2.0;
            let bmi = calculate_bmi(mid_weight, height);
            prop_assert!(bmi >= 18.5 && bmi <= 25.0,
                "Mid-range weight {} at height {} produced BMI {} (expected 18.5-25)",
                mid_weight, height, bmi);
        }
    }

    // =========================================================================
    // BMR/TDEE Tests
    // =========================================================================

    #[test]
    fn test_bmr_mifflin() {
        // 30yo male, 80kg, 180cm -> BMR ~1780
        let bmr = calculate_bmr_mifflin(80.0, 180.0, 30, BiologicalSex::Male);
        assert!((bmr - 1780.0).abs() < 50.0);

        // 30yo female, 60kg, 165cm -> BMR ~1370
        let bmr = calculate_bmr_mifflin(60.0, 165.0, 30, BiologicalSex::Female);
        assert!((bmr - 1370.0).abs() < 50.0);
    }

    #[test]
    fn test_tdee_calculation() {
        let profile = HealthProfile {
            height_cm: 180.0,
            weight_kg: 80.0,
            age_years: 30,
            sex: BiologicalSex::Male,
            activity_level: ActivityLevel::ModeratelyActive,
        };
        
        let result = calculate_tdee_result(&profile);
        
        // BMR ~1780, TDEE = BMR * 1.55 ~2760
        assert!(result.bmr > 1700.0 && result.bmr < 1900.0);
        assert!(result.tdee > 2600.0 && result.tdee < 3000.0);
        assert_eq!(result.calories_for_loss, result.tdee - 500.0);
        assert_eq!(result.calories_for_gain, result.tdee + 500.0);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: BMR is always positive
        #[test]
        fn prop_bmr_positive(
            weight in 40.0f64..150.0,
            height in 140.0f64..210.0,
            age in 18i32..80
        ) {
            let bmr_male = calculate_bmr_mifflin(weight, height, age, BiologicalSex::Male);
            let bmr_female = calculate_bmr_mifflin(weight, height, age, BiologicalSex::Female);
            prop_assert!(bmr_male > 0.0);
            prop_assert!(bmr_female > 0.0);
        }

        /// Property: Male BMR > Female BMR (same stats)
        #[test]
        fn prop_male_bmr_higher(
            weight in 50.0f64..100.0,
            height in 160.0f64..190.0,
            age in 20i32..60
        ) {
            let bmr_male = calculate_bmr_mifflin(weight, height, age, BiologicalSex::Male);
            let bmr_female = calculate_bmr_mifflin(weight, height, age, BiologicalSex::Female);
            prop_assert!(bmr_male > bmr_female);
        }

        /// Property: TDEE > BMR (activity multiplier > 1)
        #[test]
        fn prop_tdee_greater_than_bmr(
            weight in 50.0f64..100.0,
            height in 160.0f64..190.0,
            age in 20i32..60
        ) {
            let profile = HealthProfile {
                height_cm: height,
                weight_kg: weight,
                age_years: age,
                sex: BiologicalSex::Male,
                activity_level: ActivityLevel::ModeratelyActive,
            };
            let result = calculate_tdee_result(&profile);
            prop_assert!(result.tdee > result.bmr);
        }
    }

    // =========================================================================
    // Hydration Tests
    // =========================================================================

    #[test]
    fn test_hydration_calculation() {
        // 70kg sedentary -> ~2100ml
        let water = calculate_daily_water_ml(70.0, ActivityLevel::Sedentary);
        assert!((water - 2100).abs() < 100);

        // 70kg very active -> ~2730ml
        let water = calculate_daily_water_ml(70.0, ActivityLevel::VeryActive);
        assert!((water - 2730).abs() < 100);
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: More active = more water needed
        #[test]
        fn prop_activity_increases_water(weight in 50.0f64..100.0) {
            let sedentary = calculate_daily_water_ml(weight, ActivityLevel::Sedentary);
            let active = calculate_daily_water_ml(weight, ActivityLevel::VeryActive);
            prop_assert!(active > sedentary);
        }

        /// Property: Heavier = more water needed
        #[test]
        fn prop_weight_increases_water(
            weight1 in 50.0f64..70.0,
            weight2 in 80.0f64..100.0
        ) {
            let water1 = calculate_daily_water_ml(weight1, ActivityLevel::ModeratelyActive);
            let water2 = calculate_daily_water_ml(weight2, ActivityLevel::ModeratelyActive);
            prop_assert!(water2 > water1);
        }
    }

    // =========================================================================
    // Body Fat Tests
    // =========================================================================

    #[test]
    fn test_body_fat_classification() {
        assert_eq!(classify_body_fat(10.0, BiologicalSex::Male), BodyFatCategory::Athletic);
        assert_eq!(classify_body_fat(20.0, BiologicalSex::Male), BodyFatCategory::Average);
        assert_eq!(classify_body_fat(20.0, BiologicalSex::Female), BodyFatCategory::Athletic);
        assert_eq!(classify_body_fat(28.0, BiologicalSex::Female), BodyFatCategory::Average);
    }

    // =========================================================================
    // Ideal Weight Tests
    // =========================================================================

    #[test]
    fn test_ideal_weight() {
        // 180cm male
        let result = calculate_ideal_weight(180.0, BiologicalSex::Male);
        // Should be around 70-80kg
        assert!(result.average > 65.0 && result.average < 85.0);

        // 165cm female
        let result = calculate_ideal_weight(165.0, BiologicalSex::Female);
        // Should be around 55-65kg
        assert!(result.average > 50.0 && result.average < 70.0);
    }
}
