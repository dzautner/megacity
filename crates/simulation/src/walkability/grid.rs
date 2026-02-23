//! `WalkabilityGrid` resource — per-cell walkability scores and city average.

use bitcode::{Decode, Encode};

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

// =============================================================================
// Public constants
// =============================================================================

/// Maximum happiness bonus from walkability score.
pub const WALKABILITY_HAPPINESS_BONUS: f32 = 8.0;

/// Maximum land value bonus from walkability score.
pub const WALKABILITY_LAND_VALUE_BONUS: i32 = 15;

// =============================================================================
// Walkability grid resource
// =============================================================================

/// Per-cell walkability score (0-100), recomputed every slow tick.
#[derive(Resource, Clone, Encode, Decode)]
pub struct WalkabilityGrid {
    /// One score per cell, indexed as `y * GRID_WIDTH + x`.
    pub scores: Vec<u8>,
    /// City-wide average walkability score.
    pub city_average: f32,
}

impl Default for WalkabilityGrid {
    fn default() -> Self {
        Self {
            scores: vec![0; GRID_WIDTH * GRID_HEIGHT],
            city_average: 0.0,
        }
    }
}

impl WalkabilityGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.scores[y * GRID_WIDTH + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.scores[y * GRID_WIDTH + x] = val;
    }

    /// Returns the walkability score as a 0.0-1.0 fraction.
    #[inline]
    pub fn fraction(&self, x: usize, y: usize) -> f32 {
        self.get(x, y) as f32 / 100.0
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for WalkabilityGrid {
    const SAVE_KEY: &'static str = "walkability";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if all scores are zero (no city built yet)
        if self.scores.iter().all(|&s| s == 0) {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
