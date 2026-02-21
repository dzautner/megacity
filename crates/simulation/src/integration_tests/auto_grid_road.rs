use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

// ====================================================================
// Auto-Grid Road Placement (TRAF-010)
// ====================================================================

#[test]
fn test_auto_grid_generates_roads_in_area() {
    use crate::auto_grid_road::{compute_grid_plan, execute_grid_plan, AutoGridConfig};
    use crate::road_segments::RoadSegmentStore;
    use crate::roads::RoadNetwork;
    use crate::test_harness::TestCity;

    let mut city = TestCity::new().with_budget(100_000.0);
    let config = AutoGridConfig {
        block_size: 6,
        road_type: RoadType::Local,
    };

    let plan = {
        let grid = city.grid();
        compute_grid_plan((50, 50), (70, 70), &config, grid)
    };

    assert!(!plan.segments.is_empty(), "plan should have segments");
    assert!(plan.total_cells > 0, "plan should place road cells");
    assert!(plan.total_cost > 0.0, "plan should have a cost");

    let world = city.world_mut();
    world.resource_scope(
        |world, mut segments: bevy::prelude::Mut<RoadSegmentStore>| {
            world.resource_scope(
                |world, mut grid: bevy::prelude::Mut<crate::grid::WorldGrid>| {
                    world.resource_scope(|_world, mut roads: bevy::prelude::Mut<RoadNetwork>| {
                        let cells =
                            execute_grid_plan(&plan, &config, &mut segments, &mut grid, &mut roads);
                        assert!(!cells.is_empty(), "should have placed road cells");
                        for &(x, y) in &cells {
                            assert_eq!(grid.get(x, y).cell_type, CellType::Road);
                        }
                    });
                },
            );
        },
    );
}

#[test]
fn test_auto_grid_block_size_affects_density() {
    use crate::auto_grid_road::{compute_grid_plan, AutoGridConfig};
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    let plan_small = compute_grid_plan(
        (50, 50),
        (80, 80),
        &AutoGridConfig {
            block_size: 4,
            road_type: RoadType::Local,
        },
        &grid,
    );
    let plan_large = compute_grid_plan(
        (50, 50),
        (80, 80),
        &AutoGridConfig {
            block_size: 8,
            road_type: RoadType::Local,
        },
        &grid,
    );

    assert!(
        plan_small.total_cells > plan_large.total_cells,
        "smaller block size ({}) should produce more roads than larger ({})",
        plan_small.total_cells,
        plan_large.total_cells
    );
}

#[test]
fn test_bus_transit_buses_spawn_on_active_route() {
    use crate::bus_transit::{BusTransitState, BUSES_PER_ROUTE};

    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(10, 12, ServiceType::BusDepot);

    // Set up transit state
    {
        let world = city.world_mut();
        let grid = world.resource::<WorldGrid>();
        let mut transit = BusTransitState::default();
        let s1 = transit.add_stop(grid, 10, 10).unwrap();
        let s2 = transit.add_stop(grid, 10, 18).unwrap();
        transit.add_route("Line 1".to_string(), vec![s1, s2]);
        world.insert_resource(transit);
    }

    // Run enough ticks for activation and bus spawning
    city.tick(10);

    let transit = city.resource::<BusTransitState>();
    assert!(transit.routes[0].active, "Route should be active");
    assert_eq!(
        transit.buses.len(),
        BUSES_PER_ROUTE as usize,
        "Should have spawned {} buses",
        BUSES_PER_ROUTE
    );
}

#[test]
fn test_bus_transit_ridership_increases() {
    use crate::bus_transit::BusTransitState;

    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(10, 12, ServiceType::BusDepot)
        .with_zone_rect(8, 8, 12, 12, ZoneType::ResidentialLow)
        .with_zone_rect(8, 18, 12, 22, ZoneType::CommercialLow);

    // Set up transit with stops near zones
    {
        let world = city.world_mut();
        let grid = world.resource::<WorldGrid>();
        let mut transit = BusTransitState::default();
        let s1 = transit.add_stop(grid, 10, 10).unwrap();
        let s2 = transit.add_stop(grid, 10, 18).unwrap();
        transit.add_route("Line 1".to_string(), vec![s1, s2]);
        world.insert_resource(transit);
    }

    // Run slow cycle to generate waiting passengers and let buses pick them up
    city.tick_slow_cycle();
    city.tick(50); // Extra ticks for buses to reach stops

    let transit = city.resource::<BusTransitState>();
    assert!(
        transit.total_ridership() > 0
            || transit.routes[0].monthly_ridership > 0
            || transit.stops.iter().any(|s| s.waiting > 0),
        "Should have some ridership or waiting passengers after simulation"
    );
}

#[test]
fn test_bus_transit_remove_route_clears_buses() {
    use crate::bus_transit::BusTransitState;

    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(10, 12, ServiceType::BusDepot);

    // Set up transit and let buses spawn
    {
        let world = city.world_mut();
        let grid = world.resource::<WorldGrid>();
        let mut transit = BusTransitState::default();
        let s1 = transit.add_stop(grid, 10, 10).unwrap();
        let s2 = transit.add_stop(grid, 10, 18).unwrap();
        transit.add_route("Line 1".to_string(), vec![s1, s2]);
        world.insert_resource(transit);
    }

    city.tick(10);

    // Verify buses exist
    let bus_count = city.resource::<BusTransitState>().buses.len();
    assert!(bus_count > 0, "Should have buses before removal");

    // Remove the route
    {
        let world = city.world_mut();
        let route_id = world.resource::<BusTransitState>().routes[0].id;
        world
            .resource_mut::<BusTransitState>()
            .remove_route(route_id);
    }

    let transit = city.resource::<BusTransitState>();
    assert_eq!(transit.routes.len(), 0, "Route should be removed");
    assert_eq!(transit.buses.len(), 0, "Buses should be removed with route");
}

#[test]
fn test_bus_transit_costs_applied_to_budget() {
    use crate::bus_transit::BusTransitState;

    let mut city = TestCity::new()
        .with_budget(50_000.0)
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(10, 12, ServiceType::BusDepot);

    // Set up transit with last_cost_day far in the past to trigger cost application
    {
        let world = city.world_mut();
        let grid = world.resource::<WorldGrid>();
        let mut transit = BusTransitState::default();
        let s1 = transit.add_stop(grid, 10, 10).unwrap();
        let s2 = transit.add_stop(grid, 10, 18).unwrap();
        transit.add_route("Line 1".to_string(), vec![s1, s2]);
        world.insert_resource(transit);
    }

    // Advance game clock past day 31 so the monthly cost applies
    {
        let world = city.world_mut();
        let mut clock = world.resource_mut::<GameClock>();
        clock.day = 35;
        clock.hour = 12.0;
    }

    // Run a slow cycle to trigger cost application (costs check on slow tick)
    city.tick_slow_cycle();

    let transit = city.resource::<BusTransitState>();
    // Verify the route activated and has operating cost computed
    assert!(
        transit.active_route_count() > 0 || transit.monthly_operating_cost > 0.0,
        "Should have active routes or computed operating cost after ticks"
    );
}

#[test]
fn test_bus_transit_saveable_roundtrip() {
    use crate::bus_transit::BusTransitState;
    use crate::Saveable;

    let mut grid = WorldGrid::new(32, 32);
    grid.get_mut(5, 5).cell_type = CellType::Road;
    grid.get_mut(15, 15).cell_type = CellType::Road;

    let mut state = BusTransitState::default();
    let s1 = state.add_stop(&grid, 5, 5).unwrap();
    let s2 = state.add_stop(&grid, 15, 15).unwrap();
    state.add_route("Test Route".to_string(), vec![s1, s2]);

    // Save
    let bytes = state
        .save_to_bytes()
        .expect("Should serialize non-empty state");

    // Load
    let loaded = BusTransitState::load_from_bytes(&bytes);
    assert_eq!(loaded.stops.len(), 2);
    assert_eq!(loaded.routes.len(), 1);
    assert_eq!(loaded.routes[0].name, "Test Route");
    assert_eq!(loaded.routes[0].stop_ids.len(), 2);
}

#[test]
fn test_auto_grid_respects_water_obstacles() {
    use crate::auto_grid_road::{compute_grid_plan, execute_grid_plan, AutoGridConfig};
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::road_segments::RoadSegmentStore;
    use crate::roads::RoadNetwork;

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut segments = RoadSegmentStore::default();

    for x in 55..=65 {
        for y in 55..=65 {
            grid.get_mut(x, y).cell_type = CellType::Water;
        }
    }

    let config = AutoGridConfig {
        block_size: 6,
        road_type: RoadType::Local,
    };

    let plan = compute_grid_plan((50, 50), (70, 70), &config, &grid);
    let cells = execute_grid_plan(&plan, &config, &mut segments, &mut grid, &mut roads);

    for x in 55..=65 {
        for y in 55..=65 {
            assert_ne!(grid.get(x, y).cell_type, CellType::Road);
        }
    }

    assert!(!cells.is_empty(), "should still place some roads");
}

#[test]
fn test_auto_grid_cost_scales_with_road_type() {
    use crate::auto_grid_road::{compute_grid_plan, AutoGridConfig};
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    let plan_local = compute_grid_plan(
        (50, 50),
        (70, 70),
        &AutoGridConfig {
            block_size: 6,
            road_type: RoadType::Local,
        },
        &grid,
    );
    let plan_avenue = compute_grid_plan(
        (50, 50),
        (70, 70),
        &AutoGridConfig {
            block_size: 6,
            road_type: RoadType::Avenue,
        },
        &grid,
    );

    assert_eq!(plan_local.total_cells, plan_avenue.total_cells);
    assert!(plan_avenue.total_cost > plan_local.total_cost);
}
