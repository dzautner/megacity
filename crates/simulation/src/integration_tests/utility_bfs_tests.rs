//! Integration tests for utility BFS coverage propagation.
//!
//! Verifies that power and water coverage spreads correctly through the road
//! network via BFS, respects range limits, does not cross gaps, and handles
//! grid boundary conditions.

use crate::grid::{RoadType, ZoneType};
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;
use crate::weather::Weather;

// ====================================================================
// TEST-039: Utility BFS Coverage
// ====================================================================

/// Power plant placed on a road should propagate power along connected road
/// cells and to adjacent grass cells (buildings). Cells within range should
/// gain `has_power = true`.
#[test]
fn test_power_coverage_from_power_plant_along_roads() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_utility(50, 50, UtilityType::PowerPlant)
        .with_building(55, 49, ZoneType::ResidentialLow, 1)
        .with_building(60, 49, ZoneType::CommercialLow, 1);

    // Tick enough for propagation to run
    city.tick(5);

    // Road cells along the route should have power
    assert!(
        city.cell(55, 50).has_power,
        "road cell at (55,50) should have power"
    );
    assert!(
        city.cell(60, 50).has_power,
        "road cell at (60,50) should have power"
    );

    // Adjacent building cells (grass) should receive power from the road
    assert!(
        city.cell(55, 49).has_power,
        "building cell at (55,49) should have power from adjacent road"
    );
    assert!(
        city.cell(60, 49).has_power,
        "building cell at (60,49) should have power from adjacent road"
    );
}

/// Water tower placed on a road should propagate water along connected road
/// cells and to adjacent grass cells. Cells within range should gain
/// `has_water = true`.
#[test]
fn test_water_coverage_from_water_tower_along_roads() {
    let mut city = TestCity::new()
        .with_road(80, 80, 100, 80, RoadType::Local)
        .with_utility(80, 80, UtilityType::WaterTower)
        .with_building(85, 79, ZoneType::ResidentialLow, 1)
        .with_building(90, 79, ZoneType::CommercialLow, 1);

    city.tick(5);

    // Road cells should have water
    assert!(
        city.cell(85, 80).has_water,
        "road cell at (85,80) should have water"
    );
    assert!(
        city.cell(90, 80).has_water,
        "road cell at (90,80) should have water"
    );

    // Adjacent building cells should receive water
    assert!(
        city.cell(85, 79).has_water,
        "building cell at (85,79) should have water from adjacent road"
    );
    assert!(
        city.cell(90, 79).has_water,
        "building cell at (90,79) should have water from adjacent road"
    );

    // Power should NOT be set by a water tower
    assert!(
        !city.cell(85, 80).has_power,
        "water tower should not provide power"
    );
    assert!(
        !city.cell(85, 79).has_power,
        "water tower should not provide power to buildings"
    );
}

/// An area of road that is completely disconnected from the utility source
/// should receive no coverage, even if the utility has enough range.
#[test]
fn test_disconnected_area_has_no_coverage() {
    let mut city = TestCity::new()
        // First road segment with power plant
        .with_road(30, 30, 40, 30, RoadType::Local)
        .with_utility(30, 30, UtilityType::PowerPlant)
        // Second road segment, completely disconnected (gap at x=41..49)
        .with_road(50, 30, 60, 30, RoadType::Local)
        .with_building(55, 29, ZoneType::ResidentialLow, 1);

    city.tick(5);

    // Connected segment should have power
    assert!(
        city.cell(35, 30).has_power,
        "connected road cell at (35,30) should have power"
    );

    // Disconnected segment should NOT have power
    assert!(
        !city.cell(50, 30).has_power,
        "disconnected road cell at (50,30) should NOT have power"
    );
    assert!(
        !city.cell(55, 30).has_power,
        "disconnected road cell at (55,30) should NOT have power"
    );
    assert!(
        !city.cell(55, 29).has_power,
        "building on disconnected segment at (55,29) should NOT have power"
    );
}

/// Coverage should not bridge across a gap in the road network. If a road
/// is broken in the middle, cells beyond the gap should have no coverage.
#[test]
fn test_coverage_does_not_cross_gaps() {
    let mut city = TestCity::new()
        // Road from 40 to 45
        .with_road(40, 60, 45, 60, RoadType::Local)
        // Gap at x=46,47,48 (no road)
        // Road from 49 to 55
        .with_road(49, 60, 55, 60, RoadType::Local)
        .with_utility(40, 60, UtilityType::WaterTower);

    city.tick(5);

    // Before the gap: should have water
    assert!(
        city.cell(43, 60).has_water,
        "road cell at (43,60) before gap should have water"
    );
    assert!(
        city.cell(45, 60).has_water,
        "road cell at (45,60) at edge of gap should have water"
    );

    // After the gap: should NOT have water
    assert!(
        !city.cell(49, 60).has_water,
        "road cell at (49,60) after gap should NOT have water"
    );
    assert!(
        !city.cell(52, 60).has_water,
        "road cell at (52,60) after gap should NOT have water"
    );
}

/// Utility coverage BFS must respect grid boundaries and not panic or wrap
/// around when the source is placed near the edge of the grid.
#[test]
fn test_coverage_respects_grid_boundaries() {
    // Place utility near grid edge (grid is 256x256, indices 0..255)
    let mut city = TestCity::new()
        .with_road(250, 250, 254, 250, RoadType::Local)
        .with_utility(252, 250, UtilityType::PowerPlant);

    // This should not panic even though BFS tries to explore beyond boundaries
    city.tick(5);

    // Cells within the road should have power
    assert!(
        city.cell(253, 250).has_power,
        "road cell near edge at (253,250) should have power"
    );
    assert!(
        city.cell(254, 250).has_power,
        "road cell at grid edge (254,250) should have power"
    );

    // Also test near origin (0,0)
    let mut city2 = TestCity::new()
        .with_road(0, 1, 5, 1, RoadType::Local)
        .with_utility(0, 1, UtilityType::WaterTower);

    city2.tick(5);

    assert!(
        city2.cell(3, 1).has_water,
        "road cell near origin at (3,1) should have water"
    );
    // Adjacent grass cell at row 0 should get coverage
    assert!(
        city2.cell(3, 0).has_water,
        "grass cell at boundary row 0 should have water from adjacent road"
    );
}

/// BFS coverage should respect the utility's effective range. Cells beyond
/// the range should not receive coverage even if they are connected by road.
#[test]
fn test_coverage_limited_by_range() {
    // Power plant has range 120; place a very long road and verify far cells
    // are not covered (range is in BFS hops, not Euclidean distance).
    let mut city = TestCity::new()
        // Long horizontal road from x=10 to x=250
        .with_road(10, 100, 250, 100, RoadType::Local)
        .with_utility(10, 100, UtilityType::PowerPlant); // range = 120

    city.tick(5);

    // Cell well within range should have power
    assert!(
        city.cell(50, 100).has_power,
        "road cell at (50,100) within range should have power"
    );

    // Cell beyond range (120 hops from x=10 means x=130 is the boundary)
    // Cell at x=200 is 190 hops away, well beyond range 120
    assert!(
        !city.cell(200, 100).has_power,
        "road cell at (200,100) beyond range should NOT have power"
    );
}

/// Verify that a branching road network propagates coverage correctly
/// through all connected branches.
#[test]
fn test_coverage_propagates_through_branching_roads() {
    let mut city = TestCity::new()
        // Main east-west road
        .with_road(50, 80, 70, 80, RoadType::Local)
        // Branch going north from intersection at (60, 80)
        .with_road(60, 70, 60, 80, RoadType::Local)
        // Branch going south from intersection at (60, 80)
        .with_road(60, 80, 60, 90, RoadType::Local)
        .with_utility(50, 80, UtilityType::PowerPlant)
        .with_building(60, 69, ZoneType::ResidentialLow, 1)
        .with_building(60, 91, ZoneType::ResidentialLow, 1);

    city.tick(5);

    // Main road should have power
    assert!(
        city.cell(65, 80).has_power,
        "main road cell at (65,80) should have power"
    );

    // North branch should have power
    assert!(
        city.cell(60, 75).has_power,
        "north branch road cell at (60,75) should have power"
    );
    assert!(
        city.cell(60, 69).has_power,
        "building on north branch at (60,69) should have power"
    );

    // South branch should have power
    assert!(
        city.cell(60, 85).has_power,
        "south branch road cell at (60,85) should have power"
    );
    assert!(
        city.cell(60, 91).has_power,
        "building on south branch at (60,91) should have power"
    );
}

/// Verify that multiple utility sources of different types provide both
/// power and water coverage independently.
#[test]
fn test_multiple_utility_types_provide_independent_coverage() {
    let mut city = TestCity::new()
        .with_road(40, 40, 60, 40, RoadType::Local)
        .with_utility(40, 40, UtilityType::PowerPlant)
        .with_utility(42, 40, UtilityType::WaterTower)
        .with_building(50, 39, ZoneType::ResidentialLow, 1);

    city.tick(5);

    // Building should have both power and water
    let cell = city.cell(50, 39);
    assert!(
        cell.has_power,
        "building at (50,39) should have power from PowerPlant"
    );
    assert!(
        cell.has_water,
        "building at (50,39) should have water from WaterTower"
    );

    // Road cells should have both
    let road_cell = city.cell(50, 40);
    assert!(
        road_cell.has_power,
        "road cell at (50,40) should have power"
    );
    assert!(
        road_cell.has_water,
        "road cell at (50,40) should have water"
    );
}
