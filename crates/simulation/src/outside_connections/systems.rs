use bevy::prelude::*;

use crate::grid::WorldGrid;
use crate::immigration::CityAttractiveness;
use crate::natural_resources::ResourceBalance;
use crate::production::{CityGoods, GoodsType};
use crate::services::ServiceBuilding;
use crate::stats::CityStats;
use crate::tourism::Tourism;
use crate::TickCounter;

use super::detection::{
    detect_airport_connections, detect_highway_connections, detect_railway_connections,
    detect_seaport_connections,
};
use super::effects::ConnectionEffects;
use super::types::{ConnectionType, OutsideConnections};

// =============================================================================
// System
// =============================================================================

/// Update interval in ticks.
const UPDATE_INTERVAL: u64 = 100;

/// Main system: detect outside connections and apply their effects.
///
/// Runs every 100 ticks. Scans for:
/// - Highway/boulevard road cells at map edges
/// - TrainStation near map edge -> Railway
/// - FerryPier near water edge -> SeaPort
/// - SmallAirstrip/InternationalAirport -> Airport
///
/// Then computes utilization and applies economic effects.
#[allow(clippy::too_many_arguments)]
pub fn update_outside_connections(
    tick: Res<TickCounter>,
    grid: Res<WorldGrid>,
    services: Query<&ServiceBuilding>,
    stats: Res<CityStats>,
    mut outside: ResMut<OutsideConnections>,
    mut tourism: ResMut<Tourism>,
    mut attractiveness: ResMut<CityAttractiveness>,
    mut resource_balance: ResMut<ResourceBalance>,
    mut city_goods: ResMut<CityGoods>,
) {
    if !tick.0.is_multiple_of(UPDATE_INTERVAL) {
        return;
    }

    // -------------------------------------------------------------------------
    // 1. Detect connections
    // -------------------------------------------------------------------------
    let service_list: Vec<(&ServiceBuilding,)> = services.iter().map(|s| (s,)).collect();

    let mut all_connections = Vec::new();
    all_connections.extend(detect_highway_connections(&grid));
    all_connections.extend(detect_railway_connections(&service_list));
    all_connections.extend(detect_seaport_connections(&service_list, &grid));
    all_connections.extend(detect_airport_connections(&service_list));

    // -------------------------------------------------------------------------
    // 2. Compute utilization based on population and trade volume
    // -------------------------------------------------------------------------
    let pop = stats.population as f32;
    let trade_volume = city_goods.trade_balance.abs() as f32;

    for conn in &mut all_connections {
        let base_utilization = match conn.connection_type {
            ConnectionType::Highway => {
                // Utilization based on population and trade
                let pop_factor = (pop / 50_000.0).min(0.6);
                let trade_factor = (trade_volume / 100.0).min(0.4);
                pop_factor + trade_factor
            }
            ConnectionType::Railway => {
                // Utilization based on industrial production and population
                let industrial_goods: f32 = GoodsType::all()
                    .iter()
                    .map(|g| city_goods.production_rate.get(g).copied().unwrap_or(0.0))
                    .sum();
                let prod_factor = (industrial_goods / 50.0).min(0.5);
                let pop_factor = (pop / 80_000.0).min(0.5);
                prod_factor + pop_factor
            }
            ConnectionType::SeaPort => {
                // Utilization based on heavy goods trade (fuel, steel)
                let fuel_rate = city_goods
                    .production_rate
                    .get(&GoodsType::Fuel)
                    .copied()
                    .unwrap_or(0.0);
                let steel_rate = city_goods
                    .production_rate
                    .get(&GoodsType::Steel)
                    .copied()
                    .unwrap_or(0.0);
                let heavy_factor = ((fuel_rate + steel_rate) / 30.0).min(0.6);
                let pop_factor = (pop / 100_000.0).min(0.4);
                heavy_factor + pop_factor
            }
            ConnectionType::Airport => {
                // Utilization based on tourism and population
                let tourism_factor = (tourism.monthly_visitors as f32 / 5000.0).min(0.5);
                let pop_factor = (pop / 60_000.0).min(0.5);
                tourism_factor + pop_factor
            }
        };

        conn.utilization = base_utilization.clamp(0.0, 1.0);
    }

    outside.connections = all_connections;

    // -------------------------------------------------------------------------
    // 3. Compute and apply effects
    // -------------------------------------------------------------------------
    let effects = ConnectionEffects::compute(&outside);

    // Apply tourism bonus
    tourism.attractiveness = (tourism.attractiveness + effects.tourism_bonus).min(100.0);
    tourism.monthly_visitors = (tourism.attractiveness * 50.0) as u32;

    // Apply attractiveness bonus
    attractiveness.overall_score =
        (attractiveness.overall_score + effects.attractiveness_bonus).clamp(0.0, 100.0);

    // Apply import cost reduction to resource balance trade calculations
    // Modify the consumption rates to simulate cheaper imports
    if effects.import_cost_multiplier < 1.0 {
        // Reduce effective consumption costs by adjusting fuel/metal consumption
        // (simulates cheaper imports reducing the cost burden)
        let reduction = 1.0 - effects.import_cost_multiplier;
        resource_balance.fuel_consumption *= 1.0 - reduction * 0.3;
        resource_balance.metal_consumption *= 1.0 - reduction * 0.3;
    }

    // Apply industrial production bonus
    if effects.industrial_production_bonus > 1.0 {
        let bonus = effects.industrial_production_bonus - 1.0;
        resource_balance.food_production *= 1.0 + bonus;
        resource_balance.timber_production *= 1.0 + bonus;
        resource_balance.metal_production *= 1.0 + bonus;
        resource_balance.fuel_production *= 1.0 + bonus;
    }

    // Apply export price multiplier to trade balance
    if effects.export_price_multiplier > 1.0 {
        let bonus_factor = effects.export_price_multiplier;
        city_goods.trade_balance *= bonus_factor as f64;
    }
}
