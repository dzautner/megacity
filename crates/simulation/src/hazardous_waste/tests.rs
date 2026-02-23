//! Tests for hazardous waste management.

#[cfg(test)]
mod tests {
    use crate::hazardous_waste::constants::*;
    use crate::hazardous_waste::systems::*;
    use crate::hazardous_waste::types::*;
    use crate::services::{ServiceBuilding, ServiceType};

    // -------------------------------------------------------------------------
    // TreatmentType tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_treatment_type_default() {
        assert_eq!(TreatmentType::default(), TreatmentType::Chemical);
    }

    #[test]
    fn test_treatment_efficiency_values() {
        assert!((TreatmentType::Chemical.efficiency() - 1.0).abs() < f32::EPSILON);
        assert!((TreatmentType::Thermal.efficiency() - 1.2).abs() < f32::EPSILON);
        assert!((TreatmentType::Biological.efficiency() - 0.8).abs() < f32::EPSILON);
        assert!((TreatmentType::Stabilization.efficiency() - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_treatment_cost_multiplier_values() {
        assert!((TreatmentType::Chemical.cost_multiplier() - 1.0).abs() < f64::EPSILON);
        assert!((TreatmentType::Thermal.cost_multiplier() - 1.5).abs() < f64::EPSILON);
        assert!((TreatmentType::Biological.cost_multiplier() - 0.7).abs() < f64::EPSILON);
        assert!((TreatmentType::Stabilization.cost_multiplier() - 0.8).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Industrial waste generation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_industrial_waste_generation_per_level() {
        assert!((industrial_waste_generation(1) - 0.5).abs() < f32::EPSILON);
        assert!((industrial_waste_generation(2) - 1.0).abs() < f32::EPSILON);
        assert!((industrial_waste_generation(3) - 2.0).abs() < f32::EPSILON);
        assert!((industrial_waste_generation(4) - 3.5).abs() < f32::EPSILON);
        assert!((industrial_waste_generation(5) - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_industrial_waste_generation_level_zero_clamps() {
        // Level 0 should clamp to level 1 rate via saturating_sub
        assert!((industrial_waste_generation(0) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_industrial_waste_generation_high_level_clamps() {
        // Level 10 should clamp to level 5 rate
        assert!((industrial_waste_generation(10) - 5.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Medical waste identification tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_medical_waste_generators() {
        assert!(is_medical_waste_generator(ServiceType::Hospital));
        assert!(is_medical_waste_generator(ServiceType::MedicalClinic));
        assert!(is_medical_waste_generator(ServiceType::MedicalCenter));
    }

    #[test]
    fn test_non_medical_not_waste_generators() {
        assert!(!is_medical_waste_generator(ServiceType::FireStation));
        assert!(!is_medical_waste_generator(ServiceType::PoliceStation));
        assert!(!is_medical_waste_generator(ServiceType::Landfill));
        assert!(!is_medical_waste_generator(ServiceType::Incinerator));
        assert!(!is_medical_waste_generator(ServiceType::University));
    }

    // -------------------------------------------------------------------------
    // HazardousWasteState default tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_default() {
        let state = HazardousWasteState::default();
        assert_eq!(state.total_generation, 0.0);
        assert_eq!(state.treatment_capacity, 0.0);
        assert_eq!(state.overflow, 0.0);
        assert_eq!(state.illegal_dump_events, 0);
        assert_eq!(state.contamination_level, 0.0);
        assert_eq!(state.federal_fines, 0.0);
        assert_eq!(state.facility_count, 0);
        assert_eq!(state.daily_operating_cost, 0.0);
        assert_eq!(state.chemical_treated, 0.0);
        assert_eq!(state.thermal_treated, 0.0);
        assert_eq!(state.biological_treated, 0.0);
        assert_eq!(state.stabilization_treated, 0.0);
    }

    // -------------------------------------------------------------------------
    // Constants tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_facility_constants() {
        assert!((FACILITY_CAPACITY_TONS_PER_DAY - 20.0).abs() < f32::EPSILON);
        assert!((FACILITY_BUILD_COST - 3_000_000.0).abs() < f64::EPSILON);
        assert!((FACILITY_OPERATING_COST_PER_DAY - 5_000.0).abs() < f64::EPSILON);
        assert!((FEDERAL_FINE_PER_EVENT - 50_000.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Overflow and fines logic tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_overflow_when_generation_exceeds_capacity() {
        let generation = 30.0_f32;
        let capacity = 20.0_f32;
        let overflow = (generation - capacity).max(0.0);
        assert!((overflow - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_no_overflow_when_capacity_sufficient() {
        let generation = 15.0_f32;
        let capacity = 20.0_f32;
        let overflow = (generation - capacity).max(0.0);
        assert!(overflow.abs() < f32::EPSILON);
    }

    #[test]
    fn test_federal_fines_accumulate() {
        let mut fines = 0.0_f64;
        // Three illegal dump events
        fines += FEDERAL_FINE_PER_EVENT;
        fines += FEDERAL_FINE_PER_EVENT;
        fines += FEDERAL_FINE_PER_EVENT;
        assert!((fines - 150_000.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Contamination logic tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_contamination_increases_with_overflow() {
        let overflow = 5.0_f32;
        let contamination = overflow * CONTAMINATION_PER_OVERFLOW_TON;
        assert!((contamination - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_contamination_decay() {
        let mut contamination = 100.0_f32;
        contamination *= 1.0 - CONTAMINATION_DECAY_RATE;
        assert!((contamination - 99.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_contamination_decay_clamps_to_zero() {
        let mut contamination = 0.005_f32;
        contamination *= 1.0 - CONTAMINATION_DECAY_RATE;
        if contamination < 0.01 {
            contamination = 0.0;
        }
        assert!(contamination.abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Treatment capacity scaling tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_capacity_scales_with_facility_count() {
        assert!((0_u32 as f32 * FACILITY_CAPACITY_TONS_PER_DAY).abs() < f32::EPSILON);
        assert!((1_u32 as f32 * FACILITY_CAPACITY_TONS_PER_DAY - 20.0).abs() < f32::EPSILON);
        assert!((3_u32 as f32 * FACILITY_CAPACITY_TONS_PER_DAY - 60.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_operating_cost_scales_with_facility_count() {
        let cost_2 = 2_u32 as f64 * FACILITY_OPERATING_COST_PER_DAY;
        assert!((cost_2 - 10_000.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests (simulating the update logic)
    // -------------------------------------------------------------------------

    #[test]
    fn test_full_cycle_no_overflow() {
        // 2 industrial buildings (level 1 + level 2) = 0.5 + 1.0 = 1.5 tons/day
        // 1 facility = 20 tons/day capacity
        // No overflow expected
        let generation = industrial_waste_generation(1) + industrial_waste_generation(2);
        let capacity = 1.0 * FACILITY_CAPACITY_TONS_PER_DAY;
        let overflow = (generation - capacity).max(0.0);

        assert!((generation - 1.5).abs() < f32::EPSILON);
        assert!((capacity - 20.0).abs() < f32::EPSILON);
        assert!(overflow.abs() < f32::EPSILON);
    }

    #[test]
    fn test_full_cycle_with_overflow() {
        // 5 industrial buildings at level 5 = 5 * 5.0 = 25 tons/day
        // 1 medical facility = 0.8 tons/day
        // Total = 25.8 tons/day
        // 1 facility = 20 tons/day capacity
        // Overflow = 5.8 tons
        let industrial_gen = 5.0 * industrial_waste_generation(5);
        let medical_gen = MEDICAL_WASTE_RATE;
        let total = industrial_gen + medical_gen;
        let capacity = 1.0 * FACILITY_CAPACITY_TONS_PER_DAY;
        let overflow = (total - capacity).max(0.0);

        assert!((total - 25.8).abs() < 0.01);
        assert!((overflow - 5.8).abs() < 0.01);
    }

    #[test]
    fn test_full_cycle_contamination_accumulation() {
        // Simulate 3 ticks of overflow
        let mut state = HazardousWasteState::default();
        let overflow_per_tick = 5.0_f32;

        for _ in 0..3 {
            let contamination_increase = overflow_per_tick * CONTAMINATION_PER_OVERFLOW_TON;
            state.contamination_level += contamination_increase;
            state.illegal_dump_events += 1;
            state.federal_fines += FEDERAL_FINE_PER_EVENT;

            // Apply decay
            state.contamination_level *= 1.0 - CONTAMINATION_DECAY_RATE;
        }

        assert_eq!(state.illegal_dump_events, 3);
        assert!((state.federal_fines - 150_000.0).abs() < f64::EPSILON);
        // Contamination should be positive and accumulated
        assert!(state.contamination_level > 0.0);
        // After 3 ticks of 10.0 increase each with 1% decay:
        // tick 1: 10.0 * 0.99 = 9.9
        // tick 2: (9.9 + 10.0) * 0.99 = 19.701
        // tick 3: (19.701 + 10.0) * 0.99 = 29.40399
        assert!((state.contamination_level - 29.404).abs() < 0.01);
    }

    #[test]
    fn test_no_generation_no_effects() {
        // No industrial or medical buildings => zero everything
        let generation = 0.0_f32;
        let capacity = 0.0_f32;
        let overflow = (generation - capacity).max(0.0);

        assert!(generation.abs() < f32::EPSILON);
        assert!(overflow.abs() < f32::EPSILON);
    }

    #[test]
    fn test_groundwater_contamination_radius() {
        // Verify contamination radius constant
        assert_eq!(CONTAMINATION_RADIUS, 4);

        // Within radius: affected
        let dist_inside = 3_i32;
        assert!(dist_inside <= CONTAMINATION_RADIUS);

        // Outside radius: not affected
        let dist_outside = 5_i32;
        assert!(dist_outside > CONTAMINATION_RADIUS);
    }

    #[test]
    fn test_groundwater_quality_reduction_with_overflow() {
        // Simulate quality reduction logic
        let overflow = 8.0_f32;
        let quality_reduction = (overflow * 0.5).min(10.0) as u8;
        assert_eq!(quality_reduction, 4);

        // Large overflow is capped at 10
        let large_overflow = 100.0_f32;
        let capped_reduction = (large_overflow * 0.5).min(10.0) as u8;
        assert_eq!(capped_reduction, 10);
    }

    #[test]
    fn test_count_hazardous_facilities_helper() {
        // Test the helper function with mock service buildings
        let incinerator = ServiceBuilding {
            service_type: ServiceType::Incinerator,
            grid_x: 10,
            grid_y: 10,
            radius: 480.0,
        };
        let landfill = ServiceBuilding {
            service_type: ServiceType::Landfill,
            grid_x: 20,
            grid_y: 20,
            radius: 320.0,
        };
        let incinerator2 = ServiceBuilding {
            service_type: ServiceType::Incinerator,
            grid_x: 30,
            grid_y: 30,
            radius: 480.0,
        };

        let services: Vec<&ServiceBuilding> = vec![&incinerator, &landfill, &incinerator2];
        let count = count_hazardous_facilities(&services);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_count_hazardous_facilities_empty() {
        let services: Vec<&ServiceBuilding> = vec![];
        assert_eq!(count_hazardous_facilities(&services), 0);
    }

    #[test]
    fn test_multiple_treatment_types_efficiency() {
        // Verify that different treatment types have distinct efficiencies
        let types = [
            TreatmentType::Chemical,
            TreatmentType::Thermal,
            TreatmentType::Biological,
            TreatmentType::Stabilization,
        ];
        // All efficiencies should be positive
        for t in &types {
            assert!(t.efficiency() > 0.0);
        }
        // Thermal should be the most efficient
        assert!(TreatmentType::Thermal.efficiency() > TreatmentType::Chemical.efficiency());
        // Biological should be least efficient
        assert!(TreatmentType::Biological.efficiency() < TreatmentType::Chemical.efficiency());
    }
}
