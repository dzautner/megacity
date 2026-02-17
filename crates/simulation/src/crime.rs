use bevy::prelude::*;

use crate::config::{GRID_WIDTH, GRID_HEIGHT};
use crate::grid::WorldGrid;
use crate::services::{ServiceBuilding, ServiceType};
use crate::land_value::LandValueGrid;

/// Crime probability grid - higher values = more crime
#[derive(Resource)]
pub struct CrimeGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for CrimeGrid {
    fn default() -> Self {
        Self {
            levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl CrimeGrid {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }
}

pub fn update_crime(
    slow_timer: Res<crate::SlowTickTimer>,
    mut crime: ResMut<CrimeGrid>,
    grid: Res<WorldGrid>,
    land_value: Res<LandValueGrid>,
    services: Query<&ServiceBuilding>,
) {
    if !slow_timer.should_run() { return; }
    // Base crime level from land value (low value = more crime)
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.zone == crate::grid::ZoneType::None && cell.building_id.is_none() {
                crime.set(x, y, 0);
                continue;
            }

            // Base crime inversely proportional to land value
            let lv = land_value.get(x, y) as i32;
            let base_crime = ((100 - lv).max(0) / 4) as u8; // 0-25 base
            crime.set(x, y, base_crime);
        }
    }

    // Police services reduce crime in radius
    let mut prison_count = 0u32;
    for service in &services {
        if service.service_type == ServiceType::Prison {
            prison_count += 1;
            continue;
        }
        if !ServiceBuilding::is_police(service.service_type) {
            continue;
        }
        let radius = (service.radius / 16.0) as i32;
        let reduction = match service.service_type {
            ServiceType::PoliceKiosk => 10u8,
            ServiceType::PoliceStation => 20u8,
            ServiceType::PoliceHQ => 30u8,
            _ => 15u8,
        };
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = service.grid_x as i32 + dx;
                let ny = service.grid_y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                    let dist = dx.abs() + dy.abs();
                    let effect = reduction.saturating_sub(dist as u8);
                    let idx = ny as usize * GRID_WIDTH + nx as usize;
                    crime.levels[idx] = crime.levels[idx].saturating_sub(effect);
                }
            }
        }
    }

    // Prison provides flat city-wide crime reduction (10% per prison)
    if prison_count > 0 {
        let flat_reduction = (prison_count * 3).min(10) as u8; // up to -10
        for level in &mut crime.levels {
            *level = level.saturating_sub(flat_reduction);
        }
    }
}
