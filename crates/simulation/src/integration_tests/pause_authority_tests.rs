//! Integration tests for P0-09: Unified pause authority.
//!
//! Verifies that `GameClock.paused` and `AppState` stay in sync regardless
//! of which mechanism triggers the pause/unpause.

use bevy::prelude::*;

use crate::app_state::AppState;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::TickCounter;

/// When `GameClock.paused` is set to `true` (e.g. by toolbar/speed keybind),
/// the sync system should transition `AppState` to `Paused`, which stops
/// simulation systems from running.
#[test]
fn test_game_clock_pause_stops_simulation() {
    let mut city = TestCity::new();

    // Verify simulation is running initially.
    let before = city.resource::<TickCounter>().0;
    city.tick(5);
    let after = city.resource::<TickCounter>().0;
    assert!(after > before, "Simulation should tick while playing");

    // Simulate what the toolbar || button does: only set GameClock.paused.
    city.world_mut().resource_mut::<GameClock>().paused = true;

    // Run Update schedule so the sync system fires and transitions AppState.
    city.world_mut().run_schedule(Update);
    city.world_mut().run_schedule(StateTransition);

    // Verify AppState transitioned to Paused.
    let state = *city.world_mut().resource::<State<AppState>>().get();
    assert_eq!(
        state,
        AppState::Paused,
        "AppState should be Paused after GameClock.paused = true"
    );

    // Verify simulation no longer ticks.
    let before = city.resource::<TickCounter>().0;
    city.tick(20);
    let after = city.resource::<TickCounter>().0;
    assert_eq!(
        before, after,
        "TickCounter should NOT advance when GameClock.paused is true"
    );
}

/// When `GameClock.paused` is set back to `false`, the sync system should
/// transition `AppState` back to `Playing`, resuming simulation.
#[test]
fn test_game_clock_unpause_resumes_simulation() {
    let mut city = TestCity::new();

    // Pause via GameClock.
    city.world_mut().resource_mut::<GameClock>().paused = true;
    city.world_mut().run_schedule(Update);
    city.world_mut().run_schedule(StateTransition);

    // Verify paused.
    let state = *city.world_mut().resource::<State<AppState>>().get();
    assert_eq!(state, AppState::Paused);

    // Unpause via GameClock (simulate pressing a speed button).
    city.world_mut().resource_mut::<GameClock>().paused = false;
    city.world_mut().run_schedule(Update);
    city.world_mut().run_schedule(StateTransition);

    // Verify AppState is Playing again.
    let state = *city.world_mut().resource::<State<AppState>>().get();
    assert_eq!(
        state,
        AppState::Playing,
        "AppState should be Playing after GameClock.paused = false"
    );

    // Verify simulation resumes.
    let before = city.resource::<TickCounter>().0;
    city.tick(10);
    let after = city.resource::<TickCounter>().0;
    assert!(
        after > before,
        "TickCounter should advance after unpausing"
    );
}

/// When `AppState` is already `Paused` and `GameClock.paused` is also `true`,
/// no extra state transition should occur.
#[test]
fn test_already_synced_pause_is_no_op() {
    let mut city = TestCity::new();

    // Pause via both mechanisms (mimicking the ESC pause menu behavior).
    city.world_mut().resource_mut::<GameClock>().paused = true;
    city.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Paused);
    city.world_mut().run_schedule(StateTransition);

    // Run sync — should not panic or produce unexpected state.
    city.world_mut().run_schedule(Update);
    city.world_mut().run_schedule(StateTransition);

    let state = *city.world_mut().resource::<State<AppState>>().get();
    assert_eq!(state, AppState::Paused);
}

/// The sync system should not interfere with `AppState::MainMenu`.
/// Setting `GameClock.paused = true` from MainMenu should NOT transition to Paused.
#[test]
fn test_sync_does_not_affect_main_menu() {
    let mut app = bevy::app::App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.insert_resource(crate::tutorial::TutorialState {
        completed: true,
        active: false,
        ..Default::default()
    });
    // Stay in MainMenu (default state).
    app.add_plugins(crate::SimulationPlugin);
    app.update();

    // Set GameClock.paused from MainMenu.
    app.world_mut().resource_mut::<GameClock>().paused = true;
    app.world_mut().run_schedule(Update);
    app.world_mut().run_schedule(StateTransition);

    // Should still be MainMenu, not Paused.
    let state = *app.world().resource::<State<AppState>>().get();
    assert_eq!(
        state,
        AppState::MainMenu,
        "GameClock.paused should not transition MainMenu to Paused"
    );
}

/// Game clock time should not advance when paused via the GameClock flag,
/// confirming that both the clock tick and system execution are halted.
#[test]
fn test_game_clock_time_does_not_advance_when_paused() {
    let mut city = TestCity::new();

    let hour_before = city.clock().hour;

    // Pause via GameClock (toolbar style).
    city.world_mut().resource_mut::<GameClock>().paused = true;
    city.world_mut().run_schedule(Update);
    city.world_mut().run_schedule(StateTransition);

    // Tick the simulation — systems are gated, clock won't advance.
    city.tick(50);

    let hour_after = city.clock().hour;
    assert_eq!(
        hour_before, hour_after,
        "GameClock hour should not advance when paused"
    );
}
