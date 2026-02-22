//! Integration tests for building spawner logic (TEST-008).
//!
//! Tests that the building spawner:
//! - Only spawns on zoned cells
//! - Requires road adjacency
//! - Requires power and water
//! - Does not spawn when demand <= 0
//! - Matches building zone type to cell zone type

use crate::buildings::{Building, EligibleCells, UnderConstruction};
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;
use crate::zones::ZoneDemand;

// ---------------------------------------------------------------------------
// Helper: set up a standard test corridor with road, power, water, and zones.
//
// Road at y=100 from x=90..=110. Power plant at (90,100) and water tower at
// (91,100) -- both ON the road so BFS propagates through the road network.
// Zoned strip at y=99 from x=92..=108 -- directly adjacent to road (1 cell
// away) so utility BFS marks them with power+water.
// ---------------------------------------------------------------------------

fn city_with_zoned_corridor(zone: ZoneType) -> TestCity {
    TestCity::new()
        .with_road(90, 100, 110, 100, RoadType::Local)
        .with_utility(90, 100, UtilityType::PowerPlant)
        .with_utility(91, 100, UtilityType::WaterTower)
        .with_zone_rect(92, 99, 108, 99, zone)
}

/// Inject high demand for all zone types.
fn set_high_demand(city: &mut TestCity) {
    let world = city.world_mut();
    let mut demand = world.resource_mut::<ZoneDemand>();
    demand.residential = 1.0;
    demand.commercial = 1.0;
    demand.industrial = 1.0;
    demand.office = 1.0;
}

/// Inject zero demand for all zone types.
fn set_zero_demand(city: &mut TestCity) {
    let world = city.world_mut();
    let mut demand = world.resource_mut::<ZoneDemand>();
    demand.residential = 0.0;
    demand.commercial = 0.0;
    demand.industrial = 0.0;
    demand.office = 0.0;
}

// ===========================================================================
// 1. Building spawns only on zoned cells
// ===========================================================================

#[test]
fn test_spawner_only_spawns_on_zoned_cells() {
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);
    set_high_demand(&mut city);
    city.tick(20);

    // Collect building positions.
    let world = city.world_mut();
    let building_positions: Vec<(usize, usize)> = world
        .query::<&Building>()
        .iter(world)
        .map(|b| (b.grid_x, b.grid_y))
        .collect();

    // Each building's cell must be zoned.
    let grid = city.grid();
    for (bx, by) in &building_positions {
        let cell = grid.get(*bx, *by);
        assert_ne!(
            cell.zone,
            ZoneType::None,
            "Building spawned on unzoned cell ({}, {})",
            bx,
            by
        );
    }
}

#[test]
fn test_no_buildings_spawn_without_zoning() {
    // Roads and utilities but NO zoning.
    let mut city = TestCity::new()
        .with_road(90, 100, 110, 100, RoadType::Local)
        .with_utility(90, 100, UtilityType::PowerPlant)
        .with_utility(91, 100, UtilityType::WaterTower);

    set_high_demand(&mut city);
    city.tick(20);

    assert_eq!(
        city.building_count(),
        0,
        "No buildings should spawn without zoned cells"
    );
}

// ===========================================================================
// 2. Building requires road adjacency
// ===========================================================================

#[test]
fn test_spawner_requires_road_adjacency() {
    // Zone cells far from any road (center of map, no roads nearby).
    let mut city = TestCity::new().with_zone_rect(50, 50, 55, 55, ZoneType::ResidentialLow);

    // Manually set power and water on those cells.
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        for y in 50..=55 {
            for x in 50..=55 {
                let cell = grid.get_mut(x, y);
                cell.has_power = true;
                cell.has_water = true;
            }
        }
    }

    {
        let world = city.world_mut();
        let mut demand = world.resource_mut::<ZoneDemand>();
        demand.residential = 1.0;
    }

    city.tick(20);

    assert_eq!(
        city.building_count(),
        0,
        "No buildings should spawn without road adjacency"
    );
}

#[test]
fn test_spawner_spawns_when_road_adjacent() {
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);
    set_high_demand(&mut city);
    city.tick(30);

    assert!(
        city.building_count() > 0,
        "Buildings should spawn on zoned cells adjacent to roads with utilities"
    );
}

// ===========================================================================
// 3. Building requires power and water
// ===========================================================================

#[test]
fn test_spawner_requires_power() {
    // Road and zoning + water but NO power source.
    let mut city = TestCity::new()
        .with_road(90, 100, 110, 100, RoadType::Local)
        .with_utility(91, 100, UtilityType::WaterTower)
        .with_zone_rect(92, 99, 108, 99, ZoneType::ResidentialLow);

    {
        let world = city.world_mut();
        let mut demand = world.resource_mut::<ZoneDemand>();
        demand.residential = 1.0;
    }

    city.tick(20);

    assert_eq!(
        city.building_count(),
        0,
        "No buildings should spawn without power"
    );
}

#[test]
fn test_spawner_requires_water() {
    // Road and zoning + power but NO water source.
    let mut city = TestCity::new()
        .with_road(90, 100, 110, 100, RoadType::Local)
        .with_utility(90, 100, UtilityType::PowerPlant)
        .with_zone_rect(92, 99, 108, 99, ZoneType::ResidentialLow);

    {
        let world = city.world_mut();
        let mut demand = world.resource_mut::<ZoneDemand>();
        demand.residential = 1.0;
    }

    city.tick(20);

    assert_eq!(
        city.building_count(),
        0,
        "No buildings should spawn without water"
    );
}

#[test]
fn test_spawner_spawns_with_both_power_and_water() {
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);
    set_high_demand(&mut city);
    city.tick(30);

    assert!(
        city.building_count() > 0,
        "Buildings should spawn when both power and water are available"
    );
}

// ===========================================================================
// 4. No spawn when demand <= 0
// ===========================================================================

#[test]
fn test_no_spawn_when_demand_zero() {
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);

    // Keep demand at zero across multiple ticks. The update_zone_demand system
    // runs on slow ticks, so we re-zero demand before each tick batch.
    set_zero_demand(&mut city);
    city.tick(10);
    set_zero_demand(&mut city);
    city.tick(10);

    assert_eq!(
        city.building_count(),
        0,
        "No buildings should spawn when demand is zero"
    );
}

#[test]
fn test_no_spawn_when_demand_below_threshold() {
    // The spawner skips zones where demand < 0.1.
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);

    {
        let world = city.world_mut();
        let mut demand = world.resource_mut::<ZoneDemand>();
        demand.residential = 0.05;
    }
    city.tick(10);
    {
        let world = city.world_mut();
        let mut demand = world.resource_mut::<ZoneDemand>();
        demand.residential = 0.05;
    }
    city.tick(10);

    assert_eq!(
        city.building_count(),
        0,
        "No buildings should spawn when demand is below 0.1 threshold"
    );
}

#[test]
fn test_buildings_spawn_when_demand_above_threshold() {
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);
    set_high_demand(&mut city);
    city.tick(30);

    assert!(
        city.building_count() > 0,
        "Buildings should spawn when demand is above the 0.1 threshold"
    );
}

// ===========================================================================
// 5. Correct zone type matches building zone type
// ===========================================================================

#[test]
fn test_residential_zone_spawns_residential_buildings() {
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);
    set_high_demand(&mut city);
    city.tick(30);

    let building_count = city.building_count();
    assert!(building_count > 0, "Should have spawned buildings");

    let world = city.world_mut();
    let buildings: Vec<ZoneType> = world
        .query::<&Building>()
        .iter(world)
        .map(|b| b.zone_type)
        .collect();

    for zone in &buildings {
        assert_eq!(
            *zone,
            ZoneType::ResidentialLow,
            "Building zone type should match the zoned cell type"
        );
    }
}

#[test]
fn test_commercial_zone_spawns_commercial_buildings() {
    let mut city = city_with_zoned_corridor(ZoneType::CommercialLow);
    set_high_demand(&mut city);
    city.tick(30);

    let building_count = city.building_count();
    assert!(
        building_count > 0,
        "Should have spawned commercial buildings"
    );

    let world = city.world_mut();
    let buildings: Vec<ZoneType> = world
        .query::<&Building>()
        .iter(world)
        .map(|b| b.zone_type)
        .collect();

    for zone in &buildings {
        assert_eq!(
            *zone,
            ZoneType::CommercialLow,
            "Building zone type should match CommercialLow"
        );
    }
}

#[test]
fn test_industrial_zone_spawns_industrial_buildings() {
    let mut city = city_with_zoned_corridor(ZoneType::Industrial);
    set_high_demand(&mut city);
    city.tick(30);

    let building_count = city.building_count();
    assert!(
        building_count > 0,
        "Should have spawned industrial buildings"
    );

    let world = city.world_mut();
    let buildings: Vec<ZoneType> = world
        .query::<&Building>()
        .iter(world)
        .map(|b| b.zone_type)
        .collect();

    for zone in &buildings {
        assert_eq!(
            *zone,
            ZoneType::Industrial,
            "Building zone type should match Industrial"
        );
    }
}

#[test]
fn test_office_zone_spawns_office_buildings() {
    let mut city = city_with_zoned_corridor(ZoneType::Office);
    set_high_demand(&mut city);
    city.tick(30);

    let building_count = city.building_count();
    assert!(building_count > 0, "Should have spawned office buildings");

    let world = city.world_mut();
    let buildings: Vec<ZoneType> = world
        .query::<&Building>()
        .iter(world)
        .map(|b| b.zone_type)
        .collect();

    for zone in &buildings {
        assert_eq!(
            *zone,
            ZoneType::Office,
            "Building zone type should match Office"
        );
    }
}

// ===========================================================================
// Additional coverage: eligible cells, construction, and edge cases
// ===========================================================================

#[test]
fn test_eligible_cells_populated_after_tick() {
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);

    // Tick to trigger utility propagation + eligible cell rebuild.
    city.tick(3);

    let eligible = city.resource::<EligibleCells>();
    let res_low_entry = eligible
        .cells
        .iter()
        .find(|(z, _)| *z == ZoneType::ResidentialLow);

    assert!(
        res_low_entry.is_some(),
        "EligibleCells should have an entry for ResidentialLow"
    );

    let (_, cells) = res_low_entry.unwrap();
    assert!(
        !cells.is_empty(),
        "There should be eligible ResidentialLow cells near road with power+water"
    );

    // All eligible cells should be in the zoned strip y=99, x=92..=108.
    for &(x, y) in cells {
        assert_eq!(y, 99, "Eligible cell should be at y=99, got y={}", y);
        assert!(
            (92..=108).contains(&x),
            "Eligible cell should be at x in 92..=108, got x={}",
            x
        );
    }
}

#[test]
fn test_eligible_cells_empty_without_utilities() {
    // Zone cells next to road but no power/water utilities.
    let mut city = TestCity::new()
        .with_road(90, 100, 110, 100, RoadType::Local)
        .with_zone_rect(92, 99, 108, 99, ZoneType::ResidentialLow);

    city.tick(3);

    let eligible = city.resource::<EligibleCells>();
    let res_low_entry = eligible
        .cells
        .iter()
        .find(|(z, _)| *z == ZoneType::ResidentialLow);

    if let Some((_, cells)) = res_low_entry {
        assert!(
            cells.is_empty(),
            "No cells should be eligible without power and water, found {}",
            cells.len()
        );
    }
}

#[test]
fn test_spawned_building_starts_under_construction() {
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);
    set_high_demand(&mut city);
    city.tick(30);

    let world = city.world_mut();
    let under_construction_count = world
        .query::<(&Building, &UnderConstruction)>()
        .iter(world)
        .count();

    // After 30 ticks, all spawned buildings should still be under construction
    // (default construction_ticks=100).
    let total_buildings = world.query::<&Building>().iter(world).count();
    assert!(total_buildings > 0, "Should have spawned buildings");
    assert_eq!(
        under_construction_count, total_buildings,
        "All newly spawned buildings should be under construction after only 30 ticks"
    );
}

#[test]
fn test_construction_completes_after_enough_ticks() {
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);
    set_high_demand(&mut city);

    // Tick enough to spawn + complete construction (spawn_interval=2, construction=100).
    city.tick(150);

    let world = city.world_mut();
    let completed_count = world
        .query_filtered::<&Building, bevy::prelude::Without<UnderConstruction>>()
        .iter(world)
        .count();

    assert!(
        completed_count > 0,
        "Some buildings should have completed construction after 150 ticks"
    );
}

#[test]
fn test_spawned_building_has_zero_occupants() {
    let mut city = city_with_zoned_corridor(ZoneType::CommercialHigh);

    {
        let world = city.world_mut();
        let mut demand = world.resource_mut::<ZoneDemand>();
        demand.commercial = 1.0;
    }

    city.tick(20);

    let world = city.world_mut();
    for building in world.query::<&Building>().iter(world) {
        assert_eq!(
            building.occupants, 0,
            "Newly spawned building at ({}, {}) should have 0 occupants",
            building.grid_x, building.grid_y
        );
    }
}

#[test]
fn test_spawned_building_has_correct_capacity() {
    let mut city = city_with_zoned_corridor(ZoneType::Industrial);

    {
        let world = city.world_mut();
        let mut demand = world.resource_mut::<ZoneDemand>();
        demand.industrial = 1.0;
    }

    city.tick(30);

    let expected_capacity = Building::capacity_for_level(ZoneType::Industrial, 1);
    let world = city.world_mut();
    for building in world.query::<&Building>().iter(world) {
        assert_eq!(
            building.capacity, expected_capacity,
            "Industrial L1 building should have capacity {}, got {}",
            expected_capacity, building.capacity
        );
        assert_eq!(
            building.level, 1,
            "Spawned building should start at level 1"
        );
    }
}

#[test]
fn test_building_not_spawned_on_road_cell() {
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);
    set_high_demand(&mut city);
    city.tick(30);

    // Collect building positions first, then check grid (avoids borrow conflict).
    let world = city.world_mut();
    let building_positions: Vec<(usize, usize)> = world
        .query::<&Building>()
        .iter(world)
        .map(|b| (b.grid_x, b.grid_y))
        .collect();

    let grid = city.grid();
    for (bx, by) in &building_positions {
        let cell = grid.get(*bx, *by);
        assert_ne!(
            cell.cell_type,
            CellType::Road,
            "Building should not be placed on a road cell at ({}, {})",
            bx,
            by
        );
    }
}

#[test]
fn test_multiple_zone_types_spawn_independently() {
    // Two separate zoned corridors with different zone types.
    let mut city = TestCity::new()
        .with_road(90, 100, 110, 100, RoadType::Local)
        .with_road(90, 120, 110, 120, RoadType::Local)
        .with_utility(90, 100, UtilityType::PowerPlant)
        .with_utility(91, 100, UtilityType::WaterTower)
        .with_utility(90, 120, UtilityType::PowerPlant)
        .with_utility(91, 120, UtilityType::WaterTower)
        .with_zone_rect(93, 99, 108, 99, ZoneType::ResidentialLow)
        .with_zone_rect(93, 119, 108, 119, ZoneType::Industrial);

    set_high_demand(&mut city);
    city.tick(30);

    let res_count = city.buildings_in_zone(ZoneType::ResidentialLow);
    let ind_count = city.buildings_in_zone(ZoneType::Industrial);

    assert!(
        res_count > 0,
        "Should have spawned residential buildings, got {}",
        res_count
    );
    assert!(
        ind_count > 0,
        "Should have spawned industrial buildings, got {}",
        ind_count
    );
}

#[test]
fn test_building_grid_cell_marked_with_building_id() {
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);
    set_high_demand(&mut city);
    city.tick(30);

    let world = city.world_mut();
    let buildings: Vec<(usize, usize)> = world
        .query::<&Building>()
        .iter(world)
        .map(|b| (b.grid_x, b.grid_y))
        .collect();

    assert!(!buildings.is_empty(), "Should have spawned buildings");

    let grid = city.grid();
    for (x, y) in &buildings {
        let cell = grid.get(*x, *y);
        assert!(
            cell.building_id.is_some(),
            "Grid cell ({}, {}) should have building_id set after building spawn",
            x,
            y
        );
    }
}

#[test]
fn test_occupied_cell_not_eligible_for_second_building() {
    let mut city = city_with_zoned_corridor(ZoneType::ResidentialLow);
    set_high_demand(&mut city);
    city.tick(50);

    // Count buildings per grid cell -- each cell should have at most one.
    let world = city.world_mut();
    let mut cell_counts = std::collections::HashMap::new();
    for building in world.query::<&Building>().iter(world) {
        *cell_counts
            .entry((building.grid_x, building.grid_y))
            .or_insert(0u32) += 1;
    }

    for ((x, y), count) in &cell_counts {
        assert_eq!(
            *count, 1,
            "Cell ({}, {}) should have at most 1 building, found {}",
            x, y, count
        );
    }
}
