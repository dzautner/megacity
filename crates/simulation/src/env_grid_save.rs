//! Saveable implementations for environmental grid resources.
//!
//! These grids are recomputed every slow tick, but persisting them avoids
//! a full recomputation gap after a save/load cycle. Each grid stores a
//! `Vec<u8>` (or `Vec<bool>` / `Vec<f32>`) that is encoded via `bitcode`.
//!
//! The plugin registers all grids with the `SaveableRegistry` so the save
//! system picks them up automatically.

use bevy::prelude::*;

use crate::crime::CrimeGrid;
use crate::forest_fire::ForestFireGrid;
use crate::groundwater::{GroundwaterGrid, WaterQualityGrid};
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::stormwater::StormwaterGrid;
use crate::trees::TreeGrid;
use crate::water_pollution::WaterPollutionGrid;
use crate::Saveable;

// ---------------------------------------------------------------------------
// Helper: encode/decode a Vec<u8> grid via bitcode
// ---------------------------------------------------------------------------

fn encode_u8_grid(data: &[u8]) -> Vec<u8> {
    bitcode::encode(data)
}

fn decode_u8_grid(bytes: &[u8], default_len: usize) -> Vec<u8> {
    match bitcode::decode::<Vec<u8>>(bytes) {
        Ok(v) => {
            if v.len() == default_len {
                v
            } else {
                vec![0; default_len]
            }
        }
        Err(_) => vec![0; default_len],
    }
}

// ---------------------------------------------------------------------------
// PollutionGrid
// ---------------------------------------------------------------------------

impl Saveable for PollutionGrid {
    const SAVE_KEY: &'static str = "pollution_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.levels.iter().all(|&v| v == 0) {
            return None;
        }
        Some(encode_u8_grid(&self.levels))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let expected = crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT;
        let levels = decode_u8_grid(bytes, expected);
        Self {
            levels,
            width: crate::config::GRID_WIDTH,
            height: crate::config::GRID_HEIGHT,
        }
    }
}

// ---------------------------------------------------------------------------
// NoisePollutionGrid
// ---------------------------------------------------------------------------

impl Saveable for NoisePollutionGrid {
    const SAVE_KEY: &'static str = "noise_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.levels.iter().all(|&v| v == 0) {
            return None;
        }
        Some(encode_u8_grid(&self.levels))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let expected = crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT;
        let levels = decode_u8_grid(bytes, expected);
        Self {
            levels,
            width: crate::config::GRID_WIDTH,
            height: crate::config::GRID_HEIGHT,
        }
    }
}

// ---------------------------------------------------------------------------
// CrimeGrid
// ---------------------------------------------------------------------------

impl Saveable for CrimeGrid {
    const SAVE_KEY: &'static str = "crime_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.levels.iter().all(|&v| v == 0) {
            return None;
        }
        Some(encode_u8_grid(&self.levels))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let expected = crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT;
        let levels = decode_u8_grid(bytes, expected);
        Self {
            levels,
            width: crate::config::GRID_WIDTH,
            height: crate::config::GRID_HEIGHT,
        }
    }
}

// ---------------------------------------------------------------------------
// TreeGrid
// ---------------------------------------------------------------------------

impl Saveable for TreeGrid {
    const SAVE_KEY: &'static str = "tree_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.cells.iter().all(|&v| !v) {
            return None;
        }
        Some(bitcode::encode(&self.cells))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let expected = crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT;
        let cells = match bitcode::decode::<Vec<bool>>(bytes) {
            Ok(v) if v.len() == expected => v,
            _ => vec![false; expected],
        };
        Self {
            cells,
            width: crate::config::GRID_WIDTH,
            height: crate::config::GRID_HEIGHT,
        }
    }
}

// ---------------------------------------------------------------------------
// WaterPollutionGrid
// ---------------------------------------------------------------------------

impl Saveable for WaterPollutionGrid {
    const SAVE_KEY: &'static str = "water_pollution_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.levels.iter().all(|&v| v == 0) {
            return None;
        }
        Some(encode_u8_grid(&self.levels))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let expected = crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT;
        let levels = decode_u8_grid(bytes, expected);
        Self {
            levels,
            width: crate::config::GRID_WIDTH,
            height: crate::config::GRID_HEIGHT,
        }
    }
}

// ---------------------------------------------------------------------------
// GroundwaterGrid
// ---------------------------------------------------------------------------

impl Saveable for GroundwaterGrid {
    const SAVE_KEY: &'static str = "groundwater_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Default is 128 for all cells; skip if unchanged
        if self.levels.iter().all(|&v| v == 128) {
            return None;
        }
        Some(encode_u8_grid(&self.levels))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let expected = crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT;
        let levels = match bitcode::decode::<Vec<u8>>(bytes) {
            Ok(v) if v.len() == expected => v,
            _ => vec![128; expected],
        };
        Self {
            levels,
            width: crate::config::GRID_WIDTH,
            height: crate::config::GRID_HEIGHT,
        }
    }
}

// ---------------------------------------------------------------------------
// WaterQualityGrid
// ---------------------------------------------------------------------------

impl Saveable for WaterQualityGrid {
    const SAVE_KEY: &'static str = "water_quality_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Default is 200 for all cells; skip if unchanged
        if self.levels.iter().all(|&v| v == 200) {
            return None;
        }
        Some(encode_u8_grid(&self.levels))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let expected = crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT;
        let levels = match bitcode::decode::<Vec<u8>>(bytes) {
            Ok(v) if v.len() == expected => v,
            _ => vec![200; expected],
        };
        Self {
            levels,
            width: crate::config::GRID_WIDTH,
            height: crate::config::GRID_HEIGHT,
        }
    }
}

// ---------------------------------------------------------------------------
// StormwaterGrid
// ---------------------------------------------------------------------------

impl Saveable for StormwaterGrid {
    const SAVE_KEY: &'static str = "stormwater_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.runoff.iter().all(|&v| v == 0.0) {
            return None;
        }
        Some(bitcode::encode(&self.runoff))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let expected = crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT;
        let runoff = match bitcode::decode::<Vec<f32>>(bytes) {
            Ok(v) if v.len() == expected => v,
            _ => vec![0.0; expected],
        };
        Self {
            runoff,
            total_runoff: 0.0,
            total_infiltration: 0.0,
            width: crate::config::GRID_WIDTH,
            height: crate::config::GRID_HEIGHT,
        }
    }
}

// ---------------------------------------------------------------------------
// ForestFireGrid
// ---------------------------------------------------------------------------

impl Saveable for ForestFireGrid {
    const SAVE_KEY: &'static str = "forest_fire_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.intensities.iter().all(|&v| v == 0) {
            return None;
        }
        Some(encode_u8_grid(&self.intensities))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let expected = crate::config::GRID_WIDTH * crate::config::GRID_HEIGHT;
        let intensities = decode_u8_grid(bytes, expected);
        Self {
            intensities,
            width: crate::config::GRID_WIDTH,
            height: crate::config::GRID_HEIGHT,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct EnvGridSavePlugin;

impl Plugin for EnvGridSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::SaveableRegistry>();
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<PollutionGrid>();
        registry.register::<NoisePollutionGrid>();
        registry.register::<CrimeGrid>();
        registry.register::<TreeGrid>();
        registry.register::<WaterPollutionGrid>();
        registry.register::<GroundwaterGrid>();
        registry.register::<WaterQualityGrid>();
        registry.register::<StormwaterGrid>();
        registry.register::<ForestFireGrid>();
    }
}
