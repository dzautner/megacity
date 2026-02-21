// ---------------------------------------------------------------------------
// Extension map binary roundtrip
// ---------------------------------------------------------------------------

/// Test that extension map data survives a serde_json encode->decode roundtrip.
#[test]
fn test_extension_map_bytes_survive_serde_roundtrip() {
    use crate::SaveableRegistry;
    use std::collections::BTreeMap;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct ComplexState {
        values: Vec<u32>,
        label: String,
        active: bool,
    }

    impl crate::Saveable for ComplexState {
        const SAVE_KEY: &'static str = "test_complex_state";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&(&self.values, &self.label, self.active)).ok()
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let (values, label, active): (Vec<u32>, String, bool) =
                serde_json::from_slice(bytes).unwrap_or_default();
            Self {
                values,
                label,
                active,
            }
        }
    }

    app.init_resource::<ComplexState>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<ComplexState>();
    }

    {
        let mut state = app.world_mut().resource_mut::<ComplexState>();
        state.values = vec![10, 20, 30, 40, 50];
        state.label = "roundtrip_binary_test".to_string();
        state.active = true;
    }

    let extensions = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };
    let saved_bytes = extensions.get("test_complex_state").unwrap().clone();

    let mut restored_extensions: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    restored_extensions.insert("test_complex_state".to_string(), saved_bytes);

    app.world_mut().insert_resource(ComplexState::default());
    assert!(app.world().resource::<ComplexState>().values.is_empty());

    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &restored_extensions);
        app.world_mut().insert_resource(registry);
    }

    let state = app.world().resource::<ComplexState>();
    assert_eq!(state.values, vec![10, 20, 30, 40, 50]);
    assert_eq!(state.label, "roundtrip_binary_test");
    assert!(state.active);
}

/// Test that loading extensions with corrupted bytes falls back to defaults.
#[test]
fn test_extension_map_corrupted_bytes_fall_back_to_default() {
    use crate::SaveableRegistry;
    use std::collections::BTreeMap;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct SimpleCounter {
        count: u32,
    }

    impl crate::Saveable for SimpleCounter {
        const SAVE_KEY: &'static str = "test_simple_counter";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&self.count).ok()
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let count: u32 = serde_json::from_slice(bytes).unwrap_or_default();
            Self { count }
        }
    }

    app.init_resource::<SimpleCounter>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<SimpleCounter>();
    }

    app.world_mut().resource_mut::<SimpleCounter>().count = 42;

    let mut extensions: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    extensions.insert(
        "test_simple_counter".to_string(),
        vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB],
    );

    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &extensions);
        app.world_mut().insert_resource(registry);
    }

    assert_eq!(
        app.world().resource::<SimpleCounter>().count,
        0,
        "Corrupted bytes should cause fallback to default, not retain stale value"
    );
}
