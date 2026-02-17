use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_WIDTH, GRID_HEIGHT};
use crate::pollution::PollutionGrid;
use crate::services::{ServiceBuilding, ServiceType};

/// Health coverage grid - higher = better healthcare access
#[derive(Resource)]
pub struct HealthGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for HealthGrid {
    fn default() -> Self {
        Self {
            levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl HealthGrid {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }
}

pub fn update_health_grid(
    slow_timer: Res<crate::SlowTickTimer>,
    mut health: ResMut<HealthGrid>,
    pollution: Res<PollutionGrid>,
    services: Query<&ServiceBuilding>,
) {
    if !slow_timer.should_run() { return; }
    // Base health level = inverse of pollution
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let poll = pollution.get(x, y) as i32;
            let base_health = (80 - poll).max(0) as u8;
            health.levels[y * GRID_WIDTH + x] = base_health;
        }
    }

    // Health services boost health in radius
    for service in &services {
        if !ServiceBuilding::is_health(service.service_type) {
            continue;
        }
        let radius = (service.radius / 16.0) as i32;
        let boost = match service.service_type {
            ServiceType::MedicalClinic => 20u8,
            ServiceType::Hospital => 40u8,
            ServiceType::MedicalCenter => 60u8,
            _ => 30u8,
        };
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = service.grid_x as i32 + dx;
                let ny = service.grid_y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                    let dist = dx.abs() + dy.abs();
                    let effect = boost.saturating_sub((dist * 2) as u8);
                    let idx = ny as usize * GRID_WIDTH + nx as usize;
                    health.levels[idx] = health.levels[idx].saturating_add(effect).min(100);
                }
            }
        }
    }
}

/// City-wide sickness tracking
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct SicknessStats {
    pub sick_citizens: u32,
    pub sickness_rate: f32, // 0.0 to 1.0
}
