use crate::test_harness::TestCity;

// ====================================================================
// Bicycle Lanes (TRAF-013)
// ====================================================================

#[test]
fn test_bicycle_lanes_default_state_has_no_lanes() {
    let city = TestCity::new();
    let bike_state = city.resource::<crate::bicycle_lanes::BicycleLaneState>();
    assert_eq!(
        bike_state.lane_count(),
        0,
        "new city should have no bike lanes"
    );
}

#[test]
fn test_bicycle_lanes_coverage_zero_without_infrastructure() {
    let mut city = TestCity::new();
    city.tick_slow_cycle();

    let coverage = city.resource::<crate::bicycle_lanes::BicycleCoverageGrid>();
    assert_eq!(
        coverage.city_average, 0.0,
        "city without bike infrastructure should have 0 cycling coverage"
    );
    assert!(
        coverage.cycling_mode_share < 0.01,
        "cycling mode share should be ~0 without infrastructure, got {}",
        coverage.cycling_mode_share
    );
}

// Superblock policy tests
// ====================================================================

#[test]
fn test_superblock_state_initialized() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::superblock::SuperblockState>();
}

#[test]
fn test_superblock_add_and_query() {
    use crate::superblock::{Superblock, SuperblockCell, SuperblockState};
    let mut city = TestCity::new();

    // Add a 5x5 superblock
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SuperblockState>();
        let added = state.add_superblock(Superblock::new(
            50,
            50,
            54,
            54,
            "Downtown Block".to_string(),
        ));
        assert!(added, "should successfully add a valid superblock");
    }

    // Verify cell classifications
    {
        let state = city.resource::<SuperblockState>();
        // Interior cell
        assert_eq!(state.get_cell(52, 52), SuperblockCell::Interior);
        // Perimeter cell
        assert_eq!(state.get_cell(50, 50), SuperblockCell::Perimeter);
        // Outside cell
        assert_eq!(state.get_cell(40, 40), SuperblockCell::None);
        // Coverage stats
        assert_eq!(state.total_interior_cells, 9); // 3x3 interior
        assert_eq!(state.total_coverage_cells, 25); // 5x5 total
    }
}

#[test]
fn test_superblock_reject_too_small() {
    use crate::superblock::{Superblock, SuperblockState};
    let mut city = TestCity::new();

    let world = city.world_mut();
    let mut state = world.resource_mut::<SuperblockState>();

    // 2x2 is too small (minimum 3x3)
    let added = state.add_superblock(Superblock::new(10, 10, 11, 11, "Tiny".to_string()));
    assert!(!added, "should reject superblocks smaller than 3x3");
    assert!(state.superblocks.is_empty());
}

#[test]
fn test_superblock_traffic_multiplier_interior() {
    use crate::superblock::{Superblock, SuperblockState, SUPERBLOCK_TRAFFIC_PENALTY};
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<SuperblockState>();
        state.add_superblock(Superblock::new(30, 30, 36, 36, "Test Block".to_string()));
    }

    let state = city.resource::<SuperblockState>();
    // Interior cells get the penalty multiplier
    assert!(
        (state.traffic_multiplier(33, 33) - SUPERBLOCK_TRAFFIC_PENALTY).abs() < f32::EPSILON,
        "interior cells should have traffic penalty"
    );
    // Perimeter cells have normal cost
    assert!(
        (state.traffic_multiplier(30, 30) - 1.0).abs() < f32::EPSILON,
        "perimeter cells should have no traffic penalty"
    );
    // Outside cells have normal cost
    assert!(
        (state.traffic_multiplier(20, 20) - 1.0).abs() < f32::EPSILON,
        "cells outside superblock should have no penalty"
    );
}
