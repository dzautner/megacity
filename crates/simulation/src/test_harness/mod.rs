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

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::grid::WorldGrid;
use crate::groundwater;
use crate::natural_resources::ResourceGrid;
use crate::roads::RoadNetwork;
use crate::time_of_day::GameClock;
use crate::tutorial::TutorialState;
use crate::world_init::SkipWorldInit;
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

        // Insert the marker BEFORE SimulationPlugin so init_world skips.
        app.insert_resource(SkipWorldInit);
        // Skip the tutorial so it doesn't pause the GameClock on first update.
        app.insert_resource(TutorialState {
            completed: true,
            active: false,
            ..Default::default()
        });
        app.add_plugins(SimulationPlugin);

        // Insert blank world resources BEFORE the first update, so that
        // systems which depend on Res<WorldGrid> etc. don't panic.
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let (gw_grid, wq_grid) = groundwater::init_groundwater(&grid);
        app.insert_resource(grid);
        app.insert_resource(RoadNetwork::default());
        app.insert_resource(CityBudget::default());
        app.insert_resource(ResourceGrid::default());
        app.insert_resource(gw_grid);
        app.insert_resource(wq_grid);

        // Run one update so Startup systems execute (init_world will no-op).
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
        app.add_plugins(SimulationPlugin);
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
