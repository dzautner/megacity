//! Integration tests for environmental grid save/load roundtrips.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::crime::CrimeGrid;
use crate::forest_fire::ForestFireGrid;
use crate::groundwater::{GroundwaterGrid, WaterQualityGrid};
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::stormwater::StormwaterGrid;
use crate::test_harness::TestCity;
use crate::trees::TreeGrid;
use crate::water_pollution::WaterPollutionGrid;
use crate::SaveableRegistry;

// ====================================================================
// Roundtrip helper
// ====================================================================

/// Save all registered saveables, reset them, then restore from the saved
/// bytes. Operates entirely through `world_mut()`.
fn roundtrip(city: &mut TestCity) {
    // Remove the registry so we own it (avoids borrow issues with World).
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();

    // Save while we have a shared ref to the world.
    let extensions = registry.save_all(world);

    // Reset all saveables to defaults, then restore from saved bytes.
    registry.reset_all(world);
    registry.load_all(world, &extensions);

    // Put the registry back.
    world.insert_resource(registry);
}

// ====================================================================
// PollutionGrid roundtrip
// ====================================================================

#[test]
fn test_pollution_grid_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<PollutionGrid>();
        grid.set(10, 10, 42);
        grid.set(100, 200, 200);
    }

    roundtrip(&mut city);

    let grid = city.resource::<PollutionGrid>();
    assert_eq!(grid.get(10, 10), 42);
    assert_eq!(grid.get(100, 200), 200);
    assert_eq!(grid.get(0, 0), 0);
}

// ====================================================================
// NoisePollutionGrid roundtrip
// ====================================================================

#[test]
fn test_noise_grid_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<NoisePollutionGrid>();
        grid.set(5, 5, 80);
        grid.set(50, 50, 100);
    }

    roundtrip(&mut city);

    let grid = city.resource::<NoisePollutionGrid>();
    assert_eq!(grid.get(5, 5), 80);
    assert_eq!(grid.get(50, 50), 100);
    assert_eq!(grid.get(0, 0), 0);
}

// ====================================================================
// CrimeGrid roundtrip
// ====================================================================

#[test]
fn test_crime_grid_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<CrimeGrid>();
        grid.set(20, 30, 25);
    }

    roundtrip(&mut city);

    let grid = city.resource::<CrimeGrid>();
    assert_eq!(grid.get(20, 30), 25);
    assert_eq!(grid.get(0, 0), 0);
}

// ====================================================================
// TreeGrid roundtrip
// ====================================================================

#[test]
fn test_tree_grid_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<TreeGrid>();
        grid.set(15, 15, true);
        grid.set(100, 100, true);
    }

    roundtrip(&mut city);

    let grid = city.resource::<TreeGrid>();
    assert!(grid.has_tree(15, 15));
    assert!(grid.has_tree(100, 100));
    assert!(!grid.has_tree(0, 0));
}

// ====================================================================
// WaterPollutionGrid roundtrip
// ====================================================================

#[test]
fn test_water_pollution_grid_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WaterPollutionGrid>();
        grid.set(30, 30, 150);
    }

    roundtrip(&mut city);

    let grid = city.resource::<WaterPollutionGrid>();
    assert_eq!(grid.get(30, 30), 150);
    assert_eq!(grid.get(0, 0), 0);
}

// ====================================================================
// GroundwaterGrid roundtrip
// ====================================================================

#[test]
fn test_groundwater_grid_save_load_roundtrip() {
    let mut city = TestCity::new();

    // Snapshot the initial value at (0,0) -- init_groundwater computes
    // values from terrain elevation, so it may differ from Default.
    let initial_00 = city.resource::<GroundwaterGrid>().get(0, 0);

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<GroundwaterGrid>();
        grid.set(40, 40, 255);
        grid.set(41, 41, 0);
    }

    roundtrip(&mut city);

    let grid = city.resource::<GroundwaterGrid>();
    assert_eq!(grid.get(40, 40), 255);
    assert_eq!(grid.get(41, 41), 0);
    // Unmodified cell should retain its pre-save value.
    assert_eq!(grid.get(0, 0), initial_00);
}

// ====================================================================
// WaterQualityGrid roundtrip
// ====================================================================

#[test]
fn test_water_quality_grid_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WaterQualityGrid>();
        grid.set(60, 60, 10);
    }

    roundtrip(&mut city);

    let grid = city.resource::<WaterQualityGrid>();
    assert_eq!(grid.get(60, 60), 10);
    // Default is 200
    assert_eq!(grid.get(0, 0), 200);
}

// ====================================================================
// StormwaterGrid roundtrip
// ====================================================================

#[test]
fn test_stormwater_grid_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<StormwaterGrid>();
        grid.set(80, 80, 12.5);
    }

    roundtrip(&mut city);

    let grid = city.resource::<StormwaterGrid>();
    assert!((grid.get(80, 80) - 12.5).abs() < f32::EPSILON);
    assert!((grid.get(0, 0) - 0.0).abs() < f32::EPSILON);
}

// ====================================================================
// ForestFireGrid roundtrip
// ====================================================================

#[test]
fn test_forest_fire_grid_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<ForestFireGrid>();
        grid.set(90, 90, 180);
    }

    roundtrip(&mut city);

    let grid = city.resource::<ForestFireGrid>();
    assert_eq!(grid.get(90, 90), 180);
    assert_eq!(grid.get(0, 0), 0);
}

// ====================================================================
// Default grids skip saving (return None)
// ====================================================================

#[test]
fn test_default_env_grids_skip_save() {
    use crate::Saveable;

    assert!(PollutionGrid::default().save_to_bytes().is_none());
    assert!(NoisePollutionGrid::default().save_to_bytes().is_none());
    assert!(CrimeGrid::default().save_to_bytes().is_none());
    assert!(TreeGrid::default().save_to_bytes().is_none());
    assert!(WaterPollutionGrid::default().save_to_bytes().is_none());
    assert!(GroundwaterGrid::default().save_to_bytes().is_none());
    assert!(WaterQualityGrid::default().save_to_bytes().is_none());
    assert!(StormwaterGrid::default().save_to_bytes().is_none());
    assert!(ForestFireGrid::default().save_to_bytes().is_none());
}

// ====================================================================
// All env grid keys are registered
// ====================================================================

#[test]
fn test_env_grid_save_keys_registered() {
    let city = TestCity::new();
    let registry = city.resource::<SaveableRegistry>();
    let registered: std::collections::HashSet<&str> =
        registry.entries.iter().map(|e| e.key.as_str()).collect();

    let env_keys = [
        "pollution_grid",
        "noise_grid",
        "crime_grid",
        "tree_grid",
        "water_pollution_grid",
        "groundwater_grid",
        "water_quality_grid",
        "stormwater_grid",
        "forest_fire_grid",
    ];

    for key in &env_keys {
        assert!(
            registered.contains(key),
            "Expected env grid key '{}' to be registered",
            key
        );
    }
}

// ====================================================================
// Corrupted bytes fall back to default
// ====================================================================

#[test]
fn test_env_grid_corrupted_bytes_fallback_to_default() {
    use crate::Saveable;

    let garbage = vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB];

    let pollution = PollutionGrid::load_from_bytes(&garbage);
    assert_eq!(pollution.levels.len(), GRID_WIDTH * GRID_HEIGHT);
    assert!(pollution.levels.iter().all(|&v| v == 0));

    let trees = TreeGrid::load_from_bytes(&garbage);
    assert_eq!(trees.cells.len(), GRID_WIDTH * GRID_HEIGHT);
    assert!(trees.cells.iter().all(|&v| !v));

    let groundwater = GroundwaterGrid::load_from_bytes(&garbage);
    assert!(groundwater.levels.iter().all(|&v| v == 128));

    let quality = WaterQualityGrid::load_from_bytes(&garbage);
    assert!(quality.levels.iter().all(|&v| v == 200));

    let storm = StormwaterGrid::load_from_bytes(&garbage);
    assert!(storm.runoff.iter().all(|&v| v == 0.0));
}
