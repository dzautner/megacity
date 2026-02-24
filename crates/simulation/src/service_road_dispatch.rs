//! SERV-002: Service Vehicle Dispatch on Road Network
//!
//! Dispatches emergency service vehicles (fire trucks, ambulances, police cars)
//! on the road network using CSR pathfinding. Response time depends on path
//! distance and traffic conditions, and affects outcomes (fire damage, survival).
//! Vehicle count is limited by service building capacity.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::fire::{FireGrid, OnFire};
use crate::road_graph_csr::{csr_find_path, CsrGraph};
use crate::roads::RoadNode;
use crate::services::ServiceBuilding;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The kind of emergency a service vehicle responds to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub enum EmergencyKind {
    Fire,
    Medical,
    Police,
}

/// A dispatched service vehicle travelling on the road network.
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct ServiceVehicle {
    pub kind: EmergencyKind,
    pub origin: (usize, usize),
    pub target: (usize, usize),
    pub path: Vec<(usize, usize)>,
    pub path_index: usize,
    pub path_length: u32,
    pub speed: f32,
    pub arrived: bool,
    pub ticks_elapsed: u32,
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Tracks all active service vehicle dispatches and aggregate statistics.
#[derive(Resource, Default, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct ServiceDispatchState {
    pub vehicles: Vec<ServiceVehicle>,
    pub total_dispatches: u64,
    pub avg_response_time: f32,
    pub completed_responses: u64,
    pub max_vehicles: u32,
}

impl ServiceDispatchState {
    fn record_response(&mut self, ticks: u32) {
        let n = self.completed_responses as f32;
        self.avg_response_time = (self.avg_response_time * n + ticks as f32) / (n + 1.0);
        self.completed_responses += 1;
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const VEHICLE_SPEED: f32 = 2.0;
const VEHICLES_PER_BUILDING: u32 = 2;
const ON_SCENE_DURATION: u32 = 50;
const MAX_CONCURRENT_DISPATCHES: usize = 32;
const FIRE_TRUCK_SUPPRESSION_RATE: f32 = 3.0;
const FIRE_DISPATCH_THRESHOLD: f32 = 5.0;
const AMBULANCE_DISPATCH_THRESHOLD: f32 = 30.0;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn update_vehicle_capacity(
    services: Query<&ServiceBuilding>,
    mut state: ResMut<ServiceDispatchState>,
) {
    let mut capacity: u32 = 0;
    for service in &services {
        if ServiceBuilding::is_fire(service.service_type)
            || ServiceBuilding::is_health(service.service_type)
            || ServiceBuilding::is_police(service.service_type)
        {
            capacity += VEHICLES_PER_BUILDING;
        }
    }
    state.max_vehicles = capacity;
}

/// Dispatch fire trucks to active fires from the nearest fire station.
fn dispatch_fire_trucks(
    fire_buildings: Query<(&Building, &OnFire)>,
    services: Query<&ServiceBuilding>,
    csr: Res<CsrGraph>,
    mut state: ResMut<ServiceDispatchState>,
) {
    if csr.node_count() == 0 {
        return;
    }

    let fire_stations: Vec<(usize, usize)> = services
        .iter()
        .filter(|s| ServiceBuilding::is_fire(s.service_type))
        .map(|s| (s.grid_x, s.grid_y))
        .collect();
    if fire_stations.is_empty() {
        return;
    }

    let mut targets: Vec<(usize, usize, f32)> = Vec::new();
    for (building, on_fire) in &fire_buildings {
        if on_fire.intensity < FIRE_DISPATCH_THRESHOLD {
            continue;
        }
        let tgt = (building.grid_x, building.grid_y);
        if state.vehicles.iter().any(|v| v.kind == EmergencyKind::Fire && v.target == tgt) {
            continue;
        }
        targets.push((tgt.0, tgt.1, on_fire.intensity));
    }
    targets.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    for (tx, ty, _) in targets {
        if !can_dispatch(&state) {
            break;
        }
        if let Some((origin, path)) = find_best_route(&csr, &fire_stations, (tx, ty)) {
            push_vehicle(&mut state, EmergencyKind::Fire, origin, (tx, ty), path);
        }
    }
}

/// Dispatch ambulances to buildings with severe fires.
fn dispatch_ambulances(
    fire_buildings: Query<(&Building, &OnFire)>,
    services: Query<&ServiceBuilding>,
    csr: Res<CsrGraph>,
    mut state: ResMut<ServiceDispatchState>,
) {
    if csr.node_count() == 0 {
        return;
    }

    let hospitals: Vec<(usize, usize)> = services
        .iter()
        .filter(|s| ServiceBuilding::is_health(s.service_type))
        .map(|s| (s.grid_x, s.grid_y))
        .collect();
    if hospitals.is_empty() {
        return;
    }

    for (building, on_fire) in &fire_buildings {
        if on_fire.intensity < AMBULANCE_DISPATCH_THRESHOLD {
            continue;
        }
        if !can_dispatch(&state) {
            break;
        }
        let tgt = (building.grid_x, building.grid_y);
        if state.vehicles.iter().any(|v| v.kind == EmergencyKind::Medical && v.target == tgt) {
            continue;
        }
        if let Some((origin, path)) = find_best_route(&csr, &hospitals, tgt) {
            push_vehicle(&mut state, EmergencyKind::Medical, origin, tgt, path);
        }
    }
}

/// Move vehicles along their paths and handle arrival.
fn advance_vehicles(mut state: ResMut<ServiceDispatchState>) {
    for vehicle in &mut state.vehicles {
        vehicle.ticks_elapsed += 1;
        if vehicle.arrived {
            continue;
        }
        let steps = vehicle.speed as usize;
        for _ in 0..steps {
            if vehicle.path_index + 1 < vehicle.path.len() {
                vehicle.path_index += 1;
            } else {
                vehicle.arrived = true;
                break;
            }
        }
    }
}

/// Fire trucks on scene suppress fire intensity.
fn apply_on_scene_effects(
    state: Res<ServiceDispatchState>,
    mut fire_grid: ResMut<FireGrid>,
    mut burning: Query<(&Building, &mut OnFire)>,
) {
    for vehicle in &state.vehicles {
        if !vehicle.arrived || vehicle.kind != EmergencyKind::Fire {
            continue;
        }
        let (tx, ty) = vehicle.target;
        let current = fire_grid.get(tx, ty);
        if current > 0 {
            fire_grid.set(tx, ty, current.saturating_sub(FIRE_TRUCK_SUPPRESSION_RATE as u8));
        }
        for (building, mut on_fire) in &mut burning {
            if building.grid_x == tx && building.grid_y == ty {
                on_fire.intensity = (on_fire.intensity - FIRE_TRUCK_SUPPRESSION_RATE).max(0.0);
            }
        }
    }
}

/// Record response times for newly arrived vehicles.
fn record_response_times(mut state: ResMut<ServiceDispatchState>) {
    let arrivals: Vec<u32> = state
        .vehicles
        .iter()
        .filter(|v| v.arrived && v.ticks_elapsed == arrival_tick(v))
        .map(|v| v.ticks_elapsed)
        .collect();
    for ticks in arrivals {
        state.record_response(ticks);
    }
}

/// Remove vehicles that have finished their on-scene duration.
fn cleanup_completed_vehicles(mut state: ResMut<ServiceDispatchState>) {
    state.vehicles.retain(|v| {
        if v.arrived {
            v.ticks_elapsed.saturating_sub(v.path_length) < ON_SCENE_DURATION
        } else {
            true
        }
    });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn can_dispatch(state: &ServiceDispatchState) -> bool {
    state.vehicles.len() < MAX_CONCURRENT_DISPATCHES
        && (state.vehicles.len() as u32) < state.max_vehicles
}

fn push_vehicle(
    state: &mut ServiceDispatchState,
    kind: EmergencyKind,
    origin: (usize, usize),
    target: (usize, usize),
    path: Vec<RoadNode>,
) {
    let path_coords: Vec<(usize, usize)> = path.iter().map(|n| (n.0, n.1)).collect();
    let path_length = path_coords.len() as u32;
    state.vehicles.push(ServiceVehicle {
        kind,
        origin,
        target,
        path: path_coords,
        path_index: 0,
        path_length,
        speed: VEHICLE_SPEED,
        arrived: false,
        ticks_elapsed: 0,
    });
    state.total_dispatches += 1;
}

fn find_best_route(
    csr: &CsrGraph,
    stations: &[(usize, usize)],
    target: (usize, usize),
) -> Option<((usize, usize), Vec<RoadNode>)> {
    let target_node = RoadNode(target.0, target.1);
    let mut best: Option<((usize, usize), Vec<RoadNode>)> = None;
    for &(sx, sy) in stations {
        if let Some(path) = find_nearby_path(csr, RoadNode(sx, sy), target_node) {
            let shorter = best.as_ref().is_none_or(|(_, bp)| path.len() < bp.len());
            if shorter {
                best = Some(((sx, sy), path));
            }
        }
    }
    best
}

fn find_nearby_path(csr: &CsrGraph, start: RoadNode, goal: RoadNode) -> Option<Vec<RoadNode>> {
    if let Some(path) = csr_find_path(csr, start, goal) {
        return Some(path);
    }
    let starts = adjacent_nodes(start);
    let goals = adjacent_nodes(goal);
    for s in &starts {
        if let Some(path) = csr_find_path(csr, *s, goal) {
            return Some(path);
        }
        for g in &goals {
            if let Some(path) = csr_find_path(csr, *s, *g) {
                return Some(path);
            }
        }
    }
    for g in &goals {
        if let Some(path) = csr_find_path(csr, start, *g) {
            return Some(path);
        }
    }
    None
}

fn adjacent_nodes(node: RoadNode) -> Vec<RoadNode> {
    let mut out = Vec::with_capacity(4);
    if node.0 > 0 {
        out.push(RoadNode(node.0 - 1, node.1));
    }
    if node.1 > 0 {
        out.push(RoadNode(node.0, node.1 - 1));
    }
    if node.0 + 1 < GRID_WIDTH {
        out.push(RoadNode(node.0 + 1, node.1));
    }
    if node.1 + 1 < GRID_HEIGHT {
        out.push(RoadNode(node.0, node.1 + 1));
    }
    out
}

fn arrival_tick(vehicle: &ServiceVehicle) -> u32 {
    let steps = vehicle.path.len().saturating_sub(1) as f32;
    (steps / vehicle.speed).ceil() as u32 + 1
}

// ---------------------------------------------------------------------------
// Saveable
// ---------------------------------------------------------------------------

impl crate::Saveable for ServiceDispatchState {
    const SAVE_KEY: &'static str = "service_dispatch";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.total_dispatches == 0 && self.vehicles.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ServiceRoadDispatchPlugin;

impl Plugin for ServiceRoadDispatchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServiceDispatchState>();

        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ServiceDispatchState>();

        app.add_systems(
            FixedUpdate,
            (
                update_vehicle_capacity,
                dispatch_fire_trucks,
                dispatch_ambulances,
                advance_vehicles,
                apply_on_scene_effects,
                record_response_times,
                cleanup_completed_vehicles,
            )
                .chain()
                .run_if(slow_tick_guard)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

/// Only run dispatch systems every 10 ticks to limit pathfinding cost.
fn slow_tick_guard(tick: Res<crate::TickCounter>) -> bool {
    tick.0.is_multiple_of(10)
}
