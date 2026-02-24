//! Comprehensive migration chain tests: validates that every version transition
//! works correctly, sample save data roundtrips through migration, and the
//! migration chain is structurally sound.

use super::*;

use crate::save_error::SaveError;
use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::roads::RoadNetwork;
use simulation::time_of_day::GameClock;
use simulation::zones::ZoneDemand;

/// Creates a SaveData via the real `create_save_data` pipeline (not the
/// minimal test helper) and forces it to a specific version for testing.
fn real_save_at_version(version: u32) -> SaveData {
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock::default();
    let budget = CityBudget::default();
    let demand = ZoneDemand::default();

    let mut save = create_save_data(
        &grid,
        &roads,
        &clock,
        &budget,
        &demand,
        &[],
        &[],
        &[],
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    save.version = version;
    save
}

/// Verify every version from 0 to CURRENT can be migrated via the real
/// `create_save_data` pipeline (not just minimal_save).
#[test]
fn test_real_save_migration_chain_v0_to_current() {
    for v in 0..=CURRENT_SAVE_VERSION {
        let mut save = real_save_at_version(v);
        let old = migrate_save(&mut save).expect(&format!("v{v} migration should succeed"));
        assert_eq!(old, v);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }
}

/// Migrate from v0, encode, decode, verify version survives the roundtrip.
#[test]
fn test_v0_migrate_encode_decode_roundtrip() {
    let mut save = real_save_at_version(0);
    migrate_save(&mut save).unwrap();
    assert_eq!(save.version, CURRENT_SAVE_VERSION);

    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");
    assert_eq!(restored.version, CURRENT_SAVE_VERSION);
}

/// Migrate from mid-range version, encode, decode, verify version roundtrips.
#[test]
fn test_midrange_migrate_encode_decode_roundtrip() {
    let mid = CURRENT_SAVE_VERSION / 2;
    let mut save = real_save_at_version(mid);
    migrate_save(&mut save).unwrap();
    assert_eq!(save.version, CURRENT_SAVE_VERSION);

    let bytes = save.encode();
    let restored = SaveData::decode(&bytes).expect("decode should succeed");
    assert_eq!(restored.version, CURRENT_SAVE_VERSION);
}

/// Test that migrating from v0 sets all optional fields to None (default).
#[test]
fn test_v0_migration_defaults() {
    let mut save = real_save_at_version(0);
    migrate_save(&mut save).unwrap();

    // All optional infrastructure fields should be None since we created
    // a save with no optional data.
    assert!(save.policies.is_none());
    assert!(save.weather.is_none());
    assert!(save.unlock_state.is_none());
    assert!(save.stormwater_grid.is_none());
    assert!(save.drought_state.is_none());
    assert!(save.flood_state.is_none());
    assert!(save.snow_state.is_none());
    assert!(save.agriculture_state.is_none());
    assert!(save.fog_state.is_none());
}

/// Verify that future version +1 is rejected with VersionMismatch error.
#[test]
fn test_future_version_returns_version_mismatch() {
    let future = CURRENT_SAVE_VERSION + 1;
    let mut save = real_save_at_version(future);
    let err = migrate_save(&mut save).unwrap_err();
    assert!(
        matches!(err, SaveError::VersionMismatch { expected_max, found }
            if expected_max == CURRENT_SAVE_VERSION && found == future),
        "Expected VersionMismatch, got: {err:?}"
    );
}

/// Verify that future version far ahead is also rejected.
#[test]
fn test_far_future_version_rejected() {
    let mut save = real_save_at_version(CURRENT_SAVE_VERSION + 999);
    assert!(matches!(
        migrate_save(&mut save).unwrap_err(),
        SaveError::VersionMismatch { .. }
    ));
}

/// Test the detailed migration report API.
#[test]
fn test_migration_report_details() {
    let mut save = real_save_at_version(0);
    let report = migrate_save_with_report(&mut save).unwrap();

    assert_eq!(report.original_version, 0);
    assert_eq!(report.final_version, CURRENT_SAVE_VERSION);
    assert_eq!(report.steps_applied, CURRENT_SAVE_VERSION);

    // Each step should have a non-empty description
    for (i, desc) in report.step_descriptions.iter().enumerate() {
        assert!(!desc.is_empty(), "Step {i} description should not be empty");
    }
}

/// Test that the report for a current-version save shows zero steps.
#[test]
fn test_migration_report_noop() {
    let mut save = real_save_at_version(CURRENT_SAVE_VERSION);
    let report = migrate_save_with_report(&mut save).unwrap();

    assert_eq!(report.original_version, CURRENT_SAVE_VERSION);
    assert_eq!(report.final_version, CURRENT_SAVE_VERSION);
    assert_eq!(report.steps_applied, 0);
    assert!(report.step_descriptions.is_empty());
}

/// Sample save data test: create a save with some actual data, downgrade
/// its version, migrate it, and verify the data survived intact.
#[test]
fn test_sample_save_with_data_survives_migration() {
    let mut grid = WorldGrid::new(4, 4);
    simulation::terrain::generate_terrain(&mut grid, 42);
    let roads = RoadNetwork::default();
    let clock = GameClock {
        day: 100,
        hour: 14.5,
        speed: 2.0,
        ..Default::default()
    };
    let mut budget = CityBudget::default();
    budget.treasury = 50000.0;
    budget.tax_rate = 0.08;
    let demand = ZoneDemand::default();

    let mut save = create_save_data(
        &grid,
        &roads,
        &clock,
        &budget,
        &demand,
        &[],
        &[],
        &[],
        &[],
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    // Downgrade to v5 to simulate loading an old save
    save.version = 5;

    let old = migrate_save(&mut save).unwrap();
    assert_eq!(old, 5);
    assert_eq!(save.version, CURRENT_SAVE_VERSION);

    // Core data should be intact
    assert_eq!(save.clock.day, 100);
    assert!((save.clock.hour - 14.5).abs() < 0.001);
    assert!((save.budget.treasury - 50000.0).abs() < 0.001);
    assert!((save.budget.tax_rate - 0.08).abs() < 0.001);
}
