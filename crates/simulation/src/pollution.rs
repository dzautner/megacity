use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use bevy::prelude::*;

#[derive(Resource)]
pub struct PollutionGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for PollutionGrid {
    fn default() -> Self {
        Self {
            levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl PollutionGrid {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }
}

pub struct PollutionPlugin;

impl Plugin for PollutionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PollutionGrid>().add_systems(
            FixedUpdate,
            crate::wind_pollution::update_pollution_gaussian_plume
                .after(crate::education::propagate_education)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
