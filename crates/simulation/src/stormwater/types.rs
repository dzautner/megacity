use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

/// Grid tracking accumulated stormwater runoff per cell.
///
/// During rain events, runoff accumulates based on cell imperviousness.
/// Between rain events, runoff gradually drains away.
#[derive(Resource, Serialize, Deserialize)]
pub struct StormwaterGrid {
    /// Accumulated runoff volume per cell (cubic meters, scaled).
    pub runoff: Vec<f32>,
    /// Total runoff across the entire grid (for stats display).
    pub total_runoff: f32,
    /// Total infiltration across the grid this tick.
    pub total_infiltration: f32,
    pub width: usize,
    pub height: usize,
}

impl Default for StormwaterGrid {
    fn default() -> Self {
        Self {
            runoff: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            total_runoff: 0.0,
            total_infiltration: 0.0,
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl StormwaterGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.runoff[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        self.runoff[y * self.width + x] = val;
    }

    #[inline]
    pub(crate) fn add(&mut self, x: usize, y: usize, amount: f32) {
        let idx = y * self.width + x;
        self.runoff[idx] += amount;
    }
}

/// Stormwater plugin registers the grid resource and update system.
pub struct StormwaterPlugin;

impl Plugin for StormwaterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StormwaterGrid>().add_systems(
            FixedUpdate,
            super::systems::update_stormwater
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
