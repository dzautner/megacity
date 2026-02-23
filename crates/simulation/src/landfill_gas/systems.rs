//! ECS systems for landfill gas simulation.

use bevy::prelude::*;

use super::calculations::*;
use super::constants::*;
use super::state::LandfillGasState;
use crate::garbage::WasteSystem;
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

/// Updates landfill gas state each slow tick.
///
/// 1. Queries ServiceBuilding for Landfill type, counts total landfills.
/// 2. Calculates gas generation: daily_waste_input * 100 cubic_ft_per_ton_per_year.
/// 3. If collection_active: electricity = captured_gas * conversion_factor.
/// 4. Conversion: 1 MW per 1,000 tons/day of waste in landfill.
/// 5. If no collection: all methane is uncaptured (GHG emission).
/// 6. Fire/explosion risk: 0.001 per year without collection (checked each slow tick).
/// 7. Updates all state fields.
pub fn update_landfill_gas(
    slow_timer: Res<SlowTickTimer>,
    mut state: ResMut<LandfillGasState>,
    waste_system: Res<WasteSystem>,
    services: Query<&ServiceBuilding>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Count total landfills ---
    let mut total_landfills: u32 = 0;
    for service in &services {
        if service.service_type == ServiceType::Landfill {
            total_landfills += 1;
        }
    }
    state.total_landfills = total_landfills;

    // --- Phase 2: Calculate gas generation ---
    // Use the city-wide waste generation from WasteSystem as the daily waste input.
    // total_generated_tons is the cumulative total; period_generated_tons is per-tick.
    // Each slow tick represents ~1 game-day, so period_generated_tons ~ daily waste.
    let daily_waste_tons = waste_system.period_generated_tons;
    let gas_generation_cf_per_year = calculate_gas_generation(daily_waste_tons);
    state.total_gas_generation_cf_per_year = gas_generation_cf_per_year;

    // --- Phase 3: Determine collection status ---
    // Collection is active if the flag is set AND there are landfills with collection.
    // The number of landfills with collection equals total_landfills when collection
    // infrastructure has been built (controlled by collection_active flag).
    let landfills_with_collection = if state.collection_active {
        total_landfills
    } else {
        0
    };
    state.landfills_with_collection = landfills_with_collection;

    let landfills_without_collection = total_landfills - landfills_with_collection;

    // --- Phase 4: Calculate electricity generation ---
    if state.collection_active && total_landfills > 0 {
        let electricity_mw =
            calculate_electricity_mw(daily_waste_tons, state.collection_efficiency);
        state.electricity_generated_mw = electricity_mw as f32;
    } else {
        state.electricity_generated_mw = 0.0;
    }

    // --- Phase 5: Calculate uncaptured methane ---
    let uncaptured_methane = calculate_uncaptured_methane(
        gas_generation_cf_per_year,
        state.collection_active,
        state.collection_efficiency,
    );
    state.uncaptured_methane_cf = uncaptured_methane as f32;

    // --- Phase 6: Calculate infrastructure and maintenance costs ---
    state.infrastructure_cost =
        landfills_with_collection as f64 * COLLECTION_INFRA_COST_PER_LANDFILL;
    state.maintenance_cost_per_year =
        landfills_with_collection as f64 * MAINTENANCE_COST_PER_LANDFILL_YEAR;

    // --- Phase 7: Calculate fire/explosion risk ---
    state.fire_explosion_risk = calculate_fire_risk(landfills_without_collection);
}
