//! Bevy systems and plugin for heat wave mitigation.

use bevy::prelude::*;

use crate::economy::CityBudget;
use crate::heat_wave::{HeatWaveSeverity, HeatWaveState};
use crate::trees::TreeGrid;
use crate::SlowTickTimer;

use super::calculations::*;
use super::constants::COST_TICKS_PER_DAY;
use super::state::HeatMitigationState;

/// System that updates heat mitigation effects based on current heat wave state.
///
/// Runs on the slow tick timer. Only applies costs and effects when a heat wave
/// is active (severity > None). When no heat wave is active, derived effects
/// are zeroed out but player toggles remain.
pub fn update_heat_mitigation(
    timer: Res<SlowTickTimer>,
    heat_wave: Res<HeatWaveState>,
    tree_grid: Res<TreeGrid>,
    buildings: Query<&crate::buildings::Building>,
    mut mitigation: ResMut<HeatMitigationState>,
    mut budget: ResMut<CityBudget>,
) {
    if !timer.should_run() {
        return;
    }

    let is_heat_wave = heat_wave.severity != HeatWaveSeverity::None;

    // --- Green canopy: always computed (passive benefit) ---
    let tree_coverage = average_tree_coverage(&tree_grid);
    mitigation.green_canopy_temp_reduction = green_canopy_reduction(tree_coverage);

    // --- Light-colored roofs: always computed (passive benefit) ---
    let total_buildings = buildings.iter().count() as u32;
    mitigation.light_roof_temp_reduction =
        light_roof_reduction(mitigation.light_roof_count, total_buildings);

    if !is_heat_wave {
        // No heat wave: zero out active-only effects, no costs
        mitigation.mortality_reduction = 0.0;
        mitigation.misting_temp_reduction = 0.0;
        mitigation.dehydration_prevented = false;
        mitigation.last_tick_cost = 0.0;
        return;
    }

    // --- Heat wave is active: compute active mitigation effects ---

    // Cooling centers
    let cooling_active = mitigation.cooling_centers_enabled;

    // Emergency water distribution
    let water_active = mitigation.emergency_water_enabled;
    mitigation.dehydration_prevented = water_active;

    // Misting stations
    mitigation.misting_temp_reduction = misting_reduction(mitigation.misting_station_count);

    // Aggregate mortality reduction
    mitigation.mortality_reduction = total_mortality_reduction(cooling_active, water_active);

    // --- Costs: apply fractional daily cost per slow tick ---
    let daily_cost = daily_operating_cost(
        cooling_active,
        water_active,
        mitigation.misting_station_count,
    );
    // Each slow tick is approximately 1/COST_TICKS_PER_DAY of a game day
    let tick_cost = daily_cost / COST_TICKS_PER_DAY as f64;
    budget.treasury -= tick_cost;
    mitigation.last_tick_cost = tick_cost;
    mitigation.season_cost += tick_cost;
}

// =============================================================================
// Plugin
// =============================================================================

pub struct HeatMitigationPlugin;

impl Plugin for HeatMitigationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HeatMitigationState>().add_systems(
            FixedUpdate,
            update_heat_mitigation
                .after(crate::heat_wave::update_heat_wave)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<HeatMitigationState>();
    }
}
