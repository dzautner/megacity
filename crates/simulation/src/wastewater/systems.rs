//! Wastewater simulation systems and plugin registration.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails, HomeLocation};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::WorldGrid;
use crate::utilities::{UtilitySource, UtilityType};
use crate::water_demand::WaterDemand;
use crate::water_pollution::WaterPollutionGrid;

use super::types::{
    find_discharge_water_cells, is_serviced_by_sewage_plant, sewage_for_demand, WastewaterState,
    DISCHARGE_POLLUTION_AMOUNT, HAPPINESS_PENALTY_PER_TICK, HEALTH_PENALTY_PER_TICK,
    TREATMENT_CAPACITY_PER_PLANT, UNCOLLECTED_PENALTY_RADIUS,
};

// =============================================================================
// Systems
// =============================================================================

/// Main wastewater update system. Runs every slow tick.
///
/// - Queries buildings with `WaterDemand` to compute sewage generation (80% of water use)
/// - Queries `UtilitySource` for `SewagePlant` to get treatment capacity and plant locations
/// - Computes coverage ratio (fraction of buildings within service radius of a plant)
/// - If overflow (sewage > capacity), discharges pollution to nearby water cells
/// - Sets health penalty flag when residential buildings lack sewage service
#[allow(clippy::too_many_arguments)]
pub fn update_wastewater(
    slow_timer: Res<crate::SlowTickTimer>,
    mut wastewater: ResMut<WastewaterState>,
    grid: Res<WorldGrid>,
    mut water_pollution: ResMut<WaterPollutionGrid>,
    buildings: Query<(&Building, &WaterDemand)>,
    utilities: Query<&UtilitySource>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Collect sewage plant positions and compute treatment capacity ---
    let mut sewage_plants: Vec<(usize, usize)> = Vec::new();
    let mut total_capacity: f32 = 0.0;

    for utility in &utilities {
        if utility.utility_type == UtilityType::SewagePlant {
            sewage_plants.push((utility.grid_x, utility.grid_y));
            total_capacity += TREATMENT_CAPACITY_PER_PLANT;
        }
    }

    // --- Phase 2: Compute total sewage generation and coverage ---
    let mut total_sewage: f32 = 0.0;
    let mut buildings_total: u32 = 0;
    let mut buildings_serviced: u32 = 0;
    let mut residential_unserviced = false;

    for (building, demand) in &buildings {
        let sewage = sewage_for_demand(demand);
        total_sewage += sewage;
        buildings_total += 1;

        let serviced =
            is_serviced_by_sewage_plant(building.grid_x, building.grid_y, &sewage_plants);
        if serviced {
            buildings_serviced += 1;
        } else if building.zone_type.is_residential() {
            residential_unserviced = true;
        }
    }

    let coverage_ratio = if buildings_total > 0 {
        buildings_serviced as f32 / buildings_total as f32
    } else {
        1.0 // No buildings means full coverage (nothing to service)
    };

    // --- Phase 3: Compute overflow ---
    let overflow = (total_sewage - total_capacity).max(0.0);

    // --- Phase 4: If overflow, discharge pollution to nearby water cells ---
    if overflow > 0.0 {
        wastewater.pollution_events += 1;

        let water_cells = find_discharge_water_cells(&grid, &sewage_plants);
        // Scale pollution by overflow severity (more overflow = more pollution per cell)
        let severity_mult = (overflow / TREATMENT_CAPACITY_PER_PLANT).clamp(0.5, 3.0);
        let pollution_amount = (DISCHARGE_POLLUTION_AMOUNT as f32 * severity_mult) as u8;

        for &(wx, wy) in &water_cells {
            let idx = wy * water_pollution.width + wx;
            water_pollution.levels[idx] =
                water_pollution.levels[idx].saturating_add(pollution_amount);
        }
    }

    // --- Phase 5: Update state ---
    wastewater.total_sewage_generated = total_sewage;
    wastewater.total_treatment_capacity = total_capacity;
    wastewater.overflow_amount = overflow;
    wastewater.coverage_ratio = coverage_ratio;
    wastewater.health_penalty_active = residential_unserviced && buildings_total > 0;
}

/// Health and happiness penalty for citizens living near areas without sewage service.
/// Citizens whose homes are in residential buildings not covered by a sewage plant
/// suffer reduced health and happiness.
pub fn wastewater_health_penalty(
    slow_timer: Res<crate::SlowTickTimer>,
    wastewater: Res<WastewaterState>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation), With<Citizen>>,
    buildings: Query<&Building>,
    utilities: Query<&UtilitySource>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Only apply penalty if the system has detected uncollected sewage
    if !wastewater.health_penalty_active {
        return;
    }

    // Collect sewage plant positions
    let sewage_plants: Vec<(usize, usize)> = utilities
        .iter()
        .filter(|u| u.utility_type == UtilityType::SewagePlant)
        .map(|u| (u.grid_x, u.grid_y))
        .collect();

    // Build a set of residential building positions without sewage service
    let unserviced_residential: Vec<(usize, usize)> = buildings
        .iter()
        .filter(|b| b.zone_type.is_residential())
        .filter(|b| !is_serviced_by_sewage_plant(b.grid_x, b.grid_y, &sewage_plants))
        .map(|b| (b.grid_x, b.grid_y))
        .collect();

    if unserviced_residential.is_empty() {
        return;
    }

    for (mut details, home) in &mut citizens {
        let hx = home.grid_x;
        let hy = home.grid_y;

        if hx >= GRID_WIDTH || hy >= GRID_HEIGHT {
            continue;
        }

        // Check if citizen's home is near any unserviced residential building
        let near_unserviced = unserviced_residential.iter().any(|&(bx, by)| {
            let dx = (hx as i32 - bx as i32).abs();
            let dy = (hy as i32 - by as i32).abs();
            dx + dy <= UNCOLLECTED_PENALTY_RADIUS
        });

        if near_unserviced {
            details.health = (details.health - HEALTH_PENALTY_PER_TICK).max(0.0);
            details.happiness = (details.happiness - HAPPINESS_PENALTY_PER_TICK).max(0.0);
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct WastewaterPlugin;

impl Plugin for WastewaterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WastewaterState>().add_systems(
            FixedUpdate,
            (update_wastewater, wastewater_health_penalty)
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
