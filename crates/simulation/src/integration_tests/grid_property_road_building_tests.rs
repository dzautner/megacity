//! Property-based tests for road adjacency and building invariants (TEST-013b).
//!
//! Uses manual randomized testing with `rand` to verify:
//! - Building ID references point to valid entities
//! - Road adjacency symmetry: if A->B then B->A
//! - Road edges are cardinal-adjacent only
//! - Zone types match building types
//! - Building grid coordinates match the cell that references them

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{WorldGrid, ZoneType};
use crate::roads::RoadNetwork;
use crate::test_harness::TestCity;

const NUM_SAMPLES: usize = 5000;
const SEED: u64 = 0xDEAD_BEEF_CAFE_1337;

// ---------------------------------------------------------------------------
// 1. Building ID Validity
// ---------------------------------------------------------------------------

#[test]
fn test_prop_building_ids_are_valid_entities() {
    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];

    let mut rng = StdRng::seed_from_u64(SEED + 6);
    let mut city = TestCity::new();

    // Collect building data before spawning to avoid borrow issues
    let mut building_data = Vec::new();
    {
        let grid = city.world_mut().resource::<WorldGrid>();
        for _ in 0..200 {
            let x = rng.gen_range(0..GRID_WIDTH);
            let y = rng.gen_range(0..GRID_HEIGHT);
            if grid.get(x, y).building_id.is_some() {
                continue;
            }
            let zone = zones[rng.gen_range(0..zones.len())];
            let level = rng.gen_range(1..=zone.max_level());
            let capacity = Building::capacity_for_level(zone, level);
            building_data.push((x, y, zone, level, capacity));
        }
    }

    for (x, y, zone, level, capacity) in &building_data {
        let world = city.world_mut();
        let entity = world
            .spawn(Building {
                zone_type: *zone,
                level: *level,
                grid_x: *x,
                grid_y: *y,
                capacity: *capacity,
                occupants: 0,
            })
            .id();
        world.resource_mut::<WorldGrid>().get_mut(*x, *y).building_id = Some(entity);
        world.resource_mut::<WorldGrid>().get_mut(*x, *y).zone = *zone;
    }

    // Verify every building_id points to a valid entity
    let world = city.world_mut();
    let grid = world.resource::<WorldGrid>();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if let Some(entity) = grid.get(x, y).building_id {
                assert!(
                    world.get_entity(entity).is_some(),
                    "Cell ({x},{y}) building_id {entity:?} does not exist"
                );
            }
        }
    }
}

#[test]
fn test_prop_building_ids_valid_after_ticks_on_tel_aviv() {
    let mut city = TestCity::with_tel_aviv();
    city.tick(10);

    let world = city.world_mut();
    let grid = world.resource::<WorldGrid>();
    let mut checked = 0;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if let Some(entity) = grid.get(x, y).building_id {
                assert!(
                    world.get_entity(entity).is_some(),
                    "Tel Aviv cell ({x},{y}): building_id {entity:?} is dangling"
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "Tel Aviv had zero building_ids");
}

// ---------------------------------------------------------------------------
// 2. Road Adjacency Symmetry
// ---------------------------------------------------------------------------

#[test]
fn test_prop_road_adjacency_is_symmetric_random_placement() {
    let mut rng = StdRng::seed_from_u64(SEED + 7);
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    for _ in 0..1000 {
        let x = rng.gen_range(0..GRID_WIDTH);
        let y = rng.gen_range(0..GRID_HEIGHT);
        roads.place_road(&mut grid, x, y);
    }

    for (node, neighbors) in &roads.edges {
        for neighbor in neighbors {
            assert!(
                roads
                    .edges
                    .get(neighbor)
                    .map_or(false, |n| n.contains(node)),
                "Asymmetric: ({},{}) -> ({},{}) without reverse",
                node.0,
                node.1,
                neighbor.0,
                neighbor.1
            );
        }
    }
}

#[test]
fn test_prop_road_adjacency_symmetric_after_removals() {
    let mut rng = StdRng::seed_from_u64(SEED + 8);
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    for x in 10..50 {
        for y in 10..50 {
            roads.place_road(&mut grid, x, y);
        }
    }

    for _ in 0..500 {
        let x = rng.gen_range(10..50);
        let y = rng.gen_range(10..50);
        roads.remove_road(&mut grid, x, y);
    }

    for (node, neighbors) in &roads.edges {
        for neighbor in neighbors {
            assert!(
                roads
                    .edges
                    .get(neighbor)
                    .map_or(false, |n| n.contains(node)),
                "Asymmetric after removal: ({},{}) -> ({},{})",
                node.0,
                node.1,
                neighbor.0,
                neighbor.1
            );
        }
    }
}

#[test]
fn test_prop_road_adjacency_symmetric_on_tel_aviv() {
    let city = TestCity::with_tel_aviv();
    let roads = city.road_network();

    for (node, neighbors) in &roads.edges {
        for neighbor in neighbors {
            assert!(
                roads
                    .edges
                    .get(neighbor)
                    .map_or(false, |n| n.contains(node)),
                "Tel Aviv asymmetric: ({},{}) -> ({},{})",
                node.0,
                node.1,
                neighbor.0,
                neighbor.1
            );
        }
    }
}

#[test]
fn test_prop_road_neighbors_are_adjacent_cells() {
    let mut rng = StdRng::seed_from_u64(SEED + 9);
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    for _ in 0..1000 {
        let x = rng.gen_range(0..GRID_WIDTH);
        let y = rng.gen_range(0..GRID_HEIGHT);
        roads.place_road(&mut grid, x, y);
    }

    for (node, neighbors) in &roads.edges {
        for neighbor in neighbors {
            let dx = (node.0 as i32 - neighbor.0 as i32).unsigned_abs();
            let dy = (node.1 as i32 - neighbor.1 as i32).unsigned_abs();
            assert!(
                (dx == 1 && dy == 0) || (dx == 0 && dy == 1),
                "Non-cardinal edge: ({},{}) -> ({},{}) dx={dx} dy={dy}",
                node.0,
                node.1,
                neighbor.0,
                neighbor.1
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Zone-Building Type Consistency
// ---------------------------------------------------------------------------

#[test]
fn test_prop_zone_matches_building_type() {
    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];

    let mut rng = StdRng::seed_from_u64(SEED + 10);
    let mut builder = TestCity::new();
    let mut positions = Vec::new();

    for _ in 0..100 {
        let x = rng.gen_range(0..GRID_WIDTH);
        let y = rng.gen_range(0..GRID_HEIGHT);
        let zone = zones[rng.gen_range(0..zones.len())];
        let level = rng.gen_range(1..=zone.max_level());
        builder = builder.with_building(x, y, zone, level);
        positions.push((x, y, zone));
    }

    let mut city = builder;
    let world = city.world_mut();
    let grid = world.resource::<WorldGrid>();

    for (x, y, expected_zone) in &positions {
        let cell = grid.get(*x, *y);
        if let Some(entity) = cell.building_id {
            if let Some(building) = world.get::<Building>(entity) {
                assert_eq!(
                    building.zone_type, cell.zone,
                    "Cell ({x},{y}): building zone {:?} != cell zone {:?}",
                    building.zone_type, cell.zone
                );
                assert_eq!(
                    building.zone_type, *expected_zone,
                    "Cell ({x},{y}): building zone {:?} != expected {:?}",
                    building.zone_type, expected_zone
                );
            }
        }
    }
}

#[test]
fn test_prop_building_grid_coords_match_cell() {
    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::CommercialLow,
        ZoneType::Industrial,
        ZoneType::Office,
    ];
    let mut rng = StdRng::seed_from_u64(SEED + 11);
    let mut builder = TestCity::new();

    for _ in 0..200 {
        let x = rng.gen_range(0..GRID_WIDTH);
        let y = rng.gen_range(0..GRID_HEIGHT);
        let zone = zones[rng.gen_range(0..zones.len())];
        builder = builder.with_building(x, y, zone, 1);
    }

    let mut city = builder;
    let world = city.world_mut();
    let grid = world.resource::<WorldGrid>();

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if let Some(entity) = grid.get(x, y).building_id {
                if let Some(building) = world.get::<Building>(entity) {
                    assert_eq!(
                        (building.grid_x, building.grid_y),
                        (x, y),
                        "Building at cell ({x},{y}) has coords ({},{})",
                        building.grid_x,
                        building.grid_y
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Mixed: random place + remove roads, verify all invariants
// ---------------------------------------------------------------------------

#[test]
fn test_prop_grid_consistency_after_random_road_mutations() {
    let mut rng = StdRng::seed_from_u64(SEED + 15);
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();

    // Perform a mix of placements and removals
    for _ in 0..NUM_SAMPLES {
        let x = rng.gen_range(0..GRID_WIDTH);
        let y = rng.gen_range(0..GRID_HEIGHT);
        if rng.gen_bool(0.7) {
            roads.place_road(&mut grid, x, y);
        } else {
            roads.remove_road(&mut grid, x, y);
        }
    }

    // Invariant 1: Symmetry
    for (node, neighbors) in &roads.edges {
        for neighbor in neighbors {
            assert!(
                roads
                    .edges
                    .get(neighbor)
                    .map_or(false, |n| n.contains(node)),
                "Asymmetric after mutations: ({},{}) -> ({},{})",
                node.0,
                node.1,
                neighbor.0,
                neighbor.1
            );
        }
    }

    // Invariant 2: Network matches grid
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let is_road_cell = grid.get(x, y).cell_type == crate::grid::CellType::Road;
            let in_network = roads.is_road(x, y);
            assert_eq!(
                is_road_cell, in_network,
                "Cell ({x},{y}): grid says road={is_road_cell}, network says road={in_network}"
            );
        }
    }

    // Invariant 3: All edges are cardinal
    for (node, neighbors) in &roads.edges {
        for neighbor in neighbors {
            let dx = (node.0 as i32 - neighbor.0 as i32).unsigned_abs();
            let dy = (node.1 as i32 - neighbor.1 as i32).unsigned_abs();
            assert!((dx == 1 && dy == 0) || (dx == 0 && dy == 1));
        }
    }
}
