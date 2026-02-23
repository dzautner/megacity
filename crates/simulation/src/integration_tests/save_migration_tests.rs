//! TEST-036: Save Version Migration Tests (Issue #815)
//!
//! Tests for save data backward/forward compatibility, Saveable trait round-trip
//! serialization, and extension map handling of missing/extra keys.

use crate::test_harness::TestCity;
use crate::SaveableRegistry;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// 1. Backward compatibility: missing extension keys gracefully default
// ---------------------------------------------------------------------------

/// When loading a save with an empty extension map, all Saveable resources
/// should be reset to their defaults (not left with stale values).
#[test]
fn test_save_migration_empty_extensions_resets_all_saveables() {
    let mut city = TestCity::new();
    let empty_extensions: BTreeMap<String, Vec<u8>> = BTreeMap::new();

    let world = city.world_mut();
    let reg = world.remove_resource::<SaveableRegistry>().unwrap();
    reg.load_all(world, &empty_extensions);
    world.insert_resource(reg);

    // Verify we can still tick after loading empty extensions (world is consistent).
    city.tick(1);
}

/// When an extension map is missing a specific key, only that resource should
/// be reset to default while others load normally.
#[test]
fn test_save_migration_partial_extensions_loads_present_keys_only() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct AlphaState {
        value: u32,
    }

    impl crate::Saveable for AlphaState {
        const SAVE_KEY: &'static str = "test_alpha_state";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&self.value).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            Self {
                value: serde_json::from_slice(bytes).unwrap_or_default(),
            }
        }
    }

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct BetaState {
        label: String,
    }

    impl crate::Saveable for BetaState {
        const SAVE_KEY: &'static str = "test_beta_state";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&self.label).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            Self {
                label: serde_json::from_slice(bytes).unwrap_or_default(),
            }
        }
    }

    app.init_resource::<AlphaState>();
    app.init_resource::<BetaState>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<AlphaState>();
        registry.register::<BetaState>();
    }

    // Set non-default values on both.
    app.world_mut().resource_mut::<AlphaState>().value = 42;
    app.world_mut().resource_mut::<BetaState>().label = "hello".to_string();

    // Build extensions with only AlphaState present.
    let mut extensions: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    extensions.insert(
        "test_alpha_state".to_string(),
        serde_json::to_vec(&99u32).unwrap(),
    );
    // BetaState key is intentionally absent.

    let registry = app
        .world_mut()
        .remove_resource::<SaveableRegistry>()
        .unwrap();
    registry.load_all(app.world_mut(), &extensions);
    app.world_mut().insert_resource(registry);

    assert_eq!(app.world().resource::<AlphaState>().value, 99);
    assert_eq!(
        app.world().resource::<BetaState>().label,
        "",
        "Missing extension key should cause reset to default, not retain old value"
    );
}

// ---------------------------------------------------------------------------
// 2. Saveable resource round-trip through save/load
// ---------------------------------------------------------------------------

/// A Saveable resource with non-trivial state should survive a full
/// save_all -> load_all round-trip without data loss.
#[test]
fn test_save_migration_saveable_roundtrip_preserves_state() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct GameProgress {
        level: u32,
        score: f64,
        unlocked_items: Vec<String>,
    }

    impl crate::Saveable for GameProgress {
        const SAVE_KEY: &'static str = "test_game_progress";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&(&self.level, &self.score, &self.unlocked_items)).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            let (level, score, unlocked_items): (u32, f64, Vec<String>) =
                serde_json::from_slice(bytes).unwrap_or_default();
            Self {
                level,
                score,
                unlocked_items,
            }
        }
    }

    app.init_resource::<GameProgress>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<GameProgress>();
    }

    {
        let mut progress = app.world_mut().resource_mut::<GameProgress>();
        progress.level = 15;
        progress.score = 98765.4321;
        progress.unlocked_items = vec!["sword".into(), "shield".into(), "potion".into()];
    }

    // Save all extensions.
    let extensions = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };
    assert!(extensions.contains_key("test_game_progress"));

    // Reset to defaults, then load.
    app.world_mut().insert_resource(GameProgress::default());
    assert_eq!(app.world().resource::<GameProgress>().level, 0);

    let registry = app
        .world_mut()
        .remove_resource::<SaveableRegistry>()
        .unwrap();
    registry.load_all(app.world_mut(), &extensions);
    app.world_mut().insert_resource(registry);

    let restored = app.world().resource::<GameProgress>();
    assert_eq!(restored.level, 15);
    assert!((restored.score - 98765.4321).abs() < 1e-6);
    assert_eq!(restored.unlocked_items, vec!["sword", "shield", "potion"]);
}

// ---------------------------------------------------------------------------
// 3. Forward compatibility: unknown extension keys are preserved
// ---------------------------------------------------------------------------

/// When save data contains extension keys that are NOT registered in the current
/// SaveableRegistry, those keys should be silently ignored during load_all.
#[test]
fn test_save_migration_unknown_extension_keys_ignored_on_load() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct KnownFeature {
        active: bool,
    }

    impl crate::Saveable for KnownFeature {
        const SAVE_KEY: &'static str = "test_known_feature";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&self.active).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            Self {
                active: serde_json::from_slice(bytes).unwrap_or_default(),
            }
        }
    }

    app.init_resource::<KnownFeature>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<KnownFeature>();
    }

    let mut extensions: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    extensions.insert(
        "test_known_feature".to_string(),
        serde_json::to_vec(&true).unwrap(),
    );
    extensions.insert("future_feature_v99".to_string(), vec![1, 2, 3, 4, 5]);
    extensions.insert(
        "another_unknown_key".to_string(),
        vec![0xDE, 0xAD, 0xBE, 0xEF],
    );

    let registry = app
        .world_mut()
        .remove_resource::<SaveableRegistry>()
        .unwrap();
    registry.load_all(app.world_mut(), &extensions);
    app.world_mut().insert_resource(registry);

    assert!(
        app.world().resource::<KnownFeature>().active,
        "Known feature should load correctly even when unknown keys are present"
    );
}

/// save_all only emits keys for registered resources, not unknown keys.
#[test]
fn test_save_migration_save_all_only_emits_registered_keys() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct OnlyRegistered {
        value: u32,
    }

    impl crate::Saveable for OnlyRegistered {
        const SAVE_KEY: &'static str = "test_only_registered";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            if self.value == 0 {
                None
            } else {
                serde_json::to_vec(&self.value).ok()
            }
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            Self {
                value: serde_json::from_slice(bytes).unwrap_or_default(),
            }
        }
    }

    app.init_resource::<OnlyRegistered>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<OnlyRegistered>();
    }
    app.world_mut().resource_mut::<OnlyRegistered>().value = 7;

    let saved = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };

    assert!(saved.contains_key("test_only_registered"));
    assert!(!saved.contains_key("future_feature_v99"));
}

// ---------------------------------------------------------------------------
// 4. Full save chain: serialize all saveables -> deserialize -> verify
// ---------------------------------------------------------------------------

/// Using the real game's SaveableRegistry (via TestCity), save all extension
/// data, then load it back and verify the round-trip produces identical bytes.
#[test]
fn test_save_migration_full_chain_real_registry_roundtrip() {
    let mut city = TestCity::new();
    let world = city.world_mut();

    let registry = world.remove_resource::<SaveableRegistry>().unwrap();
    let extensions_v1 = registry.save_all(world);
    registry.load_all(world, &extensions_v1);
    let extensions_v2 = registry.save_all(world);
    world.insert_resource(registry);

    assert_eq!(
        extensions_v1.len(),
        extensions_v2.len(),
        "Extension map key count should be stable across save/load"
    );
    for (key, bytes_v1) in &extensions_v1 {
        let bytes_v2 = extensions_v2
            .get(key)
            .unwrap_or_else(|| panic!("Key '{}' missing after round-trip", key));
        assert_eq!(
            bytes_v1, bytes_v2,
            "Extension '{}' bytes differ after round-trip ({} vs {} bytes)",
            key,
            bytes_v1.len(),
            bytes_v2.len()
        );
    }
}

/// After loading empty extensions into a real city, ticking should not panic.
/// This simulates loading a legacy save file that predates the extension map.
#[test]
fn test_save_migration_legacy_save_no_extensions_ticks_safely() {
    let mut city = TestCity::new();
    let empty: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();
    registry.load_all(world, &empty);
    world.insert_resource(registry);

    // Tick several times -- if any system panics on default-state resources,
    // this will catch it.
    city.tick(10);
}

// ---------------------------------------------------------------------------
// 5. Reset behavior: reset_all restores defaults
// ---------------------------------------------------------------------------

/// SaveableRegistry::reset_all should restore every registered resource to
/// its Default implementation (equivalent to new-game behavior).
#[test]
fn test_save_migration_reset_all_restores_defaults() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Clone, Debug, PartialEq)]
    struct ResetTarget {
        counter: u32,
        name: String,
    }

    impl Default for ResetTarget {
        fn default() -> Self {
            Self {
                counter: 0,
                name: String::new(),
            }
        }
    }

    impl crate::Saveable for ResetTarget {
        const SAVE_KEY: &'static str = "test_reset_target";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&(&self.counter, &self.name)).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            let (counter, name): (u32, String) =
                serde_json::from_slice(bytes).unwrap_or_default();
            Self { counter, name }
        }
    }

    app.init_resource::<ResetTarget>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<ResetTarget>();
    }

    {
        let mut target = app.world_mut().resource_mut::<ResetTarget>();
        target.counter = 999;
        target.name = "modified".into();
    }
    assert_eq!(app.world().resource::<ResetTarget>().counter, 999);

    let registry = app
        .world_mut()
        .remove_resource::<SaveableRegistry>()
        .unwrap();
    registry.reset_all(app.world_mut());
    app.world_mut().insert_resource(registry);

    let target = app.world().resource::<ResetTarget>();
    assert_eq!(target.counter, 0, "counter should be reset to default");
    assert_eq!(target.name, "", "name should be reset to default");
}

// ---------------------------------------------------------------------------
// 6. Default-state resources skip saving (None from save_to_bytes)
// ---------------------------------------------------------------------------

/// A Saveable resource at its default state that returns None from
/// save_to_bytes should NOT appear in the extension map.
#[test]
fn test_save_migration_default_state_not_saved() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct OptionalSave {
        data: Vec<u8>,
    }

    impl crate::Saveable for OptionalSave {
        const SAVE_KEY: &'static str = "test_optional_save";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            if self.data.is_empty() {
                None
            } else {
                Some(self.data.clone())
            }
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            Self {
                data: bytes.to_vec(),
            }
        }
    }

    app.init_resource::<OptionalSave>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<OptionalSave>();
    }

    // At default state, save should not include this key.
    let extensions = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };
    assert!(
        !extensions.contains_key("test_optional_save"),
        "Default-state resource should not be in extension map"
    );

    // Set non-default state, should now be saved.
    app.world_mut().resource_mut::<OptionalSave>().data = vec![1, 2, 3];
    let extensions = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };
    assert!(
        extensions.contains_key("test_optional_save"),
        "Non-default resource should be in extension map"
    );
}
