//! Flood simulation system and plugin.
//!
//! The `update_flood_simulation` system runs every slow tick and performs:
//!   1. Checks if flooding conditions exist (storm drainage overflow > threshold)
//!   2. Initializes the FloodGrid from stormwater overflow
//!   3. Runs 5 iterations of water spreading (high elevation to low, 4-connected)
//!   4. Applies drainage rates (natural drain + enhanced drain for cells with drains)
//!   5. Calculates building damage using depth-damage curves
//!   6. Updates FloodState with aggregate statistics
//!   7. Clears FloodGrid when flooding subsides

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::WorldGrid;
use crate::storm_drainage::StormDrainageState;
use crate::stormwater::StormwaterGrid;
use crate::SlowTickTimer;

use super::damage_curves::{
    depth_damage_fraction, BASE_PROPERTY_VALUE_PER_CAPACITY, FLOOD_DEPTH_THRESHOLD,
    NATURAL_DRAIN_RATE, OVERFLOW_TRIGGER_THRESHOLD, RUNOFF_TO_FEET, SPREAD_ITERATIONS, SPREAD_RATE,
    STORM_DRAIN_RATE,
};
use super::resources::{FloodGrid, FloodState};

// =============================================================================
// System
// =============================================================================

/// Main flood simulation system. Runs every slow tick.
///
/// When storm drainage overflow exceeds the trigger threshold, the system
/// initialises flood depths from excess stormwater runoff, spreads water over
/// the terrain for 5 iterations, applies natural and infrastructure-assisted
/// drainage, calculates building damage via depth-damage curves, and updates
/// aggregate flood statistics.
#[allow(clippy::too_many_arguments)]
pub fn update_flood_simulation(
    slow_timer: Res<SlowTickTimer>,
    mut flood_grid: ResMut<FloodGrid>,
    mut flood_state: ResMut<FloodState>,
    world_grid: Res<WorldGrid>,
    stormwater: Res<StormwaterGrid>,
    drainage_state: Res<StormDrainageState>,
    buildings: Query<&crate::buildings::Building>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Step 1: Check if flooding conditions exist ---
    let flooding_triggered = drainage_state.overflow_cells > OVERFLOW_TRIGGER_THRESHOLD;

    if !flooding_triggered && !flood_grid.has_flooding() {
        // No new flooding and no residual water: ensure state is clean
        if flood_state.is_flooding {
            flood_state.is_flooding = false;
            flood_state.total_flooded_cells = 0;
            flood_state.total_damage = 0.0;
            flood_state.max_depth = 0.0;
            flood_grid.clear();
        }
        return;
    }

    // --- Step 2: If newly flooding, seed FloodGrid from stormwater overflow ---
    if flooding_triggered {
        // Excess runoff that the drainage system could not handle becomes flood water.
        // We only add NEW water each tick, not replace existing depths.
        let drain_cap = drainage_state.total_drain_capacity;
        let total_cells = GRID_WIDTH * GRID_HEIGHT;
        let per_cell_drain = if drain_cap > 0.0 {
            drain_cap / total_cells as f32
        } else {
            0.0
        };

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let runoff = stormwater.get(x, y);
                // Convert runoff to depth in feet; subtract the drainage capacity share
                let excess = (runoff * RUNOFF_TO_FEET - per_cell_drain).max(0.0);
                if excess > 0.0 {
                    let idx = flood_grid.index(x, y);
                    flood_grid.cells[idx] += excess;
                }
            }
        }
    }

    // --- Step 3: Run 5 iterations of water spreading ---
    for _ in 0..SPREAD_ITERATIONS {
        // Snapshot current depths to avoid order-dependent artifacts
        let snapshot: Vec<f32> = flood_grid.cells.clone();

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let idx = y * flood_grid.width + x;
                let current_depth = snapshot[idx];
                if current_depth <= 0.0 {
                    continue;
                }

                let current_elevation = world_grid.get(x, y).elevation;
                let current_surface = current_elevation + current_depth;

                // Find lower-surface neighbors (4-connected)
                let (neighbors, count) = world_grid.neighbors4(x, y);
                let mut lower: [(usize, usize, f32); 4] = [(0, 0, 0.0); 4];
                let mut lower_count = 0usize;
                let mut total_diff = 0.0_f32;

                for &(nx, ny) in &neighbors[..count] {
                    let n_idx = ny * flood_grid.width + nx;
                    let n_elevation = world_grid.get(nx, ny).elevation;
                    let n_surface = n_elevation + snapshot[n_idx];

                    if n_surface < current_surface {
                        let diff = current_surface - n_surface;
                        lower[lower_count] = (nx, ny, diff);
                        lower_count += 1;
                        total_diff += diff;
                    }
                }

                if lower_count == 0 || total_diff <= 0.0 {
                    continue;
                }

                // Distribute water proportionally to surface height difference
                let transferable = current_depth * SPREAD_RATE;
                flood_grid.cells[idx] -= transferable;

                for &(nx, ny, diff) in &lower[..lower_count] {
                    let fraction = diff / total_diff;
                    let transfer = transferable * fraction;
                    let n_idx = ny * flood_grid.width + nx;
                    flood_grid.cells[n_idx] += transfer;
                }
            }
        }
    }

    // --- Step 4: Apply drainage rates ---
    // Build a quick lookup of which cells have storm drain infrastructure.
    // We approximate this using drainage_coverage: if coverage > 0 and the cell
    // is a road cell or adjacent to a road, it gets enhanced drainage.
    let has_drain_infrastructure = drainage_state.drain_count > 0;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = flood_grid.index(x, y);
            if flood_grid.cells[idx] <= 0.0 {
                continue;
            }

            // Natural drainage applies to all cells
            let mut drain = NATURAL_DRAIN_RATE;

            // Enhanced drainage for cells with road (storm drains follow roads)
            if has_drain_infrastructure
                && world_grid.get(x, y).cell_type == crate::grid::CellType::Road
            {
                drain += STORM_DRAIN_RATE;
            }

            flood_grid.cells[idx] = (flood_grid.cells[idx] - drain).max(0.0);
        }
    }

    // --- Step 5: Calculate damage for buildings in flooded cells ---
    let mut total_damage = 0.0_f64;

    for building in &buildings {
        let bx = building.grid_x;
        let by = building.grid_y;
        if bx >= GRID_WIDTH || by >= GRID_HEIGHT {
            continue;
        }

        let depth = flood_grid.get(bx, by);
        if depth < FLOOD_DEPTH_THRESHOLD {
            continue;
        }

        let damage_fraction = depth_damage_fraction(depth, building.zone_type);
        let building_value =
            building.capacity as f64 * building.level as f64 * BASE_PROPERTY_VALUE_PER_CAPACITY;
        total_damage += building_value * damage_fraction as f64;
    }

    // --- Step 6: Update FloodState with stats ---
    let mut flooded_cells: u32 = 0;
    let mut max_depth: f32 = 0.0;

    for &depth in &flood_grid.cells {
        if depth >= FLOOD_DEPTH_THRESHOLD {
            flooded_cells += 1;
        }
        if depth > max_depth {
            max_depth = depth;
        }
    }

    flood_state.is_flooding = flooded_cells > 0;
    flood_state.total_flooded_cells = flooded_cells;
    flood_state.total_damage = total_damage;
    flood_state.max_depth = max_depth;

    // --- Step 7: If no more flooding, clear FloodGrid ---
    if !flood_state.is_flooding {
        flood_grid.clear();
        flood_state.total_damage = 0.0;
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct FloodSimulationPlugin;

impl Plugin for FloodSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FloodGrid>()
            .init_resource::<FloodState>()
            .add_systems(
                FixedUpdate,
                update_flood_simulation
                    .after(crate::storm_drainage::update_storm_drainage)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
