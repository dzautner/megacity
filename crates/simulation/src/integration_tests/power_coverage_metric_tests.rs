//! Integration tests for power/water coverage metric (issue #1968).
//!
//! Verifies that `compute_utility_coverage` only counts zoned cells that have
//! a building in the denominator, so empty zoned cells far from roads do not
//! deflate coverage.

use crate::coverage_metrics::compute_utility_coverage;
use crate::grid::ZoneType;
use crate::test_harness::TestCity;

#[test]
fn test_coverage_ignores_zoned_cells_without_buildings() {
    // Set up: road at y=10, zone a wide area from x=5..15, y=9..11,
    // but only place a building on (5,9) which has power.
    // Old behavior: denominator = all zoned grass cells (many),
    //   so coverage would be very low.
    // New behavior: denominator = only zoned cells with buildings (1),
    //   so coverage = 1.0 (100%).

    let city = TestCity::new()
        .with_road(5, 10, 15, 10, crate::grid::RoadType::Local)
        // Zone a 10x3 area around the road
        .with_zone_rect(5, 9, 14, 11, ZoneType::ResidentialLow)
        // Place a single building with power at (5, 9)
        .with_building(5, 9, ZoneType::ResidentialLow, 1);

    // Manually set power on the building cell
    {
        let grid = city.grid();
        // Verify the building cell is set up correctly
        let cell = grid.get(5, 9);
        assert!(cell.building_id.is_some(), "building should exist at (5,9)");
        assert_eq!(cell.zone, ZoneType::ResidentialLow);
    }

    // Set power on the building cell
    let mut city = city;
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(5, 9).has_power = true;
        grid.get_mut(5, 9).has_water = true;
    }

    let grid = city.grid();
    let (power, water) = compute_utility_coverage(grid);

    // With the fix, only the one cell with a building counts,
    // and it has power+water, so coverage should be 1.0.
    assert!(
        (power - 1.0).abs() < f32::EPSILON,
        "power coverage should be 1.0 but was {power}"
    );
    assert!(
        (water - 1.0).abs() < f32::EPSILON,
        "water coverage should be 1.0 but was {water}"
    );
}

#[test]
fn test_coverage_zero_when_no_buildings() {
    // Zone some cells but place no buildings — coverage should be 0.0,
    // not undefined/NaN.
    let city = TestCity::new()
        .with_road(5, 10, 15, 10, crate::grid::RoadType::Local)
        .with_zone_rect(5, 9, 14, 11, ZoneType::CommercialLow);

    let grid = city.grid();
    let (power, water) = compute_utility_coverage(grid);

    assert!(
        power.abs() < f32::EPSILON,
        "power coverage should be 0.0 when no buildings exist, but was {power}"
    );
    assert!(
        water.abs() < f32::EPSILON,
        "water coverage should be 0.0 when no buildings exist, but was {water}"
    );
}

#[test]
fn test_coverage_partial_when_some_buildings_lack_power() {
    // Place two buildings, only one has power. Coverage should be 0.5.
    let mut city = TestCity::new()
        .with_road(10, 10, 20, 10, crate::grid::RoadType::Local)
        .with_zone_rect(10, 9, 12, 9, ZoneType::ResidentialLow)
        .with_building(10, 9, ZoneType::ResidentialLow, 1)
        .with_building(11, 9, ZoneType::ResidentialLow, 1);

    // Give power to only one building
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(10, 9).has_power = true;
        // (11, 9) has NO power
    }

    let grid = city.grid();
    let (power, _water) = compute_utility_coverage(grid);

    assert!(
        (power - 0.5).abs() < f32::EPSILON,
        "power coverage should be 0.5 but was {power}"
    );
}
