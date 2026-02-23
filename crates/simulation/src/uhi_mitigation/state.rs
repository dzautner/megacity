//! [`UhiMitigationState`] resource that tracks all deployed UHI mitigations.

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

// =============================================================================
// Resources
// =============================================================================

/// Tracks all UHI mitigation measures deployed across the city.
///
/// Grid-level mitigations (cool pavement, parks, permeable surfaces) are stored
/// as boolean grids. Building-level mitigations (green roofs, cool roofs) are
/// stored as counts. Point mitigations (water features, district cooling) are
/// stored as coordinate lists.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct UhiMitigationState {
    // --- Building-level mitigations ---
    /// Number of buildings upgraded with green roofs.
    pub green_roof_count: u32,
    /// Number of buildings upgraded with cool (white) roofs.
    pub cool_roof_count: u32,

    // --- Grid-level mitigations (per-cell booleans) ---
    /// Cells with cool pavement applied.
    pub cool_pavement_cells: Vec<bool>,
    /// Cells designated as parks for UHI mitigation.
    pub park_cells: Vec<bool>,
    /// Cells with permeable surfaces applied.
    pub permeable_surface_cells: Vec<bool>,

    // --- Point mitigations ---
    /// Locations of water features (fountains). Each entry is `(x, y)`.
    pub water_features: Vec<(usize, usize)>,
    /// Locations of district cooling facilities. Each entry is `(x, y)`.
    pub district_cooling_facilities: Vec<(usize, usize)>,

    // --- Cost tracking ---
    /// Total cumulative cost of all UHI mitigation measures.
    pub total_cost: f64,

    // --- Derived (computed each update) ---
    /// Total UHI reduction applied across all cells this tick (for stats/UI).
    pub total_cells_mitigated: u32,
}

impl Default for UhiMitigationState {
    fn default() -> Self {
        let grid_size = GRID_WIDTH * GRID_HEIGHT;
        Self {
            green_roof_count: 0,
            cool_roof_count: 0,
            cool_pavement_cells: vec![false; grid_size],
            park_cells: vec![false; grid_size],
            permeable_surface_cells: vec![false; grid_size],
            water_features: Vec::new(),
            district_cooling_facilities: Vec::new(),
            total_cost: 0.0,
            total_cells_mitigated: 0,
        }
    }
}

impl UhiMitigationState {
    /// Check if cool pavement is applied at `(x, y)`.
    #[inline]
    pub fn has_cool_pavement(&self, x: usize, y: usize) -> bool {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.cool_pavement_cells[y * GRID_WIDTH + x]
        } else {
            false
        }
    }

    /// Check if a park is placed at `(x, y)`.
    #[inline]
    pub fn has_park(&self, x: usize, y: usize) -> bool {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.park_cells[y * GRID_WIDTH + x]
        } else {
            false
        }
    }

    /// Check if permeable surfaces are applied at `(x, y)`.
    #[inline]
    pub fn has_permeable_surface(&self, x: usize, y: usize) -> bool {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.permeable_surface_cells[y * GRID_WIDTH + x]
        } else {
            false
        }
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for UhiMitigationState {
    const SAVE_KEY: &'static str = "uhi_mitigation";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if nothing has been deployed
        if self.green_roof_count == 0
            && self.cool_roof_count == 0
            && self.water_features.is_empty()
            && self.district_cooling_facilities.is_empty()
            && self.total_cost == 0.0
            && !self.cool_pavement_cells.iter().any(|&v| v)
            && !self.park_cells.iter().any(|&v| v)
            && !self.permeable_surface_cells.iter().any(|&v| v)
        {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
