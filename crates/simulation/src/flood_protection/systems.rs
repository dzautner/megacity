//! Flood protection systems: placement validation, maintenance, and protection updates.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::flood_simulation::{FloodGrid, FloodState};
use crate::grid::{CellType, WorldGrid};
use crate::time_of_day::GameClock;
use crate::SlowTickTimer;

use super::types::{
    FloodProtectionState, DAYS_PER_YEAR, DEGRADATION_RATE_PER_TICK,
    MAINTENANCE_COST_PER_CELL_PER_YEAR, OVERTOPPING_AMPLIFICATION, RECOVERY_RATE_PER_TICK,
};

// =============================================================================
// Pure helper functions
// =============================================================================

/// Check if a grid cell is adjacent to water (river or coast).
pub fn is_adjacent_to_water(world_grid: &WorldGrid, x: usize, y: usize) -> bool {
    let (neighbors, count) = world_grid.neighbors4(x, y);
    for &(nx, ny) in &neighbors[..count] {
        if world_grid.get(nx, ny).cell_type == CellType::Water {
            return true;
        }
    }
    false
}

/// Check if a cell is a valid placement location for a levee (adjacent to river/water, not water itself).
pub fn can_place_levee(world_grid: &WorldGrid, x: usize, y: usize) -> bool {
    if x >= GRID_WIDTH || y >= GRID_HEIGHT {
        return false;
    }
    let cell = world_grid.get(x, y);
    // Cannot place on water cells
    if cell.cell_type == CellType::Water {
        return false;
    }
    // Must be adjacent to water
    is_adjacent_to_water(world_grid, x, y)
}

/// Check if a cell is a valid placement location for a seawall (on coast edge).
/// A coastal cell is one that is adjacent to water AND on the grid edge.
pub fn can_place_seawall(world_grid: &WorldGrid, x: usize, y: usize) -> bool {
    if x >= GRID_WIDTH || y >= GRID_HEIGHT {
        return false;
    }
    let cell = world_grid.get(x, y);
    if cell.cell_type == CellType::Water {
        return false;
    }
    // Must be adjacent to water
    is_adjacent_to_water(world_grid, x, y)
}

/// Check if a cell is a valid placement location for a floodgate.
/// Floodgates can be placed on any cell adjacent to water.
pub fn can_place_floodgate(world_grid: &WorldGrid, x: usize, y: usize) -> bool {
    can_place_levee(world_grid, x, y)
}

/// Calculate the daily maintenance cost from the annual cost.
pub fn daily_maintenance_cost(annual_cost: f64) -> f64 {
    annual_cost / DAYS_PER_YEAR as f64
}

/// Determine if a protection structure should fail this tick.
/// Uses a deterministic check based on tick counter for reproducibility.
pub fn should_fail(failure_prob: f32, tick_hash: u32) -> bool {
    // Convert failure probability to a threshold out of 10000
    let threshold = (failure_prob * 10000.0) as u32;
    (tick_hash % 10000) < threshold
}

// =============================================================================
// System
// =============================================================================

/// Main flood protection update system. Runs every slow tick.
///
/// Manages aging, maintenance, condition degradation, overtopping checks,
/// and flood depth reduction for all protection infrastructure.
#[allow(clippy::too_many_arguments)]
pub fn update_flood_protection(
    timer: Res<SlowTickTimer>,
    mut protection: ResMut<FloodProtectionState>,
    mut flood_grid: ResMut<FloodGrid>,
    flood_state: Res<FloodState>,
    world_grid: Res<WorldGrid>,
    mut budget: ResMut<CityBudget>,
    clock: Res<GameClock>,
) {
    if !timer.should_run() {
        return;
    }

    if protection.structures.is_empty() {
        return;
    }

    let current_day = clock.day;

    // --- Step 1: Age all structures ---
    for structure in &mut protection.structures {
        structure.age_days = structure.age_days.saturating_add(1);
    }

    // --- Step 2: Calculate and apply maintenance costs ---
    let total_structures = protection.structures.len() as f64;
    let annual_cost = total_structures * MAINTENANCE_COST_PER_CELL_PER_YEAR;
    protection.annual_maintenance_cost = annual_cost;

    // Charge daily maintenance
    if current_day > protection.last_maintenance_day {
        let daily_cost = daily_maintenance_cost(annual_cost);
        if budget.treasury >= daily_cost {
            budget.treasury -= daily_cost;
            protection.maintenance_funded = true;
        } else {
            protection.maintenance_funded = false;
        }
        protection.last_maintenance_day = current_day;
    }

    // --- Step 3: Degrade or recover condition ---
    let funded = protection.maintenance_funded;
    for structure in &mut protection.structures {
        if structure.failed {
            continue;
        }
        if funded {
            // Slowly recover condition when maintained
            structure.condition = (structure.condition + RECOVERY_RATE_PER_TICK).min(1.0);
        } else {
            // Degrade when not maintained
            structure.condition = (structure.condition - DEGRADATION_RATE_PER_TICK).max(0.0);
        }
    }

    // --- Step 4: Check for overtopping and apply protection ---
    let mut overtopping_events = 0u32;
    let mut damage_prevented = 0.0_f64;

    // Use current_day as a simple hash for failure determination
    let tick_hash = current_day.wrapping_mul(2654435761);

    for i in 0..protection.structures.len() {
        let structure = &protection.structures[i];
        let x = structure.grid_x as usize;
        let y = structure.grid_y as usize;

        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            continue;
        }

        if structure.failed {
            continue;
        }

        let effective_height = structure.effective_height();
        if effective_height <= 0.0 {
            continue;
        }

        let flood_depth = flood_grid.get(x, y);

        if flood_depth <= 0.0 {
            continue;
        }

        // Check for overtopping
        if flood_depth > effective_height {
            // Overtopping! The structure fails catastrophically.
            overtopping_events += 1;

            // Amplify flood depth at this cell (water bursts through)
            let amplified = flood_depth * OVERTOPPING_AMPLIFICATION;
            flood_grid.set(x, y, amplified);

            // Mark as failed
            protection.structures[i].failed = true;
        } else {
            // Protection holds: reduce flood depth at this cell
            let reduction = flood_depth.min(effective_height);
            flood_grid.set(x, y, (flood_depth - reduction).max(0.0));

            // Estimate damage prevented (rough: reduction * $1000 per ft)
            damage_prevented += reduction as f64 * 1000.0;
        }

        // Check for age/condition-based spontaneous failure
        let failure_prob = protection.structures[i].failure_probability();
        let structure_hash = tick_hash.wrapping_add(i as u32 * 37);
        if should_fail(failure_prob, structure_hash) && flood_depth > 0.0 {
            protection.structures[i].failed = true;
            protection.structures[i].condition = 0.0;
        }
    }

    protection.overtopping_events = overtopping_events;
    protection.damage_prevented = damage_prevented;

    // --- Step 5: Also protect neighboring cells behind the protection line ---
    // For each non-failed structure, reduce flood depth in cells on the
    // opposite side from the water source.
    if flood_state.is_flooding {
        for structure in &protection.structures {
            if structure.failed {
                continue;
            }
            let x = structure.grid_x as usize;
            let y = structure.grid_y as usize;
            if x >= GRID_WIDTH || y >= GRID_HEIGHT {
                continue;
            }

            let effective_height = structure.effective_height();
            if effective_height <= 0.0 {
                continue;
            }

            // Find neighboring non-water cells and reduce their flood depth
            let (neighbors, count) = world_grid.neighbors4(x, y);
            for &(nx, ny) in &neighbors[..count] {
                if world_grid.get(nx, ny).cell_type == CellType::Water {
                    continue;
                }
                let neighbor_depth = flood_grid.get(nx, ny);
                if neighbor_depth > 0.0 && neighbor_depth <= effective_height {
                    let reduction = neighbor_depth * 0.5; // 50% reduction for shielded cells
                    flood_grid.set(nx, ny, (neighbor_depth - reduction).max(0.0));
                }
            }
        }
    }

    // --- Step 6: Update aggregate statistics ---
    let mut active = 0u32;
    let mut failed = 0u32;
    for structure in &protection.structures {
        if structure.failed {
            failed += 1;
        } else {
            active += 1;
        }
    }
    protection.active_count = active;
    protection.failed_count = failed;
}

// =============================================================================
// Plugin
// =============================================================================

pub struct FloodProtectionPlugin;

impl Plugin for FloodProtectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FloodProtectionState>().add_systems(
            FixedUpdate,
            update_flood_protection
                .after(crate::flood_simulation::update_flood_simulation)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<FloodProtectionState>();
    }
}
