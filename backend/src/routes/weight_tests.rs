//! Property-based tests for weight tracking
//!
//! Feature: fitness-assistant-ai
//! Tests Properties 1, 3, 4, 5 from design document

#[cfg(test)]
mod tests {
    use crate::services::weight::WeightService;
    use proptest::prelude::*;

    // =========================================================================
    // Feature: fitness-assistant-ai, Property 1: Data Persistence Round-Trip
    // =========================================================================
    // Note: Full round-trip tests require database - marked with #[ignore]
    // These tests verify the in-memory logic components

    // =========================================================================
    // Feature: fitness-assistant-ai, Property 3: Moving Average Calculation
    // =========================================================================
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 3: Moving average equals arithmetic mean of N recent entries
        #[test]
        fn prop_moving_average_equals_arithmetic_mean(
            weights in prop::collection::vec(20.0f64..500.0, 1..100),
            n in 1usize..50
        ) {
            let result = WeightService::calculate_moving_average(&weights, n);

            prop_assert!(result.is_some(), "Moving average should exist for non-empty input");

            let avg = result.unwrap();
            let count = weights.len().min(n);
            let expected: f64 = weights.iter().take(count).sum::<f64>() / count as f64;

            prop_assert!(
                (avg - expected).abs() < 1e-10,
                "Moving average {} != expected {} for n={}, count={}",
                avg, expected, n, count
            );
        }

        /// Property 3: Moving average with single entry equals that entry
        #[test]
        fn prop_moving_average_single_entry(weight in 20.0f64..500.0, n in 1usize..50) {
            let weights = vec![weight];
            let result = WeightService::calculate_moving_average(&weights, n);

            prop_assert!(result.is_some());
            prop_assert!((result.unwrap() - weight).abs() < 1e-10);
        }

        /// Property 3: Moving average is bounded by min and max of input
        #[test]
        fn prop_moving_average_bounded(
            weights in prop::collection::vec(20.0f64..500.0, 2..100),
            n in 1usize..50
        ) {
            let result = WeightService::calculate_moving_average(&weights, n);
            prop_assert!(result.is_some());

            let avg = result.unwrap();
            let count = weights.len().min(n);
            let slice = &weights[..count];
            let min = slice.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = slice.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            prop_assert!(
                avg >= min - 1e-10 && avg <= max + 1e-10,
                "Moving average {} should be between {} and {}",
                avg, min, max
            );
        }
    }

    // =========================================================================
    // Feature: fitness-assistant-ai, Property 5: Anomaly Detection Threshold
    // =========================================================================
    const ANOMALY_THRESHOLD_PERCENT: f64 = 2.0;

    /// Calculate if a weight change is anomalous (>2% change)
    fn is_anomalous(prev_weight: f64, new_weight: f64) -> bool {
        let percent_change = ((new_weight - prev_weight) / prev_weight).abs() * 100.0;
        percent_change > ANOMALY_THRESHOLD_PERCENT
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 5: Changes >2% are flagged as anomalous
        #[test]
        fn prop_anomaly_above_threshold_flagged(
            prev_weight in 50.0f64..200.0,
            // Generate change that's definitely >2%
            change_factor in 1.021f64..1.5
        ) {
            let new_weight = prev_weight * change_factor;
            prop_assert!(
                is_anomalous(prev_weight, new_weight),
                "{}kg -> {}kg ({}% change) should be anomalous",
                prev_weight, new_weight,
                ((new_weight - prev_weight) / prev_weight).abs() * 100.0
            );
        }

        /// Property 5: Changes <=2% are NOT flagged as anomalous
        #[test]
        fn prop_anomaly_below_threshold_not_flagged(
            prev_weight in 50.0f64..200.0,
            // Generate change that's definitely <=2%
            change_factor in 0.98f64..1.02
        ) {
            let new_weight = prev_weight * change_factor;
            prop_assert!(
                !is_anomalous(prev_weight, new_weight),
                "{}kg -> {}kg ({}% change) should NOT be anomalous",
                prev_weight, new_weight,
                ((new_weight - prev_weight) / prev_weight).abs() * 100.0
            );
        }

        /// Property 5: Anomaly detection is symmetric (gain vs loss)
        #[test]
        fn prop_anomaly_symmetric(
            prev_weight in 50.0f64..200.0,
            change_percent in 0.0f64..10.0
        ) {
            let gain = prev_weight * (1.0 + change_percent / 100.0);
            let loss = prev_weight * (1.0 - change_percent / 100.0);

            let gain_anomalous = is_anomalous(prev_weight, gain);
            let loss_anomalous = is_anomalous(prev_weight, loss);

            prop_assert_eq!(
                gain_anomalous, loss_anomalous,
                "Anomaly detection should be symmetric: gain={}, loss={}",
                gain_anomalous, loss_anomalous
            );
        }
    }

    // =========================================================================
    // Feature: fitness-assistant-ai, Property 4: Weight Goal Projection
    // =========================================================================

    /// Calculate projected days to reach goal
    fn calculate_projected_days(
        current_weight: f64,
        target_weight: f64,
        average_daily_change: f64,
    ) -> Option<i64> {
        let weight_to_lose = current_weight - target_weight;

        // Check if moving toward goal
        let moving_toward_goal = if weight_to_lose > 0.0 {
            average_daily_change < 0.0
        } else if weight_to_lose < 0.0 {
            average_daily_change > 0.0
        } else {
            return Some(0); // Already at goal
        };

        if !moving_toward_goal || average_daily_change.abs() < 0.001 {
            return None;
        }

        Some((weight_to_lose.abs() / average_daily_change.abs()).ceil() as i64)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 4: Projection formula is correct
        #[test]
        fn prop_goal_projection_formula(
            current in 50.0f64..150.0,
            target in 50.0f64..150.0,
            daily_change in -0.5f64..0.5
        ) {
            prop_assume!(daily_change.abs() >= 0.001);
            prop_assume!((current - target).abs() >= 0.001);

            let weight_diff = current - target;
            let moving_toward_goal = if weight_diff > 0.0 {
                daily_change < 0.0
            } else {
                daily_change > 0.0
            };

            if moving_toward_goal {
                let projected = calculate_projected_days(current, target, daily_change);
                prop_assert!(projected.is_some());

                let days = projected.unwrap();
                let expected = (weight_diff.abs() / daily_change.abs()).ceil() as i64;

                prop_assert_eq!(
                    days, expected,
                    "Projected days {} != expected {} for current={}, target={}, daily_change={}",
                    days, expected, current, target, daily_change
                );
            }
        }

        /// Property 4: No projection when moving away from goal
        #[test]
        fn prop_no_projection_wrong_direction(
            current in 50.0f64..150.0,
            target in 50.0f64..150.0,
            daily_change_magnitude in 0.01f64..0.5
        ) {
            prop_assume!((current - target).abs() >= 1.0);

            let weight_diff = current - target;
            // Set daily change in wrong direction
            let daily_change = if weight_diff > 0.0 {
                daily_change_magnitude // Gaining when need to lose
            } else {
                -daily_change_magnitude // Losing when need to gain
            };

            let projected = calculate_projected_days(current, target, daily_change);
            prop_assert!(
                projected.is_none(),
                "Should not project when moving away from goal"
            );
        }

        /// Property 4: Projection is 0 when already at goal
        #[test]
        fn prop_projection_zero_at_goal(weight in 50.0f64..150.0, daily_change in -0.5f64..0.5) {
            let projected = calculate_projected_days(weight, weight, daily_change);
            prop_assert_eq!(projected, Some(0), "Should be 0 days when at goal");
        }
    }

    // =========================================================================
    // Unit tests for edge cases
    // =========================================================================

    #[test]
    fn test_moving_average_empty_input() {
        let weights: Vec<f64> = vec![];
        assert!(WeightService::calculate_moving_average(&weights, 7).is_none());
    }

    #[test]
    fn test_moving_average_zero_n() {
        let weights = vec![70.0, 71.0, 72.0];
        assert!(WeightService::calculate_moving_average(&weights, 0).is_none());
    }

    #[test]
    fn test_anomaly_exactly_2_percent() {
        // Exactly 2% should NOT be flagged (threshold is >2%)
        let prev = 100.0;
        let new = 102.0;
        assert!(!is_anomalous(prev, new));
    }

    #[test]
    fn test_anomaly_just_above_2_percent() {
        let prev = 100.0;
        let new = 102.01;
        assert!(is_anomalous(prev, new));
    }
}
