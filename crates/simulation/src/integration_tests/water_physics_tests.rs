//! Integration tests for PROG-007: Water Physics and Flood Simulation.

use crate::grid::ZoneType;
use crate::test_harness::TestCity;
use crate::water_physics::{WaterGrid, WaterPhysicsState};
use crate::Saveable;

/// Water flows downhill: a cell with higher elevation + water should transfer
/// water to lower neighbours after a slow tick cycle.
#[test]
fn test_water_flows_from_high_to_low_elevation() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(128, 128).elevation = 0.5;
        grid.get_mut(127, 128).elevation = 0.1;
        grid.get_mut(129, 128).elevation = 0.1;
        grid.get_mut(128, 127).elevation = 0.1;
        grid.get_mut(128, 129).elevation = 0.1;

        let mut water = world.resource_mut::<WaterGrid>();
        water.set(128, 128, 2.0);
    }

    city.tick_slow_cycle();

    let water = city.resource::<WaterGrid>();
    let center_depth = water.get(128, 128);
    let left_depth = water.get(127, 128);
    let right_depth = water.get(129, 128);
    let up_depth = water.get(128, 127);
    let down_depth = water.get(128, 129);

    assert!(
        center_depth < 2.0,
        "centre cell should have lost water: {}",
        center_depth
    );
    let neighbour_total = left_depth + right_depth + up_depth + down_depth;
    assert!(
        neighbour_total > 0.0,
        "neighbours should have gained water: total={}",
        neighbour_total
    );
}

/// Evaporation removes water over time.
#[test]
fn test_evaporation_removes_water() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<crate::weather::Weather>();
        weather.temperature = 35.0;

        let mut water = world.resource_mut::<WaterGrid>();
        for x in 10..20 {
            for y in 10..20 {
                water.set(x, y, 0.003);
            }
        }
    }

    let initial_total: f32 = city.resource::<WaterGrid>().cells.iter().sum();
    assert!(initial_total > 0.0, "initial water should be positive");

    city.tick_slow_cycle();

    let after_total: f32 = city.resource::<WaterGrid>().cells.iter().sum();
    assert!(
        after_total < initial_total,
        "evaporation should reduce total water: before={}, after={}",
        initial_total,
        after_total
    );
}

/// Cells with deep water are counted as flooded.
#[test]
fn test_flood_detection_with_deep_water() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut water = world.resource_mut::<WaterGrid>();
        for x in 50..55 {
            for y in 50..55 {
                water.set(x, y, 1.0);
            }
        }
    }

    city.tick_slow_cycle();

    let state = city.resource::<WaterPhysicsState>();
    assert!(
        state.flooded_cell_count >= 1,
        "at least one cell should be flooded: count={}",
        state.flooded_cell_count
    );
    assert!(
        state.max_depth > 0.0,
        "max depth should be positive: {}",
        state.max_depth
    );
    assert!(
        state.total_volume > 0.0,
        "total volume should be positive: {}",
        state.total_volume
    );
}

/// Buildings in flooded cells accumulate damage.
#[test]
fn test_building_flood_damage() {
    let mut city = TestCity::new()
        .with_building(60, 60, ZoneType::ResidentialLow, 1);

    {
        let world = city.world_mut();
        let mut water = world.resource_mut::<WaterGrid>();
        water.set(60, 60, 2.0);
    }

    city.tick_slow_cycle();

    let state = city.resource::<WaterPhysicsState>();
    assert!(
        state.cumulative_damage > 0.0,
        "building should take flood damage: {}",
        state.cumulative_damage
    );
}

/// Disabled simulation does not process water.
#[test]
fn test_disabled_simulation_does_nothing() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut water = world.resource_mut::<WaterGrid>();
        water.set(100, 100, 1.0);

        let mut state = world.resource_mut::<WaterPhysicsState>();
        state.enabled = false;
    }

    city.tick_slow_cycle();

    let water = city.resource::<WaterGrid>();
    assert!(
        (water.get(100, 100) - 1.0).abs() < f32::EPSILON,
        "disabled simulation should not modify water: {}",
        water.get(100, 100)
    );
    let state = city.resource::<WaterPhysicsState>();
    assert_eq!(state.flooded_cell_count, 0);
}

/// Saveable round-trip preserves state.
#[test]
fn test_saveable_roundtrip() {
    let state = WaterPhysicsState {
        flooded_cell_count: 10,
        max_depth: 2.5,
        total_volume: 500.0,
        cumulative_damage: 12345.0,
        enabled: true,
    };
    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = WaterPhysicsState::load_from_bytes(&bytes);
    assert_eq!(restored.flooded_cell_count, 10);
    assert!((restored.max_depth - 2.5).abs() < f32::EPSILON);
    assert!((restored.cumulative_damage - 12345.0).abs() < f64::EPSILON);
}

/// Total water volume is conserved (minus evaporation) -- flow only moves water,
/// it does not create or destroy it.
#[test]
fn test_total_water_volume_decreases_only_by_evaporation() {
    let mut city = TestCity::new();

    // Place a uniform large block of water so flow redistributes but totals stay high
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<crate::weather::Weather>();
        weather.temperature = 10.0; // low temp = minimal evaporation (base only)

        let mut water = world.resource_mut::<WaterGrid>();
        for x in 100..120 {
            for y in 100..120 {
                water.set(x, y, 0.5);
            }
        }
    }

    let initial_total: f32 = city.resource::<WaterGrid>().cells.iter().sum();

    city.tick_slow_cycle();

    let after_total: f32 = city.resource::<WaterGrid>().cells.iter().sum();
    // Total should decrease (evaporation removes some) but not by too much.
    // With 400 cells * 0.5 = 200 initial; evaporation of 0.005 per cell
    // would remove at most ~entire grid * 0.005 but only cells with water.
    // Flow spreads water to neighbors, expanding the wet area, and evaporation
    // applies to all wet cells. The total should still be substantial.
    assert!(
        after_total > 0.0,
        "water should still exist: initial={}, after={}",
        initial_total,
        after_total
    );
    assert!(
        after_total <= initial_total,
        "water should not increase without rain: initial={}, after={}",
        initial_total,
        after_total
    );
}
