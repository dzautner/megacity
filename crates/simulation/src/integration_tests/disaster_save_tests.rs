//! Integration tests for ActiveDisaster save/load roundtrips.

use crate::disasters::{ActiveDisaster, DisasterInstance, DisasterType};
use crate::test_harness::TestCity;
use crate::Saveable;
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
// Tornado roundtrip
// ====================================================================

#[test]
fn test_active_disaster_tornado_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut active = world.resource_mut::<ActiveDisaster>();
        active.current = Some(DisasterInstance {
            disaster_type: DisasterType::Tornado,
            center_x: 100,
            center_y: 150,
            radius: 5,
            ticks_remaining: 30,
            damage_applied: true,
        });
    }

    roundtrip(&mut city);

    let active = city.resource::<ActiveDisaster>();
    let d = active.current.as_ref().expect("disaster should persist after load");
    assert_eq!(d.disaster_type, DisasterType::Tornado);
    assert_eq!(d.center_x, 100);
    assert_eq!(d.center_y, 150);
    assert_eq!(d.radius, 5);
    assert_eq!(d.ticks_remaining, 30);
    assert!(d.damage_applied);
}

// ====================================================================
// Earthquake roundtrip
// ====================================================================

#[test]
fn test_active_disaster_earthquake_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut active = world.resource_mut::<ActiveDisaster>();
        active.current = Some(DisasterInstance {
            disaster_type: DisasterType::Earthquake,
            center_x: 50,
            center_y: 75,
            radius: 10,
            ticks_remaining: 15,
            damage_applied: false,
        });
    }

    roundtrip(&mut city);

    let active = city.resource::<ActiveDisaster>();
    let d = active.current.as_ref().expect("earthquake should persist");
    assert_eq!(d.disaster_type, DisasterType::Earthquake);
    assert_eq!(d.center_x, 50);
    assert_eq!(d.center_y, 75);
    assert_eq!(d.radius, 10);
    assert_eq!(d.ticks_remaining, 15);
    assert!(!d.damage_applied);
}

// ====================================================================
// Flood roundtrip
// ====================================================================

#[test]
fn test_active_disaster_flood_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut active = world.resource_mut::<ActiveDisaster>();
        active.current = Some(DisasterInstance {
            disaster_type: DisasterType::Flood,
            center_x: 200,
            center_y: 200,
            radius: 8,
            ticks_remaining: 80,
            damage_applied: true,
        });
    }

    roundtrip(&mut city);

    let active = city.resource::<ActiveDisaster>();
    let d = active.current.as_ref().expect("flood should persist");
    assert_eq!(d.disaster_type, DisasterType::Flood);
    assert_eq!(d.center_x, 200);
    assert_eq!(d.center_y, 200);
    assert_eq!(d.radius, 8);
    assert_eq!(d.ticks_remaining, 80);
    assert!(d.damage_applied);
}

// ====================================================================
// No disaster (default) skips save
// ====================================================================

#[test]
fn test_active_disaster_default_skips_save() {
    let default = ActiveDisaster::default();
    assert!(default.save_to_bytes().is_none());
}

// ====================================================================
// Reset clears disaster
// ====================================================================

#[test]
fn test_active_disaster_reset_clears() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut active = world.resource_mut::<ActiveDisaster>();
        active.current = Some(DisasterInstance {
            disaster_type: DisasterType::Tornado,
            center_x: 10,
            center_y: 20,
            radius: 5,
            ticks_remaining: 40,
            damage_applied: false,
        });
    }

    // Save, reset, and verify reset clears the disaster.
    {
        let world = city.world_mut();
        let registry = world.remove_resource::<SaveableRegistry>().unwrap();
        let _extensions = registry.save_all(world);
        registry.reset_all(world);

        let active = world.resource::<ActiveDisaster>();
        assert!(
            active.current.is_none(),
            "reset should clear active disaster"
        );

        world.insert_resource(registry);
    }
}

// ====================================================================
// Key is registered
// ====================================================================

#[test]
fn test_active_disaster_save_key_registered() {
    let city = TestCity::new();
    let registry = city.resource::<SaveableRegistry>();
    let registered: std::collections::HashSet<&str> =
        registry.entries.iter().map(|e| e.key.as_str()).collect();

    assert!(
        registered.contains("active_disaster"),
        "active_disaster key should be registered in SaveableRegistry"
    );
}

// ====================================================================
// Corrupted bytes fall back to default
// ====================================================================

#[test]
fn test_active_disaster_corrupted_bytes_fallback() {
    let garbage = vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB];
    let loaded = ActiveDisaster::load_from_bytes(&garbage);
    // Corrupted bytes should fall back to default (no active disaster).
    assert!(
        loaded.current.is_none(),
        "corrupted bytes should result in no active disaster"
    );
}