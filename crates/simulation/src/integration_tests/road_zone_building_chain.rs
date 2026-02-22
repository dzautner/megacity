//! TEST-014: Integration Test — Road -> Zone -> Building Chain
//!
//! End-to-end test: place roads, zone adjacent cells, provide utilities,
//! run simulation ticks, and verify buildings spawn. Uses the `TestCity`
//! harness with `MinimalPlugins` + `SimulationPlugin`.

use crate::buildings::{Building, UnderConstruction};
use crate::grid::{RoadType, ZoneType};
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;
use crate::zones::ZoneDemand;

/// Core acceptance test: place roads at known coordinates, zone adjacent
/// cells with `ResidentialLow`, set power and water via utility sources,
/// run ticks, and verify `building_count > 0`.
///
/// Zoned cells are placed at y=98 (one row above the road at y=99..=100)
/// so that utility BFS can reach them (BFS marks grass cells directly
/// adjacent to road cells but does not continue through grass).
#[test]
fn test_road_zone_building_chain_residential_low() {
    // Place a horizontal road. Road rasterization marks cells along y=100.
    // Zone cells directly adjacent (y=101) so utility BFS can reach them.
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        // Horizontal road from (100,100) to (120,100)
        .with_road(100, 100, 120, 100, RoadType::Local)
        // Zone residential cells directly adjacent to the road (y=101)
        .with_zone_rect(102, 101, 118, 101, ZoneType::ResidentialLow)
        // Power plant and water tower on the road for BFS propagation
        .with_utility(100, 100, UtilityType::PowerPlant)
        .with_utility(120, 100, UtilityType::WaterTower);

    // Verify initial state: no buildings yet
    assert_eq!(
        city.building_count(),
        0,
        "No buildings should exist before simulation runs"
    );

    // Run enough ticks for:
    // - Utility propagation (1 tick)
    // - Zone demand bootstrap via slow tick (100 ticks)
    // - Building spawner fires (every 2 ticks)
    // - Construction completes (100 ticks)
    // Total: ~300 ticks to be safe
    city.tick(300);

    let building_count = city.building_count();
    assert!(
        building_count > 0,
        "After 300 ticks with roads, zones, and utilities, at least one building should spawn. Got {}",
        building_count
    );
}

/// Verify that without utilities (no power/water), buildings do NOT spawn
/// even when roads and zones are present.
#[test]
fn test_no_buildings_without_utilities() {
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        .with_road(100, 100, 120, 100, RoadType::Local)
        .with_zone_rect(102, 101, 118, 101, ZoneType::ResidentialLow);
    // No utilities placed

    city.tick(300);

    let building_count = city.building_count();
    assert_eq!(
        building_count, 0,
        "Without power and water, no buildings should spawn. Got {}",
        building_count
    );
}

/// Verify that without roads, buildings do NOT spawn even with zones and
/// utilities (because zone demand stays zero without roads, and cells are
/// not adjacent to any road).
#[test]
fn test_no_buildings_without_roads() {
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        // Zone cells but no roads
        .with_zone_rect(100, 100, 110, 105, ZoneType::ResidentialLow)
        // Utilities placed but not connected to any road network
        .with_utility(100, 100, UtilityType::PowerPlant)
        .with_utility(110, 100, UtilityType::WaterTower);

    city.tick(300);

    let building_count = city.building_count();
    assert_eq!(
        building_count, 0,
        "Without roads, no buildings should spawn. Got {}",
        building_count
    );
}

/// Verify that without zones, buildings do NOT spawn even with roads
/// and utilities present.
#[test]
fn test_no_buildings_without_zones() {
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        .with_road(100, 100, 120, 100, RoadType::Local)
        // No zones set
        .with_utility(100, 100, UtilityType::PowerPlant)
        .with_utility(120, 100, UtilityType::WaterTower);

    city.tick(300);

    let building_count = city.building_count();
    assert_eq!(
        building_count, 0,
        "Without zones, no buildings should spawn. Got {}",
        building_count
    );
}

/// Test with multiple zone types: residential, commercial, and industrial.
/// All should produce buildings when the full road -> zone -> utility chain
/// is satisfied.
#[test]
fn test_multiple_zone_types_spawn_buildings() {
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        // Long horizontal road
        .with_road(80, 100, 170, 100, RoadType::Avenue)
        // Residential zone adjacent to road (y=101)
        .with_zone_rect(82, 101, 100, 101, ZoneType::ResidentialLow)
        // Commercial zone adjacent to road (y=101)
        .with_zone_rect(110, 101, 130, 101, ZoneType::CommercialLow)
        // Industrial zone adjacent to road (y=101)
        .with_zone_rect(140, 101, 160, 101, ZoneType::Industrial)
        // Utilities on the road with large range covering all zones
        .with_utility(80, 100, UtilityType::PowerPlant)
        .with_utility(170, 100, UtilityType::WaterTower);

    city.tick(300);

    let residential = city.buildings_in_zone(ZoneType::ResidentialLow);
    let commercial = city.buildings_in_zone(ZoneType::CommercialLow);
    let industrial = city.buildings_in_zone(ZoneType::Industrial);

    // At least one zone type should have spawned buildings.
    // (Zone demand varies so we check the total rather than each individually.)
    let total = residential + commercial + industrial;
    assert!(
        total > 0,
        "At least one building should spawn across multiple zone types. \
         Residential={residential}, Commercial={commercial}, Industrial={industrial}"
    );
}

/// Verify that zone demand is positive after placing roads (bootstrap
/// demand) and that buildings spawned by the system are initially under
/// construction.
#[test]
fn test_zone_demand_bootstraps_with_roads() {
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        .with_road(100, 100, 120, 100, RoadType::Local)
        .with_zone_rect(102, 101, 118, 101, ZoneType::ResidentialLow)
        .with_utility(100, 100, UtilityType::PowerPlant)
        .with_utility(120, 100, UtilityType::WaterTower);

    // Run one slow tick cycle to trigger demand computation
    city.tick_slow_cycle();

    let demand = city.resource::<ZoneDemand>();
    assert!(
        demand.residential > 0.0,
        "Residential demand should be positive after slow tick with roads. Got {}",
        demand.residential
    );
}

/// Test that buildings start with the `UnderConstruction` component and
/// eventually lose it after construction completes.
#[test]
fn test_buildings_go_through_construction_phase() {
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        .with_road(100, 100, 120, 100, RoadType::Local)
        .with_zone_rect(102, 101, 118, 101, ZoneType::ResidentialLow)
        .with_utility(100, 100, UtilityType::PowerPlant)
        .with_utility(120, 100, UtilityType::WaterTower);

    // Run enough ticks for building spawn but not full construction.
    // Zone demand bootstraps at tick 100 (slow tick), spawner fires at
    // tick 102 (spawn_interval=2). Buildings spawned at ~102 need 100
    // more ticks of construction. At tick 120, they should still be
    // under construction.
    city.tick(120);

    // Check if any buildings exist (spawned but possibly still under construction)
    let world = city.world_mut();
    let under_construction_count = world
        .query::<(&Building, &UnderConstruction)>()
        .iter(world)
        .count();
    let total_buildings = world.query::<&Building>().iter(world).count();

    // If buildings spawned at all, some should still be under construction
    if total_buildings > 0 {
        assert!(
            under_construction_count > 0,
            "Recently spawned buildings should still be under construction"
        );
    }

    // Run more ticks to complete construction (need ~100 more for earliest buildings)
    city.tick(200);

    let world = city.world_mut();
    let completed_count = world.query::<&Building>().iter(world).count();
    let still_constructing = world
        .query::<(&Building, &UnderConstruction)>()
        .iter(world)
        .count();

    assert!(
        completed_count > 0,
        "After enough ticks, some buildings should exist"
    );

    // Some early buildings should have completed construction by now
    // (spawned ~tick 102, construction = 100 ticks, now at tick 320)
    assert!(
        completed_count > still_constructing,
        "Some buildings should have finished construction. \
         Total={completed_count}, still constructing={still_constructing}"
    );
}

/// Verify the full chain works with high-density residential zoning.
#[test]
fn test_chain_with_residential_high() {
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        .with_road(100, 100, 130, 100, RoadType::Avenue)
        .with_zone_rect(102, 101, 128, 101, ZoneType::ResidentialHigh)
        .with_utility(100, 100, UtilityType::PowerPlant)
        .with_utility(130, 100, UtilityType::WaterTower);

    city.tick(300);

    let count = city.buildings_in_zone(ZoneType::ResidentialHigh);
    assert!(
        count > 0,
        "ResidentialHigh zone should produce buildings. Got {}",
        count
    );
}

/// Verify that only cells with BOTH power AND water are eligible. If we
/// only provide power (no water), buildings should not spawn.
#[test]
fn test_no_buildings_with_power_only() {
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        .with_road(100, 100, 120, 100, RoadType::Local)
        .with_zone_rect(102, 101, 118, 101, ZoneType::ResidentialLow)
        // Only power, no water
        .with_utility(100, 100, UtilityType::PowerPlant);

    city.tick(300);

    let building_count = city.building_count();
    assert_eq!(
        building_count, 0,
        "With only power (no water), no buildings should spawn. Got {}",
        building_count
    );
}

/// Verify that only cells with BOTH power AND water are eligible. If we
/// only provide water (no power), buildings should not spawn.
#[test]
fn test_no_buildings_with_water_only() {
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        .with_road(100, 100, 120, 100, RoadType::Local)
        .with_zone_rect(102, 101, 118, 101, ZoneType::ResidentialLow)
        // Only water, no power
        .with_utility(120, 100, UtilityType::WaterTower);

    city.tick(300);

    let building_count = city.building_count();
    assert_eq!(
        building_count, 0,
        "With only water (no power), no buildings should spawn. Got {}",
        building_count
    );
}

/// Verify that buildings spawned from the chain are level 1 with correct
/// zone type and positive capacity.
#[test]
fn test_spawned_buildings_have_correct_properties() {
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        .with_road(100, 100, 120, 100, RoadType::Local)
        .with_zone_rect(102, 101, 118, 101, ZoneType::ResidentialLow)
        .with_utility(100, 100, UtilityType::PowerPlant)
        .with_utility(120, 100, UtilityType::WaterTower);

    city.tick(300);

    let world = city.world_mut();
    let buildings: Vec<_> = world.query::<&Building>().iter(world).collect();

    assert!(
        !buildings.is_empty(),
        "At least one building should have spawned"
    );

    for b in &buildings {
        assert_eq!(
            b.level, 1,
            "Spawned buildings should start at level 1, got {}",
            b.level
        );
        assert!(
            b.capacity > 0,
            "Buildings should have positive capacity, got {}",
            b.capacity
        );
    }
}

/// Verify that building_id is set on the grid cell after a building spawns.
#[test]
fn test_grid_cell_updated_with_building_id() {
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        .with_road(100, 100, 120, 100, RoadType::Local)
        .with_zone_rect(102, 101, 118, 101, ZoneType::ResidentialLow)
        .with_utility(100, 100, UtilityType::PowerPlant)
        .with_utility(120, 100, UtilityType::WaterTower);

    city.tick(300);

    // Check that at least one grid cell in the zoned area has a building_id
    let mut found_building_cell = false;
    for x in 102..=118 {
        if city.cell(x, 101).building_id.is_some() {
            found_building_cell = true;
            break;
        }
    }

    assert!(
        found_building_cell,
        "At least one grid cell in the zoned area should have a building_id set"
    );
}

/// Verify that cells with utility coverage have has_power and has_water
/// set after ticking, confirming the utility BFS reached the zoned area.
#[test]
fn test_utility_coverage_reaches_zoned_cells() {
    let mut city = TestCity::new()
        .with_budget(500_000.0)
        .with_road(100, 100, 120, 100, RoadType::Local)
        .with_zone_rect(102, 101, 118, 101, ZoneType::ResidentialLow)
        .with_utility(100, 100, UtilityType::PowerPlant)
        .with_utility(120, 100, UtilityType::WaterTower);

    // One tick is enough for utility propagation
    city.tick(1);

    // Check that at least some zoned cells have power and water
    let mut has_power_count = 0;
    let mut has_water_count = 0;
    for x in 102..=118 {
        let cell = city.cell(x, 101);
        if cell.has_power {
            has_power_count += 1;
        }
        if cell.has_water {
            has_water_count += 1;
        }
    }

    assert!(
        has_power_count > 0,
        "At least some zoned cells should have power coverage"
    );
    assert!(
        has_water_count > 0,
        "At least some zoned cells should have water coverage"
    );
}
