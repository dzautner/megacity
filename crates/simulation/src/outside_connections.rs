use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::immigration::CityAttractiveness;
use crate::natural_resources::ResourceBalance;
use crate::production::{CityGoods, GoodsType};
use crate::services::{ServiceBuilding, ServiceType};
use crate::stats::CityStats;
use crate::tourism::Tourism;
use crate::TickCounter;

// =============================================================================
// Types
// =============================================================================

/// The kind of outside connection linking the city to the wider world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionType {
    Highway,
    Railway,
    SeaPort,
    Airport,
}

impl ConnectionType {
    pub fn name(self) -> &'static str {
        match self {
            ConnectionType::Highway => "Highway",
            ConnectionType::Railway => "Railway",
            ConnectionType::SeaPort => "Sea Port",
            ConnectionType::Airport => "Airport",
        }
    }

    /// All connection types, useful for iteration.
    pub fn all() -> &'static [ConnectionType] {
        &[
            ConnectionType::Highway,
            ConnectionType::Railway,
            ConnectionType::SeaPort,
            ConnectionType::Airport,
        ]
    }
}

/// A single outside connection point on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutsideConnection {
    pub connection_type: ConnectionType,
    pub grid_x: usize,
    pub grid_y: usize,
    /// Maximum throughput in vehicles (or equivalent units) per day.
    pub capacity: u32,
    /// Current utilization as a fraction 0.0..1.0.
    pub utilization: f32,
}

/// City-wide resource tracking all connections to the outside world.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutsideConnections {
    pub connections: Vec<OutsideConnection>,
}

impl OutsideConnections {
    /// Check if a given connection type is active (at least one connection exists).
    pub fn has_connection(&self, ct: ConnectionType) -> bool {
        self.connections.iter().any(|c| c.connection_type == ct)
    }

    /// Count the number of connections of a given type.
    pub fn count(&self, ct: ConnectionType) -> usize {
        self.connections
            .iter()
            .filter(|c| c.connection_type == ct)
            .count()
    }

    /// Average utilization across all connections of a given type.
    /// Returns 0.0 if no connections of that type exist.
    pub fn avg_utilization(&self, ct: ConnectionType) -> f32 {
        let conns: Vec<&OutsideConnection> = self
            .connections
            .iter()
            .filter(|c| c.connection_type == ct)
            .collect();
        if conns.is_empty() {
            return 0.0;
        }
        let total: f32 = conns.iter().map(|c| c.utilization).sum();
        total / conns.len() as f32
    }

    /// Human-readable effect description for a connection type.
    pub fn effect_description(ct: ConnectionType) -> &'static str {
        match ct {
            ConnectionType::Highway => "Trade import/export, +20% immigration, adds traffic",
            ConnectionType::Railway => "Bulk goods transport, cheaper imports, +10 tourism",
            ConnectionType::SeaPort => "Heavy cargo (oil/ore at 50% cost), boosts industry",
            ConnectionType::Airport => "Tourism +30, high-value exports, boosts office/tech",
        }
    }
}

// =============================================================================
// Connection effects
// =============================================================================

/// Summary of effects applied by active outside connections this tick.
#[derive(Debug, Clone, Default)]
pub struct ConnectionEffects {
    /// Multiplier for import costs (1.0 = normal, <1.0 = cheaper).
    pub import_cost_multiplier: f32,
    /// Additive tourism bonus.
    pub tourism_bonus: f32,
    /// Additive attractiveness bonus (applied to overall score).
    pub attractiveness_bonus: f32,
    /// Multiplier for immigration rate (1.0 = normal).
    pub immigration_multiplier: f32,
    /// Multiplier for industrial production (1.0 = normal).
    pub industrial_production_bonus: f32,
    /// Multiplier for export prices (1.0 = normal, >1.0 = higher value).
    pub export_price_multiplier: f32,
}

impl ConnectionEffects {
    fn compute(connections: &OutsideConnections) -> Self {
        let mut effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        // Highway: enables trade import/export, boosts immigration by 20%, adds traffic
        if connections.has_connection(ConnectionType::Highway) {
            effects.immigration_multiplier += 0.20;
            effects.attractiveness_bonus += 5.0;
        }

        // Railway: enables bulk goods transport (cheaper imports), boosts tourism +10
        if connections.has_connection(ConnectionType::Railway) {
            effects.import_cost_multiplier *= 0.85; // 15% cheaper imports
            effects.tourism_bonus += 10.0;
            effects.attractiveness_bonus += 3.0;
        }

        // SeaPort: enables heavy cargo (oil, ore imports at 50% cost), boosts industrial
        if connections.has_connection(ConnectionType::SeaPort) {
            effects.import_cost_multiplier *= 0.50; // 50% cheaper for heavy goods
            effects.industrial_production_bonus += 0.15;
            effects.attractiveness_bonus += 4.0;
        }

        // Airport: massive tourism boost (+30), high-value exports, boosts office/tech
        if connections.has_connection(ConnectionType::Airport) {
            effects.tourism_bonus += 30.0;
            effects.export_price_multiplier += 0.20; // 20% higher export prices
            effects.attractiveness_bonus += 8.0;
        }

        effects
    }
}

// =============================================================================
// Detection helpers
// =============================================================================

/// Cells within this distance of the map boundary count as "edge" cells.
const EDGE_PROXIMITY: usize = 3;

/// Check if a grid coordinate is near the map edge.
fn is_near_edge(x: usize, y: usize) -> bool {
    x < EDGE_PROXIMITY
        || x >= GRID_WIDTH - EDGE_PROXIMITY
        || y < EDGE_PROXIMITY
        || y >= GRID_HEIGHT - EDGE_PROXIMITY
}

/// Check if a grid coordinate is near a water edge (water cell within EDGE_PROXIMITY of map boundary).
fn is_near_water_edge(x: usize, y: usize, grid: &WorldGrid) -> bool {
    if !is_near_edge(x, y) {
        return false;
    }
    // Check if there's water nearby (within 5 cells)
    let search = 5isize;
    for dy in -search..=search {
        for dx in -search..=search {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx >= 0
                && ny >= 0
                && (nx as usize) < GRID_WIDTH
                && (ny as usize) < GRID_HEIGHT
            {
                if grid.get(nx as usize, ny as usize).cell_type == CellType::Water {
                    return true;
                }
            }
        }
    }
    false
}

/// Detect highway connections: road cells of type Highway at the map edge.
fn detect_highway_connections(grid: &WorldGrid) -> Vec<OutsideConnection> {
    let mut connections = Vec::new();
    let mut found_positions = Vec::new();

    // Check all four edges
    for x in 0..GRID_WIDTH {
        for &y in &[0usize, 1, 2, GRID_HEIGHT - 3, GRID_HEIGHT - 2, GRID_HEIGHT - 1] {
            if y >= GRID_HEIGHT {
                continue;
            }
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Road
                && matches!(cell.road_type, RoadType::Highway | RoadType::Boulevard)
            {
                // Avoid duplicate connections for the same road (cluster nearby cells)
                let too_close = found_positions
                    .iter()
                    .any(|&(fx, fy): &(usize, usize)| x.abs_diff(fx) + y.abs_diff(fy) < 10);
                if !too_close {
                    found_positions.push((x, y));
                    connections.push(OutsideConnection {
                        connection_type: ConnectionType::Highway,
                        grid_x: x,
                        grid_y: y,
                        capacity: 5000,
                        utilization: 0.0,
                    });
                }
            }
        }
    }

    for y in 0..GRID_HEIGHT {
        for &x in &[0usize, 1, 2, GRID_WIDTH - 3, GRID_WIDTH - 2, GRID_WIDTH - 1] {
            if x >= GRID_WIDTH {
                continue;
            }
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Road
                && matches!(cell.road_type, RoadType::Highway | RoadType::Boulevard)
            {
                let too_close = found_positions
                    .iter()
                    .any(|&(fx, fy): &(usize, usize)| x.abs_diff(fx) + y.abs_diff(fy) < 10);
                if !too_close {
                    found_positions.push((x, y));
                    connections.push(OutsideConnection {
                        connection_type: ConnectionType::Highway,
                        grid_x: x,
                        grid_y: y,
                        capacity: 5000,
                        utilization: 0.0,
                    });
                }
            }
        }
    }

    connections
}

/// Detect railway connections from TrainStation service buildings near map edge.
fn detect_railway_connections(services: &[(&ServiceBuilding,)]) -> Vec<OutsideConnection> {
    let mut connections = Vec::new();
    for (service,) in services {
        if service.service_type == ServiceType::TrainStation
            && is_near_edge(service.grid_x, service.grid_y)
        {
            connections.push(OutsideConnection {
                connection_type: ConnectionType::Railway,
                grid_x: service.grid_x,
                grid_y: service.grid_y,
                capacity: 2000,
                utilization: 0.0,
            });
        }
    }
    connections
}

/// Detect sea port connections from FerryPier service buildings near water edge.
fn detect_seaport_connections(
    services: &[(&ServiceBuilding,)],
    grid: &WorldGrid,
) -> Vec<OutsideConnection> {
    let mut connections = Vec::new();
    for (service,) in services {
        if service.service_type == ServiceType::FerryPier
            && is_near_water_edge(service.grid_x, service.grid_y, grid)
        {
            connections.push(OutsideConnection {
                connection_type: ConnectionType::SeaPort,
                grid_x: service.grid_x,
                grid_y: service.grid_y,
                capacity: 3000,
                utilization: 0.0,
            });
        }
    }
    connections
}

/// Detect airport connections from SmallAirstrip, RegionalAirport, or InternationalAirport service buildings.
fn detect_airport_connections(services: &[(&ServiceBuilding,)]) -> Vec<OutsideConnection> {
    let mut connections = Vec::new();
    for (service,) in services {
        match service.service_type {
            ServiceType::SmallAirstrip => {
                connections.push(OutsideConnection {
                    connection_type: ConnectionType::Airport,
                    grid_x: service.grid_x,
                    grid_y: service.grid_y,
                    capacity: 1000,
                    utilization: 0.0,
                });
            }
            ServiceType::RegionalAirport => {
                connections.push(OutsideConnection {
                    connection_type: ConnectionType::Airport,
                    grid_x: service.grid_x,
                    grid_y: service.grid_y,
                    capacity: 3000,
                    utilization: 0.0,
                });
            }
            ServiceType::InternationalAirport => {
                connections.push(OutsideConnection {
                    connection_type: ConnectionType::Airport,
                    grid_x: service.grid_x,
                    grid_y: service.grid_y,
                    capacity: 5000,
                    utilization: 0.0,
                });
            }
            _ => {}
        }
    }
    connections
}

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

// =============================================================================
// Connection stats for UI
// =============================================================================

/// Summary for the UI panel.
pub struct ConnectionStat {
    pub connection_type: ConnectionType,
    pub active: bool,
    pub count: usize,
    pub avg_utilization: f32,
    pub effect_description: &'static str,
}

impl OutsideConnections {
    /// Generate UI-friendly stats for all connection types.
    pub fn stats(&self) -> Vec<ConnectionStat> {
        ConnectionType::all()
            .iter()
            .map(|&ct| ConnectionStat {
                connection_type: ct,
                active: self.has_connection(ct),
                count: self.count(ct),
                avg_utilization: self.avg_utilization(ct),
                effect_description: Self::effect_description(ct),
            })
            .collect()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_has_no_connections() {
        let outside = OutsideConnections::default();
        assert!(outside.connections.is_empty());
        assert!(!outside.has_connection(ConnectionType::Highway));
        assert!(!outside.has_connection(ConnectionType::Railway));
        assert!(!outside.has_connection(ConnectionType::SeaPort));
        assert!(!outside.has_connection(ConnectionType::Airport));
        assert_eq!(outside.count(ConnectionType::Highway), 0);
        assert_eq!(outside.avg_utilization(ConnectionType::Highway), 0.0);
    }

    #[test]
    fn test_highway_detection_at_map_edges() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

        // Place a highway road cell at the south edge (y=0)
        {
            let cell = grid.get_mut(185, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }

        // Place a highway road cell at the north edge (y=255)
        {
            let cell = grid.get_mut(185, GRID_HEIGHT - 1);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }

        // Place a highway road cell NOT at the edge (should NOT be detected)
        {
            let cell = grid.get_mut(100, 128);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }

        let connections = detect_highway_connections(&grid);
        // Should detect the two edge highways but not the interior one
        assert_eq!(connections.len(), 2);
        assert!(connections.iter().all(|c| c.connection_type == ConnectionType::Highway));

        // Verify positions
        let positions: Vec<(usize, usize)> = connections.iter().map(|c| (c.grid_x, c.grid_y)).collect();
        assert!(positions.contains(&(185, 0)));
        assert!(positions.contains(&(185, GRID_HEIGHT - 1)));
    }

    #[test]
    fn test_connection_effects_computed_correctly() {
        // No connections -> no effects
        let empty = OutsideConnections::default();
        let effects = ConnectionEffects::compute(&empty);
        assert!((effects.import_cost_multiplier - 1.0).abs() < 0.001);
        assert!((effects.tourism_bonus - 0.0).abs() < 0.001);
        assert!((effects.immigration_multiplier - 1.0).abs() < 0.001);
        assert!((effects.industrial_production_bonus - 1.0).abs() < 0.001);
        assert!((effects.export_price_multiplier - 1.0).abs() < 0.001);

        // Highway only
        let mut with_highway = OutsideConnections::default();
        with_highway.connections.push(OutsideConnection {
            connection_type: ConnectionType::Highway,
            grid_x: 185,
            grid_y: 0,
            capacity: 5000,
            utilization: 0.5,
        });
        let effects = ConnectionEffects::compute(&with_highway);
        assert!((effects.immigration_multiplier - 1.20).abs() < 0.001);
        assert!((effects.attractiveness_bonus - 5.0).abs() < 0.001);

        // Airport only
        let mut with_airport = OutsideConnections::default();
        with_airport.connections.push(OutsideConnection {
            connection_type: ConnectionType::Airport,
            grid_x: 100,
            grid_y: 100,
            capacity: 5000,
            utilization: 0.3,
        });
        let effects = ConnectionEffects::compute(&with_airport);
        assert!((effects.tourism_bonus - 30.0).abs() < 0.001);
        assert!((effects.export_price_multiplier - 1.20).abs() < 0.001);
        assert!((effects.attractiveness_bonus - 8.0).abs() < 0.001);

        // Railway + SeaPort combined
        let mut with_rail_port = OutsideConnections::default();
        with_rail_port.connections.push(OutsideConnection {
            connection_type: ConnectionType::Railway,
            grid_x: 145,
            grid_y: 0,
            capacity: 2000,
            utilization: 0.4,
        });
        with_rail_port.connections.push(OutsideConnection {
            connection_type: ConnectionType::SeaPort,
            grid_x: 55,
            grid_y: 50,
            capacity: 3000,
            utilization: 0.2,
        });
        let effects = ConnectionEffects::compute(&with_rail_port);
        // Railway: 0.85 * SeaPort: 0.50 = 0.425
        assert!((effects.import_cost_multiplier - 0.425).abs() < 0.001);
        assert!((effects.tourism_bonus - 10.0).abs() < 0.001);
        // Railway: 3.0 + SeaPort: 4.0 = 7.0
        assert!((effects.attractiveness_bonus - 7.0).abs() < 0.001);
        // SeaPort: 1.0 + 0.15 = 1.15
        assert!((effects.industrial_production_bonus - 1.15).abs() < 0.001);
    }

    #[test]
    fn test_all_connection_types() {
        let all = ConnectionType::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&ConnectionType::Highway));
        assert!(all.contains(&ConnectionType::Railway));
        assert!(all.contains(&ConnectionType::SeaPort));
        assert!(all.contains(&ConnectionType::Airport));
    }

    #[test]
    fn test_connection_stats_summary() {
        let mut outside = OutsideConnections::default();
        outside.connections.push(OutsideConnection {
            connection_type: ConnectionType::Highway,
            grid_x: 185,
            grid_y: 0,
            capacity: 5000,
            utilization: 0.6,
        });
        outside.connections.push(OutsideConnection {
            connection_type: ConnectionType::Highway,
            grid_x: 185,
            grid_y: 255,
            capacity: 5000,
            utilization: 0.4,
        });

        let stats = outside.stats();
        assert_eq!(stats.len(), 4);

        let highway_stat = stats.iter().find(|s| s.connection_type == ConnectionType::Highway).unwrap();
        assert!(highway_stat.active);
        assert_eq!(highway_stat.count, 2);
        assert!((highway_stat.avg_utilization - 0.5).abs() < 0.001);

        let railway_stat = stats.iter().find(|s| s.connection_type == ConnectionType::Railway).unwrap();
        assert!(!railway_stat.active);
        assert_eq!(railway_stat.count, 0);
        assert_eq!(railway_stat.avg_utilization, 0.0);
    }

    #[test]
    fn test_is_near_edge() {
        // Corners and edges
        assert!(is_near_edge(0, 0));
        assert!(is_near_edge(1, 1));
        assert!(is_near_edge(2, 128));
        assert!(is_near_edge(128, 0));
        assert!(is_near_edge(GRID_WIDTH - 1, 128));
        assert!(is_near_edge(128, GRID_HEIGHT - 1));

        // Interior
        assert!(!is_near_edge(128, 128));
        assert!(!is_near_edge(50, 50));
        assert!(!is_near_edge(GRID_WIDTH / 2, GRID_HEIGHT / 2));
    }

    #[test]
    fn test_effect_descriptions_exist() {
        for &ct in ConnectionType::all() {
            let desc = OutsideConnections::effect_description(ct);
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn test_connection_type_names() {
        assert_eq!(ConnectionType::Highway.name(), "Highway");
        assert_eq!(ConnectionType::Railway.name(), "Railway");
        assert_eq!(ConnectionType::SeaPort.name(), "Sea Port");
        assert_eq!(ConnectionType::Airport.name(), "Airport");
    }
}
