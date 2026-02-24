//! Power Line Transmission and Service Radius (POWER-011)
//!
//! Power lines follow roads, connecting generators to consumers. Buildings
//! must be within `POWER_RANGE` (6 cells) of a power line to receive service.
//! Transmission losses of 2% per 10 cells from the generator reduce effective
//! power delivery. Buildings without power cannot function and incur a
//! happiness penalty.
//!
//! The system performs BFS from each generator along road cells carrying power
//! lines, then flood-fills service radius from every powered road cell. The
//! `has_power` flag on `Cell` is updated accordingly.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::coal_power::PowerPlant;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::{decode_or_warn, Saveable, SaveableRegistry, SimulationSet, TickCounter};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Service radius: buildings within this many cells of a power line receive power.
pub const POWER_RANGE: usize = 6;

/// Transmission loss: fraction of power lost per 10 cells of distance from generator.
const LOSS_PER_10_CELLS: f32 = 0.02;

/// How often (in ticks) the power line propagation system runs.
const PROPAGATION_INTERVAL: u64 = 8;

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Tracks which cells have power lines installed and per-cell transmission efficiency.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PowerLineGrid {
    /// True if a power line exists on this cell (indexed as `y * width + x`).
    pub has_line: Vec<bool>,
    /// Transmission efficiency at each cell (1.0 = no loss, 0.0 = total loss).
    /// Only meaningful where `has_line` is true.
    pub efficiency: Vec<f32>,
    /// Grid width (for save/load validation).
    pub width: usize,
    /// Grid height (for save/load validation).
    pub height: usize,
    /// Total cells currently powered via power lines.
    pub powered_cell_count: u32,
    /// Total cells with power lines installed.
    pub line_cell_count: u32,
}

impl Default for PowerLineGrid {
    fn default() -> Self {
        let size = GRID_WIDTH * GRID_HEIGHT;
        Self {
            has_line: vec![false; size],
            efficiency: vec![0.0; size],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
            powered_cell_count: 0,
            line_cell_count: 0,
        }
    }
}

impl Saveable for PowerLineGrid {
    const SAVE_KEY: &'static str = "power_lines";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Auto-install power lines on road cells near generators
// ---------------------------------------------------------------------------

/// Installs power lines on all road cells reachable from any generator via
/// BFS through the road network. Power lines auto-follow roads.
///
/// Also computes per-cell transmission efficiency based on distance from the
/// nearest generator.
pub fn install_power_lines(
    tick: Res<TickCounter>,
    grid: Res<WorldGrid>,
    mut power_grid: ResMut<PowerLineGrid>,
    plants: Query<&PowerPlant>,
) {
    if !tick.0.is_multiple_of(PROPAGATION_INTERVAL) {
        return;
    }

    let w = grid.width;
    let h = grid.height;
    let total = w * h;

    // Reset power line state.
    for v in power_grid.has_line.iter_mut() {
        *v = false;
    }
    for v in power_grid.efficiency.iter_mut() {
        *v = 0.0;
    }

    // BFS from each generator through road cells.
    let mut queue: VecDeque<(usize, usize, u32)> = VecDeque::new();
    let mut best_dist = vec![u32::MAX; total];

    for plant in &plants {
        let sx = plant.grid_x;
        let sy = plant.grid_y;
        if sx >= w || sy >= h {
            continue;
        }
        let idx = sy * w + sx;
        if best_dist[idx] == 0 {
            continue; // Already seeded by another generator at same cell.
        }
        best_dist[idx] = 0;
        queue.push_back((sx, sy, 0));
    }

    while let Some((x, y, dist)) = queue.pop_front() {
        let idx = y * w + x;

        // Mark as power line cell.
        power_grid.has_line[idx] = true;

        // Compute efficiency from distance.
        let eff = efficiency_at_distance(dist);
        if eff > power_grid.efficiency[idx] {
            power_grid.efficiency[idx] = eff;
        }

        // Expand to 4-connected road neighbors.
        let (neighbors, ncount) = grid.neighbors4(x, y);
        for &(nx, ny) in &neighbors[..ncount] {
            let nidx = ny * w + nx;
            let cell = grid.get(nx, ny);
            if cell.cell_type != CellType::Road {
                continue;
            }
            let new_dist = dist + 1;
            if new_dist < best_dist[nidx] {
                best_dist[nidx] = new_dist;
                queue.push_back((nx, ny, new_dist));
            }
        }
    }

    // Count line cells.
    power_grid.line_cell_count = power_grid.has_line.iter().filter(|&&v| v).count() as u32;
}

/// Computes the transmission efficiency at a given distance from a generator.
/// Loss is 2% per 10 cells, so efficiency = 1.0 - (distance / 10) * 0.02.
/// Clamped to [0.0, 1.0].
fn efficiency_at_distance(distance: u32) -> f32 {
    let loss = (distance as f32 / 10.0) * LOSS_PER_10_CELLS;
    (1.0 - loss).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Propagate has_power to buildings within service radius
// ---------------------------------------------------------------------------

/// Sets `has_power` on grid cells that are within `POWER_RANGE` of a power
/// line cell. Uses a distance-limited BFS from all power line cells.
pub fn propagate_power_coverage(
    tick: Res<TickCounter>,
    mut grid: ResMut<WorldGrid>,
    mut power_grid: ResMut<PowerLineGrid>,
) {
    if !tick.0.is_multiple_of(PROPAGATION_INTERVAL) {
        return;
    }

    let w = grid.width;
    let h = grid.height;
    let total = w * h;

    // Reset all has_power flags.
    for cell in grid.cells.iter_mut() {
        cell.has_power = false;
    }

    // Multi-source BFS from all power line cells, expanding up to POWER_RANGE.
    let mut visited = vec![false; total];
    let mut queue: VecDeque<(usize, usize, usize)> = VecDeque::new();

    // Seed: all cells with power lines.
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            if power_grid.has_line[idx] {
                visited[idx] = true;
                grid.get_mut(x, y).has_power = true;
                queue.push_back((x, y, 0));
            }
        }
    }

    // BFS expansion through non-water cells up to POWER_RANGE.
    while let Some((x, y, dist)) = queue.pop_front() {
        if dist >= POWER_RANGE {
            continue;
        }

        let (neighbors, ncount) = grid.neighbors4(x, y);
        for &(nx, ny) in &neighbors[..ncount] {
            let nidx = ny * w + nx;
            if visited[nidx] {
                continue;
            }
            let cell = grid.get(nx, ny);
            if cell.cell_type == CellType::Water {
                continue;
            }
            visited[nidx] = true;
            grid.get_mut(nx, ny).has_power = true;
            queue.push_back((nx, ny, dist + 1));
        }
    }

    // Count powered cells.
    power_grid.powered_cell_count = grid.cells.iter().filter(|c| c.has_power).count() as u32;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PowerLinePlugin;

impl Plugin for PowerLinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PowerLineGrid>();

        // Register for save/load.
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(SaveableRegistry::default);
        registry.register::<PowerLineGrid>();

        app.add_systems(
            FixedUpdate,
            (install_power_lines, propagate_power_coverage)
                .chain()
                .in_set(SimulationSet::Simulation),
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
    fn test_efficiency_at_distance() {
        // At the generator: 100% efficiency.
        assert!((efficiency_at_distance(0) - 1.0).abs() < f32::EPSILON);
        // At 10 cells: 98% efficiency.
        assert!((efficiency_at_distance(10) - 0.98).abs() < 0.001);
        // At 50 cells: 90% efficiency.
        assert!((efficiency_at_distance(50) - 0.90).abs() < 0.001);
        // At 500 cells: 0% efficiency (clamped).
        assert!((efficiency_at_distance(500)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_default_power_line_grid() {
        let plg = PowerLineGrid::default();
        assert_eq!(plg.width, GRID_WIDTH);
        assert_eq!(plg.height, GRID_HEIGHT);
        assert_eq!(plg.has_line.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(plg.powered_cell_count, 0);
        assert_eq!(plg.line_cell_count, 0);
        assert!(plg.has_line.iter().all(|&v| !v));
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut plg = PowerLineGrid::default();
        plg.has_line[100] = true;
        plg.efficiency[100] = 0.95;
        plg.line_cell_count = 1;
        plg.powered_cell_count = 7;

        let bytes = plg.save_to_bytes().unwrap();
        let restored = PowerLineGrid::load_from_bytes(&bytes);

        assert!(restored.has_line[100]);
        assert!((restored.efficiency[100] - 0.95).abs() < f32::EPSILON);
        assert_eq!(restored.line_cell_count, 1);
        assert_eq!(restored.powered_cell_count, 7);
    }
}
