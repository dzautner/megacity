//! SVC-016: Integration tests for Telecom Infrastructure.

use crate::services::ServiceType;
use crate::telecom::{TelecomCoverage, TelecomState, TELECOM_HAPPINESS_BONUS};
use crate::test_harness::TestCity;

fn tick_slow(city: &mut TestCity) {
    city.tick_slow_cycles(1);
}

// ====================================================================
// 1. Resource initialization
// ====================================================================

#[test]
fn test_telecom_resources_initialized() {
    let city = TestCity::new();
    let state = city.resource::<TelecomState>();
    assert_eq!(state.cell_tower_count, 0);
    assert_eq!(state.data_center_count, 0);
    assert_eq!(state.coverage_percentage, 0.0);
    assert_eq!(state.monthly_cost, 0.0);

    let coverage = city.resource::<TelecomCoverage>();
    assert_eq!(coverage.covered_cells(), 0);
    assert_eq!(coverage.boosted_cells(), 0);
}

// ====================================================================
// 2. Cell tower coverage
// ====================================================================

#[test]
fn test_cell_tower_provides_signal_at_center() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::CellTower);
    tick_slow(&mut city);

    let coverage = city.resource::<TelecomCoverage>();
    let signal = coverage.get_signal(50, 50);
    assert!(signal > 0, "Center cell should have signal, got {signal}");
}

#[test]
fn test_cell_tower_signal_falls_off_with_distance() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::CellTower);
    tick_slow(&mut city);

    let coverage = city.resource::<TelecomCoverage>();
    let center = coverage.get_signal(50, 50);
    let nearby = coverage.get_signal(55, 50);
    let farther = coverage.get_signal(60, 50);

    assert!(center > nearby, "Signal should decrease: center={center} > nearby={nearby}");
    assert!(
        nearby >= farther,
        "Signal should decrease further: nearby={nearby} >= farther={farther}"
    );
}

#[test]
fn test_cell_tower_no_signal_far_away() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::CellTower);
    tick_slow(&mut city);

    let coverage = city.resource::<TelecomCoverage>();
    // Cell tower radius is 15 cells; (200, 200) is well outside.
    let signal = coverage.get_signal(200, 200);
    assert_eq!(signal, 0, "Far cell should have no signal");
}

#[test]
fn test_cell_tower_updates_state() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::CellTower);
    tick_slow(&mut city);

    let state = city.resource::<TelecomState>();
    assert_eq!(state.cell_tower_count, 1);
    assert_eq!(state.data_center_count, 0);
    assert!(
        state.coverage_percentage > 0.0,
        "Coverage should be > 0 after placing a tower"
    );
    assert!(state.monthly_cost > 0.0, "Monthly cost should be positive");
}

// ====================================================================
// 3. Data center coverage
// ====================================================================

#[test]
fn test_data_center_provides_signal() {
    let mut city = TestCity::new().with_service(80, 80, ServiceType::DataCenter);
    tick_slow(&mut city);

    let coverage = city.resource::<TelecomCoverage>();
    let signal = coverage.get_signal(80, 80);
    assert!(signal > 0, "Data center should provide mobile signal");
}

#[test]
fn test_data_center_provides_commercial_boost() {
    let mut city = TestCity::new().with_service(80, 80, ServiceType::DataCenter);
    tick_slow(&mut city);

    let coverage = city.resource::<TelecomCoverage>();
    let boost = coverage.get_commercial_boost(80, 80);
    assert!(
        boost > 1.0,
        "Data center center should have commercial boost > 1.0, got {boost}"
    );
}

#[test]
fn test_data_center_boost_falls_off() {
    let mut city = TestCity::new().with_service(80, 80, ServiceType::DataCenter);
    tick_slow(&mut city);

    let coverage = city.resource::<TelecomCoverage>();
    let center_boost = coverage.get_commercial_boost(80, 80);
    let edge_boost = coverage.get_commercial_boost(100, 80);

    assert!(
        center_boost > edge_boost,
        "Boost should decrease with distance: center={center_boost} > edge={edge_boost}"
    );
}

#[test]
fn test_data_center_no_boost_far_away() {
    let mut city = TestCity::new().with_service(80, 80, ServiceType::DataCenter);
    tick_slow(&mut city);

    let coverage = city.resource::<TelecomCoverage>();
    let far_boost = coverage.get_commercial_boost(200, 200);
    assert!(
        (far_boost - 1.0).abs() < f32::EPSILON,
        "Far cell should have no commercial boost"
    );
}

#[test]
fn test_data_center_updates_state() {
    let mut city = TestCity::new().with_service(80, 80, ServiceType::DataCenter);
    tick_slow(&mut city);

    let state = city.resource::<TelecomState>();
    assert_eq!(state.cell_tower_count, 0);
    assert_eq!(state.data_center_count, 1);
    assert!(state.monthly_cost > 0.0);
}

// ====================================================================
// 4. Multiple buildings stack
// ====================================================================

#[test]
fn test_multiple_towers_stack_signal() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::CellTower)
        .with_service(55, 50, ServiceType::CellTower);
    tick_slow(&mut city);

    let coverage = city.resource::<TelecomCoverage>();
    // The overlap region between two towers should have stronger signal than
    // a single tower alone.
    let overlap_signal = coverage.get_signal(52, 50);
    assert!(
        overlap_signal > 100,
        "Overlap should have strong stacked signal, got {overlap_signal}"
    );
}

#[test]
fn test_multiple_data_centers_stack_boost() {
    let mut city = TestCity::new()
        .with_service(80, 80, ServiceType::DataCenter)
        .with_service(85, 80, ServiceType::DataCenter);
    tick_slow(&mut city);

    let coverage = city.resource::<TelecomCoverage>();
    let stacked_boost = coverage.get_commercial_boost(82, 80);
    assert!(
        stacked_boost > 1.25,
        "Stacked data centers should boost above max single, got {stacked_boost}"
    );
}

// ====================================================================
// 5. Happiness bonus
// ====================================================================

#[test]
fn test_telecom_happiness_bonus_zero_without_coverage() {
    let coverage = TelecomCoverage::default();
    let bonus = crate::telecom::telecom_happiness_bonus(&coverage, 50, 50);
    assert_eq!(bonus, 0.0);
}

#[test]
fn test_telecom_happiness_bonus_scales_with_signal() {
    let mut coverage = TelecomCoverage::default();
    let idx = TelecomCoverage::idx(50, 50);
    coverage.signal[idx] = 128;
    let bonus = crate::telecom::telecom_happiness_bonus(&coverage, 50, 50);
    assert!(bonus > 0.0 && bonus < TELECOM_HAPPINESS_BONUS);
}

#[test]
fn test_telecom_happiness_bonus_max_at_full_signal() {
    let mut coverage = TelecomCoverage::default();
    let idx = TelecomCoverage::idx(50, 50);
    coverage.signal[idx] = 255;
    let bonus = crate::telecom::telecom_happiness_bonus(&coverage, 50, 50);
    assert!((bonus - TELECOM_HAPPINESS_BONUS).abs() < 0.01);
}

// ====================================================================
// 6. State counting
// ====================================================================

#[test]
fn test_mixed_infrastructure_counts() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::CellTower)
        .with_service(80, 80, ServiceType::DataCenter)
        .with_service(120, 120, ServiceType::CellTower);
    tick_slow(&mut city);

    let state = city.resource::<TelecomState>();
    assert_eq!(state.cell_tower_count, 2);
    assert_eq!(state.data_center_count, 1);
}
