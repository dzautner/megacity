//! # TestCity — headless integration test harness for Megacity
//!
//! Provides a fluent builder that wraps `bevy::app::App` + `SimulationPlugin`
//! for running integration tests without a window or renderer.

mod assertions;
mod queries;
mod setup;
mod spawning;

use bevy::app::App;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::app_state::AppState;
use crate::time_of_day::GameClock;
use crate::tutorial::TutorialState;
use crate::SimulationPlugin;

/// A headless Bevy App wrapping `SimulationPlugin` for integration testing.
///
/// Use builder methods to set up city state, then call `tick()` to advance the
/// simulation and query/assert on the resulting ECS state.
pub struct TestCity {
    app: App,
}

impl TestCity {
    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    /// Create a new **empty** city: a 256x256 grass grid with all resources at
    /// their defaults. The Tel Aviv map is NOT loaded.
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);

        // Skip the tutorial so it doesn't pause the GameClock on first update.
        app.insert_resource(TutorialState {
            completed: true,
            active: false,
            ..Default::default()
        });
        // Start in Playing state so simulation systems run during tests.
        app.insert_state(AppState::Playing);
        app.add_plugins(SimulationPlugin);

        // Run one update so Startup systems execute.
        // SimulationPlugin registers default resources (WorldGrid, etc.)
        // via init_resource, so no manual insertion is needed.
        app.update();

        // After the first update, the tutorial starts active by default
        // Ensure tutorial is definitely inactive (belt-and-suspenders)
        // so `check_tutorial_progress` never pauses the GameClock.
        // Force both to clean state for testing.
        if let Some(mut tutorial) = app.world_mut().get_resource_mut::<TutorialState>() {
            tutorial.completed = true;
            tutorial.active = false;
            tutorial.paused_by_tutorial = false;
        }
        if let Some(mut clock) = app.world_mut().get_resource_mut::<GameClock>() {
            clock.paused = false;
        }

        // Insert TestSafetyNet to prevent destructive systems from running
        // during tests. Tests that specifically test destructive behavior
        // should remove this resource.
        app.world_mut().insert_resource(crate::TestSafetyNet);

        Self { app }
    }

    /// Create a city with the full Tel Aviv init_world map.
    /// This spawns ~10K citizens, all roads, buildings, services, and utilities.
    pub fn with_tel_aviv() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        // Skip the tutorial so it doesn't pause the GameClock on first update.
        app.insert_resource(TutorialState {
            completed: true,
            active: false,
            ..Default::default()
        });
        // Start in Playing state so simulation systems run during tests.
        app.insert_state(AppState::Playing);
        app.add_plugins(SimulationPlugin);
        // Tel Aviv init is no longer a default Startup system — add it
        // explicitly so this test harness still gets the prebuilt city.
        app.add_systems(Startup, crate::world_init::init_world);
        // Run one update so Startup systems execute (init_world runs fully).
        app.update();

        // Ensure tutorial doesn't interfere with test simulation.
        if let Some(mut tutorial) = app.world_mut().get_resource_mut::<TutorialState>() {
            tutorial.completed = true;
            tutorial.active = false;
            tutorial.paused_by_tutorial = false;
        }
        if let Some(mut clock) = app.world_mut().get_resource_mut::<GameClock>() {
            clock.paused = false;
        }

        // Insert TestSafetyNet to prevent destructive systems from running
        // during tests. Tests that specifically test destructive behavior
        // should remove this resource.
        app.world_mut().insert_resource(crate::TestSafetyNet);

        Self { app }
    }
}
