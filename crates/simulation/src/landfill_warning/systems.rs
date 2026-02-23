use bevy::prelude::*;

use crate::garbage::WasteSystem;
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

use super::calculations::{
    advance_fill, compute_days_remaining, compute_remaining_pct, tier_from_remaining_pct,
    DAYS_PER_YEAR, LANDFILL_CAPACITY_PER_BUILDING,
};
use super::types::{LandfillCapacityState, LandfillWarningEvent, LandfillWarningTier};

/// Updates landfill capacity state each slow tick.
///
/// 1. Counts `Landfill` service buildings to compute total capacity.
/// 2. Reads `WasteSystem.total_generated_tons` for the daily input rate.
/// 3. Advances fill level by one day's input per slow tick.
/// 4. Computes remaining percentage, days/years remaining, and warning tier.
/// 5. Fires `LandfillWarningEvent` when the tier changes.
/// 6. Sets `collection_halted` at Emergency tier.
pub fn update_landfill_capacity(
    slow_timer: Res<SlowTickTimer>,
    waste_system: Res<WasteSystem>,
    buildings: Query<&ServiceBuilding>,
    mut state: ResMut<LandfillCapacityState>,
    mut warning_events: EventWriter<LandfillWarningEvent>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // 1. Count landfill buildings and compute total capacity.
    let landfill_count = buildings
        .iter()
        .filter(|b| b.service_type == ServiceType::Landfill)
        .count() as u32;
    let total_capacity = landfill_count as f64 * LANDFILL_CAPACITY_PER_BUILDING;

    // 2. Read daily waste input.
    let daily_input_rate = waste_system.period_generated_tons;

    // 3. Advance fill level (one slow tick ~ one game day).
    let current_fill = advance_fill(state.current_fill, daily_input_rate, total_capacity);

    // 4. Compute derived metrics.
    let remaining_pct = compute_remaining_pct(total_capacity, current_fill);
    let days_remaining = compute_days_remaining(total_capacity, current_fill, daily_input_rate);
    let years_remaining = days_remaining / DAYS_PER_YEAR;
    let new_tier = tier_from_remaining_pct(remaining_pct);

    // 5. Fire event on tier change.
    let old_tier = state.current_tier;
    if new_tier != old_tier {
        warning_events.send(LandfillWarningEvent {
            tier: new_tier,
            remaining_pct: remaining_pct as f32,
        });
    }

    // 6. Update state.
    state.total_capacity = total_capacity;
    state.current_fill = current_fill;
    state.daily_input_rate = daily_input_rate;
    state.days_remaining = days_remaining;
    state.years_remaining = years_remaining;
    state.remaining_pct = remaining_pct as f32;
    state.current_tier = new_tier;
    state.collection_halted = new_tier == LandfillWarningTier::Emergency;
    state.landfill_count = landfill_count;
}

pub struct LandfillWarningPlugin;

impl Plugin for LandfillWarningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LandfillCapacityState>()
            .add_event::<LandfillWarningEvent>()
            .add_systems(
                FixedUpdate,
                update_landfill_capacity
                    .after(crate::imports_exports::process_trade)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
