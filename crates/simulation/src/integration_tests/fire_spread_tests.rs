//! TEST-048: Integration tests for fire spread mechanics.
//!
//! Tests cover:
//! - Fire spreads to adjacent cells (forest fire)
//! - Fire does not spread across water cells
//! - Building fire spreads between adjacent buildings
//! - Fire station coverage reduces/extinguishes building fires
//! - Buildings without coverage stay on fire
//! - Rain/storm weather suppresses forest fire

use crate::fire::OnFire;
use crate::forest_fire::ForestFireGrid;
use crate::buildings::Building;
use crate::grid::{WorldGrid, ZoneType};
use crate::happiness::ServiceCoverageGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::trees::TreeGrid;
use crate::weather::{Weather, WeatherCondition};
use crate::TickCounter;

// ====================================================================
// Helper: set up a patch of trees at (cx, cy) with given radius
// ====================================================================

fn plant_trees(city: &mut TestCity, cx: usize, cy: usize, radius: usize) {
    let world = city.world_mut();
    let mut tree_grid = world.resource_mut::<TreeGrid>();
    for dy in 0..=(radius * 2) {
        for dx in 0..=(radius * 2) {
            let x = cx.saturating_sub(radius) + dx;
            let y = cy.saturating_sub(radius) + dy;
            if x < 256 && y < 256 {
                tree_grid.set(x, y, true);
            }
        }
    }
}

/// Ignite a forest fire cell at (x, y) with given intensity.
fn ignite_forest_fire(city: &mut TestCity, x: usize, y: usize, intensity: u8) {
    let world = city.world_mut();
    let mut ff_grid = world.resource_mut::<ForestFireGrid>();
    ff_grid.set(x, y, intensity);
}

/// Count total cells with active forest fire.
fn count_forest_fires(city: &mut TestCity) -> u32 {
    let world = city.world_mut();
    let ff_grid = world.resource::<ForestFireGrid>();
    ff_grid
        .intensities
        .iter()
        .filter(|&&v| v > 0)
        .count() as u32
}

/// Get the forest fire intensity at a specific cell.
fn get_fire_intensity(city: &mut TestCity, x: usize, y: usize) -> u8 {
    let world = city.world_mut();
    let ff_grid = world.resource::<ForestFireGrid>();
    ff_grid.get(x, y)
}

/// Set weather conditions directly for deterministic testing.
fn set_weather(city: &mut TestCity, condition: WeatherCondition, temperature: f32) {
    let world = city.world_mut();
    let mut weather = world.resource_mut::<Weather>();
    weather.current_event = condition;
    weather.temperature = temperature;
}

// ====================================================================
// Test 1: Forest fire spreads to adjacent cells with trees
// ====================================================================

/// A burning cell with trees should spread fire to its 8-connected neighbors
/// that also have trees, after enough ticks.
#[test]
fn test_fire_spreads_to_adjacent_tree_cells() {
    let mut city = TestCity::new();

    // Set up: plant a 7x7 patch of trees centered at (128, 128)
    plant_trees(&mut city, 128, 128, 3);

    // Set calm, dry weather so rain/storm don't interfere
    set_weather(&mut city, WeatherCondition::Sunny, 25.0);

    // Ignite the center cell with high intensity to maximize spread chance
    ignite_forest_fire(&mut city, 128, 128, 200);

    // Run enough ticks for the fire system to execute multiple times.
    // FIRE_UPDATE_INTERVAL = 10 ticks, so 200 ticks = 20 fire updates.
    city.tick(200);

    // After 20 fire update cycles, fire should have spread beyond the origin.
    let total_fires = count_forest_fires(&mut city);
    assert!(
        total_fires > 1,
        "Fire should spread to adjacent tree cells, but only {} cell(s) are burning",
        total_fires
    );

    // Verify at least one neighbor of the center is also on fire
    let mut neighbor_on_fire = false;
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = (128i32 + dx) as usize;
            let ny = (128i32 + dy) as usize;
            if get_fire_intensity(&mut city, nx, ny) > 0 {
                neighbor_on_fire = true;
            }
        }
    }
    assert!(
        neighbor_on_fire,
        "At least one 8-connected neighbor of the ignition point should be burning"
    );
}

// ====================================================================
// Test 2: Fire does not spread across water cells
// ====================================================================

/// Water cells should block fire spread entirely. A fire adjacent to water
/// should not cross to the other side.
#[test]
fn test_fire_does_not_spread_across_water() {
    use crate::grid::CellType;

    let mut city = TestCity::new();

    // Plant trees on two sides separated by a water column.
    // Need to avoid double mutable borrow by doing tree_grid first, then grid.
    {
        let world = city.world_mut();
        let mut tree_grid = world.resource_mut::<TreeGrid>();
        for y in 126..=130 {
            // Left side: trees at x=125..127
            for x in 125..=127 {
                tree_grid.set(x, y, true);
            }
            // Right side: trees at x=129..131
            for x in 129..=131 {
                tree_grid.set(x, y, true);
            }
        }
    }
    {
        // Water barrier at x=128, y=126..130
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        for y in 126..=130 {
            grid.get_mut(128, y).cell_type = CellType::Water;
        }
    }

    set_weather(&mut city, WeatherCondition::Sunny, 25.0);

    // Ignite center of left patch
    ignite_forest_fire(&mut city, 126, 128, 200);

    // Run 300 ticks (30 fire updates)
    city.tick(300);

    // Check that no cell on the right side of the water barrier has fire
    let mut right_side_fire = false;
    for y in 126..=130 {
        for x in 129..=131 {
            if get_fire_intensity(&mut city, x, y) > 0 {
                right_side_fire = true;
            }
        }
    }
    assert!(
        !right_side_fire,
        "Fire should not spread across water barrier cells"
    );
}

// ====================================================================
// Test 3: Building fire spreads between adjacent buildings
// ====================================================================

/// When a building is on fire (OnFire component), the fire should spread
/// to adjacent buildings via the spread_fire system.
#[test]
fn test_building_fire_spreads_to_adjacent_buildings() {
    use bevy::prelude::*;

    let mut city = TestCity::new()
        .with_building(80, 80, ZoneType::Industrial, 1)
        .with_building(81, 80, ZoneType::Industrial, 1);

    // Manually add OnFire to the first building
    {
        let world = city.world_mut();
        let mut query = world.query::<(Entity, &Building)>();
        let entities: Vec<(Entity, usize, usize)> = query
            .iter(world)
            .map(|(e, b)| (e, b.grid_x, b.grid_y))
            .collect();
        for (entity, gx, gy) in entities {
            if gx == 80 && gy == 80 {
                world.entity_mut(entity).insert(OnFire {
                    intensity: 50.0,
                    ticks_burning: 100,
                });
            }
        }
    }

    // Run many ticks. spread_fire has SPREAD_CHANCE = 0.05 per tick per neighbor,
    // so we need many ticks for the stochastic spread to trigger.
    city.tick(500);

    // Check if the second building caught fire
    let mut second_building_on_fire = false;
    {
        let world = city.world_mut();
        let mut query = world.query::<(&Building, &OnFire)>();
        for (b, _fire) in query.iter(world) {
            if b.grid_x == 81 && b.grid_y == 80 {
                second_building_on_fire = true;
            }
        }
    }

    // Due to randomness (5% per tick), after 500 ticks the probability of
    // NOT spreading is (0.95)^500 which is essentially 0.
    assert!(
        second_building_on_fire,
        "Fire should have spread to the adjacent building after 500 ticks"
    );
}

// ====================================================================
// Test 4: Fire station coverage reduces/extinguishes building fires
// ====================================================================

/// Buildings within fire station coverage should have their fire intensity
/// reduced and eventually extinguished by the extinguish_fires system.
#[test]
fn test_fire_station_coverage_extinguishes_building_fire() {
    use bevy::prelude::*;

    // Place a fire station at (100, 100) and a building within its coverage
    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::FireStation)
        .with_building(105, 100, ZoneType::Industrial, 1);

    // Run one slow cycle to compute service coverage
    city.tick_slow_cycles(1);

    // Verify the building cell has fire coverage
    {
        let cov = city.resource::<ServiceCoverageGrid>();
        let idx = ServiceCoverageGrid::idx(105, 100);
        assert!(
            cov.has_fire(idx),
            "Building at (105,100) should be within fire station coverage"
        );
    }

    // Set the building on fire
    {
        let world = city.world_mut();
        let mut query = world.query::<(Entity, &Building)>();
        let entities: Vec<(Entity, usize, usize)> = query
            .iter(world)
            .map(|(e, b)| (e, b.grid_x, b.grid_y))
            .collect();
        for (entity, gx, gy) in entities {
            if gx == 105 && gy == 100 {
                world.entity_mut(entity).insert(OnFire {
                    intensity: 20.0,
                    ticks_burning: 0,
                });
            }
        }
    }

    // COVERAGE_REDUCTION_PER_TICK = 2.0, so after ~10 ticks intensity 20 -> 0
    city.tick(15);

    // The building should no longer be on fire
    {
        let world = city.world_mut();
        let mut query = world.query::<(&Building, &OnFire)>();
        let still_burning: Vec<_> = query
            .iter(world)
            .filter(|(b, _)| b.grid_x == 105 && b.grid_y == 100)
            .collect();
        assert!(
            still_burning.is_empty(),
            "Building within fire coverage should have its fire extinguished"
        );
    }
}

// ====================================================================
// Test 5: Building WITHOUT fire coverage stays on fire longer
// ====================================================================

/// A building outside fire station coverage should NOT have its fire reduced
/// by the extinguish_fires system. Instead, intensity should grow over time.
#[test]
fn test_building_without_fire_coverage_stays_on_fire() {
    use bevy::prelude::*;

    // Place a building far from any fire station
    let mut city = TestCity::new()
        .with_building(200, 200, ZoneType::Industrial, 1);

    // Run a slow cycle (service coverage computes but no fire station exists)
    city.tick_slow_cycles(1);

    // Verify no fire coverage at the building
    {
        let cov = city.resource::<ServiceCoverageGrid>();
        let idx = ServiceCoverageGrid::idx(200, 200);
        assert!(
            !cov.has_fire(idx),
            "Building at (200,200) should NOT have fire coverage"
        );
    }

    // Set the building on fire with moderate intensity
    {
        let world = city.world_mut();
        let mut query = world.query::<(Entity, &Building)>();
        let entities: Vec<(Entity, usize, usize)> = query
            .iter(world)
            .map(|(e, b)| (e, b.grid_x, b.grid_y))
            .collect();
        for (entity, gx, gy) in entities {
            if gx == 200 && gy == 200 {
                world.entity_mut(entity).insert(OnFire {
                    intensity: 10.0,
                    ticks_burning: 0,
                });
            }
        }
    }

    // Run some ticks — without coverage, extinguish_fires does nothing,
    // and spread_fire increases intensity via INTENSITY_GROWTH_RATE.
    city.tick(30);

    // The building should still be on fire and intensity should have grown
    {
        let world = city.world_mut();
        let mut query = world.query::<(&Building, &OnFire)>();
        let mut found = false;
        for (b, fire) in query.iter(world) {
            if b.grid_x == 200 && b.grid_y == 200 {
                found = true;
                assert!(
                    fire.intensity >= 10.0,
                    "Without fire coverage, intensity should not decrease. Got {}",
                    fire.intensity
                );
                assert!(
                    fire.ticks_burning > 0,
                    "ticks_burning should have incremented"
                );
            }
        }
        assert!(
            found,
            "Building at (200,200) should still have OnFire component without fire coverage"
        );
    }
}

// ====================================================================
// Test 6: Rain weather suppresses forest fire intensity
// ====================================================================

/// Forest fire intensity should decrease faster during rain conditions
/// due to the RAIN_REDUCTION constant (8 per fire update).
#[test]
fn test_rain_suppresses_forest_fire_intensity() {
    let mut city = TestCity::new();

    // Plant a single tree and ignite it
    {
        let world = city.world_mut();
        let mut tree_grid = world.resource_mut::<TreeGrid>();
        tree_grid.set(128, 128, true);
    }

    // Set rainy weather
    set_weather(&mut city, WeatherCondition::Rain, 15.0);

    // Ignite with moderate intensity
    ignite_forest_fire(&mut city, 128, 128, 50);

    // Run enough ticks for several fire updates (FIRE_UPDATE_INTERVAL = 10)
    // BURNOUT_RATE = 2, RAIN_REDUCTION = 8, so total reduction per update = 10
    // But intensity also grows by 3 if the tree is still alive and intensity < 200.
    // Net reduction per update in rain = 2 + 8 - 3 = 7 per fire update.
    // Starting at 50, after ~8 fire updates (80 ticks), should be near 0.
    city.tick(100);

    let intensity = get_fire_intensity(&mut city, 128, 128);
    assert!(
        intensity < 50,
        "Rain should reduce fire intensity. Started at 50, now at {}",
        intensity
    );
}

// ====================================================================
// Test 7: Storm weather extinguishes forest fire quickly
// ====================================================================

/// Storm weather should suppress fire even more aggressively than rain.
/// Both RAIN_REDUCTION and STORM_REDUCTION apply since storm is_precipitation.
#[test]
fn test_storm_extinguishes_forest_fire_quickly() {
    let mut city = TestCity::new();

    // Plant a single tree and ignite it
    {
        let world = city.world_mut();
        let mut tree_grid = world.resource_mut::<TreeGrid>();
        tree_grid.set(128, 128, true);
    }

    // Set storm weather
    set_weather(&mut city, WeatherCondition::Storm, 15.0);

    // Ignite with moderate intensity
    ignite_forest_fire(&mut city, 128, 128, 40);

    // BURNOUT_RATE = 2, RAIN_REDUCTION = 8, STORM_REDUCTION = 15
    // Total reduction per fire update = 2 + 8 + 15 - 3 (growth) = 22 net
    // (Storm is_precipitation and is_storm, so both reductions apply)
    // Starting at 40, should be extinguished in ~2 fire updates (20 ticks).
    city.tick(50);

    let intensity = get_fire_intensity(&mut city, 128, 128);
    assert_eq!(
        intensity, 0,
        "Storm should fully extinguish a moderate forest fire. Remaining intensity: {}",
        intensity
    );
}
