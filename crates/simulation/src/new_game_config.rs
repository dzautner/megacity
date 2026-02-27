//! PLAY-019: New Game Map Options.
//!
//! Provides a `NewGameConfig` resource that stores the player's chosen
//! city name and terrain seed for new games. The main menu UI writes to
//! this resource before sending `NewGameEvent`, and the new-game and save
//! systems read from it.

use bevy::prelude::*;

use crate::Saveable;

/// Configuration chosen by the player when starting a new game.
///
/// Persists across save/load so the city name and original seed are remembered.
#[derive(Resource, Debug, Clone, serde::Serialize, serde::Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct NewGameConfig {
    /// Player-chosen city name (displayed in UI and save metadata).
    pub city_name: String,
    /// Terrain generation seed.
    pub seed: u64,
}

impl Default for NewGameConfig {
    fn default() -> Self {
        Self {
            city_name: "New City".to_string(),
            seed: random_seed(),
        }
    }
}

/// Generate a random seed from the current system time.
pub fn random_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42)
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl Saveable for NewGameConfig {
    const SAVE_KEY: &'static str = "new_game_config";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct NewGameConfigPlugin;

impl Plugin for NewGameConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NewGameConfig>();

        app.init_resource::<crate::SaveableRegistry>();
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<NewGameConfig>();
    }
}
