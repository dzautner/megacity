use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::economy::CityBudget;
use crate::grid::ZoneType;
use crate::time_of_day::GameClock;

/// Trade connections at map edges
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct TradeConnections {
    pub export_income_per_industrial: f64,
    pub import_cost_per_commercial: f64,
    pub last_trade_day: u32,
}

impl Default for TradeConnections {
    fn default() -> Self {
        Self {
            export_income_per_industrial: 2.0,
            import_cost_per_commercial: 1.0,
            last_trade_day: 0,
        }
    }
}

pub fn process_trade(
    clock: Res<GameClock>,
    mut trade: ResMut<TradeConnections>,
    mut budget: ResMut<CityBudget>,
    buildings: Query<&Building>,
) {
    // Process every 30 days
    if clock.day <= trade.last_trade_day + 30 {
        return;
    }
    trade.last_trade_day = clock.day;

    let industrial_count = buildings
        .iter()
        .filter(|b| b.zone_type == ZoneType::Industrial && b.occupants > 0)
        .count() as f64;

    let commercial_count = buildings
        .iter()
        .filter(|b| (b.zone_type.is_commercial() || b.zone_type.is_mixed_use()) && b.occupants > 0)
        .count() as f64;

    let export_income = industrial_count * trade.export_income_per_industrial;
    let import_cost = commercial_count * trade.import_cost_per_commercial;

    budget.treasury += export_income - import_cost;
}

pub struct ImportsExportsPlugin;

impl Plugin for ImportsExportsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TradeConnections>().add_systems(
            FixedUpdate,
            process_trade
                .after(crate::building_upgrade::downgrade_buildings)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
