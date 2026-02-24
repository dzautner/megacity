//! Saveable implementation for HeatingGrid.
//!
//! Persists heating network coverage across save/load cycles so the
//! heating overlay matches pre-save state without waiting for a full
//! recomputation tick.

use bevy::prelude::*;

use crate::heating::HeatingGrid;
use crate::Saveable;

impl Saveable for HeatingGrid {
    const SAVE_KEY: &'static str = "heating_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.levels.iter().all(|&v| v == 0) {
            return None;
        }
        Some(bitcode::encode(&self.levels))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let expected = crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT;
        let levels = match bitcode::decode::<Vec<u8>>(bytes) {
            Ok(v) if v.len() == expected => v,
            _ => vec![0; expected],
        };
        Self {
            levels,
            width: crate::config::GRID_WIDTH,
            height: crate::config::GRID_HEIGHT,
        }
    }
}

/// Plugin that registers `HeatingGrid` with the saveable registry.
pub struct HeatingSavePlugin;

impl Plugin for HeatingSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::SaveableRegistry>();
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<HeatingGrid>();
    }
}
