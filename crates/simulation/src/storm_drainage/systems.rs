//! Storm drainage simulation systems.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::stormwater::StormwaterGrid;
use crate::SlowTickTimer;

use super::types::{
    StormDrainageInfrastructure, StormDrainageState, StormDrainageType, DRAIN_CAPACITY_PER_DRAIN,
    FLOOD_THRESHOLD, RAIN_GARDEN_NEIGHBOR_ABSORB, RETENTION_POND_CAPACITY, RETENTION_RELEASE_RATE,
    RUNOFF_TO_GALLONS, RUNOFF_TO_INCHES_PER_HR,
};

/// Updates storm drainage infrastructure state each slow tick.
///
/// 1. Counts storm drains, retention ponds, and rain gardens from infrastructure queries.
/// 2. Computes total drain capacity and retention capacity.
/// 3. Reads the StormwaterGrid to determine per-cell runoff.
/// 4. Rain gardens absorb local + neighbor runoff.
/// 5. Fills retention ponds with excess runoff beyond drain capacity.
/// 6. Slowly releases retention pond stored water.
/// 7. Tracks overflow cells where runoff exceeds all drainage.
/// 8. Computes drainage coverage as fraction of road cells with nearby drains.
pub fn update_storm_drainage(
    slow_timer: Res<SlowTickTimer>,
    mut drainage_state: ResMut<StormDrainageState>,
    stormwater: Res<StormwaterGrid>,
    grid: Res<WorldGrid>,
    infrastructure: Query<&StormDrainageInfrastructure>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Count infrastructure and compute capacities ---
    let mut drain_count: u32 = 0;
    let mut retention_pond_count: u32 = 0;
    let mut rain_garden_count: u32 = 0;

    // Track drain positions for coverage calculation
    let mut drain_positions: Vec<(usize, usize)> = Vec::new();
    // Track rain garden positions for runoff absorption
    let mut rain_garden_positions: Vec<(usize, usize)> = Vec::new();

    for infra in &infrastructure {
        match infra.drainage_type {
            StormDrainageType::StormDrain => {
                drain_count += 1;
                drain_positions.push((infra.grid_x, infra.grid_y));
            }
            StormDrainageType::RetentionPond => {
                retention_pond_count += 1;
            }
            StormDrainageType::RainGarden => {
                rain_garden_count += 1;
                rain_garden_positions.push((infra.grid_x, infra.grid_y));
            }
        }
    }

    let total_drain_capacity = drain_count as f32 * DRAIN_CAPACITY_PER_DRAIN;
    let total_retention_capacity = retention_pond_count as f32 * RETENTION_POND_CAPACITY;

    // --- Phase 2: Compute effective runoff per cell (after rain garden absorption) ---
    let total_cells = GRID_WIDTH * GRID_HEIGHT;
    let mut effective_runoff = vec![0.0_f32; total_cells];

    // Copy raw runoff from the stormwater grid
    effective_runoff[..total_cells].copy_from_slice(&stormwater.runoff[..total_cells]);

    // Rain gardens absorb 100% of their own cell + 50% from 4 cardinal neighbors
    for &(gx, gy) in &rain_garden_positions {
        if gx < GRID_WIDTH && gy < GRID_HEIGHT {
            // Absorb 100% of local cell runoff
            let idx = gy * GRID_WIDTH + gx;
            effective_runoff[idx] = 0.0;

            // Absorb 50% from each cardinal neighbor
            let (neighbors, count) = grid.neighbors4(gx, gy);
            for &(nx, ny) in &neighbors[..count] {
                let nidx = ny * GRID_WIDTH + nx;
                effective_runoff[nidx] *= 1.0 - RAIN_GARDEN_NEIGHBOR_ABSORB;
            }
        }
    }

    // --- Phase 3: Compute total effective runoff in inches/hr ---
    let mut total_effective_runoff_in_hr = 0.0_f32;
    for val in &effective_runoff {
        total_effective_runoff_in_hr += val * RUNOFF_TO_INCHES_PER_HR;
    }

    // --- Phase 4: Determine excess runoff beyond drain capacity ---
    let excess_runoff_in_hr = (total_effective_runoff_in_hr - total_drain_capacity).max(0.0);

    // Convert excess to gallons for retention storage
    let excess_gallons = excess_runoff_in_hr * RUNOFF_TO_GALLONS;

    // --- Phase 5: Fill retention ponds ---
    let mut current_stored = drainage_state.current_retention_stored;
    let available_storage = (total_retention_capacity - current_stored).max(0.0);
    let stored_this_tick = excess_gallons.min(available_storage);
    current_stored += stored_this_tick;

    // --- Phase 6: Slowly release stored water ---
    let released = current_stored * RETENTION_RELEASE_RATE;
    current_stored = (current_stored - released).max(0.0);

    // --- Phase 7: Count overflow cells ---
    // A cell overflows when its effective runoff exceeds the per-cell drain
    // capacity share AND retention ponds are full (cannot absorb more).
    let per_cell_drain_capacity = if drain_count > 0 {
        total_drain_capacity / total_cells as f32
    } else {
        0.0
    };

    let retention_full = current_stored >= total_retention_capacity * 0.99;

    let mut overflow_cells: u32 = 0;
    for val in &effective_runoff {
        let cell_runoff_in_hr = val * RUNOFF_TO_INCHES_PER_HR;
        if cell_runoff_in_hr > per_cell_drain_capacity + FLOOD_THRESHOLD && retention_full {
            overflow_cells += 1;
        }
    }

    // --- Phase 8: Compute drainage coverage ---
    // Fraction of road cells that have a storm drain on them or adjacent (Manhattan dist <= 1)
    let mut road_cell_count: u32 = 0;
    let mut covered_road_cells: u32 = 0;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type == CellType::Road {
                road_cell_count += 1;

                // Check if any drain is at this cell or adjacent
                let is_covered = drain_positions.iter().any(|&(dx, dy)| {
                    let dist_x = (x as i32 - dx as i32).unsigned_abs() as usize;
                    let dist_y = (y as i32 - dy as i32).unsigned_abs() as usize;
                    dist_x + dist_y <= 1
                });

                if is_covered {
                    covered_road_cells += 1;
                }
            }
        }
    }

    let drainage_coverage = if road_cell_count > 0 {
        covered_road_cells as f32 / road_cell_count as f32
    } else {
        0.0
    };

    // --- Update state ---
    drainage_state.drain_count = drain_count;
    drainage_state.retention_pond_count = retention_pond_count;
    drainage_state.rain_garden_count = rain_garden_count;
    drainage_state.total_drain_capacity = total_drain_capacity;
    drainage_state.total_retention_capacity = total_retention_capacity;
    drainage_state.current_retention_stored = current_stored;
    drainage_state.overflow_cells = overflow_cells;
    drainage_state.drainage_coverage = drainage_coverage;
}
