//! Integration tests for TrafficGrid save/load roundtrips.

use crate::test_harness::TestCity;
use crate::traffic::TrafficGrid;
use crate::SaveableRegistry;

// ====================================================================
// Roundtrip helper
// ====================================================================

fn roundtrip(city: &mut TestCity) {
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();

    let extensions = registry.save_all(world);
    registry.reset_all(world);
    registry.load_all(world, &extensions);

    world.insert_resource(registry);
}

// ====================================================================
// Basic roundtrip
// ====================================================================

#[test]
fn test_traffic_grid_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<TrafficGrid>();
        grid.set(10, 10, 5);
        grid.set(100, 200, 42);
        grid.set(50, 50, u16::MAX);
    }

    roundtrip(&mut city);

    let grid = city.resource::<TrafficGrid>();
    assert_eq!(grid.get(10, 10), 5);
    assert_eq!(grid.get(100, 200), 42);
    assert_eq!(grid.get(50, 50), u16::MAX);
    assert_eq!(grid.get(0, 0), 0);
}

// ====================================================================
// Default grid skips saving (returns None)
// ====================================================================

#[test]
fn test_traffic_grid_default_skips_save() {
    use crate::Saveable;

    assert!(TrafficGrid::default().save_to_bytes().is_none());
}

// ====================================================================
// Reset clears traffic data (no stale data)
// ====================================================================

#[test]
fn test_traffic_grid_reset_clears_data() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<TrafficGrid>();
        grid.set(10, 10, 99);
    }

    // Reset via registry
    {
        let world = city.world_mut();
        let registry = world.remove_resource::<SaveableRegistry>().unwrap();
        registry.reset_all(world);
        world.insert_resource(registry);
    }

    let grid = city.resource::<TrafficGrid>();
    assert_eq!(grid.get(10, 10), 0, "stale traffic data should be cleared after reset");
}

// ====================================================================
// Corrupted bytes fall back to default
// ====================================================================

#[test]
fn test_traffic_grid_corrupted_bytes_fallback() {
    use crate::Saveable;

    let garbage = vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB];
    let grid = TrafficGrid::load_from_bytes(&garbage);

    assert_eq!(
        grid.density.len(),
        crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT
    );
    assert!(grid.density.iter().all(|&v| v == 0));
}

// ====================================================================
// Save key is registered
// ====================================================================

#[test]
fn test_traffic_grid_save_key_registered() {
    let city = TestCity::new();
    let registry = city.resource::<SaveableRegistry>();
    let registered: std::collections::HashSet<&str> =
        registry.entries.iter().map(|e| e.key.as_str()).collect();

    assert!(
        registered.contains("traffic_grid"),
        "Expected 'traffic_grid' to be registered in SaveableRegistry"
    );
}

// ====================================================================
// Traffic overlay functional after load (non-zero values preserved)
// ====================================================================

#[test]
fn test_traffic_overlay_functional_after_load() {
    let mut city = TestCity::new();

    // Simulate some traffic density
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<TrafficGrid>();
        grid.set(20, 20, 15);
        grid.set(30, 30, 20);
    }

    roundtrip(&mut city);

    // Verify congestion_level works correctly after load
    let grid = city.resource::<TrafficGrid>();
    let congestion = grid.congestion_level(20, 20);
    assert!(congestion > 0.0, "traffic overlay should show congestion after load");
    assert!((congestion - 15.0 / 20.0).abs() < 0.01);

    let congestion_30 = grid.congestion_level(30, 30);
    assert!((congestion_30 - 1.0).abs() < 0.01, "20/20 should be fully congested");
}
