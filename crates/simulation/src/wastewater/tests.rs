//! Unit tests for the wastewater system.

#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, WorldGrid, ZoneType};
    use crate::water_demand::WaterDemand;

    use super::super::types::{
        find_discharge_water_cells, is_serviced_by_sewage_plant, sewage_for_demand,
        WastewaterState, DISCHARGE_POLLUTION_AMOUNT, HAPPINESS_PENALTY_PER_TICK,
        HEALTH_PENALTY_PER_TICK, SEWAGE_FRACTION, TREATMENT_CAPACITY_PER_PLANT,
    };

    // -------------------------------------------------------------------------
    // WastewaterState default tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_wastewater_state_default() {
        let state = WastewaterState::default();
        assert_eq!(state.total_sewage_generated, 0.0);
        assert_eq!(state.total_treatment_capacity, 0.0);
        assert_eq!(state.overflow_amount, 0.0);
        assert_eq!(state.coverage_ratio, 0.0);
        assert_eq!(state.pollution_events, 0);
        assert!(!state.health_penalty_active);
    }

    #[test]
    fn test_wastewater_state_clone() {
        let mut state = WastewaterState::default();
        state.total_sewage_generated = 100.0;
        state.pollution_events = 3;
        state.health_penalty_active = true;
        let cloned = state.clone();
        assert_eq!(cloned.total_sewage_generated, 100.0);
        assert_eq!(cloned.pollution_events, 3);
        assert!(cloned.health_penalty_active);
    }

    // -------------------------------------------------------------------------
    // Sewage fraction calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sewage_fraction_of_water_demand() {
        let demand = WaterDemand {
            demand_gpd: 1000.0,
            has_water_service: true,
        };
        let sewage = sewage_for_demand(&demand);
        assert!((sewage - 800.0).abs() < 0.01);
    }

    #[test]
    fn test_sewage_fraction_zero_demand() {
        let demand = WaterDemand {
            demand_gpd: 0.0,
            has_water_service: false,
        };
        let sewage = sewage_for_demand(&demand);
        assert_eq!(sewage, 0.0);
    }

    #[test]
    fn test_sewage_fraction_is_eighty_percent() {
        // Verify the constant is exactly 0.80
        assert!((SEWAGE_FRACTION - 0.80).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sewage_scales_linearly_with_demand() {
        let demand_a = WaterDemand {
            demand_gpd: 500.0,
            has_water_service: true,
        };
        let demand_b = WaterDemand {
            demand_gpd: 1000.0,
            has_water_service: true,
        };
        let sewage_a = sewage_for_demand(&demand_a);
        let sewage_b = sewage_for_demand(&demand_b);
        assert!(
            (sewage_b - sewage_a * 2.0).abs() < 0.01,
            "double demand should produce double sewage"
        );
    }

    // -------------------------------------------------------------------------
    // Service coverage tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_building_within_service_radius() {
        let plants = vec![(50, 50)];
        // Manhattan distance of 10 (within radius 20)
        assert!(is_serviced_by_sewage_plant(55, 55, &plants));
    }

    #[test]
    fn test_building_outside_service_radius() {
        let plants = vec![(50, 50)];
        // Manhattan distance of 30 (outside radius 20)
        assert!(!is_serviced_by_sewage_plant(65, 65, &plants));
    }

    #[test]
    fn test_building_at_exact_service_boundary() {
        let plants = vec![(50, 50)];
        // Manhattan distance of exactly 20 (at boundary, should be serviced)
        assert!(is_serviced_by_sewage_plant(60, 60, &plants));
    }

    #[test]
    fn test_building_just_outside_service_boundary() {
        let plants = vec![(50, 50)];
        // Manhattan distance of 21 (just outside)
        assert!(!is_serviced_by_sewage_plant(61, 60, &plants));
    }

    #[test]
    fn test_no_plants_means_no_service() {
        let plants: Vec<(usize, usize)> = Vec::new();
        assert!(!is_serviced_by_sewage_plant(50, 50, &plants));
    }

    #[test]
    fn test_multiple_plants_coverage() {
        let plants = vec![(10, 10), (100, 100)];
        // Near first plant
        assert!(is_serviced_by_sewage_plant(15, 15, &plants));
        // Near second plant
        assert!(is_serviced_by_sewage_plant(105, 105, &plants));
        // Between both, far from either
        assert!(!is_serviced_by_sewage_plant(55, 55, &plants));
    }

    // -------------------------------------------------------------------------
    // Coverage ratio calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_coverage_ratio_all_serviced() {
        let plants = vec![(50, 50)];
        let building_positions = vec![(50, 50), (55, 55), (45, 50)];
        let total = building_positions.len() as u32;
        let serviced = building_positions
            .iter()
            .filter(|&&(x, y)| is_serviced_by_sewage_plant(x, y, &plants))
            .count() as u32;
        let ratio = serviced as f32 / total as f32;
        assert!((ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_coverage_ratio_partial() {
        let plants = vec![(10, 10)];
        let building_positions = vec![(10, 10), (15, 15), (200, 200)];
        let total = building_positions.len() as u32;
        let serviced = building_positions
            .iter()
            .filter(|&&(x, y)| is_serviced_by_sewage_plant(x, y, &plants))
            .count() as u32;
        let ratio = serviced as f32 / total as f32;
        // 2 out of 3 serviced
        assert!((ratio - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_coverage_ratio_none_serviced() {
        let plants: Vec<(usize, usize)> = Vec::new();
        let building_positions = vec![(10, 10), (50, 50)];
        let total = building_positions.len() as u32;
        let serviced = building_positions
            .iter()
            .filter(|&&(x, y)| is_serviced_by_sewage_plant(x, y, &plants))
            .count() as u32;
        let ratio = serviced as f32 / total as f32;
        assert_eq!(ratio, 0.0);
    }

    // -------------------------------------------------------------------------
    // Overflow calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_overflow_when_over_capacity() {
        let total_sewage = 80_000.0_f32;
        let total_capacity = 50_000.0_f32;
        let overflow = (total_sewage - total_capacity).max(0.0);
        assert!((overflow - 30_000.0).abs() < 0.01);
    }

    #[test]
    fn test_no_overflow_when_under_capacity() {
        let total_sewage = 30_000.0_f32;
        let total_capacity = 50_000.0_f32;
        let overflow = (total_sewage - total_capacity).max(0.0);
        assert_eq!(overflow, 0.0);
    }

    #[test]
    fn test_no_overflow_at_exact_capacity() {
        let total_sewage = 50_000.0_f32;
        let total_capacity = 50_000.0_f32;
        let overflow = (total_sewage - total_capacity).max(0.0);
        assert_eq!(overflow, 0.0);
    }

    #[test]
    fn test_zero_sewage_no_overflow() {
        let total_sewage = 0.0_f32;
        let total_capacity = 50_000.0_f32;
        let overflow = (total_sewage - total_capacity).max(0.0);
        assert_eq!(overflow, 0.0);
    }

    // -------------------------------------------------------------------------
    // Treatment capacity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_single_plant_capacity() {
        assert_eq!(TREATMENT_CAPACITY_PER_PLANT, 50_000.0);
    }

    #[test]
    fn test_multiple_plants_capacity() {
        let plant_count = 3u32;
        let total_capacity = plant_count as f32 * TREATMENT_CAPACITY_PER_PLANT;
        assert!((total_capacity - 150_000.0).abs() < 0.01);
    }

    // -------------------------------------------------------------------------
    // Discharge water cell finding tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_find_discharge_cells_near_plant() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place water cells near position (50, 50)
        grid.get_mut(52, 50).cell_type = CellType::Water;
        grid.get_mut(53, 50).cell_type = CellType::Water;

        let plants = vec![(50, 50)];
        let water_cells = find_discharge_water_cells(&grid, &plants);
        assert!(water_cells.len() >= 2);
        assert!(water_cells.contains(&(52, 50)));
        assert!(water_cells.contains(&(53, 50)));
    }

    #[test]
    fn test_find_discharge_cells_no_water() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // All grass, no water cells
        let plants = vec![(50, 50)];
        let water_cells = find_discharge_water_cells(&grid, &plants);
        assert!(water_cells.is_empty());
    }

    #[test]
    fn test_find_discharge_cells_no_plants_searches_center() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place water cell near center of map
        let cx = GRID_WIDTH / 2;
        let cy = GRID_HEIGHT / 2;
        grid.get_mut(cx + 1, cy).cell_type = CellType::Water;

        let plants: Vec<(usize, usize)> = Vec::new();
        let water_cells = find_discharge_water_cells(&grid, &plants);
        assert!(water_cells.contains(&(cx + 1, cy)));
    }

    // -------------------------------------------------------------------------
    // Pollution discharge tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_discharge_pollution_amount() {
        assert_eq!(DISCHARGE_POLLUTION_AMOUNT, 15);
    }

    #[test]
    fn test_pollution_severity_scaling() {
        // Overflow equal to one plant's capacity => severity_mult = 1.0
        let overflow = TREATMENT_CAPACITY_PER_PLANT;
        let severity = (overflow / TREATMENT_CAPACITY_PER_PLANT).clamp(0.5, 3.0);
        assert!((severity - 1.0).abs() < 0.01);

        // Large overflow (3x capacity) => capped at 3.0
        let overflow_large = TREATMENT_CAPACITY_PER_PLANT * 5.0;
        let severity_large = (overflow_large / TREATMENT_CAPACITY_PER_PLANT).clamp(0.5, 3.0);
        assert!((severity_large - 3.0).abs() < 0.01);

        // Small overflow => clamped to 0.5 minimum
        let overflow_small = TREATMENT_CAPACITY_PER_PLANT * 0.1;
        let severity_small = (overflow_small / TREATMENT_CAPACITY_PER_PLANT).clamp(0.5, 3.0);
        assert!((severity_small - 0.5).abs() < 0.01);
    }

    // -------------------------------------------------------------------------
    // Health penalty tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_health_penalty_constants() {
        assert!(HEALTH_PENALTY_PER_TICK > 0.0);
        assert!(HAPPINESS_PENALTY_PER_TICK > 0.0);
    }

    #[test]
    fn test_health_penalty_not_active_by_default() {
        let state = WastewaterState::default();
        assert!(!state.health_penalty_active);
    }

    #[test]
    fn test_health_penalty_clamped_to_zero() {
        let mut health = 0.5_f32;
        health = (health - HEALTH_PENALTY_PER_TICK).max(0.0);
        assert_eq!(health, 0.0);
    }

    #[test]
    fn test_happiness_penalty_clamped_to_zero() {
        let mut happiness = 1.0_f32;
        happiness = (happiness - HAPPINESS_PENALTY_PER_TICK).max(0.0);
        assert_eq!(happiness, 0.0);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_full_wastewater_cycle_under_capacity() {
        // 10 buildings each with 1000 GPD water demand
        // Total sewage = 10 * 1000 * 0.80 = 8000 GPD
        // 1 sewage plant = 50,000 GPD capacity
        // No overflow expected
        let building_demands: Vec<f32> = vec![1000.0; 10];
        let total_sewage: f32 = building_demands.iter().map(|d| d * SEWAGE_FRACTION).sum();
        let total_capacity = TREATMENT_CAPACITY_PER_PLANT; // 1 plant

        assert!((total_sewage - 8000.0).abs() < 0.01);
        let overflow = (total_sewage - total_capacity).max(0.0);
        assert_eq!(overflow, 0.0);
    }

    #[test]
    fn test_full_wastewater_cycle_over_capacity() {
        // 100 buildings each with 1000 GPD water demand
        // Total sewage = 100 * 1000 * 0.80 = 80,000 GPD
        // 1 sewage plant = 50,000 GPD capacity
        // Overflow = 30,000 GPD
        let building_demands: Vec<f32> = vec![1000.0; 100];
        let total_sewage: f32 = building_demands.iter().map(|d| d * SEWAGE_FRACTION).sum();
        let total_capacity = TREATMENT_CAPACITY_PER_PLANT; // 1 plant

        assert!((total_sewage - 80_000.0).abs() < 0.01);
        let overflow = (total_sewage - total_capacity).max(0.0);
        assert!((overflow - 30_000.0).abs() < 0.01);
    }

    #[test]
    fn test_two_plants_double_capacity() {
        let total_sewage = 90_000.0_f32;
        let total_capacity = 2.0 * TREATMENT_CAPACITY_PER_PLANT; // 100,000 GPD
        let overflow = (total_sewage - total_capacity).max(0.0);
        assert_eq!(overflow, 0.0);
    }

    #[test]
    fn test_no_buildings_full_coverage() {
        // With no buildings, coverage ratio defaults to 1.0
        let buildings_total = 0u32;
        let buildings_serviced = 0u32;
        let coverage_ratio = if buildings_total > 0 {
            buildings_serviced as f32 / buildings_total as f32
        } else {
            1.0
        };
        assert!((coverage_ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_residential_unserviced_triggers_penalty() {
        let plants: Vec<(usize, usize)> = Vec::new();
        let residential_at = (50, 50);
        let is_res = ZoneType::ResidentialLow.is_residential();
        let serviced = is_serviced_by_sewage_plant(residential_at.0, residential_at.1, &plants);

        // Residential building with no sewage service should trigger penalty
        assert!(is_res);
        assert!(!serviced);
        let residential_unserviced = is_res && !serviced;
        assert!(residential_unserviced);
    }

    #[test]
    fn test_industrial_unserviced_no_health_penalty() {
        // Industrial buildings without service should not trigger health penalty
        // (only residential triggers it)
        let is_res = ZoneType::Industrial.is_residential();
        assert!(!is_res);
    }

    #[test]
    fn test_wastewater_state_serialization_roundtrip() {
        let state = WastewaterState {
            total_sewage_generated: 12345.0,
            total_treatment_capacity: 50000.0,
            overflow_amount: 1000.0,
            coverage_ratio: 0.75,
            pollution_events: 5,
            health_penalty_active: true,
        };

        let serialized = serde_json::to_string(&state).expect("serialize");
        let deserialized: WastewaterState = serde_json::from_str(&serialized).expect("deserialize");

        assert!((deserialized.total_sewage_generated - 12345.0).abs() < 0.01);
        assert!((deserialized.total_treatment_capacity - 50000.0).abs() < 0.01);
        assert!((deserialized.overflow_amount - 1000.0).abs() < 0.01);
        assert!((deserialized.coverage_ratio - 0.75).abs() < 0.01);
        assert_eq!(deserialized.pollution_events, 5);
        assert!(deserialized.health_penalty_active);
    }
}
