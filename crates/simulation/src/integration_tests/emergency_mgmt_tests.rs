//! SVC-011: Integration tests for the Emergency Management System.
//!
//! Tests that:
//! - EOC (CityHall) presence reduces disaster severity modifiers
//! - Sirens (PoliceStation/FireStation) reduce casualty modifier
//! - Preparedness score reflects infrastructure investments
//! - Without EOC, penalties are applied (+50% response time, +100% casualties)

use crate::emergency_management::{
    EmergencyManagementState, EOC_PREPAREDNESS_BONUS, MAX_SHELTER_PREPAREDNESS,
    MAX_SIREN_PREPAREDNESS, NO_EOC_CASUALTY_MULTIPLIER, NO_EOC_RESPONSE_MULTIPLIER,
    SIREN_CASUALTY_REDUCTION, SIREN_PREPAREDNESS_BONUS,
};
use crate::grid::{RoadType, ZoneType};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ====================================================================
// Test 1: City with no emergency infrastructure has penalties
// ====================================================================

#[test]
fn test_no_eoc_applies_penalty_modifiers() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_building(11, 12, ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();

    let state = city.resource::<EmergencyManagementState>();
    assert!(!state.has_eoc, "no EOC should be present");
    assert!(
        (state.response_time_modifier - NO_EOC_RESPONSE_MULTIPLIER).abs() < f32::EPSILON,
        "response time should be penalized without EOC"
    );
    assert!(
        (state.casualty_modifier - NO_EOC_CASUALTY_MULTIPLIER).abs() < f32::EPSILON,
        "casualty rate should be penalized without EOC"
    );
}

// ====================================================================
// Test 2: CityHall acts as EOC, reducing severity
// ====================================================================

#[test]
fn test_city_hall_acts_as_eoc() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(15, 15, ServiceType::CityHall);

    city.tick_slow_cycle();

    let state = city.resource::<EmergencyManagementState>();
    assert!(state.has_eoc, "CityHall should provide EOC");
    assert!(
        (state.severity_modifier - 0.70).abs() < 0.01,
        "EOC should reduce severity by 30%: got {}",
        state.severity_modifier
    );
    assert!(
        (state.response_time_modifier - 1.0).abs() < f32::EPSILON,
        "response time should be normal with EOC"
    );
}

// ====================================================================
// Test 3: Sirens reduce casualty rate by 20%
// ====================================================================

#[test]
fn test_sirens_reduce_casualty_rate() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(15, 15, ServiceType::CityHall)
        .with_service(20, 20, ServiceType::PoliceStation);

    city.tick_slow_cycle();

    let state = city.resource::<EmergencyManagementState>();
    assert!(state.has_eoc);
    assert!(state.siren_count >= 1, "should have at least one siren");
    // With EOC (casualty=1.0) + siren (-20%) = 0.80
    let expected = 1.0 * (1.0 - SIREN_CASUALTY_REDUCTION);
    assert!(
        (state.casualty_modifier - expected).abs() < 0.01,
        "casualty modifier should be {}: got {}",
        expected,
        state.casualty_modifier
    );
}

// ====================================================================
// Test 4: Preparedness score reflects infrastructure
// ====================================================================

#[test]
fn test_preparedness_score_with_full_infrastructure() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 30, RoadType::Local)
        .with_service(15, 15, ServiceType::CityHall) // EOC
        .with_service(20, 20, ServiceType::PoliceStation) // siren 1
        .with_service(25, 25, ServiceType::FireStation) // siren 2
        .with_building(12, 12, ZoneType::ResidentialLow, 1); // shelter

    city.tick_slow_cycle();

    let state = city.resource::<EmergencyManagementState>();
    assert!(state.has_eoc);
    assert!(state.siren_count >= 2);
    assert!(state.shelter_capacity > 0);

    // Preparedness should include EOC bonus + siren bonus + shelter bonus
    let expected_min = EOC_PREPAREDNESS_BONUS + 2.0 * SIREN_PREPAREDNESS_BONUS;
    assert!(
        state.preparedness_score >= expected_min,
        "preparedness {} should be at least {}",
        state.preparedness_score,
        expected_min
    );
}

// ====================================================================
// Test 5: No sirens means no siren casualty reduction
// ====================================================================

#[test]
fn test_no_sirens_no_casualty_reduction() {
    // EOC but no sirens: casualty modifier should be base (1.0)
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(15, 15, ServiceType::CityHall);

    city.tick_slow_cycle();

    let state = city.resource::<EmergencyManagementState>();
    assert!(state.has_eoc);
    assert_eq!(state.siren_count, 0);
    assert!(
        (state.casualty_modifier - 1.0).abs() < f32::EPSILON,
        "casualty modifier should be 1.0 without sirens: got {}",
        state.casualty_modifier
    );
}

// ====================================================================
// Test 6: Preparedness capped at 100
// ====================================================================

#[test]
fn test_preparedness_score_capped_at_100() {
    // Build lots of infrastructure to try to exceed 100
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 50, RoadType::Local)
        .with_service(15, 15, ServiceType::CityHall);

    // Add many sirens
    for i in 0..10u32 {
        city = city.with_service(20 + i as usize * 3, 20, ServiceType::PoliceStation);
    }
    // Add many residential buildings for shelter capacity
    for i in 0..20u32 {
        city = city.with_building(12, 30 + i as usize, ZoneType::ResidentialLow, 1);
    }

    city.tick_slow_cycle();

    let state = city.resource::<EmergencyManagementState>();
    assert!(
        state.preparedness_score <= 100.0,
        "preparedness should be capped at 100: got {}",
        state.preparedness_score
    );
}

// ====================================================================
// Test 7: Shelter capacity from residential buildings
// ====================================================================

#[test]
fn test_shelter_capacity_from_buildings() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_building(12, 12, ZoneType::ResidentialLow, 1)
        .with_building(14, 14, ZoneType::ResidentialMedium, 2)
        .with_building(16, 16, ZoneType::ResidentialHigh, 3);

    city.tick_slow_cycle();

    let state = city.resource::<EmergencyManagementState>();
    assert_eq!(
        state.shelter_capacity,
        3 * crate::emergency_management::SHELTER_CAPACITY_PER_BUILDING,
        "3 residential buildings should give shelter capacity"
    );
}

// ====================================================================
// Test 8: Siren coverage grid populated near police/fire stations
// ====================================================================

#[test]
fn test_siren_coverage_near_station() {
    let mut city = TestCity::new()
        .with_road(60, 60, 60, 80, RoadType::Local)
        .with_service(65, 65, ServiceType::PoliceStation);

    city.tick_slow_cycle();

    let state = city.resource::<EmergencyManagementState>();
    // The station at (65,65) should cover (65,65) itself
    assert!(
        state.has_siren_coverage(65, 65),
        "siren should cover its own cell"
    );
    // And nearby cells
    assert!(
        state.has_siren_coverage(66, 65),
        "siren should cover adjacent cells"
    );
}
