//! Plugin that wires up the game-actions subsystem: queue, executor, and log.

use bevy::prelude::*;

use super::executor::execute_queued_actions;
use super::result_log::ActionResultLog;
use super::ActionQueue;
use crate::SimulationSet;

/// Registers the action queue, result log, and executor system.
pub struct GameActionsPlugin;

impl Plugin for GameActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionQueue>();
        app.init_resource::<ActionResultLog>();

        app.add_systems(
            FixedUpdate,
            execute_queued_actions.in_set(SimulationSet::PreSim),
        );
    }
}
