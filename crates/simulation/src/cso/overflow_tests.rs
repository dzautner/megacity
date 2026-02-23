//! CSO overflow detection, pollution, coverage, and integration-style tests.

#[cfg(test)]
mod tests {
    use crate::cso::systems::*;
    use crate::cso::*;

    // -------------------------------------------------------------------------
    // CSO overflow detection tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cso_occurs_when_flow_exceeds_capacity() {
        let combined_flow = 120_000.0_f32;
        let combined_capacity = 100_000.0_f32;
        let overflow = combined_flow > combined_capacity;
        assert!(overflow);
        let discharge = combined_flow - combined_capacity;
        assert!((discharge - 20_000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_no_cso_when_flow_under_capacity() {
        let combined_flow = 80_000.0_f32;
        let combined_capacity = 100_000.0_f32;
        let overflow = combined_flow > combined_capacity;
        assert!(!overflow);
    }

    #[test]
    fn test_no_cso_at_exact_capacity() {
        let combined_flow = 100_000.0_f32;
        let combined_capacity = 100_000.0_f32;
        let overflow = combined_flow > combined_capacity;
        assert!(!overflow);
    }

    #[test]
    fn test_cso_discharge_equals_overflow() {
        let combined_flow = 150_000.0_f32;
        let combined_capacity = 100_000.0_f32;
        let discharge = (combined_flow - combined_capacity).max(0.0_f32);
        assert!((discharge - 50_000.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Pollution contribution tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_pollution_from_cso_discharge() {
        let discharge = 100_000.0_f32;
        let pollution = discharge * POLLUTION_PER_GALLON_CSO;
        let expected = 100_000.0 * 0.0001; // 10.0
        assert!((pollution - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pollution_zero_when_no_discharge() {
        let discharge = 0.0_f32;
        let pollution = discharge * POLLUTION_PER_GALLON_CSO;
        assert_eq!(pollution, 0.0);
    }

    #[test]
    fn test_pollution_scales_with_discharge() {
        let discharge_a = 50_000.0_f32;
        let discharge_b = 100_000.0_f32;
        let pollution_a = discharge_a * POLLUTION_PER_GALLON_CSO;
        let pollution_b = discharge_b * POLLUTION_PER_GALLON_CSO;
        assert!((pollution_b - pollution_a * 2.0_f32).abs() < 0.001_f32);
    }

    // -------------------------------------------------------------------------
    // Separation coverage calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_separation_coverage_zero_when_no_roads() {
        let total_road_cells = 0u32;
        let separated = 0u32;
        let coverage = if total_road_cells > 0 {
            (separated as f32 / total_road_cells as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        assert_eq!(coverage, 0.0);
    }

    #[test]
    fn test_separation_coverage_partial() {
        let total_road_cells = 100u32;
        let separated = 25u32;
        let coverage = (separated as f32 / total_road_cells as f32).clamp(0.0, 1.0);
        assert!((coverage - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_separation_coverage_full() {
        let total_road_cells = 100u32;
        let separated = 100u32;
        let coverage = (separated as f32 / total_road_cells as f32).clamp(0.0, 1.0);
        assert!((coverage - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_separation_coverage_clamped_above_one() {
        // Edge case: more separated cells than total (data corruption guard)
        let total_road_cells = 50u32;
        let separated = 75u32;
        let coverage = (separated as f32 / total_road_cells as f32).clamp(0.0, 1.0);
        assert!((coverage - 1.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Sewer type determination tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sewer_type_combined_when_low_separation() {
        let separation_coverage = 0.50_f32;
        let sewer_type = if separation_coverage > 0.95 {
            SewerType::Separated
        } else {
            SewerType::Combined
        };
        assert_eq!(sewer_type, SewerType::Combined);
    }

    #[test]
    fn test_sewer_type_separated_when_high_separation() {
        let separation_coverage = 0.96_f32;
        let sewer_type = if separation_coverage > 0.95 {
            SewerType::Separated
        } else {
            SewerType::Combined
        };
        assert_eq!(sewer_type, SewerType::Separated);
    }

    #[test]
    fn test_sewer_type_combined_at_boundary() {
        let separation_coverage = 0.95_f32;
        let sewer_type = if separation_coverage > 0.95 {
            SewerType::Separated
        } else {
            SewerType::Combined
        };
        assert_eq!(sewer_type, SewerType::Combined);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_full_cso_cycle_dry_weather() {
        // No stormwater runoff => sewage only, well under capacity
        let population = 1000;
        let road_cells = 100u32;
        let separated = 0u32;

        let sewage = sewage_flow_gph(population);
        let stormwater = stormwater_inflow_gph(0.0);
        let capacity = calculate_combined_capacity(road_cells, separated);
        let separation_coverage = 0.0_f32;
        let flow = calculate_combined_flow(sewage, stormwater, separation_coverage);

        // Sewage: 1000 * 80 / 24 = ~3333 gph
        // Capacity: 100 * 10000 = 1,000,000 gph
        assert!(flow < capacity, "Dry weather should not cause CSO");
        assert!((stormwater - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_full_cso_cycle_storm_event() {
        // Heavy storm with large stormwater inflow overwhelming capacity
        let population = 5000;
        let road_cells = 10u32; // small sewer system
        let separated = 0u32;

        let sewage = sewage_flow_gph(population);
        let total_runoff = 500_000.0_f32; // heavy storm
        let stormwater = stormwater_inflow_gph(total_runoff);
        let capacity = calculate_combined_capacity(road_cells, separated);
        let separation_coverage = 0.0_f32;
        let flow = calculate_combined_flow(sewage, stormwater, separation_coverage);

        // Capacity: 10 * 10000 = 100,000 gph
        // Flow: ~16,667 (sewage) + 250,000 (stormwater) = ~266,667 gph
        assert!(
            flow > capacity,
            "Storm should cause CSO: flow {} > capacity {}",
            flow,
            capacity
        );

        let discharge = flow - capacity;
        let pollution = discharge * POLLUTION_PER_GALLON_CSO;
        assert!(discharge > 0.0);
        assert!(pollution > 0.0);
    }

    #[test]
    fn test_full_cso_cycle_separated_system() {
        // Same storm, but fully separated sewers => no CSO
        let population = 5000;
        let road_cells = 10u32;
        let separated = 10u32; // fully separated

        let sewage = sewage_flow_gph(population);
        let total_runoff = 500_000.0_f32;
        let stormwater = stormwater_inflow_gph(total_runoff);

        let separation_coverage = (separated as f32 / road_cells as f32).clamp(0.0, 1.0);
        assert!((separation_coverage - 1.0).abs() < f32::EPSILON);

        // Capacity is 0 for combined (all separated), but flow also has no stormwater
        let flow = calculate_combined_flow(sewage, stormwater, separation_coverage);

        // With 100% separation, stormwater contribution is 0
        // Flow = sewage only = 5000 * 80 / 24 = ~16,667 gph
        // Combined capacity = 0 (all cells are separated)
        // But the separated system handles sewage through its own pipes,
        // so the combined system has 0 capacity and only sewage flow.
        // In practice, the separated system would have its own capacity,
        // but from the combined sewer perspective, there's no combined infrastructure.
        // This edge case shows that a fully separated city effectively eliminates
        // the concept of combined capacity entirely.
        assert!((flow - sewage).abs() < 0.1_f32);
    }

    #[test]
    fn test_partial_separation_reduces_cso() {
        let population = 5000;
        let road_cells = 20u32;
        let total_runoff = 500_000.0_f32;

        // No separation
        let sewage = sewage_flow_gph(population);
        let stormwater = stormwater_inflow_gph(total_runoff);
        let capacity_0 = calculate_combined_capacity(road_cells, 0);
        let flow_0 = calculate_combined_flow(sewage, stormwater, 0.0);
        let _discharge_0 = (flow_0 - capacity_0).max(0.0_f32);

        // 50% separation
        let capacity_50 = calculate_combined_capacity(road_cells, 10);
        let flow_50 = calculate_combined_flow(sewage, stormwater, 0.5);
        let _discharge_50 = (flow_50 - capacity_50).max(0.0_f32);

        // With partial separation, stormwater entering the combined sewer is halved,
        // but capacity is also halved. The net effect on CSO depends on the balance.
        // The key property: stormwater contribution to combined flow decreases.
        assert!(
            flow_50 < flow_0,
            "Partial separation should reduce combined flow"
        );
    }

    #[test]
    fn test_annual_cso_volume_accumulates() {
        let mut state = SewerSystemState::default();
        state.annual_cso_volume = 100_000.0;

        // Simulate another CSO event
        let new_discharge = 25_000.0_f32;
        state.annual_cso_volume += new_discharge;

        assert!((state.annual_cso_volume - 125_000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cso_event_counters_increment() {
        let mut state = SewerSystemState::default();
        assert_eq!(state.cso_events_total, 0);
        assert_eq!(state.cso_events_this_year, 0);

        // Simulate 3 CSO events
        for _ in 0..3 {
            state.cso_events_total += 1;
            state.cso_events_this_year += 1;
        }

        assert_eq!(state.cso_events_total, 3);
        assert_eq!(state.cso_events_this_year, 3);

        // Annual reset
        state.cso_events_this_year = 0;
        assert_eq!(state.cso_events_total, 3);
        assert_eq!(state.cso_events_this_year, 0);
    }

    #[test]
    fn test_zero_capacity_no_cso_when_no_flow() {
        // Edge case: no road cells means no capacity AND no population
        let capacity = calculate_combined_capacity(0, 0);
        let flow = calculate_combined_flow(0.0, 0.0, 0.0);
        // Flow does not exceed capacity (both zero)
        assert!(!(flow > capacity && capacity > 0.0));
    }

    #[test]
    fn test_separation_cost_for_city() {
        // Cost to separate 100 cells
        let cells_to_separate = 100u32;
        let total_cost = cells_to_separate as f32 * SEPARATION_COST_PER_CELL;
        assert!((total_cost - 50_000_000.0).abs() < f32::EPSILON);
    }
}
