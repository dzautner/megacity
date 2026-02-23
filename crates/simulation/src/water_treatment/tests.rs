//! Unit tests for water treatment types and effluent quality calculations.

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::water_treatment::{
        calculate_effluent_quality, PlantState, TreatmentLevel, WaterTreatmentState,
    };

    // -------------------------------------------------------------------------
    // TreatmentLevel enum tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_treatment_level_default_is_none() {
        assert_eq!(TreatmentLevel::default(), TreatmentLevel::None);
    }

    #[test]
    fn test_removal_efficiency_values() {
        assert!((TreatmentLevel::None.removal_efficiency() - 0.0).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Primary.removal_efficiency() - 0.60).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Secondary.removal_efficiency() - 0.85).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Tertiary.removal_efficiency() - 0.95).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Advanced.removal_efficiency() - 0.99).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cost_per_million_gallons_values() {
        assert!((TreatmentLevel::None.cost_per_million_gallons() - 0.0).abs() < f64::EPSILON);
        assert!(
            (TreatmentLevel::Primary.cost_per_million_gallons() - 1_000.0).abs() < f64::EPSILON
        );
        assert!(
            (TreatmentLevel::Secondary.cost_per_million_gallons() - 2_000.0).abs() < f64::EPSILON
        );
        assert!(
            (TreatmentLevel::Tertiary.cost_per_million_gallons() - 5_000.0).abs() < f64::EPSILON
        );
        assert!(
            (TreatmentLevel::Advanced.cost_per_million_gallons() - 10_000.0).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_cost_scales_with_level() {
        // Each successive level should cost more per MG
        let levels = [
            TreatmentLevel::Primary,
            TreatmentLevel::Secondary,
            TreatmentLevel::Tertiary,
            TreatmentLevel::Advanced,
        ];
        for pair in levels.windows(2) {
            assert!(
                pair[1].cost_per_million_gallons() > pair[0].cost_per_million_gallons(),
                "{:?} should cost more than {:?}",
                pair[1],
                pair[0]
            );
        }
    }

    #[test]
    fn test_upgrade_cost_values() {
        assert_eq!(TreatmentLevel::None.upgrade_cost(), Some(25_000.0));
        assert_eq!(TreatmentLevel::Primary.upgrade_cost(), Some(50_000.0));
        assert_eq!(TreatmentLevel::Secondary.upgrade_cost(), Some(100_000.0));
        assert_eq!(TreatmentLevel::Tertiary.upgrade_cost(), Some(200_000.0));
        assert_eq!(TreatmentLevel::Advanced.upgrade_cost(), Option::None);
    }

    #[test]
    fn test_next_level_chain() {
        let mut level = TreatmentLevel::None;
        let expected = [
            TreatmentLevel::Primary,
            TreatmentLevel::Secondary,
            TreatmentLevel::Tertiary,
            TreatmentLevel::Advanced,
        ];
        for expected_next in &expected {
            let next = level.next_level().expect("should have a next level");
            assert_eq!(next, *expected_next);
            level = next;
        }
        assert!(
            level.next_level().is_none(),
            "Advanced should have no next level"
        );
    }

    #[test]
    fn test_base_capacity_mgd_values() {
        assert!((TreatmentLevel::None.base_capacity_mgd() - 0.0).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Primary.base_capacity_mgd() - 10.0).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Secondary.base_capacity_mgd() - 8.0).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Tertiary.base_capacity_mgd() - 5.0).abs() < f32::EPSILON);
        assert!((TreatmentLevel::Advanced.base_capacity_mgd() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_treatment_level_names() {
        assert_eq!(TreatmentLevel::None.name(), "None");
        assert_eq!(TreatmentLevel::Primary.name(), "Primary");
        assert_eq!(TreatmentLevel::Secondary.name(), "Secondary");
        assert_eq!(TreatmentLevel::Tertiary.name(), "Tertiary");
        assert_eq!(TreatmentLevel::Advanced.name(), "Advanced");
    }

    // -------------------------------------------------------------------------
    // PlantState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_plant_state_new() {
        let plant = PlantState::new(TreatmentLevel::Primary);
        assert_eq!(plant.level, TreatmentLevel::Primary);
        assert!((plant.capacity_mgd - 10.0).abs() < f32::EPSILON);
        assert!((plant.current_flow_mgd - 0.0).abs() < f32::EPSILON);
        assert!((plant.effluent_quality - 0.0).abs() < f32::EPSILON);
        assert!((plant.period_cost - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_plant_state_new_advanced() {
        let plant = PlantState::new(TreatmentLevel::Advanced);
        assert_eq!(plant.level, TreatmentLevel::Advanced);
        assert!((plant.capacity_mgd - 3.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // WaterTreatmentState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_water_treatment_state_default() {
        let state = WaterTreatmentState::default();
        assert!(state.plants.is_empty());
        assert!((state.total_capacity_mgd - 0.0).abs() < f32::EPSILON);
        assert!((state.total_flow_mgd - 0.0).abs() < f32::EPSILON);
        assert!((state.avg_effluent_quality - 0.0).abs() < f32::EPSILON);
        assert!((state.total_period_cost - 0.0).abs() < f64::EPSILON);
        assert!((state.city_demand_mgd - 0.0).abs() < f32::EPSILON);
        assert!((state.treatment_coverage - 0.0).abs() < f32::EPSILON);
        assert!((state.avg_input_quality - 0.5).abs() < f32::EPSILON);
        assert!((state.disease_risk - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_register_and_remove_plant() {
        let mut state = WaterTreatmentState::default();
        let entity = Entity::from_raw(42);
        state.register_plant(entity, TreatmentLevel::Primary);
        assert!(state.plants.contains_key(&entity));
        assert_eq!(state.plants[&entity].level, TreatmentLevel::Primary);

        state.remove_plant(entity);
        assert!(!state.plants.contains_key(&entity));
    }

    #[test]
    fn test_upgrade_plant() {
        let mut state = WaterTreatmentState::default();
        let entity = Entity::from_raw(1);
        state.register_plant(entity, TreatmentLevel::Primary);

        // Upgrade from Primary -> Secondary
        let cost = state.upgrade_plant(entity);
        assert_eq!(cost, Some(50_000.0));
        assert_eq!(state.plants[&entity].level, TreatmentLevel::Secondary);
        assert!((state.plants[&entity].capacity_mgd - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_upgrade_plant_at_max_level() {
        let mut state = WaterTreatmentState::default();
        let entity = Entity::from_raw(2);
        state.register_plant(entity, TreatmentLevel::Advanced);

        let cost = state.upgrade_plant(entity);
        assert!(cost.is_none(), "Advanced should not be upgradeable");
        assert_eq!(state.plants[&entity].level, TreatmentLevel::Advanced);
    }

    #[test]
    fn test_upgrade_nonexistent_plant() {
        let mut state = WaterTreatmentState::default();
        let entity = Entity::from_raw(999);

        let cost = state.upgrade_plant(entity);
        assert!(cost.is_none(), "Nonexistent plant should return None");
    }

    #[test]
    fn test_upgrade_chain_full() {
        let mut state = WaterTreatmentState::default();
        let entity = Entity::from_raw(10);
        state.register_plant(entity, TreatmentLevel::None);

        let expected_costs = [25_000.0, 50_000.0, 100_000.0, 200_000.0];
        let expected_levels = [
            TreatmentLevel::Primary,
            TreatmentLevel::Secondary,
            TreatmentLevel::Tertiary,
            TreatmentLevel::Advanced,
        ];

        for (i, (&expected_cost, &expected_level)) in expected_costs
            .iter()
            .zip(expected_levels.iter())
            .enumerate()
        {
            let cost = state.upgrade_plant(entity);
            assert_eq!(
                cost,
                Some(expected_cost),
                "Upgrade {} should cost {}",
                i,
                expected_cost
            );
            assert_eq!(state.plants[&entity].level, expected_level);
        }

        // Final upgrade should fail
        assert!(state.upgrade_plant(entity).is_none());
    }

    // -------------------------------------------------------------------------
    // Effluent quality calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_effluent_quality_no_treatment() {
        // No treatment: output equals input
        let result = calculate_effluent_quality(0.3, TreatmentLevel::None);
        assert!(
            (result - 0.3).abs() < 0.001,
            "No treatment should pass through input quality, got {}",
            result
        );
    }

    #[test]
    fn test_effluent_quality_primary() {
        // Input 0.3, Primary (60% removal): 1.0 - (0.7 * 0.4) = 0.72
        let result = calculate_effluent_quality(0.3, TreatmentLevel::Primary);
        let expected = 1.0 - (0.7 * 0.4);
        assert!(
            (result - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            result
        );
    }

    #[test]
    fn test_effluent_quality_secondary() {
        // Input 0.3, Secondary (85% removal): 1.0 - (0.7 * 0.15) = 0.895
        let result = calculate_effluent_quality(0.3, TreatmentLevel::Secondary);
        let expected = 1.0 - (0.7 * 0.15);
        assert!(
            (result - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            result
        );
    }

    #[test]
    fn test_effluent_quality_tertiary() {
        // Input 0.3, Tertiary (95% removal): 1.0 - (0.7 * 0.05) = 0.965
        let result = calculate_effluent_quality(0.3, TreatmentLevel::Tertiary);
        let expected = 1.0 - (0.7 * 0.05);
        assert!(
            (result - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            result
        );
    }

    #[test]
    fn test_effluent_quality_advanced() {
        // Input 0.3, Advanced (99% removal): 1.0 - (0.7 * 0.01) = 0.993
        let result = calculate_effluent_quality(0.3, TreatmentLevel::Advanced);
        let expected = 1.0 - (0.7 * 0.01);
        assert!(
            (result - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            result
        );
    }

    #[test]
    fn test_effluent_quality_pure_input() {
        // Already pure water: output should stay pure
        let result = calculate_effluent_quality(1.0, TreatmentLevel::Primary);
        assert!(
            (result - 1.0).abs() < 0.001,
            "Pure input should produce pure output, got {}",
            result
        );
    }

    #[test]
    fn test_effluent_quality_fully_contaminated_input() {
        // Fully contaminated (0.0 quality), Primary: 1.0 - (1.0 * 0.4) = 0.6
        let result = calculate_effluent_quality(0.0, TreatmentLevel::Primary);
        assert!(
            (result - 0.60).abs() < 0.001,
            "Expected 0.60 for fully contaminated + Primary, got {}",
            result
        );
    }

    #[test]
    fn test_effluent_quality_increases_with_level() {
        let input = 0.3;
        let levels = [
            TreatmentLevel::None,
            TreatmentLevel::Primary,
            TreatmentLevel::Secondary,
            TreatmentLevel::Tertiary,
            TreatmentLevel::Advanced,
        ];
        let mut prev_quality = 0.0_f32;
        for level in &levels {
            let quality = calculate_effluent_quality(input, *level);
            assert!(
                quality >= prev_quality,
                "{:?} quality {} should be >= previous {}",
                level,
                quality,
                prev_quality
            );
            prev_quality = quality;
        }
    }

    #[test]
    fn test_effluent_quality_clamped() {
        // Input quality beyond bounds should be clamped
        let result = calculate_effluent_quality(1.5, TreatmentLevel::Advanced);
        assert!(result <= 1.0, "Quality should be clamped to 1.0");
        assert!(result >= 0.0, "Quality should be non-negative");

        let result_neg = calculate_effluent_quality(-0.5, TreatmentLevel::Advanced);
        assert!(result_neg <= 1.0, "Quality should be clamped to 1.0");
        assert!(result_neg >= 0.0, "Quality should be non-negative");
    }
}
