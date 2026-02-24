//! Saveable implementation for `TrafficGrid`.
//!
//! Traffic density is recomputed every 5 ticks, but persisting it avoids a
//! blank traffic overlay immediately after load. The grid stores `Vec<u16>`
//! encoded via `bitcode`.

use bevy::prelude::*;

use crate::traffic::TrafficGrid;
use crate::Saveable;

impl Saveable for TrafficGrid {
    const SAVE_KEY: &'static str = "traffic_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.density.iter().all(|&v| v == 0) {
            return None;
        }
        Some(bitcode::encode(&self.density))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let expected = crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT;
        let density = match bitcode::decode::<Vec<u16>>(bytes) {
            Ok(v) if v.len() == expected => v,
            _ => vec![0; expected],
        };
        Self {
            density,
            width: crate::config::GRID_WIDTH,
            height: crate::config::GRID_HEIGHT,
        }
    }
}

/// Plugin that registers `TrafficGrid` with the `SaveableRegistry`.
pub struct TrafficGridSavePlugin;

impl Plugin for TrafficGridSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::SaveableRegistry>();
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<TrafficGrid>();
    }
}
