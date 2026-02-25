//! SERV-004: Garbage Collection Routing
//!
//! Dispatches garbage trucks from waste facilities (landfills, incinerators,
//! recycling centers) along collection routes. Buildings accumulate garbage;
//! trucks collect using nearest-unvisited heuristic. Full trucks (capacity
//! 20 units) return to dump at their facility.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::garbage::GarbageGrid;
use crate::road_graph_csr::{csr_find_path, CsrGraph};
use crate::roads::RoadNode;
use crate::services::ServiceBuilding;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const TRUCK_CAPACITY: u8 = 20;
const TRUCKS_PER_FACILITY: u32 = 2;
const MAX_GARBAGE_TRUCKS: usize = 64;
const TRUCK_SPEED: f32 = 1.5;
const COLLECTION_PER_STOP: u8 = 5;
const HAPPINESS_PENALTY_THRESHOLD: u8 = 10;
const DISPATCH_INTERVAL: u64 = 10;
const MAX_ROUTE_STOPS: usize = 8;

/// Grid position with an associated path (used for facility routing results).
type FacilityRoute = ((usize, usize), Vec<(usize, usize)>);

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A dispatched garbage truck travelling on the road network.
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct GarbageTruck {
    pub facility: (usize, usize),
    pub position: (usize, usize),
    pub load: u8,
    pub returning: bool,
    pub arrived: bool,
    pub path: Vec<(usize, usize)>,
    pub path_index: usize,
    pub route: Vec<(usize, usize)>,
    pub ticks_elapsed: u32,
}

impl GarbageTruck {
    fn new(facility: (usize, usize)) -> Self {
        Self {
            facility,
            position: facility,
            load: 0,
            returning: false,
            arrived: true,
            path: Vec::new(),
            path_index: 0,
            route: Vec::new(),
            ticks_elapsed: 0,
        }
    }

    fn is_full(&self) -> bool {
        self.load >= TRUCK_CAPACITY
    }
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// City-wide garbage collection routing state.
#[derive(Resource, Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct GarbageCollectionState {
    pub trucks: Vec<GarbageTruck>,
    pub total_dispatches: u64,
    pub total_collected: u64,
    pub max_trucks: u32,
    pub buildings_over_threshold: u32,
    pub avg_load_efficiency: f32,
    pub completed_trips: u64,
}

// ---------------------------------------------------------------------------
// Saveable
// ---------------------------------------------------------------------------

impl crate::Saveable for GarbageCollectionState {
    const SAVE_KEY: &'static str = "garbage_collection";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.total_dispatches == 0 && self.trucks.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn update_truck_capacity(
    services: Query<&ServiceBuilding>,
    mut state: ResMut<GarbageCollectionState>,
) {
    let mut capacity: u32 = 0;
    for service in &services {
        if ServiceBuilding::is_garbage(service.service_type) {
            capacity += TRUCKS_PER_FACILITY;
        }
    }
    state.max_trucks = capacity;
}

/// Dispatch garbage trucks to dirty buildings using nearest-unvisited heuristic.
fn dispatch_garbage_trucks(
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
    garbage_grid: Res<GarbageGrid>,
    csr: Res<CsrGraph>,
    mut state: ResMut<GarbageCollectionState>,
) {
    if csr.node_count() == 0 {
        return;
    }
    let facilities: Vec<(usize, usize)> = services
        .iter()
        .filter(|s| ServiceBuilding::is_garbage(s.service_type))
        .map(|s| (s.grid_x, s.grid_y))
        .collect();
    if facilities.is_empty() {
        return;
    }

    let mut dirty: Vec<(usize, usize, u8)> = Vec::new();
    for building in &buildings {
        let level = garbage_grid.get(building.grid_x, building.grid_y);
        if level > 0 {
            let targeted = state.trucks.iter().any(|t| {
                !t.returning && t.route.contains(&(building.grid_x, building.grid_y))
            });
            if !targeted {
                dirty.push((building.grid_x, building.grid_y, level));
            }
        }
    }
    dirty.sort_by(|a, b| b.2.cmp(&a.2));

    while !dirty.is_empty() && can_dispatch(&state) {
        let start = dirty[0];
        let Some((facility, _)) = find_nearest_facility(&csr, &facilities, (start.0, start.1))
        else {
            dirty.remove(0);
            continue;
        };
        let route = build_collection_route(&dirty, (start.0, start.1));
        let first = route.first().copied().unwrap_or((start.0, start.1));
        let path = find_path_coords(&csr, facility, first);

        let mut truck = GarbageTruck::new(facility);
        truck.route = route.clone();
        truck.path = path;
        truck.path_index = 0;
        truck.arrived = false;
        state.trucks.push(truck);
        state.total_dispatches += 1;

        for stop in &route {
            dirty.retain(|b| (b.0, b.1) != *stop);
        }
    }
}

/// Advance trucks along paths; handle collection stops and dump returns.
fn advance_garbage_trucks(
    mut state: ResMut<GarbageCollectionState>,
    mut garbage_grid: ResMut<GarbageGrid>,
    csr: Res<CsrGraph>,
) {
    for truck in &mut state.trucks {
        truck.ticks_elapsed += 1;
        if truck.arrived {
            handle_arrived_truck(truck, &mut garbage_grid, &csr);
        } else {
            move_truck_along_path(truck);
        }
    }
}

fn handle_arrived_truck(truck: &mut GarbageTruck, grid: &mut GarbageGrid, csr: &CsrGraph) {
    if truck.returning {
        truck.load = 0;
        truck.returning = false;
        if let Some(next) = truck.route.first().copied() {
            set_truck_path(truck, csr, truck.position, next);
        }
    } else if !truck.route.is_empty() {
        let (bx, by) = truck.route[0];
        let current = grid.get(bx, by);
        let collected = current.min(COLLECTION_PER_STOP);
        grid.set(bx, by, current.saturating_sub(collected));
        truck.load = truck.load.saturating_add(collected);
        truck.route.remove(0);

        if truck.is_full() || truck.route.is_empty() {
            truck.returning = true;
            set_truck_path(truck, csr, truck.position, truck.facility);
        } else if let Some(next) = truck.route.first().copied() {
            set_truck_path(truck, csr, truck.position, next);
        }
    } else {
        truck.returning = true;
        set_truck_path(truck, csr, truck.position, truck.facility);
    }
}

fn set_truck_path(truck: &mut GarbageTruck, csr: &CsrGraph, from: (usize, usize), to: (usize, usize)) {
    truck.path = find_path_coords(csr, from, to);
    truck.path_index = 0;
    truck.arrived = false;
}

fn move_truck_along_path(truck: &mut GarbageTruck) {
    let steps = TRUCK_SPEED as usize;
    for _ in 0..steps {
        if truck.path_index + 1 < truck.path.len() {
            truck.path_index += 1;
            truck.position = truck.path[truck.path_index];
        } else {
            truck.arrived = true;
            if let Some(&last) = truck.path.last() {
                truck.position = last;
            }
            break;
        }
    }
}

fn cleanup_garbage_trucks(mut state: ResMut<GarbageCollectionState>) {
    let before = state.trucks.len();
    state.trucks.retain(|t| {
        !(t.arrived && !t.returning && t.route.is_empty() && t.load == 0)
    });
    state.completed_trips += (before - state.trucks.len()) as u64;
}

fn count_over_threshold(
    slow_timer: Res<SlowTickTimer>,
    buildings: Query<&Building>,
    garbage_grid: Res<GarbageGrid>,
    mut state: ResMut<GarbageCollectionState>,
) {
    if !slow_timer.should_run() {
        return;
    }
    let mut count = 0u32;
    for building in &buildings {
        if garbage_grid.get(building.grid_x, building.grid_y) > HAPPINESS_PENALTY_THRESHOLD {
            count += 1;
        }
    }
    state.buildings_over_threshold = count;
    state.total_collected = state.completed_trips * TRUCK_CAPACITY as u64;
    if state.completed_trips > 0 {
        state.avg_load_efficiency =
            state.total_collected as f32 / (state.completed_trips as f32 * TRUCK_CAPACITY as f32);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn can_dispatch(state: &GarbageCollectionState) -> bool {
    state.trucks.len() < MAX_GARBAGE_TRUCKS && (state.trucks.len() as u32) < state.max_trucks
}

fn find_nearest_facility(
    csr: &CsrGraph,
    facilities: &[(usize, usize)],
    target: (usize, usize),
) -> Option<FacilityRoute> {
    let tgt = RoadNode(target.0, target.1);
    let mut best: Option<FacilityRoute> = None;
    for &(fx, fy) in facilities {
        if let Some(path) = find_nearby_road_path(csr, RoadNode(fx, fy), tgt) {
            let coords: Vec<(usize, usize)> = path.iter().map(|n| (n.0, n.1)).collect();
            if best.as_ref().is_none_or(|(_, bp)| coords.len() < bp.len()) {
                best = Some(((fx, fy), coords));
            }
        }
    }
    best
}

fn build_collection_route(
    dirty: &[(usize, usize, u8)],
    start: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut route = Vec::new();
    let mut remaining: Vec<(usize, usize)> = dirty
        .iter()
        .take(MAX_ROUTE_STOPS * 2)
        .map(|&(x, y, _)| (x, y))
        .collect();
    let mut current = start;
    while !remaining.is_empty() && route.len() < MAX_ROUTE_STOPS {
        let (best_idx, _) = remaining
            .iter()
            .enumerate()
            .min_by_key(|(_, &(bx, by))| {
                let dx = (bx as i64 - current.0 as i64).unsigned_abs();
                let dy = (by as i64 - current.1 as i64).unsigned_abs();
                dx * dx + dy * dy
            })
            .unwrap();
        let next = remaining.remove(best_idx);
        route.push(next);
        current = next;
    }
    route
}

fn find_path_coords(csr: &CsrGraph, from: (usize, usize), to: (usize, usize)) -> Vec<(usize, usize)> {
    if let Some(path) = find_nearby_road_path(csr, RoadNode(from.0, from.1), RoadNode(to.0, to.1)) {
        path.iter().map(|n| (n.0, n.1)).collect()
    } else {
        vec![from, to]
    }
}

fn find_nearby_road_path(csr: &CsrGraph, start: RoadNode, goal: RoadNode) -> Option<Vec<RoadNode>> {
    if let Some(p) = csr_find_path(csr, start, goal) {
        return Some(p);
    }
    for s in &adjacent_nodes(start) {
        if let Some(p) = csr_find_path(csr, *s, goal) {
            return Some(p);
        }
        for g in &adjacent_nodes(goal) {
            if let Some(p) = csr_find_path(csr, *s, *g) {
                return Some(p);
            }
        }
    }
    for g in &adjacent_nodes(goal) {
        if let Some(p) = csr_find_path(csr, start, *g) {
            return Some(p);
        }
    }
    None
}

fn adjacent_nodes(node: RoadNode) -> Vec<RoadNode> {
    let mut out = Vec::with_capacity(4);
    if node.0 > 0 { out.push(RoadNode(node.0 - 1, node.1)); }
    if node.1 > 0 { out.push(RoadNode(node.0, node.1 - 1)); }
    if node.0 + 1 < GRID_WIDTH { out.push(RoadNode(node.0 + 1, node.1)); }
    if node.1 + 1 < GRID_HEIGHT { out.push(RoadNode(node.0, node.1 + 1)); }
    out
}

fn dispatch_tick_guard(tick: Res<crate::TickCounter>) -> bool {
    tick.0.is_multiple_of(DISPATCH_INTERVAL)
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct GarbageCollectionPlugin;

impl Plugin for GarbageCollectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GarbageCollectionState>();
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<GarbageCollectionState>();

        app.add_systems(
            FixedUpdate,
            (
                update_truck_capacity,
                dispatch_garbage_trucks,
                advance_garbage_trucks,
                cleanup_garbage_trucks,
            )
                .chain()
                .run_if(dispatch_tick_guard)
                .in_set(crate::SimulationSet::Simulation),
        );
        app.add_systems(
            FixedUpdate,
            count_over_threshold
                .after(crate::garbage::update_garbage)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truck_capacity_and_state() {
        let mut truck = GarbageTruck::new((10, 10));
        assert!(!truck.is_full());
        assert_eq!(truck.facility, (10, 10));
        assert_eq!(truck.load, 0);
        assert!(truck.arrived);
        truck.load = TRUCK_CAPACITY;
        assert!(truck.is_full());
    }

    #[test]
    fn test_can_dispatch_limit() {
        let mut state = GarbageCollectionState::default();
        state.max_trucks = 4;
        assert!(can_dispatch(&state));
        for _ in 0..4 {
            state.trucks.push(GarbageTruck::new((0, 0)));
        }
        assert!(!can_dispatch(&state));
    }

    #[test]
    fn test_build_collection_route_nearest_first() {
        let dirty = vec![(10, 10, 5u8), (12, 12, 8), (11, 11, 6)];
        let route = build_collection_route(&dirty, (10, 10));
        assert!(!route.is_empty());
        assert!(route.len() <= MAX_ROUTE_STOPS);
        assert_eq!(route[0], (10, 10));
    }

    #[test]
    fn test_build_collection_route_max_stops() {
        let dirty: Vec<(usize, usize, u8)> = (0..20).map(|i| (i, i, 5)).collect();
        let route = build_collection_route(&dirty, (0, 0));
        assert!(route.len() <= MAX_ROUTE_STOPS);
    }

    #[test]
    fn test_default_state() {
        let state = GarbageCollectionState::default();
        assert!(state.trucks.is_empty());
        assert_eq!(state.total_dispatches, 0);
        assert_eq!(state.max_trucks, 0);
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = GarbageCollectionState::default();
        state.total_dispatches = 42;
        state.total_collected = 100;
        let bytes = state.save_to_bytes().unwrap();
        let restored = GarbageCollectionState::load_from_bytes(&bytes);
        assert_eq!(restored.total_dispatches, 42);
        assert_eq!(restored.total_collected, 100);
    }

    #[test]
    fn test_saveable_skip_default() {
        use crate::Saveable;
        assert!(GarbageCollectionState::default().save_to_bytes().is_none());
    }

    #[test]
    fn test_adjacent_nodes_count() {
        assert_eq!(adjacent_nodes(RoadNode(5, 5)).len(), 4);
        assert_eq!(adjacent_nodes(RoadNode(0, 0)).len(), 2);
    }
}
