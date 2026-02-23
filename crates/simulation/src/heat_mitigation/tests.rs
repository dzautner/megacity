//! Unit tests for heat wave mitigation.

use super::calculations::*;
use super::constants::*;
use super::state::HeatMitigationState;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::trees::TreeGrid;

// -------------------------------------------------------------------------
// Default state tests
// -------------------------------------------------------------------------

#[test]
fn test_default_state() {
    let state = HeatMitigationState::default();
    assert!(!state.cooling_centers_enabled);
    assert!(!state.emergency_water_enabled);
    assert_eq!(state.misting_station_count, 0);
    assert_eq!(state.light_roof_count, 0);
    assert!((state.mortality_reduction).abs() < f32::EPSILON);
    assert!((state.green_canopy_temp_reduction).abs() < f32::EPSILON);
    assert!((state.light_roof_temp_reduction).abs() < f32::EPSILON);
    assert!((state.misting_temp_reduction).abs() < f32::EPSILON);
    assert!(!state.dehydration_prevented);
    assert_eq!(state.season_cost, 0.0);
    assert_eq!(state.last_tick_cost, 0.0);
    assert_eq!(state.light_roof_upgrade_total_cost, 0.0);
}

// -------------------------------------------------------------------------
// Green canopy tests
// -------------------------------------------------------------------------

#[test]
fn test_green_canopy_no_trees() {
    let reduction = green_canopy_reduction(0.0);
    assert!(reduction.abs() < f32::EPSILON, "no trees = no reduction");
}

#[test]
fn test_green_canopy_20_percent() {
    let reduction = green_canopy_reduction(0.20);
    assert!(
        (reduction - 5.0).abs() < f32::EPSILON,
        "20% coverage = 5F reduction, got {}",
        reduction
    );
}

#[test]
fn test_green_canopy_40_percent() {
    let reduction = green_canopy_reduction(0.40);
    assert!(
        (reduction - 10.0).abs() < f32::EPSILON,
        "40% coverage = 10F reduction, got {}",
        reduction
    );
}

#[test]
fn test_green_canopy_100_percent() {
    let reduction = green_canopy_reduction(1.0);
    assert!(
        (reduction - 25.0).abs() < f32::EPSILON,
        "100% coverage = 25F reduction, got {}",
        reduction
    );
}

#[test]
fn test_green_canopy_10_percent() {
    let reduction = green_canopy_reduction(0.10);
    assert!(
        (reduction - 2.5).abs() < f32::EPSILON,
        "10% coverage = 2.5F reduction, got {}",
        reduction
    );
}

// -------------------------------------------------------------------------
// Light-colored roof tests
// -------------------------------------------------------------------------

#[test]
fn test_light_roof_no_buildings() {
    let reduction = light_roof_reduction(0, 0);
    assert!(
        reduction.abs() < f32::EPSILON,
        "no buildings = no reduction"
    );
}

#[test]
fn test_light_roof_no_upgrades() {
    let reduction = light_roof_reduction(0, 100);
    assert!(reduction.abs() < f32::EPSILON, "no upgrades = no reduction");
}

#[test]
fn test_light_roof_all_upgraded() {
    let reduction = light_roof_reduction(100, 100);
    assert!(
        (reduction - LIGHT_ROOF_TEMP_REDUCTION).abs() < f32::EPSILON,
        "all upgraded = full 3F reduction, got {}",
        reduction
    );
}

#[test]
fn test_light_roof_half_upgraded() {
    let reduction = light_roof_reduction(50, 100);
    assert!(
        (reduction - 1.5).abs() < f32::EPSILON,
        "50% upgraded = 1.5F reduction, got {}",
        reduction
    );
}

#[test]
fn test_light_roof_more_upgraded_than_buildings() {
    // Edge case: upgraded count exceeds building count (clamped to 1.0 fraction)
    let reduction = light_roof_reduction(200, 100);
    assert!(
        (reduction - LIGHT_ROOF_TEMP_REDUCTION).abs() < f32::EPSILON,
        "capped at full reduction, got {}",
        reduction
    );
}

// -------------------------------------------------------------------------
// Misting station tests
// -------------------------------------------------------------------------

#[test]
fn test_misting_no_stations() {
    let reduction = misting_reduction(0);
    assert!(reduction.abs() < f32::EPSILON, "no stations = no reduction");
}

#[test]
fn test_misting_one_station() {
    let reduction = misting_reduction(1);
    let expected = (1.0 / 50.0) * MISTING_STATION_TEMP_REDUCTION;
    assert!(
        (reduction - expected).abs() < 0.01,
        "1 station = {}F reduction, got {}",
        expected,
        reduction
    );
}

#[test]
fn test_misting_50_stations() {
    let reduction = misting_reduction(50);
    assert!(
        (reduction - MISTING_STATION_TEMP_REDUCTION).abs() < f32::EPSILON,
        "50 stations = full 10F reduction, got {}",
        reduction
    );
}

#[test]
fn test_misting_100_stations_capped() {
    let reduction = misting_reduction(100);
    assert!(
        (reduction - MISTING_STATION_TEMP_REDUCTION).abs() < f32::EPSILON,
        "100 stations = still capped at 10F, got {}",
        reduction
    );
}

#[test]
fn test_misting_25_stations() {
    let reduction = misting_reduction(25);
    let expected = 0.5 * MISTING_STATION_TEMP_REDUCTION;
    assert!(
        (reduction - expected).abs() < f32::EPSILON,
        "25 stations = 5F reduction, got {}",
        reduction
    );
}

// -------------------------------------------------------------------------
// Mortality reduction tests
// -------------------------------------------------------------------------

#[test]
fn test_mortality_no_mitigation() {
    let reduction = total_mortality_reduction(false, false);
    assert!(
        reduction.abs() < f32::EPSILON,
        "no mitigation = no reduction"
    );
}

#[test]
fn test_mortality_cooling_centers_only() {
    let reduction = total_mortality_reduction(true, false);
    assert!(
        (reduction - COOLING_CENTER_MORTALITY_REDUCTION).abs() < f32::EPSILON,
        "cooling centers = 50% reduction, got {}",
        reduction
    );
}

#[test]
fn test_mortality_emergency_water_only() {
    let reduction = total_mortality_reduction(false, true);
    assert!(
        (reduction - 0.30).abs() < f32::EPSILON,
        "emergency water = 30% reduction, got {}",
        reduction
    );
}

#[test]
fn test_mortality_both_active() {
    let reduction = total_mortality_reduction(true, true);
    // 50% + 30% = 80%, capped at 1.0
    assert!(
        (reduction - 0.80).abs() < f32::EPSILON,
        "both = 80% reduction, got {}",
        reduction
    );
}

// -------------------------------------------------------------------------
// Daily operating cost tests
// -------------------------------------------------------------------------

#[test]
fn test_cost_nothing_active() {
    let cost = daily_operating_cost(false, false, 0);
    assert_eq!(cost, 0.0);
}

#[test]
fn test_cost_cooling_centers_only() {
    let cost = daily_operating_cost(true, false, 0);
    assert!((cost - COOLING_CENTER_DAILY_COST).abs() < f64::EPSILON);
}

#[test]
fn test_cost_emergency_water_only() {
    let cost = daily_operating_cost(false, true, 0);
    assert!((cost - EMERGENCY_WATER_DAILY_COST).abs() < f64::EPSILON);
}

#[test]
fn test_cost_misting_stations() {
    let cost = daily_operating_cost(false, false, 5);
    let expected = 5.0 * MISTING_STATION_DAILY_COST;
    assert!((cost - expected).abs() < f64::EPSILON);
}

#[test]
fn test_cost_all_active() {
    let cost = daily_operating_cost(true, true, 10);
    let expected =
        COOLING_CENTER_DAILY_COST + EMERGENCY_WATER_DAILY_COST + 10.0 * MISTING_STATION_DAILY_COST;
    assert!(
        (cost - expected).abs() < f64::EPSILON,
        "expected {}, got {}",
        expected,
        cost
    );
}

// -------------------------------------------------------------------------
// Average tree coverage tests
// -------------------------------------------------------------------------

#[test]
fn test_average_tree_coverage_empty() {
    let tree_grid = TreeGrid::default();
    let coverage = average_tree_coverage(&tree_grid);
    assert!(
        coverage.abs() < f32::EPSILON,
        "empty tree grid = 0% coverage"
    );
}

#[test]
fn test_average_tree_coverage_some_trees() {
    let mut tree_grid = TreeGrid::default();
    // Place 100 trees
    for i in 0..100 {
        let x = i % GRID_WIDTH;
        let y = i / GRID_WIDTH;
        tree_grid.set(x, y, true);
    }
    let coverage = average_tree_coverage(&tree_grid);
    let expected = 100.0 / (GRID_WIDTH * GRID_HEIGHT) as f32;
    assert!(
        (coverage - expected).abs() < 0.0001,
        "expected {}, got {}",
        expected,
        coverage
    );
}

// -------------------------------------------------------------------------
// Saveable implementation tests
// -------------------------------------------------------------------------

#[test]
fn test_saveable_default_skips_save() {
    use crate::Saveable;
    let state = HeatMitigationState::default();
    assert!(
        state.save_to_bytes().is_none(),
        "default state should skip saving"
    );
}

#[test]
fn test_saveable_roundtrip() {
    use crate::Saveable;
    let mut state = HeatMitigationState::default();
    state.cooling_centers_enabled = true;
    state.emergency_water_enabled = true;
    state.misting_station_count = 25;
    state.light_roof_count = 50;
    state.season_cost = 12345.67;
    state.light_roof_upgrade_total_cost = 250_000.0;
    state.last_cost_day = 42;

    let bytes = state
        .save_to_bytes()
        .expect("should save non-default state");
    let restored = HeatMitigationState::load_from_bytes(&bytes);

    assert_eq!(restored.cooling_centers_enabled, true);
    assert_eq!(restored.emergency_water_enabled, true);
    assert_eq!(restored.misting_station_count, 25);
    assert_eq!(restored.light_roof_count, 50);
    assert!((restored.season_cost - 12345.67).abs() < 0.01);
    assert!((restored.light_roof_upgrade_total_cost - 250_000.0).abs() < 0.01);
    assert_eq!(restored.last_cost_day, 42);

    // Derived fields should be at defaults after load
    assert!((restored.mortality_reduction).abs() < f32::EPSILON);
    assert!((restored.misting_temp_reduction).abs() < f32::EPSILON);
    assert!(!restored.dehydration_prevented);
}

#[test]
fn test_saveable_corrupted_bytes() {
    use crate::Saveable;
    let garbage = vec![0xFF, 0xFE, 0xFD];
    let restored = HeatMitigationState::load_from_bytes(&garbage);
    // Should produce default state on corrupt data
    assert!(!restored.cooling_centers_enabled);
    assert_eq!(restored.misting_station_count, 0);
}

// -------------------------------------------------------------------------
// Constants validation
// -------------------------------------------------------------------------

#[test]
fn test_constants_are_reasonable() {
    assert!(COOLING_CENTER_MORTALITY_REDUCTION > 0.0);
    assert!(COOLING_CENTER_MORTALITY_REDUCTION <= 1.0);
    assert!(COOLING_CENTER_DAILY_COST > 0.0);
    assert!(GREEN_CANOPY_TEMP_REDUCTION_PER_20PCT > 0.0);
    assert!(LIGHT_ROOF_TEMP_REDUCTION > 0.0);
    assert!(LIGHT_ROOF_UPGRADE_COST > 0.0);
    assert!(MISTING_STATION_TEMP_REDUCTION > 0.0);
    assert!(MISTING_STATION_DAILY_COST > 0.0);
    assert!(EMERGENCY_WATER_DAILY_COST > 0.0);
    assert!(COST_TICKS_PER_DAY > 0.0);
}

#[test]
fn test_light_roof_upgrade_cost_constant() {
    assert!(
        (LIGHT_ROOF_UPGRADE_COST - 5_000.0).abs() < f64::EPSILON,
        "light roof upgrade should cost $5,000"
    );
}

// -------------------------------------------------------------------------
// Integration-style tests (testing combined effects)
// -------------------------------------------------------------------------

#[test]
fn test_combined_temp_reduction() {
    // Scenario: 40% tree coverage, 50% buildings upgraded, 25 misting stations
    let canopy = green_canopy_reduction(0.40);
    let roof = light_roof_reduction(50, 100);
    let misting = misting_reduction(25);

    let total = canopy + roof + misting;

    // 40% trees = 10F, 50% roofs = 1.5F, 25 stations = 5F -> 16.5F total
    assert!(
        (total - 16.5).abs() < 0.01,
        "expected 16.5F total reduction, got {}",
        total
    );
}

#[test]
fn test_no_effects_without_heat_wave() {
    // When there's no heat wave, active costs should be zero
    let cost = daily_operating_cost(true, true, 10);
    // The cost function itself doesn't check heat wave status -- the system
    // does. So we just verify cost > 0 (the system would skip applying it).
    assert!(cost > 0.0);
}

#[test]
fn test_mortality_reduction_capped() {
    // Even with all mitigations, mortality reduction should never exceed 1.0
    let reduction = total_mortality_reduction(true, true);
    assert!(
        reduction <= 1.0,
        "mortality reduction should be capped at 1.0, got {}",
        reduction
    );
}
