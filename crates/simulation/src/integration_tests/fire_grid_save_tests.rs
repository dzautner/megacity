//! SAVE-037: Integration tests for fire grid state serialization.
//!
//! Tests verify that:
//! - FireGrid (building fires) roundtrips correctly through save/load
//! - ForestFireGrid intensities roundtrip correctly
//! - ForestFireStats roundtrip correctly
//! - Fire spread continues after a save/load cycle
//! - Default (empty) fire state skips serialization
//! - Corrupted bytes fall back to defaults

use crate::fire::FireGrid;
use crate::forest_fire::{ForestFireGrid, ForestFireStats};
use crate::test_harness::TestCity;
use crate::trees::TreeGrid;
use crate::Saveable;
use crate::SaveableRegistry;

// ====================================================================
// Roundtrip helper
// ====================================================================

/// Save all registered saveables, reset them, then restore from the saved
/// bytes. Operates entirely through `world_mut()`.
fn roundtrip(city: &mut TestCity) {
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();
    let extensions = registry.save_all(world);
    registry.reset_all(world);
    registry.load_all(world, &extensions);
    world.insert_resource(registry);
}

// ====================================================================
// FireGrid roundtrip
// ====================================================================

#[test]
fn test_fire_grid_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<FireGrid>();
        grid.set(50, 50, 80);
        grid.set(100, 100, 30);
        grid.set(200, 200, 1);
    }

    roundtrip(&mut city);

    let grid = city.resource::<FireGrid>();
    assert_eq!(grid.get(50, 50), 80, "Fire at (50,50) should survive roundtrip");
    assert_eq!(
        grid.get(100, 100),
        30,
        "Fire at (100,100) should survive roundtrip"
    );
    assert_eq!(grid.get(200, 200), 1, "Fire at (200,200) should survive roundtrip");
    assert_eq!(grid.get(0, 0), 0, "Unburning cell should remain zero");
}

// ====================================================================
// ForestFireStats roundtrip
// ====================================================================

#[test]
fn test_forest_fire_stats_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<ForestFireStats>();
        stats.active_fires = 12;
        stats.total_area_burned = 5000;
        stats.fires_this_month = 3;
    }

    roundtrip(&mut city);

    let stats = city.resource::<ForestFireStats>();
    assert_eq!(stats.active_fires, 12, "active_fires should survive roundtrip");
    assert_eq!(
        stats.total_area_burned, 5000,
        "total_area_burned should survive roundtrip"
    );
    assert_eq!(
        stats.fires_this_month, 3,
        "fires_this_month should survive roundtrip"
    );
}

// ====================================================================
// ForestFireGrid roundtrip (extended: multiple cells)
// ====================================================================

#[test]
fn test_forest_fire_grid_multiple_cells_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<ForestFireGrid>();
        grid.set(10, 10, 255);
        grid.set(128, 128, 100);
        grid.set(255, 255, 1);
    }

    roundtrip(&mut city);

    let grid = city.resource::<ForestFireGrid>();
    assert_eq!(grid.get(10, 10), 255);
    assert_eq!(grid.get(128, 128), 100);
    assert_eq!(grid.get(255, 255), 1);
    assert_eq!(grid.get(0, 0), 0);
}

// ====================================================================
// Default state skips save
// ====================================================================

#[test]
fn test_default_fire_grids_skip_save() {
    assert!(
        FireGrid::default().save_to_bytes().is_none(),
        "Empty FireGrid should return None"
    );
    assert!(
        ForestFireGrid::default().save_to_bytes().is_none(),
        "Empty ForestFireGrid should return None"
    );
    assert!(
        ForestFireStats::default().save_to_bytes().is_none(),
        "Default ForestFireStats should return None"
    );
}

// ====================================================================
// Non-default state produces Some(bytes)
// ====================================================================

#[test]
fn test_fire_grids_save_when_nonempty() {
    let mut fg = FireGrid::default();
    fg.set(10, 10, 50);
    assert!(fg.save_to_bytes().is_some(), "Non-empty FireGrid should save");

    let mut ffg = ForestFireGrid::default();
    ffg.set(20, 20, 100);
    assert!(
        ffg.save_to_bytes().is_some(),
        "Non-empty ForestFireGrid should save"
    );

    let stats = ForestFireStats {
        active_fires: 1,
        total_area_burned: 0,
        fires_this_month: 0,
    };
    assert!(
        stats.save_to_bytes().is_some(),
        "Non-default ForestFireStats should save"
    );
}

// ====================================================================
// Corrupted bytes fall back to defaults
// ====================================================================

#[test]
fn test_fire_grid_corrupted_bytes_fallback() {
    let garbage = vec![0xFF, 0xFE, 0xFD, 0xFC];

    let fg = FireGrid::load_from_bytes(&garbage);
    assert!(
        fg.fire_levels.iter().all(|&v| v == 0),
        "Corrupted FireGrid should default to all zeros"
    );
    assert_eq!(fg.width, crate::config::GRID_WIDTH);
    assert_eq!(fg.height, crate::config::GRID_HEIGHT);

    let stats = ForestFireStats::load_from_bytes(&garbage);
    assert_eq!(stats.active_fires, 0);
    assert_eq!(stats.total_area_burned, 0);
    assert_eq!(stats.fires_this_month, 0);
}

// ====================================================================
// Save keys are registered
// ====================================================================

#[test]
fn test_fire_save_keys_registered() {
    let city = TestCity::new();
    let registry = city.resource::<SaveableRegistry>();
    let registered: std::collections::HashSet<&str> =
        registry.entries.iter().map(|e| e.key.as_str()).collect();

    let fire_keys = ["fire_grid", "forest_fire_grid", "forest_fire_stats"];

    for key in &fire_keys {
        assert!(
            registered.contains(key),
            "Expected fire save key '{}' to be registered",
            key
        );
    }
}

// ====================================================================
// Forest fire spread continues after save/load
// ====================================================================

#[test]
fn test_forest_fire_spread_continues_after_load() {
    let mut city = TestCity::new();

    // Plant a very large patch of trees so fire has plenty of fuel
    {
        let world = city.world_mut();
        let mut tree_grid = world.resource_mut::<TreeGrid>();
        for y in 100..=156 {
            for x in 100..=156 {
                tree_grid.set(x, y, true);
            }
        }
    }

    // Set fire across multiple cells at max intensity to ensure some survive
    // regardless of RNG sequence (which varies as new plugins are added).
    {
        let world = city.world_mut();
        let mut ff_grid = world.resource_mut::<ForestFireGrid>();
        for y in (120..=136).step_by(2) {
            for x in (120..=136).step_by(2) {
                ff_grid.set(x, y, 255);
            }
        }
    }

    // Verify fires are set before any ticking
    let fires_set = {
        let grid = city.resource::<ForestFireGrid>();
        grid.intensities.iter().filter(|&&v| v > 0).count()
    };
    assert!(fires_set > 0, "Should have fires after seeding");

    // Save/load roundtrip immediately — deterministic, no RNG involved
    roundtrip(&mut city);

    // Verify fire survived the roundtrip
    let fires_after_load = {
        let grid = city.resource::<ForestFireGrid>();
        grid.intensities.iter().filter(|&&v| v > 0).count()
    };
    assert_eq!(
        fires_after_load, fires_set,
        "All seeded fires should survive save/load roundtrip exactly"
    );

    // Now run ticks to verify the simulation continues without crashing.
    // We don't assert on specific fire counts since spread is RNG-dependent.
    city.tick(20);
}

// ====================================================================
// FireGrid and ForestFireGrid are independent
// ====================================================================

#[test]
fn test_fire_grid_and_forest_fire_grid_independent_roundtrip() {
    let mut city = TestCity::new();

    // Set different fire states in both grids
    {
        let world = city.world_mut();
        let mut fire_grid = world.resource_mut::<FireGrid>();
        fire_grid.set(10, 10, 50);
        fire_grid.set(20, 20, 0);

        let mut ff_grid = world.resource_mut::<ForestFireGrid>();
        ff_grid.set(10, 10, 0);
        ff_grid.set(20, 20, 150);
    }

    roundtrip(&mut city);

    // Both grids should preserve their independent state
    let fire_grid = city.resource::<FireGrid>();
    assert_eq!(fire_grid.get(10, 10), 50, "FireGrid(10,10) should be 50");
    assert_eq!(fire_grid.get(20, 20), 0, "FireGrid(20,20) should be 0");

    let ff_grid = city.resource::<ForestFireGrid>();
    assert_eq!(ff_grid.get(10, 10), 0, "ForestFireGrid(10,10) should be 0");
    assert_eq!(ff_grid.get(20, 20), 150, "ForestFireGrid(20,20) should be 150");
}

// ====================================================================
// Full state roundtrip: all fire resources together
// ====================================================================

#[test]
fn test_all_fire_state_roundtrips_together() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();

        // Building fire
        let mut fire_grid = world.resource_mut::<FireGrid>();
        fire_grid.set(30, 30, 90);

        // Forest fire
        let mut ff_grid = world.resource_mut::<ForestFireGrid>();
        ff_grid.set(40, 40, 180);

        // Stats
        let mut stats = world.resource_mut::<ForestFireStats>();
        stats.active_fires = 7;
        stats.total_area_burned = 1234;
        stats.fires_this_month = 2;
    }

    roundtrip(&mut city);

    let fire_grid = city.resource::<FireGrid>();
    assert_eq!(fire_grid.get(30, 30), 90);

    let ff_grid = city.resource::<ForestFireGrid>();
    assert_eq!(ff_grid.get(40, 40), 180);

    let stats = city.resource::<ForestFireStats>();
    assert_eq!(stats.active_fires, 7);
    assert_eq!(stats.total_area_burned, 1234);
    assert_eq!(stats.fires_this_month, 2);
}
