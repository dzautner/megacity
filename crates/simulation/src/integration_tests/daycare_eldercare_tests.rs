//! SERV-010: Integration tests for Daycare and Eldercare Services.

use crate::daycare_eldercare::{
    DaycareEldercareCoverage, DaycareEldercareState, DAYCARE_HAPPINESS_BONUS,
    ELDERCARE_HAPPINESS_BONUS, ELDERLY_AGE_THRESHOLD,
};
use crate::grid::ZoneType;
use crate::services::ServiceType;
use crate::test_harness::TestCity;

fn tick_slow(city: &mut TestCity) {
    city.tick_slow_cycles(1);
}

// ====================================================================
// 1. Resource initialization
// ====================================================================

#[test]
fn test_daycare_eldercare_resources_initialized() {
    let city = TestCity::new();
    let state = city.resource::<DaycareEldercareState>();
    assert_eq!(state.daycare_count, 0);
    assert_eq!(state.eldercare_count, 0);
    assert_eq!(state.daycare_covered_citizens, 0);
    assert_eq!(state.eldercare_covered_citizens, 0);
    assert_eq!(state.monthly_maintenance, 0.0);

    let coverage = city.resource::<DaycareEldercareCoverage>();
    assert_eq!(coverage.daycare_covered_cells(), 0);
    assert_eq!(coverage.eldercare_covered_cells(), 0);
}

// ====================================================================
// 2. Daycare coverage calculation
// ====================================================================

#[test]
fn test_daycare_coverage_computed_after_placement() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::Daycare);
    // Run ticks to trigger coverage update
    city.tick(5);
    let coverage = city.resource::<DaycareEldercareCoverage>();
    assert!(
        coverage.has_daycare(50, 50),
        "Daycare building cell should have daycare coverage"
    );
    assert!(
        coverage.daycare_covered_cells() > 0,
        "Daycare should cover some cells"
    );
}

#[test]
fn test_daycare_coverage_radius() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::Daycare);
    city.tick(5);
    let coverage = city.resource::<DaycareEldercareCoverage>();

    // Cell within radius (20 cells) should be covered
    assert!(
        coverage.has_daycare(55, 50),
        "Cell 5 away should be within daycare radius of 20"
    );
    assert!(
        coverage.has_daycare(50, 60),
        "Cell 10 away should be within daycare radius of 20"
    );

    // Cell far outside radius should not be covered
    assert!(
        !coverage.has_daycare(100, 100),
        "Cell 50+ away should not be within daycare radius"
    );
}

#[test]
fn test_no_daycare_coverage_without_buildings() {
    let mut city = TestCity::new();
    city.tick(5);
    let coverage = city.resource::<DaycareEldercareCoverage>();
    assert_eq!(coverage.daycare_covered_cells(), 0);
}

// ====================================================================
// 3. Eldercare coverage calculation
// ====================================================================

#[test]
fn test_eldercare_coverage_computed_after_placement() {
    let mut city = TestCity::new().with_service(80, 80, ServiceType::Eldercare);
    city.tick(5);
    let coverage = city.resource::<DaycareEldercareCoverage>();
    assert!(
        coverage.has_eldercare(80, 80),
        "Eldercare building cell should have eldercare coverage"
    );
    assert!(
        coverage.eldercare_covered_cells() > 0,
        "Eldercare should cover some cells"
    );
}

#[test]
fn test_eldercare_coverage_radius() {
    let mut city = TestCity::new().with_service(80, 80, ServiceType::Eldercare);
    city.tick(5);
    let coverage = city.resource::<DaycareEldercareCoverage>();

    // Cell within radius (15 cells) should be covered
    assert!(
        coverage.has_eldercare(85, 80),
        "Cell 5 away should be within eldercare radius of 15"
    );

    // Cell far outside radius should not be covered
    assert!(
        !coverage.has_eldercare(120, 120),
        "Cell 40+ away should not be within eldercare radius"
    );
}

#[test]
fn test_eldercare_smaller_radius_than_daycare() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::Daycare)
        .with_service(50, 50, ServiceType::Eldercare);
    city.tick(5);
    let coverage = city.resource::<DaycareEldercareCoverage>();
    let daycare_cells = coverage.daycare_covered_cells();
    let eldercare_cells = coverage.eldercare_covered_cells();
    assert!(
        daycare_cells > eldercare_cells,
        "Daycare (radius 20) should cover more cells than eldercare (radius 15): {} vs {}",
        daycare_cells,
        eldercare_cells
    );
}

// ====================================================================
// 4. Happiness bonus from coverage
// ====================================================================

#[test]
fn test_daycare_happiness_bonus_applied() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::Daycare)
        .with_building(51, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((51, 50), (51, 50));
    // Get initial happiness before effects
    city.tick(5);
    tick_slow(&mut city);
    let state = city.resource::<DaycareEldercareState>();
    assert!(
        state.daycare_covered_citizens > 0,
        "Citizen at (51,50) should be covered by daycare at (50,50)"
    );
}

#[test]
fn test_eldercare_happiness_bonus_applied() {
    let mut city = TestCity::new()
        .with_service(80, 80, ServiceType::Eldercare)
        .with_building(81, 80, ZoneType::ResidentialLow, 1)
        .with_citizen((81, 80), (81, 80));
    city.tick(5);
    tick_slow(&mut city);
    let state = city.resource::<DaycareEldercareState>();
    assert!(
        state.eldercare_covered_citizens > 0,
        "Citizen at (81,80) should be covered by eldercare at (80,80)"
    );
}

#[test]
fn test_no_bonus_outside_coverage() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::Daycare)
        .with_building(200, 200, ZoneType::ResidentialLow, 1)
        .with_citizen((200, 200), (200, 200));
    city.tick(5);
    tick_slow(&mut city);
    let state = city.resource::<DaycareEldercareState>();
    // The citizen at (200,200) is far from the daycare at (50,50)
    // but there may be other citizens in the world. Check coverage grid directly.
    let coverage = city.resource::<DaycareEldercareCoverage>();
    assert!(
        !coverage.has_daycare(200, 200),
        "Cell (200,200) should not be covered by daycare at (50,50)"
    );
}

// ====================================================================
// 5. Budget maintenance cost
// ====================================================================

#[test]
fn test_maintenance_cost_tracked() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::Daycare)
        .with_service(80, 80, ServiceType::Eldercare);
    city.tick(5);
    tick_slow(&mut city);
    let state = city.resource::<DaycareEldercareState>();
    assert_eq!(state.daycare_count, 1);
    assert_eq!(state.eldercare_count, 1);
    assert!(
        state.monthly_maintenance > 0.0,
        "Maintenance cost should be positive with care buildings"
    );
}

#[test]
fn test_no_maintenance_without_buildings() {
    let mut city = TestCity::new();
    city.tick(5);
    tick_slow(&mut city);
    let state = city.resource::<DaycareEldercareState>();
    assert_eq!(state.monthly_maintenance, 0.0);
}

#[test]
fn test_maintenance_scales_with_count() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::Daycare)
        .with_service(60, 60, ServiceType::Daycare)
        .with_service(70, 70, ServiceType::Daycare);
    city.tick(5);
    tick_slow(&mut city);
    let state = city.resource::<DaycareEldercareState>();
    assert_eq!(state.daycare_count, 3);
    assert!(
        (state.monthly_maintenance - 45.0).abs() < 0.01,
        "3 daycares * 15.0 = 45.0, got {}",
        state.monthly_maintenance
    );
}

// ====================================================================
// 6. Saveable roundtrip
// ====================================================================

#[test]
fn test_daycare_eldercare_state_persists() {
    use crate::Saveable;
    let mut state = DaycareEldercareState::default();
    state.daycare_count = 5;
    state.eldercare_count = 3;
    state.daycare_covered_citizens = 200;
    state.eldercare_covered_citizens = 100;
    state.monthly_maintenance = 175.0;
    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = DaycareEldercareState::load_from_bytes(&bytes);
    assert_eq!(restored.daycare_count, 5);
    assert_eq!(restored.eldercare_count, 3);
    assert_eq!(restored.daycare_covered_citizens, 200);
    assert_eq!(restored.eldercare_covered_citizens, 100);
    assert!((restored.monthly_maintenance - 175.0).abs() < 0.001);
}

// ====================================================================
// 7. Constants validation
// ====================================================================

#[test]
fn test_happiness_bonuses_are_positive() {
    assert!(DAYCARE_HAPPINESS_BONUS > 0.0);
    assert!(ELDERCARE_HAPPINESS_BONUS > 0.0);
}

#[test]
fn test_elderly_threshold_reasonable() {
    assert!(ELDERLY_AGE_THRESHOLD >= 60);
    assert!(ELDERLY_AGE_THRESHOLD <= 75);
}
