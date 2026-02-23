#[cfg(test)]
mod tests {
    use super::super::constants::*;
    use super::super::coverage::ServiceCoverageGrid;

    #[test]
    fn test_happiness_bounds() {
        // Base happiness should be in range
        assert!(BASE_HAPPINESS >= 0.0 && BASE_HAPPINESS <= 100.0);
    }

    #[test]
    fn test_all_factors_affect_output() {
        // Verify all bonuses/penalties are non-zero
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
        // Max theoretical happiness: all bonuses, no penalties, max land value (255/50 = 5.1)
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
        // With all bonuses the raw sum exceeds 100, but clamp caps at 100
        assert!(
            max > 100.0,
            "max happiness {} should exceed 100 before clamping",
            max
        );
        // Verify it is meaningful without land value
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
}
