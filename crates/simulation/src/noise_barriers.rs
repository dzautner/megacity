//! POLL-011: Noise Barrier Attenuation
//!
//! Post-processes the `NoisePollutionGrid` to account for noise barriers:
//!
//! 1. **Building attenuation**: Buildings between a noise source and a receiver
//!    absorb and reflect sound. Each building cell along the line of sight
//!    between source and receiver reduces noise by a configurable factor.
//!
//! 2. **Terrain attenuation**: When terrain elevation is higher between source
//!    and receiver than the line-of-sight elevation, the terrain acts as a
//!    natural sound barrier, reducing noise further.
//!
//! The system runs after `update_noise_pollution` and applies attenuation as a
//! post-processing reduction on already-computed noise values.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::noise::NoisePollutionGrid;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Configuration constants
// ---------------------------------------------------------------------------

/// Noise reduction factor per building cell between source and receiver.
/// A value of 0.7 means each building reduces transmitted noise to 70%.
/// Two buildings in a row: 0.7 * 0.7 = 0.49 (49% of original).
pub const BUILDING_ATTENUATION_FACTOR: f32 = 0.7;

/// Noise reduction factor when terrain blocks line of sight.
/// Applied once if any terrain cell along the path is higher than both
/// source and receiver elevation interpolation.
pub const TERRAIN_ATTENUATION_FACTOR: f32 = 0.5;

/// Minimum noise level (0-100) for a cell to be considered a noise source
/// worth tracing barriers for. Cells below this threshold are already quiet
/// and do not benefit from barrier analysis.
const SOURCE_THRESHOLD: u8 = 15;

/// Maximum radius (in cells) to search for noise sources around each cell.
/// Kept small for performance; buildings beyond this distance have minimal
/// barrier effect because noise has already attenuated naturally.
const BARRIER_SEARCH_RADIUS: i32 = 12;

// ---------------------------------------------------------------------------
// Line-of-sight barrier counting
// ---------------------------------------------------------------------------

/// Counts the number of building cells and terrain obstructions along a
/// straight line from `(sx, sy)` to `(rx, ry)` using Bresenham's algorithm.
///
/// Returns `(building_count, terrain_blocks)` where:
/// - `building_count`: number of cells with a building between source and
///   receiver (excluding endpoints).
/// - `terrain_blocks`: true if any intermediate cell has elevation higher
///   than the interpolated line-of-sight elevation between source and receiver.
fn count_barriers(
    grid: &WorldGrid,
    sx: i32,
    sy: i32,
    rx: i32,
    ry: i32,
) -> (u32, bool) {
    let dx = (rx - sx).abs();
    let dy = (ry - sy).abs();

    // Skip adjacent cells (no room for barriers)
    if dx <= 1 && dy <= 1 {
        return (0, false);
    }

    let step_x: i32 = if sx < rx { 1 } else { -1 };
    let step_y: i32 = if sy < ry { 1 } else { -1 };

    let src_elev = grid.get(sx as usize, sy as usize).elevation;
    let rcv_elev = grid.get(rx as usize, ry as usize).elevation;

    let total_dist = ((dx * dx + dy * dy) as f32).sqrt();

    let mut building_count = 0u32;
    let mut terrain_blocked = false;

    // Bresenham's line algorithm
    let mut x = sx;
    let mut y = sy;
    let mut err = dx - dy;

    loop {
        // Move to next cell
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += step_x;
        }
        if e2 < dx {
            err += dx;
            y += step_y;
        }

        // Stop if we've reached the receiver
        if x == rx && y == ry {
            break;
        }

        // Bounds check
        if x < 0 || y < 0 || x >= GRID_WIDTH as i32 || y >= GRID_HEIGHT as i32 {
            break;
        }

        let cell = grid.get(x as usize, y as usize);

        // Check building barrier
        if cell.building_id.is_some() {
            building_count += 1;
        }

        // Check terrain barrier: interpolate expected elevation at this point
        let dist_from_src =
            (((x - sx) * (x - sx) + (y - sy) * (y - sy)) as f32).sqrt();
        let t = if total_dist > 0.0 {
            dist_from_src / total_dist
        } else {
            0.5
        };
        let expected_elev = src_elev + t * (rcv_elev - src_elev);

        // If terrain is significantly higher than the interpolated line of
        // sight, it blocks noise propagation.
        if cell.elevation > expected_elev + 0.05
            && cell.cell_type != CellType::Water
        {
            terrain_blocked = true;
        }
    }

    (building_count, terrain_blocked)
}

/// Compute the attenuation multiplier given barrier counts.
///
/// Each building cell multiplies by `BUILDING_ATTENUATION_FACTOR`.
/// If terrain blocks line-of-sight, an additional `TERRAIN_ATTENUATION_FACTOR`
/// is applied.
fn barrier_multiplier(building_count: u32, terrain_blocked: bool) -> f32 {
    let mut mult = BUILDING_ATTENUATION_FACTOR.powi(building_count as i32);
    if terrain_blocked {
        mult *= TERRAIN_ATTENUATION_FACTOR;
    }
    mult
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Post-processes the noise grid to apply barrier attenuation.
///
/// For each cell with noise above `SOURCE_THRESHOLD`, the system traces lines
/// to nearby cells and checks for intervening buildings or terrain. If barriers
/// exist, the receiver cell's noise is reduced.
///
/// The approach works by:
/// 1. Snapshot the current noise grid.
/// 2. For each cell with noise >= SOURCE_THRESHOLD (potential source), scan
///    surrounding cells within BARRIER_SEARCH_RADIUS.
/// 3. For each nearby cell, trace a line from the source to that cell and
///    count barriers.
/// 4. If barriers exist, compute a reduction and apply it to the receiver cell.
pub fn apply_noise_barriers(
    slow_timer: Res<SlowTickTimer>,
    mut noise: ResMut<NoisePollutionGrid>,
    grid: Res<WorldGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Snapshot the noise grid before modifications
    let snapshot = noise.levels.clone();

    // Collect reduction amounts per cell (accumulated from all nearby sources)
    let total_cells = GRID_WIDTH * GRID_HEIGHT;
    let mut reductions = vec![0.0f32; total_cells];

    // Find all noise source cells (cells with significant noise)
    for sy in 0..GRID_HEIGHT {
        for sx in 0..GRID_WIDTH {
            let src_noise = snapshot[sy * GRID_WIDTH + sx];
            if src_noise < SOURCE_THRESHOLD {
                continue;
            }

            // Check receiver cells around this source
            let r = BARRIER_SEARCH_RADIUS;
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let rx = sx as i32 + dx;
                    let ry = sy as i32 + dy;
                    if rx < 0
                        || ry < 0
                        || rx >= GRID_WIDTH as i32
                        || ry >= GRID_HEIGHT as i32
                    {
                        continue;
                    }

                    let rux = rx as usize;
                    let ruy = ry as usize;
                    let rcv_noise = snapshot[ruy * GRID_WIDTH + rux];
                    if rcv_noise == 0 {
                        continue;
                    }

                    let (buildings, terrain) =
                        count_barriers(&grid, sx as i32, sy as i32, rx, ry);

                    // Only apply if there are actual barriers
                    if buildings == 0 && !terrain {
                        continue;
                    }

                    let mult = barrier_multiplier(buildings, terrain);
                    // The portion of receiver noise attributable to this source
                    // is approximated as proportional to the source strength.
                    // We compute the reduction as: noise_contribution * (1 - mult)
                    let contribution_fraction = src_noise as f32
                        / (src_noise as f32 + rcv_noise as f32);
                    let noise_from_source = rcv_noise as f32 * contribution_fraction;
                    let reduction = noise_from_source * (1.0 - mult);

                    reductions[ruy * GRID_WIDTH + rux] += reduction;
                }
            }
        }
    }

    // Apply reductions (cap total reduction at 80% of original noise to avoid
    // artifacts where heavy overlapping barriers zero out noise entirely)
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let reduction = reductions[idx];
            if reduction > 0.0 {
                let current = noise.levels[idx] as f32;
                let max_reduction = current * 0.8;
                let clamped = reduction.min(max_reduction);
                noise.levels[idx] = (current - clamped).max(0.0) as u8;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct NoiseBarriersPlugin;

impl Plugin for NoiseBarriersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            apply_noise_barriers
                .after(crate::noise::update_noise_pollution)
                .before(crate::noise_effects::apply_noise_land_value_effects)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barrier_multiplier_no_barriers() {
        let mult = barrier_multiplier(0, false);
        assert!(
            (mult - 1.0).abs() < f32::EPSILON,
            "no barriers should give multiplier 1.0, got {}",
            mult
        );
    }

    #[test]
    fn test_barrier_multiplier_one_building() {
        let mult = barrier_multiplier(1, false);
        assert!(
            (mult - BUILDING_ATTENUATION_FACTOR).abs() < f32::EPSILON,
            "one building should give {}, got {}",
            BUILDING_ATTENUATION_FACTOR,
            mult
        );
    }

    #[test]
    fn test_barrier_multiplier_two_buildings() {
        let expected = BUILDING_ATTENUATION_FACTOR * BUILDING_ATTENUATION_FACTOR;
        let mult = barrier_multiplier(2, false);
        assert!(
            (mult - expected).abs() < 0.001,
            "two buildings should give {}, got {}",
            expected,
            mult
        );
    }

    #[test]
    fn test_barrier_multiplier_terrain_only() {
        let mult = barrier_multiplier(0, true);
        assert!(
            (mult - TERRAIN_ATTENUATION_FACTOR).abs() < f32::EPSILON,
            "terrain only should give {}, got {}",
            TERRAIN_ATTENUATION_FACTOR,
            mult
        );
    }

    #[test]
    fn test_barrier_multiplier_building_plus_terrain() {
        let expected = BUILDING_ATTENUATION_FACTOR * TERRAIN_ATTENUATION_FACTOR;
        let mult = barrier_multiplier(1, true);
        assert!(
            (mult - expected).abs() < 0.001,
            "building+terrain should give {}, got {}",
            expected,
            mult
        );
    }

    #[test]
    fn test_barrier_multiplier_many_buildings() {
        let mult = barrier_multiplier(5, false);
        let expected = BUILDING_ATTENUATION_FACTOR.powi(5);
        assert!(
            (mult - expected).abs() < 0.001,
            "5 buildings should give {}, got {}",
            expected,
            mult
        );
        // 0.7^5 = 0.168 -- significant reduction
        assert!(mult < 0.2, "5 buildings should reduce significantly");
    }

    #[test]
    fn test_count_barriers_adjacent_cells() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let (buildings, terrain) = count_barriers(&grid, 10, 10, 11, 10);
        assert_eq!(buildings, 0, "adjacent cells should have no barriers");
        assert!(!terrain, "flat terrain should not block");
    }

    #[test]
    fn test_count_barriers_no_obstacles() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let (buildings, terrain) = count_barriers(&grid, 10, 10, 15, 10);
        assert_eq!(buildings, 0);
        assert!(!terrain);
    }

    #[test]
    fn test_attenuation_factor_in_valid_range() {
        assert!(
            BUILDING_ATTENUATION_FACTOR > 0.0 && BUILDING_ATTENUATION_FACTOR < 1.0,
            "building attenuation factor should be between 0 and 1"
        );
        assert!(
            TERRAIN_ATTENUATION_FACTOR > 0.0 && TERRAIN_ATTENUATION_FACTOR < 1.0,
            "terrain attenuation factor should be between 0 and 1"
        );
    }
}
