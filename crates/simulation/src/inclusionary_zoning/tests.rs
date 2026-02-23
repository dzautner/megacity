//! Tests for inclusionary zoning module.

#[cfg(test)]
mod tests {
    use crate::districts::DistrictMap;
    use crate::inclusionary_zoning::*;

    // -------------------------------------------------------------------------
    // Default state tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_state() {
        let state = InclusionaryZoningState::default();
        assert!(state.district_configs.is_empty());
        assert_eq!(state.total_affordable_units, 0);
        assert_eq!(state.total_affected_units, 0);
        assert_eq!(state.total_monthly_cost, 0.0);
    }

    #[test]
    fn test_default_config() {
        let config = DistrictInclusionaryConfig::default();
        assert!(!config.enabled);
        assert!(
            (config.affordable_percentage - DEFAULT_AFFORDABLE_PERCENTAGE).abs() < f32::EPSILON
        );
    }

    // -------------------------------------------------------------------------
    // Enable/disable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_enable_district() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        assert!(state.is_enabled(0));
        assert!(!state.is_enabled(1));
    }

    #[test]
    fn test_disable_district() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.disable(0);
        assert!(!state.is_enabled(0));
    }

    #[test]
    fn test_enable_multiple_districts() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.enable(3);
        state.enable(5);
        assert!(state.is_enabled(0));
        assert!(!state.is_enabled(1));
        assert!(state.is_enabled(3));
        assert!(state.is_enabled(5));
        assert_eq!(state.enabled_district_count(), 3);
    }

    #[test]
    fn test_enable_idempotent() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.enable(0);
        assert_eq!(state.enabled_district_count(), 1);
    }

    #[test]
    fn test_disable_nonexistent() {
        let mut state = InclusionaryZoningState::default();
        state.disable(5); // never enabled
        assert!(!state.is_enabled(5));
    }

    // -------------------------------------------------------------------------
    // Affordable percentage tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_affordable_percentage() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        let pct = state.affordable_percentage(0);
        assert!((pct - DEFAULT_AFFORDABLE_PERCENTAGE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_affordable_percentage() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.set_affordable_percentage(0, 0.18);
        assert!((state.affordable_percentage(0) - 0.18).abs() < f32::EPSILON);
    }

    #[test]
    fn test_affordable_percentage_clamped_min() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.set_affordable_percentage(0, 0.01); // below min
        assert!((state.affordable_percentage(0) - MIN_AFFORDABLE_PERCENTAGE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_affordable_percentage_clamped_max() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.set_affordable_percentage(0, 0.50); // above max
        assert!((state.affordable_percentage(0) - MAX_AFFORDABLE_PERCENTAGE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_affordable_percentage_zero_when_disabled() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.set_affordable_percentage(0, 0.15);
        state.disable(0);
        assert_eq!(state.affordable_percentage(0), 0.0);
    }

    #[test]
    fn test_affordable_percentage_zero_when_not_configured() {
        let state = InclusionaryZoningState::default();
        assert_eq!(state.affordable_percentage(99), 0.0);
    }

    // -------------------------------------------------------------------------
    // FAR bonus tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_far_bonus_at_min() {
        let bonus = calculate_far_bonus(MIN_AFFORDABLE_PERCENTAGE);
        assert!((bonus - MIN_FAR_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_far_bonus_at_max() {
        let bonus = calculate_far_bonus(MAX_AFFORDABLE_PERCENTAGE);
        assert!((bonus - MAX_FAR_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_far_bonus_at_midpoint() {
        let mid_pct = (MIN_AFFORDABLE_PERCENTAGE + MAX_AFFORDABLE_PERCENTAGE) / 2.0;
        let expected = (MIN_FAR_BONUS + MAX_FAR_BONUS) / 2.0;
        let bonus = calculate_far_bonus(mid_pct);
        assert!(
            (bonus - expected).abs() < 0.001,
            "midpoint bonus should be ~{}: got {}",
            expected,
            bonus
        );
    }

    #[test]
    fn test_far_bonus_zero_when_no_policy() {
        let bonus = calculate_far_bonus(0.0);
        assert!(bonus.abs() < f32::EPSILON);
    }

    #[test]
    fn test_far_bonus_clamped_below_min() {
        // Even a very small percentage gets clamped to MIN range
        let bonus = calculate_far_bonus(0.05);
        assert!((bonus - MIN_FAR_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_far_bonus_clamped_above_max() {
        let bonus = calculate_far_bonus(0.50);
        assert!((bonus - MAX_FAR_BONUS).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Affordable units calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_affordable_units_basic() {
        // 100 units at 15% = 15 affordable
        let units = calculate_affordable_units(100, 0.15);
        assert_eq!(units, 15);
    }

    #[test]
    fn test_affordable_units_rounds_up() {
        // 10 units at 15% = 1.5, ceil = 2
        let units = calculate_affordable_units(10, 0.15);
        assert_eq!(units, 2);
    }

    #[test]
    fn test_affordable_units_zero_capacity() {
        let units = calculate_affordable_units(0, 0.15);
        assert_eq!(units, 0);
    }

    #[test]
    fn test_affordable_units_zero_percentage() {
        let units = calculate_affordable_units(100, 0.0);
        assert_eq!(units, 0);
    }

    #[test]
    fn test_affordable_units_at_min() {
        // 100 units at 10% = 10
        let units = calculate_affordable_units(100, MIN_AFFORDABLE_PERCENTAGE);
        assert_eq!(units, 10);
    }

    #[test]
    fn test_affordable_units_at_max() {
        // 100 units at 20% = 20
        let units = calculate_affordable_units(100, MAX_AFFORDABLE_PERCENTAGE);
        assert_eq!(units, 20);
    }

    #[test]
    fn test_affordable_units_capped_at_capacity() {
        // Edge case: very high percentage shouldn't exceed capacity
        let units = calculate_affordable_units(5, 0.20);
        assert!(units <= 5);
    }

    // -------------------------------------------------------------------------
    // Effective capacity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_effective_capacity_basic() {
        // 100 units at 15% affordable = 85 effective
        let effective = calculate_effective_capacity(100, 0.15);
        assert_eq!(effective, 85);
    }

    #[test]
    fn test_effective_capacity_zero_percentage() {
        let effective = calculate_effective_capacity(100, 0.0);
        assert_eq!(effective, 100);
    }

    #[test]
    fn test_effective_capacity_small_building() {
        // 10 units at 15% = 2 affordable, 8 effective
        let effective = calculate_effective_capacity(10, 0.15);
        assert_eq!(effective, 8);
    }

    #[test]
    fn test_effective_capacity_at_max() {
        // 100 units at 20% = 20 affordable, 80 effective
        let effective = calculate_effective_capacity(100, MAX_AFFORDABLE_PERCENTAGE);
        assert_eq!(effective, 80);
    }

    // -------------------------------------------------------------------------
    // Monthly admin cost tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_admin_cost_none() {
        let cost = calculate_monthly_admin_cost(0);
        assert!(cost.abs() < f64::EPSILON);
    }

    #[test]
    fn test_admin_cost_one_district() {
        let cost = calculate_monthly_admin_cost(1);
        assert!((cost - MONTHLY_ADMIN_COST_PER_DISTRICT).abs() < f64::EPSILON);
    }

    #[test]
    fn test_admin_cost_multiple_districts() {
        let cost = calculate_monthly_admin_cost(3);
        assert!((cost - 3.0 * MONTHLY_ADMIN_COST_PER_DISTRICT).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Cell query tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_cell_in_inclusionary_district() {
        let mut state = InclusionaryZoningState::default();
        let mut district_map = DistrictMap::default();

        district_map.assign_cell_to_district(10, 10, 0);

        // Not enabled yet
        assert!(!is_cell_in_inclusionary_district(
            10,
            10,
            &state,
            &district_map
        ));

        // Enable
        state.enable(0);
        assert!(is_cell_in_inclusionary_district(
            10,
            10,
            &state,
            &district_map
        ));

        // Cell not in any district
        assert!(!is_cell_in_inclusionary_district(
            100,
            100,
            &state,
            &district_map
        ));
    }

    #[test]
    fn test_affordable_percentage_for_cell() {
        let mut state = InclusionaryZoningState::default();
        let mut district_map = DistrictMap::default();

        district_map.assign_cell_to_district(10, 10, 0);
        state.enable(0);
        state.set_affordable_percentage(0, 0.18);

        let pct = affordable_percentage_for_cell(10, 10, &state, &district_map);
        assert!((pct - 0.18).abs() < f32::EPSILON);

        // Cell not in district
        let pct2 = affordable_percentage_for_cell(100, 100, &state, &district_map);
        assert_eq!(pct2, 0.0);
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let state = InclusionaryZoningState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_active() {
        use crate::Saveable;
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.set_affordable_percentage(0, 0.18);
        state.enable(3);
        state.total_affordable_units = 42;
        state.total_affected_units = 200;
        state.total_monthly_cost = 16_000.0;

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = InclusionaryZoningState::load_from_bytes(&bytes);

        assert!(restored.is_enabled(0));
        assert!((restored.affordable_percentage(0) - 0.18).abs() < f32::EPSILON);
        assert!(restored.is_enabled(3));
        assert!(!restored.is_enabled(1));
        assert_eq!(restored.total_affordable_units, 42);
        assert_eq!(restored.total_affected_units, 200);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(InclusionaryZoningState::SAVE_KEY, "inclusionary_zoning");
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_are_reasonable() {
        assert!(MIN_AFFORDABLE_PERCENTAGE > 0.0);
        assert!(MIN_AFFORDABLE_PERCENTAGE < MAX_AFFORDABLE_PERCENTAGE);
        assert!(MAX_AFFORDABLE_PERCENTAGE <= 1.0);
        assert!(DEFAULT_AFFORDABLE_PERCENTAGE >= MIN_AFFORDABLE_PERCENTAGE);
        assert!(DEFAULT_AFFORDABLE_PERCENTAGE <= MAX_AFFORDABLE_PERCENTAGE);
        assert!(MIN_FAR_BONUS > 0.0);
        assert!(MIN_FAR_BONUS <= MAX_FAR_BONUS);
        assert!(CONSTRUCTION_RATE_PENALTY > 0.0);
        assert!(CONSTRUCTION_RATE_PENALTY <= 1.0);
        assert!(MONTHLY_ADMIN_COST_PER_DISTRICT > 0.0);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_enable_disable_cycle() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        assert!(state.is_enabled(0));
        assert_eq!(state.enabled_district_count(), 1);

        state.disable(0);
        assert!(!state.is_enabled(0));
        // Config still exists but is disabled
        assert!(state.district_configs.contains_key(&0));

        // Re-enable preserves custom percentage
        state.set_affordable_percentage(0, 0.18);
        state.enable(0);
        assert!((state.affordable_percentage(0) - 0.18).abs() < f32::EPSILON);
    }

    #[test]
    fn test_far_bonus_scales_with_affordable_percentage() {
        // Higher affordable percentage should give a higher FAR bonus
        let bonus_10 = calculate_far_bonus(0.10);
        let bonus_15 = calculate_far_bonus(0.15);
        let bonus_20 = calculate_far_bonus(0.20);
        assert!(bonus_10 < bonus_15);
        assert!(bonus_15 < bonus_20);
    }

    #[test]
    fn test_effective_capacity_plus_affordable_equals_total() {
        // Effective + affordable should always equal the original capacity
        for capacity in [10, 50, 100, 500, 1000] {
            for pct in [0.10, 0.15, 0.20] {
                let affordable = calculate_affordable_units(capacity, pct);
                let effective = calculate_effective_capacity(capacity, pct);
                assert_eq!(
                    affordable + effective,
                    capacity,
                    "capacity={}, pct={}: affordable={} + effective={} != {}",
                    capacity,
                    pct,
                    affordable,
                    effective,
                    capacity
                );
            }
        }
    }
}
