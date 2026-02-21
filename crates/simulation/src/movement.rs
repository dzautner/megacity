use std::collections::HashSet;
use std::sync::Arc;

use bevy::prelude::*;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use std::time::{Duration, Instant};

use crate::buildings::Building;
use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, HomeLocation, Needs, PathCache,
    PathRequest, Position, Velocity, WorkLocation,
};
use crate::game_params::GameParams;
use crate::grid::{RoadType, WorldGrid};
use crate::lod::LodTier;
use crate::mode_choice::ChosenTransportMode;
use crate::pathfinding_sys::nearest_road_grid;
use crate::road_graph_csr::{csr_find_path_with_traffic, CsrGraph, PathfindingData};
use crate::roads::{RoadNetwork, RoadNode};
use crate::services::{ServiceBuilding, ServiceType};
use crate::time_of_day::GameClock;
use crate::traffic::TrafficGrid;
use crate::traffic_congestion::TrafficCongestion;

/// Maximum number of async pathfinding tasks to spawn per tick.
/// This controls how many new tasks are dispatched each frame, not the total
/// number of in-flight tasks. Tasks complete across multiple cores so effective
/// throughput is much higher than the old synchronous cap.
const MAX_SPAWN_PER_TICK: usize = 256;

/// Fallback count limit for WASM where Instant has poor resolution.
const MAX_PATHS_PER_TICK_WASM: usize = 256;

/// Time budget for synchronous pathfinding per tick (WASM fallback).
const PATH_BUDGET_WASM: Duration = Duration::from_millis(2);

/// Per-citizen tick counter for activity durations
#[derive(Component, Debug, Clone, Default)]
pub struct ActivityTimer(pub u32);

/// Marker component for citizens whose pathfinding is being computed
/// asynchronously. Holds the spawned task and the target state to transition
/// to once the path is ready.
#[derive(Component)]
pub struct ComputingPath {
    task: Task<Option<Vec<RoadNode>>>,
    target_state: CitizenState,
}

/// Shared read-only snapshot of pathfinding data (CSR graph + road types +
/// traffic density), wrapped in `Arc` so async tasks can reference it
/// without cloning per-task.
#[derive(Resource)]
pub struct PathfindingSnapshot {
    pub data: Arc<PathfindingData>,
    /// Monotonic version counter; bumped whenever the snapshot is refreshed.
    pub version: u64,
}

impl Default for PathfindingSnapshot {
    fn default() -> Self {
        Self {
            data: Arc::new(PathfindingData::default()),
            version: 0,
        }
    }
}

/// Cached destination lists to avoid per-tick Vec allocations.
#[derive(Resource, Default)]
pub struct DestinationCache {
    pub shops: Vec<(usize, usize)>,
    pub leisure: Vec<(usize, usize)>,
    pub schools: Vec<(usize, usize)>,
}

/// Rebuild destination caches only when buildings or services change.
pub fn refresh_destination_cache(
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
    added_buildings: Query<Entity, Added<Building>>,
    added_services: Query<Entity, Added<ServiceBuilding>>,
    mut removed_buildings: RemovedComponents<Building>,
    mut removed_services: RemovedComponents<ServiceBuilding>,
    mut cache: ResMut<DestinationCache>,
) {
    let has_removals =
        removed_buildings.read().next().is_some() || removed_services.read().next().is_some();

    // Rebuild when entities are added, removed, or on first run when cache is empty
    if cache.shops.is_empty()
        || !added_buildings.is_empty()
        || !added_services.is_empty()
        || has_removals
    {
        cache.shops = buildings
            .iter()
            .filter(|b| b.zone_type.is_commercial() || b.zone_type.is_mixed_use())
            .map(|b| (b.grid_x, b.grid_y))
            .collect();

        cache.leisure = services
            .iter()
            .filter(|s| is_leisure_service(s.service_type))
            .map(|s| (s.grid_x, s.grid_y))
            .collect();

        cache.schools = services
            .iter()
            .filter(|s| is_school_service(s.service_type))
            .map(|s| (s.grid_x, s.grid_y))
            .collect();
    }
}

/// Invalidate cached paths that reference recently-deleted road nodes.
///
/// When a road is bulldozed, `RoadNetwork::remove_road` pushes the deleted
/// node into `recently_removed`. This system drains that buffer, builds a
/// lookup set, and clears the `PathCache` of any commuting citizen whose
/// remaining waypoints overlap the deleted nodes. Affected citizens are sent
/// home immediately.
pub fn invalidate_paths_on_road_removal(
    mut roads: ResMut<RoadNetwork>,
    mut query: Query<(&mut PathCache, &mut CitizenStateComp), With<Citizen>>,
) {
    if roads.recently_removed.is_empty() {
        return;
    }

    let removed: HashSet<RoadNode> = roads.drain_removed();

    for (mut path, mut state) in &mut query {
        // Only invalidate citizens that are actively commuting with a non-empty path
        if !state.0.is_commuting() || path.is_complete() {
            continue;
        }

        // Check remaining waypoints (from current_index onward) against removed set
        let stale = path.waypoints[path.current_index..]
            .iter()
            .any(|wp| removed.contains(wp));

        if stale {
            // Clear the path and send citizen home
            *path = PathCache::new(Vec::new());
            state.0 = CitizenState::AtHome;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn citizen_state_machine(
    clock: Res<GameClock>,
    dest_cache: Res<DestinationCache>,
    game_params: Res<GameParams>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut CitizenStateComp,
            &PathCache,
            &HomeLocation,
            Option<&WorkLocation>,
            &CitizenDetails,
            &Needs,
            &mut ActivityTimer,
            Option<&LodTier>,
            &Position,
        ),
        (With<Citizen>, Without<PathRequest>, Without<ComputingPath>),
    >,
) {
    if clock.paused {
        return;
    }

    let hour = clock.hour_of_day();

    let shops = &dest_cache.shops;
    let leisure_spots = &dest_cache.leisure;
    let school_spots = &dest_cache.schools;

    for (entity, mut state, path, home, work, details, needs, mut timer, lod, pos) in &mut query {
        // Per-entity departure jitter: spread departures across the commute window
        let jitter = entity.index() % 120;

        // Skip abstract-tier citizens (state machine only, no pathfinding)
        if lod == Some(&LodTier::Abstract) {
            match state.0 {
                CitizenState::AtHome if clock.is_morning_commute() && work.is_some() => {
                    state.0 = CitizenState::Working;
                }
                CitizenState::Working if clock.is_evening_commute() => {
                    state.0 = CitizenState::AtHome;
                }
                _ => {}
            }
            continue;
        }

        match state.0 {
            // ---- AT HOME ----
            CitizenState::AtHome => {
                let life_stage = details.life_stage();
                let minute = ((clock.hour - clock.hour.floor()) * 60.0) as u32;

                // Children: go to school during school hours (with jitter)
                if life_stage.should_attend_school()
                    && (game_params.citizen.school_hours_start
                        ..game_params.citizen.school_hours_end)
                        .contains(&hour)
                    && (minute % 60 == jitter % 60)
                {
                    if let Some(dest) = find_nearest(school_spots, home.grid_x, home.grid_y, 30) {
                        commands.entity(entity).insert(PathRequest {
                            from_gx: home.grid_x,
                            from_gy: home.grid_y,
                            to_gx: dest.0,
                            to_gy: dest.1,
                            target_state: CitizenState::CommutingToSchool,
                        });
                        continue;
                    }
                }

                // Working adults: morning commute
                if life_stage.can_work()
                    && clock.is_morning_commute()
                    && (minute % 60 == jitter % 60)
                {
                    if let Some(work_loc) = work {
                        commands.entity(entity).insert(PathRequest {
                            from_gx: home.grid_x,
                            from_gy: home.grid_y,
                            to_gx: work_loc.grid_x,
                            to_gy: work_loc.grid_y,
                            target_state: CitizenState::CommutingToWork,
                        });
                        continue;
                    }
                }

                // Retired/unemployed: go shopping or leisure based on needs
                if !life_stage.should_attend_school() && (10..=20).contains(&hour) {
                    if needs.hunger < 40.0 {
                        if let Some(dest) = find_nearest(shops, home.grid_x, home.grid_y, 25) {
                            timer.0 = 0;
                            commands.entity(entity).insert(PathRequest {
                                from_gx: home.grid_x,
                                from_gy: home.grid_y,
                                to_gx: dest.0,
                                to_gy: dest.1,
                                target_state: CitizenState::CommutingToShop,
                            });
                            continue;
                        }
                    }
                    if needs.fun < 30.0 || needs.social < 30.0 {
                        if let Some(dest) =
                            find_nearest(leisure_spots, home.grid_x, home.grid_y, 25)
                        {
                            timer.0 = 0;
                            commands.entity(entity).insert(PathRequest {
                                from_gx: home.grid_x,
                                from_gy: home.grid_y,
                                to_gx: dest.0,
                                to_gy: dest.1,
                                target_state: CitizenState::CommutingToLeisure,
                            });
                            continue;
                        }
                    }
                }
            }

            // ---- COMMUTING TO WORK ----
            CitizenState::CommutingToWork => {
                if path.is_complete() {
                    state.0 = CitizenState::Working;
                }
            }

            // ---- WORKING ----
            CitizenState::Working => {
                if clock.is_evening_commute() {
                    // After work: check if needs drive a detour
                    if needs.hunger < 35.0 {
                        if let Some(work_loc) = work {
                            if let Some(dest) =
                                find_nearest(shops, work_loc.grid_x, work_loc.grid_y, 20)
                            {
                                timer.0 = 0;
                                commands.entity(entity).insert(PathRequest {
                                    from_gx: work_loc.grid_x,
                                    from_gy: work_loc.grid_y,
                                    to_gx: dest.0,
                                    to_gy: dest.1,
                                    target_state: CitizenState::CommutingToShop,
                                });
                                continue;
                            }
                        }
                    }
                    if needs.fun < 25.0 || needs.social < 25.0 {
                        if let Some(work_loc) = work {
                            if let Some(dest) =
                                find_nearest(leisure_spots, work_loc.grid_x, work_loc.grid_y, 20)
                            {
                                timer.0 = 0;
                                commands.entity(entity).insert(PathRequest {
                                    from_gx: work_loc.grid_x,
                                    from_gy: work_loc.grid_y,
                                    to_gx: dest.0,
                                    to_gy: dest.1,
                                    target_state: CitizenState::CommutingToLeisure,
                                });
                                continue;
                            }
                        }
                    }

                    // Default: commute home
                    let from = work
                        .map(|w| (w.grid_x, w.grid_y))
                        .unwrap_or((home.grid_x, home.grid_y));
                    commands.entity(entity).insert(PathRequest {
                        from_gx: from.0,
                        from_gy: from.1,
                        to_gx: home.grid_x,
                        to_gy: home.grid_y,
                        target_state: CitizenState::CommutingHome,
                    });
                }
            }

            // ---- COMMUTING HOME ----
            CitizenState::CommutingHome => {
                if path.is_complete() {
                    state.0 = CitizenState::AtHome;
                }
            }

            // ---- COMMUTING TO SHOP ----
            CitizenState::CommutingToShop => {
                if path.is_complete() {
                    timer.0 = 0;
                    state.0 = CitizenState::Shopping;
                }
            }

            // ---- SHOPPING ----
            CitizenState::Shopping => {
                timer.0 += 1;
                if timer.0 >= game_params.citizen.shopping_duration_ticks {
                    let (gx, gy) = WorldGrid::world_to_grid(pos.x, pos.y);
                    commands.entity(entity).insert(PathRequest {
                        from_gx: gx.max(0) as usize,
                        from_gy: gy.max(0) as usize,
                        to_gx: home.grid_x,
                        to_gy: home.grid_y,
                        target_state: CitizenState::CommutingHome,
                    });
                }
            }

            // ---- COMMUTING TO LEISURE ----
            CitizenState::CommutingToLeisure => {
                if path.is_complete() {
                    timer.0 = 0;
                    state.0 = CitizenState::AtLeisure;
                }
            }

            // ---- AT LEISURE ----
            CitizenState::AtLeisure => {
                timer.0 += 1;
                if timer.0 >= game_params.citizen.leisure_duration_ticks || hour >= 21 {
                    let (gx, gy) = WorldGrid::world_to_grid(pos.x, pos.y);
                    commands.entity(entity).insert(PathRequest {
                        from_gx: gx.max(0) as usize,
                        from_gy: gy.max(0) as usize,
                        to_gx: home.grid_x,
                        to_gy: home.grid_y,
                        target_state: CitizenState::CommutingHome,
                    });
                }
            }

            // ---- COMMUTING TO SCHOOL ----
            CitizenState::CommutingToSchool => {
                if path.is_complete() {
                    state.0 = CitizenState::AtSchool;
                }
            }

            // ---- AT SCHOOL ----
            CitizenState::AtSchool => {
                if hour >= game_params.citizen.school_hours_end {
                    let (gx, gy) = WorldGrid::world_to_grid(pos.x, pos.y);
                    commands.entity(entity).insert(PathRequest {
                        from_gx: gx.max(0) as usize,
                        from_gy: gy.max(0) as usize,
                        to_gx: home.grid_x,
                        to_gy: home.grid_y,
                        target_state: CitizenState::CommutingHome,
                    });
                }
            }
        }
    }
}

/// Refresh the `PathfindingSnapshot` each tick.
///
/// The snapshot is always rebuilt because traffic density changes every tick.
/// Building a new `PathfindingData` is cheap: it copies the CSR graph data
/// (only when changed), extracts road types per CSR node, and copies the
/// traffic density flat array (~128 KB for 256x256).
pub fn update_pathfinding_snapshot(
    csr: Res<CsrGraph>,
    grid: Res<WorldGrid>,
    traffic: Res<TrafficGrid>,
    mut snapshot: ResMut<PathfindingSnapshot>,
) {
    // Build compact per-node road type lookup from grid
    let node_road_types: Vec<RoadType> = csr
        .nodes
        .iter()
        .map(|n| grid.get(n.0, n.1).road_type)
        .collect();

    snapshot.data = Arc::new(PathfindingData {
        nodes: csr.nodes.clone(),
        node_offsets: csr.node_offsets.clone(),
        edges: csr.edges.clone(),
        weights: csr.weights.clone(),
        node_road_types,
        traffic_density: traffic.density.clone(),
        traffic_width: traffic.width,
    });
    snapshot.version += 1;
}

/// Dispatch pathfinding requests as async tasks on the `AsyncComputeTaskPool`.
///
/// For each pending `PathRequest`, this system:
/// 1. Resolves start/goal grid cells to road nodes (fast, synchronous)
/// 2. Spawns an A* computation on the async task pool
/// 3. Replaces the `PathRequest` with a `ComputingPath` marker holding the task
///
/// On WASM (single-threaded), falls back to synchronous processing since the
/// async task pool has no extra threads.
pub fn process_path_requests(
    mut commands: Commands,
    grid: Res<WorldGrid>,
    csr: Res<CsrGraph>,
    traffic: Res<TrafficGrid>,
    snapshot: Res<PathfindingSnapshot>,
    mut query: Query<(Entity, &PathRequest, &mut PathCache, &mut CitizenStateComp), With<Citizen>>,
) {
    // On WASM, use synchronous processing (no multi-threading available)
    if cfg!(target_arch = "wasm32") {
        let start = Instant::now();
        for (processed, (entity, request, mut path, mut state)) in query.iter_mut().enumerate() {
            if processed >= MAX_PATHS_PER_TICK_WASM {
                break;
            }
            if start.elapsed() >= PATH_BUDGET_WASM {
                break;
            }

            if let Some(route) = compute_route_csr(
                &grid,
                &csr,
                &traffic,
                request.from_gx,
                request.from_gy,
                request.to_gx,
                request.to_gy,
            ) {
                *path = PathCache::new(route);
                state.0 = request.target_state;
            }
            commands.entity(entity).remove::<PathRequest>();
        }
        return;
    }

    // Native: spawn async tasks for pathfinding
    let data = Arc::clone(&snapshot.data);

    let pool = AsyncComputeTaskPool::get();

    for (spawned, (entity, request, _path, _state)) in query.iter_mut().enumerate() {
        if spawned >= MAX_SPAWN_PER_TICK {
            break;
        }

        // Resolve grid positions to road nodes synchronously (O(1) lookups)
        let start_node = nearest_road_grid(&grid, request.from_gx, request.from_gy);
        let goal_node = nearest_road_grid(&grid, request.to_gx, request.to_gy);

        let target_state = request.target_state;

        // Remove PathRequest and add ComputingPath with the async task
        commands.entity(entity).remove::<PathRequest>();

        match (start_node, goal_node) {
            (Some(start), Some(goal)) => {
                let data_clone = Arc::clone(&data);
                let task =
                    pool.spawn(async move { data_clone.find_path_with_traffic(start, goal) });
                commands
                    .entity(entity)
                    .insert(ComputingPath { task, target_state });
            }
            _ => {
                // No valid road nodes found; skip pathfinding (citizen stays in current state)
            }
        }
    }
}

/// Poll in-flight async pathfinding tasks and apply completed results.
///
/// When a task completes, the computed path is written into the citizen's
/// `PathCache` and their state transitions to the requested target state.
pub fn collect_path_results(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut ComputingPath,
            &mut PathCache,
            &mut CitizenStateComp,
        ),
        With<Citizen>,
    >,
) {
    for (entity, mut computing, mut path, mut state) in &mut query {
        if let Some(result) = block_on(futures_lite::future::poll_once(&mut computing.task)) {
            if let Some(route) = result {
                *path = PathCache::new(route);
                state.0 = computing.target_state;
            }
            commands.entity(entity).remove::<ComputingPath>();
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn move_citizens(
    clock: Res<GameClock>,
    game_params: Res<GameParams>,
    weather: Res<crate::weather::Weather>,
    fog: Res<crate::fog::FogState>,
    snow_stats: Res<crate::snow::SnowStats>,
    congestion: Res<TrafficCongestion>,
    mut query: Query<
        (
            Entity,
            &CitizenStateComp,
            &mut Position,
            &mut Velocity,
            &mut PathCache,
            Option<&LodTier>,
            Option<&ChosenTransportMode>,
        ),
        With<Citizen>,
    >,
) {
    if clock.paused {
        return;
    }

    // Combine weather-based speed reduction with snow-based and fog-based speed reduction.
    // Snow and fog speed multipliers stack multiplicatively with weather speed multiplier.
    let snow_mult = snow_stats.road_speed_multiplier.max(0.2);
    let speed_per_tick = (game_params.citizen.speed / 10.0)
        * weather.travel_speed_multiplier_with_fog(fog.traffic_speed_modifier)
        * snow_mult;

    query.par_iter_mut().for_each(
        |(entity, state, mut pos, mut vel, mut path, lod, transport_mode)| {
            // Skip abstract citizens entirely
            if lod == Some(&LodTier::Abstract) {
                vel.x = 0.0;
                vel.y = 0.0;
                return;
            }
            // Move during any commuting state
            if !state.0.is_commuting() {
                vel.x = 0.0;
                vel.y = 0.0;
                return;
            }

            if let Some(target) = path.current_target() {
                // Compute smoothed target using Catmull-Rom interpolation
                let (tx, ty) = smoothed_waypoint_target(&path, *target, pos.x, pos.y);
                let dx = tx - pos.x;
                let dy = ty - pos.y;
                let dist = (dx * dx + dy * dy).sqrt();

                // Check arrival against the actual waypoint (not the smoothed target)
                let (raw_tx, raw_ty) = WorldGrid::grid_to_world(target.0, target.1);
                let raw_dist = ((raw_tx - pos.x).powi(2) + (raw_ty - pos.y).powi(2)).sqrt();

                // Apply traffic congestion: look up the speed multiplier for the
                // citizen's current grid cell. Congested roads reduce speed, creating
                // visible traffic bunching.
                let congestion_mult = congestion.get(target.0, target.1);
                let mode_mult = transport_mode
                    .map(|m| m.0.speed_multiplier())
                    .unwrap_or(1.0);
                let effective_speed = speed_per_tick * congestion_mult * mode_mult;

                // Use a fixed minimum arrival threshold so that even at very low speeds
                // (heavy snow/fog/congestion), citizens can still reach waypoints without orbiting.
                let arrival_dist = effective_speed.max(2.0);
                if raw_dist < arrival_dist {
                    pos.x = raw_tx;
                    pos.y = raw_ty;
                    vel.x = dx;
                    vel.y = dy;
                    path.advance();
                } else if dist > 0.001 {
                    let nx = dx / dist;
                    let ny = dy / dist;

                    // Per-entity lane offset: shift perpendicular to travel direction.
                    // Scale the offset by (speed / raw_dist) clamped to [0, 1] so that
                    // lateral drift diminishes as the citizen approaches the waypoint,
                    // preventing orbiting at low speeds (issue #1163).
                    let lane = (entity.index() % 3) as f32 - 1.0;
                    let lane_offset = lane * 2.5;
                    let perp_x = -ny;
                    let perp_y = nx;
                    let offset_scale = (effective_speed / raw_dist).min(1.0);

                    pos.x += nx * effective_speed + perp_x * lane_offset * 0.02 * offset_scale;
                    pos.y += ny * effective_speed + perp_y * lane_offset * 0.02 * offset_scale;
                    vel.x = nx * effective_speed;
                    vel.y = ny * effective_speed;
                }
            } else {
                vel.x = 0.0;
                vel.y = 0.0;
            }
        },
    );
}

/// Compute a smoothed waypoint target using Catmull-Rom interpolation.
/// Looks at the previous, current, and next waypoints to create a smooth curve.
fn smoothed_waypoint_target(
    path: &PathCache,
    current_target: RoadNode,
    current_x: f32,
    current_y: f32,
) -> (f32, f32) {
    let (tx, ty) = WorldGrid::grid_to_world(current_target.0, current_target.1);

    // Get the next waypoint after current (if available)
    let next = path.peek_next();
    let next_pos = if let Some(n) = next {
        let (nx, ny) = WorldGrid::grid_to_world(n.0, n.1);
        (nx, ny)
    } else {
        (tx, ty) // No next - use current target
    };

    // Use current position as "previous" point for the spline
    let prev_pos = (current_x, current_y);

    // Catmull-Rom: interpolate between prev_pos and current target,
    // with current target and next_pos as control points.
    // We want a point slightly ahead of the current target direction.
    let blend = 0.3; // How much to blend toward the smooth curve

    // Direction from prev to current target
    let d1x = tx - prev_pos.0;
    let d1y = ty - prev_pos.1;
    // Direction from current target to next
    let d2x = next_pos.0 - tx;
    let d2y = next_pos.1 - ty;

    // Smoothed direction: blend the two directions
    let sx = tx + (d2x - d1x) * blend * 0.25;
    let sy = ty + (d2y - d1y) * blend * 0.25;

    (sx, sy)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn compute_route_csr(
    grid: &WorldGrid,
    csr: &CsrGraph,
    traffic: &TrafficGrid,
    from_gx: usize,
    from_gy: usize,
    to_gx: usize,
    to_gy: usize,
) -> Option<Vec<RoadNode>> {
    let start = nearest_road_grid(grid, from_gx, from_gy)?;
    let goal = nearest_road_grid(grid, to_gx, to_gy)?;
    csr_find_path_with_traffic(csr, start, goal, grid, traffic)
}

/// Find the nearest destination within `max_dist` grid cells.
fn find_nearest(
    spots: &[(usize, usize)],
    from_x: usize,
    from_y: usize,
    max_dist: i32,
) -> Option<(usize, usize)> {
    spots
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

fn is_leisure_service(st: ServiceType) -> bool {
    matches!(
        st,
        ServiceType::SmallPark
            | ServiceType::LargePark
            | ServiceType::Playground
            | ServiceType::Plaza
            | ServiceType::SportsField
            | ServiceType::Stadium
            | ServiceType::Museum
    )
}

fn is_school_service(st: ServiceType) -> bool {
    matches!(
        st,
        ServiceType::ElementarySchool
            | ServiceType::HighSchool
            | ServiceType::University
            | ServiceType::Kindergarten
    )
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DestinationCache>()
            .init_resource::<PathfindingSnapshot>()
            .add_systems(
                FixedUpdate,
                (
                    invalidate_paths_on_road_removal,
                    refresh_destination_cache,
                    citizen_state_machine,
                    bevy::ecs::schedule::apply_deferred,
                    update_pathfinding_snapshot,
                    process_path_requests,
                    bevy::ecs::schedule::apply_deferred,
                    collect_path_results,
                    bevy::ecs::schedule::apply_deferred,
                    move_citizens,
                )
                    .chain()
                    .after(crate::citizen_spawner::spawn_citizens)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // find_nearest: basic nearest lookup
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_returns_closest_destination() {
        let spots = vec![(10, 10), (20, 20), (5, 5), (50, 50)];
        let result = find_nearest(&spots, 6, 6, 30);
        assert_eq!(
            result,
            Some((5, 5)),
            "should return (5,5) as closest to (6,6)"
        );
    }

    #[test]
    fn test_find_nearest_empty_returns_none() {
        let spots: Vec<(usize, usize)> = vec![];
        let result = find_nearest(&spots, 10, 10, 100);
        assert_eq!(result, None, "empty destination list should return None");
    }

    #[test]
    fn test_find_nearest_all_beyond_max_dist_returns_none() {
        let spots = vec![(100, 100), (200, 200)];
        let result = find_nearest(&spots, 0, 0, 10);
        assert_eq!(result, None, "all spots beyond max_dist should return None");
    }

    // ------------------------------------------------------------------
    // find_nearest: multiple destinations correct closest
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_multiple_destinations_various_query_points() {
        let spots = vec![(10, 10), (50, 50), (90, 90), (200, 200)];

        // From (12, 12): closest is (10, 10) with Manhattan dist 4
        assert_eq!(find_nearest(&spots, 12, 12, 100), Some((10, 10)));

        // From (48, 52): closest is (50, 50) with Manhattan dist 4
        assert_eq!(find_nearest(&spots, 48, 52, 100), Some((50, 50)));

        // From (88, 91): closest is (90, 90) with Manhattan dist 3
        assert_eq!(find_nearest(&spots, 88, 91, 100), Some((90, 90)));

        // From (199, 201): closest is (200, 200) with Manhattan dist 2
        assert_eq!(find_nearest(&spots, 199, 201, 250), Some((200, 200)));
    }

    // ------------------------------------------------------------------
    // find_nearest: exact position match
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_exact_match() {
        let spots = vec![(10, 10), (20, 20)];
        let result = find_nearest(&spots, 10, 10, 30);
        assert_eq!(
            result,
            Some((10, 10)),
            "querying from an exact destination position should return it"
        );
    }

    // ------------------------------------------------------------------
    // find_nearest: max_dist boundary
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_at_exact_max_dist_boundary() {
        let spots = vec![(15, 15)];
        // Manhattan distance from (10, 10) to (15, 15) = 10
        let result = find_nearest(&spots, 10, 10, 10);
        assert_eq!(
            result,
            Some((15, 15)),
            "spot at exactly max_dist should be included"
        );

        // max_dist = 9 -> should exclude
        let result2 = find_nearest(&spots, 10, 10, 9);
        assert_eq!(
            result2, None,
            "spot at dist 10 with max_dist 9 should be excluded"
        );
    }

    // ------------------------------------------------------------------
    // find_nearest: tiebreaker (min_by_key picks first minimum)
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_equidistant_returns_first() {
        // Two spots equidistant from query point
        let spots = vec![(10, 12), (12, 10)];
        // Manhattan dist from (11, 11): |10-11|+|12-11|=2 and |12-11|+|10-11|=2
        let result = find_nearest(&spots, 11, 11, 30);
        // min_by_key returns the first encountered minimum
        assert_eq!(
            result,
            Some((10, 12)),
            "should return first equidistant spot"
        );
    }

    // ------------------------------------------------------------------
    // find_nearest: single destination within range
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_single_spot_in_range() {
        let spots = vec![(100, 100)];
        let result = find_nearest(&spots, 95, 95, 20);
        assert_eq!(result, Some((100, 100)));
    }

    // ------------------------------------------------------------------
    // find_nearest: grid edge positions (boundary conditions)
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_at_grid_edges() {
        let spots = vec![(0, 0), (255, 255), (0, 255), (255, 0)];

        // From origin, closest is (0, 0)
        assert_eq!(find_nearest(&spots, 0, 0, 30), Some((0, 0)));

        // From max corner, closest is (255, 255)
        assert_eq!(find_nearest(&spots, 255, 255, 30), Some((255, 255)));

        // From (1, 254), closest is (0, 255) with dist 2
        assert_eq!(find_nearest(&spots, 1, 254, 30), Some((0, 255)));
    }

    // ------------------------------------------------------------------
    // find_nearest: large max_dist covers all
    // ------------------------------------------------------------------

    #[test]
    fn test_find_nearest_large_max_dist_returns_closest() {
        let spots = vec![(10, 10), (200, 200)];
        // Even with a huge max_dist, we should still get the closest
        let result = find_nearest(&spots, 12, 12, 1000);
        assert_eq!(
            result,
            Some((10, 10)),
            "should return closest even with large max_dist"
        );
    }
}
