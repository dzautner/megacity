use bevy::prelude::*;

use super::balance::ResourceBalance;
use super::grid::ResourceGrid;
use super::types::ResourceType;

/// System: update resource production from industrial buildings on resource deposits.
/// Finite resources (ore, oil) deplete over time. Renewable resources (forests, fertile land) regenerate.
pub fn update_resource_production(
    mut resource_grid: ResMut<ResourceGrid>,
    buildings: Query<&crate::buildings::Building>,
    mut balance: ResMut<ResourceBalance>,
    stats: Res<crate::stats::CityStats>,
) {
    // Reset production
    balance.food_production = 0.0;
    balance.timber_production = 0.0;
    balance.metal_production = 0.0;
    balance.fuel_production = 0.0;

    // Industrial buildings on resource deposits produce resources
    for building in &buildings {
        if building.zone_type != crate::grid::ZoneType::Industrial {
            continue;
        }
        if let Some(deposit) = resource_grid.get(building.grid_x, building.grid_y) {
            if deposit.amount == 0 {
                continue; // Depleted deposit produces nothing
            }
            let output = building.occupants as f32 * 0.5; // Per occupied worker
            match deposit.resource_type {
                ResourceType::FertileLand => balance.food_production += output,
                ResourceType::Forest => balance.timber_production += output,
                ResourceType::Ore => balance.metal_production += output,
                ResourceType::Oil => balance.fuel_production += output,
            }

            // Deplete finite resources; regenerate renewable ones
            let deposit = resource_grid.get_mut(building.grid_x, building.grid_y);
            if let Some(ref mut d) = deposit {
                if d.resource_type.is_renewable() {
                    // Renewable resources slowly regenerate (but extraction draws down)
                    let extraction = (output * 0.1) as u32;
                    d.amount = d.amount.saturating_sub(extraction);
                    // Regenerate a small amount each tick
                    d.amount = (d.amount + 1).min(d.max_amount);
                } else {
                    // Finite resources deplete permanently
                    let extraction = (output * 0.2) as u32;
                    d.amount = d.amount.saturating_sub(extraction.max(1));
                }
            }
        }
    }

    // Consumption based on population
    let pop = stats.population as f32;
    balance.food_consumption = pop * 0.02; // Each citizen needs food
    balance.timber_consumption = pop * 0.005; // Construction materials
    balance.metal_consumption = pop * 0.003; // Manufactured goods
    balance.fuel_consumption = pop * 0.004; // Energy supplement
}
