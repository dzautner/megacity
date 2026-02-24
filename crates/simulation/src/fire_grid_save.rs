//! Saveable implementations for fire-related resources.
//!
//! This module persists both the building fire grid (`FireGrid`) and the
//! forest fire statistics (`ForestFireStats`) across save/load cycles.
//! The `ForestFireGrid` is already handled by `env_grid_save.rs`.
//!
//! Together these ensure that active fires (both building and forest)
//! survive a save/load roundtrip and continue spreading after load.

use bevy::prelude::*;

use crate::fire::FireGrid;
use crate::forest_fire::ForestFireStats;
use crate::Saveable;

// ---------------------------------------------------------------------------
// FireGrid
// ---------------------------------------------------------------------------

impl Saveable for FireGrid {
    const SAVE_KEY: &'static str = "fire_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.fire_levels.iter().all(|&v| v == 0) {
            return None;
        }
        Some(bitcode::encode(&self.fire_levels))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let expected = crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT;
        let fire_levels = match bitcode::decode::<Vec<u8>>(bytes) {
            Ok(v) if v.len() == expected => v,
            _ => vec![0; expected],
        };
        Self {
            fire_levels,
            width: crate::config::GRID_WIDTH,
            height: crate::config::GRID_HEIGHT,
        }
    }
}

// ---------------------------------------------------------------------------
// ForestFireStats
// ---------------------------------------------------------------------------

/// Serializable form of `ForestFireStats` for bitcode encode/decode.
#[derive(bitcode::Encode, bitcode::Decode)]
struct ForestFireStatsData {
    active_fires: u32,
    total_area_burned: u64,
    fires_this_month: u32,
}

impl Saveable for ForestFireStats {
    const SAVE_KEY: &'static str = "forest_fire_stats";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.active_fires == 0 && self.total_area_burned == 0 && self.fires_this_month == 0 {
            return None;
        }
        let data = ForestFireStatsData {
            active_fires: self.active_fires,
            total_area_burned: self.total_area_burned,
            fires_this_month: self.fires_this_month,
        };
        Some(bitcode::encode(&data))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        match bitcode::decode::<ForestFireStatsData>(bytes) {
            Ok(data) => Self {
                active_fires: data.active_fires,
                total_area_burned: data.total_area_burned,
                fires_this_month: data.fires_this_month,
            },
            Err(_) => Self::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct FireGridSavePlugin;

impl Plugin for FireGridSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::SaveableRegistry>();
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<FireGrid>();
        registry.register::<ForestFireStats>();
    }
}
