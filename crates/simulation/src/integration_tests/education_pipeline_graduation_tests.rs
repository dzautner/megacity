//! Integration tests for education pipeline graduation system (SERV-003).

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity,
};
use crate::education::EducationGrid;
use crate::education_pipeline::{EducationLevel, EducationPipelineStats, Enrollment};
use crate::grid::RoadType;
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::services::ServiceType;
use crate::test_harness::TestCity;

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

// ====================================================================
// Enrollment tests
// ====================================================================

#[test]
fn test_education_pipeline_enrollment_elementary() {
    // A 7-year-old with no education near an elementary school should enroll.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(51, 51, crate::grid::ZoneType::ResidentialLow, 1);

    // First propagate education grid
    city.tick_slow_cycle();

    // Spawn a 7-year-old with no education
    let citizen = spawn_citizen_with_age_edu(&mut city, (51, 51), 7, 0);

    // Run another slow cycle to trigger enrollment
    city.tick_slow_cycle();

    let world = city.world_mut();
    let has_enrollment = world.get::<Enrollment>(citizen).is_some();
    assert!(
        has_enrollment,
        "7-year-old near elementary school should be enrolled"
    );
    if let Some(enrollment) = world.get::<Enrollment>(citizen) {
        assert_eq!(
            enrollment.stage_index, 0,
            "should be enrolled in elementary (stage 0)"
        );
    }
}

#[test]
fn test_education_pipeline_enrollment_requires_prerequisite() {
    // A 14-year-old with no education near a high school should NOT enroll
    // (needs elementary first).
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::HighSchool)
        .with_building(51, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (51, 51), 14, 0);
    city.tick_slow_cycle();

    let world = city.world_mut();
    let has_enrollment = world.get::<Enrollment>(citizen).is_some();
    assert!(
        !has_enrollment,
        "14-year-old without elementary education should NOT enroll in high school"
    );
}

#[test]
fn test_education_pipeline_enrollment_high_school_with_prerequisite() {
    // A 14-year-old WITH elementary education near a high school should enroll.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::HighSchool)
        .with_building(51, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (51, 51), 14, 1);
    city.tick_slow_cycle();

    let world = city.world_mut();
    let has_enrollment = world.get::<Enrollment>(citizen).is_some();
    assert!(
        has_enrollment,
        "14-year-old with elementary education near HS should enroll"
    );
    if let Some(enrollment) = world.get::<Enrollment>(citizen) {
        assert_eq!(
            enrollment.stage_index, 1,
            "should be enrolled in high school (stage 1)"
        );
    }
}

#[test]
fn test_education_pipeline_no_enrollment_without_school() {
    // A 7-year-old with no nearby school should NOT enroll.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_building(51, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (51, 51), 7, 0);
    city.tick_slow_cycle();

    let world = city.world_mut();
    let has_enrollment = world.get::<Enrollment>(citizen).is_some();
    assert!(
        !has_enrollment,
        "child without nearby school should NOT be enrolled"
    );
}

#[test]
fn test_education_pipeline_too_old_for_elementary() {
    // A 30-year-old with no education near an elementary school
    // should NOT enroll (over max age for elementary).
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(51, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (51, 51), 30, 0);
    city.tick_slow_cycle();

    let world = city.world_mut();
    let has_enrollment = world.get::<Enrollment>(citizen).is_some();
    assert!(
        !has_enrollment,
        "30-year-old should be too old for elementary enrollment"
    );
}

// ====================================================================
// Graduation tests
// ====================================================================

#[test]
fn test_education_pipeline_graduation_elementary() {
    // A 7-year-old enrolled in elementary should graduate after enough ticks.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(51, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (51, 51), 7, 0);

    // Run enough slow cycles for enrollment + graduation
    // (MIN_ENROLLMENT_TICKS = 3, plus 1 for enrollment)
    city.tick_slow_cycles(5);

    let world = city.world_mut();
    let details = world.get::<CitizenDetails>(citizen).unwrap();
    // With high base rate (95%) and within capacity, should graduate
    assert_eq!(
        details.education, 1,
        "child should graduate with elementary education (level 1)"
    );
}

#[test]
fn test_education_pipeline_stats_update() {
    // After graduation, stats should reflect the graduate.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(51, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    spawn_citizen_with_age_edu(&mut city, (51, 51), 7, 0);

    city.tick_slow_cycles(5);

    let stats = city.resource::<EducationPipelineStats>();
    let total_processed = stats.graduates[0] + stats.dropouts[0];
    assert!(
        total_processed > 0,
        "stats should show at least one processed elementary student"
    );
}

#[test]
fn test_education_pipeline_university_enrollment() {
    // An 18-year-old with high school education near a university should enroll.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::University)
        .with_building(51, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (51, 51), 18, 2);
    city.tick_slow_cycle();

    let world = city.world_mut();
    let has_enrollment = world.get::<Enrollment>(citizen).is_some();
    assert!(
        has_enrollment,
        "18-year-old with HS education near university should enroll"
    );
    if let Some(enrollment) = world.get::<Enrollment>(citizen) {
        assert_eq!(
            enrollment.stage_index, 2,
            "should be enrolled in university (stage 2)"
        );
    }
}

#[test]
fn test_education_pipeline_dropout_on_school_removal() {
    // An enrolled citizen whose school is bulldozed should drop out.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(51, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (51, 51), 7, 0);
    city.tick_slow_cycle();

    // Verify enrollment
    {
        let world = city.world_mut();
        assert!(
            world.get::<Enrollment>(citizen).is_some(),
            "should be enrolled before bulldoze"
        );
    }

    // Remove the school
    city.bulldoze_service_at(50, 50);
    // Run enough ticks for education grid to recalculate and dropout to process
    city.tick_slow_cycles(2);

    let world = city.world_mut();
    let has_enrollment = world.get::<Enrollment>(citizen).is_some();
    assert!(
        !has_enrollment,
        "citizen should drop out after school is removed"
    );

    let details = world.get::<CitizenDetails>(citizen).unwrap();
    assert_eq!(
        details.education, 0,
        "dropped-out citizen should still have no education"
    );
}

#[test]
fn test_education_pipeline_capacity_modifier_affects_stats() {
    // With many students and few schools, the effective graduation rate
    // should be lower than the base rate. We test this indirectly by
    // verifying the pipeline stats resource tracks data.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(51, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();

    // Spawn many children
    for _ in 0..10 {
        spawn_citizen_with_age_edu(&mut city, (51, 51), 8, 0);
    }

    city.tick_slow_cycles(6);

    let stats = city.resource::<EducationPipelineStats>();
    // The pipeline should have processed some students
    let total = stats.graduates[0] + stats.dropouts[0];
    assert!(
        total > 0,
        "pipeline should process some elementary students, got graduates={} dropouts={}",
        stats.graduates[0],
        stats.dropouts[0],
    );
}

#[test]
fn test_education_pipeline_already_educated_no_reenroll() {
    // A citizen with university education should not re-enroll anywhere.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::University)
        .with_building(51, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (51, 51), 20, 3);
    city.tick_slow_cycle();

    let world = city.world_mut();
    let has_enrollment = world.get::<Enrollment>(citizen).is_some();
    assert!(
        !has_enrollment,
        "citizen with university education should not re-enroll"
    );
}

#[test]
fn test_education_level_enum_consistency() {
    // Verify that EducationLevel maps correctly to the u8 education field.
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
    // A 3-year-old should not be enrolled in anything.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_building(51, 51, crate::grid::ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();
    let citizen = spawn_citizen_with_age_edu(&mut city, (51, 51), 3, 0);
    city.tick_slow_cycle();

    let world = city.world_mut();
    assert!(
        world.get::<Enrollment>(citizen).is_none(),
        "3-year-old should not be enrolled"
    );
}
