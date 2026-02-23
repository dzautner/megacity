//! Integration tests for the service budget framework (SVC-020).

use crate::budget::ExtendedBudget;
use crate::economy::CityBudget;
use crate::service_budget::{Department, DepartmentEffects, ServiceBudgetState};
use crate::services::ServiceType;
use crate::stats::CityStats;
use crate::test_harness::TestCity;

#[test]
fn test_service_budget_state_initializes_with_defaults() {
    let city = TestCity::new();
    let state = city.resource::<ServiceBudgetState>();
    // All departments should start at zero spending
    for dept in Department::ALL {
        let db = state.department(dept);
        assert!(
            db.actual_spending.abs() < f64::EPSILON,
            "{}: spending should be 0 initially",
            dept.name()
        );
    }
    // Default effects should all be 1.0
    let effects = &state.effects;
    assert!((effects.police_effectiveness - 1.0).abs() < f32::EPSILON);
    assert!((effects.fire_response - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_service_budget_tracks_police_spending() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(10, 10, ServiceType::PoliceStation);

    // Run a slow cycle so the system computes budgets
    city.tick_slow_cycle();

    let state = city.resource::<ServiceBudgetState>();
    let police = state.department(Department::Police);
    assert!(
        police.building_count >= 1,
        "Should count at least 1 police building, got {}",
        police.building_count
    );
}

#[test]
fn test_service_budget_tracks_fire_spending() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(10, 10, ServiceType::FireStation)
        .with_service(12, 12, ServiceType::FireHouse);

    city.tick_slow_cycle();

    let state = city.resource::<ServiceBudgetState>();
    let fire = state.department(Department::FireEms);
    assert!(
        fire.building_count >= 2,
        "Should count at least 2 fire buildings, got {}",
        fire.building_count
    );
}

#[test]
fn test_service_budget_tracks_education_spending() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(10, 10, ServiceType::ElementarySchool)
        .with_service(14, 14, ServiceType::University);

    city.tick_slow_cycle();

    let state = city.resource::<ServiceBudgetState>();
    let edu = state.department(Department::Education);
    assert!(
        edu.building_count >= 2,
        "Should count at least 2 education buildings, got {}",
        edu.building_count
    );
}

#[test]
fn test_service_budget_recommended_proportions_sum_correctly() {
    let sum: f32 = Department::ALL
        .iter()
        .map(|d| d.recommended_proportion())
        .sum();
    assert!(
        (sum - 1.0).abs() < 0.01,
        "Recommended proportions should sum to ~1.0, got {sum}"
    );
}

#[test]
fn test_service_budget_effects_default_to_baseline() {
    let effects = DepartmentEffects::default();
    assert!((effects.police_effectiveness - 1.0).abs() < f32::EPSILON);
    assert!((effects.fire_response - 1.0).abs() < f32::EPSILON);
    assert!((effects.road_quality - 1.0).abs() < f32::EPSILON);
    assert!((effects.education_quality - 1.0).abs() < f32::EPSILON);
    assert!((effects.healthcare_quality - 1.0).abs() < f32::EPSILON);
    assert!((effects.park_quality - 1.0).abs() < f32::EPSILON);
    assert!((effects.sanitation_efficiency - 1.0).abs() < f32::EPSILON);
    assert!((effects.transit_efficiency - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_service_budget_per_capita_with_population() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(10, 10, ServiceType::PoliceStation);

    // Inject some population into stats for per-capita calculation
    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.population = 1000;
    }

    city.tick_slow_cycle();

    let state = city.resource::<ServiceBudgetState>();
    // With population > 0, per_capita should be computed
    let police = state.department(Department::Police);
    // actual_spending should be > 0 since we have a police station
    assert!(
        police.actual_spending > 0.0,
        "Police should have spending from the police station"
    );
    assert!(
        police.per_capita_spending > 0.0,
        "Per-capita spending should be positive with population=1000"
    );
}

#[test]
fn test_service_budget_multiple_departments() {
    let mut city = TestCity::new()
        .with_budget(100_000.0)
        .with_service(10, 10, ServiceType::PoliceStation)
        .with_service(15, 10, ServiceType::FireStation)
        .with_service(20, 10, ServiceType::Hospital)
        .with_service(25, 10, ServiceType::ElementarySchool);

    city.tick_slow_cycle();

    let state = city.resource::<ServiceBudgetState>();
    assert!(state.department(Department::Police).building_count >= 1);
    assert!(state.department(Department::FireEms).building_count >= 1);
    assert!(state.department(Department::Healthcare).building_count >= 1);
    assert!(state.department(Department::Education).building_count >= 1);
}

#[test]
fn test_department_for_service_coverage() {
    // Verify key service types map to the correct department
    assert_eq!(
        Department::for_service(ServiceType::PoliceStation),
        Some(Department::Police)
    );
    assert_eq!(
        Department::for_service(ServiceType::FireStation),
        Some(Department::FireEms)
    );
    assert_eq!(
        Department::for_service(ServiceType::Hospital),
        Some(Department::Healthcare)
    );
    assert_eq!(
        Department::for_service(ServiceType::University),
        Some(Department::Education)
    );
    assert_eq!(
        Department::for_service(ServiceType::SmallPark),
        Some(Department::ParksRecreation)
    );
    assert_eq!(
        Department::for_service(ServiceType::Landfill),
        Some(Department::Sanitation)
    );
    assert_eq!(
        Department::for_service(ServiceType::BusDepot),
        Some(Department::Transport)
    );
    // CityHall should be None (no department)
    assert_eq!(Department::for_service(ServiceType::CityHall), None);
}

#[test]
fn test_service_budget_saveable_roundtrip() {
    use crate::Saveable;

    let mut state = ServiceBudgetState::default();
    state.total_revenue = 50_000.0;
    state.total_spending = 45_000.0;
    state.effects.police_effectiveness = 0.8;

    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = ServiceBudgetState::load_from_bytes(&bytes);

    assert!(
        (restored.total_revenue - 50_000.0).abs() < f64::EPSILON,
        "Revenue should round-trip"
    );
    assert!(
        (restored.effects.police_effectiveness - 0.8).abs() < f32::EPSILON,
        "Effects should round-trip"
    );
}
