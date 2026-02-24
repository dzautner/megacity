//! Saveable implementation for `ActiveDisaster` so in-progress disasters
//! persist across save/load cycles.

use bevy::prelude::*;

use crate::disasters::{ActiveDisaster, DisasterInstance};
use crate::Saveable;

impl Saveable for ActiveDisaster {
    const SAVE_KEY: &'static str = "active_disaster";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no disaster is active (default state).
        let instance = self.current.as_ref()?;
        Some(bitcode::encode(instance))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        match bitcode::decode::<DisasterInstance>(bytes) {
            Ok(instance) => Self {
                current: Some(instance),
            },
            Err(e) => {
                warn!(
                    "Saveable active_disaster: failed to decode {} bytes, \
                     falling back to no active disaster: {}",
                    bytes.len(),
                    e
                );
                Self::default()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct DisasterSavePlugin;

impl Plugin for DisasterSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ActiveDisaster>();
    }
}
