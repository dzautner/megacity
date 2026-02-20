use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::services::ServiceBuilding;
use crate::wind::WindState;
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

pub fn update_pollution(
    slow_timer: Res<crate::SlowTickTimer>,
    mut pollution: ResMut<PollutionGrid>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
    policies: Res<crate::policies::Policies>,
    wind: Res<WindState>,
) {
    if !slow_timer.should_run() {
        return;
    }
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
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                    {
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
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                    {
                        let dist = dx.abs() + dy.abs();
                        let effect = reduction.saturating_sub(dist as u8);
                        let cur = pollution.get(nx as usize, ny as usize);
                        pollution.set(nx as usize, ny as usize, cur.saturating_sub(effect));
                    }
                }
            }
        }
    }

    // Apply wind drift: shift pollution in the wind direction
    apply_wind_drift(&mut pollution, &wind);
}

/// Shifts pollution in the wind direction using a temporary buffer.
///
/// For each cell with pollution, a fraction (proportional to wind speed) is
/// transferred to the downwind neighbor cell. This causes industrial pollution
/// to drift downwind, making upwind residential zones cleaner.
fn apply_wind_drift(pollution: &mut PollutionGrid, wind: &WindState) {
    if wind.speed < 0.05 {
        // Wind is negligible; skip drift
        return;
    }

    let (dx, dy) = wind.direction_vector();
    // drift_fraction: how much pollution moves downwind (0..~0.45)
    let drift_fraction = wind.speed * 0.45;

    // We use a temporary f32 buffer to accumulate drifted pollution, then write back.
    // This avoids order-dependent artifacts from in-place mutation.
    let total = GRID_WIDTH * GRID_HEIGHT;
    let mut drift_buf: Vec<f32> = vec![0.0; total];

    // Copy current pollution into the drift buffer as the base
    for (i, buf) in drift_buf.iter_mut().enumerate().take(total) {
        *buf = pollution.levels[i] as f32;
    }

    // For each cell, move `drift_fraction` of its pollution toward the downwind cell(s).
    // We use bilinear distribution to the 2 or 4 neighboring cells based on the
    // fractional wind direction components.
    let abs_dx = dx.abs();
    let abs_dy = dy.abs();
    // Normalize so the largest component is 1.0 (preserves direction proportions)
    let max_comp = abs_dx.max(abs_dy).max(0.001);
    let norm_dx = dx / max_comp;
    let norm_dy = dy / max_comp;

    // Step offsets: the primary downwind cell and diagonal
    let step_x: i32 = if norm_dx > 0.0 {
        1
    } else if norm_dx < 0.0 {
        -1
    } else {
        0
    };
    let step_y: i32 = if norm_dy > 0.0 {
        1
    } else if norm_dy < 0.0 {
        -1
    } else {
        0
    };

    // Weights for distributing drift between x-neighbor, y-neighbor, and diagonal
    let wx = abs_dx / (abs_dx + abs_dy + 0.001);
    let wy = 1.0 - wx;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let val = pollution.levels[idx] as f32;
            if val < 1.0 {
                continue;
            }

            let moved = val * drift_fraction;

            // Subtract from source
            drift_buf[idx] -= moved;

            // Distribute to downwind neighbors
            let nx_x = x as i32 + step_x;
            let nx_y = y as i32 + step_y;

            // X-neighbor (horizontal drift component)
            if step_x != 0 {
                let tx = nx_x;
                let ty = y as i32;
                if tx >= 0 && (tx as usize) < GRID_WIDTH && ty >= 0 && (ty as usize) < GRID_HEIGHT {
                    drift_buf[ty as usize * GRID_WIDTH + tx as usize] += moved * wx;
                }
                // If x-neighbor is out of bounds, pollution dissipates (leaves the map)
            }

            // Y-neighbor (vertical drift component)
            if step_y != 0 {
                let tx = x as i32;
                let ty = nx_y;
                if tx >= 0 && (tx as usize) < GRID_WIDTH && ty >= 0 && (ty as usize) < GRID_HEIGHT {
                    drift_buf[ty as usize * GRID_WIDTH + tx as usize] += moved * wy;
                }
            }

            // If wind is purely horizontal or purely vertical, one of the above
            // handles all drift. If diagonal, both contribute proportionally.
            // Edge case: if both step_x and step_y are 0 (should not happen given
            // speed > 0.05 check), moved pollution is simply lost.
        }
    }

    // Write the drift buffer back to the pollution grid, clamping to u8 range
    for (level, buf) in pollution
        .levels
        .iter_mut()
        .zip(drift_buf.iter())
        .take(total)
    {
        *level = buf.clamp(0.0, 255.0) as u8;
    }
}

pub struct PollutionPlugin;

impl Plugin for PollutionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PollutionGrid>().add_systems(
            FixedUpdate,
            update_pollution.after(crate::education::propagate_education),
        );
    }
}
