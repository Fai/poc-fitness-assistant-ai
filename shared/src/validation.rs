//! Input validation functions
//!
//! This module provides validation utilities for user input.
//! Uses both custom validators and the `validator` crate for derive macros.

/// Validate email format
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.is_empty() {
        return Err("Email cannot be empty".to_string());
    }
    if !email.contains('@') || !email.contains('.') {
        return Err("Invalid email format".to_string());
    }
    if email.len() > 255 {
        return Err("Email too long".to_string());
    }
    // Basic email regex check
    let email_regex = regex_lite::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    if !email_regex.is_match(email) {
        return Err("Invalid email format".to_string());
    }
    Ok(())
}

/// Validate password strength
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters".to_string());
    }
    if password.len() > 128 {
        return Err("Password too long".to_string());
    }
    Ok(())
}

/// Validate weight value (in kg)
pub fn validate_weight(weight_kg: f64) -> Result<(), String> {
    if weight_kg < 20.0 {
        return Err("Weight must be at least 20 kg".to_string());
    }
    if weight_kg > 500.0 {
        return Err("Weight must be at most 500 kg".to_string());
    }
    if weight_kg.is_nan() || weight_kg.is_infinite() {
        return Err("Weight must be a valid number".to_string());
    }
    Ok(())
}

/// Validate calorie value
pub fn validate_calories(calories: f64) -> Result<(), String> {
    if calories < 0.0 {
        return Err("Calories cannot be negative".to_string());
    }
    if calories > 50000.0 {
        return Err("Calorie value unreasonably high".to_string());
    }
    if calories.is_nan() || calories.is_infinite() {
        return Err("Calories must be a valid number".to_string());
    }
    Ok(())
}

/// Validate percentage value (0-100)
pub fn validate_percentage(value: f64) -> Result<(), String> {
    if value < 0.0 || value > 100.0 {
        return Err("Percentage must be between 0 and 100".to_string());
    }
    if value.is_nan() || value.is_infinite() {
        return Err("Percentage must be a valid number".to_string());
    }
    Ok(())
}

/// Validate heart rate (bpm)
pub fn validate_heart_rate(bpm: i32) -> Result<(), String> {
    if bpm < 20 {
        return Err("Heart rate too low".to_string());
    }
    if bpm > 300 {
        return Err("Heart rate too high".to_string());
    }
    Ok(())
}

/// Validate duration in minutes
pub fn validate_duration_minutes(minutes: i32) -> Result<(), String> {
    if minutes < 0 {
        return Err("Duration cannot be negative".to_string());
    }
    if minutes > 1440 {
        // 24 hours
        return Err("Duration cannot exceed 24 hours".to_string());
    }
    Ok(())
}

// ============================================================================
// Profile Validation
// ============================================================================

/// Validate height value (in cm)
/// Valid range: 50-300 cm (covers infants to tallest recorded humans)
pub fn validate_height_cm(height_cm: f64) -> Result<(), String> {
    if height_cm.is_nan() || height_cm.is_infinite() {
        return Err("Height must be a valid number".to_string());
    }
    if height_cm < 50.0 {
        return Err("Height must be at least 50 cm".to_string());
    }
    if height_cm > 300.0 {
        return Err("Height must be at most 300 cm".to_string());
    }
    Ok(())
}

/// Validate date of birth
/// Must not be in the future, and age must be between 1 and 150 years
pub fn validate_date_of_birth(dob: chrono::NaiveDate) -> Result<(), String> {
    let today = chrono::Utc::now().date_naive();
    
    if dob > today {
        return Err("Date of birth cannot be in the future".to_string());
    }
    
    // Calculate age
    let age_years = today.years_since(dob);
    
    match age_years {
        Some(age) if age < 1 => {
            Err("Age must be at least 1 year".to_string())
        }
        Some(age) if age > 150 => {
            Err("Age cannot exceed 150 years".to_string())
        }
        None => {
            // years_since returns None if dob is after today (already checked)
            Err("Invalid date of birth".to_string())
        }
        _ => Ok(()),
    }
}

/// Valid activity levels
pub const VALID_ACTIVITY_LEVELS: &[&str] = &[
    "sedentary",
    "lightly_active",
    "moderately_active",
    "very_active",
    "extra_active",
];

/// Validate activity level
pub fn validate_activity_level(level: &str) -> Result<(), String> {
    let normalized = level.to_lowercase();
    if VALID_ACTIVITY_LEVELS.contains(&normalized.as_str()) {
        Ok(())
    } else {
        Err(format!(
            "Invalid activity level. Must be one of: {}",
            VALID_ACTIVITY_LEVELS.join(", ")
        ))
    }
}

/// Valid biological sex values
pub const VALID_BIOLOGICAL_SEX: &[&str] = &["male", "female"];

/// Validate biological sex
pub fn validate_biological_sex(sex: &str) -> Result<(), String> {
    let normalized = sex.to_lowercase();
    if VALID_BIOLOGICAL_SEX.contains(&normalized.as_str()) {
        Ok(())
    } else {
        Err(format!(
            "Invalid biological sex. Must be one of: {}",
            VALID_BIOLOGICAL_SEX.join(", ")
        ))
    }
}

// ============================================================================
// User-Friendly Field Labels
// ============================================================================

/// Map technical field names to user-friendly display labels
pub fn get_field_display_label(field_name: &str) -> &str {
    match field_name {
        "weight" => "Current Weight",
        "height" | "height_cm" => "Height",
        "date_of_birth" => "Date of Birth",
        "biological_sex" => "Biological Sex",
        "activity_level" => "Activity Level",
        "weight_unit" => "Weight Unit",
        "height_unit" => "Height Unit",
        "distance_unit" => "Distance Unit",
        "energy_unit" => "Energy Unit",
        "temperature_unit" => "Temperature Unit",
        "timezone" => "Timezone",
        "daily_calorie_goal" => "Daily Calorie Goal",
        "daily_water_goal_ml" => "Daily Water Goal",
        "daily_step_goal" => "Daily Step Goal",
        _ => field_name,
    }
}

/// Convert a list of technical field names to user-friendly labels
pub fn get_missing_fields_labels(fields: &[String]) -> Vec<String> {
    fields
        .iter()
        .map(|f| get_field_display_label(f).to_string())
        .collect()
}

/// Validation error with field context
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub display_label: String,
}

impl ValidationError {
    pub fn new(field: &str, message: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
            display_label: get_field_display_label(field).to_string(),
        }
    }
    
    /// Format as user-friendly error message
    pub fn user_message(&self) -> String {
        format!("{}: {}", self.display_label, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use proptest::prelude::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name@domain.co.uk").is_ok());
        assert!(validate_email("").is_err());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("no@dot").is_err());
        assert!(validate_email("spaces in@email.com").is_err());
    }

    #[test]
    fn test_validate_password() {
        assert!(validate_password("password123").is_ok());
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password("short").is_err());
        assert!(validate_password(&"a".repeat(129)).is_err());
    }

    #[test]
    fn test_validate_weight() {
        assert!(validate_weight(70.0).is_ok());
        assert!(validate_weight(20.0).is_ok());
        assert!(validate_weight(500.0).is_ok());
        assert!(validate_weight(10.0).is_err());
        assert!(validate_weight(600.0).is_err());
        assert!(validate_weight(f64::NAN).is_err());
        assert!(validate_weight(f64::INFINITY).is_err());
    }

    #[test]
    fn test_validate_calories() {
        assert!(validate_calories(0.0).is_ok());
        assert!(validate_calories(2000.0).is_ok());
        assert!(validate_calories(-1.0).is_err());
        assert!(validate_calories(100000.0).is_err());
    }

    #[test]
    fn test_validate_percentage() {
        assert!(validate_percentage(0.0).is_ok());
        assert!(validate_percentage(50.0).is_ok());
        assert!(validate_percentage(100.0).is_ok());
        assert!(validate_percentage(-1.0).is_err());
        assert!(validate_percentage(101.0).is_err());
    }

    #[test]
    fn test_validate_heart_rate() {
        assert!(validate_heart_rate(60).is_ok());
        assert!(validate_heart_rate(180).is_ok());
        assert!(validate_heart_rate(10).is_err());
        assert!(validate_heart_rate(350).is_err());
    }

    // =========================================================================
    // Profile Validation Tests
    // =========================================================================

    #[test]
    fn test_validate_height_cm() {
        // Valid heights
        assert!(validate_height_cm(170.0).is_ok());
        assert!(validate_height_cm(50.0).is_ok());  // Minimum
        assert!(validate_height_cm(300.0).is_ok()); // Maximum
        
        // Invalid heights
        assert!(validate_height_cm(49.9).is_err());  // Below minimum
        assert!(validate_height_cm(300.1).is_err()); // Above maximum
        assert!(validate_height_cm(-10.0).is_err()); // Negative
        assert!(validate_height_cm(f64::NAN).is_err());
        assert!(validate_height_cm(f64::INFINITY).is_err());
    }

    #[test]
    fn test_validate_date_of_birth() {
        let today = chrono::Utc::now().date_naive();
        
        // Valid DOB - 30 years ago
        let valid_dob = today - chrono::Duration::days(30 * 365);
        assert!(validate_date_of_birth(valid_dob).is_ok());
        
        // Valid DOB - 1 year ago (minimum age)
        let one_year_ago = today - chrono::Duration::days(366);
        assert!(validate_date_of_birth(one_year_ago).is_ok());
        
        // Invalid - future date
        let future = today + chrono::Duration::days(1);
        assert!(validate_date_of_birth(future).is_err());
        
        // Invalid - too young (less than 1 year)
        let too_young = today - chrono::Duration::days(100);
        assert!(validate_date_of_birth(too_young).is_err());
        
        // Invalid - too old (over 150 years)
        let too_old = NaiveDate::from_ymd_opt(1800, 1, 1).unwrap();
        assert!(validate_date_of_birth(too_old).is_err());
    }

    #[test]
    fn test_validate_activity_level() {
        // Valid levels
        assert!(validate_activity_level("sedentary").is_ok());
        assert!(validate_activity_level("lightly_active").is_ok());
        assert!(validate_activity_level("moderately_active").is_ok());
        assert!(validate_activity_level("very_active").is_ok());
        assert!(validate_activity_level("extra_active").is_ok());
        
        // Case insensitive
        assert!(validate_activity_level("SEDENTARY").is_ok());
        assert!(validate_activity_level("Lightly_Active").is_ok());
        
        // Invalid
        assert!(validate_activity_level("invalid").is_err());
        assert!(validate_activity_level("").is_err());
        assert!(validate_activity_level("super_active").is_err());
    }

    #[test]
    fn test_validate_biological_sex() {
        // Valid
        assert!(validate_biological_sex("male").is_ok());
        assert!(validate_biological_sex("female").is_ok());
        
        // Case insensitive
        assert!(validate_biological_sex("MALE").is_ok());
        assert!(validate_biological_sex("Female").is_ok());
        
        // Invalid
        assert!(validate_biological_sex("other").is_err());
        assert!(validate_biological_sex("").is_err());
    }

    #[test]
    fn test_field_display_labels() {
        assert_eq!(get_field_display_label("date_of_birth"), "Date of Birth");
        assert_eq!(get_field_display_label("biological_sex"), "Biological Sex");
        assert_eq!(get_field_display_label("height_cm"), "Height");
        assert_eq!(get_field_display_label("activity_level"), "Activity Level");
        assert_eq!(get_field_display_label("unknown_field"), "unknown_field");
    }

    #[test]
    fn test_validation_error() {
        let err = ValidationError::new("height_cm", "must be at least 50 cm");
        assert_eq!(err.field, "height_cm");
        assert_eq!(err.display_label, "Height");
        assert_eq!(err.user_message(), "Height: must be at least 50 cm");
    }

    // Property-based tests
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_valid_weight_range(weight in 20.0f64..=500.0) {
            prop_assert!(validate_weight(weight).is_ok());
        }

        #[test]
        fn prop_invalid_weight_below_min(weight in 0.0f64..20.0) {
            prop_assert!(validate_weight(weight).is_err());
        }

        #[test]
        fn prop_invalid_weight_above_max(weight in 500.1f64..1000.0) {
            prop_assert!(validate_weight(weight).is_err());
        }

        #[test]
        fn prop_valid_percentage_range(pct in 0.0f64..=100.0) {
            prop_assert!(validate_percentage(pct).is_ok());
        }

        #[test]
        fn prop_valid_heart_rate_range(bpm in 20i32..=300) {
            prop_assert!(validate_heart_rate(bpm).is_ok());
        }

        #[test]
        fn prop_password_length_valid(len in 8usize..=128) {
            let password: String = (0..len).map(|_| 'a').collect();
            prop_assert!(validate_password(&password).is_ok());
        }

        // Property 27: Profile Height Validation
        // Feature: fitness-assistant-ai, Property 27: Profile Height Validation
        #[test]
        fn prop_valid_height_range(height in 50.0f64..=300.0) {
            prop_assert!(validate_height_cm(height).is_ok(),
                "Height {} should be valid", height);
        }

        #[test]
        fn prop_invalid_height_below_min(height in 0.0f64..50.0) {
            prop_assert!(validate_height_cm(height).is_err(),
                "Height {} should be invalid (below minimum)", height);
        }

        #[test]
        fn prop_invalid_height_above_max(height in 300.1f64..500.0) {
            prop_assert!(validate_height_cm(height).is_err(),
                "Height {} should be invalid (above maximum)", height);
        }

        // Property 28: Date of Birth Validation
        // Feature: fitness-assistant-ai, Property 28: Date of Birth Validation
        #[test]
        fn prop_valid_age_range(years_ago in 1u32..=150) {
            let today = chrono::Utc::now().date_naive();
            // Approximate DOB by subtracting years
            if let Some(dob) = today.checked_sub_signed(chrono::Duration::days(years_ago as i64 * 365 + 1)) {
                // This is approximate - actual validation uses years_since
                let result = validate_date_of_birth(dob);
                // We can't guarantee exact year calculation due to leap years,
                // but ages 2-149 should definitely be valid
                if years_ago >= 2 && years_ago <= 149 {
                    prop_assert!(result.is_ok(),
                        "DOB {} years ago should be valid, got: {:?}", years_ago, result);
                }
            }
        }
    }
}
