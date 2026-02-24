use std::sync::Arc;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};

use crate::citizen::{
    Citizen, CitizenState, CitizenStateComp, PathCache, PathRequest, Position, Velocity,
};
use crate::game_params::GameParams;
use crate::grid::{RoadType, WorldGrid};
use crate::lod::LodTier;
use crate::mode_choice::ChosenTransportMode;
use crate::pathfinding_sys::nearest_road_grid;
use crate::road_graph_csr::{csr_find_path_with_traffic, CsrGraph, PathfindingData};
use crate::roads::RoadNode;
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
    #[cfg(feature = "trace")]
    let _span = bevy::log::info_span!("move_citizens").entered();
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
