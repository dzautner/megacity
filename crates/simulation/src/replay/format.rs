//! Core data types for the deterministic replay file format.
//!
//! A replay file captures a sequence of `GameAction` entries tagged by tick,
//! allowing exact reproduction of a game session when replayed with the same
//! seed and simulation code.

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::game_actions::GameAction;

/// Format version for forward-compatibility checks.
pub const CURRENT_FORMAT_VERSION: u32 = 1;

/// Header metadata written at the start of a replay file.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq)]
pub struct ReplayHeader {
    /// Format version (start at 1, bump on breaking changes).
    pub format_version: u32,
    /// RNG seed used for the simulation.
    pub seed: u64,
    /// Name of the city at recording start.
    pub city_name: String,
    /// Tick number when recording began.
    pub start_tick: u64,
}

/// A single recorded action with its tick number.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq)]
pub struct ReplayEntry {
    /// Tick at which this action was queued.
    pub tick: u64,
    /// The game action that was performed.
    pub action: GameAction,
}

/// Footer metadata written after all entries.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq)]
pub struct ReplayFooter {
    /// Tick number when recording stopped.
    pub end_tick: u64,
    /// Hash of the final simulation state (0 if unavailable).
    pub final_state_hash: u64,
    /// Number of entries in the replay (for validation).
    pub entry_count: u64,
}

/// Complete replay file: header + entries + footer.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq)]
pub struct ReplayFile {
    pub header: ReplayHeader,
    pub entries: Vec<ReplayEntry>,
    pub footer: ReplayFooter,
}

impl ReplayFile {
    /// Encode the replay file to compact binary bytes via bitcode.
    pub fn to_bytes(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    /// Decode a replay file from bitcode bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        bitcode::decode(bytes).map_err(|e| format!("bitcode decode error: {e}"))
    }

    /// Serialize to JSON for human-readable debugging output.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
    }

    /// Deserialize from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("JSON decode error: {e}"))
    }

    /// Validate internal consistency:
    /// - `footer.entry_count` matches `entries.len()`
    /// - Entries are sorted by tick (non-decreasing)
    pub fn validate(&self) -> Result<(), String> {
        if self.footer.entry_count != self.entries.len() as u64 {
            return Err(format!(
                "entry_count mismatch: footer says {} but found {} entries",
                self.footer.entry_count,
                self.entries.len()
            ));
        }

        for window in self.entries.windows(2) {
            if window[1].tick < window[0].tick {
                return Err(format!(
                    "entries not sorted by tick: tick {} followed by {}",
                    window[0].tick, window[1].tick
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::RoadType;

    fn sample_replay() -> ReplayFile {
        ReplayFile {
            header: ReplayHeader {
                format_version: CURRENT_FORMAT_VERSION,
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
                    tick: 5,
                    action: GameAction::PlaceRoadLine {
                        start: (10, 10),
                        end: (20, 10),
                        road_type: RoadType::Local,
                    },
                },
                ReplayEntry {
                    tick: 5,
                    action: GameAction::SetPaused { paused: false },
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
    fn bitcode_roundtrip() {
        let original = sample_replay();
        let bytes = original.to_bytes();
        let decoded = ReplayFile::from_bytes(&bytes).expect("decode should succeed");
        assert_eq!(original, decoded);
    }

    #[test]
    fn json_roundtrip() {
        let original = sample_replay();
        let json = original.to_json();
        let decoded = ReplayFile::from_json(&json).expect("JSON decode should succeed");
        assert_eq!(original, decoded);
    }

    #[test]
    fn validate_correct_file() {
        let replay = sample_replay();
        assert!(replay.validate().is_ok());
    }

    #[test]
    fn validate_catches_entry_count_mismatch() {
        let mut replay = sample_replay();
        replay.footer.entry_count = 999;
        let err = replay.validate().unwrap_err();
        assert!(err.contains("entry_count mismatch"));
    }

    #[test]
    fn validate_catches_unsorted_ticks() {
        let mut replay = sample_replay();
        // Swap entries so tick order is broken: [1, 5, 5] -> [5, 1, 5]
        replay.entries.swap(0, 1);
        let err = replay.validate().unwrap_err();
        assert!(err.contains("not sorted by tick"));
    }
}
