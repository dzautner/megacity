//! Recycling simulation systems and plugin registration.

use bevy::prelude::*;

use super::economics::{RecyclingEconomics, COLLECTION_COST_PER_TON, PROCESSING_COST_PER_TON};
use super::state::RecyclingState;
use crate::time_of_day::GameClock;
use crate::SlowTickTimer;

/// Advances the recycling market cycle and recalculates daily economics.
///
/// Runs on the slow tick (~every 10 game seconds, treated as ~1 game day).
/// Reads the current waste generation from `WasteSystem` and applies the
/// selected recycling tier's diversion rate, contamination, and economics.
#[allow(clippy::too_many_arguments)]
pub fn update_recycling_economics(
    slow_timer: Res<SlowTickTimer>,
    clock: Res<GameClock>,
    mut economics: ResMut<RecyclingEconomics>,
    mut state: ResMut<RecyclingState>,
    waste_system: Res<crate::garbage::WasteSystem>,
    stats: Res<crate::stats::CityStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Advance market cycle
    economics.update_market_cycle(clock.day);

    let tier = state.tier;
    let generated_tons = waste_system.period_generated_tons;

    // Compute households (approximate: population / 3 avg household size)
    let households = (stats.population as f64 / 3.0).max(0.0);
    let participating = (households * tier.participation_rate() as f64) as u32;
    state.participating_households = participating;

    // Diversion: fraction of total waste diverted based on tier
    let gross_diverted = generated_tons * tier.diversion_rate() as f64;

    // Contamination: fraction of diverted material that is actually waste
    let contaminated = gross_diverted * tier.contamination_rate() as f64;
    let net_diverted = gross_diverted - contaminated;

    state.daily_tons_diverted = net_diverted;
    state.daily_tons_contaminated = contaminated;

    // Revenue from selling clean recyclables
    let revenue_per_ton = economics.revenue_per_ton() * tier.revenue_potential() as f64;
    let revenue = net_diverted * revenue_per_ton;

    // Costs: per-household annual cost prorated to daily + per-ton processing
    let daily_household_cost = participating as f64 * tier.cost_per_household_year() / 365.0;
    let per_ton_cost = gross_diverted * (PROCESSING_COST_PER_TON + COLLECTION_COST_PER_TON);
    let total_cost = daily_household_cost + per_ton_cost;

    state.daily_revenue = revenue;
    state.daily_cost = total_cost;
    state.total_revenue += revenue;
    state.total_cost += total_cost;
}

pub struct RecyclingPlugin;

impl Plugin for RecyclingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RecyclingEconomics>()
            .init_resource::<RecyclingState>()
            .add_systems(
                FixedUpdate,
                update_recycling_economics
                    .after(crate::garbage::update_waste_generation)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
