//! Main stormwater management system.
//!
//! Runs every slow tick after flood simulation. Coordinates:
//! 1. Green infrastructure runoff reduction
//! 2. Flood risk overlay computation
//! 3. Flood road damage
//! 4. Citizen displacement tracking

use bevy::prelude::*;

use crate::buildings::Building;
use crate::citizen::CitizenDetails;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::flood_simulation::{FloodGrid, FloodState};
use crate::grid::WorldGrid;
use crate::road_maintenance::RoadConditionGrid;
use crate::services::ServiceBuilding;
use crate::services::ServiceType;
use crate::storm_drainage::StormDrainageState;
use crate::stormwater::StormwaterGrid;
use crate::trees::TreeGrid;
use crate::SlowTickTimer;

use super::flood_risk::{compute_cell_risk, FloodRiskGrid};
use super::green_infra::apply_green_infrastructure;
use super::road_damage::apply_flood_road_damage;
use super::state::StormwaterMgmtState;

use crate::citizen::HomeLocation;
use crate::flood_simulation::damage_curves::FLOOD_DEPTH_THRESHOLD;
use crate::stormwater::imperviousness;

/// Happiness penalty per tick for citizens living in a flooded area.
const FLOOD_HAPPINESS_PENALTY: f32 = 2.0;

/// Health penalty per tick for citizens living in a flooded area.
const FLOOD_HEALTH_PENALTY: f32 = 0.5;

/// Flood risk score above which a cell is considered "high risk".
const HIGH_RISK_THRESHOLD: u8 = 180;

/// Main stormwater management update system.
///
/// Runs every slow tick after flood simulation completes.
#[allow(clippy::too_many_arguments)]
pub fn update_stormwater_mgmt(
    slow_timer: Res<SlowTickTimer>,
    mut mgmt_state: ResMut<StormwaterMgmtState>,
    mut stormwater: ResMut<StormwaterGrid>,
    mut flood_risk: ResMut<FloodRiskGrid>,
    flood_grid: Res<FloodGrid>,
    flood_state: Res<FloodState>,
    world_grid: Res<WorldGrid>,
    drainage_state: Res<StormDrainageState>,
    tree_grid: Res<TreeGrid>,
    mut road_condition: ResMut<RoadConditionGrid>,
    services: Query<&ServiceBuilding>,
    buildings: Query<&Building>,
    mut citizens: Query<(&HomeLocation, &mut CitizenDetails)>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Identify park cells for green infrastructure ---
    let park_cells = build_park_cell_mask(&services);

    // --- Phase 2: Apply green infrastructure runoff reduction ---
    let absorbed = apply_green_infrastructure(&mut stormwater, &tree_grid, &park_cells);
    mgmt_state.green_infra_absorbed = absorbed;

    // --- Phase 3: Update flood risk overlay ---
    update_flood_risk_grid(
        &mut flood_risk,
        &world_grid,
        drainage_state.drainage_coverage,
    );
    let (avg_risk, high_risk_count) = compute_risk_stats(&flood_risk);
    mgmt_state.avg_flood_risk = avg_risk;
    mgmt_state.high_risk_cells = high_risk_count;

    // --- Phase 4: Apply flood road damage ---
    let damaged_roads = if flood_state.is_flooding {
        apply_flood_road_damage(&flood_grid, &world_grid, &mut road_condition)
    } else {
        0
    };
    mgmt_state.flood_damaged_roads = damaged_roads;

    // --- Phase 5: Citizen displacement from flooded buildings ---
    let displaced = if flood_state.is_flooding {
        apply_citizen_displacement(&flood_grid, &buildings, &mut citizens)
    } else {
        0
    };
    mgmt_state.displaced_citizens = displaced;
}

/// Build a boolean mask of cells that are covered by park services.
fn build_park_cell_mask(services: &Query<&ServiceBuilding>) -> Vec<bool> {
    let mut mask = vec![false; GRID_WIDTH * GRID_HEIGHT];

    for service in services.iter() {
        let is_park = matches!(
            service.service_type,
            ServiceType::SmallPark | ServiceType::LargePark | ServiceType::Playground
        );
        if !is_park {
            continue;
        }

        let (fw, fh) = ServiceBuilding::footprint(service.service_type);
        for dy in 0..fh {
            for dx in 0..fw {
                let x = service.grid_x + dx;
                let y = service.grid_y + dy;
                if x < GRID_WIDTH && y < GRID_HEIGHT {
                    mask[y * GRID_WIDTH + x] = true;
                }
            }
        }
    }

    mask
}

/// Recompute the flood risk grid based on current world state.
fn update_flood_risk_grid(
    risk_grid: &mut FloodRiskGrid,
    world_grid: &WorldGrid,
    drainage_coverage: f32,
) {
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = world_grid.get(x, y);
            let has_building = cell.building_id.is_some();
            let imperv = imperviousness(cell.cell_type, cell.zone, has_building);
            let risk = compute_cell_risk(cell.elevation, imperv, drainage_coverage);
            risk_grid.set(x, y, risk);
        }
    }
}

/// Compute aggregate risk statistics.
fn compute_risk_stats(risk_grid: &FloodRiskGrid) -> (f32, u32) {
    let mut sum = 0u64;
    let mut high_count = 0u32;

    for &r in &risk_grid.risk {
        sum += r as u64;
        if r > HIGH_RISK_THRESHOLD {
            high_count += 1;
        }
    }

    let avg = if risk_grid.risk.is_empty() {
        0.0
    } else {
        sum as f32 / risk_grid.risk.len() as f32
    };

    (avg, high_count)
}

/// Apply happiness and health penalties to citizens living in flooded buildings.
/// Returns the number of affected citizens.
fn apply_citizen_displacement(
    flood_grid: &FloodGrid,
    buildings: &Query<&Building>,
    citizens: &mut Query<(&HomeLocation, &mut CitizenDetails)>,
) -> u32 {
    let mut displaced = 0u32;

    for (home, mut details) in citizens.iter_mut() {
        let gx = home.grid_x;
        let gy = home.grid_y;

        if gx >= GRID_WIDTH || gy >= GRID_HEIGHT {
            continue;
        }

        let depth = flood_grid.get(gx, gy);
        if depth < FLOOD_DEPTH_THRESHOLD {
            continue;
        }

        // Verify there is a building at the home location
        if let Ok(_building) = buildings.get(home.building) {
            details.happiness = (details.happiness - FLOOD_HAPPINESS_PENALTY).max(0.0);
            details.health = (details.health - FLOOD_HEALTH_PENALTY).max(0.0);
            displaced += 1;
        }
    }

    displaced
}
