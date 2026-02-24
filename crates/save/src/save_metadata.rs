// ---------------------------------------------------------------------------
// save_metadata – Quick-access metadata for save file load screens
// ---------------------------------------------------------------------------
//
// SaveMetadata is a small, self-contained section encoded between the file
// header and the main save payload. It contains summary information (city
// name, population, treasury, game day/hour, play time) that the load screen
// can read without fully decoding the SaveData.
//
// File layout:
//   [Header 32 bytes] [Metadata (variable)] [Data payload]
//
// The header's `metadata_size` field (bytes 28..32) tells the reader how
// many bytes of metadata follow immediately after the header. If zero, no
// metadata is present (backward compatible with v1 headers).

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Quick-access metadata for the load screen.
///
/// This is encoded separately from the main `SaveData` so that the load
/// screen can display it without performing a full decode of the entire
/// save file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct SaveMetadata {
    /// Display name for the city (derived from population milestones).
    pub city_name: String,
    /// Total population (real ECS citizens + virtual citizens).
    pub population: u32,
    /// Treasury balance at the time of save.
    pub treasury: f64,
    /// In-game day number.
    pub day: u32,
    /// In-game hour (0.0 .. 24.0).
    pub hour: f32,
    /// Total wall-clock play time in seconds.
    pub play_time_seconds: f64,
}

impl Default for SaveMetadata {
    fn default() -> Self {
        Self {
            city_name: "Settlement".to_string(),
            population: 0,
            treasury: 0.0,
            day: 1,
            hour: 6.0,
            play_time_seconds: 0.0,
        }
    }
}

impl SaveMetadata {
    /// Encode metadata to bytes using bitcode.
    pub fn encode(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    /// Decode metadata from bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, bitcode::Error> {
        bitcode::decode(bytes)
    }
}
