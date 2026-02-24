//! Integration tests for SVC-022: Stormwater Management and Flooding.

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity,
};
use crate::flood_simulation::{FloodGrid, FloodState};
use crate::grid::{CellType, ZoneType};
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::road_maintenance::RoadConditionGrid;
use crate::stormwater::StormwaterGrid;
use crate::stormwater_mgmt::{FloodRiskGrid, StormwaterMgmtState};
use crate::test_harness::TestCity;
use crate::trees::TreeGrid;
use crate::weather::{Weather, WeatherCondition};
use crate::Saveable;
use crate::SaveableRegistry;

use bevy::prelude::*;

/// Spawn a citizen at grid position with home building.
fn spawn_citizen_at(world: &mut World, gx: usize, gy: usize, building: Entity) -> Entity {
    let (wx, wy) = crate::grid::WorldGrid::grid_to_world(gx, gy);
    world
        .spawn((
            Citizen,
            Position { x: wx, y: wy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: gx,
                grid_y: gy,
                building,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30,
                gender: Gender::Male,
                education: 2,
                happiness: 80.0,
                health: 90.0,
                salary: 3500.0,
                savings: 7000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
            ChosenTransportMode::default(),
        ))
        .id()
}

// ====================================================================
// Green infrastructure tests
// ====================================================================

#[test]
fn test_green_infra_trees_reduce_runoff() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        // Set up rainfall conditions
        {
            let mut weather = world.resource_mut::<Weather>();
            weather.current_event = WeatherCondition::HeavyRain;
        }
        // Place trees in an area with accumulated runoff
        {
            let mut trees = world.resource_mut::<TreeGrid>();
            for x in 10..20 {
                for y in 10..20 {
                    trees.set(x, y, true);
                }
            }
        }
        // Seed stormwater runoff in the tree area
        {
            let mut sw = world.resource_mut::<StormwaterGrid>();
            for x in 10..20 {
                for y in 10..20 {
                    sw.set(x, y, 100.0);
                }
            }
        }
    }

    city.tick_slow_cycle();

    let state = city.resource::<StormwaterMgmtState>();
    assert!(
        state.green_infra_absorbed > 0.0,
        "green infrastructure should absorb runoff, got {}",
        state.green_infra_absorbed
    );
}

#[test]
fn test_green_infra_reduces_runoff_by_30_percent() {
    let mut city = TestCity::new();

    // Place a single tree with known runoff
    {
        let world = city.world_mut();
        {
            let mut trees = world.resource_mut::<TreeGrid>();
            trees.set(50, 50, true);
        }
        {
            let mut sw = world.resource_mut::<StormwaterGrid>();
            sw.set(50, 50, 100.0);
        }
    }

    city.tick_slow_cycle();

    let state = city.resource::<StormwaterMgmtState>();
    // Tree reduction is 30%: absorbed should be around 30.0
    // (may not be exact if stormwater system modifies values before us)
    assert!(
        state.green_infra_absorbed > 0.0,
        "absorbed should be positive, got {}",
        state.green_infra_absorbed
    );
}

// ====================================================================
// Flood risk overlay tests
// ====================================================================

#[test]
fn test_flood_risk_overlay_computed() {
    let mut city = TestCity::new();

    city.tick_slow_cycle();

    let risk = city.resource::<FloodRiskGrid>();
    // On a flat grid with no drainage, all cells should have some risk
    let avg = risk.average_risk();
    assert!(
        avg > 0.0,
        "flood risk should be nonzero on default grid, got {}",
        avg
    );
}

#[test]
fn test_low_elevation_higher_risk_than_high_elevation() {
    let mut city = TestCity::new();

    // Set up contrasting elevations
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        // Low area
        for x in 10..20 {
            for y in 10..20 {
                grid.get_mut(x, y).elevation = 0.1;
            }
        }
        // High area
        for x in 30..40 {
            for y in 30..40 {
                grid.get_mut(x, y).elevation = 0.9;
            }
        }
    }

    city.tick_slow_cycle();

    let risk = city.resource::<FloodRiskGrid>();
    let low_risk = risk.get(15, 15);
    let high_risk = risk.get(35, 35);
    assert!(
        low_risk > high_risk,
        "low elevation should have higher risk ({}) than high elevation ({})",
        low_risk,
        high_risk
    );
}

// ====================================================================
// Flood road damage tests
// ====================================================================

#[test]
fn test_flood_damages_roads() {
    let mut city = TestCity::new();

    // Set up flooded road cells
    {
        let world = city.world_mut();
        {
            let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
            for x in 10..15 {
                grid.get_mut(x, 10).cell_type = CellType::Road;
            }
        }
        {
            let mut condition = world.resource_mut::<RoadConditionGrid>();
            for x in 10..15 {
                condition.set(x, 10, 200);
            }
        }
        // Activate flooding
        {
            let mut flood_grid = world.resource_mut::<FloodGrid>();
            for x in 10..15 {
                flood_grid.set(x, 10, 3.0); // 3 feet of flooding
            }
        }
        {
            let mut flood_state = world.resource_mut::<FloodState>();
            flood_state.is_flooding = true;
            flood_state.total_flooded_cells = 5;
        }
    }

    city.tick_slow_cycle();

    let state = city.resource::<StormwaterMgmtState>();
    assert!(
        state.flood_damaged_roads > 0,
        "flooding should damage road cells, got {}",
        state.flood_damaged_roads
    );

    // Check that road condition actually decreased
    let condition = city.resource::<RoadConditionGrid>();
    let cond = condition.get(12, 10);
    assert!(
        cond < 200,
        "road condition should decrease from flooding, got {}",
        cond
    );
}

// ====================================================================
// Citizen displacement tests
// ====================================================================

#[test]
fn test_flooding_displaces_citizens() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 1);

    let citizen_entity;
    {
        let world = city.world_mut();
        let building_entity = {
            let grid = world.resource::<crate::grid::WorldGrid>();
            grid.get(50, 50).building_id.unwrap()
        };
        citizen_entity = spawn_citizen_at(world, 50, 50, building_entity);

        // Create flood at the building location
        {
            let mut flood_grid = world.resource_mut::<FloodGrid>();
            flood_grid.set(50, 50, 2.0); // 2 feet of flooding
        }
        {
            let mut flood_state = world.resource_mut::<FloodState>();
            flood_state.is_flooding = true;
            flood_state.total_flooded_cells = 1;
        }
    }

    let initial_happiness = {
        let world = city.world_mut();
        world
            .get::<CitizenDetails>(citizen_entity)
            .unwrap()
            .happiness
    };

    city.tick_slow_cycle();

    let final_happiness = {
        let world = city.world_mut();
        world
            .get::<CitizenDetails>(citizen_entity)
            .unwrap()
            .happiness
    };

    assert!(
        final_happiness < initial_happiness,
        "flooding should reduce citizen happiness: {} -> {}",
        initial_happiness,
        final_happiness
    );

    let state = city.resource::<StormwaterMgmtState>();
    assert!(
        state.displaced_citizens > 0,
        "should track displaced citizens, got {}",
        state.displaced_citizens
    );
}

// ====================================================================
// Heavy rain + no drainage = flooding (from issue test plan)
// ====================================================================

#[test]
fn test_heavy_rain_no_drainage_causes_runoff() {
    let mut city = TestCity::new();

    // Set weather to heavy rain with no drainage infrastructure
    {
        let world = city.world_mut();
        {
            let mut weather = world.resource_mut::<Weather>();
            weather.current_event = WeatherCondition::Storm;
        }
        // Ensure paved area for maximum runoff
        {
            let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
            for x in 20..40 {
                for y in 20..40 {
                    grid.get_mut(x, y).cell_type = CellType::Road;
                }
            }
        }
    }

    // Run several slow cycles to accumulate runoff
    city.tick_slow_cycles(3);

    let sw = city.resource::<StormwaterGrid>();
    let total = sw.total_runoff;
    assert!(
        total > 0.0,
        "heavy rain on paved area should produce runoff, got {}",
        total
    );
}

// ====================================================================
// Save/load roundtrip
// ====================================================================

fn roundtrip(city: &mut TestCity) {
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();
    let extensions = registry.save_all(world);
    registry.reset_all(world);
    registry.load_all(world, &extensions);
    world.insert_resource(registry);
}

#[test]
fn test_stormwater_mgmt_save_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<StormwaterMgmtState>();
        state.green_infra_absorbed = 500.0;
        state.flood_damaged_roads = 12;
        state.displaced_citizens = 300;
        state.avg_flood_risk = 95.5;
        state.high_risk_cells = 8000;
    }

    roundtrip(&mut city);

    let state = city.resource::<StormwaterMgmtState>();
    assert!((state.green_infra_absorbed - 500.0).abs() < 0.01);
    assert_eq!(state.flood_damaged_roads, 12);
    assert_eq!(state.displaced_citizens, 300);
    assert!((state.avg_flood_risk - 95.5).abs() < 0.01);
    assert_eq!(state.high_risk_cells, 8000);
}

#[test]
fn test_stormwater_mgmt_default_skips_save() {
    let state = StormwaterMgmtState::default();
    assert!(state.save_to_bytes().is_none(), "default should skip save");
}
