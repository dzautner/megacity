use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::pollution::PollutionGrid;
use crate::services::ServiceBuilding;
use bevy::prelude::*;

#[derive(Resource)]
pub struct LandValueGrid {
    pub values: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for LandValueGrid {
    fn default() -> Self {
        Self {
            values: vec![50; GRID_WIDTH * GRID_HEIGHT], // start at baseline 50
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl LandValueGrid {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.values[y * self.width + x]
    }
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.values[y * self.width + x] = val;
    }
    pub fn average(&self) -> f32 {
        if self.values.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.values.iter().map(|&v| v as u64).sum();
        sum as f32 / self.values.len() as f32
    }
}

pub fn update_land_value(
    slow_timer: Res<crate::SlowTickTimer>,
    mut land_value: ResMut<LandValueGrid>,
    grid: Res<WorldGrid>,
    pollution: Res<PollutionGrid>,
    services: Query<&ServiceBuilding>,
) {
    if !slow_timer.should_run() {
        return;
    }
    // Reset to base value
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            let mut value: i32 = 50;

            // Water proximity boost
            if cell.cell_type == CellType::Water {
                value = 30; // water itself isn't valuable
            } else {
                // Check for nearby water
                let (lv_n4, lv_n4c) = grid.neighbors4(x, y);
                for &(nx, ny) in &lv_n4[..lv_n4c] {
                    if grid.get(nx, ny).cell_type == CellType::Water {
                        value += 15;
                        break;
                    }
                }
            }

            // Industrial reduces nearby land value
            if cell.zone == ZoneType::Industrial {
                value -= 15;
            }

            // Pollution reduces value
            let poll = pollution.get(x, y) as i32;
            value -= poll / 3;

            land_value.set(x, y, value.clamp(0, 255) as u8);
        }
    }

    // Parks and services boost land value in radius
    for service in &services {
        let (boost, radius): (i32, i32) = if ServiceBuilding::is_park(service.service_type) {
            (20, 8)
        } else {
            (10, 6)
        };

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = service.grid_x as i32 + dx;
                let ny = service.grid_y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                    let dist = dx.abs() + dy.abs();
                    let effect = (boost - dist * 2).max(0);
                    let cur = land_value.get(nx as usize, ny as usize);
                    land_value.set(
                        nx as usize,
                        ny as usize,
                        (cur as i32 + effect).min(255) as u8,
                    );
                }
            }
        }
    }
}
