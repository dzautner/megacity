//! Integration tests for DistrictMap save/load (SAVE-029).

use crate::districts::DistrictMap;
use crate::SaveableRegistry;
use crate::Saveable;

/// Test that an empty/default DistrictMap returns None from save_to_bytes
/// (skipped when no changes have been made).
#[test]
fn test_district_map_save_default_returns_none() {
    let map = DistrictMap::default();
    assert!(
        map.save_to_bytes().is_none(),
        "Default DistrictMap should skip saving"
    );
}

/// Test that a DistrictMap with cell assignments round-trips through
/// save_to_bytes / load_from_bytes correctly.
#[test]
fn test_district_map_save_load_round_trip() {
    let mut map = DistrictMap::default();
    // Assign some cells to districts
    map.assign_cell_to_district(10, 20, 0);
    map.assign_cell_to_district(11, 20, 0);
    map.assign_cell_to_district(50, 50, 1);
    map.assign_cell_to_district(100, 100, 2);

    let bytes = map
        .save_to_bytes()
        .expect("DistrictMap with assignments should produce bytes");

    let loaded = DistrictMap::load_from_bytes(&bytes);

    // Verify cell assignments are preserved
    assert_eq!(loaded.get_district_index_at(10, 20), Some(0));
    assert_eq!(loaded.get_district_index_at(11, 20), Some(0));
    assert_eq!(loaded.get_district_index_at(50, 50), Some(1));
    assert_eq!(loaded.get_district_index_at(100, 100), Some(2));

    // Verify unassigned cells remain None
    assert_eq!(loaded.get_district_index_at(0, 0), None);
    assert_eq!(loaded.get_district_index_at(200, 200), None);

    // Verify district cell sets are consistent
    assert!(loaded.districts[0].cells.contains(&(10, 20)));
    assert!(loaded.districts[0].cells.contains(&(11, 20)));
    assert!(loaded.districts[1].cells.contains(&(50, 50)));
    assert!(loaded.districts[2].cells.contains(&(100, 100)));
}

/// Test that district names are preserved across save/load.
#[test]
fn test_district_map_save_load_preserves_names() {
    let mut map = DistrictMap::default();
    map.districts[0].name = "My Custom District".to_string();
    map.districts[1].name = "Uptown".to_string();
    // Need at least one cell assigned so save_to_bytes doesn't skip
    map.assign_cell_to_district(5, 5, 0);

    let bytes = map.save_to_bytes().expect("Should produce bytes");
    let loaded = DistrictMap::load_from_bytes(&bytes);

    assert_eq!(loaded.districts[0].name, "My Custom District");
    assert_eq!(loaded.districts[1].name, "Uptown");
}

/// Test that district policies are preserved across save/load.
#[test]
fn test_district_map_save_load_preserves_policies() {
    let mut map = DistrictMap::default();
    map.districts[0].policies.tax_rate = Some(0.15);
    map.districts[0].policies.noise_ordinance = true;
    map.districts[1].policies.heavy_industry_ban = true;
    map.districts[2].policies.speed_limit = Some(30.0);

    // Assign a cell so save doesn't skip
    map.assign_cell_to_district(1, 1, 0);

    let bytes = map.save_to_bytes().expect("Should produce bytes");
    let loaded = DistrictMap::load_from_bytes(&bytes);

    assert_eq!(loaded.districts[0].policies.tax_rate, Some(0.15));
    assert!(loaded.districts[0].policies.noise_ordinance);
    assert!(loaded.districts[1].policies.heavy_industry_ban);
    assert_eq!(loaded.districts[2].policies.speed_limit, Some(30.0));
    // Default policies should remain default
    assert!(!loaded.districts[2].policies.noise_ordinance);
    assert!(!loaded.districts[0].policies.heavy_industry_ban);
}

/// Test that district colors are preserved across save/load.
#[test]
fn test_district_map_save_load_preserves_colors() {
    let mut map = DistrictMap::default();
    map.districts[0].color = [1.0, 0.0, 0.0, 1.0];
    map.assign_cell_to_district(1, 1, 0);

    let bytes = map.save_to_bytes().expect("Should produce bytes");
    let loaded = DistrictMap::load_from_bytes(&bytes);

    assert_eq!(loaded.districts[0].color, [1.0, 0.0, 0.0, 1.0]);
}

/// Test that DistrictMap works correctly through the SaveableRegistry
/// (end-to-end save/load via the registry).
#[test]
fn test_district_map_saveable_registry_round_trip() {
    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();
    app.init_resource::<DistrictMap>();

    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<DistrictMap>();
    }

    // Modify the district map
    {
        let mut map = app.world_mut().resource_mut::<DistrictMap>();
        map.assign_cell_to_district(30, 40, 0);
        map.assign_cell_to_district(31, 40, 0);
        map.districts[0].name = "Test District".to_string();
        map.districts[0].policies.tax_rate = Some(0.10);
    }

    // Save via registry
    let extensions = {
        let registry = app.world().resource::<SaveableRegistry>();
        registry.save_all(app.world())
    };
    assert!(
        extensions.contains_key("district_map"),
        "district_map key should be present in extensions"
    );

    // Reset the district map
    app.world_mut().insert_resource(DistrictMap::default());

    // Verify reset
    {
        let map = app.world().resource::<DistrictMap>();
        assert_eq!(map.get_district_index_at(30, 40), None);
        assert_eq!(map.districts[0].name, "Downtown");
    }

    // Load via registry
    {
        let registry = app
            .world_mut()
            .remove_resource::<SaveableRegistry>()
            .unwrap();
        registry.load_all(app.world_mut(), &extensions);
        app.world_mut().insert_resource(registry);
    }

    // Verify restored state
    {
        let map = app.world().resource::<DistrictMap>();
        assert_eq!(map.get_district_index_at(30, 40), Some(0));
        assert_eq!(map.get_district_index_at(31, 40), Some(0));
        assert_eq!(map.districts[0].name, "Test District");
        assert_eq!(map.districts[0].policies.tax_rate, Some(0.10));
        assert!(map.districts[0].cells.contains(&(30, 40)));
        assert!(map.districts[0].cells.contains(&(31, 40)));
    }
}

/// Test that the SAVE_KEY constant matches what we expect.
#[test]
fn test_district_map_save_key() {
    assert_eq!(DistrictMap::SAVE_KEY, "district_map");
}

/// Test that loading from empty/garbage bytes returns a valid default.
#[test]
fn test_district_map_load_from_invalid_bytes_returns_default() {
    let loaded = DistrictMap::load_from_bytes(&[0xFF, 0xFE, 0xFD]);
    // Should return default without panicking
    assert_eq!(loaded.districts.len(), 8); // DEFAULT_DISTRICTS count
    assert!(loaded.cell_map.iter().all(|c| c.is_none()));
}
