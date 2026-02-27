//! PLAY-P0-02: Tutorial step threshold sensitivity tests (issue #1745).
//!
//! Verifies that tutorial steps require minimum thresholds before advancing,
//! preventing players from accidentally skipping through the tutorial.

use bevy::prelude::*;

use crate::grid::{RoadType, ZoneType};
use crate::stats::CityStats;
use crate::test_harness::TestCity;
use crate::tutorial::{
    TutorialState, TutorialStep, MIN_COMMERCIAL_ZONES, MIN_RESIDENTIAL_ZONES, MIN_ROAD_CELLS,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Activate the tutorial at a specific step on a fresh TestCity.
fn activate_tutorial_at(city: &mut TestCity, step: TutorialStep) {
    let world = city.world_mut();
    let mut tutorial = world.resource_mut::<TutorialState>();
    tutorial.active = true;
    tutorial.completed = false;
    tutorial.current_step = step;
    tutorial.paused_by_tutorial = false;
}

/// Run one Update tick so `check_tutorial_progress` executes.
fn run_update(city: &mut TestCity) {
    city.world_mut().run_schedule(Update);
}

// ---------------------------------------------------------------------------
// PlaceRoad threshold tests
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_place_road_single_cell_does_not_advance() {
    // A single road cell should NOT be enough to complete the PlaceRoad step.
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 10, RoadType::Local);

    activate_tutorial_at(&mut city, TutorialStep::PlaceRoad);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::PlaceRoad,
        "PlaceRoad should NOT advance with fewer than {MIN_ROAD_CELLS} road cells"
    );
}

#[test]
fn test_tutorial_place_road_below_threshold_does_not_advance() {
    // Place a short road segment (3 cells: 10,10 to 10,12).
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 12, RoadType::Local);

    activate_tutorial_at(&mut city, TutorialStep::PlaceRoad);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::PlaceRoad,
        "PlaceRoad should NOT advance with fewer than {MIN_ROAD_CELLS} road cells"
    );
}

#[test]
fn test_tutorial_place_road_at_threshold_advances() {
    // Place a road segment that produces at least MIN_ROAD_CELLS cells.
    // Road from (10,10) to (10,15) = 6 cells >= 5.
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 15, RoadType::Local);

    activate_tutorial_at(&mut city, TutorialStep::PlaceRoad);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ZoneResidential,
        "PlaceRoad should advance when >= {MIN_ROAD_CELLS} road cells are placed"
    );
}

// ---------------------------------------------------------------------------
// ZoneResidential threshold tests
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_zone_residential_single_cell_does_not_advance() {
    let mut city = TestCity::new()
        .with_zone(12, 10, ZoneType::ResidentialLow);

    activate_tutorial_at(&mut city, TutorialStep::ZoneResidential);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ZoneResidential,
        "ZoneResidential should NOT advance with 1 cell (need {MIN_RESIDENTIAL_ZONES})"
    );
}

#[test]
fn test_tutorial_zone_residential_below_threshold_does_not_advance() {
    // Place 9 residential cells (just below threshold of 10)
    let mut city = TestCity::new()
        .with_zone_rect(12, 10, 14, 12, ZoneType::ResidentialLow); // 3x3 = 9 cells

    activate_tutorial_at(&mut city, TutorialStep::ZoneResidential);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ZoneResidential,
        "ZoneResidential should NOT advance with 9 cells (need {MIN_RESIDENTIAL_ZONES})"
    );
}

#[test]
fn test_tutorial_zone_residential_at_threshold_advances() {
    // Place exactly 10 residential cells (2x5 = 10)
    let mut city = TestCity::new()
        .with_zone_rect(12, 10, 13, 14, ZoneType::ResidentialLow);

    activate_tutorial_at(&mut city, TutorialStep::ZoneResidential);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ZoneCommercial,
        "ZoneResidential should advance when >= {MIN_RESIDENTIAL_ZONES} cells are zoned"
    );
}

// ---------------------------------------------------------------------------
// ZoneCommercial threshold tests
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_zone_commercial_single_cell_does_not_advance() {
    let mut city = TestCity::new()
        .with_zone(14, 10, ZoneType::CommercialLow);

    activate_tutorial_at(&mut city, TutorialStep::ZoneCommercial);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ZoneCommercial,
        "ZoneCommercial should NOT advance with 1 cell (need {MIN_COMMERCIAL_ZONES})"
    );
}

#[test]
fn test_tutorial_zone_commercial_below_threshold_does_not_advance() {
    // Place 4 commercial cells (just below threshold of 5)
    let mut city = TestCity::new()
        .with_zone_rect(14, 10, 17, 10, ZoneType::CommercialLow); // 4x1 = 4 cells

    activate_tutorial_at(&mut city, TutorialStep::ZoneCommercial);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ZoneCommercial,
        "ZoneCommercial should NOT advance with 4 cells (need {MIN_COMMERCIAL_ZONES})"
    );
}

#[test]
fn test_tutorial_zone_commercial_at_threshold_advances() {
    // Place exactly 5 commercial cells (5x1)
    let mut city = TestCity::new()
        .with_zone_rect(14, 10, 18, 10, ZoneType::CommercialLow);

    activate_tutorial_at(&mut city, TutorialStep::ZoneCommercial);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::PlacePowerPlant,
        "ZoneCommercial should advance when >= {MIN_COMMERCIAL_ZONES} cells are zoned"
    );
}

// ---------------------------------------------------------------------------
// ObserveGrowth requires both building and population
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_observe_growth_population_alone_does_not_advance() {
    // Population >= 5 but NO buildings should NOT advance.
    let mut city = TestCity::new();
    activate_tutorial_at(&mut city, TutorialStep::ObserveGrowth);

    {
        let mut stats = city.world_mut().resource_mut::<CityStats>();
        stats.population = 10;
    }

    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ObserveGrowth,
        "ObserveGrowth should NOT advance with population alone (needs a building too)"
    );
}

#[test]
fn test_tutorial_observe_growth_building_alone_does_not_advance() {
    // A building exists but population is 0: should NOT advance.
    let mut city = TestCity::new()
        .with_building(12, 10, ZoneType::ResidentialLow, 1);
    activate_tutorial_at(&mut city, TutorialStep::ObserveGrowth);

    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ObserveGrowth,
        "ObserveGrowth should NOT advance with a building but no population"
    );
}

#[test]
fn test_tutorial_observe_growth_building_and_population_advances() {
    // Both a building AND population >= 5: should advance.
    let mut city = TestCity::new()
        .with_building(12, 10, ZoneType::ResidentialLow, 1);
    activate_tutorial_at(&mut city, TutorialStep::ObserveGrowth);

    {
        let mut stats = city.world_mut().resource_mut::<CityStats>();
        stats.population = 5;
    }

    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ManageBudget,
        "ObserveGrowth should advance when both building exists and population >= 5"
    );
}
