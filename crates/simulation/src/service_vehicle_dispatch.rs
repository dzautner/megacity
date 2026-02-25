//! SVC-003: Service Vehicle Dispatch System
//!
//! Spawns service vehicles (fire trucks, ambulances, police cars) from service
//! buildings to respond to incidents. Each service building has a vehicle pool.
//! Dispatch selects the nearest available vehicle via BFS road distance.
//! Vehicles travel to the scene, spend time resolving, then return to station.

use std::collections::VecDeque;

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::fire::OnFire;
use crate::grid::{CellType, WorldGrid};
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub enum VehicleType {
    FireTruck,
    Ambulance,
    PoliceCar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub enum VehicleStatus {
    Responding,
    AtScene,
    Returning,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// A dispatched service vehicle entity.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ServiceVehicle {
    pub vehicle_type: VehicleType,
    pub owning_station: Entity,
    pub status: VehicleStatus,
    pub target_x: usize,
    pub target_y: usize,
    pub station_x: usize,
    pub station_y: usize,
    pub travel_ticks_remaining: u32,
    pub scene_ticks_remaining: u32,
}

/// Vehicle pool attached to service buildings that can dispatch.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct VehiclePool {
    pub total_vehicles: u32,
    pub dispatched_count: u32,
}

impl VehiclePool {
    pub fn new(service_type: ServiceType) -> Self {
        Self {
            total_vehicles: vehicle_pool_size(service_type),
            dispatched_count: 0,
        }
    }

    pub fn available(&self) -> u32 {
        self.total_vehicles.saturating_sub(self.dispatched_count)
    }
}

/// A pending incident needing vehicle dispatch.
#[derive(Debug, Clone)]
pub struct IncidentRequest {
    pub vehicle_type: VehicleType,
    pub target_x: usize,
    pub target_y: usize,
}

/// Resource holding pending incident requests.
#[derive(Resource, Default)]
pub struct PendingIncidents {
    pub requests: Vec<IncidentRequest>,
}

// ---------------------------------------------------------------------------
// Metrics (saveable)
// ---------------------------------------------------------------------------

/// Response time statistics per vehicle type.
#[derive(Resource, Default, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct DispatchMetrics {
    pub fire_dispatches: u32,
    pub fire_total_response_ticks: u32,
    pub medical_dispatches: u32,
    pub medical_total_response_ticks: u32,
    pub police_dispatches: u32,
    pub police_total_response_ticks: u32,
    pub failed_dispatches: u32,
}

impl DispatchMetrics {
    pub fn avg_fire_response(&self) -> f32 {
        if self.fire_dispatches == 0 { 0.0 }
        else { self.fire_total_response_ticks as f32 / self.fire_dispatches as f32 }
    }
    pub fn avg_medical_response(&self) -> f32 {
        if self.medical_dispatches == 0 { 0.0 }
        else { self.medical_total_response_ticks as f32 / self.medical_dispatches as f32 }
    }
    pub fn avg_police_response(&self) -> f32 {
        if self.police_dispatches == 0 { 0.0 }
        else { self.police_total_response_ticks as f32 / self.police_dispatches as f32 }
    }
}

impl crate::Saveable for DispatchMetrics {
    const SAVE_KEY: &'static str = "vehicle_dispatch";
    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.fire_dispatches == 0
            && self.medical_dispatches == 0
            && self.police_dispatches == 0
        {
            return None;
        }
        Some(bitcode::encode(self))
    }
    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Constants & helpers
// ---------------------------------------------------------------------------

const VEHICLE_SPEED: f32 = 2.0;
const SCENE_DURATION_TICKS: u32 = 30;
const MAX_DISPATCH_DISTANCE: u32 = 80;
const SCAN_INTERVAL: u32 = 5;

/// Number of vehicles per service building type.
pub fn vehicle_pool_size(st: ServiceType) -> u32 {
    match st {
        ServiceType::FireStation => 3,
        ServiceType::FireHouse => 1,
        ServiceType::FireHQ => 6,
        ServiceType::PoliceStation => 4,
        ServiceType::PoliceKiosk => 1,
        ServiceType::PoliceHQ => 8,
        ServiceType::Hospital => 4,
        ServiceType::MedicalClinic => 1,
        ServiceType::MedicalCenter => 8,
        _ => 0,
    }
}

fn vehicle_type_for_service(st: ServiceType) -> Option<VehicleType> {
    if ServiceBuilding::is_fire(st) {
        Some(VehicleType::FireTruck)
    } else if ServiceBuilding::is_police(st) {
        Some(VehicleType::PoliceCar)
    } else if ServiceBuilding::is_health(st) {
        Some(VehicleType::Ambulance)
    } else {
        None
    }
}

/// BFS road distance from (sx,sy) to (tx,ty). Returns None if unreachable.
pub fn bfs_road_distance(
    grid: &WorldGrid, sx: usize, sy: usize, tx: usize, ty: usize,
) -> Option<u32> {
    let total = GRID_WIDTH * GRID_HEIGHT;
    let mut dist = vec![u32::MAX; total];
    let mut queue = VecDeque::with_capacity(256);
    seed_bfs(grid, sx, sy, &mut dist, &mut queue);

    while let Some((cx, cy)) = queue.pop_front() {
        let d = dist[cy * GRID_WIDTH + cx];
        if d >= MAX_DISPATCH_DISTANCE { continue; }
        // Adjacent to target or at target?
        let dx = (cx as i32 - tx as i32).unsigned_abs();
        let dy = (cy as i32 - ty as i32).unsigned_abs();
        if dx + dy <= 1 { return Some(d); }
        for (nx, ny) in neighbors4(cx, cy) {
            if !grid.in_bounds(nx, ny) { continue; }
            let ni = ny * GRID_WIDTH + nx;
            let nd = d + 1;
            if nd < dist[ni] && grid.get(nx, ny).cell_type == CellType::Road {
                dist[ni] = nd;
                queue.push_back((nx, ny));
            }
        }
    }
    None
}

fn seed_bfs(
    grid: &WorldGrid, sx: usize, sy: usize,
    dist: &mut [u32], queue: &mut VecDeque<(usize, usize)>,
) {
    if grid.in_bounds(sx, sy) && grid.get(sx, sy).cell_type == CellType::Road {
        dist[sy * GRID_WIDTH + sx] = 0;
        queue.push_back((sx, sy));
        return;
    }
    for (nx, ny) in neighbors4(sx, sy) {
        if !grid.in_bounds(nx, ny) { continue; }
        let ni = ny * GRID_WIDTH + nx;
        if grid.get(nx, ny).cell_type == CellType::Road && dist[ni] == u32::MAX {
            dist[ni] = 1;
            queue.push_back((nx, ny));
        }
    }
}

fn neighbors4(x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(4);
    if x > 0 { out.push((x - 1, y)); }
    if y > 0 { out.push((x, y - 1)); }
    out.push((x + 1, y));
    out.push((x, y + 1));
    out
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn attach_vehicle_pools(
    mut commands: Commands,
    new_services: Query<(Entity, &ServiceBuilding), Added<ServiceBuilding>>,
) {
    for (entity, service) in &new_services {
        if vehicle_pool_size(service.service_type) > 0 {
            commands.entity(entity).insert(VehiclePool::new(service.service_type));
        }
    }
}

fn scan_fire_incidents(
    timer: Res<SlowTickTimer>,
    burning: Query<(&crate::buildings::Building, &OnFire)>,
    vehicles: Query<&ServiceVehicle>,
    mut pending: ResMut<PendingIncidents>,
) {
    if !timer.counter.is_multiple_of(SCAN_INTERVAL) { return; }
    let active: std::collections::HashSet<(usize, usize)> = vehicles
        .iter()
        .filter(|v| v.vehicle_type == VehicleType::FireTruck && v.status != VehicleStatus::Returning)
        .map(|v| (v.target_x, v.target_y))
        .collect();
    for (building, _) in &burning {
        let pos = (building.grid_x, building.grid_y);
        if !active.contains(&pos) {
            pending.requests.push(IncidentRequest {
                vehicle_type: VehicleType::FireTruck,
                target_x: building.grid_x,
                target_y: building.grid_y,
            });
        }
    }
}

fn dispatch_vehicles(
    mut commands: Commands,
    mut pending: ResMut<PendingIncidents>,
    grid: Res<WorldGrid>,
    mut stations: Query<(Entity, &ServiceBuilding, &mut VehiclePool)>,
    mut metrics: ResMut<DispatchMetrics>,
) {
    let requests: Vec<IncidentRequest> = pending.requests.drain(..).collect();
    for request in requests {
        let mut best: Option<(Entity, usize, usize, u32, VehicleType)> = None;
        for (entity, service, pool) in &stations {
            if pool.available() == 0 { continue; }
            let Some(vt) = vehicle_type_for_service(service.service_type) else { continue };
            if vt != request.vehicle_type { continue; }
            if let Some(dist) = bfs_road_distance(
                &grid, service.grid_x, service.grid_y, request.target_x, request.target_y,
            ) {
                let dominated = best.as_ref().is_some_and(|b| dist >= b.3);
                if !dominated { best = Some((entity, service.grid_x, service.grid_y, dist, vt)); }
            }
        }
        if let Some((station_entity, sx, sy, dist, vt)) = best {
            let travel = (dist as f32 / VEHICLE_SPEED).ceil() as u32;
            match vt {
                VehicleType::FireTruck => {
                    metrics.fire_dispatches += 1;
                    metrics.fire_total_response_ticks += travel;
                }
                VehicleType::Ambulance => {
                    metrics.medical_dispatches += 1;
                    metrics.medical_total_response_ticks += travel;
                }
                VehicleType::PoliceCar => {
                    metrics.police_dispatches += 1;
                    metrics.police_total_response_ticks += travel;
                }
            }
            commands.spawn(ServiceVehicle {
                vehicle_type: vt,
                owning_station: station_entity,
                status: VehicleStatus::Responding,
                target_x: request.target_x, target_y: request.target_y,
                station_x: sx, station_y: sy,
                travel_ticks_remaining: travel,
                scene_ticks_remaining: SCENE_DURATION_TICKS,
            });
            if let Ok((_, _, mut pool)) = stations.get_mut(station_entity) {
                pool.dispatched_count += 1;
            }
        } else {
            metrics.failed_dispatches += 1;
        }
    }
}

fn update_vehicles(
    mut commands: Commands,
    mut vehicles: Query<(Entity, &mut ServiceVehicle)>,
    mut stations: Query<&mut VehiclePool>,
) {
    for (entity, mut v) in &mut vehicles {
        match v.status {
            VehicleStatus::Responding => {
                if v.travel_ticks_remaining > 0 { v.travel_ticks_remaining -= 1; }
                else { v.status = VehicleStatus::AtScene; }
            }
            VehicleStatus::AtScene => {
                if v.scene_ticks_remaining > 0 { v.scene_ticks_remaining -= 1; }
                else {
                    v.status = VehicleStatus::Returning;
                    let dx = (v.target_x as i32 - v.station_x as i32).unsigned_abs();
                    let dy = (v.target_y as i32 - v.station_y as i32).unsigned_abs();
                    v.travel_ticks_remaining = ((dx + dy) as f32 / VEHICLE_SPEED).ceil() as u32;
                }
            }
            VehicleStatus::Returning => {
                if v.travel_ticks_remaining > 0 { v.travel_ticks_remaining -= 1; }
                else {
                    if let Ok(mut pool) = stations.get_mut(v.owning_station) {
                        pool.dispatched_count = pool.dispatched_count.saturating_sub(1);
                    }
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct ServiceVehicleDispatchPlugin;

impl Plugin for ServiceVehicleDispatchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingIncidents>();
        app.init_resource::<DispatchMetrics>();
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<DispatchMetrics>();
        app.add_systems(
            FixedUpdate,
            (attach_vehicle_pools, scan_fire_incidents, dispatch_vehicles, update_vehicles)
                .chain()
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
