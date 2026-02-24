// ===========================================================================
// Stale extension state tests (issue #1603)
//
// Verifies that sequential loads properly reset Saveable-registered resources
// when extension keys are absent in the loaded save. Uses REAL Saveable types
// from the full simulation (not test-only stubs) within a TestCity context.
// ===========================================================================

use crate::climate_change::state::ClimateState;
use crate::heating_service::HeatingServiceState;
use crate::road_hierarchy::{HierarchyViolation, RoadHierarchyState};
use crate::SaveableRegistry;
use std::collections::BTreeMap;

/// Helper: snapshot the full extension map from the registry within a TestCity.
fn snapshot_extensions(city: &mut crate::test_harness::TestCity) -> BTreeMap<String, Vec<u8>> {
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();
    let extensions = registry.save_all(world);
    world.insert_resource(registry);
    extensions
}

/// Helper: load an extension map into the TestCity's SaveableRegistry.
fn load_extensions(
    city: &mut crate::test_harness::TestCity,
    extensions: &BTreeMap<String, Vec<u8>>,
) {
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();
    registry.load_all(world, extensions);
    world.insert_resource(registry);
}

/// Load Save A (with non-default ClimateState), then Load Save B (empty
/// extensions). ClimateState must reset to default, not retain Save A's value.
#[test]
fn test_sequential_load_resets_climate_state_when_absent_in_save_b() {
    let mut city = crate::test_harness::TestCity::new();

    // Mutate ClimateState to a non-default value (simulating Save A's state)
    {
        let mut climate = city.world_mut().resource_mut::<ClimateState>();
        climate.cumulative_co2 = 500_000.0;
        climate.temperature_increase_f = 2.5;
        climate.sea_level_rise_applied = true;
        climate.flooded_cells_count = 42;
    }

    // Snapshot Save A (should contain "climate_change" key)
    let save_a = snapshot_extensions(&mut city);
    assert!(
        save_a.contains_key("climate_change"),
        "Save A should have climate_change extension key"
    );

    // Build Save B: completely empty (simulates an older save with no extensions)
    let save_b: BTreeMap<String, Vec<u8>> = BTreeMap::new();

    // Load Save B (empty extensions)
    load_extensions(&mut city, &save_b);

    // ClimateState must be reset to default
    let climate = city.world_mut().resource::<ClimateState>();
    assert_eq!(
        climate.cumulative_co2, 0.0,
        "cumulative_co2 must reset to default after loading save with no climate key"
    );
    assert_eq!(
        climate.temperature_increase_f, 0.0,
        "temperature_increase_f must reset to default"
    );
    assert!(
        !climate.sea_level_rise_applied,
        "sea_level_rise_applied must reset to false"
    );
    assert_eq!(
        climate.flooded_cells_count, 0,
        "flooded_cells_count must reset to 0"
    );
}

/// Full A -> B -> A cycle: load Save A, then Save B (empty), then Save A again.
/// After the second load of A, values from A must be fully restored.
#[test]
fn test_sequential_load_a_b_a_cycle_with_real_saveable_types() {
    let mut city = crate::test_harness::TestCity::new();

    // Set non-default values for ClimateState and HeatingServiceState
    {
        let mut climate = city.world_mut().resource_mut::<ClimateState>();
        climate.cumulative_co2 = 1_000_000.0;
        climate.yearly_co2 = 50_000.0;
        climate.temperature_increase_f = 3.0;
        climate.disaster_frequency_multiplier = 1.15;
    }
    {
        let mut heating = city.world_mut().resource_mut::<HeatingServiceState>();
        heating.individual_heating_count = 200;
        heating.district_heating_count = 15;
        heating.heating_energy_mw = 450.0;
        heating.cold_affected_citizens = 30;
    }

    // Snapshot Save A
    let save_a = snapshot_extensions(&mut city);
    assert!(save_a.contains_key("climate_change"));
    assert!(save_a.contains_key("heating_service"));

    // Build Save B: only has heating_service, no climate_change
    let mut save_b: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    // HeatingServiceState always saves, so create a default heating state.
    {
        let default_heating = HeatingServiceState::default();
        if let Some(bytes) = crate::Saveable::save_to_bytes(&default_heating) {
            save_b.insert("heating_service".to_string(), bytes);
        }
    }
    // Save B has NO climate_change key

    // --- Load Save B ---
    load_extensions(&mut city, &save_b);

    // ClimateState must be reset (key absent)
    {
        let climate = city.world_mut().resource::<ClimateState>();
        assert_eq!(
            climate.cumulative_co2, 0.0,
            "After loading Save B, ClimateState.cumulative_co2 should be default"
        );
        assert_eq!(climate.temperature_increase_f, 0.0);
    }
    // HeatingServiceState must reflect Save B's data (default values)
    {
        let heating = city.world_mut().resource::<HeatingServiceState>();
        assert_eq!(
            heating.individual_heating_count, 0,
            "After loading Save B, HeatingServiceState should be default"
        );
        assert_eq!(heating.cold_affected_citizens, 0);
    }

    // --- Load Save A again ---
    load_extensions(&mut city, &save_a);

    // ClimateState must be fully restored from Save A
    {
        let climate = city.world_mut().resource::<ClimateState>();
        assert_eq!(
            climate.cumulative_co2, 1_000_000.0,
            "After re-loading Save A, ClimateState.cumulative_co2 must be restored"
        );
        assert!(
            (climate.temperature_increase_f - 3.0).abs() < f32::EPSILON,
            "temperature_increase_f must be 3.0 from Save A"
        );
        assert!(
            (climate.disaster_frequency_multiplier - 1.15).abs() < 0.001,
            "disaster_frequency_multiplier must be 1.15 from Save A"
        );
    }
    // HeatingServiceState must be fully restored from Save A
    {
        let heating = city.world_mut().resource::<HeatingServiceState>();
        assert_eq!(
            heating.individual_heating_count, 200,
            "After re-loading Save A, individual_heating_count must be 200"
        );
        assert_eq!(heating.district_heating_count, 15);
        assert_eq!(heating.cold_affected_citizens, 30);
    }
}

/// When a Saveable type skips saving at default (returns None from
/// save_to_bytes), that key should be absent from the extension map.
/// Loading such a save must still reset other resources that were non-default.
#[test]
fn test_saveable_skip_default_causes_reset_on_load() {
    let mut city = crate::test_harness::TestCity::new();

    // RoadHierarchyState skips saving when violations is empty.
    // First, verify that a default RoadHierarchyState produces no key.
    let snapshot_default = snapshot_extensions(&mut city);
    assert!(
        !snapshot_default.contains_key("road_hierarchy"),
        "Default RoadHierarchyState should NOT produce an extension key"
    );

    // Now set a non-default value
    {
        let mut rh = city.world_mut().resource_mut::<RoadHierarchyState>();
        rh.violations.push(HierarchyViolation {
            node_id: 1,
            grid_x: 10,
            grid_y: 20,
            low_segment_id: 100,
            high_segment_id: 200,
            low_road_type: 0,
            high_road_type: 3,
            levels_skipped: 2,
        });
    }

    // Snapshot Save A: should now contain road_hierarchy
    let save_a = snapshot_extensions(&mut city);
    assert!(
        save_a.contains_key("road_hierarchy"),
        "Non-default RoadHierarchyState should produce extension key"
    );

    // Load from a save with no road_hierarchy key (empty)
    let save_b: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    load_extensions(&mut city, &save_b);

    // road_hierarchy must be reset to default (empty violations)
    let rh = city.world_mut().resource::<RoadHierarchyState>();
    assert!(
        rh.violations.is_empty(),
        "RoadHierarchyState.violations must be empty after loading save with no key"
    );
}

/// Verify that ALL registered Saveable resources are reset when loading
/// a completely empty extension map. This is a broad safety net test.
#[test]
fn test_empty_extension_map_resets_all_registered_resources() {
    let mut city = crate::test_harness::TestCity::new();

    // Count how many entries are registered
    let entry_count = {
        let registry = city.world_mut().resource::<SaveableRegistry>();
        registry.entries.len()
    };
    assert!(
        entry_count > 0,
        "SaveableRegistry should have registered entries in a full TestCity"
    );

    // Mutate a few known resources to non-default values
    city.world_mut()
        .resource_mut::<ClimateState>()
        .cumulative_co2 = 999.0;
    city.world_mut()
        .resource_mut::<HeatingServiceState>()
        .cold_affected_citizens = 77;

    // Load completely empty extension map
    let empty: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    load_extensions(&mut city, &empty);

    // Verify the mutated resources are back to default
    assert_eq!(
        city.world_mut().resource::<ClimateState>().cumulative_co2,
        0.0,
        "ClimateState should be default after loading empty extensions"
    );
    assert_eq!(
        city.world_mut()
            .resource::<HeatingServiceState>()
            .cold_affected_citizens,
        0,
        "HeatingServiceState should be default after loading empty extensions"
    );
}

/// Verify that extension keys present in the save but NOT registered in the
/// SaveableRegistry are silently ignored (no panic, no data corruption).
#[test]
fn test_unknown_extension_keys_ignored_without_panic() {
    let mut city = crate::test_harness::TestCity::new();

    // Build a save with a totally unknown key
    let mut extensions: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    extensions.insert(
        "nonexistent_future_feature".to_string(),
        vec![1, 2, 3, 4, 5],
    );
    // Also include a valid key
    {
        let mut heating = HeatingServiceState::default();
        heating.cold_affected_citizens = 10;
        if let Some(bytes) = crate::Saveable::save_to_bytes(&heating) {
            extensions.insert("heating_service".to_string(), bytes);
        }
    }

    // Load should not panic
    load_extensions(&mut city, &extensions);

    // Valid key should be applied
    assert_eq!(
        city.world_mut()
            .resource::<HeatingServiceState>()
            .cold_affected_citizens,
        10,
        "Valid extension key should be applied despite unknown keys in the map"
    );
}
