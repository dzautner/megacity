//! SVC-013: Integration tests for Social Services building types.

use crate::grid::ZoneType;
use crate::services::ServiceType;
use crate::social_services::{
    SocialServicesCoverage, SocialServicesState, COMMUNITY_CENTER_HAPPINESS_BONUS,
    SENIOR_CENTER_HAPPINESS_BONUS, YOUTH_CENTER_HAPPINESS_BONUS,
};
use crate::test_harness::TestCity;

fn tick_slow(city: &mut TestCity) {
    city.tick_slow_cycles(1);
}

// ====================================================================
// 1. Resource initialization
// ====================================================================

#[test]
fn test_social_services_resources_initialized() {
    let city = TestCity::new();
    let state = city.resource::<SocialServicesState>();
    assert_eq!(state.community_center_count, 0);
    assert_eq!(state.substance_treatment_count, 0);
    assert_eq!(state.senior_center_count, 0);
    assert_eq!(state.youth_center_count, 0);
    assert_eq!(state.monthly_maintenance, 0.0);

    let coverage = city.resource::<SocialServicesCoverage>();
    assert_eq!(coverage.community_covered_cells(), 0);
    assert_eq!(coverage.youth_covered_cells(), 0);
}

// ====================================================================
// 2. CommunityCenter coverage
// ====================================================================

#[test]
fn test_community_center_coverage_computed() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::CommunityCenter);
    city.tick(5);
    let coverage = city.resource::<SocialServicesCoverage>();
    assert!(
        coverage.has_community_center(50, 50),
        "CommunityCenter cell should have coverage"
    );
    assert!(
        coverage.community_covered_cells() > 0,
        "CommunityCenter should cover cells"
    );
}

#[test]
fn test_community_center_coverage_radius() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::CommunityCenter);
    city.tick(5);
    let coverage = city.resource::<SocialServicesCoverage>();
    // Radius is 18 cells; cell 10 away should be covered
    assert!(
        coverage.has_community_center(60, 50),
        "Cell 10 away should be within community center radius"
    );
    // Cell 50+ away should not be covered
    assert!(
        !coverage.has_community_center(120, 120),
        "Cell far away should not be within community center radius"
    );
}

// ====================================================================
// 3. SeniorCenter coverage
// ====================================================================

#[test]
fn test_senior_center_coverage_computed() {
    let mut city = TestCity::new().with_service(80, 80, ServiceType::SeniorCenter);
    city.tick(5);
    let coverage = city.resource::<SocialServicesCoverage>();
    assert!(
        coverage.has_senior_center(80, 80),
        "SeniorCenter cell should have coverage"
    );
}

// ====================================================================
// 4. YouthCenter coverage
// ====================================================================

#[test]
fn test_youth_center_coverage_computed() {
    let mut city = TestCity::new().with_service(60, 60, ServiceType::YouthCenter);
    city.tick(5);
    let coverage = city.resource::<SocialServicesCoverage>();
    assert!(
        coverage.has_youth_center(60, 60),
        "YouthCenter cell should have coverage"
    );
    assert!(
        coverage.youth_covered_cells() > 0,
        "YouthCenter should cover cells"
    );
}

// ====================================================================
// 5. SubstanceAbuseTreatmentCenter coverage
// ====================================================================

#[test]
fn test_substance_treatment_coverage_computed() {
    let mut city =
        TestCity::new().with_service(70, 70, ServiceType::SubstanceAbuseTreatmentCenter);
    city.tick(5);
    let coverage = city.resource::<SocialServicesCoverage>();
    assert!(
        coverage.has_substance_treatment(70, 70),
        "SubstanceAbuseTreatmentCenter cell should have coverage"
    );
}

// ====================================================================
// 6. CommunityCenter happiness bonus
// ====================================================================

#[test]
fn test_community_center_happiness_bonus_applied() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::CommunityCenter)
        .with_building(51, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((51, 50), (51, 50));
    city.tick(5);
    tick_slow(&mut city);
    let state = city.resource::<SocialServicesState>();
    assert!(
        state.citizens_covered_community > 0,
        "Citizen at (51,50) should be covered by CommunityCenter at (50,50)"
    );
}

#[test]
fn test_community_center_no_coverage_far_away() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::CommunityCenter)
        .with_building(200, 200, ZoneType::ResidentialLow, 1)
        .with_citizen((200, 200), (200, 200));
    city.tick(5);
    tick_slow(&mut city);
    let coverage = city.resource::<SocialServicesCoverage>();
    assert!(
        !coverage.has_community_center(200, 200),
        "Cell (200,200) should not be covered by CommunityCenter at (50,50)"
    );
}

// ====================================================================
// 7. SeniorCenter happiness bonus for retired citizens
// ====================================================================

#[test]
fn test_senior_center_covers_retired_citizens() {
    // We need to spawn a retired citizen (age >= 65)
    use bevy::prelude::*;
    use crate::citizen::*;
    use crate::mode_choice::ChosenTransportMode;
    use crate::movement::ActivityTimer;

    let mut city = TestCity::new()
        .with_service(80, 80, ServiceType::SeniorCenter)
        .with_building(81, 80, ZoneType::ResidentialLow, 1);

    // Manually spawn a retired citizen (age 70)
    let world = city.world_mut();
    let home_entity = {
        let grid = world.resource::<crate::grid::WorldGrid>();
        grid.get(81, 80).building_id.unwrap_or(Entity::PLACEHOLDER)
    };
    let (hx, hy) = crate::grid::WorldGrid::grid_to_world(81, 80);
    world.spawn((
        Citizen,
        Position { x: hx, y: hy },
        Velocity { x: 0.0, y: 0.0 },
        HomeLocation {
            grid_x: 81,
            grid_y: 80,
            building: home_entity,
        },
        CitizenStateComp(CitizenState::AtHome),
        PathCache::new(Vec::new()),
        CitizenDetails {
            age: 70,
            gender: Gender::Male,
            education: 1,
            happiness: 50.0,
            health: 70.0,
            salary: 0.0,
            savings: 5000.0,
        },
        Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.5,
            resilience: 0.5,
        },
        Needs::default(),
        Family::default(),
        ActivityTimer::default(),
        ChosenTransportMode::default(),
    ));

    city.tick(5);
    tick_slow(&mut city);
    let state = city.resource::<SocialServicesState>();
    assert!(
        state.citizens_covered_senior > 0,
        "Retired citizen at (81,80) should be covered by SeniorCenter at (80,80)"
    );
}

// ====================================================================
// 8. Building count and maintenance tracking
// ====================================================================

#[test]
fn test_maintenance_cost_tracked() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::CommunityCenter)
        .with_service(60, 60, ServiceType::YouthCenter)
        .with_service(70, 70, ServiceType::SeniorCenter);
    city.tick(5);
    tick_slow(&mut city);
    let state = city.resource::<SocialServicesState>();
    assert_eq!(state.community_center_count, 1);
    assert_eq!(state.youth_center_count, 1);
    assert_eq!(state.senior_center_count, 1);
    assert!(
        state.monthly_maintenance > 0.0,
        "Maintenance cost should be positive with social service buildings"
    );
}

#[test]
fn test_no_maintenance_without_buildings() {
    let mut city = TestCity::new();
    city.tick(5);
    tick_slow(&mut city);
    let state = city.resource::<SocialServicesState>();
    assert_eq!(state.monthly_maintenance, 0.0);
}

#[test]
fn test_multiple_buildings_counted() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::CommunityCenter)
        .with_service(60, 60, ServiceType::CommunityCenter)
        .with_service(70, 70, ServiceType::SubstanceAbuseTreatmentCenter);
    city.tick(5);
    tick_slow(&mut city);
    let state = city.resource::<SocialServicesState>();
    assert_eq!(state.community_center_count, 2);
    assert_eq!(state.substance_treatment_count, 1);
}

// ====================================================================
// 9. No coverage without buildings
// ====================================================================

#[test]
fn test_no_coverage_without_buildings() {
    let mut city = TestCity::new();
    city.tick(5);
    let coverage = city.resource::<SocialServicesCoverage>();
    assert_eq!(coverage.community_covered_cells(), 0);
    assert_eq!(coverage.youth_covered_cells(), 0);
}

// ====================================================================
// 10. Saveable roundtrip
// ====================================================================

#[test]
fn test_social_services_state_persists() {
    use crate::Saveable;
    let mut state = SocialServicesState::default();
    state.community_center_count = 5;
    state.substance_treatment_count = 2;
    state.senior_center_count = 3;
    state.youth_center_count = 4;
    state.citizens_covered_community = 300;
    state.citizens_covered_senior = 80;
    state.citizens_covered_youth = 120;
    state.monthly_maintenance = 340.0;
    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = SocialServicesState::load_from_bytes(&bytes);
    assert_eq!(restored.community_center_count, 5);
    assert_eq!(restored.substance_treatment_count, 2);
    assert_eq!(restored.senior_center_count, 3);
    assert_eq!(restored.youth_center_count, 4);
    assert_eq!(restored.citizens_covered_community, 300);
    assert_eq!(restored.citizens_covered_senior, 80);
    assert_eq!(restored.citizens_covered_youth, 120);
    assert!((restored.monthly_maintenance - 340.0).abs() < 0.001);
}

// ====================================================================
// 11. Constants validation
// ====================================================================

#[test]
fn test_happiness_bonuses_positive() {
    assert!(COMMUNITY_CENTER_HAPPINESS_BONUS > 0.0);
    assert!(SENIOR_CENTER_HAPPINESS_BONUS > 0.0);
    assert!(YOUTH_CENTER_HAPPINESS_BONUS > 0.0);
}

#[test]
fn test_community_center_bonus_is_five() {
    assert!(
        (COMMUNITY_CENTER_HAPPINESS_BONUS - 5.0).abs() < f32::EPSILON,
        "CommunityCenter happiness bonus should be +5"
    );
}

#[test]
fn test_senior_center_bonus_is_ten() {
    assert!(
        (SENIOR_CENTER_HAPPINESS_BONUS - 10.0).abs() < f32::EPSILON,
        "SeniorCenter happiness bonus should be +10 for retired citizens"
    );
}
