//! SAVE-033: Integration tests for crime grid state serialization.
//!
//! Tests verify that:
//! - CrimeGrid roundtrips correctly through save/load
//! - Crime hotspots are visible immediately after load
//! - Default (all-zero) crime grid skips serialization
//! - Corrupted bytes fall back to defaults
//! - The save key is registered in the SaveableRegistry

use crate::crime::CrimeGrid;
use crate::test_harness::TestCity;
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
// Basic roundtrip
// ====================================================================

#[test]
fn test_crime_grid_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<CrimeGrid>();
        grid.set(50, 50, 20);
        grid.set(100, 100, 15);
        grid.set(200, 200, 5);
    }

    roundtrip(&mut city);

    let grid = city.resource::<CrimeGrid>();
    assert_eq!(
        grid.get(50, 50),
        20,
        "Crime at (50,50) should survive roundtrip"
    );
    assert_eq!(
        grid.get(100, 100),
        15,
        "Crime at (100,100) should survive roundtrip"
    );
    assert_eq!(
        grid.get(200, 200),
        5,
        "Crime at (200,200) should survive roundtrip"
    );
    assert_eq!(grid.get(0, 0), 0, "Zero-crime cell should remain zero");
}

// ====================================================================
// Crime hotspots visible immediately after load
// ====================================================================

#[test]
fn test_crime_hotspots_visible_immediately_after_load() {
    let mut city = TestCity::new();

    // Create a "hotspot" pattern: high crime in a cluster of cells
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<CrimeGrid>();
        for y in 120..=130 {
            for x in 120..=130 {
                grid.set(x, y, 25); // max base crime
            }
        }
    }

    roundtrip(&mut city);

    // Verify hotspot is intact immediately after load (no ticks needed)
    let grid = city.resource::<CrimeGrid>();
    for y in 120..=130 {
        for x in 120..=130 {
            assert_eq!(
                grid.get(x, y),
                25,
                "Crime hotspot at ({x}, {y}) should be visible immediately after load"
            );
        }
    }
    // Cells outside the hotspot should still be zero
    assert_eq!(
        grid.get(110, 110),
        0,
        "Cell outside hotspot should remain zero"
    );
}

// ====================================================================
// Default state skips save
// ====================================================================

#[test]
fn test_default_crime_grid_skips_save() {
    assert!(
        CrimeGrid::default().save_to_bytes().is_none(),
        "All-zero CrimeGrid should return None (skip save)"
    );
}

// ====================================================================
// Non-default state produces Some(bytes)
// ====================================================================

#[test]
fn test_crime_grid_saves_when_nonempty() {
    let mut grid = CrimeGrid::default();
    grid.set(10, 10, 15);
    assert!(
        grid.save_to_bytes().is_some(),
        "Non-zero CrimeGrid should produce Some(bytes)"
    );
}

// ====================================================================
// Corrupted bytes fall back to defaults
// ====================================================================

#[test]
fn test_crime_grid_corrupted_bytes_fallback() {
    let garbage = vec![0xFF, 0xFE, 0xFD, 0xFC];

    let grid = CrimeGrid::load_from_bytes(&garbage);
    assert!(
        grid.levels.iter().all(|&v| v == 0),
        "Corrupted CrimeGrid should default to all zeros"
    );
    assert_eq!(grid.width, crate::config::GRID_WIDTH);
    assert_eq!(grid.height, crate::config::GRID_HEIGHT);
}

// ====================================================================
// Save key is registered
// ====================================================================

#[test]
fn test_crime_grid_save_key_registered() {
    let city = TestCity::new();
    let registry = city.resource::<SaveableRegistry>();
    let registered: std::collections::HashSet<&str> =
        registry.entries.iter().map(|e| e.key.as_str()).collect();

    assert!(
        registered.contains("crime_grid"),
        "Expected 'crime_grid' save key to be registered"
    );
}

// ====================================================================
// Crime history maintained across save/load
// ====================================================================

#[test]
fn test_crime_history_maintained_across_save_load() {
    let mut city = TestCity::new();

    // Simulate varied crime levels across the grid (representing history)
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<CrimeGrid>();
        // Low crime area
        grid.set(30, 30, 3);
        grid.set(31, 30, 5);
        // Medium crime area
        grid.set(80, 80, 12);
        grid.set(81, 80, 14);
        // High crime area
        grid.set(150, 150, 22);
        grid.set(151, 150, 25);
    }

    roundtrip(&mut city);

    let grid = city.resource::<CrimeGrid>();
    // Verify all distinct crime levels survived
    assert_eq!(grid.get(30, 30), 3, "Low crime cell should persist");
    assert_eq!(grid.get(31, 30), 5, "Low crime cell should persist");
    assert_eq!(grid.get(80, 80), 12, "Medium crime cell should persist");
    assert_eq!(grid.get(81, 80), 14, "Medium crime cell should persist");
    assert_eq!(grid.get(150, 150), 22, "High crime cell should persist");
    assert_eq!(grid.get(151, 150), 25, "High crime cell should persist");
}

// ====================================================================
// Multiple roundtrips preserve state
// ====================================================================

#[test]
fn test_crime_grid_multiple_roundtrips() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<CrimeGrid>();
        grid.set(64, 64, 18);
        grid.set(192, 192, 7);
    }

    // First roundtrip
    roundtrip(&mut city);

    let g1_a = city.resource::<CrimeGrid>().get(64, 64);
    let g1_b = city.resource::<CrimeGrid>().get(192, 192);
    assert_eq!(g1_a, 18);
    assert_eq!(g1_b, 7);

    // Second roundtrip
    roundtrip(&mut city);

    let g2_a = city.resource::<CrimeGrid>().get(64, 64);
    let g2_b = city.resource::<CrimeGrid>().get(192, 192);
    assert_eq!(g2_a, 18, "Value should survive second roundtrip");
    assert_eq!(g2_b, 7, "Value should survive second roundtrip");
}
