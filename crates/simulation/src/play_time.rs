// ---------------------------------------------------------------------------
// play_time – Tracks total wall-clock play time for save metadata
// ---------------------------------------------------------------------------

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Tracks the total wall-clock play time in seconds.
///
/// Accumulated every frame using real (wall-clock) time, regardless of game
/// speed or pause state. Persisted via the `Saveable` extension map.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct PlayTime {
    /// Total play time in seconds (wall-clock).
    pub total_seconds: f64,
}

impl crate::Saveable for PlayTime {
    const SAVE_KEY: &'static str = "play_time";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

/// Accumulates wall-clock time each frame.
fn tick_play_time(time: Res<Time>, mut play_time: ResMut<PlayTime>) {
    play_time.total_seconds += time.delta_secs_f64();
}

pub struct PlayTimePlugin;

impl Plugin for PlayTimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayTime>()
            .add_systems(Update, tick_play_time);

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<PlayTime>();
    }
}
