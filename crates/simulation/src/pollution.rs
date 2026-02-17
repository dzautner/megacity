use bevy::prelude::*;
use crate::config::{GRID_WIDTH, GRID_HEIGHT};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::buildings::Building;
use crate::services::ServiceBuilding;

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

pub fn update_pollution(
    slow_timer: Res<crate::SlowTickTimer>,
    mut pollution: ResMut<PollutionGrid>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
    policies: Res<crate::policies::Policies>,
) {
    if !slow_timer.should_run() { return; }
    pollution.levels.fill(0);

    // Roads add +2 pollution
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type == CellType::Road {
                let idx = y * GRID_WIDTH + x;
                pollution.levels[idx] = pollution.levels[idx].saturating_add(2);
            }
        }
    }

    // Industrial buildings radiate pollution (reduced by policy)
    let pollution_mult = policies.pollution_multiplier();
    for building in &buildings {
        if building.zone_type == ZoneType::Industrial {
            let intensity = ((5 + building.level as i32 * 3) as f32 * pollution_mult) as i32;
            let radius = 8i32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = building.grid_x as i32 + dx;
                    let ny = building.grid_y as i32 + dy;
                    if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                        let dist = dx.abs() + dy.abs();
                        let decay = (intensity - dist).max(0) as u8;
                        let cur = pollution.get(nx as usize, ny as usize);
                        pollution.set(nx as usize, ny as usize, cur.saturating_add(decay));
                    }
                }
            }
        }
    }

    // Parks reduce pollution (negative effect)
    for service in &services {
        if ServiceBuilding::is_park(service.service_type) {
            let radius = 6i32;
            let reduction = 8u8;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = service.grid_x as i32 + dx;
                    let ny = service.grid_y as i32 + dy;
                    if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                        let dist = dx.abs() + dy.abs();
                        let effect = reduction.saturating_sub(dist as u8);
                        let cur = pollution.get(nx as usize, ny as usize);
                        pollution.set(nx as usize, ny as usize, cur.saturating_sub(effect));
                    }
                }
            }
        }
    }
}
