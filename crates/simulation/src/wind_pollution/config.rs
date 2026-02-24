//! WindPollutionConfig resource and Saveable implementation.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{decode_or_warn, Saveable};

/// Configuration for wind-aware pollution dispersion, including technology
/// upgrades like scrubbers.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode, Default)]
pub struct WindPollutionConfig {
    /// Whether scrubber technology is installed (reduces emissions by 50%).
    pub scrubbers_enabled: bool,
}

impl Saveable for WindPollutionConfig {
    const SAVE_KEY: &'static str = "wind_pollution_config";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if !self.scrubbers_enabled {
            return None; // skip saving default state
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        decode_or_warn::<Self>(Self::SAVE_KEY, bytes)
    }
}
