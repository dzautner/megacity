//! Replay player: feeds recorded `GameAction`s into the `ActionQueue` at the
//! correct tick during playback.
//!
//! The player system runs in `PreSim` before the executor, injecting actions
//! with `ActionSource::Replay` so the simulation processes them through the
//! same code path as live play — deterministic by design.

use bevy::prelude::*;

use crate::game_actions::{ActionQueue, ActionSource};
use crate::TickCounter;

use super::format::ReplayFile;

/// Resource that drives replay playback by feeding actions at the right tick.
#[derive(Resource, Default)]
pub struct ReplayPlayer {
    replay: Option<ReplayFile>,
    /// Index into `replay.entries` — entries before this index have been fed.
    cursor: usize,
    playing: bool,
}

impl ReplayPlayer {
    /// Load a replay file and reset the cursor to the beginning.
    pub fn load(&mut self, replay: ReplayFile) {
        self.replay = Some(replay);
        self.cursor = 0;
        self.playing = true;
    }

    /// Whether playback is currently active.
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Whether playback has reached the end of the entry list.
    pub fn is_finished(&self) -> bool {
        match &self.replay {
            Some(replay) => self.cursor >= replay.entries.len(),
            None => true,
        }
    }

    /// Return all actions whose tick matches `tick` and advance the cursor.
    ///
    /// Entries must be sorted by tick (guaranteed by `ReplayFile::validate`).
    /// Returns an empty vec if no entries match or playback is inactive.
    pub fn actions_for_tick(
        &mut self,
        tick: u64,
    ) -> Vec<crate::game_actions::GameAction> {
        if !self.playing {
            return Vec::new();
        }
        let replay = match &self.replay {
            Some(r) => r,
            None => return Vec::new(),
        };

        let mut actions = Vec::new();
        while self.cursor < replay.entries.len() {
            let entry = &replay.entries[self.cursor];
            if entry.tick == tick {
                actions.push(entry.action.clone());
                self.cursor += 1;
            } else if entry.tick > tick {
                // Entries are sorted — no more matches for this tick.
                break;
            } else {
                // entry.tick < tick: this entry was for an earlier tick that
                // we missed (e.g. player loaded mid-session). Skip it.
                self.cursor += 1;
            }
        }
        actions
    }

    /// Stop playback and release the replay data.
    pub fn stop(&mut self) {
        self.playing = false;
        self.replay = None;
        self.cursor = 0;
    }

    /// Current cursor position (number of entries already fed).
    pub fn cursor(&self) -> usize {
        self.cursor
    }
}

/// System that feeds replay actions into the `ActionQueue` at the correct tick.
///
/// Runs in `PreSim` so that the executor (or any system that drains the queue)
/// sees the replayed actions in the same frame they would have been enqueued
/// during live play.
pub fn feed_replay_actions(
    tick: Res<TickCounter>,
    mut player: ResMut<ReplayPlayer>,
    mut queue: ResMut<ActionQueue>,
) {
    if !player.is_playing() {
        return;
    }
    let actions = player.actions_for_tick(tick.0);
    for action in actions {
        queue.push(tick.0, ActionSource::Replay, action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_actions::GameAction;
    use crate::grid::RoadType;
    use crate::replay::format::{ReplayEntry, ReplayFooter, ReplayHeader};

    fn test_replay() -> ReplayFile {
        ReplayFile {
            header: ReplayHeader {
                format_version: 1,
                seed: 42,
                city_name: "TestCity".to_string(),
                start_tick: 0,
            },
            entries: vec![
                ReplayEntry {
                    tick: 1,
                    action: GameAction::SetSpeed { speed: 2 },
                },
                ReplayEntry {
                    tick: 1,
                    action: GameAction::SetPaused { paused: false },
                },
                ReplayEntry {
                    tick: 5,
                    action: GameAction::PlaceRoadLine {
                        start: (10, 10),
                        end: (20, 10),
                        road_type: RoadType::Local,
                    },
                },
            ],
            footer: ReplayFooter {
                end_tick: 100,
                final_state_hash: 0,
                entry_count: 3,
            },
        }
    }

    #[test]
    fn feeds_correct_actions_for_each_tick() {
        let mut player = ReplayPlayer::default();
        player.load(test_replay());

        assert!(player.is_playing());
        assert!(!player.is_finished());

        // Tick 0: no entries
        let actions = player.actions_for_tick(0);
        assert!(actions.is_empty());

        // Tick 1: two entries
        let actions = player.actions_for_tick(1);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0], GameAction::SetSpeed { speed: 2 });
        assert_eq!(actions[1], GameAction::SetPaused { paused: false });

        // Tick 2-4: no entries
        assert!(player.actions_for_tick(2).is_empty());
        assert!(player.actions_for_tick(3).is_empty());

        // Tick 5: one entry
        let actions = player.actions_for_tick(5);
        assert_eq!(actions.len(), 1);

        assert!(player.is_finished());
    }

    #[test]
    fn stop_clears_state() {
        let mut player = ReplayPlayer::default();
        player.load(test_replay());
        player.stop();

        assert!(!player.is_playing());
        assert!(player.is_finished());
        assert_eq!(player.cursor(), 0);
    }
}
