use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::types::ResourceType;

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
