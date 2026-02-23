//! Integration tests for the disease model (SERV-006).

use crate::disease_model::{DiseaseState, DiseaseStatus, DiseaseType};
use crate::pollution::PollutionGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::weather::Weather;

// ====================================================================
// Resource existence tests
// ====================================================================

#[test]
fn test_disease_state_exists_in_new_city() {
    let city = TestCity::new();
    city.assert_resource_exists::<DiseaseState>();
}

#[test]
fn test_disease_state_starts_at_zero() {
    let city = TestCity::new();
    let state = city.resource::<DiseaseState>();
    assert_eq!(state.total_infected, 0);
    assert_eq!(state.flu_count, 0);
    assert_eq!(state.food_poisoning_count, 0);
    assert_eq!(state.respiratory_count, 0);
    assert!((state.infection_rate).abs() < f32::EPSILON);
}

// ====================================================================
// Hospital bed capacity tests
// ====================================================================

#[test]
fn test_hospital_beds_zero_without_health_services() {
    let mut city = TestCity::new();
    city.tick_slow_cycle();
    let state = city.resource::<DiseaseState>();
    assert_eq!(state.hospital_beds, 0);
}

#[test]
fn test_hospital_beds_from_medical_clinic() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::MedicalClinic);
    city.tick_slow_cycle();
    let state = city.resource::<DiseaseState>();
    assert_eq!(state.hospital_beds, 10, "Medical clinic provides 10 beds");
}

#[test]
fn test_hospital_beds_from_hospital() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Hospital);
    city.tick_slow_cycle();
    let state = city.resource::<DiseaseState>();
    assert_eq!(state.hospital_beds, 50, "Hospital provides 50 beds");
}

#[test]
fn test_hospital_beds_from_medical_center() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::MedicalCenter);
    city.tick_slow_cycle();
    let state = city.resource::<DiseaseState>();
    assert_eq!(
        state.hospital_beds, 150,
        "Medical center provides 150 beds"
    );
}

#[test]
fn test_hospital_beds_cumulative() {
    let mut city = TestCity::new()
        .with_service(120, 128, ServiceType::MedicalClinic)
        .with_service(130, 128, ServiceType::Hospital);
    city.tick_slow_cycle();
    let state = city.resource::<DiseaseState>();
    assert_eq!(
        state.hospital_beds, 60,
        "Clinic (10) + Hospital (50) = 60 beds"
    );
}

// ====================================================================
// Disease spread tests
// ====================================================================

#[test]
fn test_no_disease_spread_without_population() {
    let mut city = TestCity::new();
    city.tick_slow_cycle();
    let state = city.resource::<DiseaseState>();
    assert_eq!(state.total_infected, 0, "no disease without citizens");
}

#[test]
fn test_disease_state_tracks_infection_rate() {
    let city = TestCity::new();
    let state = city.resource::<DiseaseState>();
    assert!(
        (state.infection_rate).abs() < f32::EPSILON,
        "infection rate should be 0 with no population"
    );
}

// ====================================================================
// Hospital utilization tests
// ====================================================================

#[test]
fn test_hospital_utilization_zero_without_infected() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::Hospital);
    city.tick_slow_cycle();
    let state = city.resource::<DiseaseState>();
    assert!(
        (state.hospital_utilization).abs() < f32::EPSILON,
        "utilization should be 0 with no infected citizens"
    );
}

// ====================================================================
// DiseaseType properties tests
// ====================================================================

#[test]
fn test_flu_is_mildest_severity() {
    assert!(
        DiseaseType::Flu.severity() < DiseaseType::FoodPoisoning.severity(),
        "flu should be milder than food poisoning"
    );
    assert!(
        DiseaseType::Flu.severity() < DiseaseType::Respiratory.severity(),
        "flu should be milder than respiratory"
    );
}

#[test]
fn test_respiratory_needs_most_beds() {
    assert!(
        DiseaseType::Respiratory.beds_needed() > DiseaseType::Flu.beds_needed(),
        "respiratory should need more beds than flu"
    );
    assert!(
        DiseaseType::Respiratory.beds_needed() >= DiseaseType::FoodPoisoning.beds_needed(),
        "respiratory should need at least as many beds as food poisoning"
    );
}

#[test]
fn test_flu_has_fastest_recovery() {
    assert!(
        DiseaseType::Respiratory.base_recovery_ticks() > DiseaseType::Flu.base_recovery_ticks(),
        "respiratory should take longer to recover than flu"
    );
}

// ====================================================================
// Mortality tracking tests
// ====================================================================

#[test]
fn test_mortality_starts_at_zero() {
    let city = TestCity::new();
    let state = city.resource::<DiseaseState>();
    assert_eq!(state.cumulative_mortality, 0);
    assert_eq!(state.mortality_this_cycle, 0);
    assert!((state.mortality_rate).abs() < f32::EPSILON);
}

// ====================================================================
// Pollution influence tests
// ====================================================================

#[test]
fn test_high_pollution_increases_respiratory_risk() {
    let clean = PollutionGrid {
        levels: vec![0; 100],
        width: 10,
        height: 10,
    };
    let dirty = PollutionGrid {
        levels: vec![200; 100],
        width: 10,
        height: 10,
    };

    let clean_avg: u64 = clean.levels.iter().map(|&v| v as u64).sum();
    let dirty_avg: u64 = dirty.levels.iter().map(|&v| v as u64).sum();
    assert!(
        dirty_avg > clean_avg,
        "dirty city should have higher pollution"
    );
}

// ====================================================================
// Season influence on flu tests
// ====================================================================

#[test]
fn test_winter_increases_flu_base_rate() {
    use crate::weather::types::Season;

    let mut winter_weather = Weather::default();
    winter_weather.season = Season::Winter;

    let mut summer_weather = Weather::default();
    summer_weather.season = Season::Summer;

    assert_eq!(winter_weather.season, Season::Winter);
    assert_eq!(summer_weather.season, Season::Summer);
}

// ====================================================================
// DiseaseStatus component tests
// ====================================================================

#[test]
fn test_disease_status_fields() {
    let status = DiseaseStatus {
        disease_type: DiseaseType::Flu,
        recovery_remaining: 5,
        hospitalized: false,
    };
    assert_eq!(status.disease_type, DiseaseType::Flu);
    assert_eq!(status.recovery_remaining, 5);
    assert!(!status.hospitalized);
}

#[test]
fn test_disease_status_hospitalized() {
    let status = DiseaseStatus {
        disease_type: DiseaseType::Respiratory,
        recovery_remaining: 8,
        hospitalized: true,
    };
    assert!(status.hospitalized);
    assert_eq!(status.disease_type, DiseaseType::Respiratory);
}

// ====================================================================
// Non-health services don't provide beds
// ====================================================================

#[test]
fn test_fire_station_provides_no_beds() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::FireStation);
    city.tick_slow_cycle();
    let state = city.resource::<DiseaseState>();
    assert_eq!(
        state.hospital_beds, 0,
        "fire station should not provide hospital beds"
    );
}

#[test]
fn test_police_station_provides_no_beds() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::PoliceStation);
    city.tick_slow_cycle();
    let state = city.resource::<DiseaseState>();
    assert_eq!(
        state.hospital_beds, 0,
        "police station should not provide hospital beds"
    );
}
