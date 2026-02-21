// ===========================================================================
// Behavioral integration tests (issue #1248)
// ===========================================================================

// ---------------------------------------------------------------------------
// Save/load: sequential load A -> B -> A (extension map cross-save safety)
// ---------------------------------------------------------------------------

/// Test that loading save A (with extension data), then save B (without that
/// extension key), correctly resets the extension resource to its default.
/// Then loading save A again must restore the original value -- not retain
/// save B's empty state.
#[test]
fn test_extension_map_sequential_load_a_b_a_restores_correctly() {
    use crate::SaveableRegistry;
    use std::collections::BTreeMap;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct FeatureAlpha {
        level: u32,
        name: String,
    }

    impl crate::Saveable for FeatureAlpha {
        const SAVE_KEY: &'static str = "test_feature_alpha";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&(self.level, &self.name)).ok()
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let (level, name): (u32, String) = serde_json::from_slice(bytes).unwrap_or_default();
            Self { level, name }
        }
    }

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct FeatureBeta {
        score: f64,
    }

    impl crate::Saveable for FeatureBeta {
        const SAVE_KEY: &'static str = "test_feature_beta";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&self.score).ok()
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let score: f64 = serde_json::from_slice(bytes).unwrap_or_default();
            Self { score }
        }
    }

    app.init_resource::<FeatureAlpha>();
    app.init_resource::<FeatureBeta>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<FeatureAlpha>();
        registry.register::<FeatureBeta>();
    }

    // --- Build save A: both features have data ---
    {
        let mut alpha = app.world_mut().resource_mut::<FeatureAlpha>();
        alpha.level = 7;
        alpha.name = "save_a_alpha".to_string();
    }
    {
        let mut beta = app.world_mut().resource_mut::<FeatureBeta>();
        beta.score = 99.5;
    }
    let save_a = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };
    assert_eq!(save_a.len(), 2, "save A should contain both extensions");

    // --- Build save B: only FeatureBeta has data; FeatureAlpha key is absent ---
    let mut save_b: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    save_b.insert(
        "test_feature_beta".to_string(),
        serde_json::to_vec(&42.0_f64).unwrap(),
    );

    // --- Load save A ---
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &save_a);
        app.world_mut().insert_resource(registry);
    }
    assert_eq!(app.world().resource::<FeatureAlpha>().level, 7);
    assert_eq!(app.world().resource::<FeatureAlpha>().name, "save_a_alpha");
    assert!((app.world().resource::<FeatureBeta>().score - 99.5).abs() < f64::EPSILON);

    // --- Load save B (missing FeatureAlpha) ---
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &save_b);
        app.world_mut().insert_resource(registry);
    }
    assert_eq!(
        app.world().resource::<FeatureAlpha>().level,
        0,
        "FeatureAlpha.level should reset to default after loading save B (key absent)"
    );
    assert!(
        app.world().resource::<FeatureAlpha>().name.is_empty(),
        "FeatureAlpha.name should reset to default after loading save B (key absent)"
    );
    assert!(
        (app.world().resource::<FeatureBeta>().score - 42.0).abs() < f64::EPSILON,
        "FeatureBeta.score should be 42.0 from save B"
    );

    // --- Load save A again ---
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &save_a);
        app.world_mut().insert_resource(registry);
    }
    assert_eq!(
        app.world().resource::<FeatureAlpha>().level,
        7,
        "FeatureAlpha.level should be restored from save A after A->B->A sequence"
    );
    assert_eq!(
        app.world().resource::<FeatureAlpha>().name,
        "save_a_alpha",
        "FeatureAlpha.name should be restored from save A after A->B->A sequence"
    );
    assert!(
        (app.world().resource::<FeatureBeta>().score - 99.5).abs() < f64::EPSILON,
        "FeatureBeta.score should be restored from save A after A->B->A sequence"
    );
}

/// Test that loading a save with completely empty extensions resets ALL
/// registered saveable resources to defaults.
#[test]
fn test_extension_map_load_empty_save_resets_all_to_defaults() {
    use crate::SaveableRegistry;
    use std::collections::BTreeMap;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct CounterRes {
        count: u64,
    }

    impl crate::Saveable for CounterRes {
        const SAVE_KEY: &'static str = "test_counter_res";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&self.count).ok()
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let count: u64 = serde_json::from_slice(bytes).unwrap_or_default();
            Self { count }
        }
    }

    app.init_resource::<CounterRes>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<CounterRes>();
    }

    app.world_mut().resource_mut::<CounterRes>().count = 12345;

    let empty_extensions: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &empty_extensions);
        app.world_mut().insert_resource(registry);
    }

    assert_eq!(
        app.world().resource::<CounterRes>().count,
        0,
        "CounterRes should reset to default when loading a save with no extension data"
    );
}
