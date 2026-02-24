//! Integration tests for LifecycleTimer save/load roundtrip (SAVE-007, issue #703).

use crate::lifecycle::LifecycleTimer;
use crate::SaveableRegistry;
use std::collections::BTreeMap;

/// Test that LifecycleTimer fields roundtrip correctly through save/load.
#[test]
fn test_lifecycle_timer_save_load_roundtrip() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();
    app.init_resource::<LifecycleTimer>();

    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<LifecycleTimer>();
    }

    // Set non-default values.
    {
        let mut timer = app.world_mut().resource_mut::<LifecycleTimer>();
        timer.last_aging_day = 730;
        timer.last_emigration_tick = 15;
    }

    // Save.
    let extensions = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };

    assert!(
        extensions.contains_key("lifecycle_timer"),
        "lifecycle_timer key should be present in saved extensions"
    );

    // Reset to default.
    {
        let mut timer = app.world_mut().resource_mut::<LifecycleTimer>();
        timer.last_aging_day = 0;
        timer.last_emigration_tick = 0;
    }

    // Load.
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &extensions);
        app.world_mut().insert_resource(registry);
    }

    let timer = app.world().resource::<LifecycleTimer>();
    assert_eq!(
        timer.last_aging_day, 730,
        "last_aging_day should survive save/load roundtrip"
    );
    assert_eq!(
        timer.last_emigration_tick, 15,
        "last_emigration_tick should survive save/load roundtrip"
    );
}

/// Test that loading an old save (without lifecycle_timer data) resets to
/// default values instead of leaving stale 0 values that would cause
/// immediate aging/emigration bursts.
#[test]
fn test_lifecycle_timer_load_missing_key_resets_to_default() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();
    app.init_resource::<LifecycleTimer>();

    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<LifecycleTimer>();
    }

    // Set non-default values to simulate a running game.
    {
        let mut timer = app.world_mut().resource_mut::<LifecycleTimer>();
        timer.last_aging_day = 500;
        timer.last_emigration_tick = 20;
    }

    // Load an empty extension map (simulating an old save without this key).
    let empty_extensions: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &empty_extensions);
        app.world_mut().insert_resource(registry);
    }

    let timer = app.world().resource::<LifecycleTimer>();
    assert_eq!(
        timer.last_aging_day, 0,
        "last_aging_day should reset to default when key is absent from save"
    );
    assert_eq!(
        timer.last_emigration_tick, 0,
        "last_emigration_tick should reset to default when key is absent from save"
    );
}

/// Test that saving a default LifecycleTimer still produces data (since we
/// always save to prevent ambiguity with missing-key fallback).
#[test]
fn test_lifecycle_timer_save_default_produces_bytes() {
    use crate::Saveable;
    let timer = LifecycleTimer::default();
    let bytes = timer.save_to_bytes();
    assert!(
        bytes.is_some(),
        "default LifecycleTimer should still produce save bytes"
    );
}

/// Test that corrupted bytes gracefully fall back to default.
#[test]
fn test_lifecycle_timer_load_corrupted_bytes_falls_back_to_default() {
    use crate::Saveable;
    let corrupted = vec![0xFF, 0xFE, 0xFD];
    let timer = LifecycleTimer::load_from_bytes(&corrupted);
    assert_eq!(
        timer.last_aging_day, 0,
        "corrupted bytes should fall back to default last_aging_day"
    );
    assert_eq!(
        timer.last_emigration_tick, 0,
        "corrupted bytes should fall back to default last_emigration_tick"
    );
}
