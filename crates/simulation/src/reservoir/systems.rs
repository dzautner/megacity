use bevy::prelude::*;

use super::types::{
    warning_tier_from_fill, ReservoirState, ReservoirWarningEvent, ReservoirWarningTier,
    BASE_EVAPORATION_RATE, CATCHMENT_FACTOR, MGD_TO_GPD, TEMPERATURE_EVAP_FACTOR,
};
use crate::water_demand::WaterSupply;
use crate::water_sources::{WaterSource, WaterSourceType};
use crate::weather::Weather;
use crate::SlowTickTimer;

// =============================================================================
// Systems
// =============================================================================

/// System: Update reservoir levels based on rainfall inflow, demand outflow,
/// and evaporation. Fires `ReservoirWarningEvent` when the warning tier changes.
///
/// Runs on the SlowTickTimer.
pub fn update_reservoir_levels(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    water_supply: Res<WaterSupply>,
    mut reservoir_state: ResMut<ReservoirState>,
    mut sources: Query<&mut WaterSource>,
    mut warning_events: EventWriter<ReservoirWarningEvent>,
) {
    if !timer.should_run() {
        return;
    }

    // ---- Step 1-2: Find reservoirs, sum capacity and current levels ----
    let mut total_capacity_gallons: f32 = 0.0;
    let mut total_stored_gallons: f32 = 0.0;
    let mut reservoir_count: u32 = 0;

    for source in sources.iter() {
        if source.source_type != WaterSourceType::Reservoir {
            continue;
        }
        total_capacity_gallons += source.storage_capacity;
        total_stored_gallons += source.stored_gallons;
        reservoir_count += 1;
    }

    reservoir_state.reservoir_count = reservoir_count;

    // Convert gallons to million gallons for the resource.
    reservoir_state.total_storage_capacity_mg = total_capacity_gallons / MGD_TO_GPD;
    reservoir_state.current_level_mg = total_stored_gallons / MGD_TO_GPD;

    // If there are no reservoirs, zero everything out and return early.
    if reservoir_count == 0 {
        reservoir_state.inflow_rate_mgd = 0.0;
        reservoir_state.outflow_rate_mgd = 0.0;
        reservoir_state.evaporation_rate_mgd = 0.0;
        reservoir_state.net_change_mgd = 0.0;
        reservoir_state.storage_days = 0.0;
        // Tier stays Normal when there are no reservoirs.
        let old_tier = reservoir_state.warning_tier;
        reservoir_state.warning_tier = ReservoirWarningTier::Normal;
        if old_tier != ReservoirWarningTier::Normal {
            warning_events.send(ReservoirWarningEvent {
                old_tier,
                new_tier: ReservoirWarningTier::Normal,
                fill_pct: 0.0,
            });
        }
        return;
    }

    // ---- Step 3: Calculate inflow from rainfall ----
    // precipitation_intensity is in inches/hour. CATCHMENT_FACTOR converts to MGD.
    let inflow_mgd = weather.precipitation_intensity * CATCHMENT_FACTOR * reservoir_count as f32;

    // ---- Step 4: Calculate outflow from water demand ----
    // WaterSupply.total_demand_gpd is in gallons per day; convert to MGD.
    let outflow_mgd = water_supply.total_demand_gpd / MGD_TO_GPD;

    // ---- Step 5: Calculate evaporation ----
    let temp_above_20 = (weather.temperature - 20.0).max(0.0);
    let evaporation_mgd =
        reservoir_count as f32 * (BASE_EVAPORATION_RATE + temp_above_20 * TEMPERATURE_EVAP_FACTOR);

    // ---- Step 6: Net change ----
    let net_change_mgd = inflow_mgd - outflow_mgd - evaporation_mgd;

    // Store rates on the resource.
    reservoir_state.inflow_rate_mgd = inflow_mgd;
    reservoir_state.outflow_rate_mgd = outflow_mgd;
    reservoir_state.evaporation_rate_mgd = evaporation_mgd;
    reservoir_state.net_change_mgd = net_change_mgd;

    // ---- Step 7: Distribute net change to each reservoir proportionally ----
    let net_change_gallons = net_change_mgd * MGD_TO_GPD;
    let mut new_total_stored: f32 = 0.0;

    for mut source in &mut sources {
        if source.source_type != WaterSourceType::Reservoir {
            continue;
        }
        // Distribute proportionally to each reservoir's share of total capacity.
        let share = if total_capacity_gallons > 0.0 {
            source.storage_capacity / total_capacity_gallons
        } else {
            0.0
        };
        let delta = net_change_gallons * share;
        source.stored_gallons = (source.stored_gallons + delta).clamp(0.0, source.storage_capacity);
        new_total_stored += source.stored_gallons;
    }

    // Update resource with post-distribution totals.
    reservoir_state.current_level_mg = new_total_stored / MGD_TO_GPD;

    // ---- Step 8: Determine warning tier from fill percentage ----
    let fill_pct = reservoir_state.fill_pct();
    let new_tier = warning_tier_from_fill(fill_pct);

    // ---- Step 9: Fire event when tier changes ----
    let old_tier = reservoir_state.warning_tier;
    reservoir_state.warning_tier = new_tier;

    // Calculate storage days: current level / daily demand.
    reservoir_state.storage_days = if outflow_mgd > 0.0 {
        reservoir_state.current_level_mg / outflow_mgd
    } else {
        f32::INFINITY
    };

    if old_tier != new_tier {
        warning_events.send(ReservoirWarningEvent {
            old_tier,
            new_tier,
            fill_pct,
        });
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct ReservoirPlugin;

impl Plugin for ReservoirPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ReservoirState>()
            .add_event::<ReservoirWarningEvent>()
            .add_systems(
                FixedUpdate,
                update_reservoir_levels
                    .after(crate::water_sources::replenish_reservoirs)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
