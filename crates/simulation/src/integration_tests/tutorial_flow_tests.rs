//! Integration tests for the complete tutorial flow (issue #1711).
//!
//! Tests each tutorial step's auto-advance condition, skip behavior,
//! pause/unpause mechanics, and the full Welcome-to-Completed flow.

use bevy::prelude::*;

use crate::grid::{RoadType, ZoneType};
use crate::stats::CityStats;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::tutorial::{TutorialState, TutorialStep};
use crate::utilities::UtilityType;

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
// 1. Full flow: Welcome → Completed
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_full_flow_welcome_to_completed() {
    let mut state = TutorialState {
        active: true,
        completed: false,
        current_step: TutorialStep::Welcome,
        paused_by_tutorial: false,
    };

    // Verify all 9 steps have titles and descriptions
    let expected_steps = [
        TutorialStep::Welcome,
        TutorialStep::PlaceRoad,
        TutorialStep::ZoneResidential,
        TutorialStep::ZoneCommercial,
        TutorialStep::PlacePowerPlant,
        TutorialStep::PlaceWaterTower,
        TutorialStep::ObserveGrowth,
        TutorialStep::ManageBudget,
        TutorialStep::Completed,
    ];

    for (i, &expected) in expected_steps.iter().enumerate() {
        assert_eq!(
            state.current_step, expected,
            "Step {i} mismatch: expected {expected:?}, got {:?}",
            state.current_step
        );
        assert!(
            !state.current_step.title().is_empty(),
            "Step {i} ({expected:?}) has empty title"
        );
        assert!(
            !state.current_step.description().is_empty(),
            "Step {i} ({expected:?}) has empty description"
        );
        assert!(
            !state.current_step.hint().is_empty(),
            "Step {i} ({expected:?}) has empty hint"
        );

        if expected != TutorialStep::Completed {
            assert!(state.advance(), "Failed to advance from step {i}");
        }
    }

    assert!(state.completed);
    assert!(!state.active);
    assert_eq!(state.current_step, TutorialStep::Completed);
}

// ---------------------------------------------------------------------------
// 2. PlaceRoad auto-advances
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_place_road_auto_advances() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 15, RoadType::Local);

    activate_tutorial_at(&mut city, TutorialStep::PlaceRoad);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ZoneResidential,
        "PlaceRoad should auto-advance to ZoneResidential after placing a road"
    );
}

// ---------------------------------------------------------------------------
// 3. ZoneResidential auto-advances
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_zone_residential_auto_advances() {
    let mut city = TestCity::new()
        .with_zone(12, 10, ZoneType::ResidentialLow);

    activate_tutorial_at(&mut city, TutorialStep::ZoneResidential);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ZoneCommercial,
        "ZoneResidential should auto-advance to ZoneCommercial after zoning residential"
    );
}

// ---------------------------------------------------------------------------
// 4. ZoneCommercial auto-advances
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_zone_commercial_auto_advances() {
    let mut city = TestCity::new()
        .with_zone(14, 10, ZoneType::CommercialLow);

    activate_tutorial_at(&mut city, TutorialStep::ZoneCommercial);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::PlacePowerPlant,
        "ZoneCommercial should auto-advance to PlacePowerPlant after zoning commercial"
    );
}

// ---------------------------------------------------------------------------
// 5. PlacePowerPlant auto-advances
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_place_power_auto_advances() {
    let mut city = TestCity::new()
        .with_utility(20, 20, UtilityType::PowerPlant);

    activate_tutorial_at(&mut city, TutorialStep::PlacePowerPlant);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::PlaceWaterTower,
        "PlacePowerPlant should auto-advance to PlaceWaterTower after placing power"
    );
}

// ---------------------------------------------------------------------------
// 6. PlaceWaterTower auto-advances
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_place_water_auto_advances() {
    let mut city = TestCity::new()
        .with_utility(22, 20, UtilityType::WaterTower);

    activate_tutorial_at(&mut city, TutorialStep::PlaceWaterTower);
    run_update(&mut city);

    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ObserveGrowth,
        "PlaceWaterTower should auto-advance to ObserveGrowth after placing water tower"
    );
}

// ---------------------------------------------------------------------------
// 7. ObserveGrowth waits for population
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_observe_growth_waits_for_population() {
    let mut city = TestCity::new();
    activate_tutorial_at(&mut city, TutorialStep::ObserveGrowth);

    // With population = 0, should NOT advance
    run_update(&mut city);
    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ObserveGrowth,
        "ObserveGrowth should not advance with population 0"
    );

    // Set population to 5 via CityStats
    {
        let mut stats = city.world_mut().resource_mut::<CityStats>();
        stats.population = 5;
    }

    run_update(&mut city);
    let tutorial = city.resource::<TutorialState>();
    assert_eq!(
        tutorial.current_step,
        TutorialStep::ManageBudget,
        "ObserveGrowth should advance to ManageBudget when population >= 5"
    );
}

// ---------------------------------------------------------------------------
// 8. Skip works at every step
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_skip_at_every_step() {
    let action_steps = [
        TutorialStep::Welcome,
        TutorialStep::PlaceRoad,
        TutorialStep::ZoneResidential,
        TutorialStep::ZoneCommercial,
        TutorialStep::PlacePowerPlant,
        TutorialStep::PlaceWaterTower,
        TutorialStep::ObserveGrowth,
        TutorialStep::ManageBudget,
    ];

    for &step in &action_steps {
        let mut state = TutorialState {
            active: true,
            completed: false,
            current_step: step,
            paused_by_tutorial: false,
        };

        state.skip();

        assert_eq!(
            state.current_step,
            TutorialStep::Completed,
            "Skip from {step:?} should jump to Completed"
        );
        assert!(
            state.completed,
            "Skip from {step:?} should mark completed"
        );
        assert!(
            !state.active,
            "Skip from {step:?} should deactivate tutorial"
        );
    }
}

// ---------------------------------------------------------------------------
// 9. Simulation pauses during instruction steps
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_pauses_during_instruction_steps() {
    let pause_steps = [
        TutorialStep::Welcome,
        TutorialStep::PlaceRoad,
        TutorialStep::ZoneResidential,
        TutorialStep::ZoneCommercial,
        TutorialStep::PlacePowerPlant,
        TutorialStep::PlaceWaterTower,
    ];

    for &step in &pause_steps {
        let mut city = TestCity::new();
        activate_tutorial_at(&mut city, step);

        // Ensure clock starts unpaused
        {
            let mut clock = city.world_mut().resource_mut::<GameClock>();
            clock.paused = false;
        }

        run_update(&mut city);

        let clock = city.resource::<GameClock>();
        assert!(
            clock.paused,
            "GameClock should be paused during instruction step {step:?}"
        );

        let tutorial = city.resource::<TutorialState>();
        assert!(
            tutorial.paused_by_tutorial,
            "paused_by_tutorial should be set during {step:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// 10. Skip unpauses when paused_by_tutorial was set
// ---------------------------------------------------------------------------

#[test]
fn test_tutorial_unpauses_on_skip() {
    let mut city = TestCity::new();
    activate_tutorial_at(&mut city, TutorialStep::PlaceRoad);

    // Run update to trigger the pause
    run_update(&mut city);

    // Verify paused state
    assert!(
        city.resource::<GameClock>().paused,
        "Clock should be paused after entering PlaceRoad step"
    );
    assert!(
        city.resource::<TutorialState>().paused_by_tutorial,
        "paused_by_tutorial should be true"
    );

    // Now skip the tutorial
    {
        let mut tutorial = city.world_mut().resource_mut::<TutorialState>();
        let was_paused_by_tutorial = tutorial.paused_by_tutorial;
        tutorial.skip();

        // The skip() method clears paused_by_tutorial, but does not directly
        // touch GameClock. In the real game, the UI skip handler unpauses.
        // Here we verify the flag is cleared so the UI knows to unpause.
        assert!(
            !tutorial.paused_by_tutorial,
            "skip() should clear paused_by_tutorial"
        );

        // Simulate what the UI handler would do: unpause if tutorial had paused
        if was_paused_by_tutorial {
            drop(tutorial);
            let mut clock = city.world_mut().resource_mut::<GameClock>();
            clock.paused = false;
        }
    }

    let clock = city.resource::<GameClock>();
    assert!(
        !clock.paused,
        "Clock should be unpaused after skip when tutorial had paused it"
    );

    let tutorial = city.resource::<TutorialState>();
    assert!(tutorial.completed);
    assert!(!tutorial.active);
}
