// ===========================================================================
// Tests for SaveError integration with decode/migrate paths (issue #705)
// ===========================================================================

use crate::save_error::SaveError;
use crate::save_migrate::migrate_save;
use crate::save_types::*;
use std::collections::BTreeMap;

/// Helper to create a minimal SaveData for testing.
fn minimal_save(version: u32) -> SaveData {
    SaveData {
        version,
        grid: SaveGrid {
            cells: vec![],
            width: 1,
            height: 1,
        },
        roads: SaveRoadNetwork {
            road_positions: vec![],
        },
        clock: SaveClock {
            day: 0,
            hour: 0.0,
            speed: 1.0,
        },
        budget: SaveBudget {
            treasury: 0.0,
            tax_rate: 0.0,
            last_collection_day: 0,
        },
        demand: SaveDemand {
            residential: 0.0,
            commercial: 0.0,
            industrial: 0.0,
            office: 0.0,
            vacancy_residential: 0.0,
            vacancy_commercial: 0.0,
            vacancy_industrial: 0.0,
            vacancy_office: 0.0,
        },
        buildings: vec![],
        citizens: vec![],
        utility_sources: vec![],
        service_buildings: vec![],
        road_segments: None,
        policies: None,
        weather: None,
        unlock_state: None,
        extended_budget: None,
        loan_book: None,
        lifecycle_timer: None,
        virtual_population: None,
        life_sim_timer: None,
        stormwater_grid: None,
        water_sources: None,
        degree_days: None,
        construction_modifiers: None,
        recycling_state: None,
        wind_damage_state: None,
        uhi_grid: None,
        drought_state: None,
        heat_wave_state: None,
        composting_state: None,
        cold_snap_state: None,
        water_treatment_state: None,
        groundwater_depletion_state: None,
        wastewater_state: None,
        hazardous_waste_state: None,
        storm_drainage_state: None,
        landfill_capacity_state: None,
        flood_state: None,
        reservoir_state: None,
        landfill_gas_state: None,
        cso_state: None,
        water_conservation_state: None,
        fog_state: None,
        urban_growth_boundary: None,
        snow_state: None,
        agriculture_state: None,
        extensions: BTreeMap::new(),
    }
}

// ---------------------------------------------------------------------------
// Decode error tests
// ---------------------------------------------------------------------------

#[test]
fn test_decode_garbage_bytes_returns_error() {
    let garbage = vec![0xFF, 0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x42];
    let result = SaveData::decode(&garbage);
    assert!(result.is_err(), "Decoding garbage should fail");

    // Verify the bitcode::Error converts to SaveError::Decode
    let bitcode_err = match result {
        Err(e) => e,
        Ok(_) => panic!("Expected error"),
    };
    let save_err: SaveError = bitcode_err.into();
    assert!(
        matches!(save_err, SaveError::Decode(_)),
        "Should be Decode variant, got: {save_err:?}"
    );
}

#[test]
fn test_decode_empty_bytes_returns_error() {
    let result = SaveData::decode(&[]);
    assert!(result.is_err(), "Decoding empty bytes should fail");
}

#[test]
fn test_decode_truncated_save_returns_error() {
    // Encode a valid save, then truncate it
    let save = minimal_save(CURRENT_SAVE_VERSION);
    let bytes = save.encode();
    assert!(bytes.len() > 4, "Encoded save should have some bytes");

    // Truncate to just 4 bytes (not enough for a full save)
    let truncated = &bytes[..4];
    let result = SaveData::decode(truncated);
    assert!(result.is_err(), "Decoding truncated save should fail");
}

// ---------------------------------------------------------------------------
// Migration error tests
// ---------------------------------------------------------------------------

#[test]
fn test_migrate_future_version_returns_version_mismatch() {
    let future_version = CURRENT_SAVE_VERSION + 10;
    let mut save = minimal_save(future_version);

    let result = migrate_save(&mut save);
    assert!(result.is_err());

    match result.unwrap_err() {
        SaveError::VersionMismatch {
            expected_max,
            found,
        } => {
            assert_eq!(expected_max, CURRENT_SAVE_VERSION);
            assert_eq!(found, future_version);
        }
        other => panic!("Expected VersionMismatch, got: {other:?}"),
    }
}

#[test]
fn test_migrate_version_mismatch_display_contains_versions() {
    let mut save = minimal_save(CURRENT_SAVE_VERSION + 1);
    let err = migrate_save(&mut save).unwrap_err();
    let msg = format!("{err}");

    assert!(
        msg.contains(&format!("v{}", CURRENT_SAVE_VERSION)),
        "Display should contain expected max version, got: {msg}"
    );
    assert!(
        msg.contains(&format!("v{}", CURRENT_SAVE_VERSION + 1)),
        "Display should contain found version, got: {msg}"
    );
}

#[test]
fn test_migrate_current_version_succeeds() {
    let mut save = minimal_save(CURRENT_SAVE_VERSION);
    let result = migrate_save(&mut save);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), CURRENT_SAVE_VERSION);
}

#[test]
fn test_migrate_old_version_succeeds_and_bumps() {
    let mut save = minimal_save(0);
    let result = migrate_save(&mut save);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0); // returns original version
    assert_eq!(save.version, CURRENT_SAVE_VERSION); // bumped to current
}

// ---------------------------------------------------------------------------
// Encode -> decode roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_encode_decode_roundtrip_succeeds() {
    let save = minimal_save(CURRENT_SAVE_VERSION);
    let bytes = save.encode();
    let result = SaveData::decode(&bytes);
    assert!(result.is_ok(), "Roundtrip should succeed");
    match result {
        Ok(decoded) => assert_eq!(decoded.version, CURRENT_SAVE_VERSION),
        Err(e) => panic!("Decode failed: {e}"),
    }
}

// ---------------------------------------------------------------------------
// SaveError variant coverage
// ---------------------------------------------------------------------------

#[test]
fn test_save_error_io_from_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let save_err: SaveError = io_err.into();
    assert!(matches!(save_err, SaveError::Io(_)));

    // Verify std::error::Error::source works
    let dyn_err: &dyn std::error::Error = &save_err;
    assert!(dyn_err.source().is_some());
}

#[test]
fn test_save_error_no_data_display() {
    let err = SaveError::NoData;
    let msg = format!("{err}");
    assert!(msg.contains("No save data"), "got: {msg}");
}

#[test]
fn test_save_error_missing_resource_display() {
    let err = SaveError::MissingResource("WorldGrid".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("WorldGrid"), "got: {msg}");
    assert!(msg.contains("Missing"), "got: {msg}");
}

#[test]
fn test_save_error_encode_display() {
    let err = SaveError::Encode("serialization overflow".to_string());
    let msg = format!("{err}");
    assert!(msg.contains("Encoding error"), "got: {msg}");
    assert!(msg.contains("serialization overflow"), "got: {msg}");
}
