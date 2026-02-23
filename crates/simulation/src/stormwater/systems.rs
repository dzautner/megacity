use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::weather::Weather;
use crate::SlowTickTimer;

use super::calculations::{imperviousness, infiltration, rainfall_intensity, runoff, DRAIN_RATE};
use super::types::StormwaterGrid;

/// Stormwater update system. Only runs during rain/storm weather events.
///
/// Each tick during precipitation:
/// 1. Calculate per-cell runoff based on imperviousness and rainfall intensity
/// 2. Accumulate runoff in the stormwater grid
/// 3. Drain accumulated runoff to downstream cells (based on elevation)
/// 4. Water cells act as sinks (runoff drains into them and disappears)
pub fn update_stormwater(
    slow_timer: Res<SlowTickTimer>,
    mut stormwater: ResMut<StormwaterGrid>,
    grid: Res<WorldGrid>,
    weather: Res<Weather>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let rain = rainfall_intensity(&weather);

    // If no precipitation, just drain existing runoff
    if rain <= 0.0 {
        // Drain all accumulated runoff gradually
        let mut any_runoff = false;
        for val in stormwater.runoff.iter_mut() {
            if *val > 0.0 {
                *val *= 1.0 - DRAIN_RATE * 3.0; // faster drain when no rain
                if *val < 0.01 {
                    *val = 0.0;
                }
                any_runoff = true;
            }
        }
        if !any_runoff {
            stormwater.total_runoff = 0.0;
            stormwater.total_infiltration = 0.0;
        }
        return;
    }

    // --- Phase 1: Calculate per-cell runoff and accumulate ---
    let mut tick_runoff = 0.0_f32;
    let mut tick_infiltration = 0.0_f32;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);

            // Water cells are sinks
            if cell.cell_type == CellType::Water {
                stormwater.set(x, y, 0.0);
                continue;
            }

            let has_building = cell.building_id.is_some();
            let imperv = imperviousness(cell.cell_type, cell.zone, has_building);

            let cell_runoff = runoff(rain, imperv);
            let cell_infiltration = infiltration(rain, imperv);

            stormwater.add(x, y, cell_runoff);
            tick_runoff += cell_runoff;
            tick_infiltration += cell_infiltration;
        }
    }

    // --- Phase 2: Drain runoff to downstream neighbors (based on elevation) ---
    // Use a snapshot to avoid order-dependent artifacts
    let snapshot: Vec<f32> = stormwater.runoff.clone();

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * stormwater.width + x;
            let current_runoff = snapshot[idx];
            if current_runoff <= 0.0 {
                continue;
            }

            let current_elevation = grid.get(x, y).elevation;

            // Find lower-elevation neighbors
            let (neighbors, count) = grid.neighbors4(x, y);
            let mut lower_neighbors: [(usize, usize, f32); 4] = [(0, 0, 0.0); 4];
            let mut lower_count = 0usize;
            let mut total_drop = 0.0_f32;

            for &(nx, ny) in &neighbors[..count] {
                let n_elevation = grid.get(nx, ny).elevation;
                if n_elevation < current_elevation {
                    let drop = current_elevation - n_elevation;
                    lower_neighbors[lower_count] = (nx, ny, drop);
                    lower_count += 1;
                    total_drop += drop;
                }
            }

            if lower_count == 0 || total_drop <= 0.0 {
                // No downhill neighbors; water pools here
                continue;
            }

            // Distribute drain proportionally to elevation drop
            let drain_amount = current_runoff * DRAIN_RATE;
            stormwater.runoff[idx] -= drain_amount;

            for &(nx, ny, drop) in &lower_neighbors[..lower_count] {
                let fraction = drop / total_drop;
                let transfer = drain_amount * fraction;

                // Water cells absorb runoff (sink)
                if grid.get(nx, ny).cell_type == CellType::Water {
                    // Runoff absorbed by water body, don't add to grid
                    continue;
                }

                stormwater.add(nx, ny, transfer);
            }
        }
    }

    stormwater.total_runoff = tick_runoff;
    stormwater.total_infiltration = tick_infiltration;
}
