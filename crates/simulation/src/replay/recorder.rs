//! Replay recorder: captures `GameAction`s from the `ActionQueue` each tick.
//!
//! The recorder system runs in `PreSim` and snapshots all pending actions
//! before the executor drains them. This ensures every action that the
//! simulation processes is faithfully captured for later replay.

use bevy::prelude::*;

use crate::game_actions::{ActionQueue, ActionSource, GameAction};
use crate::TickCounter;

use super::format::{ReplayEntry, ReplayFile, ReplayFooter, ReplayHeader, CURRENT_FORMAT_VERSION};

/// Resource that accumulates `ReplayEntry` items while recording is active.
#[derive(Resource, Default)]
pub struct ReplayRecorder {
    header: Option<ReplayHeader>,
    entries: Vec<ReplayEntry>,
    recording: bool,
}

impl ReplayRecorder {
    /// Begin recording a new replay session.
    ///
    /// Clears any previous entries and stores the header metadata.
    pub fn start(&mut self, seed: u64, city_name: String, start_tick: u64) {
        self.recording = true;
        self.entries.clear();
        self.header = Some(ReplayHeader {
            format_version: CURRENT_FORMAT_VERSION,
            seed,
            city_name,
            start_tick,
        });
    }

    /// Append a single action entry. Only records if currently recording.
    pub fn record(&mut self, tick: u64, action: GameAction) {
        if self.recording {
            self.entries.push(ReplayEntry { tick, action });
        }
    }

    /// Stop recording and produce a finalized `ReplayFile`.
    ///
    /// `state_hash` is the hash of the simulation state at `end_tick`
    /// (pass 0 if hashing is unavailable).
    pub fn stop(&mut self, end_tick: u64, state_hash: u64) -> ReplayFile {
        self.recording = false;
        let header = self.header.take().unwrap_or(ReplayHeader {
            format_version: CURRENT_FORMAT_VERSION,
            seed: 0,
            city_name: String::new(),
            start_tick: 0,
        });
        let entries = std::mem::take(&mut self.entries);
        let entry_count = entries.len() as u64;
        ReplayFile {
            header,
            entries,
            footer: ReplayFooter {
                end_tick,
                final_state_hash: state_hash,
                entry_count,
            },
        }
    }

    /// Whether the recorder is currently capturing actions.
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Number of entries recorded so far.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// System that snapshots pending actions from the `ActionQueue` into the
/// `ReplayRecorder` before the executor drains them.
///
/// Only player and agent actions are recorded — actions sourced from `Replay`
/// are skipped to prevent feedback loops during nested recording.
pub fn record_actions(
    queue: Res<ActionQueue>,
    tick: Res<TickCounter>,
    mut recorder: ResMut<ReplayRecorder>,
) {
    if !recorder.is_recording() {
        return;
    }
    for queued in queue.iter() {
        // Skip replay-sourced actions to avoid recording our own playback.
        if queued.source == ActionSource::Replay {
            continue;
        }
        recorder.record(tick.0, queued.action.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::RoadType;

    #[test]
    fn start_record_stop_produces_valid_file() {
        let mut recorder = ReplayRecorder::default();
        recorder.start(42, "TestCity".to_string(), 0);

        assert!(recorder.is_recording());
        assert_eq!(recorder.entry_count(), 0);

        recorder.record(1, GameAction::SetSpeed { speed: 2 });
        recorder.record(
            5,
            GameAction::PlaceRoadLine {
                start: (10, 10),
                end: (20, 10),
                road_type: RoadType::Local,
            },
        );

        assert_eq!(recorder.entry_count(), 2);

        let replay = recorder.stop(100, 0);

        assert!(!recorder.is_recording());
        assert_eq!(recorder.entry_count(), 0);

        assert_eq!(replay.header.seed, 42);
        assert_eq!(replay.header.city_name, "TestCity");
        assert_eq!(replay.entries.len(), 2);
        assert_eq!(replay.footer.entry_count, 2);
        assert_eq!(replay.footer.end_tick, 100);
        assert!(replay.validate().is_ok());
    }

    #[test]
    fn record_while_not_recording_is_noop() {
        let mut recorder = ReplayRecorder::default();
        recorder.record(1, GameAction::SetSpeed { speed: 1 });
        assert_eq!(recorder.entry_count(), 0);
    }
}
