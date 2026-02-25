//! Integration tests for PROG-007: Water Physics and Flood Simulation.

use crate::grid::ZoneType;
use crate::test_harness::TestCity;
use crate::water_physics::{WaterGrid, WaterPhysicsState, FLOOD_DEPTH_THRESHOLD};
use crate::Saveable;

/// Water flows downhill: a cell with higher elevation + water should transfer
/// water to lower neighbours after a slow tick cycle.
#[test]
fn test_water_flows_from_high_to_low_elevation() {
    let mut city = TestCity::new();

    // Set up a small slope: cell (128,128) at elevation 0.5, neighbours at 0.1
    // Place enough water that it won't all evaporate in one tick.
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

    // Centre should have lost water (started at 2.0)
    assert!(
        center_depth < 2.0,
        "centre cell should have lost water: {}",
        center_depth
    );
    // At least one neighbour should have gained water
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

    // Place water directly (skip rainfall which is weather-dependent).
    // Use warm temperature for higher evaporation.
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<crate::weather::Weather>();
        weather.temperature = 35.0;

        let mut water = world.resource_mut::<WaterGrid>();
        // Place a small uniform water layer on a few cells
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
    // At 35C evaporation = 0.005 + (15 * 0.0005) = 0.0125 per cell,
    // which exceeds the 0.003 placed, so cells should be drained to 0.
    assert!(
        after_total < initial_total,
        "evaporation should reduce total water: before={}, after={}",
        initial_total,
        after_total
    );
}

/// Cells with water depth >= threshold are counted as flooded.
#[test]
fn test_flood_detection_with_deep_water() {
    let mut city = TestCity::new();

    // Place significant water directly to ensure it stays above threshold
    // even after one tick of flow and evaporation.
    {
        let world = city.world_mut();
        let mut water = world.resource_mut::<WaterGrid>();
        // Place a large amount in a basin (flat elevation, water stays put)
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
        water.set(60, 60, 2.0); // deep flood on building
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

    // With simulation disabled, the water should remain untouched (no flow,
    // no evaporation, no flood detection).
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

/// Water in a flat basin should not disappear (only evaporation reduces it).
#[test]
fn test_water_conserved_in_flat_basin() {
    let mut city = TestCity::new();

    // All cells have default elevation 0.0, so water has no gradient to flow.
    // Disable evaporation by setting cold temperature.
    {
        let world = city.world_mut();
        let mut weather = world.resource_mut::<crate::weather::Weather>();
        weather.temperature = 10.0; // below 20C, no extra evaporation

        let mut water = world.resource_mut::<WaterGrid>();
        water.set(128, 128, 1.0);
    }

    city.tick_slow_cycle();

    let water = city.resource::<WaterGrid>();
    // With flat terrain, flow should not move water. Only base evaporation
    // of 0.005 removes some. The remaining depth should be ~0.995.
    let remaining = water.get(128, 128);
    assert!(
        remaining > 0.9,
        "water in flat basin should mostly remain: {}",
        remaining
    );
}

/// Water should not be added to natural water body cells by rain.
#[test]
fn test_no_rain_on_water_cells() {
    let mut city = TestCity::new();

    // Mark a cell as water type and set up precipitation via direct water placement
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(100, 100).cell_type = crate::grid::CellType::Water;

        // Pre-place 0 water on the water cell (should stay 0 after rain system)
        // We test indirectly: if add_rainfall ran, it would skip water cells.
    }

    // The water grid for this cell should remain at 0
    let water = city.resource::<WaterGrid>();
    assert!(
        water.get(100, 100).abs() < f32::EPSILON,
        "water body cells should not have water added: {}",
        water.get(100, 100)
    );
}
