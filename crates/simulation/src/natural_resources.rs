use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    FertileLand,
    Forest,
    Ore,
    Oil,
}

impl ResourceType {
    pub fn is_renewable(self) -> bool {
        matches!(self, ResourceType::FertileLand | ResourceType::Forest)
    }
    pub fn name(self) -> &'static str {
        match self {
            ResourceType::FertileLand => "Fertile Land",
            ResourceType::Forest => "Forest",
            ResourceType::Ore => "Ore Deposit",
            ResourceType::Oil => "Oil Deposit",
        }
    }
}

/// Grid of natural resource deposits, generated alongside terrain
#[derive(Resource)]
pub struct ResourceGrid {
    pub deposits: Vec<Option<ResourceDeposit>>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDeposit {
    pub resource_type: ResourceType,
    pub amount: u32, // Remaining amount (finite resources deplete)
    pub max_amount: u32,
}

impl Default for ResourceGrid {
    fn default() -> Self {
        Self {
            deposits: vec![None; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl ResourceGrid {
    pub fn get(&self, x: usize, y: usize) -> &Option<ResourceDeposit> {
        &self.deposits[y * self.width + x]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Option<ResourceDeposit> {
        &mut self.deposits[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, deposit: ResourceDeposit) {
        self.deposits[y * self.width + x] = Some(deposit);
    }
}

/// Tracks city-wide resource production and consumption
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceBalance {
    pub food_production: f32,
    pub food_consumption: f32,
    pub timber_production: f32,
    pub timber_consumption: f32,
    pub metal_production: f32,
    pub metal_consumption: f32,
    pub fuel_production: f32,
    pub fuel_consumption: f32,
}

impl ResourceBalance {
    pub fn surplus(&self, resource: ResourceType) -> f32 {
        match resource {
            ResourceType::FertileLand => self.food_production - self.food_consumption,
            ResourceType::Forest => self.timber_production - self.timber_consumption,
            ResourceType::Ore => self.metal_production - self.metal_consumption,
            ResourceType::Oil => self.fuel_production - self.fuel_consumption,
        }
    }

    /// Trade income/cost from surplus/deficit. Surplus = export income, deficit = import cost
    pub fn trade_balance(&self) -> f64 {
        let mut balance = 0.0f64;
        for &rt in &[
            ResourceType::FertileLand,
            ResourceType::Forest,
            ResourceType::Ore,
            ResourceType::Oil,
        ] {
            let surplus = self.surplus(rt);
            if surplus > 0.0 {
                balance += surplus as f64 * 3.0; // Export income per unit
            } else {
                balance += surplus as f64 * 5.0; // Import cost per unit (more expensive)
            }
        }
        balance
    }
}

/// Generate resource deposits based on terrain elevation and noise
pub fn generate_resources(grid: &mut ResourceGrid, elevation: &[f32], seed: u32) {
    let width = grid.width;
    let height = grid.height;

    for y in 0..height {
        for x in 0..width {
            let elev = elevation[y * width + x];
            // Simple deterministic placement based on position hash + elevation
            let hash =
                (x.wrapping_mul(seed as usize + 7) ^ y.wrapping_mul(seed as usize + 13)) % 1000;

            if elev < 0.35 {
                continue; // Water - no resources
            }

            let deposit = if elev < 0.45 && hash < 30 {
                // Low elevation near water = fertile land
                Some(ResourceDeposit {
                    resource_type: ResourceType::FertileLand,
                    amount: 10000,
                    max_amount: 10000,
                })
            } else if elev > 0.45 && elev < 0.6 && hash < 25 {
                // Mid elevation = forest
                Some(ResourceDeposit {
                    resource_type: ResourceType::Forest,
                    amount: 8000,
                    max_amount: 8000,
                })
            } else if elev > 0.65 && hash < 15 {
                // High elevation = ore
                Some(ResourceDeposit {
                    resource_type: ResourceType::Ore,
                    amount: 5000,
                    max_amount: 5000,
                })
            } else if elev > 0.5 && elev < 0.65 && hash < 8 {
                // Mid-high = oil (rare)
                Some(ResourceDeposit {
                    resource_type: ResourceType::Oil,
                    amount: 3000,
                    max_amount: 3000,
                })
            } else {
                None
            };

            if let Some(d) = deposit {
                grid.set(x, y, d);
            }
        }
    }
}

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
