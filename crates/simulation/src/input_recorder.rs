//! Input action recorder for deterministic replay (STAB-03).
//!
//! Records simulation-affecting player actions tagged with their tick number,
//! enabling deterministic replay for bug reproduction and automated testing.
//!
//! The recorder listens to `CityAction` events (the same events used by the
//! undo/redo system) and logs them with the current `TickCounter` value.
//! During replay, it re-fires those events at the correct tick.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::undo_redo::CityAction;
use crate::TickCounter;

// Re-export types from sibling module for external consumers.
pub use crate::input_recorder_types::{
    RecordedAction, RecordedRoadType, RecordedServiceType, RecordedUtilityType, RecordedZoneType,
};

// ---------------------------------------------------------------------------
// RecorderMode
// ---------------------------------------------------------------------------

/// Operating mode for the input recorder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Encode, Decode)]
pub enum RecorderMode {
    /// Recorder is inactive — no recording or replay.
    #[default]
    Off,
    /// Actively recording player actions with their tick numbers.
    Recording,
    /// Playing back a recorded action log, firing events at the right tick.
    Replaying,
}

// ---------------------------------------------------------------------------
// InputRecorder resource
// ---------------------------------------------------------------------------

/// Records and replays player actions for deterministic reproduction.
///
/// In `Recording` mode, the system captures `CityAction` events each tick.
/// In `Replaying` mode, the system fires `CityAction` events at the
/// corresponding tick, enabling deterministic replay of player input.
#[derive(Resource, Default, Encode, Decode)]
pub struct InputRecorder {
    /// Current operating mode.
    pub mode: RecorderMode,
    /// Recorded actions tagged with the tick they occurred on.
    /// Composite actions are flattened: each sub-action gets its own entry
    /// at the same tick number.
    pub actions: Vec<(u64, RecordedAction)>,
    /// Current position in the action log during replay.
    pub replay_cursor: usize,
}

impl InputRecorder {
    /// Start recording, clearing any previous log.
    pub fn start_recording(&mut self) {
        self.mode = RecorderMode::Recording;
        self.actions.clear();
        self.replay_cursor = 0;
    }

    /// Stop recording (or replay).
    pub fn stop(&mut self) {
        self.mode = RecorderMode::Off;
        self.replay_cursor = 0;
    }

    /// Start replaying from the beginning of the log.
    pub fn start_replay(&mut self) {
        self.mode = RecorderMode::Replaying;
        self.replay_cursor = 0;
    }

    /// Record an action at the given tick.
    pub fn record(&mut self, tick: u64, action: RecordedAction) {
        if self.mode == RecorderMode::Recording {
            self.actions.push((tick, action));
        }
    }

    /// Record a `CityAction` at the given tick, flattening composites.
    pub fn record_city_action(&mut self, tick: u64, action: &CityAction) {
        if self.mode != RecorderMode::Recording {
            return;
        }
        for recorded in RecordedAction::from_city_action_list(action) {
            self.actions.push((tick, recorded));
        }
    }

    /// Return the number of recorded actions.
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    /// Check if replay is finished (cursor past end of log).
    pub fn replay_finished(&self) -> bool {
        self.replay_cursor >= self.actions.len()
    }
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl crate::Saveable for InputRecorder {
    const SAVE_KEY: &'static str = "input_recorder";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.actions.is_empty() {
            return None; // skip saving empty recorder
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn::<Self>("input_recorder", bytes)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// System that captures `CityAction` events during recording mode.
///
/// Runs in `PostSim` so that all actions from the current tick have been
/// fired before we capture them.
pub fn capture_actions(
    mut events: EventReader<CityAction>,
    tick: Res<TickCounter>,
    mut recorder: ResMut<InputRecorder>,
) {
    if recorder.mode != RecorderMode::Recording {
        // Still drain events to avoid stale buffering.
        events.clear();
        return;
    }
    for action in events.read() {
        recorder.record_city_action(tick.0, action);
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin that registers the `InputRecorder` resource and capture system.
pub struct InputRecorderPlugin;

impl Plugin for InputRecorderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputRecorder>();

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<InputRecorder>();

        // Capture actions in PostSim so all CityAction events for the tick
        // have been fired before we record them.
        app.add_systems(
            FixedUpdate,
            capture_actions.in_set(crate::SimulationSet::PostSim),
        );
    }
}
