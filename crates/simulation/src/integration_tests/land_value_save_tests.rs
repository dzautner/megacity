//! Integration tests for LandValueGrid save/load roundtrip (SAVE-035).
//!
//! Validates that:
//! - Land value grid data survives a full ECS save/load cycle
//! - Land value overlay matches pre-save state after restoration
//! - Modified land values (from services, pollution, etc.) are preserved
//! - Building upgrade decisions remain consistent after load

use crate::land_value::LandValueGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::SaveableRegistry;

// ====================================================================
// Roundtrip helper (mirrors env_grid_save_tests pattern)
// ====================================================================

/// Save all registered saveables via the SaveableRegistry, reset them to
/// defaults, then restore from the saved bytes.
fn roundtrip(city: &mut TestCity) {
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();

    let extensions = registry.save_all(world);
    registry.reset_all(world);
    registry.load_all(world, &extensions);

    world.insert_resource(registry);
}

// ====================================================================
// 1. Basic roundtrip — manually set values survive save/load
// ====================================================================

#[test]
fn test_land_value_grid_save_load_roundtrip_basic() {
    let mut city = TestCity::new();

    // Set several non-default values across the grid
    {
        let world = city.world_mut();
        let mut lv = world.resource_mut::<LandValueGrid>();
        lv.set(0, 0, 0);
        lv.set(10, 20, 200);
        lv.set(128, 128, 100);
        lv.set(255, 255, 255);
    }

    roundtrip(&mut city);

    let lv = city.resource::<LandValueGrid>();
    assert_eq!(lv.get(0, 0), 0, "Corner (0,0) should be 0 after roundtrip");
    assert_eq!(lv.get(10, 20), 200, "Cell (10,20) should be 200 after roundtrip");
    assert_eq!(lv.get(128, 128), 100, "Centre cell should be 100 after roundtrip");
    assert_eq!(lv.get(255, 255), 255, "Corner (255,255) should be 255 after roundtrip");
}

// ====================================================================
// 2. Grid dimensions preserved after roundtrip
// ====================================================================

#[test]
fn test_land_value_grid_dimensions_preserved_after_roundtrip() {
    let mut city = TestCity::new();

    roundtrip(&mut city);

    let lv = city.resource::<LandValueGrid>();
    assert_eq!(lv.width, 256, "Width should be 256 after roundtrip");
    assert_eq!(lv.height, 256, "Height should be 256 after roundtrip");
    assert_eq!(
        lv.values.len(),
        256 * 256,
        "Values vec length should be 256*256 after roundtrip"
    );
}

// ====================================================================
// 3. Overlay matches pre-save state — every cell identical
// ====================================================================

#[test]
fn test_land_value_overlay_matches_pre_save_state() {
    let mut city = TestCity::new();

    // Run simulation to produce a non-uniform land value distribution
    // (parks boost nearby cells, producing spatial variation)
    city = city.with_service(80, 80, ServiceType::SmallPark);
    city.tick_slow_cycles(20);

    // Snapshot the entire grid before save
    let snapshot: Vec<u8> = city.resource::<LandValueGrid>().values.clone();

    roundtrip(&mut city);

    let restored = &city.resource::<LandValueGrid>().values;
    assert_eq!(
        snapshot.len(),
        restored.len(),
        "Grid size should match after roundtrip"
    );
    assert_eq!(
        &snapshot, restored,
        "Every cell in the land value overlay must match pre-save state"
    );
}

// ====================================================================
// 4. Simulated land values (park boost) preserved after load
// ====================================================================

#[test]
fn test_land_value_park_boost_preserved_after_save_load() {
    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::SmallPark);

    // Let the park boost converge for several cycles
    city.tick_slow_cycles(30);

    let before = city.resource::<LandValueGrid>().get(100, 100);
    assert!(
        before > 50,
        "Park should boost land value above baseline 50 before save, got {before}"
    );

    roundtrip(&mut city);

    let after = city.resource::<LandValueGrid>().get(100, 100);
    assert_eq!(
        before, after,
        "Park-boosted land value should be identical after roundtrip: before={before}, after={after}"
    );
}

// ====================================================================
// 5. Low land values (industrial area) preserved after load
// ====================================================================

#[test]
fn test_land_value_industrial_penalty_preserved_after_save_load() {
    use crate::grid::ZoneType;

    let mut city = TestCity::new()
        .with_zone(100, 100, ZoneType::Industrial);

    city.tick_slow_cycles(30);

    let before = city.resource::<LandValueGrid>().get(100, 100);
    assert!(
        before < 50,
        "Industrial zone should reduce land value below 50 before save, got {before}"
    );

    roundtrip(&mut city);

    let after = city.resource::<LandValueGrid>().get(100, 100);
    assert_eq!(
        before, after,
        "Industrial-penalised land value should be identical after roundtrip: before={before}, after={after}"
    );
}

// ====================================================================
// 6. Reset clears to default, then load restores
// ====================================================================

#[test]
fn test_land_value_reset_then_load_restores_values() {
    let mut city = TestCity::new();

    // Set distinctive values
    {
        let world = city.world_mut();
        let mut lv = world.resource_mut::<LandValueGrid>();
        lv.set(50, 50, 222);
        lv.set(150, 150, 11);
    }

    // Save
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();
    let extensions = registry.save_all(world);

    // Reset — values should go back to default (50)
    registry.reset_all(world);
    {
        let lv = world.resource::<LandValueGrid>();
        assert_eq!(
            lv.get(50, 50),
            50,
            "After reset, cell should be default 50"
        );
        assert_eq!(
            lv.get(150, 150),
            50,
            "After reset, cell should be default 50"
        );
    }

    // Load — values should be restored
    registry.load_all(world, &extensions);
    {
        let lv = world.resource::<LandValueGrid>();
        assert_eq!(
            lv.get(50, 50),
            222,
            "After load, cell should be restored to 222"
        );
        assert_eq!(
            lv.get(150, 150),
            11,
            "After load, cell should be restored to 11"
        );
    }

    world.insert_resource(registry);
}

// ====================================================================
// 7. Building upgrade decisions consistent after load
// ====================================================================
// Building upgrades depend on land value thresholds. This test verifies
// that cells above and below upgrade thresholds remain in the same
// bucket after a save/load cycle.

#[test]
fn test_land_value_upgrade_thresholds_consistent_after_load() {
    let mut city = TestCity::new();

    // Set values at typical upgrade decision boundaries
    {
        let world = city.world_mut();
        let mut lv = world.resource_mut::<LandValueGrid>();
        lv.set(10, 10, 30);  // low: no upgrade
        lv.set(20, 20, 80);  // medium: eligible for upgrade
        lv.set(30, 30, 150); // high: premium upgrade
        lv.set(40, 40, 250); // very high: max tier
    }

    roundtrip(&mut city);

    let lv = city.resource::<LandValueGrid>();
    assert_eq!(lv.get(10, 10), 30, "Low value must be exact after roundtrip");
    assert_eq!(lv.get(20, 20), 80, "Medium value must be exact after roundtrip");
    assert_eq!(lv.get(30, 30), 150, "High value must be exact after roundtrip");
    assert_eq!(lv.get(40, 40), 250, "Very high value must be exact after roundtrip");
}

// ====================================================================
// 8. Average land value identical after roundtrip
// ====================================================================

#[test]
fn test_land_value_average_preserved_after_roundtrip() {
    let mut city = TestCity::new()
        .with_service(60, 60, ServiceType::LargePark);

    city.tick_slow_cycles(15);

    let avg_before = city.resource::<LandValueGrid>().average();

    roundtrip(&mut city);

    let avg_after = city.resource::<LandValueGrid>().average();
    assert!(
        (avg_before - avg_after).abs() < f32::EPSILON,
        "Average land value should be identical after roundtrip: before={avg_before}, after={avg_after}"
    );
}
