//! Integration tests for VirtualPopulation save/load roundtrip (SAVE-021, Issue #717).
//!
//! Verifies that `VirtualPopulation` state — total count, employment stats,
//! per-district statistics, and dynamic citizen cap — survives a save/load cycle.

use crate::test_harness::TestCity;
use crate::virtual_population::VirtualPopulation;
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
// Tests
// ====================================================================

#[test]
fn test_virtual_population_save_load_roundtrip() {
    let mut city = TestCity::new();

    // Set up non-default virtual population state
    {
        let world = city.world_mut();
        let mut vp = world.resource_mut::<VirtualPopulation>();
        vp.add_virtual_citizen(0, 25, true, 80.0, 1200.0, 0.15);
        vp.add_virtual_citizen(0, 40, false, 60.0, 0.0, 0.0);
        vp.add_virtual_citizen(1, 70, false, 50.0, 0.0, 0.0);
    }

    // Snapshot before save
    let (total_before, employed_before, districts_before) = {
        let vp = city.resource::<VirtualPopulation>();
        (
            vp.total_virtual,
            vp.virtual_employed,
            vp.district_stats.clone(),
        )
    };

    assert_eq!(total_before, 3);
    assert_eq!(employed_before, 1);
    assert_eq!(districts_before.len(), 2);

    // Roundtrip through save/load
    roundtrip(&mut city);

    // Verify state survived
    let vp = city.resource::<VirtualPopulation>();
    assert_eq!(vp.total_virtual, total_before, "total_virtual lost on roundtrip");
    assert_eq!(
        vp.virtual_employed, employed_before,
        "virtual_employed lost on roundtrip"
    );
    assert_eq!(
        vp.district_stats.len(),
        districts_before.len(),
        "district_stats length mismatch"
    );
    assert_eq!(vp.district_stats[0].population, 2);
    assert_eq!(vp.district_stats[0].employed, 1);
    assert_eq!(vp.district_stats[1].population, 1);
    assert!((vp.district_stats[0].avg_happiness - 70.0).abs() < 0.01);
}

#[test]
fn test_virtual_population_default_on_missing_key() {
    // Simulates loading an old save that has no virtual_population key:
    // roundtrip with an empty (default) VirtualPopulation should yield default.
    let mut city = TestCity::new();

    // Leave VirtualPopulation at default (0 virtual citizens)
    roundtrip(&mut city);

    let vp = city.resource::<VirtualPopulation>();
    assert_eq!(vp.total_virtual, 0);
    assert_eq!(vp.virtual_employed, 0);
    assert!(vp.district_stats.is_empty());
}

#[test]
fn test_virtual_population_large_count_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut vp = world.resource_mut::<VirtualPopulation>();
        // Simulate a city with 500K virtual population across 10 districts
        for district in 0..10 {
            for i in 0..1000 {
                let age = (20 + (i % 50)) as u8;
                let employed = i % 3 != 0;
                let happiness = 40.0 + (i % 60) as f32;
                let salary = if employed { 800.0 + (i % 500) as f32 } else { 0.0 };
                vp.add_virtual_citizen(district, age, employed, happiness, salary, 0.1);
            }
        }
        // Also set a non-default cap
        vp.max_real_citizens = 100_000;
    }

    let total_before = city.resource::<VirtualPopulation>().total_virtual;
    let cap_before = city.resource::<VirtualPopulation>().max_real_citizens;
    assert_eq!(total_before, 10_000);
    assert_eq!(cap_before, 100_000);

    roundtrip(&mut city);

    let vp = city.resource::<VirtualPopulation>();
    assert_eq!(vp.total_virtual, total_before, "large population lost on roundtrip");
    assert_eq!(
        vp.max_real_citizens, cap_before,
        "max_real_citizens lost on roundtrip"
    );
    assert_eq!(vp.district_stats.len(), 10);
    // Each district should have 1000 citizens
    for (i, ds) in vp.district_stats.iter().enumerate() {
        assert_eq!(ds.population, 1000, "district {i} population mismatch");
    }
}

#[test]
fn test_virtual_population_reset_clears_state() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut vp = world.resource_mut::<VirtualPopulation>();
        vp.add_virtual_citizen(0, 30, true, 70.0, 1000.0, 0.1);
    }

    // Reset via registry (simulates "new game")
    {
        let world = city.world_mut();
        let registry = world.remove_resource::<SaveableRegistry>().unwrap();
        registry.reset_all(world);
        world.insert_resource(registry);
    }

    let vp = city.resource::<VirtualPopulation>();
    assert_eq!(vp.total_virtual, 0, "reset should clear total_virtual");
    assert_eq!(vp.virtual_employed, 0, "reset should clear virtual_employed");
    assert!(
        vp.district_stats.is_empty(),
        "reset should clear district_stats"
    );
}
