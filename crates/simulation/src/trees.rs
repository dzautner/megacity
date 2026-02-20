use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::land_value::LandValueGrid;
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::TickCounter;

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

/// Runs every 50 ticks: trees reduce noise pollution and air pollution in
/// neighboring cells (radius 2), and boost land value in radius 1.
pub fn tree_effects(
    tick: Res<TickCounter>,
    tree_grid: Res<TreeGrid>,
    mut pollution: ResMut<PollutionGrid>,
    mut noise: ResMut<NoisePollutionGrid>,
    mut land_value: ResMut<LandValueGrid>,
) {
    if !tick.0.is_multiple_of(50) {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if !tree_grid.has_tree(x, y) {
                continue;
            }

            // Radius 2: reduce noise and air pollution
            let radius = 2i32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                    {
                        let ux = nx as usize;
                        let uy = ny as usize;
                        let dist = dx.abs() + dy.abs();
                        // Stronger effect closer to tree
                        let reduction = (3 - dist).max(0) as u8; // 3 at center, 2 at dist 1, 1 at dist 2

                        // Reduce air pollution
                        let cur_pol = pollution.get(ux, uy);
                        pollution.set(ux, uy, cur_pol.saturating_sub(reduction));

                        // Reduce noise pollution
                        let cur_noise = noise.get(ux, uy);
                        noise.set(ux, uy, cur_noise.saturating_sub(reduction));
                    }
                }
            }

            // Radius 1: +2 land value boost
            let lv_radius = 1i32;
            for dy in -lv_radius..=lv_radius {
                for dx in -lv_radius..=lv_radius {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                    {
                        let ux = nx as usize;
                        let uy = ny as usize;
                        let cur = land_value.get(ux, uy);
                        land_value.set(ux, uy, cur.saturating_add(2));
                    }
                }
            }
        }
    }
}

pub struct TreesPlugin;

impl Plugin for TreesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TreeGrid>()
            .add_systems(
                FixedUpdate,
                tree_effects.after(crate::imports_exports::process_trade),
            );
    }
}
