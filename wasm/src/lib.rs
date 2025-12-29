//! Fitness Assistant WASM Module
//!
//! This crate provides WebAssembly bindings for performance-critical
//! calculations that can run in the browser.

use wasm_bindgen::prelude::*;

/// Calculate moving average for a series of values
#[wasm_bindgen]
pub fn calculate_moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
    if values.is_empty() || window_size == 0 {
        return vec![];
    }

    let mut result = Vec::with_capacity(values.len());
    
    for i in 0..values.len() {
        let start = if i >= window_size { i - window_size + 1 } else { 0 };
        let window = &values[start..=i];
        let avg = window.iter().sum::<f64>() / window.len() as f64;
        result.push(avg);
    }
    
    result
}

/// Calculate BMI from weight (kg) and height (cm)
#[wasm_bindgen]
pub fn calculate_bmi(weight_kg: f64, height_cm: f64) -> f64 {
    if height_cm <= 0.0 {
        return 0.0;
    }
    let height_m = height_cm / 100.0;
    weight_kg / (height_m * height_m)
}

/// Calculate TDEE (Total Daily Energy Expenditure)
/// Uses Mifflin-St Jeor equation
#[wasm_bindgen]
pub fn calculate_tdee(
    weight_kg: f64,
    height_cm: f64,
    age_years: u32,
    is_male: bool,
    activity_multiplier: f64,
) -> f64 {
    let bmr = if is_male {
        10.0 * weight_kg + 6.25 * height_cm - 5.0 * age_years as f64 + 5.0
    } else {
        10.0 * weight_kg + 6.25 * height_cm - 5.0 * age_years as f64 - 161.0
    };
    
    bmr * activity_multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moving_average() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = calculate_moving_average(&values, 3);
        assert_eq!(result.len(), 5);
        assert!((result[2] - 2.0).abs() < 0.001); // avg of [1,2,3]
        assert!((result[4] - 4.0).abs() < 0.001); // avg of [3,4,5]
    }

    #[test]
    fn test_bmi() {
        let bmi = calculate_bmi(70.0, 175.0);
        assert!((bmi - 22.86).abs() < 0.1);
    }
}
