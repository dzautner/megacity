//! Mode evaluation helpers for computing perceived travel times.

use crate::grid::{CellType, WorldGrid};
use crate::services::ServiceType;

use super::constants::*;
use super::types::ModeInfrastructureCache;

// =============================================================================
// Mode evaluation helpers
// =============================================================================

/// Manhattan distance between two grid positions.
pub fn manhattan_distance(from: (usize, usize), to: (usize, usize)) -> f32 {
    let dx = (from.0 as f32 - to.0 as f32).abs();
    let dy = (from.1 as f32 - to.1 as f32).abs();
    dx + dy
}

/// Evaluate walking perceived time. Always available.
pub fn evaluate_walk(distance: f32) -> f32 {
    let travel_time = distance / WALK_SPEED_MULTIPLIER;
    travel_time / WALK_COMFORT
}

/// Evaluate biking perceived time. Returns None if no bike infrastructure nearby.
pub(crate) fn evaluate_bike(
    distance: f32,
    from: (usize, usize),
    infra: &ModeInfrastructureCache,
) -> Option<f32> {
    if distance > MAX_PRACTICAL_BIKE_DISTANCE {
        return None;
    }

    // Check if there's a bike path within access distance
    let has_bike_access = infra
        .bike_paths
        .iter()
        .any(|&pos| manhattan_distance(from, pos) <= MAX_BIKE_ACCESS_DISTANCE);

    if !has_bike_access {
        return None;
    }

    let travel_time = distance / BIKE_SPEED_MULTIPLIER;
    Some(travel_time / BIKE_COMFORT)
}

/// Evaluate driving perceived time. Returns None if no vehicle road nearby.
pub(crate) fn evaluate_drive(distance: f32, from: (usize, usize), grid: &WorldGrid) -> Option<f32> {
    // Check if there's a vehicle-accessible road within 3 cells of origin
    let has_road_access = has_nearby_vehicle_road(grid, from.0, from.1, 3);

    if !has_road_access {
        return None;
    }

    let travel_time = (distance + DRIVE_PARKING_OVERHEAD) / DRIVE_SPEED_MULTIPLIER;
    Some(travel_time / DRIVE_COMFORT)
}

/// Evaluate transit perceived time. Returns None if no transit stops nearby.
pub(crate) fn evaluate_transit(
    distance: f32,
    from: (usize, usize),
    to: (usize, usize),
    infra: &ModeInfrastructureCache,
) -> Option<f32> {
    // Need a transit stop near both origin and destination
    let has_origin_stop = infra
        .transit_stops
        .iter()
        .any(|&pos| manhattan_distance(from, pos) <= MAX_TRANSIT_ACCESS_DISTANCE);

    let has_dest_stop = infra
        .transit_stops
        .iter()
        .any(|&pos| manhattan_distance(to, pos) <= MAX_TRANSIT_ACCESS_DISTANCE);

    if !has_origin_stop || !has_dest_stop {
        return None;
    }

    // Transit time = walk to stop + wait + ride + walk from stop
    let walk_access = MAX_TRANSIT_ACCESS_DISTANCE * 0.5; // avg walk to stop
    let ride_time = distance / TRANSIT_SPEED_MULTIPLIER;
    let total_time = walk_access * 2.0 + TRANSIT_WAIT_OVERHEAD + ride_time;
    Some(total_time / TRANSIT_COMFORT)
}

/// Check if there's a vehicle-accessible road within `radius` cells of (cx, cy).
pub(crate) fn has_nearby_vehicle_road(
    grid: &WorldGrid,
    cx: usize,
    cy: usize,
    radius: usize,
) -> bool {
    let x_start = cx.saturating_sub(radius);
    let y_start = cy.saturating_sub(radius);
    let x_end = (cx + radius).min(grid.width - 1);
    let y_end = (cy + radius).min(grid.height - 1);

    for y in y_start..=y_end {
        for x in x_start..=x_end {
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Road && cell.road_type.allows_vehicles() {
                return true;
            }
        }
    }
    false
}

/// Check if a service type is a transit stop.
pub(crate) fn is_transit_stop(st: ServiceType) -> bool {
    matches!(
        st,
        ServiceType::BusDepot
            | ServiceType::TrainStation
            | ServiceType::SubwayStation
            | ServiceType::TramDepot
            | ServiceType::FerryPier
    )
}
