use bevy::prelude::*;

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails, HomeLocation};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};

use crate::utilities::UtilitySource;

/// Grid tracking contamination level (0-255) of water cells and cells near water.
#[derive(Resource)]
pub struct WaterPollutionGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for WaterPollutionGrid {
    fn default() -> Self {
        Self {
            levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl WaterPollutionGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }

    fn add(&mut self, x: usize, y: usize, amount: u8) {
        let idx = y * self.width + x;
        self.levels[idx] = self.levels[idx].saturating_add(amount);
    }

    fn sub(&mut self, x: usize, y: usize, amount: u8) {
        let idx = y * self.width + x;
        self.levels[idx] = self.levels[idx].saturating_sub(amount);
    }
}

/// Update water pollution every 100 ticks (via SlowTickTimer).
///
/// Sources:
///   - Industrial buildings near water cells generate contamination (radius 3).
///   - Pollution diffuses between adjacent water cells.
///   - Water treatment plants (UtilityType::WaterTreatment) reduce pollution.
pub fn update_water_pollution(
    slow_timer: Res<crate::SlowTickTimer>,
    mut water_pollution: ResMut<WaterPollutionGrid>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    utilities: Query<&UtilitySource>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Decay existing pollution slightly (natural remediation) ---
    for val in water_pollution.levels.iter_mut() {
        *val = val.saturating_sub(1);
    }

    // --- Phase 2: Industrial buildings near water generate pollution ---
    for building in &buildings {
        if building.zone_type != ZoneType::Industrial {
            continue;
        }

        // Intensity scales with building level (10 base + 7 per level, range 17-45)
        let intensity = 10i32 + building.level as i32 * 7;
        let radius = 3i32;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = building.grid_x as i32 + dx;
                let ny = building.grid_y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;

                // Only pollute water cells
                if grid.get(ux, uy).cell_type != CellType::Water {
                    continue;
                }

                let dist = dx.abs() + dy.abs();
                let decay = (intensity - dist * 3).max(0) as u8;
                if decay > 0 {
                    water_pollution.add(ux, uy, decay);
                }
            }
        }
    }

    // --- Phase 3: Diffusion -- polluted water spreads to neighboring water cells ---
    // Snapshot current levels, then spread
    let snapshot: Vec<u8> = water_pollution.levels.clone();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type != CellType::Water {
                continue;
            }
            let current = snapshot[y * GRID_WIDTH + x];
            if current < 4 {
                continue;
            }

            // Spread 1/8 of the pollution to each cardinal neighbor that is water
            let spread_amount = current / 8;
            if spread_amount == 0 {
                continue;
            }

            let neighbors: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
            for (dx, dy) in neighbors {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;
                if grid.get(ux, uy).cell_type == CellType::Water {
                    let neighbor_level = snapshot[uy * GRID_WIDTH + ux];
                    // Only spread downhill (from higher to lower pollution)
                    if current > neighbor_level {
                        water_pollution.add(ux, uy, spread_amount);
                    }
                }
            }
        }
    }

    // --- Phase 4: Water treatment plants reduce pollution ---
    for utility in &utilities {
        if utility.utility_type != crate::utilities::UtilityType::WaterTreatment {
            continue;
        }
        let radius = 8i32;
        let reduction = 15u8;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = utility.grid_x as i32 + dx;
                let ny = utility.grid_y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;

                let dist = dx.abs() + dy.abs();
                let effect = reduction.saturating_sub(dist as u8);
                if effect > 0 {
                    water_pollution.sub(ux, uy, effect);
                }
            }
        }
    }
}

/// Citizens living near heavily polluted water (contamination > 50) suffer health penalties.
/// Runs on the slow tick alongside water pollution updates.
pub fn water_pollution_health_penalty(
    slow_timer: Res<crate::SlowTickTimer>,
    water_pollution: Res<WaterPollutionGrid>,
    grid: Res<WorldGrid>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation), With<Citizen>>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for (mut details, home) in &mut citizens {
        // Check pollution in a 2-cell radius around the citizen's home
        let check_radius = 2i32;
        let mut max_pollution: u8 = 0;

        for dy in -check_radius..=check_radius {
            for dx in -check_radius..=check_radius {
                let nx = home.grid_x as i32 + dx;
                let ny = home.grid_y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;
                if grid.get(ux, uy).cell_type == CellType::Water {
                    max_pollution = max_pollution.max(water_pollution.get(ux, uy));
                }
            }
        }

        // Threshold of 50: above this, health penalty proportional to excess
        if max_pollution > 50 {
            let excess = (max_pollution - 50) as f32;
            // Up to ~2.0 health loss per slow tick at max pollution (255)
            let penalty = excess * 0.01;
            details.health = (details.health - penalty).max(0.0);
        }
    }
}

pub struct WaterPollutionPlugin;

impl Plugin for WaterPollutionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterPollutionGrid>().add_systems(
            FixedUpdate,
            (update_water_pollution, water_pollution_health_penalty)
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
