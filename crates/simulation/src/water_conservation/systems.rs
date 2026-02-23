use bevy::prelude::*;

use crate::buildings::Building;
use crate::weather::Weather;
use crate::SlowTickTimer;

use super::calculations::{
    calculate_annual_savings, calculate_demand_reduction, calculate_retrofit_cost,
};
use super::constants::GREYWATER_SEWAGE_REDUCTION;
use super::types::WaterConservationState;

/// System: Recalculate water conservation metrics every slow tick.
///
/// 1. Counts buildings to determine retrofit scope.
/// 2. Computes aggregate `demand_reduction_pct` (capped at 0.60).
/// 3. Computes `sewage_reduction_pct` from greywater policy.
/// 4. Computes `total_retrofit_cost` from per-building policy costs.
/// 5. Adjusts rainwater harvesting effectiveness by current precipitation.
/// 6. Updates estimated `annual_savings_gallons`.
pub fn update_water_conservation(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    mut conservation: ResMut<WaterConservationState>,
    buildings: Query<&Building>,
) {
    if !timer.should_run() {
        return;
    }

    let building_count = buildings.iter().count() as u32;

    // 1. Demand reduction (precipitation-aware for rainwater harvesting)
    let precipitation = weather.precipitation_intensity;
    conservation.demand_reduction_pct = calculate_demand_reduction(&conservation, precipitation);

    // 2. Sewage reduction
    conservation.sewage_reduction_pct = if conservation.greywater_recycling {
        GREYWATER_SEWAGE_REDUCTION
    } else {
        0.0
    };

    // 3. Retrofit costs
    conservation.buildings_retrofitted = building_count;
    conservation.total_retrofit_cost = calculate_retrofit_cost(&conservation, building_count);

    // 4. Annual savings estimate
    conservation.annual_savings_gallons =
        calculate_annual_savings(conservation.demand_reduction_pct, building_count);
}

pub struct WaterConservationPlugin;

impl Plugin for WaterConservationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterConservationState>().add_systems(
            FixedUpdate,
            update_water_conservation
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
