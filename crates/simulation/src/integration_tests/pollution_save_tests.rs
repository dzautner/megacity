//! Integration tests for pollution grid save/load (issue #730).
//!
//! Verifies that air (PollutionGrid), noise (NoisePollutionGrid), and water
//! (WaterPollutionGrid) pollution grids all roundtrip correctly through
//! the SaveableRegistry.

use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::water_pollution::WaterPollutionGrid;
use crate::SaveableRegistry;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers: round-trip a grid through the SaveableRegistry
// ---------------------------------------------------------------------------

fn round_trip_pollution_grid(grid: &PollutionGrid) -> PollutionGrid {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();
    app.insert_resource(PollutionGrid {
        levels: grid.levels.clone(),
        width: grid.width,
        height: grid.height,
    });

    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<PollutionGrid>();
    }

    // Save
    let extensions: BTreeMap<String, Vec<u8>> = {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        let ext = registry.save_all(app.world());
        app.world_mut().insert_resource(registry);
        ext
    };

    // Reset to default
    app.insert_resource(PollutionGrid::default());

    // Load
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &extensions);
        app.world_mut().insert_resource(registry);
    }

    let restored = app.world().resource::<PollutionGrid>();
    PollutionGrid {
        levels: restored.levels.clone(),
        width: restored.width,
        height: restored.height,
    }
}

fn round_trip_noise_grid(grid: &NoisePollutionGrid) -> NoisePollutionGrid {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();
    app.insert_resource(NoisePollutionGrid {
        levels: grid.levels.clone(),
        width: grid.width,
        height: grid.height,
    });

    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<NoisePollutionGrid>();
    }

    // Save
    let extensions: BTreeMap<String, Vec<u8>> = {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        let ext = registry.save_all(app.world());
        app.world_mut().insert_resource(registry);
        ext
    };

    // Reset to default
    app.insert_resource(NoisePollutionGrid::default());

    // Load
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &extensions);
        app.world_mut().insert_resource(registry);
    }

    let restored = app.world().resource::<NoisePollutionGrid>();
    NoisePollutionGrid {
        levels: restored.levels.clone(),
        width: restored.width,
        height: restored.height,
    }
}

fn round_trip_water_grid(grid: &WaterPollutionGrid) -> WaterPollutionGrid {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();
    app.insert_resource(WaterPollutionGrid {
        levels: grid.levels.clone(),
        width: grid.width,
        height: grid.height,
    });

    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<WaterPollutionGrid>();
    }

    // Save
    let extensions: BTreeMap<String, Vec<u8>> = {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        let ext = registry.save_all(app.world());
        app.world_mut().insert_resource(registry);
        ext
    };

    // Reset to default
    app.insert_resource(WaterPollutionGrid::default());

    // Load
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &extensions);
        app.world_mut().insert_resource(registry);
    }

    let restored = app.world().resource::<WaterPollutionGrid>();
    WaterPollutionGrid {
        levels: restored.levels.clone(),
        width: restored.width,
        height: restored.height,
    }
}

// ---------------------------------------------------------------------------
// Air pollution tests
// ---------------------------------------------------------------------------

/// Air pollution grid with non-zero values roundtrips through save/load.
#[test]
fn test_pollution_save_air_grid_roundtrips() {
    let mut grid = PollutionGrid::default();
    grid.set(10, 20, 75);
    grid.set(100, 50, 200);
    grid.set(0, 0, 1);
    grid.set(255, 255, 255);

    let restored = round_trip_pollution_grid(&grid);

    assert_eq!(restored.get(10, 20), 75);
    assert_eq!(restored.get(100, 50), 200);
    assert_eq!(restored.get(0, 0), 1);
    assert_eq!(restored.get(255, 255), 255);
}

/// Air pollution grid preserves zero cells alongside non-zero cells.
#[test]
fn test_pollution_save_air_grid_zeros_preserved() {
    let mut grid = PollutionGrid::default();
    grid.set(5, 5, 42);

    let restored = round_trip_pollution_grid(&grid);

    assert_eq!(restored.get(5, 5), 42);
    assert_eq!(restored.get(5, 6), 0, "Adjacent cell should remain zero");
    assert_eq!(restored.get(128, 128), 0, "Distant cell should remain zero");
}

/// Default air pollution grid produces Some from save_to_bytes (grids always save).
#[test]
fn test_pollution_save_air_default_saves() {
    use crate::Saveable;
    let grid = PollutionGrid::default();
    assert!(grid.save_to_bytes().is_some());
}

// ---------------------------------------------------------------------------
// Noise pollution tests
// ---------------------------------------------------------------------------

/// Noise pollution grid roundtrips through save/load.
#[test]
fn test_pollution_save_noise_grid_roundtrips() {
    let mut grid = NoisePollutionGrid::default();
    grid.set(15, 30, 80);
    grid.set(200, 100, 55);
    grid.set(0, 0, 100);

    let restored = round_trip_noise_grid(&grid);

    assert_eq!(restored.get(15, 30), 80);
    assert_eq!(restored.get(200, 100), 55);
    assert_eq!(restored.get(0, 0), 100);
}

/// Noise pollution grid preserves the full range of values 0-100.
#[test]
fn test_pollution_save_noise_grid_value_range() {
    let mut grid = NoisePollutionGrid::default();
    // Set various values across the valid range
    grid.set(1, 1, 0);
    grid.set(2, 2, 50);
    grid.set(3, 3, 100);

    let restored = round_trip_noise_grid(&grid);

    assert_eq!(restored.get(1, 1), 0);
    assert_eq!(restored.get(2, 2), 50);
    assert_eq!(restored.get(3, 3), 100);
}

/// Noise pollution grid dimensions are preserved across save/load.
#[test]
fn test_pollution_save_noise_grid_dimensions_preserved() {
    let grid = NoisePollutionGrid::default();
    let restored = round_trip_noise_grid(&grid);

    assert_eq!(restored.width, grid.width);
    assert_eq!(restored.height, grid.height);
    assert_eq!(restored.levels.len(), grid.levels.len());
}

// ---------------------------------------------------------------------------
// Water pollution tests
// ---------------------------------------------------------------------------

/// Water pollution grid roundtrips through save/load.
#[test]
fn test_pollution_save_water_grid_roundtrips() {
    let mut grid = WaterPollutionGrid::default();
    grid.set(50, 50, 120);
    grid.set(10, 200, 255);
    grid.set(128, 128, 33);

    let restored = round_trip_water_grid(&grid);

    assert_eq!(restored.get(50, 50), 120);
    assert_eq!(restored.get(10, 200), 255);
    assert_eq!(restored.get(128, 128), 33);
}

/// Water pollution grid preserves max value (255).
#[test]
fn test_pollution_save_water_grid_max_value() {
    let mut grid = WaterPollutionGrid::default();
    grid.set(0, 0, 255);

    let restored = round_trip_water_grid(&grid);

    assert_eq!(restored.get(0, 0), 255);
}

/// Water pollution grid dimensions are preserved across save/load.
#[test]
fn test_pollution_save_water_grid_dimensions_preserved() {
    let grid = WaterPollutionGrid::default();
    let restored = round_trip_water_grid(&grid);

    assert_eq!(restored.width, grid.width);
    assert_eq!(restored.height, grid.height);
    assert_eq!(restored.levels.len(), grid.levels.len());
}

// ---------------------------------------------------------------------------
// Cross-grid tests
// ---------------------------------------------------------------------------

/// All three pollution grids can be saved and loaded in the same registry.
#[test]
fn test_pollution_save_all_three_grids_coexist() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    // Set up non-default pollution values
    let mut air = PollutionGrid::default();
    air.set(10, 10, 42);
    app.insert_resource(air);

    let mut noise = NoisePollutionGrid::default();
    noise.set(20, 20, 88);
    app.insert_resource(noise);

    let mut water = WaterPollutionGrid::default();
    water.set(30, 30, 150);
    app.insert_resource(water);

    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<PollutionGrid>();
        registry.register::<NoisePollutionGrid>();
        registry.register::<WaterPollutionGrid>();
    }

    // Save all
    let extensions: BTreeMap<String, Vec<u8>> = {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        let ext = registry.save_all(app.world());
        app.world_mut().insert_resource(registry);
        ext
    };

    // Verify all three keys are present
    assert!(
        extensions.contains_key("pollution_grid"),
        "Air pollution should be saved"
    );
    assert!(
        extensions.contains_key("noise_grid"),
        "Noise pollution should be saved"
    );
    assert!(
        extensions.contains_key("water_pollution_grid"),
        "Water pollution should be saved"
    );

    // Reset all to default
    app.insert_resource(PollutionGrid::default());
    app.insert_resource(NoisePollutionGrid::default());
    app.insert_resource(WaterPollutionGrid::default());

    // Load all
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &extensions);
        app.world_mut().insert_resource(registry);
    }

    // Verify all values restored
    assert_eq!(app.world().resource::<PollutionGrid>().get(10, 10), 42);
    assert_eq!(
        app.world().resource::<NoisePollutionGrid>().get(20, 20),
        88
    );
    assert_eq!(
        app.world().resource::<WaterPollutionGrid>().get(30, 30),
        150
    );
}

/// Sequential save A / load B / load A restores correct state for all grids.
#[test]
fn test_pollution_save_sequential_load_a_b_a() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();
    app.init_resource::<PollutionGrid>();
    app.init_resource::<NoisePollutionGrid>();
    app.init_resource::<WaterPollutionGrid>();

    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<PollutionGrid>();
        registry.register::<NoisePollutionGrid>();
        registry.register::<WaterPollutionGrid>();
    }

    // Build save A with specific values
    {
        let mut air = app.world_mut().resource_mut::<PollutionGrid>();
        air.set(5, 5, 99);
        let mut noise = app.world_mut().resource_mut::<NoisePollutionGrid>();
        noise.set(6, 6, 77);
        let mut water = app.world_mut().resource_mut::<WaterPollutionGrid>();
        water.set(7, 7, 200);
    }
    let save_a = {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        let ext = registry.save_all(app.world());
        app.world_mut().insert_resource(registry);
        ext
    };

    // Build save B with different values
    {
        let mut air = app.world_mut().resource_mut::<PollutionGrid>();
        air.set(5, 5, 10);
        let mut noise = app.world_mut().resource_mut::<NoisePollutionGrid>();
        noise.set(6, 6, 20);
        let mut water = app.world_mut().resource_mut::<WaterPollutionGrid>();
        water.set(7, 7, 30);
    }
    let save_b = {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        let ext = registry.save_all(app.world());
        app.world_mut().insert_resource(registry);
        ext
    };

    // Load save B
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &save_b);
        app.world_mut().insert_resource(registry);
    }
    assert_eq!(app.world().resource::<PollutionGrid>().get(5, 5), 10);
    assert_eq!(
        app.world().resource::<NoisePollutionGrid>().get(6, 6),
        20
    );
    assert_eq!(
        app.world().resource::<WaterPollutionGrid>().get(7, 7),
        30
    );

    // Load save A again -- should restore original values
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &save_a);
        app.world_mut().insert_resource(registry);
    }
    assert_eq!(app.world().resource::<PollutionGrid>().get(5, 5), 99);
    assert_eq!(
        app.world().resource::<NoisePollutionGrid>().get(6, 6),
        77
    );
    assert_eq!(
        app.world().resource::<WaterPollutionGrid>().get(7, 7),
        200
    );
}
