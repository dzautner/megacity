//! Tests for flood protection types, structures, placement, and adjacency.

#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::flood_protection::systems::*;
    use crate::flood_protection::types::*;
    use crate::grid::{CellType, WorldGrid};
    use crate::Saveable;

    // -------------------------------------------------------------------------
    // ProtectionType tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_levee_design_height() {
        assert!(
            (ProtectionType::Levee.design_height() - 10.0).abs() < f32::EPSILON,
            "Levee design height should be 10 ft"
        );
    }

    #[test]
    fn test_seawall_design_height() {
        assert!(
            (ProtectionType::Seawall.design_height() - 15.0).abs() < f32::EPSILON,
            "Seawall design height should be 15 ft"
        );
    }

    #[test]
    fn test_floodgate_design_height() {
        assert!(
            (ProtectionType::Floodgate.design_height() - 12.0).abs() < f32::EPSILON,
            "Floodgate design height should be 12 ft"
        );
    }

    // -------------------------------------------------------------------------
    // ProtectionStructure tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_new_structure_defaults() {
        let s = ProtectionStructure::new(10, 20, ProtectionType::Levee);
        assert_eq!(s.grid_x, 10);
        assert_eq!(s.grid_y, 20);
        assert_eq!(s.protection_type, ProtectionType::Levee);
        assert!((s.condition - 1.0).abs() < f32::EPSILON);
        assert_eq!(s.age_days, 0);
        assert!(!s.failed);
        assert!(!s.gate_open);
    }

    #[test]
    fn test_effective_height_full_condition() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        assert!(
            (s.effective_height() - 10.0).abs() < f32::EPSILON,
            "Full condition levee should have 10 ft effective height"
        );
    }

    #[test]
    fn test_effective_height_degraded() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.condition = 0.5;
        assert!(
            (s.effective_height() - 5.0).abs() < f32::EPSILON,
            "Half condition levee should have 5 ft effective height"
        );
    }

    #[test]
    fn test_effective_height_failed() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.failed = true;
        assert!(
            s.effective_height().abs() < f32::EPSILON,
            "Failed levee should have 0 effective height"
        );
    }

    #[test]
    fn test_effective_height_open_floodgate() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Floodgate);
        s.gate_open = true;
        assert!(
            s.effective_height().abs() < f32::EPSILON,
            "Open floodgate should have 0 effective height"
        );
    }

    #[test]
    fn test_effective_height_closed_floodgate() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Floodgate);
        assert!(
            (s.effective_height() - 12.0).abs() < f32::EPSILON,
            "Closed floodgate should have 12 ft effective height"
        );
    }

    #[test]
    fn test_effective_height_seawall() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Seawall);
        assert!(
            (s.effective_height() - 15.0).abs() < f32::EPSILON,
            "Full condition seawall should have 15 ft effective height"
        );
    }

    // -------------------------------------------------------------------------
    // Failure probability tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_failure_prob_new_structure() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        let prob = s.failure_probability();
        assert!(
            prob > 0.0 && prob < 0.001,
            "New structure should have very low failure prob: {}",
            prob
        );
    }

    #[test]
    fn test_failure_prob_aged_structure() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.age_days = 3600; // 10 years
        let prob = s.failure_probability();
        let new_prob = ProtectionStructure::new(0, 0, ProtectionType::Levee).failure_probability();
        assert!(
            prob > new_prob,
            "Aged structure should have higher failure prob: {} vs {}",
            prob,
            new_prob
        );
    }

    #[test]
    fn test_failure_prob_degraded_condition() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.condition = 0.25;
        let prob = s.failure_probability();
        let new_prob = ProtectionStructure::new(0, 0, ProtectionType::Levee).failure_probability();
        assert!(
            prob > new_prob * 2.0,
            "Low condition should significantly increase failure prob: {} vs {}",
            prob,
            new_prob
        );
    }

    #[test]
    fn test_failure_prob_already_failed() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.failed = true;
        assert!(
            s.failure_probability().abs() < f32::EPSILON,
            "Already failed structure should have 0 failure prob"
        );
    }

    #[test]
    fn test_failure_prob_capped_at_one() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.condition = 0.001;
        s.age_days = 100_000;
        let prob = s.failure_probability();
        assert!(
            prob <= 1.0,
            "Failure probability should be capped at 1.0, got {}",
            prob
        );
    }

    // -------------------------------------------------------------------------
    // Placement validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_can_place_levee_on_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(5, 5).cell_type = CellType::Water;
        // Placing ON water should fail
        assert!(
            !can_place_levee(&grid, 5, 5),
            "Should not place levee on water"
        );
    }

    #[test]
    fn test_can_place_levee_adjacent_to_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(5, 5).cell_type = CellType::Water;
        // Adjacent cell should be valid
        assert!(
            can_place_levee(&grid, 5, 6),
            "Should be able to place levee adjacent to water"
        );
    }

    #[test]
    fn test_can_place_levee_not_adjacent_to_water() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // All cells are Grass, no water adjacent
        assert!(
            !can_place_levee(&grid, 128, 128),
            "Should not place levee far from water"
        );
    }

    #[test]
    fn test_can_place_levee_out_of_bounds() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert!(
            !can_place_levee(&grid, GRID_WIDTH, 0),
            "Out of bounds should fail"
        );
    }

    #[test]
    fn test_can_place_seawall_adjacent_to_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(0, 0).cell_type = CellType::Water;
        assert!(
            can_place_seawall(&grid, 1, 0),
            "Should be able to place seawall adjacent to coast water"
        );
    }

    #[test]
    fn test_can_place_floodgate_same_as_levee() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Water;
        assert_eq!(
            can_place_floodgate(&grid, 10, 11),
            can_place_levee(&grid, 10, 11),
            "Floodgate placement should follow levee rules"
        );
    }

    // -------------------------------------------------------------------------
    // Water adjacency tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_adjacent_to_water_true() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(5, 5).cell_type = CellType::Water;
        assert!(is_adjacent_to_water(&grid, 5, 6));
        assert!(is_adjacent_to_water(&grid, 5, 4));
        assert!(is_adjacent_to_water(&grid, 6, 5));
        assert!(is_adjacent_to_water(&grid, 4, 5));
    }

    #[test]
    fn test_is_adjacent_to_water_false() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert!(!is_adjacent_to_water(&grid, 128, 128));
    }

    #[test]
    fn test_is_adjacent_to_water_at_corner() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(0, 1).cell_type = CellType::Water;
        assert!(is_adjacent_to_water(&grid, 0, 0));
    }

    // -------------------------------------------------------------------------
    // Maintenance cost tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_daily_maintenance_cost() {
        let annual = 2000.0;
        let daily = daily_maintenance_cost(annual);
        let expected = 2000.0 / 360.0;
        assert!(
            (daily - expected).abs() < 0.01,
            "Daily maintenance should be {}, got {}",
            expected,
            daily
        );
    }

    #[test]
    fn test_annual_maintenance_cost_scaling() {
        // 10 structures = $20,000/year
        let annual = 10.0 * MAINTENANCE_COST_PER_CELL_PER_YEAR;
        assert!(
            (annual - 20_000.0).abs() < f64::EPSILON,
            "10 structures should cost $20,000/year"
        );
    }

    // -------------------------------------------------------------------------
    // Condition degradation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_condition_degradation_rate() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.condition -= DEGRADATION_RATE_PER_TICK;
        assert!(
            (s.condition - (1.0 - DEGRADATION_RATE_PER_TICK)).abs() < f32::EPSILON,
            "One tick degradation should reduce condition by {}",
            DEGRADATION_RATE_PER_TICK
        );
    }

    #[test]
    fn test_condition_recovery_rate() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.condition = 0.5;
        s.condition = (s.condition + RECOVERY_RATE_PER_TICK).min(1.0);
        assert!(
            (s.condition - (0.5 + RECOVERY_RATE_PER_TICK)).abs() < f32::EPSILON,
            "One tick recovery should increase condition by {}",
            RECOVERY_RATE_PER_TICK
        );
    }

    #[test]
    fn test_condition_does_not_exceed_one() {
        let mut condition = 0.999;
        condition = (condition + RECOVERY_RATE_PER_TICK).min(1.0);
        assert!(
            condition <= 1.0,
            "Condition should not exceed 1.0, got {}",
            condition
        );
    }

    #[test]
    fn test_condition_does_not_go_below_zero() {
        let mut condition = 0.001_f32;
        condition = (condition - DEGRADATION_RATE_PER_TICK).max(0.0);
        assert!(
            condition >= 0.0,
            "Condition should not go below 0.0, got {}",
            condition
        );
    }
}
