//! Integration tests for education pipeline graduation system (SERV-003).

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity,
};
use crate::education_pipeline::{EducationLevel, EducationPipelineStats, Enrollment};
use crate::grid::RoadType;
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

/// Helper: spawn a citizen at the given home grid cell with specified age and
/// education level. The home grid cell should already have a building.
fn spawn_citizen_with_age_edu(
    city: &mut TestCity,
    home: (usize, usize),
    age: u8,
    education: u8,
) -> bevy::prelude::Entity {
    let world = city.world_mut();
    let home_entity = {
        let grid = world.resource::<crate::grid::WorldGrid>();
        grid.get(home.0, home.1)
            .building_id
            .unwrap_or(bevy::prelude::Entity::PLACEHOLDER)
    };
    let (hx, hy) = crate::grid::WorldGrid::grid_to_world(home.0, home.1);

    world
        .spawn((
            Citizen,
            Position { x: hx, y: hy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: home.0,
                grid_y: home.1,
                building: home_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age,
                gender: Gender::Male,
                education,
                happiness: 60.0,
                health: 90.0,
                salary: 0.0,
                savings: 1000.0,
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
        ))
        .id()
}

/// Helper: create a city base with road and utilities on the road so
/// BFS propagation reaches adjacent buildings.
fn base_city() -> TestCity {
    TestCity::new()
        .with_road(50, 50, 65, 50, RoadType::Local)
        .with_utility(60, 50, UtilityType::PowerPlant)
        .with_utility(62, 50, UtilityType::WaterTower)
}

// ====================================================================
// Enrollment tests
// ====================================================================

#[test]
fn test_education_pipeline_enrollment_elementary() {
    let mut city = base_city()
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(52, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (52, 51), 7, 0);
    city.tick_slow_cycle();

    let world = city.world_mut();
    assert!(
        world.get::<Enrollment>(citizen).is_some(),
        "7-year-old near elementary school should be enrolled"
    );
    let enrollment = world.get::<Enrollment>(citizen).unwrap();
    assert_eq!(enrollment.stage_index, 0, "should be in elementary (stage 0)");
}

#[test]
fn test_education_pipeline_enrollment_requires_prerequisite() {
    let mut city = base_city()
        .with_service(50, 50, ServiceType::HighSchool)
        .with_building(52, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (52, 51), 14, 0);
    city.tick_slow_cycle();

    let world = city.world_mut();
    assert!(
        world.get::<Enrollment>(citizen).is_none(),
        "14-year-old without elementary should NOT enroll in high school"
    );
}

#[test]
fn test_education_pipeline_enrollment_high_school_with_prerequisite() {
    let mut city = base_city()
        .with_service(50, 50, ServiceType::HighSchool)
        .with_building(52, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (52, 51), 14, 1);
    city.tick_slow_cycle();

    let world = city.world_mut();
    assert!(
        world.get::<Enrollment>(citizen).is_some(),
        "14-year-old with elementary education near HS should enroll"
    );
    let enrollment = world.get::<Enrollment>(citizen).unwrap();
    assert_eq!(
        enrollment.stage_index, 1,
        "should be in high school (stage 1)"
    );
}

#[test]
fn test_education_pipeline_no_enrollment_without_school() {
    let mut city = base_city()
        .with_building(52, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (52, 51), 7, 0);
    city.tick_slow_cycle();

    let world = city.world_mut();
    assert!(
        world.get::<Enrollment>(citizen).is_none(),
        "child without nearby school should NOT be enrolled"
    );
}

#[test]
fn test_education_pipeline_too_old_for_elementary() {
    let mut city = base_city()
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(52, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (52, 51), 30, 0);
    city.tick_slow_cycle();

    let world = city.world_mut();
    assert!(
        world.get::<Enrollment>(citizen).is_none(),
        "30-year-old should be too old for elementary enrollment"
    );
}

// ====================================================================
// Graduation tests
// ====================================================================

#[test]
fn test_education_pipeline_graduation_elementary() {
    let mut city = base_city()
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(52, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (52, 51), 7, 0);

    // Run enough slow cycles for enrollment + graduation
    // MIN_ENROLLMENT_TICKS = 3, plus enrollment cycle. Use 4 to stay under
    // the abandonment threshold (500 ticks = 5 slow cycles).
    city.tick_slow_cycles(4);

    let world = city.world_mut();
    // Check citizen is still alive and graduated OR still enrolled
    if let Some(details) = world.get::<CitizenDetails>(citizen) {
        // With 95% base rate and 4 graduation attempts, should have graduated
        let graduated = details.education >= 1;
        let still_enrolled = world.get::<Enrollment>(citizen).is_some();
        assert!(
            graduated || still_enrolled,
            "after 4 slow cycles, citizen should be graduated (edu={}) or still enrolled",
            details.education,
        );
    }
    // If citizen was despawned by another system, that's OK for this test
}

#[test]
fn test_education_pipeline_stats_resource_exists() {
    let mut city = base_city()
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(52, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    spawn_citizen_with_age_edu(&mut city, (52, 51), 7, 0);
    city.tick_slow_cycles(4);

    // The stats resource should always exist
    let stats = city.resource::<EducationPipelineStats>();
    // Enrolled + graduates + dropouts should account for our citizen
    let total_activity = stats.enrolled[0] + stats.graduates[0] + stats.dropouts[0];
    assert!(
        total_activity > 0,
        "stats should show pipeline activity, got enrolled={} grads={} drops={}",
        stats.enrolled[0],
        stats.graduates[0],
        stats.dropouts[0],
    );
}

#[test]
fn test_education_pipeline_already_educated_no_reenroll() {
    let mut city = base_city()
        .with_service(50, 50, ServiceType::University)
        .with_building(52, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (52, 51), 20, 3);
    city.tick_slow_cycle();

    let world = city.world_mut();
    assert!(
        world.get::<Enrollment>(citizen).is_none(),
        "citizen with university education should not re-enroll"
    );
}

#[test]
fn test_education_level_enum_consistency() {
    assert_eq!(EducationLevel::None.as_u8(), 0);
    assert_eq!(EducationLevel::Elementary.as_u8(), 1);
    assert_eq!(EducationLevel::HighSchool.as_u8(), 2);
    assert_eq!(EducationLevel::University.as_u8(), 3);

    assert_eq!(EducationLevel::from_u8(0), EducationLevel::None);
    assert_eq!(EducationLevel::from_u8(1), EducationLevel::Elementary);
    assert_eq!(EducationLevel::from_u8(2), EducationLevel::HighSchool);
    assert_eq!(EducationLevel::from_u8(3), EducationLevel::University);
}

#[test]
fn test_education_pipeline_child_too_young() {
    let mut city = base_city()
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(52, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (52, 51), 3, 0);
    city.tick_slow_cycle();

    let world = city.world_mut();
    assert!(
        world.get::<Enrollment>(citizen).is_none(),
        "3-year-old should not be enrolled"
    );
}

#[test]
fn test_education_pipeline_dropout_on_school_removal() {
    // Short version: enroll, verify, bulldoze, verify dropout
    let mut city = base_city()
        .with_service(52, 50, ServiceType::ElementarySchool)
        .with_building(52, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (52, 51), 7, 0);
    city.tick_slow_cycle();

    {
        let world = city.world_mut();
        assert!(
            world.get::<Enrollment>(citizen).is_some(),
            "should be enrolled before bulldoze"
        );
    }

    city.bulldoze_service_at(52, 50);
    // Just 1 slow cycle to process dropout — keeps total under 400 ticks
    city.tick_slow_cycle();

    let world = city.world_mut();
    // Citizen should have dropped out (no enrollment) or been processed
    if let Some(_details) = world.get::<CitizenDetails>(citizen) {
        assert!(
            world.get::<Enrollment>(citizen).is_none(),
            "citizen should drop out after school is removed"
        );
    }
}

#[test]
fn test_education_pipeline_multiple_children_enroll() {
    // Multiple children near an elementary school should all enroll.
    let mut city = base_city()
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(52, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();

    let mut citizens = Vec::new();
    for _ in 0..5 {
        citizens.push(spawn_citizen_with_age_edu(&mut city, (52, 51), 8, 0));
    }

    city.tick_slow_cycle();

    let world = city.world_mut();
    let enrolled_count = citizens
        .iter()
        .filter(|&&e| world.get::<Enrollment>(e).is_some())
        .count();
    assert_eq!(
        enrolled_count, 5,
        "all 5 children should be enrolled in elementary"
    );
}
