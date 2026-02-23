//! Systems for computing and aggregating per-building energy demand.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::degree_days::DegreeDays;
use crate::services::ServiceBuilding;
use crate::time_of_day::GameClock;
use crate::weather::Weather;
use crate::TickCounter;

use super::types::{EnergyConsumer, EnergyGrid};

/// Demand aggregation runs every N ticks.
const AGGREGATION_INTERVAL: u64 = 4;

/// Hours per month for converting kWh/month to kW instantaneous.
/// 30 days * 24 hours = 720 hours/month.
const HOURS_PER_MONTH: f32 = 720.0;

// =============================================================================
// Time-of-use demand curve
// =============================================================================

/// Time-of-use demand multiplier based on hour of day.
///
/// - Off-peak (22:00 - 06:00): 0.6x
/// - Mid-peak (06:00 - 14:00): 1.0x
/// - On-peak  (14:00 - 22:00): 1.5x
pub fn time_of_use_multiplier(hour: f32) -> f32 {
    let h = hour as u32;
    match h {
        22..=23 | 0..=5 => 0.6,
        6..=13 => 1.0,
        14..=21 => 1.5,
        _ => 1.0,
    }
}

/// Compute the instantaneous demand (MW) for a single consumer.
///
/// Formula:
///   demand_kw = (base_demand_kwh / 720) * tou_mult * hvac_mod * power_mult
///   demand_mw = demand_kw / 1000
pub fn compute_demand_mw(
    base_demand_kwh: f32,
    tou_multiplier: f32,
    hvac_modifier: f32,
    power_multiplier: f32,
) -> f32 {
    let demand_kw =
        (base_demand_kwh / HOURS_PER_MONTH) * tou_multiplier * hvac_modifier * power_multiplier;
    demand_kw / 1000.0
}

// =============================================================================
// Component attachment systems
// =============================================================================

/// Attaches `EnergyConsumer` components to zoned buildings that lack one.
pub fn attach_energy_consumer_to_buildings(
    mut commands: Commands,
    buildings: Query<(Entity, &Building), Without<EnergyConsumer>>,
) {
    for (entity, building) in &buildings {
        let base = EnergyConsumer::base_demand_for_zone(building.zone_type);
        if base > 0.0 {
            let priority = EnergyConsumer::priority_for_zone(building.zone_type);
            commands
                .entity(entity)
                .insert(EnergyConsumer::new(base, priority));
        }
    }
}

/// Attaches `EnergyConsumer` components to service buildings that lack one.
pub fn attach_energy_consumer_to_services(
    mut commands: Commands,
    services: Query<(Entity, &ServiceBuilding), Without<EnergyConsumer>>,
) {
    for (entity, service) in &services {
        let base = EnergyConsumer::base_demand_for_service(service.service_type);
        if base > 0.0 {
            let priority = EnergyConsumer::priority_for_service(service.service_type);
            commands
                .entity(entity)
                .insert(EnergyConsumer::new(base, priority));
        }
    }
}

// =============================================================================
// Aggregation system
// =============================================================================

/// Aggregates total energy demand from all consumers every 4 ticks.
///
/// Applies time-of-use curve, HVAC modifier, and seasonal power multiplier.
pub fn aggregate_energy_demand(
    tick: Res<TickCounter>,
    clock: Res<GameClock>,
    weather: Res<Weather>,
    degree_days: Res<DegreeDays>,
    consumers: Query<&EnergyConsumer>,
    mut energy_grid: ResMut<EnergyGrid>,
) {
    if tick.0 % AGGREGATION_INTERVAL != 0 {
        return;
    }

    let tou = time_of_use_multiplier(clock.hour);
    let hvac = degree_days.hvac_modifier();
    let power = weather.power_multiplier();

    let mut total_demand = 0.0_f32;
    let mut count = 0u32;

    for consumer in &consumers {
        total_demand += compute_demand_mw(consumer.base_demand_kwh, tou, hvac, power);
        count += 1;
    }

    energy_grid.total_demand_mwh = total_demand;
    energy_grid.consumer_count = count;

    // Compute reserve margin
    if energy_grid.total_supply_mwh > 0.0 {
        energy_grid.reserve_margin =
            (energy_grid.total_supply_mwh - total_demand) / energy_grid.total_supply_mwh;
    } else if total_demand > 0.0 {
        energy_grid.reserve_margin = -1.0;
    } else {
        energy_grid.reserve_margin = 1.0;
    }
}
