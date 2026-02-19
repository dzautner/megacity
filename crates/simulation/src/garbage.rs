use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::services::{ServiceBuilding, ServiceType};
use bevy::prelude::*;

#[derive(Resource)]
pub struct GarbageGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for GarbageGrid {
    fn default() -> Self {
        Self {
            levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl GarbageGrid {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }
}

pub fn update_garbage(
    slow_timer: Res<crate::SlowTickTimer>,
    mut garbage: ResMut<GarbageGrid>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
    policies: Res<crate::policies::Policies>,
) {
    if !slow_timer.should_run() {
        return;
    }
    // Buildings produce garbage proportional to occupants (reduced by recycling policy)
    let garbage_mult = policies.garbage_multiplier();
    for building in &buildings {
        let production = ((building.occupants / 5).min(10) as f32 * garbage_mult) as u8;
        let cur = garbage.get(building.grid_x, building.grid_y);
        garbage.set(
            building.grid_x,
            building.grid_y,
            cur.saturating_add(production),
        );
    }

    // Garbage service buildings collect in radius
    for service in &services {
        if !ServiceBuilding::is_garbage(service.service_type) {
            continue;
        }
        let radius = (service.radius / 16.0) as i32;
        let collection = match service.service_type {
            ServiceType::Landfill => 3u8,
            ServiceType::RecyclingCenter => 5u8,
            ServiceType::Incinerator => 8u8,
            ServiceType::TransferStation => 4u8,
            _ => 0,
        };
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = service.grid_x as i32 + dx;
                let ny = service.grid_y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                    let cur = garbage.get(nx as usize, ny as usize);
                    garbage.set(nx as usize, ny as usize, cur.saturating_sub(collection));
                }
            }
        }
    }
}
