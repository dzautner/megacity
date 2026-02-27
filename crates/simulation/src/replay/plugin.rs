//! Bevy plugin that registers replay resources and systems.

use bevy::prelude::*;

use super::player::{ReplayPlayer, feed_replay_actions};
use super::recorder::{ReplayRecorder, record_actions};
use crate::SimulationSet;

/// Plugin that provides deterministic replay recording and playback.
///
/// - `ReplayRecorder` snapshots pending actions before the executor drains them.
/// - `ReplayPlayer` injects recorded actions at the correct tick during playback.
///
/// Both systems run in `PreSim` with explicit ordering:
/// `feed_replay_actions` → `record_actions` (so replayed actions are visible
/// to the recorder if nested recording is desired in the future).
pub struct ReplayPlugin;

impl Plugin for ReplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ReplayRecorder>();
        app.init_resource::<ReplayPlayer>();

        app.add_systems(
            FixedUpdate,
            (
                feed_replay_actions,
                record_actions.after(feed_replay_actions),
            )
                .in_set(SimulationSet::PreSim),
        );
    }
}
