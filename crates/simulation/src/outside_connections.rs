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
    !(EDGE_PROXIMITY..GRID_WIDTH - EDGE_PROXIMITY).contains(&x)
        || !(EDGE_PROXIMITY..GRID_HEIGHT - EDGE_PROXIMITY).contains(&y)
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
                && grid.get(nx as usize, ny as usize).cell_type == CellType::Water
            {
                return true;
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
        for &y in &[
            0usize,
            1,
            2,
            GRID_HEIGHT - 3,
            GRID_HEIGHT - 2,
            GRID_HEIGHT - 1,
        ] {
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

    // =========================================================================
    // Helper: create an OutsideConnection with given type, position, capacity
    // =========================================================================
    fn make_conn(
        ct: ConnectionType,
        x: usize,
        y: usize,
        capacity: u32,
        utilization: f32,
    ) -> OutsideConnection {
        OutsideConnection {
            connection_type: ct,
            grid_x: x,
            grid_y: y,
            capacity,
            utilization,
        }
    }

    // =========================================================================
    // 1. Default / basic state tests
    // =========================================================================

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
    fn test_all_connection_types() {
        let all = ConnectionType::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&ConnectionType::Highway));
        assert!(all.contains(&ConnectionType::Railway));
        assert!(all.contains(&ConnectionType::SeaPort));
        assert!(all.contains(&ConnectionType::Airport));
    }

    #[test]
    fn test_connection_type_names() {
        assert_eq!(ConnectionType::Highway.name(), "Highway");
        assert_eq!(ConnectionType::Railway.name(), "Railway");
        assert_eq!(ConnectionType::SeaPort.name(), "Sea Port");
        assert_eq!(ConnectionType::Airport.name(), "Airport");
    }

    #[test]
    fn test_effect_descriptions_exist() {
        for &ct in ConnectionType::all() {
            let desc = OutsideConnections::effect_description(ct);
            assert!(!desc.is_empty());
        }
    }

    // =========================================================================
    // 2. has_connection / count / avg_utilization queries
    // =========================================================================

    #[test]
    fn test_has_connection_returns_true_when_present() {
        let mut outside = OutsideConnections::default();
        outside
            .connections
            .push(make_conn(ConnectionType::Railway, 10, 0, 2000, 0.3));
        assert!(outside.has_connection(ConnectionType::Railway));
        assert!(!outside.has_connection(ConnectionType::Highway));
    }

    #[test]
    fn test_count_multiple_connections_of_same_type() {
        let mut outside = OutsideConnections::default();
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 50, 0, 5000, 0.2));
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 200, 0, 5000, 0.8));
        outside
            .connections
            .push(make_conn(ConnectionType::Railway, 10, 0, 2000, 0.5));
        assert_eq!(outside.count(ConnectionType::Highway), 2);
        assert_eq!(outside.count(ConnectionType::Railway), 1);
        assert_eq!(outside.count(ConnectionType::Airport), 0);
    }

    #[test]
    fn test_avg_utilization_single_connection() {
        let mut outside = OutsideConnections::default();
        outside
            .connections
            .push(make_conn(ConnectionType::Airport, 100, 100, 5000, 0.75));
        assert!((outside.avg_utilization(ConnectionType::Airport) - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_avg_utilization_multiple_connections() {
        let mut outside = OutsideConnections::default();
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 50, 0, 5000, 0.2));
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 200, 0, 5000, 0.8));
        // Average of 0.2 and 0.8 = 0.5
        assert!((outside.avg_utilization(ConnectionType::Highway) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_avg_utilization_no_connections_returns_zero() {
        let outside = OutsideConnections::default();
        assert_eq!(outside.avg_utilization(ConnectionType::SeaPort), 0.0);
    }

    // =========================================================================
    // 3. Edge detection helpers
    // =========================================================================

    #[test]
    fn test_is_near_edge() {
        // Corners and edges (within EDGE_PROXIMITY=3)
        assert!(is_near_edge(0, 0));
        assert!(is_near_edge(1, 1));
        assert!(is_near_edge(2, 128));
        assert!(is_near_edge(128, 0));
        assert!(is_near_edge(GRID_WIDTH - 1, 128));
        assert!(is_near_edge(128, GRID_HEIGHT - 1));

        // Boundary: exactly at EDGE_PROXIMITY
        assert!(!is_near_edge(EDGE_PROXIMITY, EDGE_PROXIMITY));

        // Interior
        assert!(!is_near_edge(128, 128));
        assert!(!is_near_edge(50, 50));
        assert!(!is_near_edge(GRID_WIDTH / 2, GRID_HEIGHT / 2));
    }

    #[test]
    fn test_is_near_edge_boundary_values() {
        // x=2 is within EDGE_PROXIMITY=3 (range check: !(3..253).contains(&2) => true)
        assert!(is_near_edge(2, 128));
        // x=3 is NOT near edge (range check: !(3..253).contains(&3) => false)
        assert!(!is_near_edge(3, 128));
        // x=GRID_WIDTH-3 = 253 is near edge (range check: !(3..253).contains(&253) => true)
        assert!(is_near_edge(GRID_WIDTH - 3, 128));
        // x=GRID_WIDTH-4 = 252 is NOT near edge
        assert!(!is_near_edge(GRID_WIDTH - 4, 128));
    }

    #[test]
    fn test_is_near_water_edge_no_water() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Edge cell but no water nearby
        assert!(!is_near_water_edge(0, 0, &grid));
        assert!(!is_near_water_edge(128, 0, &grid));
    }

    #[test]
    fn test_is_near_water_edge_with_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place water near the edge
        grid.get_mut(2, 2).cell_type = CellType::Water;
        // Cell at (0,0) is near edge and water is within 5 cells
        assert!(is_near_water_edge(0, 0, &grid));
    }

    #[test]
    fn test_is_near_water_edge_interior_cell_returns_false() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place water at interior
        grid.get_mut(128, 128).cell_type = CellType::Water;
        // Interior cell is not near edge, so returns false even with water
        assert!(!is_near_water_edge(128, 128, &grid));
    }

    // =========================================================================
    // 4. Highway detection
    // =========================================================================

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
        assert_eq!(connections.len(), 2);
        assert!(connections
            .iter()
            .all(|c| c.connection_type == ConnectionType::Highway));

        let positions: Vec<(usize, usize)> =
            connections.iter().map(|c| (c.grid_x, c.grid_y)).collect();
        assert!(positions.contains(&(185, 0)));
        assert!(positions.contains(&(185, GRID_HEIGHT - 1)));
    }

    #[test]
    fn test_boulevard_detected_as_highway_connection() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        {
            let cell = grid.get_mut(100, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Boulevard;
        }
        let connections = detect_highway_connections(&grid);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].connection_type, ConnectionType::Highway);
        assert_eq!(connections[0].grid_x, 100);
        assert_eq!(connections[0].grid_y, 0);
    }

    #[test]
    fn test_highway_detection_left_edge() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        {
            let cell = grid.get_mut(0, 128);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let connections = detect_highway_connections(&grid);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].grid_x, 0);
        assert_eq!(connections[0].grid_y, 128);
    }

    #[test]
    fn test_highway_detection_right_edge() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        {
            let cell = grid.get_mut(GRID_WIDTH - 1, 128);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let connections = detect_highway_connections(&grid);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].grid_x, GRID_WIDTH - 1);
        assert_eq!(connections[0].grid_y, 128);
    }

    #[test]
    fn test_highway_clustering_avoids_duplicates() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place two highway cells close together at south edge (within 10 Manhattan distance)
        for x in 50..55 {
            let cell = grid.get_mut(x, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let connections = detect_highway_connections(&grid);
        // Should cluster into 1 connection, not 5
        assert_eq!(connections.len(), 1);
    }

    #[test]
    fn test_highway_two_distant_clusters_detected_separately() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Two highway cells far apart on the same edge (>10 apart)
        {
            let cell = grid.get_mut(20, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        {
            let cell = grid.get_mut(100, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let connections = detect_highway_connections(&grid);
        assert_eq!(connections.len(), 2);
    }

    #[test]
    fn test_highway_capacity_is_5000() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        {
            let cell = grid.get_mut(100, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let connections = detect_highway_connections(&grid);
        assert_eq!(connections[0].capacity, 5000);
    }

    #[test]
    fn test_highway_initial_utilization_is_zero() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        {
            let cell = grid.get_mut(100, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let connections = detect_highway_connections(&grid);
        assert!((connections[0].utilization - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_non_highway_road_at_edge_not_detected() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Local road at edge should NOT be detected
        {
            let cell = grid.get_mut(100, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Local;
        }
        let connections = detect_highway_connections(&grid);
        assert!(connections.is_empty());
    }

    #[test]
    fn test_empty_grid_no_highway_connections() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let connections = detect_highway_connections(&grid);
        assert!(connections.is_empty());
    }

    // =========================================================================
    // 5. Railway detection
    // =========================================================================

    #[test]
    fn test_railway_detection_from_train_station_near_edge() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::TrainStation,
            grid_x: 1,
            grid_y: 128,
            radius: 50.0,
        },)];
        let connections = detect_railway_connections(&services);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].connection_type, ConnectionType::Railway);
        assert_eq!(connections[0].capacity, 2000);
        assert_eq!(connections[0].grid_x, 1);
        assert_eq!(connections[0].grid_y, 128);
    }

    #[test]
    fn test_railway_not_detected_for_interior_train_station() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::TrainStation,
            grid_x: 128,
            grid_y: 128,
            radius: 50.0,
        },)];
        let connections = detect_railway_connections(&services);
        assert!(connections.is_empty());
    }

    #[test]
    fn test_railway_not_detected_for_non_train_service_at_edge() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::FireStation,
            grid_x: 0,
            grid_y: 128,
            radius: 50.0,
        },)];
        let connections = detect_railway_connections(&services);
        assert!(connections.is_empty());
    }

    #[test]
    fn test_multiple_railway_connections() {
        let services = vec![
            (&ServiceBuilding {
                service_type: ServiceType::TrainStation,
                grid_x: 0,
                grid_y: 50,
                radius: 50.0,
            },),
            (&ServiceBuilding {
                service_type: ServiceType::TrainStation,
                grid_x: GRID_WIDTH - 1,
                grid_y: 200,
                radius: 50.0,
            },),
        ];
        let connections = detect_railway_connections(&services);
        assert_eq!(connections.len(), 2);
    }

    // =========================================================================
    // 6. SeaPort detection
    // =========================================================================

    #[test]
    fn test_seaport_detection_from_ferry_pier_near_water_edge() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place water near the edge
        grid.get_mut(1, 1).cell_type = CellType::Water;

        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::FerryPier,
            grid_x: 0,
            grid_y: 0,
            radius: 30.0,
        },)];
        let connections = detect_seaport_connections(&services, &grid);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].connection_type, ConnectionType::SeaPort);
        assert_eq!(connections[0].capacity, 3000);
    }

    #[test]
    fn test_seaport_not_detected_without_water() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // FerryPier at edge but no water
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::FerryPier,
            grid_x: 0,
            grid_y: 128,
            radius: 30.0,
        },)];
        let connections = detect_seaport_connections(&services, &grid);
        assert!(connections.is_empty());
    }

    #[test]
    fn test_seaport_not_detected_for_interior_ferry_pier() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Water in interior
        grid.get_mut(128, 128).cell_type = CellType::Water;
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::FerryPier,
            grid_x: 128,
            grid_y: 128,
            radius: 30.0,
        },)];
        let connections = detect_seaport_connections(&services, &grid);
        assert!(connections.is_empty());
    }

    // =========================================================================
    // 7. Airport detection
    // =========================================================================

    #[test]
    fn test_airport_detection_small_airstrip() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::SmallAirstrip,
            grid_x: 100,
            grid_y: 100,
            radius: 50.0,
        },)];
        let connections = detect_airport_connections(&services);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].connection_type, ConnectionType::Airport);
        assert_eq!(connections[0].capacity, 1000);
    }

    #[test]
    fn test_airport_detection_regional_airport() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::RegionalAirport,
            grid_x: 80,
            grid_y: 80,
            radius: 80.0,
        },)];
        let connections = detect_airport_connections(&services);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].capacity, 3000);
    }

    #[test]
    fn test_airport_detection_international_airport() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::InternationalAirport,
            grid_x: 60,
            grid_y: 60,
            radius: 120.0,
        },)];
        let connections = detect_airport_connections(&services);
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0].capacity, 5000);
    }

    #[test]
    fn test_airport_not_detected_for_non_airport_service() {
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::Hospital,
            grid_x: 100,
            grid_y: 100,
            radius: 50.0,
        },)];
        let connections = detect_airport_connections(&services);
        assert!(connections.is_empty());
    }

    #[test]
    fn test_multiple_airport_types_detected() {
        let services = vec![
            (&ServiceBuilding {
                service_type: ServiceType::SmallAirstrip,
                grid_x: 30,
                grid_y: 30,
                radius: 50.0,
            },),
            (&ServiceBuilding {
                service_type: ServiceType::InternationalAirport,
                grid_x: 200,
                grid_y: 200,
                radius: 120.0,
            },),
        ];
        let connections = detect_airport_connections(&services);
        assert_eq!(connections.len(), 2);
        // Total capacity: 1000 + 5000 = 6000
        let total_capacity: u32 = connections.iter().map(|c| c.capacity).sum();
        assert_eq!(total_capacity, 6000);
    }

    #[test]
    fn test_airport_detection_does_not_require_edge() {
        // Unlike railway, airports don't need to be near the edge
        let services = vec![(&ServiceBuilding {
            service_type: ServiceType::InternationalAirport,
            grid_x: 128,
            grid_y: 128,
            radius: 120.0,
        },)];
        let connections = detect_airport_connections(&services);
        assert_eq!(connections.len(), 1);
    }

    // =========================================================================
    // 8. Connection capacity limits
    // =========================================================================

    #[test]
    fn test_connection_capacity_values_by_type() {
        // Verify each detection function assigns the correct capacity
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        {
            let cell = grid.get_mut(100, 0);
            cell.cell_type = CellType::Road;
            cell.road_type = RoadType::Highway;
        }
        let highway_conns = detect_highway_connections(&grid);
        assert_eq!(
            highway_conns[0].capacity, 5000,
            "Highway capacity should be 5000"
        );

        let rail_services = vec![(&ServiceBuilding {
            service_type: ServiceType::TrainStation,
            grid_x: 0,
            grid_y: 128,
            radius: 50.0,
        },)];
        let rail_conns = detect_railway_connections(&rail_services);
        assert_eq!(
            rail_conns[0].capacity, 2000,
            "Railway capacity should be 2000"
        );

        let mut water_grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        water_grid.get_mut(1, 1).cell_type = CellType::Water;
        let port_services = vec![(&ServiceBuilding {
            service_type: ServiceType::FerryPier,
            grid_x: 0,
            grid_y: 0,
            radius: 30.0,
        },)];
        let port_conns = detect_seaport_connections(&port_services, &water_grid);
        assert_eq!(
            port_conns[0].capacity, 3000,
            "SeaPort capacity should be 3000"
        );

        let air_services_small = vec![(&ServiceBuilding {
            service_type: ServiceType::SmallAirstrip,
            grid_x: 100,
            grid_y: 100,
            radius: 50.0,
        },)];
        let air_conns = detect_airport_connections(&air_services_small);
        assert_eq!(
            air_conns[0].capacity, 1000,
            "SmallAirstrip capacity should be 1000"
        );

        let air_services_regional = vec![(&ServiceBuilding {
            service_type: ServiceType::RegionalAirport,
            grid_x: 100,
            grid_y: 100,
            radius: 80.0,
        },)];
        let air_conns = detect_airport_connections(&air_services_regional);
        assert_eq!(
            air_conns[0].capacity, 3000,
            "RegionalAirport capacity should be 3000"
        );

        let air_services_intl = vec![(&ServiceBuilding {
            service_type: ServiceType::InternationalAirport,
            grid_x: 100,
            grid_y: 100,
            radius: 120.0,
        },)];
        let air_conns = detect_airport_connections(&air_services_intl);
        assert_eq!(
            air_conns[0].capacity, 5000,
            "InternationalAirport capacity should be 5000"
        );
    }

    // =========================================================================
    // 9. ConnectionEffects computation
    // =========================================================================

    #[test]
    fn test_connection_effects_no_connections() {
        let empty = OutsideConnections::default();
        let effects = ConnectionEffects::compute(&empty);
        assert!((effects.import_cost_multiplier - 1.0).abs() < 0.001);
        assert!((effects.tourism_bonus - 0.0).abs() < 0.001);
        assert!((effects.attractiveness_bonus - 0.0).abs() < 0.001);
        assert!((effects.immigration_multiplier - 1.0).abs() < 0.001);
        assert!((effects.industrial_production_bonus - 1.0).abs() < 0.001);
        assert!((effects.export_price_multiplier - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_connection_effects_highway_only() {
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::Highway, 185, 0, 5000, 0.5));
        let effects = ConnectionEffects::compute(&conns);
        assert!(
            (effects.immigration_multiplier - 1.20).abs() < 0.001,
            "Highway should boost immigration by 20%"
        );
        assert!(
            (effects.attractiveness_bonus - 5.0).abs() < 0.001,
            "Highway should add 5.0 attractiveness"
        );
        // Other effects should remain at default
        assert!((effects.import_cost_multiplier - 1.0).abs() < 0.001);
        assert!((effects.tourism_bonus - 0.0).abs() < 0.001);
        assert!((effects.industrial_production_bonus - 1.0).abs() < 0.001);
        assert!((effects.export_price_multiplier - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_connection_effects_railway_only() {
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::Railway, 145, 0, 2000, 0.4));
        let effects = ConnectionEffects::compute(&conns);
        assert!(
            (effects.import_cost_multiplier - 0.85).abs() < 0.001,
            "Railway should reduce imports by 15%"
        );
        assert!(
            (effects.tourism_bonus - 10.0).abs() < 0.001,
            "Railway should add 10 tourism"
        );
        assert!(
            (effects.attractiveness_bonus - 3.0).abs() < 0.001,
            "Railway should add 3.0 attractiveness"
        );
    }

    #[test]
    fn test_connection_effects_seaport_only() {
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::SeaPort, 55, 0, 3000, 0.2));
        let effects = ConnectionEffects::compute(&conns);
        assert!(
            (effects.import_cost_multiplier - 0.50).abs() < 0.001,
            "SeaPort should halve import costs"
        );
        assert!(
            (effects.industrial_production_bonus - 1.15).abs() < 0.001,
            "SeaPort should boost industrial production by 15%"
        );
        assert!(
            (effects.attractiveness_bonus - 4.0).abs() < 0.001,
            "SeaPort should add 4.0 attractiveness"
        );
    }

    #[test]
    fn test_connection_effects_airport_only() {
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::Airport, 100, 100, 5000, 0.3));
        let effects = ConnectionEffects::compute(&conns);
        assert!(
            (effects.tourism_bonus - 30.0).abs() < 0.001,
            "Airport should add 30 tourism"
        );
        assert!(
            (effects.export_price_multiplier - 1.20).abs() < 0.001,
            "Airport should boost export prices by 20%"
        );
        assert!(
            (effects.attractiveness_bonus - 8.0).abs() < 0.001,
            "Airport should add 8.0 attractiveness"
        );
    }

    #[test]
    fn test_connection_effects_railway_plus_seaport_combined() {
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::Railway, 145, 0, 2000, 0.4));
        conns
            .connections
            .push(make_conn(ConnectionType::SeaPort, 55, 50, 3000, 0.2));
        let effects = ConnectionEffects::compute(&conns);
        // Railway: 0.85 * SeaPort: 0.50 = 0.425
        assert!(
            (effects.import_cost_multiplier - 0.425).abs() < 0.001,
            "Combined import cost should be 0.85 * 0.50 = 0.425"
        );
        // Tourism: Railway 10 only (SeaPort adds none)
        assert!((effects.tourism_bonus - 10.0).abs() < 0.001);
        // Attractiveness: Railway 3 + SeaPort 4 = 7
        assert!((effects.attractiveness_bonus - 7.0).abs() < 0.001);
        // Industrial: SeaPort 1.15 only
        assert!((effects.industrial_production_bonus - 1.15).abs() < 0.001);
    }

    #[test]
    fn test_connection_effects_all_four_types_combined() {
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::Highway, 185, 0, 5000, 0.5));
        conns
            .connections
            .push(make_conn(ConnectionType::Railway, 145, 0, 2000, 0.4));
        conns
            .connections
            .push(make_conn(ConnectionType::SeaPort, 55, 50, 3000, 0.2));
        conns
            .connections
            .push(make_conn(ConnectionType::Airport, 100, 100, 5000, 0.3));

        let effects = ConnectionEffects::compute(&conns);

        // Import cost: Railway 0.85 * SeaPort 0.50 = 0.425
        assert!(
            (effects.import_cost_multiplier - 0.425).abs() < 0.001,
            "All types: import cost should be 0.425"
        );
        // Tourism: Railway 10 + Airport 30 = 40
        assert!(
            (effects.tourism_bonus - 40.0).abs() < 0.001,
            "All types: tourism should be 40"
        );
        // Attractiveness: Highway 5 + Railway 3 + SeaPort 4 + Airport 8 = 20
        assert!(
            (effects.attractiveness_bonus - 20.0).abs() < 0.001,
            "All types: attractiveness should be 20"
        );
        // Immigration: 1.0 + Highway 0.20 = 1.20
        assert!(
            (effects.immigration_multiplier - 1.20).abs() < 0.001,
            "All types: immigration multiplier should be 1.20"
        );
        // Industrial: 1.0 + SeaPort 0.15 = 1.15
        assert!(
            (effects.industrial_production_bonus - 1.15).abs() < 0.001,
            "All types: industrial bonus should be 1.15"
        );
        // Export price: 1.0 + Airport 0.20 = 1.20
        assert!(
            (effects.export_price_multiplier - 1.20).abs() < 0.001,
            "All types: export price multiplier should be 1.20"
        );
    }

    #[test]
    fn test_connection_effects_duplicate_types_only_counted_once() {
        // Having two highways shouldn't double the effect (it's presence-based, not count-based)
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::Highway, 50, 0, 5000, 0.3));
        conns
            .connections
            .push(make_conn(ConnectionType::Highway, 200, 0, 5000, 0.7));
        let effects = ConnectionEffects::compute(&conns);
        // Should still be 1.20 (not 1.40)
        assert!(
            (effects.immigration_multiplier - 1.20).abs() < 0.001,
            "Duplicate highways should not stack effects"
        );
    }

    // =========================================================================
    // 10. Import/export flow — effects on resource balance and trade
    // =========================================================================

    #[test]
    fn test_import_cost_reduction_applied_to_resource_balance() {
        // Simulate the effect application logic from update_outside_connections
        let mut resource_balance = ResourceBalance {
            fuel_consumption: 100.0,
            metal_consumption: 80.0,
            food_production: 50.0,
            food_consumption: 0.0,
            timber_production: 0.0,
            timber_consumption: 0.0,
            metal_production: 0.0,
            fuel_production: 0.0,
        };

        // Railway + SeaPort => import_cost_multiplier = 0.425
        let effects = ConnectionEffects {
            import_cost_multiplier: 0.425,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        // Apply the same logic as the system
        if effects.import_cost_multiplier < 1.0 {
            let reduction = 1.0 - effects.import_cost_multiplier;
            resource_balance.fuel_consumption *= 1.0 - reduction * 0.3;
            resource_balance.metal_consumption *= 1.0 - reduction * 0.3;
        }

        // reduction = 0.575, so multiplier = 1.0 - 0.575*0.3 = 0.8275
        let expected_fuel = 100.0 * (1.0 - 0.575 * 0.3);
        let expected_metal = 80.0 * (1.0 - 0.575 * 0.3);
        assert!(
            (resource_balance.fuel_consumption - expected_fuel).abs() < 0.01,
            "Fuel consumption should be reduced to {expected_fuel}, got {}",
            resource_balance.fuel_consumption
        );
        assert!(
            (resource_balance.metal_consumption - expected_metal).abs() < 0.01,
            "Metal consumption should be reduced to {expected_metal}, got {}",
            resource_balance.metal_consumption
        );
    }

    #[test]
    fn test_no_import_cost_reduction_when_multiplier_is_one() {
        let mut resource_balance = ResourceBalance {
            fuel_consumption: 100.0,
            metal_consumption: 80.0,
            food_production: 0.0,
            food_consumption: 0.0,
            timber_production: 0.0,
            timber_consumption: 0.0,
            metal_production: 0.0,
            fuel_production: 0.0,
        };

        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        // Should NOT modify consumption
        if effects.import_cost_multiplier < 1.0 {
            let reduction = 1.0 - effects.import_cost_multiplier;
            resource_balance.fuel_consumption *= 1.0 - reduction * 0.3;
            resource_balance.metal_consumption *= 1.0 - reduction * 0.3;
        }

        assert!(
            (resource_balance.fuel_consumption - 100.0).abs() < f32::EPSILON,
            "No reduction should be applied when multiplier is 1.0"
        );
        assert!(
            (resource_balance.metal_consumption - 80.0).abs() < f32::EPSILON,
            "No reduction should be applied when multiplier is 1.0"
        );
    }

    #[test]
    fn test_industrial_production_bonus_applied_to_resources() {
        let mut resource_balance = ResourceBalance {
            food_production: 100.0,
            timber_production: 80.0,
            metal_production: 60.0,
            fuel_production: 40.0,
            food_consumption: 0.0,
            timber_consumption: 0.0,
            metal_consumption: 0.0,
            fuel_consumption: 0.0,
        };

        // SeaPort: industrial_production_bonus = 1.15
        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.15,
            export_price_multiplier: 1.0,
        };

        // Apply the same logic as the system
        if effects.industrial_production_bonus > 1.0 {
            let bonus = effects.industrial_production_bonus - 1.0;
            resource_balance.food_production *= 1.0 + bonus;
            resource_balance.timber_production *= 1.0 + bonus;
            resource_balance.metal_production *= 1.0 + bonus;
            resource_balance.fuel_production *= 1.0 + bonus;
        }

        // bonus = 0.15, multiplier = 1.15
        assert!(
            (resource_balance.food_production - 115.0).abs() < 0.01,
            "Food production should be boosted by 15%"
        );
        assert!(
            (resource_balance.timber_production - 92.0).abs() < 0.01,
            "Timber production should be boosted by 15%"
        );
        assert!(
            (resource_balance.metal_production - 69.0).abs() < 0.01,
            "Metal production should be boosted by 15%"
        );
        assert!(
            (resource_balance.fuel_production - 46.0).abs() < 0.01,
            "Fuel production should be boosted by 15%"
        );
    }

    #[test]
    fn test_no_industrial_bonus_when_multiplier_is_one() {
        let mut resource_balance = ResourceBalance {
            food_production: 100.0,
            timber_production: 80.0,
            metal_production: 60.0,
            fuel_production: 40.0,
            food_consumption: 0.0,
            timber_consumption: 0.0,
            metal_consumption: 0.0,
            fuel_consumption: 0.0,
        };

        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        if effects.industrial_production_bonus > 1.0 {
            let bonus = effects.industrial_production_bonus - 1.0;
            resource_balance.food_production *= 1.0 + bonus;
        }

        assert!(
            (resource_balance.food_production - 100.0).abs() < f32::EPSILON,
            "No bonus should be applied when multiplier is 1.0"
        );
    }

    // =========================================================================
    // 11. Trade balance calculation
    // =========================================================================

    #[test]
    fn test_export_price_multiplier_applied_to_trade_balance() {
        let mut trade_balance: f64 = 1000.0;

        // Airport: export_price_multiplier = 1.20
        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.20,
        };

        // Apply the same logic as the system
        if effects.export_price_multiplier > 1.0 {
            let bonus_factor = effects.export_price_multiplier;
            trade_balance *= bonus_factor as f64;
        }

        assert!(
            (trade_balance - 1200.0).abs() < 0.01,
            "Trade balance should be boosted by 20% from airport"
        );
    }

    #[test]
    fn test_export_price_multiplier_on_negative_trade_balance() {
        let mut trade_balance: f64 = -500.0;

        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.20,
        };

        if effects.export_price_multiplier > 1.0 {
            let bonus_factor = effects.export_price_multiplier;
            trade_balance *= bonus_factor as f64;
        }

        // Negative trade balance gets more negative (amplified deficit)
        assert!(
            (trade_balance - (-600.0)).abs() < 0.01,
            "Negative trade balance should also be multiplied"
        );
    }

    #[test]
    fn test_no_export_multiplier_when_at_one() {
        let mut trade_balance: f64 = 1000.0;

        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        if effects.export_price_multiplier > 1.0 {
            trade_balance *= effects.export_price_multiplier as f64;
        }

        assert!(
            (trade_balance - 1000.0).abs() < f64::EPSILON,
            "Trade balance should not change with multiplier = 1.0"
        );
    }

    #[test]
    fn test_tourism_bonus_applied_and_capped() {
        let mut tourism = Tourism::default();
        assert_eq!(tourism.attractiveness, 0.0);

        // Airport gives +30 tourism bonus
        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 30.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        // Apply same logic as system
        tourism.attractiveness = (tourism.attractiveness + effects.tourism_bonus).min(100.0);
        tourism.monthly_visitors = (tourism.attractiveness * 50.0) as u32;

        assert!((tourism.attractiveness - 30.0).abs() < 0.001);
        assert_eq!(tourism.monthly_visitors, 1500);
    }

    #[test]
    fn test_tourism_attractiveness_capped_at_100() {
        let mut tourism = Tourism {
            attractiveness: 85.0,
            ..Default::default()
        };

        // Railway(10) + Airport(30) = 40 tourism bonus, but 85 + 40 > 100
        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 40.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        tourism.attractiveness = (tourism.attractiveness + effects.tourism_bonus).min(100.0);
        assert!(
            (tourism.attractiveness - 100.0).abs() < 0.001,
            "Tourism attractiveness should be capped at 100"
        );
    }

    #[test]
    fn test_attractiveness_bonus_applied_and_clamped() {
        let mut attractiveness = CityAttractiveness::default();
        assert!((attractiveness.overall_score - 50.0).abs() < 0.001);

        // All four types: attractiveness bonus = 20
        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 20.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        attractiveness.overall_score =
            (attractiveness.overall_score + effects.attractiveness_bonus).clamp(0.0, 100.0);
        assert!(
            (attractiveness.overall_score - 70.0).abs() < 0.001,
            "Attractiveness should go from 50 to 70"
        );
    }

    #[test]
    fn test_attractiveness_clamped_at_100() {
        let mut attractiveness = CityAttractiveness {
            overall_score: 90.0,
            ..Default::default()
        };

        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 20.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        attractiveness.overall_score =
            (attractiveness.overall_score + effects.attractiveness_bonus).clamp(0.0, 100.0);
        assert!(
            (attractiveness.overall_score - 100.0).abs() < 0.001,
            "Attractiveness should be clamped at 100"
        );
    }

    // =========================================================================
    // 12. Connection stats / UI summary
    // =========================================================================

    #[test]
    fn test_connection_stats_summary() {
        let mut outside = OutsideConnections::default();
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 185, 0, 5000, 0.6));
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 185, 255, 5000, 0.4));

        let stats = outside.stats();
        assert_eq!(stats.len(), 4);

        let highway_stat = stats
            .iter()
            .find(|s| s.connection_type == ConnectionType::Highway)
            .unwrap();
        assert!(highway_stat.active);
        assert_eq!(highway_stat.count, 2);
        assert!((highway_stat.avg_utilization - 0.5).abs() < 0.001);

        let railway_stat = stats
            .iter()
            .find(|s| s.connection_type == ConnectionType::Railway)
            .unwrap();
        assert!(!railway_stat.active);
        assert_eq!(railway_stat.count, 0);
        assert_eq!(railway_stat.avg_utilization, 0.0);
    }

    #[test]
    fn test_stats_all_types_active() {
        let mut outside = OutsideConnections::default();
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 185, 0, 5000, 0.5));
        outside
            .connections
            .push(make_conn(ConnectionType::Railway, 1, 128, 2000, 0.3));
        outside
            .connections
            .push(make_conn(ConnectionType::SeaPort, 0, 0, 3000, 0.2));
        outside
            .connections
            .push(make_conn(ConnectionType::Airport, 100, 100, 5000, 0.8));

        let stats = outside.stats();
        for stat in &stats {
            assert!(stat.active, "{:?} should be active", stat.connection_type);
            assert_eq!(stat.count, 1);
            assert!(!stat.effect_description.is_empty());
        }
    }

    #[test]
    fn test_stats_no_connections_all_inactive() {
        let outside = OutsideConnections::default();
        let stats = outside.stats();
        assert_eq!(stats.len(), 4);
        for stat in &stats {
            assert!(
                !stat.active,
                "{:?} should be inactive",
                stat.connection_type
            );
            assert_eq!(stat.count, 0);
            assert_eq!(stat.avg_utilization, 0.0);
        }
    }
}

pub struct OutsideConnectionsPlugin;

impl Plugin for OutsideConnectionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OutsideConnections>().add_systems(
            FixedUpdate,
            update_outside_connections
                .after(crate::airport::update_airports)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
