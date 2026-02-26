//! Extended integration tests for `AppState` transitions (PLAY-016 / #1713).
//!
//! Complements `app_state_tests.rs` with additional transition scenarios:
//! MainMenu → Playing, Paused → MainMenu, and a full cycle through all states
//! verifying that simulation only runs in Playing.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::app_state::AppState;
use crate::TickCounter;

/// Helper: build a minimal app that starts in MainMenu (the default state).
fn build_main_menu_app() -> App {
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
    app
}

/// Transitioning from MainMenu to Playing should start the simulation.
#[test]
fn test_app_state_transition_main_menu_to_playing() {
    let mut app = build_main_menu_app();

    // Confirm we are in MainMenu and simulation is frozen.
    let before = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().resource::<TickCounter>().0,
        before,
        "TickCounter should not advance while in MainMenu"
    );

    // Transition to Playing.
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Playing);
    app.world_mut().run_schedule(StateTransition);

    // Simulation should now tick.
    let before = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let after = app.world().resource::<TickCounter>().0;
    assert!(
        after > before,
        "TickCounter should advance after transitioning to Playing: before={before}, after={after}"
    );
}

/// Transitioning from Paused back to MainMenu should keep simulation frozen.
#[test]
fn test_app_state_transition_paused_to_main_menu() {
    let mut app = build_main_menu_app();

    // Go to Playing first.
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Playing);
    app.world_mut().run_schedule(StateTransition);

    // Confirm simulation runs.
    let before = app.world().resource::<TickCounter>().0;
    for _ in 0..5 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert!(
        app.world().resource::<TickCounter>().0 > before,
        "Simulation should tick in Playing state"
    );

    // Pause.
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Paused);
    app.world_mut().run_schedule(StateTransition);

    // Confirm paused.
    let paused_tick = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().resource::<TickCounter>().0,
        paused_tick,
        "Simulation should be frozen while Paused"
    );

    // Go back to MainMenu.
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::MainMenu);
    app.world_mut().run_schedule(StateTransition);

    // Simulation should still be frozen in MainMenu.
    let menu_tick = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let after = app.world().resource::<TickCounter>().0;
    assert_eq!(
        menu_tick, after,
        "TickCounter should NOT advance in MainMenu: before={menu_tick}, after={after}"
    );
}

/// Full cycle: MainMenu → Playing → Paused → Playing → Paused → MainMenu.
/// Verifies that the simulation only advances in the Playing state at each step.
#[test]
fn test_app_state_full_lifecycle_simulation_only_in_playing() {
    let mut app = build_main_menu_app();

    // --- MainMenu: simulation frozen ---
    let t0 = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().resource::<TickCounter>().0,
        t0,
        "Step 1: MainMenu should not tick"
    );

    // --- Transition to Playing ---
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Playing);
    app.world_mut().run_schedule(StateTransition);

    let t1 = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let t2 = app.world().resource::<TickCounter>().0;
    assert!(t2 > t1, "Step 2: Playing should tick: t1={t1}, t2={t2}");

    // --- Transition to Paused ---
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Paused);
    app.world_mut().run_schedule(StateTransition);

    let t3 = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().resource::<TickCounter>().0,
        t3,
        "Step 3: Paused should not tick"
    );

    // --- Resume to Playing ---
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Playing);
    app.world_mut().run_schedule(StateTransition);

    let t4 = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let t5 = app.world().resource::<TickCounter>().0;
    assert!(t5 > t4, "Step 4: Resumed Playing should tick: t4={t4}, t5={t5}");

    // --- Pause again ---
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Paused);
    app.world_mut().run_schedule(StateTransition);

    let t6 = app.world().resource::<TickCounter>().0;
    for _ in 0..5 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().resource::<TickCounter>().0,
        t6,
        "Step 5: Paused again should not tick"
    );

    // --- Back to MainMenu ---
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::MainMenu);
    app.world_mut().run_schedule(StateTransition);

    let t7 = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().resource::<TickCounter>().0,
        t7,
        "Step 6: Back to MainMenu should not tick"
    );
}
