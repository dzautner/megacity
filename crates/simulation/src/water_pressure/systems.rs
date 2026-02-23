//! Water pressure update system and plugin registration.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::grid::WorldGrid;
use crate::SlowTickTimer;

use super::state::WaterPressureState;
use super::types::{
    classify_pressure, effective_pressure_elevation, pressure_factor, BoosterPumpStation,
    BOOSTER_PUMP_COST,
};

// =============================================================================
// System
// =============================================================================

/// System: Recalculate water pressure zone statistics every slow tick.
///
/// 1. Counts active booster pump stations.
/// 2. Computes effective pressure elevation.
/// 3. Evaluates each building's elevation against the pressure zone.
/// 4. Updates statistics on the `WaterPressureState` resource.
pub fn update_water_pressure(
    timer: Res<SlowTickTimer>,
    grid: Res<WorldGrid>,
    boosters: Query<&BoosterPumpStation>,
    buildings: Query<&Building>,
    mut pressure_state: ResMut<WaterPressureState>,
) {
    if !timer.should_run() {
        return;
    }

    // Step 1: Count booster pump stations.
    let booster_count = boosters.iter().count() as u32;

    // Step 2: Compute effective elevation.
    let effective_elev = effective_pressure_elevation(booster_count);

    // Step 3: Evaluate each building.
    let mut full_count: u32 = 0;
    let mut reduced_count: u32 = 0;
    let mut no_count: u32 = 0;
    let mut pressure_sum: f32 = 0.0;
    let mut building_count: u32 = 0;

    for building in &buildings {
        let elevation = grid.get(building.grid_x, building.grid_y).elevation;
        let factor = pressure_factor(elevation, effective_elev);

        match classify_pressure(factor) {
            super::types::PressureCategory::Full => full_count += 1,
            super::types::PressureCategory::Reduced => reduced_count += 1,
            super::types::PressureCategory::None => no_count += 1,
        }

        pressure_sum += factor;
        building_count += 1;
    }

    // Step 4: Update state.
    pressure_state.booster_count = booster_count;
    pressure_state.effective_elevation = effective_elev;
    pressure_state.buildings_full_pressure = full_count;
    pressure_state.buildings_reduced_pressure = reduced_count;
    pressure_state.buildings_no_pressure = no_count;
    pressure_state.total_booster_cost = booster_count as f64 * BOOSTER_PUMP_COST;

    pressure_state.average_pressure_factor = if building_count > 0 {
        pressure_sum / building_count as f32
    } else {
        1.0 // Default to full pressure when there are no buildings.
    };
}

// =============================================================================
// Plugin
// =============================================================================

pub struct WaterPressurePlugin;

impl Plugin for WaterPressurePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterPressureState>();

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<WaterPressureState>();

        app.add_systems(
            FixedUpdate,
            update_water_pressure
                .after(crate::utilities::propagate_utilities)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
