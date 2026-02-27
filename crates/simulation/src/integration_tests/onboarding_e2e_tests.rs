//! PLAY-017: Onboarding Golden Path End-to-End Tests (issue #1714).
//!
//! These tests exercise the full new-player journey through the app state
//! machine, terrain generation, tutorial activation, and simulation ticking.
//! They verify that all playability features work together as an integrated
//! whole rather than in isolation.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::app_state::AppState;
use crate::grid::{CellType, WorldGrid};
use crate::terrain_generation::generate_procedural_terrain;
use crate::test_harness::TestCity;
use crate::tutorial::{TutorialState, TutorialStep};
use crate::TickCounter;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal app that starts in MainMenu (the default state) with
/// terrain NOT yet generated, mimicking a fresh launch.
fn build_fresh_launch_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    // Do NOT skip the tutorial -- we want it to activate as a new player
    // would experience. But we start it inactive (the New Game flow
    // activates it explicitly).
    app.insert_resource(TutorialState::default());
    // Do NOT insert AppState::Playing -- let it default to MainMenu.
    // SimulationPlugin registers default resources (WorldGrid, etc.)
    // via init_resource, so no manual insertion is needed.
    app.add_plugins(crate::SimulationPlugin);

    // First update runs Startup systems.
    app.update();
    app
}

// ---------------------------------------------------------------------------
// 1. Full new player golden path
// ---------------------------------------------------------------------------

/// Simulates the full new-player journey:
/// 1. App starts in MainMenu -- simulation is frozen.
/// 2. Terrain is generated (procedural).
/// 3. Transition to Playing state.
/// 4. Tutorial activates and starts at Welcome step.
/// 5. Simulation advances (tick counter increases).
#[test]
fn test_new_player_golden_path() {
    let mut app = build_fresh_launch_app();

    // --- Step 1: Confirm we start in MainMenu, simulation frozen ---
    let t0 = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().resource::<TickCounter>().0,
        t0,
        "Simulation must not tick in MainMenu"
    );

    // --- Step 2: Generate terrain (what the New Game button would trigger) ---
    {
        let mut grid = app.world_mut().resource_mut::<WorldGrid>();
        let _biomes = generate_procedural_terrain(&mut grid, 42, 10_000);
    }

    // Verify terrain has varied elevation (not a flat default grid)
    {
        let grid = app.world().resource::<WorldGrid>();
        let n = grid.cells.len() as f32;
        let mean = grid.cells.iter().map(|c| c.elevation).sum::<f32>() / n;
        let variance = grid
            .cells
            .iter()
            .map(|c| (c.elevation - mean).powi(2))
            .sum::<f32>()
            / n;
        let stddev = variance.sqrt();
        assert!(
            stddev > 0.01,
            "Terrain should have varied elevation after generation (stddev={stddev})"
        );

        // Verify both water and land exist
        let water = grid
            .cells
            .iter()
            .filter(|c| c.cell_type == CellType::Water)
            .count();
        let land = grid.cells.len() - water;
        assert!(water > 0, "Generated terrain should contain water cells");
        assert!(
            land > grid.cells.len() * 3 / 10,
            "Generated terrain should have at least 30% land"
        );
    }

    // --- Step 3: Activate tutorial (what New Game flow does) ---
    {
        let mut tutorial = app.world_mut().resource_mut::<TutorialState>();
        tutorial.active = true;
        tutorial.completed = false;
        tutorial.current_step = TutorialStep::Welcome;
    }

    // Verify tutorial is active at Welcome step
    {
        let tutorial = app.world().resource::<TutorialState>();
        assert!(tutorial.active, "Tutorial should be active for new player");
        assert_eq!(
            tutorial.current_step,
            TutorialStep::Welcome,
            "Tutorial should start at Welcome step"
        );
    }

    // --- Step 4: Transition to Playing ---
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Playing);
    app.world_mut().run_schedule(StateTransition);

    // --- Step 5: Verify simulation advances ---
    let before = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let after = app.world().resource::<TickCounter>().0;
    assert!(
        after > before,
        "Simulation should tick after transitioning to Playing: before={before}, after={after}"
    );
}

// ---------------------------------------------------------------------------
// 2. Pause and resume preserves state
// ---------------------------------------------------------------------------

/// Verifies that pausing freezes the simulation and resuming continues it,
/// with the tick counter preserved across transitions.
#[test]
fn test_pause_resume_preserves_state() {
    let mut city = TestCity::new();

    // --- Phase 1: Run in Playing state, record tick count ---
    city.tick(20);
    let after_playing = city.resource::<TickCounter>().0;
    assert!(
        after_playing > 0,
        "Simulation should have advanced in Playing state"
    );

    // --- Phase 2: Pause -- ticks should NOT advance ---
    city.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Paused);
    city.world_mut().run_schedule(StateTransition);

    let before_paused_ticks = city.resource::<TickCounter>().0;
    city.tick(20);
    let after_paused_ticks = city.resource::<TickCounter>().0;
    assert_eq!(
        before_paused_ticks, after_paused_ticks,
        "TickCounter must NOT advance while Paused"
    );

    // --- Phase 3: Resume -- ticks should advance again ---
    city.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Playing);
    city.world_mut().run_schedule(StateTransition);

    let before_resumed = city.resource::<TickCounter>().0;
    city.tick(20);
    let after_resumed = city.resource::<TickCounter>().0;
    assert!(
        after_resumed > before_resumed,
        "TickCounter should advance after resuming: before={before_resumed}, after={after_resumed}"
    );

    // Verify continuity: the counter after resume should be greater than
    // the counter when we paused (no reset occurred).
    assert!(
        after_resumed > after_playing,
        "Tick counter should be continuous across pause/resume: \
         playing={after_playing}, resumed={after_resumed}"
    );
}

// ---------------------------------------------------------------------------
// 3. Full state cycle: MainMenu -> Playing -> Paused -> Playing -> MainMenu
// ---------------------------------------------------------------------------

/// Exercises the complete lifecycle of app states, verifying that simulation
/// only runs when in Playing state and that transitions in any direction work.
#[test]
fn test_full_state_cycle() {
    let mut app = build_fresh_launch_app();

    // --- MainMenu: frozen ---
    let t0 = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().resource::<TickCounter>().0,
        t0,
        "MainMenu: simulation must be frozen"
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
    assert!(
        t2 > t1,
        "Playing: simulation should tick (t1={t1}, t2={t2})"
    );

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
        "Paused: simulation must be frozen"
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
    assert!(
        t5 > t4,
        "Resumed Playing: simulation should tick (t4={t4}, t5={t5})"
    );

    // --- Back to MainMenu ---
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::MainMenu);
    app.world_mut().run_schedule(StateTransition);

    let t6 = app.world().resource::<TickCounter>().0;
    for _ in 0..10 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    assert_eq!(
        app.world().resource::<TickCounter>().0,
        t6,
        "Back to MainMenu: simulation must be frozen"
    );

    // Verify monotonicity: each Playing phase produced higher tick counts
    assert!(
        t5 > t2,
        "Second Playing phase should have higher ticks than first (t2={t2}, t5={t5})"
    );
}
