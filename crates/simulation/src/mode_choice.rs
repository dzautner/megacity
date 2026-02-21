//! TRAF-007: Citizen Mode Choice (Car/Transit/Walk/Bike)
//!
//! Closes #858
//!
//! Citizens choose a transport mode for each trip based on distance, available
//! infrastructure, and perceived travel time. This is the core mechanic that
//! makes transit investment worthwhile.
//!
//! ## Transport Modes
//!
//! | Mode    | Speed (cells/tick) | Multiplier | Availability                    |
//! |---------|-------------------|------------|----------------------------------|
//! | Walk    | 0.3x base         | 0.30       | Always available, practical <25 cells |
//! | Bike    | 0.6x base         | 0.60       | Requires Path road type nearby   |
//! | Drive   | 1.0x base         | 1.00       | Requires road access (vehicle road) |
//! | Transit | 0.8x base         | 0.80       | Requires transit stop within 15 cells |
//!
//! ## Mode Choice Algorithm
//!
//! For each trip, perceived time = travel_time / comfort_factor:
//! - Walk:    distance / walk_speed, comfort 1.0 (pleasant for short trips)
//! - Bike:    distance / bike_speed, comfort 0.95
//! - Drive:   distance / drive_speed + parking_overhead, comfort 0.90
//! - Transit: walk_to_stop + wait_time + ride_time + walk_from_stop, comfort 0.85
//!
//! Citizens pick the mode with the lowest perceived time from the set of
//! available modes.
//!
//! ## Statistics
//!
//! `ModeShareStats` tracks the percentage of trips by each mode, updated
//! every slow tick (~10 seconds). This feeds into the transportation panel.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::citizen::{Citizen, CitizenStateComp, PathRequest};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Maximum biking distance in cells (~5km at 16m/cell ≈ 312 cells).
/// We use a practical limit of ~80 cells for gameplay.
const MAX_PRACTICAL_BIKE_DISTANCE: f32 = 80.0;

/// Maximum distance to a transit stop for transit to be available (in cells).
const MAX_TRANSIT_ACCESS_DISTANCE: f32 = 15.0;

/// Maximum distance to a bike-friendly road (Path type) for biking (in cells).
const MAX_BIKE_ACCESS_DISTANCE: f32 = 10.0;

/// Speed multiplier for walking mode (relative to base citizen speed).
pub const WALK_SPEED_MULTIPLIER: f32 = 0.30;

/// Speed multiplier for biking mode.
pub const BIKE_SPEED_MULTIPLIER: f32 = 0.60;

/// Speed multiplier for driving mode (baseline).
pub const DRIVE_SPEED_MULTIPLIER: f32 = 1.00;

/// Speed multiplier for transit mode.
pub const TRANSIT_SPEED_MULTIPLIER: f32 = 0.80;

/// Comfort factor for walking (pleasant for short trips).
const WALK_COMFORT: f32 = 1.0;

/// Comfort factor for biking (slightly less comfortable).
const BIKE_COMFORT: f32 = 0.95;

/// Comfort factor for driving (parking stress, traffic stress).
const DRIVE_COMFORT: f32 = 0.90;

/// Comfort factor for transit (waiting, transfers, crowding).
const TRANSIT_COMFORT: f32 = 0.85;

/// Overhead time for driving (finding parking, walking to/from car), in
/// equivalent cells of travel distance.
const DRIVE_PARKING_OVERHEAD: f32 = 5.0;

/// Wait time overhead for transit (average wait at stop), in equivalent cells.
const TRANSIT_WAIT_OVERHEAD: f32 = 8.0;

// =============================================================================
// TransportMode enum
// =============================================================================

/// The available transport modes for citizen trips.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Encode, Decode,
)]
pub enum TransportMode {
    /// Walking: always available, slow, best for short distances.
    Walk,
    /// Bicycle: requires bike infrastructure (Path roads), medium speed.
    Bike,
    /// Car/Drive: requires vehicle-accessible road, fastest on uncongested roads.
    #[default]
    Drive,
    /// Public transit: requires transit stops nearby, reliable for medium/long trips.
    Transit,
}

impl TransportMode {
    /// Speed multiplier relative to the base citizen movement speed.
    pub fn speed_multiplier(self) -> f32 {
        match self {
            TransportMode::Walk => WALK_SPEED_MULTIPLIER,
            TransportMode::Bike => BIKE_SPEED_MULTIPLIER,
            TransportMode::Drive => DRIVE_SPEED_MULTIPLIER,
            TransportMode::Transit => TRANSIT_SPEED_MULTIPLIER,
        }
    }

    /// Comfort factor for perceived-time calculation.
    pub fn comfort_factor(self) -> f32 {
        match self {
            TransportMode::Walk => WALK_COMFORT,
            TransportMode::Bike => BIKE_COMFORT,
            TransportMode::Drive => DRIVE_COMFORT,
            TransportMode::Transit => TRANSIT_COMFORT,
        }
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            TransportMode::Walk => "Walking",
            TransportMode::Bike => "Bicycle",
            TransportMode::Drive => "Car",
            TransportMode::Transit => "Transit",
        }
    }
}

// =============================================================================
// Components
// =============================================================================

/// Component attached to each citizen indicating their current trip's transport mode.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ChosenTransportMode(pub TransportMode);

// =============================================================================
// Mode Share Statistics
// =============================================================================

/// City-wide mode share statistics, tracking the percentage of active trips
/// by each transport mode.
#[derive(Resource, Debug, Clone, Encode, Decode)]
pub struct ModeShareStats {
    /// Number of citizens currently using each mode.
    pub walk_count: u32,
    pub bike_count: u32,
    pub drive_count: u32,
    pub transit_count: u32,
    /// Percentage (0.0-100.0) of trips by each mode.
    pub walk_pct: f32,
    pub bike_pct: f32,
    pub drive_pct: f32,
    pub transit_pct: f32,
}

impl Default for ModeShareStats {
    fn default() -> Self {
        Self {
            walk_count: 0,
            bike_count: 0,
            drive_count: 0,
            transit_count: 0,
            walk_pct: 0.0,
            bike_pct: 0.0,
            drive_pct: 100.0,
            transit_pct: 0.0,
        }
    }
}

impl ModeShareStats {
    /// Total number of active trips.
    pub fn total(&self) -> u32 {
        self.walk_count + self.bike_count + self.drive_count + self.transit_count
    }
}

// =============================================================================
// Infrastructure cache
// =============================================================================

/// Cached positions of transit stops and bike-friendly roads, rebuilt when
/// services change. Avoids per-citizen iteration over all service buildings.
#[derive(Resource, Default)]
pub struct ModeInfrastructureCache {
    /// Positions of transit stops (bus depot, train station, subway, tram, ferry).
    pub transit_stops: Vec<(usize, usize)>,
    /// Positions of bike-friendly road cells (Path type).
    pub bike_paths: Vec<(usize, usize)>,
    /// Positions of vehicle-accessible road cells (any non-Path road).
    pub vehicle_roads: Vec<(usize, usize)>,
}

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
fn evaluate_bike(
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
fn evaluate_drive(distance: f32, from: (usize, usize), grid: &WorldGrid) -> Option<f32> {
    // Check if there's a vehicle-accessible road within 3 cells of origin
    let has_road_access = has_nearby_vehicle_road(grid, from.0, from.1, 3);

    if !has_road_access {
        return None;
    }

    let travel_time = (distance + DRIVE_PARKING_OVERHEAD) / DRIVE_SPEED_MULTIPLIER;
    Some(travel_time / DRIVE_COMFORT)
}

/// Evaluate transit perceived time. Returns None if no transit stops nearby.
fn evaluate_transit(
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
fn has_nearby_vehicle_road(grid: &WorldGrid, cx: usize, cy: usize, radius: usize) -> bool {
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
fn is_transit_stop(st: ServiceType) -> bool {
    matches!(
        st,
        ServiceType::BusDepot
            | ServiceType::TrainStation
            | ServiceType::SubwayStation
            | ServiceType::TramDepot
            | ServiceType::FerryPier
    )
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for ModeShareStats {
    const SAVE_KEY: &'static str = "mode_share_stats";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no trips recorded
        if self.total() == 0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct ModeChoicePlugin;

impl Plugin for ModeChoicePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModeShareStats>()
            .init_resource::<ModeInfrastructureCache>()
            .add_systems(
                FixedUpdate,
                (
                    refresh_infrastructure_cache,
                    assign_transport_mode
                        .after(refresh_infrastructure_cache)
                        .before(crate::movement::process_path_requests),
                    update_mode_share_stats,
                )
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ModeShareStats>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // TransportMode tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_transport_mode_speed_multipliers() {
        assert!(
            (TransportMode::Walk.speed_multiplier() - WALK_SPEED_MULTIPLIER).abs() < f32::EPSILON
        );
        assert!(
            (TransportMode::Bike.speed_multiplier() - BIKE_SPEED_MULTIPLIER).abs() < f32::EPSILON
        );
        assert!(
            (TransportMode::Drive.speed_multiplier() - DRIVE_SPEED_MULTIPLIER).abs() < f32::EPSILON
        );
        assert!(
            (TransportMode::Transit.speed_multiplier() - TRANSIT_SPEED_MULTIPLIER).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_transport_mode_comfort_factors() {
        assert!((TransportMode::Walk.comfort_factor() - WALK_COMFORT).abs() < f32::EPSILON);
        assert!((TransportMode::Bike.comfort_factor() - BIKE_COMFORT).abs() < f32::EPSILON);
        assert!((TransportMode::Drive.comfort_factor() - DRIVE_COMFORT).abs() < f32::EPSILON);
        assert!((TransportMode::Transit.comfort_factor() - TRANSIT_COMFORT).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transport_mode_labels() {
        assert_eq!(TransportMode::Walk.label(), "Walking");
        assert_eq!(TransportMode::Bike.label(), "Bicycle");
        assert_eq!(TransportMode::Drive.label(), "Car");
        assert_eq!(TransportMode::Transit.label(), "Transit");
    }

    #[test]
    fn test_default_mode_is_drive() {
        assert_eq!(TransportMode::default(), TransportMode::Drive);
    }

    // -------------------------------------------------------------------------
    // Distance helpers
    // -------------------------------------------------------------------------

    #[test]
    fn test_manhattan_distance() {
        assert!((manhattan_distance((0, 0), (10, 10)) - 20.0).abs() < f32::EPSILON);
        assert!((manhattan_distance((5, 5), (5, 5)) - 0.0).abs() < f32::EPSILON);
        assert!((manhattan_distance((0, 0), (3, 4)) - 7.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Mode evaluation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_walk_always_available() {
        let time = evaluate_walk(10.0);
        assert!(time > 0.0);
    }

    #[test]
    fn test_walk_perceived_time_calculation() {
        // walk time = distance / walk_speed / comfort
        let distance = 10.0;
        let expected = (distance / WALK_SPEED_MULTIPLIER) / WALK_COMFORT;
        let actual = evaluate_walk(distance);
        assert!((actual - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bike_unavailable_without_infrastructure() {
        let infra = ModeInfrastructureCache::default();
        assert!(evaluate_bike(10.0, (128, 128), &infra).is_none());
    }

    #[test]
    fn test_bike_available_with_nearby_path() {
        let infra = ModeInfrastructureCache {
            bike_paths: vec![(128, 128)],
            ..Default::default()
        };
        assert!(evaluate_bike(10.0, (128, 128), &infra).is_some());
    }

    #[test]
    fn test_bike_unavailable_for_long_distance() {
        let infra = ModeInfrastructureCache {
            bike_paths: vec![(128, 128)],
            ..Default::default()
        };
        assert!(evaluate_bike(MAX_PRACTICAL_BIKE_DISTANCE + 1.0, (128, 128), &infra).is_none());
    }

    #[test]
    fn test_drive_unavailable_without_roads() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert!(evaluate_drive(10.0, (128, 128), &grid).is_none());
    }

    #[test]
    fn test_drive_available_with_road() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(128, 128).cell_type = CellType::Road;
        grid.get_mut(128, 128).road_type = RoadType::Local;
        assert!(evaluate_drive(10.0, (128, 128), &grid).is_some());
    }

    #[test]
    fn test_transit_unavailable_without_stops() {
        let infra = ModeInfrastructureCache::default();
        assert!(evaluate_transit(10.0, (128, 128), (140, 140), &infra).is_none());
    }

    #[test]
    fn test_transit_available_with_stops_at_both_ends() {
        let infra = ModeInfrastructureCache {
            transit_stops: vec![(128, 128), (140, 140)],
            ..Default::default()
        };
        assert!(evaluate_transit(10.0, (128, 128), (140, 140), &infra).is_some());
    }

    #[test]
    fn test_transit_unavailable_with_stop_at_origin_only() {
        let infra = ModeInfrastructureCache {
            transit_stops: vec![(128, 128)],
            ..Default::default()
        };
        // Destination (200, 200) is far from any transit stop
        assert!(evaluate_transit(10.0, (128, 128), (200, 200), &infra).is_none());
    }

    // -------------------------------------------------------------------------
    // Mode choice preference tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_walking_preferred_for_short_distance() {
        // For a very short trip (2 cells), walking should be preferred over driving
        // because driving has a parking overhead of 5 cells.
        // Walk: 2 / 0.3 / 1.0 = 6.67
        // Drive: (2 + 5) / 1.0 / 0.9 = 7.78
        let distance = 2.0;
        let walk_time = evaluate_walk(distance);

        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(128, 128).cell_type = CellType::Road;
        grid.get_mut(128, 128).road_type = RoadType::Local;
        let drive_time = evaluate_drive(distance, (128, 128), &grid).unwrap();

        assert!(
            walk_time < drive_time,
            "For 2-cell trip, walking ({walk_time}) should be faster than driving ({drive_time})"
        );
    }

    #[test]
    fn test_driving_preferred_for_long_distance() {
        // For a long trip (100 cells), driving should be preferred over walking.
        let distance = 100.0;
        let walk_time = evaluate_walk(distance);

        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(50, 50).cell_type = CellType::Road;
        grid.get_mut(50, 50).road_type = RoadType::Local;
        let drive_time = evaluate_drive(distance, (50, 50), &grid).unwrap();

        assert!(
            drive_time < walk_time,
            "For 100-cell trip, driving ({drive_time}) should be faster than walking ({walk_time})"
        );
    }

    #[test]
    fn test_bike_faster_than_walk_for_medium_distance() {
        let distance = 30.0;
        let walk_time = evaluate_walk(distance);

        let infra = ModeInfrastructureCache {
            bike_paths: vec![(128, 128)],
            ..Default::default()
        };
        let bike_time = evaluate_bike(distance, (128, 128), &infra).unwrap();

        assert!(
            bike_time < walk_time,
            "For 30-cell trip, biking ({bike_time}) should be faster than walking ({walk_time})"
        );
    }

    // -------------------------------------------------------------------------
    // Mode share stats tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mode_share_default() {
        let stats = ModeShareStats::default();
        assert_eq!(stats.total(), 0);
        assert!((stats.drive_pct - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mode_share_total() {
        let stats = ModeShareStats {
            walk_count: 10,
            bike_count: 20,
            drive_count: 50,
            transit_count: 20,
            ..Default::default()
        };
        assert_eq!(stats.total(), 100);
    }

    // -------------------------------------------------------------------------
    // Transit stop classification
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_transit_stop() {
        assert!(is_transit_stop(ServiceType::BusDepot));
        assert!(is_transit_stop(ServiceType::TrainStation));
        assert!(is_transit_stop(ServiceType::SubwayStation));
        assert!(is_transit_stop(ServiceType::TramDepot));
        assert!(is_transit_stop(ServiceType::FerryPier));
        assert!(!is_transit_stop(ServiceType::FireStation));
        assert!(!is_transit_stop(ServiceType::Hospital));
    }

    // -------------------------------------------------------------------------
    // Saveable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let stats = ModeShareStats::default();
        assert!(stats.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_non_zero() {
        use crate::Saveable;
        let stats = ModeShareStats {
            walk_count: 5,
            bike_count: 10,
            drive_count: 80,
            transit_count: 5,
            walk_pct: 5.0,
            bike_pct: 10.0,
            drive_pct: 80.0,
            transit_pct: 5.0,
        };
        assert!(stats.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let stats = ModeShareStats {
            walk_count: 10,
            bike_count: 20,
            drive_count: 50,
            transit_count: 20,
            walk_pct: 10.0,
            bike_pct: 20.0,
            drive_pct: 50.0,
            transit_pct: 20.0,
        };
        let bytes = stats.save_to_bytes().expect("should serialize");
        let restored = ModeShareStats::load_from_bytes(&bytes);
        assert_eq!(restored.walk_count, 10);
        assert_eq!(restored.bike_count, 20);
        assert_eq!(restored.drive_count, 50);
        assert_eq!(restored.transit_count, 20);
        assert!((restored.walk_pct - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(ModeShareStats::SAVE_KEY, "mode_share_stats");
    }

    // -------------------------------------------------------------------------
    // Nearby vehicle road check
    // -------------------------------------------------------------------------

    #[test]
    fn test_has_nearby_vehicle_road_none() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert!(!has_nearby_vehicle_road(&grid, 128, 128, 3));
    }

    #[test]
    fn test_has_nearby_vehicle_road_local() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(129, 128).cell_type = CellType::Road;
        grid.get_mut(129, 128).road_type = RoadType::Local;
        assert!(has_nearby_vehicle_road(&grid, 128, 128, 3));
    }

    #[test]
    fn test_has_nearby_vehicle_road_path_not_vehicle() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(129, 128).cell_type = CellType::Road;
        grid.get_mut(129, 128).road_type = RoadType::Path;
        // Path roads don't allow vehicles
        assert!(!has_nearby_vehicle_road(&grid, 128, 128, 3));
    }

    #[test]
    fn test_has_nearby_vehicle_road_out_of_range() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(135, 128).cell_type = CellType::Road;
        grid.get_mut(135, 128).road_type = RoadType::Local;
        // Road is 7 cells away, radius is 3
        assert!(!has_nearby_vehicle_road(&grid, 128, 128, 3));
    }
}
