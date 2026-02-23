//! Tests for flood protection systems: failure logic, state, save/load, and effectiveness.

#[cfg(test)]
mod tests {
    use crate::flood_protection::systems::*;
    use crate::flood_protection::types::*;
    use crate::Saveable;

    // -------------------------------------------------------------------------
    // Should-fail deterministic test
    // -------------------------------------------------------------------------

    #[test]
    fn test_should_fail_zero_prob() {
        assert!(
            !should_fail(0.0, 12345),
            "Zero probability should never fail"
        );
    }

    #[test]
    fn test_should_fail_certain() {
        assert!(
            should_fail(1.0, 12345),
            "Probability 1.0 should always fail"
        );
    }

    #[test]
    fn test_should_fail_deterministic() {
        let result1 = should_fail(0.5, 42);
        let result2 = should_fail(0.5, 42);
        assert_eq!(result1, result2, "Same inputs should give same result");
    }

    // -------------------------------------------------------------------------
    // Overtopping amplification tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_overtopping_amplification_factor() {
        let flood_depth = 12.0_f32;
        let amplified = flood_depth * OVERTOPPING_AMPLIFICATION;
        assert!(
            (amplified - 18.0).abs() < f32::EPSILON,
            "12 ft flood with 1.5x amplification should give 18 ft, got {}",
            amplified
        );
    }

    // -------------------------------------------------------------------------
    // FloodProtectionState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_state() {
        let state = FloodProtectionState::default();
        assert!(state.structures.is_empty());
        assert_eq!(state.active_count, 0);
        assert_eq!(state.failed_count, 0);
        assert!(state.annual_maintenance_cost.abs() < f64::EPSILON);
        assert!(state.maintenance_funded);
        assert!(state.damage_prevented.abs() < f64::EPSILON);
        assert_eq!(state.last_maintenance_day, 0);
        assert_eq!(state.overtopping_events, 0);
    }

    #[test]
    fn test_state_with_structures() {
        let mut state = FloodProtectionState::default();
        state
            .structures
            .push(ProtectionStructure::new(5, 5, ProtectionType::Levee));
        state
            .structures
            .push(ProtectionStructure::new(6, 5, ProtectionType::Seawall));
        state
            .structures
            .push(ProtectionStructure::new(7, 5, ProtectionType::Floodgate));
        assert_eq!(state.structures.len(), 3);
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_empty_returns_none() {
        let state = FloodProtectionState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Empty state should return None for save"
        );
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut state = FloodProtectionState::default();
        state
            .structures
            .push(ProtectionStructure::new(10, 20, ProtectionType::Levee));
        state
            .structures
            .push(ProtectionStructure::new(11, 20, ProtectionType::Seawall));
        state.active_count = 2;
        state.annual_maintenance_cost = 4000.0;
        state.damage_prevented = 50000.0;

        let bytes = state.save_to_bytes().expect("should have bytes");
        let loaded = FloodProtectionState::load_from_bytes(&bytes);

        assert_eq!(loaded.structures.len(), 2);
        assert_eq!(loaded.structures[0].grid_x, 10);
        assert_eq!(loaded.structures[0].grid_y, 20);
        assert_eq!(loaded.structures[0].protection_type, ProtectionType::Levee);
        assert_eq!(
            loaded.structures[1].protection_type,
            ProtectionType::Seawall
        );
        assert_eq!(loaded.active_count, 2);
        assert!((loaded.annual_maintenance_cost - 4000.0).abs() < f64::EPSILON);
        assert!((loaded.damage_prevented - 50000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_saveable_key() {
        assert_eq!(
            FloodProtectionState::SAVE_KEY,
            "flood_protection",
            "Save key should be 'flood_protection'"
        );
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_positive() {
        assert!(LEVEE_DESIGN_HEIGHT > 0.0);
        assert!(SEAWALL_DESIGN_HEIGHT > 0.0);
        assert!(FLOODGATE_DESIGN_HEIGHT > 0.0);
        assert!(MAINTENANCE_COST_PER_CELL_PER_YEAR > 0.0);
        assert!(DEGRADATION_RATE_PER_TICK > 0.0);
        assert!(RECOVERY_RATE_PER_TICK > 0.0);
        assert!(BASE_FAILURE_PROB > 0.0);
        assert!(AGE_FAILURE_FACTOR > 0.0);
        assert!(OVERTOPPING_AMPLIFICATION > 1.0);
    }

    #[test]
    fn test_seawall_higher_than_levee() {
        assert!(
            SEAWALL_DESIGN_HEIGHT > LEVEE_DESIGN_HEIGHT,
            "Seawall should have higher protection than levee"
        );
    }

    #[test]
    fn test_floodgate_between_levee_and_seawall() {
        assert!(
            FLOODGATE_DESIGN_HEIGHT > LEVEE_DESIGN_HEIGHT,
            "Floodgate should be higher than levee"
        );
        assert!(
            FLOODGATE_DESIGN_HEIGHT < SEAWALL_DESIGN_HEIGHT,
            "Floodgate should be lower than seawall"
        );
    }

    #[test]
    fn test_degradation_faster_than_recovery() {
        assert!(
            DEGRADATION_RATE_PER_TICK > RECOVERY_RATE_PER_TICK,
            "Degradation should be faster than recovery"
        );
    }

    // -------------------------------------------------------------------------
    // Protection effectiveness tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_levee_protects_below_design_height() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        let flood_depth = 8.0_f32;
        let effective = s.effective_height();
        // Flood is below design height, levee should protect
        assert!(
            flood_depth <= effective,
            "8 ft flood should be within 10 ft levee protection"
        );
    }

    #[test]
    fn test_levee_overtopped_above_design_height() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        let flood_depth = 12.0_f32;
        let effective = s.effective_height();
        // Flood exceeds design height
        assert!(
            flood_depth > effective,
            "12 ft flood should overtop 10 ft levee"
        );
    }

    #[test]
    fn test_degraded_levee_overtopped_at_lower_depth() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.condition = 0.6; // effective height = 6.0
        let flood_depth = 7.0_f32;
        let effective = s.effective_height();
        assert!(
            flood_depth > effective,
            "7 ft flood should overtop degraded levee with 6 ft effective height"
        );
    }

    #[test]
    fn test_seawall_protects_higher_surge() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Seawall);
        let surge = 14.0_f32;
        assert!(
            surge <= s.effective_height(),
            "14 ft surge should be within 15 ft seawall protection"
        );
    }

    // -------------------------------------------------------------------------
    // Integration-style data structure tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_multiple_structure_types() {
        let mut state = FloodProtectionState::default();
        state
            .structures
            .push(ProtectionStructure::new(1, 1, ProtectionType::Levee));
        state
            .structures
            .push(ProtectionStructure::new(2, 1, ProtectionType::Seawall));
        state
            .structures
            .push(ProtectionStructure::new(3, 1, ProtectionType::Floodgate));

        assert_eq!(state.structures[0].effective_height(), 10.0);
        assert_eq!(state.structures[1].effective_height(), 15.0);
        assert_eq!(state.structures[2].effective_height(), 12.0);
    }

    #[test]
    fn test_aging_increases_failure_probability() {
        let young = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        let mut old = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        old.age_days = 7200; // 20 years

        let prob_young = young.failure_probability();
        let prob_old = old.failure_probability();

        assert!(
            prob_old > prob_young,
            "20-year-old structure should have higher failure prob ({}) than new ({})",
            prob_old,
            prob_young
        );
    }

    #[test]
    fn test_low_condition_increases_failure_probability() {
        let good = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        let mut poor = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        poor.condition = 0.25;

        let prob_good = good.failure_probability();
        let prob_poor = poor.failure_probability();

        assert!(
            prob_poor > prob_good * 4.0,
            "Condition 0.25 should multiply failure prob by ~16x: poor={}, good={}",
            prob_poor,
            prob_good
        );
    }

    #[test]
    fn test_floodgate_toggle() {
        let mut gate = ProtectionStructure::new(5, 5, ProtectionType::Floodgate);
        assert!(!gate.gate_open);
        assert!((gate.effective_height() - 12.0).abs() < f32::EPSILON);

        gate.gate_open = true;
        assert!(gate.effective_height().abs() < f32::EPSILON);

        gate.gate_open = false;
        assert!((gate.effective_height() - 12.0).abs() < f32::EPSILON);
    }
}
