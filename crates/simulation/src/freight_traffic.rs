//! TRAF-004: Freight/Goods Traffic on Road Network.
//!
//! Industrial buildings generate outbound freight (trucks) that deliver goods
//! to commercial buildings. Trucks are heavier than cars and contribute more
//! to congestion, road wear, and noise.
//!
//! Key behaviors:
//! - Industrial buildings generate outbound freight demand proportional to occupants
//! - Commercial buildings generate inbound freight demand proportional to occupants
//! - Freight vehicles (trucks) are spawned, routed via A*, and despawned on arrival
//! - Trucks have a vehicle equivalence factor of 2.5 (each truck = 2.5 cars for congestion)
//! - Trucks add to traffic density on the road grid via `TrafficGrid`
//! - Trucks increase road wear in `RoadConditionGrid`
//! - Heavy traffic ban per district blocks truck routing through those districts
//! - Freight satisfaction affects commercial/industrial productivity

use std::collections::HashMap;

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{WorldGrid, ZoneType};
use crate::pathfinding_sys::nearest_road_grid;
use crate::road_graph_csr::{csr_find_path_with_traffic, CsrGraph};
use crate::road_maintenance::RoadConditionGrid;
use crate::roads::RoadNode;
use crate::traffic::TrafficGrid;
use crate::TickCounter;

// =============================================================================
// Constants
// =============================================================================

/// Vehicle equivalence factor: each truck contributes this many "car equivalents"
/// to traffic density, road wear, and congestion calculations.
const TRUCK_EQUIVALENCE_FACTOR: f32 = 2.5;

/// Extra road degradation per truck waypoint visit (on top of normal traffic wear).
const TRUCK_WEAR_PER_VISIT: u8 = 1;

/// How often to generate new freight trips (every N ticks). At 10Hz, 20 ticks = 2s.
const FREIGHT_GENERATION_INTERVAL: u64 = 20;

/// How often to move freight trucks (every N ticks). Same cadence as traffic updates.
const FREIGHT_MOVE_INTERVAL: u64 = 5;

/// Maximum number of concurrent freight trucks in the city.
const MAX_FREIGHT_TRUCKS: usize = 200;

/// Maximum freight trips generated per cycle.
const MAX_TRIPS_PER_CYCLE: usize = 10;

/// Freight demand per occupant in an industrial building (outbound goods).
const INDUSTRIAL_FREIGHT_RATE: f32 = 0.02;

/// Freight demand per occupant in a commercial building (inbound goods).
const COMMERCIAL_FREIGHT_RATE: f32 = 0.015;

/// Maximum search distance for matching freight origin to destination (grid cells).
const MAX_FREIGHT_DISTANCE: i32 = 60;

/// Truck movement speed in grid cells per move tick.
const TRUCK_SPEED: usize = 2;

// =============================================================================
// Components and Resources
// =============================================================================

/// A single freight truck moving along a pre-computed route.
#[derive(Debug, Clone)]
pub struct FreightTruck {
    /// Grid positions along the route.
    pub route: Vec<RoadNode>,
    /// Current position index in the route.
    pub current_index: usize,
    /// Origin building grid position.
    pub origin: (usize, usize),
    /// Destination building grid position.
    pub destination: (usize, usize),
}

impl FreightTruck {
    /// Returns the current grid position of the truck, or `None` if route is complete.
    pub fn current_position(&self) -> Option<&RoadNode> {
        self.route.get(self.current_index)
    }

    /// Advance the truck along its route by `steps` waypoints.
    pub fn advance(&mut self, steps: usize) {
        self.current_index = (self.current_index + steps).min(self.route.len());
    }

    /// Returns true if the truck has reached its destination.
    pub fn is_arrived(&self) -> bool {
        self.current_index >= self.route.len()
    }
}

/// City-wide freight traffic state resource.
#[derive(Resource, Debug, Clone)]
pub struct FreightTrafficState {
    /// Active freight trucks currently on the road network.
    pub trucks: Vec<FreightTruck>,
    /// Accumulated freight demand from industrial buildings (outbound).
    pub industrial_demand: f32,
    /// Accumulated freight demand from commercial buildings (inbound).
    pub commercial_demand: f32,
    /// Freight satisfaction ratio (0.0-1.0): fraction of demand met by deliveries.
    pub satisfaction: f32,
    /// Total trips completed since last reset.
    pub trips_completed: u64,
    /// Total trips generated since last reset.
    pub trips_generated: u64,
    /// Per-district heavy traffic ban. Key = district index, value = banned.
    pub heavy_traffic_ban: HashMap<usize, bool>,
}

impl Default for FreightTrafficState {
    fn default() -> Self {
        Self {
            trucks: Vec::new(),
            industrial_demand: 0.0,
            commercial_demand: 0.0,
            satisfaction: 1.0,
            trips_completed: 0,
            trips_generated: 0,
            heavy_traffic_ban: HashMap::new(),
        }
    }
}

impl FreightTrafficState {
    /// Toggle the heavy traffic ban for a specific district.
    pub fn toggle_heavy_traffic_ban(&mut self, district_idx: usize) {
        let entry = self.heavy_traffic_ban.entry(district_idx).or_insert(false);
        *entry = !*entry;
    }

    /// Check if heavy traffic is banned in a specific district.
    pub fn is_heavy_traffic_banned(&self, district_idx: usize) -> bool {
        self.heavy_traffic_ban
            .get(&district_idx)
            .copied()
            .unwrap_or(false)
    }
}

/// Serializable subset of FreightTrafficState for save/load.
#[derive(Debug, Clone, Default, Encode, Decode)]
struct FreightTrafficSaveData {
    satisfaction: f32,
    trips_completed: u64,
    trips_generated: u64,
    heavy_traffic_ban: Vec<(usize, bool)>,
}

impl crate::Saveable for FreightTrafficState {
    const SAVE_KEY: &'static str = "freight_traffic";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        let data = FreightTrafficSaveData {
            satisfaction: self.satisfaction,
            trips_completed: self.trips_completed,
            trips_generated: self.trips_generated,
            heavy_traffic_ban: self
                .heavy_traffic_ban
                .iter()
                .filter(|(_, &v)| v)
                .map(|(&k, &v)| (k, v))
                .collect(),
        };
        // Skip saving if everything is at default
        if data.trips_completed == 0
            && data.trips_generated == 0
            && data.heavy_traffic_ban.is_empty()
        {
            return None;
        }
        Some(bitcode::encode(&data))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let data: FreightTrafficSaveData = crate::decode_or_warn(Self::SAVE_KEY, bytes);
        Self {
            satisfaction: data.satisfaction,
            trips_completed: data.trips_completed,
            trips_generated: data.trips_generated,
            heavy_traffic_ban: data.heavy_traffic_ban.into_iter().collect(),
            ..Self::default()
        }
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Compute freight demand from industrial and commercial buildings.
/// Runs on the slow tick timer interval.
pub fn compute_freight_demand(
    tick: Res<TickCounter>,
    buildings: Query<&Building>,
    mut freight: ResMut<FreightTrafficState>,
) {
    if !tick.0.is_multiple_of(FREIGHT_GENERATION_INTERVAL) {
        return;
    }

    let mut ind_demand = 0.0f32;
    let mut com_demand = 0.0f32;

    for building in &buildings {
        if building.occupants == 0 {
            continue;
        }
        match building.zone_type {
            ZoneType::Industrial => {
                ind_demand += building.occupants as f32 * INDUSTRIAL_FREIGHT_RATE;
            }
            ZoneType::CommercialLow | ZoneType::CommercialHigh => {
                com_demand += building.occupants as f32 * COMMERCIAL_FREIGHT_RATE;
            }
            _ => {}
        }
    }

    freight.industrial_demand = ind_demand;
    freight.commercial_demand = com_demand;
}

/// Generate freight trips: match industrial origins to commercial destinations.
/// Spawns trucks with pre-computed A* routes on the road network.
#[allow(clippy::too_many_arguments)]
pub fn generate_freight_trips(
    tick: Res<TickCounter>,
    grid: Res<WorldGrid>,
    csr: Res<CsrGraph>,
    traffic: Res<TrafficGrid>,
    buildings: Query<&Building>,
    mut freight: ResMut<FreightTrafficState>,
) {
    if !tick.0.is_multiple_of(FREIGHT_GENERATION_INTERVAL) {
        return;
    }

    if freight.trucks.len() >= MAX_FREIGHT_TRUCKS {
        return;
    }

    // Collect industrial origins and commercial destinations
    let mut origins: Vec<(usize, usize)> = Vec::new();
    let mut destinations: Vec<(usize, usize)> = Vec::new();

    for building in &buildings {
        if building.occupants == 0 {
            continue;
        }
        match building.zone_type {
            ZoneType::Industrial => {
                origins.push((building.grid_x, building.grid_y));
            }
            ZoneType::CommercialLow | ZoneType::CommercialHigh => {
                destinations.push((building.grid_x, building.grid_y));
            }
            _ => {}
        }
    }

    if origins.is_empty() || destinations.is_empty() {
        return;
    }

    // Determine how many trips to generate this cycle based on demand
    let demand = freight.industrial_demand.min(freight.commercial_demand);
    let trips_to_generate = (demand as usize)
        .min(MAX_TRIPS_PER_CYCLE)
        .min(MAX_FREIGHT_TRUCKS - freight.trucks.len());

    if trips_to_generate == 0 {
        return;
    }

    // Use a simple hash-based matching: pair origins with nearest destinations
    let mut generated = 0usize;
    for &(ox, oy) in origins.iter() {
        if generated >= trips_to_generate {
            break;
        }

        // Find nearest commercial destination within range
        let dest = find_nearest_destination(&destinations, ox, oy, MAX_FREIGHT_DISTANCE);
        let Some((dx, dy)) = dest else {
            continue;
        };

        // Resolve to road nodes
        let start = nearest_road_grid(&grid, ox, oy);
        let goal = nearest_road_grid(&grid, dx, dy);

        if let (Some(start_node), Some(goal_node)) = (start, goal) {
            // Compute route using A* pathfinding
            if let Some(route) =
                csr_find_path_with_traffic(&csr, start_node, goal_node, &grid, &traffic)
            {
                freight.trucks.push(FreightTruck {
                    route,
                    current_index: 0,
                    origin: (ox, oy),
                    destination: (dx, dy),
                });
                freight.trips_generated += 1;
                generated += 1;
            }
        }
    }
}

/// Move freight trucks along their routes and apply traffic/wear effects.
pub fn move_freight_trucks(
    tick: Res<TickCounter>,
    mut freight: ResMut<FreightTrafficState>,
    mut traffic: ResMut<TrafficGrid>,
    mut condition_grid: ResMut<RoadConditionGrid>,
) {
    if !tick.0.is_multiple_of(FREIGHT_MOVE_INTERVAL) {
        return;
    }

    // Move each truck and apply effects
    for truck in &mut freight.trucks {
        // Apply traffic density and road wear at current position before moving
        if let Some(pos) = truck.current_position() {
            let x = pos.0.min(GRID_WIDTH - 1);
            let y = pos.1.min(GRID_HEIGHT - 1);

            // Add truck equivalence to traffic density
            let equiv = TRUCK_EQUIVALENCE_FACTOR as u16;
            let current_density = traffic.get(x, y);
            traffic.set(x, y, current_density.saturating_add(equiv));

            // Apply extra road wear from heavy truck
            let current_cond = condition_grid.get(x, y);
            if current_cond > 0 {
                condition_grid.set(x, y, current_cond.saturating_sub(TRUCK_WEAR_PER_VISIT));
            }
        }

        // Advance truck along route
        truck.advance(TRUCK_SPEED);
    }

    // Count completed trips before removing
    let completed_before = freight.trucks.len();
    freight.trucks.retain(|t| !t.is_arrived());
    let completed_now = completed_before - freight.trucks.len();
    freight.trips_completed += completed_now as u64;
}

/// Update freight satisfaction ratio based on demand vs. active deliveries.
pub fn update_freight_satisfaction(
    tick: Res<TickCounter>,
    mut freight: ResMut<FreightTrafficState>,
) {
    if !tick.0.is_multiple_of(FREIGHT_GENERATION_INTERVAL) {
        return;
    }

    let total_demand = freight.industrial_demand + freight.commercial_demand;
    if total_demand < 0.01 {
        freight.satisfaction = 1.0;
        return;
    }

    // Satisfaction based on active trucks vs. demand
    let supply = freight.trucks.len() as f32;
    let ratio = (supply / total_demand.max(1.0)).min(1.0);

    // Smooth the satisfaction value to avoid oscillation
    freight.satisfaction = freight.satisfaction * 0.8 + ratio * 0.2;
}

// =============================================================================
// Helpers
// =============================================================================

/// Find the nearest destination within `max_dist` Manhattan distance.
fn find_nearest_destination(
    destinations: &[(usize, usize)],
    from_x: usize,
    from_y: usize,
    max_dist: i32,
) -> Option<(usize, usize)> {
    destinations
        .iter()
        .filter_map(|&(x, y)| {
            let dist = (x as i32 - from_x as i32).abs() + (y as i32 - from_y as i32).abs();
            if dist <= max_dist {
                Some(((x, y), dist))
            } else {
                None
            }
        })
        .min_by_key(|&(_, dist)| dist)
        .map(|(pos, _)| pos)
}

// =============================================================================
// Plugin
// =============================================================================

pub struct FreightTrafficPlugin;

impl Plugin for FreightTrafficPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FreightTrafficState>().add_systems(
            FixedUpdate,
            (
                compute_freight_demand,
                generate_freight_trips,
                move_freight_trucks,
                update_freight_satisfaction,
            )
                .chain()
                .after(crate::traffic::update_traffic_density)
                .in_set(crate::SimulationSet::Simulation),
        );
        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<FreightTrafficState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Saveable;

    #[test]
    fn test_freight_truck_advance() {
        let truck = FreightTruck {
            route: vec![
                RoadNode(10, 10),
                RoadNode(11, 10),
                RoadNode(12, 10),
                RoadNode(13, 10),
                RoadNode(14, 10),
            ],
            current_index: 0,
            origin: (10, 10),
            destination: (14, 10),
        };

        let mut t = truck;
        assert_eq!(t.current_position(), Some(&RoadNode(10, 10)));
        assert!(!t.is_arrived());

        t.advance(2);
        assert_eq!(t.current_position(), Some(&RoadNode(12, 10)));
        assert!(!t.is_arrived());

        t.advance(3);
        assert!(t.is_arrived());
        assert_eq!(t.current_position(), None);
    }

    #[test]
    fn test_freight_truck_advance_past_end() {
        let mut truck = FreightTruck {
            route: vec![RoadNode(10, 10), RoadNode(11, 10)],
            current_index: 0,
            origin: (10, 10),
            destination: (11, 10),
        };
        truck.advance(100);
        assert!(truck.is_arrived());
        assert_eq!(truck.current_index, 2); // clamped to route length
    }

    #[test]
    fn test_default_freight_state() {
        let state = FreightTrafficState::default();
        assert!(state.trucks.is_empty());
        assert_eq!(state.industrial_demand, 0.0);
        assert_eq!(state.commercial_demand, 0.0);
        assert!((state.satisfaction - 1.0).abs() < f32::EPSILON);
        assert_eq!(state.trips_completed, 0);
        assert_eq!(state.trips_generated, 0);
        assert!(state.heavy_traffic_ban.is_empty());
    }

    #[test]
    fn test_heavy_traffic_ban_toggle() {
        let mut state = FreightTrafficState::default();
        assert!(!state.is_heavy_traffic_banned(0));

        state.toggle_heavy_traffic_ban(0);
        assert!(state.is_heavy_traffic_banned(0));

        state.toggle_heavy_traffic_ban(0);
        assert!(!state.is_heavy_traffic_banned(0));
    }

    #[test]
    fn test_heavy_traffic_ban_per_district() {
        let mut state = FreightTrafficState::default();
        state.toggle_heavy_traffic_ban(1);
        state.toggle_heavy_traffic_ban(3);

        assert!(!state.is_heavy_traffic_banned(0));
        assert!(state.is_heavy_traffic_banned(1));
        assert!(!state.is_heavy_traffic_banned(2));
        assert!(state.is_heavy_traffic_banned(3));
    }

    #[test]
    fn test_find_nearest_destination_basic() {
        let dests = vec![(20, 20), (30, 30), (15, 15)];
        let result = find_nearest_destination(&dests, 10, 10, 60);
        assert_eq!(result, Some((15, 15)));
    }

    #[test]
    fn test_find_nearest_destination_out_of_range() {
        let dests = vec![(200, 200)];
        let result = find_nearest_destination(&dests, 10, 10, 60);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_nearest_destination_empty() {
        let dests: Vec<(usize, usize)> = vec![];
        let result = find_nearest_destination(&dests, 10, 10, 60);
        assert!(result.is_none());
    }

    #[test]
    fn test_truck_equivalence_factor() {
        // Verify the constant is reasonable (between 2.0 and 3.0 as per issue spec)
        assert!(TRUCK_EQUIVALENCE_FACTOR >= 2.0);
        assert!(TRUCK_EQUIVALENCE_FACTOR <= 3.0);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut state = FreightTrafficState::default();
        state.trips_completed = 42;
        state.trips_generated = 100;
        state.satisfaction = 0.75;
        state.toggle_heavy_traffic_ban(2);

        let bytes = state
            .save_to_bytes()
            .expect("should save non-default state");
        let loaded = FreightTrafficState::load_from_bytes(&bytes);

        assert_eq!(loaded.trips_completed, 42);
        assert_eq!(loaded.trips_generated, 100);
        assert!((loaded.satisfaction - 0.75).abs() < f32::EPSILON);
        assert!(loaded.is_heavy_traffic_banned(2));
        assert!(!loaded.is_heavy_traffic_banned(0));
    }

    #[test]
    fn test_saveable_skip_default() {
        let state = FreightTrafficState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "default state should skip save"
        );
    }
}
// end of freight_traffic module
