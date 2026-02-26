//! Integration tests for the `AppState` state machine (PLAY-001).
//!
//! Verifies that the simulation only ticks when `AppState::Playing` is active
//! and that state transitions work correctly.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::app_state::AppState;
use crate::test_harness::TestCity;
use crate::TickCounter;

/// The default `AppState` should be `MainMenu`.
#[test]
fn test_app_state_default_is_main_menu() {
    assert_eq!(AppState::default(), AppState::MainMenu);
}

/// When `AppState` is `Playing` (the test harness default), simulation
/// systems should run and advance the tick counter.
#[test]
fn test_simulation_ticks_in_playing_state() {
    let mut city = TestCity::new();
    let before = city.resource::<TickCounter>().0;
    city.tick(10);
    let after = city.resource::<TickCounter>().0;
    assert!(
        after > before,
        "TickCounter should advance in Playing state: before={before}, after={after}"
    );
}

/// When `AppState` is `MainMenu`, simulation systems should NOT run.
#[test]
fn test_simulation_does_not_tick_in_main_menu() {
    // Build a minimal app in MainMenu state (no insert_state override).
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.insert_resource(crate::world_init::SkipWorldInit);
    app.insert_resource(crate::tutorial::TutorialState {
        completed: true,
        active: false,
        ..Default::default()
    });
    // Do NOT insert AppState::Playing — let it default to MainMenu.
    app.add_plugins(crate::SimulationPlugin);

    // Insert blank world resources.
    let grid = crate::grid::WorldGrid::new(
        crate::config::GRID_WIDTH,
        crate::config::GRID_HEIGHT,
    );
    let (gw_grid, wq_grid) = crate::groundwater::init_groundwater(&grid);
    app.insert_resource(grid);
    app.insert_resource(crate::roads::RoadNetwork::default());
    app.insert_resource(crate::economy::CityBudget::default());
    app.insert_resource(crate::natural_resources::ResourceGrid::default());
    app.insert_resource(gw_grid);
    app.insert_resource(wq_grid);

    // First update runs Startup systems.
    app.update();

    let before = app.world().resource::<TickCounter>().0;

    // Run several FixedUpdate ticks — none should advance the counter.
    for _ in 0..20 {
        app.world_mut().run_schedule(FixedUpdate);
    }

    let after = app.world().resource::<TickCounter>().0;
    assert_eq!(
        before, after,
        "TickCounter should NOT advance in MainMenu state: before={before}, after={after}"
    );
}

/// When `AppState` is `Paused`, simulation systems should NOT run.
#[test]
fn test_simulation_does_not_tick_in_paused_state() {
    let mut city = TestCity::new();

    // Verify simulation ticks first.
    city.tick(5);
    let mid = city.resource::<TickCounter>().0;
    assert!(mid > 0, "Simulation should have ticked in Playing state");

    // Transition to Paused.
    city.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Paused);
    // Run one full app update to apply the state transition.
    city.world_mut().run_schedule(StateTransition);

    let before = city.resource::<TickCounter>().0;
    city.tick(20);
    let after = city.resource::<TickCounter>().0;
    assert_eq!(
        before, after,
        "TickCounter should NOT advance in Paused state: before={before}, after={after}"
    );
}

/// Transitioning from `Paused` back to `Playing` resumes simulation.
#[test]
fn test_resume_from_paused_to_playing() {
    let mut city = TestCity::new();

    // Pause.
    city.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Paused);
    city.world_mut().run_schedule(StateTransition);

    // Confirm paused.
    let before = city.resource::<TickCounter>().0;
    city.tick(10);
    assert_eq!(
        city.resource::<TickCounter>().0,
        before,
        "Should not tick while paused"
    );

    // Resume.
    city.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Playing);
    city.world_mut().run_schedule(StateTransition);

    // Should tick again.
    let before = city.resource::<TickCounter>().0;
    city.tick(10);
    let after = city.resource::<TickCounter>().0;
    assert!(
        after > before,
        "TickCounter should advance after resuming: before={before}, after={after}"
    );
}
