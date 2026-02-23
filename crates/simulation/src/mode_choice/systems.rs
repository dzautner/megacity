//! ECS systems for mode choice: infrastructure caching, mode assignment, and statistics.

use bevy::prelude::*;

use crate::citizen::{Citizen, CitizenStateComp, PathRequest};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

use super::evaluation::{
    evaluate_bike, evaluate_drive, evaluate_transit, evaluate_walk, is_transit_stop,
    manhattan_distance,
};
use super::types::{ChosenTransportMode, ModeInfrastructureCache, ModeShareStats, TransportMode};

// =============================================================================
// Systems
// =============================================================================

/// Rebuild the infrastructure cache when services or roads change.
pub fn refresh_infrastructure_cache(
    services: Query<&ServiceBuilding>,
    added_services: Query<Entity, Added<ServiceBuilding>>,
    mut removed_services: RemovedComponents<ServiceBuilding>,
    grid: Res<WorldGrid>,
    mut cache: ResMut<ModeInfrastructureCache>,
) {
    let has_removals = removed_services.read().next().is_some();

    // Rebuild when services change or on first run
    if cache.transit_stops.is_empty() || !added_services.is_empty() || has_removals {
        cache.transit_stops = services
            .iter()
            .filter(|s| is_transit_stop(s.service_type))
            .map(|s| (s.grid_x, s.grid_y))
            .collect();

        // Rebuild bike path and vehicle road caches from the grid.
        // We sample a subset of cells to keep this fast -- check every 4th cell.
        cache.bike_paths.clear();
        cache.vehicle_roads.clear();

        for y in (0..GRID_HEIGHT).step_by(4) {
            for x in (0..GRID_WIDTH).step_by(4) {
                let cell = grid.get(x, y);
                if cell.cell_type == CellType::Road {
                    if cell.road_type == RoadType::Path {
                        cache.bike_paths.push((x, y));
                    } else {
                        cache.vehicle_roads.push((x, y));
                    }
                }
            }
        }
    }
}

/// Assign transport mode to citizens when they receive a path request.
///
/// This system runs BEFORE pathfinding and examines citizens that just got a
/// `PathRequest` (i.e., are about to start a trip). It evaluates available
/// modes based on distance and infrastructure, then picks the one with the
/// lowest perceived travel time.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn assign_transport_mode(
    infra: Res<ModeInfrastructureCache>,
    grid: Res<WorldGrid>,
    mut query: Query<(&PathRequest, &mut ChosenTransportMode), (With<Citizen>, Added<PathRequest>)>,
) {
    for (request, mut mode) in &mut query {
        let from = (request.from_gx, request.from_gy);
        let to = (request.to_gx, request.to_gy);

        let distance = manhattan_distance(from, to);

        // Evaluate each mode's availability and perceived time
        let walk_time = evaluate_walk(distance);
        let bike_time = evaluate_bike(distance, from, &infra);
        let drive_time = evaluate_drive(distance, from, &grid);
        let transit_time = evaluate_transit(distance, from, to, &infra);

        // Pick the mode with the lowest perceived time
        let mut best_mode = TransportMode::Walk;
        let mut best_time = walk_time;

        if let Some(bt) = bike_time {
            if bt < best_time {
                best_time = bt;
                best_mode = TransportMode::Bike;
            }
        }

        if let Some(dt) = drive_time {
            if dt < best_time {
                best_time = dt;
                best_mode = TransportMode::Drive;
            }
        }

        if let Some(tt) = transit_time {
            if tt < best_time {
                best_mode = TransportMode::Transit;
            }
        }

        mode.0 = best_mode;
    }
}

/// Update city-wide mode share statistics on the slow tick.
pub fn update_mode_share_stats(
    timer: Res<SlowTickTimer>,
    query: Query<(&CitizenStateComp, &ChosenTransportMode), With<Citizen>>,
    mut stats: ResMut<ModeShareStats>,
) {
    if !timer.should_run() {
        return;
    }

    let mut walk = 0u32;
    let mut bike = 0u32;
    let mut drive = 0u32;
    let mut transit = 0u32;

    for (state, mode) in &query {
        // Only count citizens currently commuting (active trips)
        if !state.0.is_commuting() {
            continue;
        }
        match mode.0 {
            TransportMode::Walk => walk += 1,
            TransportMode::Bike => bike += 1,
            TransportMode::Drive => drive += 1,
            TransportMode::Transit => transit += 1,
        }
    }

    let total = walk + bike + drive + transit;
    stats.walk_count = walk;
    stats.bike_count = bike;
    stats.drive_count = drive;
    stats.transit_count = transit;

    if total > 0 {
        let t = total as f32;
        stats.walk_pct = walk as f32 / t * 100.0;
        stats.bike_pct = bike as f32 / t * 100.0;
        stats.drive_pct = drive as f32 / t * 100.0;
        stats.transit_pct = transit as f32 / t * 100.0;
    } else {
        stats.walk_pct = 0.0;
        stats.bike_pct = 0.0;
        stats.drive_pct = 100.0;
        stats.transit_pct = 0.0;
    }
}
