use bevy::prelude::*;

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails, HomeLocation};
use crate::services::ServiceBuilding;
use crate::weather::Weather;
use crate::SlowTickTimer;

use super::calculations::{
    base_demand_for_building, base_demand_for_service, supply_capacity_for_utility,
};
use super::types::{WaterDemand, WaterSupply};

// =============================================================================
// Systems
// =============================================================================

/// System: Calculate per-building water demand for zoned buildings.
/// Attaches/updates `WaterDemand` components. Runs on the slow tick.
pub fn calculate_building_water_demand(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    grid: Res<crate::grid::WorldGrid>,
    mut commands: Commands,
    buildings_without_demand: Query<(Entity, &Building), Without<WaterDemand>>,
    mut buildings_with_demand: Query<(Entity, &Building, &mut WaterDemand)>,
) {
    if !timer.should_run() {
        return;
    }

    let water_mult = weather.water_multiplier();

    // Attach WaterDemand to buildings that don't have it yet
    for (entity, building) in &buildings_without_demand {
        let base = base_demand_for_building(building);
        let has_water = grid.get(building.grid_x, building.grid_y).has_water;
        commands.entity(entity).insert(WaterDemand {
            demand_gpd: base * water_mult,
            has_water_service: has_water,
        });
    }

    // Update existing WaterDemand components
    for (_entity, building, mut demand) in &mut buildings_with_demand {
        let base = base_demand_for_building(building);
        demand.demand_gpd = base * water_mult;
        demand.has_water_service = grid.get(building.grid_x, building.grid_y).has_water;
    }
}

/// System: Aggregate city-wide water demand and supply totals.
/// Runs on the slow tick.
pub fn aggregate_water_supply(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    mut water_supply: ResMut<WaterSupply>,
    building_demands: Query<&WaterDemand>,
    services: Query<&ServiceBuilding>,
    utilities: Query<&crate::utilities::UtilitySource>,
) {
    if !timer.should_run() {
        return;
    }

    let water_mult = weather.water_multiplier();

    // Sum building demands
    let mut total_demand: f32 = 0.0;
    let mut served: u32 = 0;
    let mut unserved: u32 = 0;

    for demand in &building_demands {
        total_demand += demand.demand_gpd;
        if demand.has_water_service {
            served += 1;
        } else {
            unserved += 1;
        }
    }

    // Add service building demands (hospitals, schools, parks, etc.)
    for service in &services {
        let base = base_demand_for_service(service);
        total_demand += base * water_mult;
    }

    // Compute total supply from water utilities
    let mut total_supply: f32 = 0.0;
    for utility in &utilities {
        total_supply += supply_capacity_for_utility(utility.utility_type);
    }

    water_supply.total_demand_gpd = total_demand;
    water_supply.total_supply_gpd = total_supply;
    water_supply.buildings_served = served;
    water_supply.buildings_unserved = unserved;
    water_supply.supply_ratio = if total_demand > 0.0 {
        total_supply / total_demand
    } else {
        1.0
    };
}

/// Happiness penalty applied to citizens living in buildings without water service.
pub const NO_WATER_SERVICE_PENALTY: f32 = 8.0;

/// System: Citizens in buildings without water service suffer a happiness penalty.
/// Runs on the slow tick.
pub fn water_service_happiness_penalty(
    timer: Res<SlowTickTimer>,
    grid: Res<crate::grid::WorldGrid>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation), With<Citizen>>,
) {
    if !timer.should_run() {
        return;
    }

    for (mut details, home) in &mut citizens {
        let hx = home.grid_x;
        let hy = home.grid_y;
        if hx >= crate::config::GRID_WIDTH || hy >= crate::config::GRID_HEIGHT {
            continue;
        }

        let cell = grid.get(hx, hy);
        if !cell.has_water {
            // Reduce happiness for citizens without water service
            details.happiness = (details.happiness - NO_WATER_SERVICE_PENALTY).max(0.0);
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct WaterDemandPlugin;

impl Plugin for WaterDemandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterSupply>()
            .add_systems(
                FixedUpdate,
                (calculate_building_water_demand, aggregate_water_supply)
                    .after(crate::imports_exports::process_trade)
                    .in_set(crate::SimulationSet::Simulation),
            )
            .add_systems(
                FixedUpdate,
                water_service_happiness_penalty
                    .after(crate::imports_exports::process_trade)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
