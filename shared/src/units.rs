//! Unit conversion and normalization module
//!
//! This module provides type-safe unit handling with automatic conversion.
//! All data is stored in SI units internally, converted on API boundaries.
//!
//! # Design Principles
//! 
//! 1. **Internal Consistency**: All storage uses SI units (kg, meters, etc.)
//! 2. **Type Safety**: Units are explicit in types, not just f64
//! 3. **Conversion at Boundaries**: Convert on input/output, not in business logic
//! 4. **Extensibility**: Easy to add new unit systems (e.g., stones for UK)

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Weight Units
// ============================================================================

/// Weight unit preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WeightUnit {
    #[default]
    Kg,
    Lbs,
    Stone, // For UK users
}

impl WeightUnit {
    /// Convert from this unit to kilograms
    pub fn to_kg(&self, value: f64) -> f64 {
        match self {
            WeightUnit::Kg => value,
            WeightUnit::Lbs => value * 0.453592,
            WeightUnit::Stone => value * 6.35029,
        }
    }

    /// Convert from kilograms to this unit
    pub fn from_kg(&self, kg: f64) -> f64 {
        match self {
            WeightUnit::Kg => kg,
            WeightUnit::Lbs => kg / 0.453592,
            WeightUnit::Stone => kg / 6.35029,
        }
    }

    /// Get the unit abbreviation
    pub fn abbreviation(&self) -> &'static str {
        match self {
            WeightUnit::Kg => "kg",
            WeightUnit::Lbs => "lbs",
            WeightUnit::Stone => "st",
        }
    }
}

impl fmt::Display for WeightUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

impl std::str::FromStr for WeightUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "kg" | "kilogram" | "kilograms" => Ok(WeightUnit::Kg),
            "lbs" | "lb" | "pound" | "pounds" => Ok(WeightUnit::Lbs),
            "st" | "stone" | "stones" => Ok(WeightUnit::Stone),
            _ => Err(format!("Unknown weight unit: {}", s)),
        }
    }
}

// ============================================================================
// Height/Distance Units
// ============================================================================

/// Height/distance unit preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HeightUnit {
    #[default]
    Cm,
    Meters,
    Inches,
    FeetInches, // Special case: stored as total inches
}

impl HeightUnit {
    /// Convert from this unit to centimeters
    pub fn to_cm(&self, value: f64) -> f64 {
        match self {
            HeightUnit::Cm => value,
            HeightUnit::Meters => value * 100.0,
            HeightUnit::Inches | HeightUnit::FeetInches => value * 2.54,
        }
    }

    /// Convert from centimeters to this unit
    pub fn from_cm(&self, cm: f64) -> f64 {
        match self {
            HeightUnit::Cm => cm,
            HeightUnit::Meters => cm / 100.0,
            HeightUnit::Inches | HeightUnit::FeetInches => cm / 2.54,
        }
    }

    /// Get the unit abbreviation
    pub fn abbreviation(&self) -> &'static str {
        match self {
            HeightUnit::Cm => "cm",
            HeightUnit::Meters => "m",
            HeightUnit::Inches => "in",
            HeightUnit::FeetInches => "ft/in",
        }
    }
}

impl fmt::Display for HeightUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

impl std::str::FromStr for HeightUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cm" | "centimeter" | "centimeters" => Ok(HeightUnit::Cm),
            "m" | "meter" | "meters" => Ok(HeightUnit::Meters),
            "in" | "inch" | "inches" => Ok(HeightUnit::Inches),
            "ft" | "feet" | "ft/in" | "feet/inches" => Ok(HeightUnit::FeetInches),
            _ => Err(format!("Unknown height unit: {}", s)),
        }
    }
}

// ============================================================================
// Distance Units (for exercise)
// ============================================================================

/// Distance unit preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DistanceUnit {
    #[default]
    Km,
    Miles,
    Meters,
}

impl DistanceUnit {
    /// Convert from this unit to meters
    pub fn to_meters(&self, value: f64) -> f64 {
        match self {
            DistanceUnit::Meters => value,
            DistanceUnit::Km => value * 1000.0,
            DistanceUnit::Miles => value * 1609.344,
        }
    }

    /// Convert from meters to this unit
    pub fn from_meters(&self, meters: f64) -> f64 {
        match self {
            DistanceUnit::Meters => meters,
            DistanceUnit::Km => meters / 1000.0,
            DistanceUnit::Miles => meters / 1609.344,
        }
    }

    /// Get the unit abbreviation
    pub fn abbreviation(&self) -> &'static str {
        match self {
            DistanceUnit::Km => "km",
            DistanceUnit::Miles => "mi",
            DistanceUnit::Meters => "m",
        }
    }
}

impl fmt::Display for DistanceUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

// ============================================================================
// Energy Units
// ============================================================================

/// Energy unit preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EnergyUnit {
    #[default]
    Kcal,
    Kj,
}

impl EnergyUnit {
    /// Convert from this unit to kcal
    pub fn to_kcal(&self, value: f64) -> f64 {
        match self {
            EnergyUnit::Kcal => value,
            EnergyUnit::Kj => value / 4.184,
        }
    }

    /// Convert from kcal to this unit
    pub fn from_kcal(&self, kcal: f64) -> f64 {
        match self {
            EnergyUnit::Kcal => kcal,
            EnergyUnit::Kj => kcal * 4.184,
        }
    }

    /// Get the unit abbreviation
    pub fn abbreviation(&self) -> &'static str {
        match self {
            EnergyUnit::Kcal => "kcal",
            EnergyUnit::Kj => "kJ",
        }
    }
}

impl fmt::Display for EnergyUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

// ============================================================================
// Temperature Units
// ============================================================================

/// Temperature unit preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TemperatureUnit {
    #[default]
    Celsius,
    Fahrenheit,
}

impl TemperatureUnit {
    /// Convert from this unit to Celsius
    pub fn to_celsius(&self, value: f64) -> f64 {
        match self {
            TemperatureUnit::Celsius => value,
            TemperatureUnit::Fahrenheit => (value - 32.0) * 5.0 / 9.0,
        }
    }

    /// Convert from Celsius to this unit
    pub fn from_celsius(&self, celsius: f64) -> f64 {
        match self {
            TemperatureUnit::Celsius => celsius,
            TemperatureUnit::Fahrenheit => celsius * 9.0 / 5.0 + 32.0,
        }
    }
}

// ============================================================================
// User Unit Preferences
// ============================================================================

/// Complete unit preferences for a user
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnitPreferences {
    pub weight: WeightUnit,
    pub height: HeightUnit,
    pub distance: DistanceUnit,
    pub energy: EnergyUnit,
    pub temperature: TemperatureUnit,
}

impl UnitPreferences {
    /// Create metric preferences (SI units)
    pub fn metric() -> Self {
        Self {
            weight: WeightUnit::Kg,
            height: HeightUnit::Cm,
            distance: DistanceUnit::Km,
            energy: EnergyUnit::Kcal,
            temperature: TemperatureUnit::Celsius,
        }
    }

    /// Create imperial preferences (US units)
    pub fn imperial() -> Self {
        Self {
            weight: WeightUnit::Lbs,
            height: HeightUnit::FeetInches,
            distance: DistanceUnit::Miles,
            energy: EnergyUnit::Kcal,
            temperature: TemperatureUnit::Fahrenheit,
        }
    }

    /// Create UK preferences
    pub fn uk() -> Self {
        Self {
            weight: WeightUnit::Stone,
            height: HeightUnit::FeetInches,
            distance: DistanceUnit::Miles,
            energy: EnergyUnit::Kcal,
            temperature: TemperatureUnit::Celsius,
        }
    }
}

// ============================================================================
// Height Display Helper
// ============================================================================

/// Represents height in feet and inches for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeetInchesHeight {
    pub feet: i32,
    pub inches: f64,
}

impl FeetInchesHeight {
    /// Create from total inches
    pub fn from_total_inches(total_inches: f64) -> Self {
        let feet = (total_inches / 12.0).floor() as i32;
        let inches = total_inches % 12.0;
        Self { feet, inches }
    }

    /// Convert to total inches
    pub fn to_total_inches(&self) -> f64 {
        (self.feet as f64 * 12.0) + self.inches
    }

    /// Create from centimeters
    pub fn from_cm(cm: f64) -> Self {
        let total_inches = cm / 2.54;
        Self::from_total_inches(total_inches)
    }

    /// Convert to centimeters
    pub fn to_cm(&self) -> f64 {
        self.to_total_inches() * 2.54
    }
}

impl fmt::Display for FeetInchesHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}'{:.0}\"", self.feet, self.inches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // =========================================================================
    // Weight Unit Tests
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Weight conversion round-trip preserves value
        #[test]
        fn prop_weight_roundtrip_kg(kg in 20.0f64..500.0) {
            let lbs = WeightUnit::Lbs.from_kg(kg);
            let back_to_kg = WeightUnit::Lbs.to_kg(lbs);
            prop_assert!((kg - back_to_kg).abs() < 0.0001,
                "Round-trip failed: {} -> {} -> {}", kg, lbs, back_to_kg);
        }

        #[test]
        fn prop_weight_roundtrip_lbs(lbs in 44.0f64..1100.0) {
            let kg = WeightUnit::Lbs.to_kg(lbs);
            let back_to_lbs = WeightUnit::Lbs.from_kg(kg);
            prop_assert!((lbs - back_to_lbs).abs() < 0.0001,
                "Round-trip failed: {} -> {} -> {}", lbs, kg, back_to_lbs);
        }

        #[test]
        fn prop_weight_roundtrip_stone(stone in 3.0f64..80.0) {
            let kg = WeightUnit::Stone.to_kg(stone);
            let back_to_stone = WeightUnit::Stone.from_kg(kg);
            prop_assert!((stone - back_to_stone).abs() < 0.0001,
                "Round-trip failed: {} -> {} -> {}", stone, kg, back_to_stone);
        }

        /// Property: Kg identity conversion
        #[test]
        fn prop_kg_identity(kg in 20.0f64..500.0) {
            prop_assert_eq!(WeightUnit::Kg.to_kg(kg), kg);
            prop_assert_eq!(WeightUnit::Kg.from_kg(kg), kg);
        }
    }

    #[test]
    fn test_known_weight_conversions() {
        // 1 kg = 2.20462 lbs
        let kg = 1.0;
        let lbs = WeightUnit::Lbs.from_kg(kg);
        assert!((lbs - 2.20462).abs() < 0.001);

        // 100 lbs = 45.3592 kg
        let lbs = 100.0;
        let kg = WeightUnit::Lbs.to_kg(lbs);
        assert!((kg - 45.3592).abs() < 0.001);

        // 1 stone = 6.35029 kg
        let stone = 1.0;
        let kg = WeightUnit::Stone.to_kg(stone);
        assert!((kg - 6.35029).abs() < 0.001);
    }

    // =========================================================================
    // Height Unit Tests
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Height conversion round-trip preserves value
        #[test]
        fn prop_height_roundtrip_cm(cm in 100.0f64..250.0) {
            let inches = HeightUnit::Inches.from_cm(cm);
            let back_to_cm = HeightUnit::Inches.to_cm(inches);
            prop_assert!((cm - back_to_cm).abs() < 0.0001,
                "Round-trip failed: {} -> {} -> {}", cm, inches, back_to_cm);
        }

        #[test]
        fn prop_height_roundtrip_meters(m in 1.0f64..2.5) {
            let cm = HeightUnit::Meters.to_cm(m);
            let back_to_m = HeightUnit::Meters.from_cm(cm);
            prop_assert!((m - back_to_m).abs() < 0.0001,
                "Round-trip failed: {} -> {} -> {}", m, cm, back_to_m);
        }
    }

    #[test]
    fn test_known_height_conversions() {
        // 180 cm = 70.866 inches
        let cm = 180.0;
        let inches = HeightUnit::Inches.from_cm(cm);
        assert!((inches - 70.866).abs() < 0.01);

        // 6 feet = 72 inches = 182.88 cm
        let ft_in = FeetInchesHeight { feet: 6, inches: 0.0 };
        let cm = ft_in.to_cm();
        assert!((cm - 182.88).abs() < 0.01);
    }

    // =========================================================================
    // Distance Unit Tests
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_distance_roundtrip_km(km in 0.1f64..100.0) {
            let meters = DistanceUnit::Km.to_meters(km);
            let back_to_km = DistanceUnit::Km.from_meters(meters);
            prop_assert!((km - back_to_km).abs() < 0.0001);
        }

        #[test]
        fn prop_distance_roundtrip_miles(miles in 0.1f64..100.0) {
            let meters = DistanceUnit::Miles.to_meters(miles);
            let back_to_miles = DistanceUnit::Miles.from_meters(meters);
            prop_assert!((miles - back_to_miles).abs() < 0.0001);
        }
    }

    // =========================================================================
    // Energy Unit Tests
    // =========================================================================

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_energy_roundtrip_kj(kj in 100.0f64..10000.0) {
            let kcal = EnergyUnit::Kj.to_kcal(kj);
            let back_to_kj = EnergyUnit::Kj.from_kcal(kcal);
            prop_assert!((kj - back_to_kj).abs() < 0.0001);
        }
    }

    // =========================================================================
    // Temperature Unit Tests
    // =========================================================================

    #[test]
    fn test_temperature_conversions() {
        // 0°C = 32°F
        assert!((TemperatureUnit::Fahrenheit.from_celsius(0.0) - 32.0).abs() < 0.001);
        
        // 100°C = 212°F
        assert!((TemperatureUnit::Fahrenheit.from_celsius(100.0) - 212.0).abs() < 0.001);
        
        // 98.6°F = 37°C (body temp)
        assert!((TemperatureUnit::Fahrenheit.to_celsius(98.6) - 37.0).abs() < 0.1);
    }

    // =========================================================================
    // FeetInches Tests
    // =========================================================================

    #[test]
    fn test_feet_inches_conversion() {
        let height = FeetInchesHeight { feet: 5, inches: 10.0 };
        let cm = height.to_cm();
        // 5'10" = 70 inches = 177.8 cm
        assert!((cm - 177.8).abs() < 0.1);

        let back = FeetInchesHeight::from_cm(cm);
        assert_eq!(back.feet, 5);
        assert!((back.inches - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_feet_inches_display() {
        let height = FeetInchesHeight { feet: 6, inches: 2.0 };
        assert_eq!(format!("{}", height), "6'2\"");
    }

    // =========================================================================
    // Unit Preferences Tests
    // =========================================================================

    #[test]
    fn test_metric_preferences() {
        let prefs = UnitPreferences::metric();
        assert_eq!(prefs.weight, WeightUnit::Kg);
        assert_eq!(prefs.height, HeightUnit::Cm);
        assert_eq!(prefs.distance, DistanceUnit::Km);
    }

    #[test]
    fn test_imperial_preferences() {
        let prefs = UnitPreferences::imperial();
        assert_eq!(prefs.weight, WeightUnit::Lbs);
        assert_eq!(prefs.height, HeightUnit::FeetInches);
        assert_eq!(prefs.distance, DistanceUnit::Miles);
    }

    // =========================================================================
    // String Parsing Tests
    // =========================================================================

    #[test]
    fn test_weight_unit_parsing() {
        assert_eq!("kg".parse::<WeightUnit>().unwrap(), WeightUnit::Kg);
        assert_eq!("lbs".parse::<WeightUnit>().unwrap(), WeightUnit::Lbs);
        assert_eq!("pounds".parse::<WeightUnit>().unwrap(), WeightUnit::Lbs);
        assert_eq!("stone".parse::<WeightUnit>().unwrap(), WeightUnit::Stone);
        assert!("invalid".parse::<WeightUnit>().is_err());
    }
}
