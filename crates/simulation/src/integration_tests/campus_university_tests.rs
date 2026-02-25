//! Integration tests for the Campus/University District System (SERV-011).

use crate::campus_university::{CampusEffects, CampusType, CampusUniversityState};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

#[test]
fn test_campus_university_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<CampusUniversityState>();
    city.assert_resource_exists::<CampusEffects>();
}

#[test]
fn test_campus_university_default_state() {
    let city = TestCity::new();
    let state = city.resource::<CampusUniversityState>();
    assert!(state.districts.is_empty());
    assert_eq!(state.next_id, 1);
    assert_eq!(state.total_students, 0);
    assert_eq!(state.total_academic_works, 0);
    assert_eq!(state.total_graduates, 0);
}

#[test]
fn test_campus_creation_and_level_1_effects() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::University);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<CampusUniversityState>();
        state.create_district(CampusType::University, 50, 50);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<CampusEffects>();
    let edu = effects.education_at(50, 50);
    assert!(
        edu >= 5.0,
        "Level 1 campus should provide >= 5.0 education at center, got {edu}"
    );

    let land_val = effects.land_value_at(50, 50);
    assert!(
        land_val >= 3.0,
        "Level 1 campus should provide >= 3.0 land value at center, got {land_val}"
    );

    let happy = effects.happiness_at(50, 50);
    assert!(
        happy >= 2.0,
        "Level 1 campus should provide >= 2.0 happiness at center, got {happy}"
    );
}

#[test]
fn test_campus_levels_up_with_students_and_works() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::University)
        .with_service(51, 50, ServiceType::Library);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<CampusUniversityState>();
        let id = state.create_district(CampusType::University, 50, 50);
        if let Some(d) = state.get_district_mut(id) {
            d.total_students = 1000;
            d.academic_works = 50;
        }
    }

    city.tick_slow_cycle();

    let state = city.resource::<CampusUniversityState>();
    let district = state.get_district(1).expect("district should exist");
    assert!(
        district.level >= 3,
        "Campus with 1000 students and 50 works should be >= L3, got L{}",
        district.level
    );
}

#[test]
fn test_campus_effect_radius() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::University);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<CampusUniversityState>();
        state.create_district(CampusType::University, 128, 128);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<CampusEffects>();

    let center_edu = effects.education_at(128, 128);
    assert!(center_edu > 0.0, "Center should have education bonus");

    let nearby = effects.education_at(132, 128);
    assert!(nearby > 0.0, "Cell within radius should have education bonus");

    let far = effects.education_at(160, 128);
    assert!(
        far < f32::EPSILON,
        "Cell far outside radius should have no education, got {far}"
    );
}

#[test]
fn test_campus_land_value_bonus() {
    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::University)
        .with_service(101, 100, ServiceType::Library);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<CampusUniversityState>();
        state.create_district(CampusType::University, 100, 100);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<CampusEffects>();
    let lv = effects.land_value_at(100, 100);
    assert!(
        lv >= 3.0,
        "Campus district should provide land value bonus, got {lv}"
    );
}

#[test]
fn test_campus_removal_clears_effects() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::University);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<CampusUniversityState>();
        state.create_district(CampusType::University, 50, 50);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<CampusEffects>();
    assert!(effects.education_at(50, 50) > 0.0);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<CampusUniversityState>();
        state.remove_district(1);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<CampusEffects>();
    assert!(
        effects.education_at(50, 50) < f32::EPSILON,
        "Effects should clear after district removal"
    );
}

#[test]
fn test_campus_saveable_roundtrip() {
    use crate::Saveable;

    let mut state = CampusUniversityState::default();
    state.create_district(CampusType::University, 50, 50);
    state.create_district(CampusType::TradeSchool, 100, 100);
    state.total_students = 2000;
    state.total_academic_works = 75;

    let bytes = state.save_to_bytes().expect("should produce bytes");
    let restored = CampusUniversityState::load_from_bytes(&bytes);

    assert_eq!(restored.districts.len(), 2);
    assert_eq!(restored.total_students, 2000);
    assert_eq!(restored.total_academic_works, 75);
}

#[test]
fn test_multiple_campuses_independent_effects() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::University)
        .with_service(200, 200, ServiceType::University);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<CampusUniversityState>();
        state.create_district(CampusType::University, 30, 30);
        state.create_district(CampusType::TradeSchool, 200, 200);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<CampusEffects>();
    assert!(
        effects.education_at(30, 30) > 0.0,
        "District 1 should affect its center"
    );
    assert!(
        effects.education_at(200, 200) > 0.0,
        "District 2 should affect its center"
    );
}

#[test]
fn test_athletic_buildings_generate_revenue() {
    use crate::grid::{RoadType, ZoneType};

    let mut city = TestCity::new()
        .with_road(48, 48, 48, 58, RoadType::Local)
        .with_zone_rect(49, 48, 50, 58, ZoneType::ResidentialLow)
        .with_building(49, 50, ZoneType::ResidentialLow, 1)
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((49, 50), (50, 50))
        .with_citizen((49, 50), (50, 50))
        .with_service(50, 54, ServiceType::University)
        .with_service(51, 54, ServiceType::Stadium);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<CampusUniversityState>();
        state.create_district(CampusType::University, 50, 54);
    }

    city.tick_slow_cycle();

    let state = city.resource::<CampusUniversityState>();
    let district = state.get_district(1).unwrap();
    assert!(
        district.athletic_building_count > 0,
        "Should detect athletic buildings"
    );
    assert!(
        district.sports_revenue > 0.0,
        "Athletic buildings should generate sports revenue, got {}",
        district.sports_revenue
    );
}
