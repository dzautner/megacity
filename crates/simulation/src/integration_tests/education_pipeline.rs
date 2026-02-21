use crate::grid::{RoadType, ZoneType};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ====================================================================
// Education Pipeline Tests (TEST-058)
// ====================================================================

#[test]
fn test_education_elementary_school_propagates_level_1() {
    // An elementary school should propagate education level 1 to nearby
    // cells reachable via roads.
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // The school cell itself should have education level 1
    assert_eq!(
        edu.get(50, 50),
        1,
        "elementary school cell should have education level 1"
    );
}

#[test]
fn test_education_high_school_propagates_level_2() {
    // A high school should propagate education level 2 to cells it reaches.
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::HighSchool);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    assert_eq!(
        edu.get(50, 50),
        2,
        "high school cell should have education level 2"
    );
}

#[test]
fn test_education_university_propagates_level_3() {
    // A university should propagate education level 3 to cells it reaches.
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::University);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    assert_eq!(
        edu.get(50, 50),
        3,
        "university cell should have education level 3"
    );
}

#[test]
fn test_education_spreads_along_road() {
    // Education should spread along roads from the school.
    // Place a school at (50,50) with a road going east to (60,50).
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // Cells along the road should get education level 1
    let level_at_55 = edu.get(55, 50);
    assert_eq!(
        level_at_55, 1,
        "road cell at (55,50) should have education level 1, got {level_at_55}"
    );
}

#[test]
fn test_education_does_not_reach_disconnected_area() {
    // Education should NOT reach cells that are not road-connected to the school.
    // Place school at (50,50) with road going east, but check a far-off cell.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // A disconnected cell far away should have no education
    assert_eq!(
        edu.get(200, 200),
        0,
        "disconnected cell at (200,200) should have education level 0"
    );
}

#[test]
fn test_education_university_overrides_lower_level() {
    // When a university (level 3) and elementary school (level 1) both
    // cover a cell, the higher level should win because sources are sorted
    // highest-first.
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool)
        .with_service(55, 50, ServiceType::University);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // The university cell should have level 3 (not overridden by elementary)
    assert_eq!(
        edu.get(55, 50),
        3,
        "university cell should have education level 3 even with nearby elementary school"
    );
    // The school cell at (50, 50) should also have level 3 because the
    // university BFS runs first and reaches it via the road
    let level_at_school = edu.get(50, 50);
    assert!(
        level_at_school >= 1,
        "elementary school cell should have at least education level 1, got {level_at_school}"
    );
}

#[test]
fn test_education_grid_resets_each_slow_tick() {
    // Education grid is recalculated from scratch each slow tick.
    // After removing the school (despawning it), the grid should reset to 0.
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool);
    city.tick_slow_cycle();
    let edu_before = city
        .resource::<crate::education::EducationGrid>()
        .get(50, 50);
    assert_eq!(edu_before, 1, "should have education before removal");

    // Bulldoze the school service building
    city.bulldoze_service_at(50, 50);
    city.tick_slow_cycle();
    let edu_after = city
        .resource::<crate::education::EducationGrid>()
        .get(50, 50);
    assert_eq!(
        edu_after, 0,
        "education should be 0 after school is removed"
    );
}

#[test]
fn test_education_kindergarten_propagates_level_1() {
    // Kindergarten should also propagate education level 1.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::Kindergarten);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    assert_eq!(
        edu.get(50, 50),
        1,
        "kindergarten cell should have education level 1"
    );
}

#[test]
fn test_education_library_propagates_level_1() {
    // Library should propagate education level 1.
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::Library);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    assert_eq!(
        edu.get(50, 50),
        1,
        "library cell should have education level 1"
    );
}

#[test]
fn test_education_reaches_adjacent_grass_but_not_beyond() {
    // BFS spreads through roads and marks adjacent grass cells, but does
    // not continue propagating through grass (only through roads).
    // Layout: school at (50,50), road (50,50)-(55,50), grass cell at (55,51)
    let mut city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::HighSchool);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // The road cell at (55,50) should have level 2
    assert_eq!(
        edu.get(55, 50),
        2,
        "road cell should have education level 2"
    );
    // Adjacent grass at (55,51) should get level 2 (BFS marks grass neighbors)
    let grass_level = edu.get(55, 51);
    assert_eq!(
        grass_level, 2,
        "adjacent grass cell at (55,51) should get education level 2, got {grass_level}"
    );
}

#[test]
fn test_education_no_schools_means_zero_everywhere() {
    // With no education buildings, the entire grid should remain at 0.
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_building(51, 51, ZoneType::ResidentialLow, 1);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // Spot-check a few cells
    assert_eq!(edu.get(50, 50), 0, "no schools => level 0 at (50,50)");
    assert_eq!(edu.get(55, 50), 0, "no schools => level 0 at (55,50)");
    assert_eq!(edu.get(51, 51), 0, "no schools => level 0 at (51,51)");
}

#[test]
fn test_education_multiple_schools_different_levels() {
    // Place elementary (level 1) and university (level 3) on separate roads.
    // Each school's area should reflect its own level.
    let mut city = TestCity::new()
        // Elementary school area: road from (30,30) to (35,30)
        .with_road(30, 30, 35, 30, RoadType::Local)
        .with_service(30, 30, ServiceType::ElementarySchool)
        // University area: road from (80,80) to (85,80), far from elementary
        .with_road(80, 80, 85, 80, RoadType::Local)
        .with_service(80, 80, ServiceType::University);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    assert_eq!(edu.get(30, 30), 1, "elementary area should have level 1");
    assert_eq!(edu.get(80, 80), 3, "university area should have level 3");
    // Each area should not be contaminated by the other
    assert_eq!(edu.get(35, 30), 1, "near elementary should be level 1");
    assert_eq!(edu.get(85, 80), 3, "near university should be level 3");
}

#[test]
fn test_education_range_limited_by_radius() {
    // Education spread is limited by the school's radius converted to grid cells.
    // ElementarySchool has coverage_radius = 15.0 * CELL_SIZE = 15 * 16 = 240 pixels.
    // Range in grid cells = 240 / 16 = 15 cells.
    // Place a long road but check that cells beyond range have level 0.
    let mut city = TestCity::new()
        .with_road(50, 50, 100, 50, RoadType::Local)
        .with_service(50, 50, ServiceType::ElementarySchool);
    city.tick_slow_cycle();
    let edu = city.resource::<crate::education::EducationGrid>();
    // Close to school (within 15 cells) should be level 1
    assert_eq!(
        edu.get(55, 50),
        1,
        "within range should have education level 1"
    );
    // Far beyond range (50 cells away) should be 0
    assert_eq!(
        edu.get(100, 50),
        0,
        "beyond range should have education level 0"
    );
}
