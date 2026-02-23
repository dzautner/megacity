#[cfg(test)]
mod tests {
    use super::super::constants::*;
    use super::super::coverage::ServiceCoverageGrid;

    // ===================================================================
    // Original tests (preserved)
    // ===================================================================

    #[test]
    fn test_happiness_bounds() {
        assert!(BASE_HAPPINESS >= 0.0 && BASE_HAPPINESS <= 100.0);
    }

    #[test]
    fn test_all_factors_affect_output() {
        assert!(EMPLOYED_BONUS > 0.0);
        assert!(SHORT_COMMUTE_BONUS > 0.0);
        assert!(POWER_BONUS > 0.0);
        assert!(WATER_BONUS > 0.0);
        assert!(HEALTH_COVERAGE_BONUS > 0.0);
        assert!(EDUCATION_BONUS > 0.0);
        assert!(POLICE_BONUS > 0.0);
        assert!(PARK_BONUS > 0.0);
        assert!(ENTERTAINMENT_BONUS > 0.0);
        assert!(HIGH_TAX_PENALTY > 0.0);
        assert!(CONGESTION_PENALTY > 0.0);
        assert!(GARBAGE_PENALTY > 0.0);
    }

    #[test]
    fn test_max_happiness_reachable() {
        let max_land_bonus: f32 = 255.0 / 50.0;
        let max = BASE_HAPPINESS
            + EMPLOYED_BONUS
            + SHORT_COMMUTE_BONUS
            + POWER_BONUS
            + WATER_BONUS
            + HEALTH_COVERAGE_BONUS
            + EDUCATION_BONUS
            + POLICE_BONUS
            + PARK_BONUS
            + ENTERTAINMENT_BONUS
            + max_land_bonus;
        assert!(
            max > 100.0,
            "max happiness {} should exceed 100 before clamping",
            max
        );
        let max_no_land = BASE_HAPPINESS
            + EMPLOYED_BONUS
            + SHORT_COMMUTE_BONUS
            + POWER_BONUS
            + WATER_BONUS
            + HEALTH_COVERAGE_BONUS
            + EDUCATION_BONUS
            + POLICE_BONUS
            + PARK_BONUS
            + ENTERTAINMENT_BONUS;
        assert!(
            max_no_land > 80.0,
            "max happiness {} (no land) should be > 80 to be meaningful",
            max_no_land
        );
    }

    #[test]
    fn test_service_coverage_dirty_flag_default() {
        let grid = ServiceCoverageGrid::default();
        assert!(grid.dirty, "should start dirty so first update runs");
    }

    #[test]
    fn test_service_coverage_clear_resets_all() {
        let mut grid = ServiceCoverageGrid::default();
        let idx = ServiceCoverageGrid::idx(10, 10);
        grid.flags[idx] = COVERAGE_HEALTH
            | COVERAGE_EDUCATION
            | COVERAGE_POLICE
            | COVERAGE_PARK
            | COVERAGE_ENTERTAINMENT;
        grid.clear();
        assert!(!grid.has_health(idx));
        assert!(!grid.has_education(idx));
        assert!(!grid.has_police(idx));
        assert!(!grid.has_park(idx));
        assert!(!grid.has_entertainment(idx));
    }

    // ===================================================================
    // Diminishing returns function tests
    // ===================================================================

    #[test]
    fn test_diminishing_returns_zero_input() {
        let result = diminishing_returns(0.0, DIMINISHING_K_DEFAULT);
        assert!(
            result.abs() < 0.001,
            "diminishing_returns(0) should be ~0, got {}",
            result
        );
    }

    #[test]
    fn test_diminishing_returns_one_input() {
        let result = diminishing_returns(1.0, DIMINISHING_K_DEFAULT);
        assert!(
            result > 0.9,
            "diminishing_returns(1) should be close to 1.0, got {}",
            result
        );
        assert!(
            result < 1.0,
            "diminishing_returns(1) should be < 1.0, got {}",
            result
        );
    }

    #[test]
    fn test_diminishing_returns_monotonically_increasing() {
        let steps = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let mut prev = -1.0;
        for &x in &steps {
            let val = diminishing_returns(x, DIMINISHING_K_DEFAULT);
            assert!(
                val > prev,
                "diminishing_returns should be monotonically increasing: f({}) = {} <= {}",
                x,
                val,
                prev
            );
            prev = val;
        }
    }

    #[test]
    fn test_diminishing_returns_marginal_decrease() {
        // The increment from 0.0->0.25 should be larger than 0.75->1.0
        let first_quarter = diminishing_returns(0.25, DIMINISHING_K_DEFAULT)
            - diminishing_returns(0.0, DIMINISHING_K_DEFAULT);
        let last_quarter = diminishing_returns(1.0, DIMINISHING_K_DEFAULT)
            - diminishing_returns(0.75, DIMINISHING_K_DEFAULT);
        assert!(
            first_quarter > last_quarter,
            "First quarter gain ({}) should exceed last quarter gain ({})",
            first_quarter,
            last_quarter
        );
    }

    #[test]
    fn test_diminishing_returns_clamps_input() {
        // Negative input should be clamped to 0
        let neg = diminishing_returns(-0.5, DIMINISHING_K_DEFAULT);
        assert!(
            neg.abs() < 0.001,
            "diminishing_returns(-0.5) should be ~0, got {}",
            neg
        );
        // Input > 1 should be clamped to 1
        let over = diminishing_returns(2.0, DIMINISHING_K_DEFAULT);
        let at_one = diminishing_returns(1.0, DIMINISHING_K_DEFAULT);
        assert!(
            (over - at_one).abs() < 0.001,
            "diminishing_returns(2.0) should equal diminishing_returns(1.0)"
        );
    }

    #[test]
    fn test_diminishing_returns_different_k_values() {
        // Higher k should saturate faster
        let low_k = diminishing_returns(0.5, 1.0);
        let high_k = diminishing_returns(0.5, 5.0);
        assert!(
            high_k > low_k,
            "Higher k should give higher output at x=0.5: k=1 -> {}, k=5 -> {}",
            low_k,
            high_k
        );
    }

    // ===================================================================
    // Service coverage ratio tests
    // ===================================================================

    #[test]
    fn test_service_coverage_ratio_none() {
        assert!(
            service_coverage_ratio(0).abs() < 0.001,
            "No coverage flags should give ratio 0"
        );
    }

    #[test]
    fn test_service_coverage_ratio_all() {
        let all = COVERAGE_HEALTH
            | COVERAGE_EDUCATION
            | COVERAGE_POLICE
            | COVERAGE_PARK
            | COVERAGE_ENTERTAINMENT
            | COVERAGE_TELECOM
            | COVERAGE_TRANSPORT
            | COVERAGE_FIRE;
        let ratio = service_coverage_ratio(all);
        assert!(
            (ratio - 1.0).abs() < 0.001,
            "All 8 flags should give ratio 1.0, got {}",
            ratio
        );
    }

    #[test]
    fn test_service_coverage_ratio_partial() {
        let partial = COVERAGE_HEALTH | COVERAGE_EDUCATION; // 2 out of 8
        let ratio = service_coverage_ratio(partial);
        assert!(
            (ratio - 0.25).abs() < 0.001,
            "2/8 flags should give ratio 0.25, got {}",
            ratio
        );
    }

    // ===================================================================
    // Critical threshold tests
    // ===================================================================

    #[test]
    fn test_critical_thresholds_are_positive() {
        assert!(CRITICAL_NO_WATER_PENALTY > 0.0);
        assert!(CRITICAL_NO_POWER_PENALTY > 0.0);
        assert!(CRITICAL_HEALTH_PENALTY > 0.0);
        assert!(CRITICAL_NEEDS_PENALTY > 0.0);
        assert!(CRITICAL_CRIME_PENALTY > 0.0);
    }

    #[test]
    fn test_critical_no_water_severe() {
        // Total penalty for no water should be at least 40
        let total = NO_WATER_PENALTY + CRITICAL_NO_WATER_PENALTY;
        assert!(
            total >= 40.0,
            "Total no-water penalty should be >= 40, got {}",
            total
        );
    }

    #[test]
    fn test_critical_no_power_severe() {
        let total = NO_POWER_PENALTY + CRITICAL_NO_POWER_PENALTY;
        assert!(
            total >= 35.0,
            "Total no-power penalty should be >= 35, got {}",
            total
        );
    }

    // ===================================================================
    // Wealth satisfaction tests
    // ===================================================================

    #[test]
    fn test_wealth_satisfaction_zero_savings() {
        let result = wealth_satisfaction(0.0);
        assert!(
            (result - (-WEALTH_POVERTY_PENALTY)).abs() < 0.001,
            "Zero savings should give poverty penalty, got {}",
            result
        );
    }

    #[test]
    fn test_wealth_satisfaction_negative_savings() {
        let result = wealth_satisfaction(-1000.0);
        assert!(
            (result - (-WEALTH_POVERTY_PENALTY)).abs() < 0.001,
            "Negative savings should give poverty penalty, got {}",
            result
        );
    }

    #[test]
    fn test_wealth_satisfaction_comfortable() {
        let result = wealth_satisfaction(WEALTH_COMFORTABLE_SAVINGS);
        assert!(
            result > 0.0,
            "Comfortable savings should give positive bonus, got {}",
            result
        );
        assert!(
            result <= WEALTH_SATISFACTION_MAX_BONUS,
            "Should not exceed max bonus, got {}",
            result
        );
    }

    #[test]
    fn test_wealth_satisfaction_diminishing() {
        // First $2500 should matter more than going from $7500 to $10000
        let first_quarter = wealth_satisfaction(2500.0) - wealth_satisfaction(0.01);
        let last_quarter =
            wealth_satisfaction(WEALTH_COMFORTABLE_SAVINGS) - wealth_satisfaction(7500.0);
        assert!(
            first_quarter > last_quarter,
            "First $2500 gain ({}) should exceed last $2500 gain ({})",
            first_quarter,
            last_quarter
        );
    }

    // ===================================================================
    // Weather happiness factor tests
    // ===================================================================

    #[test]
    fn test_weather_happiness_sunny_positive() {
        let result = weather_happiness_factor(3.0);
        assert!(
            result > 0.0,
            "Positive weather should give positive happiness, got {}",
            result
        );
        assert!(
            result <= WEATHER_HAPPINESS_MAX_BONUS,
            "Should not exceed max bonus {}, got {}",
            WEATHER_HAPPINESS_MAX_BONUS,
            result
        );
    }

    #[test]
    fn test_weather_happiness_storm_negative() {
        let result = weather_happiness_factor(-8.0);
        assert!(
            result < 0.0,
            "Negative weather should give negative happiness, got {}",
            result
        );
        assert!(
            result >= -WEATHER_HAPPINESS_MAX_PENALTY,
            "Should not exceed max penalty {}, got {}",
            WEATHER_HAPPINESS_MAX_PENALTY,
            result
        );
    }

    #[test]
    fn test_weather_happiness_zero_modifier() {
        let result = weather_happiness_factor(0.0);
        assert!(
            result.abs() < 0.001,
            "Zero weather modifier should give ~0 happiness, got {}",
            result
        );
    }

    // ===================================================================
    // Update interval test
    // ===================================================================

    #[test]
    fn test_happiness_update_interval() {
        assert!(
            HAPPINESS_UPDATE_INTERVAL <= 50u64,
            "Update interval should be <= 50 ticks, got {}",
            HAPPINESS_UPDATE_INTERVAL
        );
        assert!(
            HAPPINESS_UPDATE_INTERVAL >= 10u64,
            "Update interval should be >= 10 ticks, got {}",
            HAPPINESS_UPDATE_INTERVAL
        );
    }
}
