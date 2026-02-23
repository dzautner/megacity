use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::trees::TreeGrid;
use crate::wind::WindState;
use crate::TickCounter;

use super::types::{
    power_outage_probability, rand_f32, tree_knockdown_probability, wind_damage_amount,
    WindDamageEvent, WindDamageState, WindDamageTier, WIND_DAMAGE_THRESHOLD,
};

// =============================================================================
// Systems
// =============================================================================

/// Wind damage update interval in ticks (aligns with wind update interval).
const WIND_DAMAGE_INTERVAL: u64 = 100;

/// Updates wind damage state each interval based on current wind speed.
///
/// - Classifies wind into a damage tier
/// - Accumulates building damage for speeds > 0.4
/// - Probabilistically knocks down trees at high wind speeds
/// - Sets power outage flag based on outage probability
///
/// Resets accumulated counters when wind drops below damage threshold.
pub fn update_wind_damage(
    tick: Res<TickCounter>,
    wind: Res<WindState>,
    mut state: ResMut<WindDamageState>,
    mut tree_grid: ResMut<TreeGrid>,
    mut events: EventWriter<WindDamageEvent>,
) {
    if tick.0 == 0 || !tick.0.is_multiple_of(WIND_DAMAGE_INTERVAL) {
        return;
    }

    let speed = wind.speed;
    let tier = WindDamageTier::from_speed(speed);
    state.current_tier = tier;

    // If below damage threshold, reset storm counters and exit
    if speed <= WIND_DAMAGE_THRESHOLD {
        // Only reset when transitioning from a damaging state
        if state.accumulated_building_damage > 0.0 || state.trees_knocked_down > 0 {
            state.accumulated_building_damage = 0.0;
            state.trees_knocked_down = 0;
        }
        state.power_outage_active = false;
        return;
    }

    // --- Building damage ---
    let damage = wind_damage_amount(speed);
    state.accumulated_building_damage += damage;

    // --- Power outage ---
    let outage_prob = power_outage_probability(speed);
    let outage_seed = tick.0.wrapping_mul(0xdeadbeef_cafebabe);
    let outage_roll = rand_f32(outage_seed);
    state.power_outage_active = outage_roll < outage_prob;

    // --- Tree knockdown ---
    let knockdown_prob = tree_knockdown_probability(speed);
    let mut trees_knocked_this_tick: u32 = 0;

    if knockdown_prob > 0.0 {
        // Iterate over the grid to find trees and probabilistically knock them down.
        // Use deterministic hash based on tick + position for each cell.
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if tree_grid.has_tree(x, y) {
                    let cell_seed = tick
                        .0
                        .wrapping_mul(0x517cc1b727220a95)
                        .wrapping_add((y * GRID_WIDTH + x) as u64);
                    let roll = rand_f32(cell_seed);
                    if roll < knockdown_prob {
                        tree_grid.set(x, y, false);
                        trees_knocked_this_tick += 1;
                    }
                }
            }
        }
    }

    state.trees_knocked_down += trees_knocked_this_tick;

    // Fire event if any damage occurred
    if damage > 0.0 || trees_knocked_this_tick > 0 || state.power_outage_active {
        events.send(WindDamageEvent {
            tier,
            building_damage: damage,
            trees_knocked: trees_knocked_this_tick,
            power_outage: state.power_outage_active,
        });
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct WindDamagePlugin;

impl Plugin for WindDamagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindDamageState>()
            .add_event::<WindDamageEvent>()
            .add_systems(
                FixedUpdate,
                update_wind_damage
                    .after(crate::imports_exports::process_trade)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
