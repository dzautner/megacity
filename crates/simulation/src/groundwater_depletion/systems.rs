//! Pure helper functions, the main depletion update system, and plugin registration.

use bevy::prelude::*;

use super::types::*;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::groundwater::GroundwaterGrid;
use crate::SlowTickTimer;

// =============================================================================
// Pure helper functions
// =============================================================================

/// Compute the well yield modifier based on average groundwater level.
///
/// - At or above `WELL_YIELD_REDUCTION_THRESHOLD` (50): modifier is 1.0 (full yield).
/// - At 0: modifier is 0.0 (no yield).
/// - Linear interpolation between 0 and the threshold.
pub fn compute_well_yield_modifier(avg_level: f32) -> f32 {
    let threshold = WELL_YIELD_REDUCTION_THRESHOLD as f32;
    if avg_level >= threshold {
        1.0
    } else if avg_level <= 0.0 {
        0.0
    } else {
        avg_level / threshold
    }
}

/// Compute the sustainability ratio from recharge and extraction rates.
///
/// Returns `f32::INFINITY` when extraction is zero (perfectly sustainable).
pub fn compute_sustainability_ratio(recharge: f32, extraction: f32) -> f32 {
    if extraction <= 0.0 {
        f32::INFINITY
    } else {
        recharge / extraction
    }
}

/// Determine whether average groundwater constitutes critical depletion.
pub fn is_critical_depletion(avg_level: f32) -> bool {
    avg_level < GROUNDWATER_CRITICAL_LEVEL as f32
}

// =============================================================================
// System
// =============================================================================

/// Main depletion update system. Runs every slow tick.
///
/// 1. Compares current `GroundwaterGrid` levels against the previous snapshot
///    to compute per-tick extraction and recharge rates.
/// 2. Computes average groundwater level and sustainability metrics.
/// 3. Updates the well yield modifier.
/// 4. Tracks per-cell subsidence counters for cells below the threshold.
/// 5. Stores the current levels as the snapshot for the next tick.
pub fn update_groundwater_depletion(
    timer: Res<SlowTickTimer>,
    groundwater: Res<GroundwaterGrid>,
    mut state: ResMut<GroundwaterDepletionState>,
) {
    if !timer.should_run() {
        return;
    }

    let total = GRID_WIDTH * GRID_HEIGHT;
    let levels = &groundwater.levels;

    // Ensure ticks_below_threshold is properly sized
    if state.ticks_below_threshold.len() != total {
        state.ticks_below_threshold.resize(total, 0);
    }

    // --- Phase 1: Compute extraction and recharge from level deltas ---
    let mut extraction_sum: f32 = 0.0;
    let mut recharge_sum: f32 = 0.0;

    if state.previous_levels.len() == total {
        for (&curr, &prev) in levels.iter().zip(state.previous_levels.iter()) {
            let delta = curr as f32 - prev as f32;
            if delta < 0.0 {
                extraction_sum -= delta; // delta is negative, so negate to get positive extraction
            } else if delta > 0.0 {
                recharge_sum += delta;
            }
        }
    }

    state.extraction_rate = extraction_sum;
    state.recharge_rate = recharge_sum;
    state.sustainability_ratio =
        compute_sustainability_ratio(state.recharge_rate, state.extraction_rate);

    // --- Phase 2: Compute average groundwater level ---
    let mut level_sum: u64 = 0;
    let mut over_extracted: u32 = 0;
    let mut at_risk: u32 = 0;

    for &level in levels.iter() {
        level_sum += level as u64;
        if level < GROUNDWATER_CRITICAL_LEVEL {
            over_extracted += 1;
        }
        if level < SUBSIDENCE_THRESHOLD {
            at_risk += 1;
        }
    }

    let avg_level = level_sum as f32 / total as f32;
    state.avg_groundwater_level = avg_level;
    state.over_extracted_cells = over_extracted;
    state.cells_at_risk = at_risk;

    // --- Phase 3: Critical depletion check ---
    state.critical_depletion = is_critical_depletion(avg_level);

    // --- Phase 4: Well yield modifier ---
    state.well_yield_modifier = compute_well_yield_modifier(avg_level);

    // --- Phase 5: Subsidence tracking ---
    let mut subsided_count: u32 = 0;
    for (&level, tick) in levels.iter().zip(state.ticks_below_threshold.iter_mut()) {
        if level < SUBSIDENCE_THRESHOLD {
            // Cell is below subsidence threshold -- increment counter
            if *tick < SUBSIDENCE_TICKS {
                *tick = tick.saturating_add(1);
            }
        } else {
            // Cell is above threshold -- reset counter (recovery)
            *tick = 0;
        }

        if *tick >= SUBSIDENCE_TICKS {
            subsided_count += 1;
        }
    }
    state.subsidence_cells = subsided_count;

    // --- Phase 6: Store snapshot for next tick ---
    state.previous_levels = levels.clone();
}

// =============================================================================
// Plugin
// =============================================================================

pub struct GroundwaterDepletionPlugin;

impl Plugin for GroundwaterDepletionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GroundwaterDepletionState>()
            .add_systems(
                FixedUpdate,
                update_groundwater_depletion
                    .after(crate::imports_exports::process_trade)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
