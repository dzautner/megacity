mod calculations;
mod systems;
mod types;

#[cfg(test)]
mod tests;

pub use calculations::{
    base_demand_for_building, base_demand_for_service, supply_capacity_for_utility,
};
pub use systems::{
    aggregate_water_supply, calculate_building_water_demand, water_service_happiness_penalty,
    WaterDemandPlugin, NO_WATER_SERVICE_PENALTY,
};
pub use types::{WaterDemand, WaterSupply};
