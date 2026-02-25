use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

/// Tracks which grid cells have player-placed trees.
#[derive(Resource)]
pub struct TreeGrid {
    pub cells: Vec<bool>,
    pub width: usize,
    pub height: usize,
}

impl Default for TreeGrid {
    fn default() -> Self {
        Self {
            cells: vec![false; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl TreeGrid {
    #[inline]
    pub fn has_tree(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x]
        } else {
            false
        }
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: bool) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = val;
        }
    }
}

/// ECS marker component for player-placed tree entities.
#[derive(Component)]
pub struct PlantedTree {
    pub grid_x: usize,
    pub grid_y: usize,
}

/// Cost of planting a single tree.
pub const TREE_PLANT_COST: f64 = 50.0;

/// TreesPlugin registers only the TreeGrid resource. The old flat-reduction
/// `tree_effects` system has been replaced by the percentage-based
/// `tree_absorption::tree_absorption_effects` (POLL-018).
pub struct TreesPlugin;

impl Plugin for TreesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TreeGrid>();
    }
}
