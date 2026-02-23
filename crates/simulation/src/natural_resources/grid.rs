use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

use super::types::ResourceDeposit;

/// Grid of natural resource deposits, generated alongside terrain
#[derive(Resource)]
pub struct ResourceGrid {
    pub deposits: Vec<Option<ResourceDeposit>>,
    pub width: usize,
    pub height: usize,
}

impl Default for ResourceGrid {
    fn default() -> Self {
        Self {
            deposits: vec![None; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl ResourceGrid {
    pub fn get(&self, x: usize, y: usize) -> &Option<ResourceDeposit> {
        &self.deposits[y * self.width + x]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Option<ResourceDeposit> {
        &mut self.deposits[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, deposit: ResourceDeposit) {
        self.deposits[y * self.width + x] = Some(deposit);
    }
}
