use crate::test_harness::TestCity;

#[test]
fn test_saveable_registry_contains_all_expected_keys() {
    let city = TestCity::new();
    let registry = city.resource::<crate::SaveableRegistry>();

    let registered: std::collections::HashSet<&str> =
        registry.entries.iter().map(|e| e.key.as_str()).collect();

    // Every key in EXPECTED_SAVEABLE_KEYS must be registered.
    let mut missing = Vec::new();
    for &expected in crate::EXPECTED_SAVEABLE_KEYS {
        if !registered.contains(expected) {
            missing.push(expected);
        }
    }
    assert!(
        missing.is_empty(),
        "SaveableRegistry is missing {} expected key(s): {:?}. \
         Each type implementing `Saveable` must be registered via `register_saveable` \
         in its plugin's `build()` method.",
        missing.len(),
        missing,
    );

    // Every registered key must be in the expected list (catches stale entries
    // in EXPECTED_SAVEABLE_KEYS or unexpected registrations).
    let expected_set: std::collections::HashSet<&str> =
        crate::EXPECTED_SAVEABLE_KEYS.iter().copied().collect();
    let mut unexpected: Vec<&str> = registered.difference(&expected_set).copied().collect();
    unexpected.sort();
    assert!(
        unexpected.is_empty(),
        "SaveableRegistry contains {} key(s) not in EXPECTED_SAVEABLE_KEYS: {:?}. \
         Add them to the list in simulation/src/lib.rs.",
        unexpected.len(),
        unexpected,
    );
}

#[test]
fn test_saveable_registry_has_no_duplicate_keys() {
    let city = TestCity::new();
    let registry = city.resource::<crate::SaveableRegistry>();

    let mut seen = std::collections::HashSet::new();
    for entry in &registry.entries {
        assert!(
            seen.insert(entry.key.as_str()),
            "SaveableRegistry: duplicate key '{}' — two types share the same SAVE_KEY",
            entry.key,
        );
    }
}
