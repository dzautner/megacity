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

// ====================================================================
// Helpers
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

fn ignite_forest_fire(city: &mut TestCity, x: usize, y: usize, intensity: u8) {
    let world = city.world_mut();
    let mut ff_grid = world.resource_mut::<ForestFireGrid>();
    ff_grid.set(x, y, intensity);
}

fn count_forest_fires(city: &mut TestCity) -> u32 {
    let world = city.world_mut();
    let ff_grid = world.resource::<ForestFireGrid>();
    ff_grid.intensities.iter().filter(|&&v| v > 0).count() as u32
}

fn get_fire_intensity(city: &mut TestCity, x: usize, y: usize) -> u8 {
    let world = city.world_mut();
    let ff_grid = world.resource::<ForestFireGrid>();
    ff_grid.get(x, y)
}

/// Set weather by configuring atmospheric state so the weather system
/// derives the desired condition even after hourly updates.
fn set_weather_persistent(city: &mut TestCity, condition: WeatherCondition, temperature: f32) {
    let world = city.world_mut();
    let mut weather = world.resource_mut::<Weather>();
    weather.current_event = condition;
    weather.temperature = temperature;
    match condition {
        WeatherCondition::Storm => {
            weather.cloud_cover = 0.95;
            weather.atmo_precipitation = 0.85;
            weather.humidity = 0.95;
            weather.event_days_remaining = 100;
        }
        WeatherCondition::Rain => {
            weather.cloud_cover = 0.75;
            weather.atmo_precipitation = 0.3;
            weather.humidity = 0.85;
            weather.event_days_remaining = 100;
        }
        WeatherCondition::Sunny => {
            weather.cloud_cover = 0.1;
            weather.atmo_precipitation = 0.0;
            weather.humidity = 0.3;
            weather.event_days_remaining = 0;
        }
        _ => {}
    }
}

// ====================================================================
// Test 1: Forest fire spreads to adjacent cells with trees
// ====================================================================

#[test]
fn test_fire_spreads_to_adjacent_tree_cells() {
    let mut city = TestCity::new();

    plant_trees(&mut city, 128, 128, 3);
    set_weather_persistent(&mut city, WeatherCondition::Sunny, 25.0);
    ignite_forest_fire(&mut city, 128, 128, 200);

    // 200 ticks = 20 fire updates (FIRE_UPDATE_INTERVAL = 10)
    city.tick(200);

    let total_fires = count_forest_fires(&mut city);
    assert!(
        total_fires > 1,
        "Fire should spread to adjacent tree cells, but only {} cell(s) burning",
        total_fires
    );

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

#[test]
fn test_fire_does_not_spread_across_water() {
    use crate::grid::CellType;

    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut tree_grid = world.resource_mut::<TreeGrid>();
        for y in 126..=130 {
            for x in 125..=127 {
                tree_grid.set(x, y, true);
            }
            for x in 129..=131 {
                tree_grid.set(x, y, true);
            }
        }
    }
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        for y in 126..=130 {
            grid.get_mut(128, y).cell_type = CellType::Water;
        }
    }

    set_weather_persistent(&mut city, WeatherCondition::Sunny, 25.0);
    ignite_forest_fire(&mut city, 126, 128, 200);

    city.tick(300);

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

#[test]
fn test_building_fire_spreads_to_adjacent_buildings() {
    use bevy::prelude::*;

    let mut city = TestCity::new()
        .with_building(80, 80, ZoneType::Industrial, 1)
        .with_building(81, 80, ZoneType::Industrial, 1)
        .with_building(80, 81, ZoneType::Industrial, 1)
        .with_building(81, 81, ZoneType::Industrial, 1);

    // Set first building on fire
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
                    intensity: 30.0,
                    ticks_burning: 0,
                });
            }
        }
    }

    // 190 ticks < DESTRUCTION_TICK_THRESHOLD (200)
    city.tick(190);

    let mut spread_count = 0;
    {
        let world = city.world_mut();
        let mut query = world.query::<(&Building, &OnFire)>();
        for (b, _) in query.iter(world) {
            if !(b.grid_x == 80 && b.grid_y == 80) {
                spread_count += 1;
            }
        }
    }

    assert!(
        spread_count > 0,
        "Fire should have spread to at least one adjacent building after 190 ticks"
    );
}

// ====================================================================
// Test 4: Fire station coverage reduces building fire intensity
// ====================================================================

/// With fire station coverage, the `extinguish_fires` system reduces
/// intensity by COVERAGE_REDUCTION_PER_TICK (2.0) each tick. For a
/// small initial fire (intensity 5), this outruns the growth formula
/// (ticks_burning * 0.5) and extinguishes the fire within a few ticks.
#[test]
fn test_fire_station_coverage_extinguishes_building_fire() {
    use bevy::prelude::*;

    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::FireStation)
        .with_building(105, 100, ZoneType::Industrial, 1);

    // Run one slow cycle to compute service coverage
    city.tick_slow_cycles(1);

    // Verify fire coverage
    {
        let cov = city.resource::<ServiceCoverageGrid>();
        let idx = ServiceCoverageGrid::idx(105, 100);
        assert!(
            cov.has_fire(idx),
            "Building at (105,100) should have fire station coverage"
        );
    }

    // Set a small fire. With intensity=5, ticks_burning=0:
    //   Tick 1: growth=max(0.5, 5)=5, then 5-2=3
    //   Tick 2: growth=max(1.0, 3)=3, then 3-2=1
    //   Tick 3: growth=max(1.5, 1)=1.5, then 1.5-2=-0.5 -> extinguished
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
                    intensity: 5.0,
                    ticks_burning: 0,
                });
            }
        }
    }

    // Run 10 ticks — well beyond the 3 needed for extinguishment
    city.tick(10);

    let mut still_burning = false;
    {
        let world = city.world_mut();
        let mut query = world.query::<(&Building, &OnFire)>();
        for (b, _) in query.iter(world) {
            if b.grid_x == 105 && b.grid_y == 100 {
                still_burning = true;
            }
        }
    }
    assert!(
        !still_burning,
        "Small fire in building with fire coverage should be extinguished"
    );
}

// ====================================================================
// Test 5: Building WITHOUT fire coverage stays on fire
// ====================================================================

#[test]
fn test_building_without_fire_coverage_stays_on_fire() {
    use bevy::prelude::*;

    let mut city = TestCity::new()
        .with_building(200, 200, ZoneType::Industrial, 1);

    city.tick_slow_cycles(1);

    {
        let cov = city.resource::<ServiceCoverageGrid>();
        let idx = ServiceCoverageGrid::idx(200, 200);
        assert!(!cov.has_fire(idx), "Should NOT have fire coverage");
    }

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

    city.tick(30);

    {
        let world = city.world_mut();
        let mut query = world.query::<(&Building, &OnFire)>();
        let mut found = false;
        for (b, fire) in query.iter(world) {
            if b.grid_x == 200 && b.grid_y == 200 {
                found = true;
                assert!(
                    fire.intensity >= 10.0,
                    "Without coverage, intensity should not decrease. Got {}",
                    fire.intensity
                );
                assert!(fire.ticks_burning > 0, "ticks_burning should increment");
            }
        }
        assert!(found, "Building should still be on fire without coverage");
    }
}

// ====================================================================
// Test 6: Rain weather suppresses forest fire intensity
// ====================================================================

#[test]
fn test_rain_suppresses_forest_fire_intensity() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut tree_grid = world.resource_mut::<TreeGrid>();
        tree_grid.set(128, 128, true);
    }

    set_weather_persistent(&mut city, WeatherCondition::Rain, 15.0);
    ignite_forest_fire(&mut city, 128, 128, 50);

    // Re-apply weather between tick batches to survive hourly updates
    for _ in 0..10 {
        set_weather_persistent(&mut city, WeatherCondition::Rain, 15.0);
        city.tick(10);
    }

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

#[test]
fn test_storm_extinguishes_forest_fire_quickly() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut tree_grid = world.resource_mut::<TreeGrid>();
        tree_grid.set(128, 128, true);
    }

    set_weather_persistent(&mut city, WeatherCondition::Storm, 15.0);
    ignite_forest_fire(&mut city, 128, 128, 40);

    for _ in 0..5 {
        set_weather_persistent(&mut city, WeatherCondition::Storm, 15.0);
        city.tick(10);
    }

    let intensity = get_fire_intensity(&mut city, 128, 128);
    assert_eq!(
        intensity, 0,
        "Storm should fully extinguish a moderate forest fire. Remaining: {}",
        intensity
    );
}
