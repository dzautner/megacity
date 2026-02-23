use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

// =============================================================================
// Resources
// =============================================================================

/// Per-cell forest fire intensity grid. 0 = no fire, 1-255 = burning intensity.
#[derive(Resource)]
pub struct ForestFireGrid {
    pub intensities: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for ForestFireGrid {
    fn default() -> Self {
        Self {
            intensities: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl ForestFireGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.intensities[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.intensities[y * self.width + x] = val;
    }
}

/// Tracks forest fire statistics over time.
#[derive(Resource, Default, Debug)]
pub struct ForestFireStats {
    pub active_fires: u32,
    pub total_area_burned: u64,
    pub fires_this_month: u32,
}
