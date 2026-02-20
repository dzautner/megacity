use std::collections::HashSet;

use bevy::prelude::*;
use std::time::{Duration, Instant};

use crate::buildings::Building;
use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, HomeLocation, Needs, PathCache,
    PathRequest, Position, Velocity, WorkLocation,
};
use crate::grid::WorldGrid;
use crate::lod::LodTier;
use crate::pathfinding_sys::nearest_road_grid;
use crate::road_graph_csr::{csr_find_path, CsrGraph};
use crate::roads::{RoadNetwork, RoadNode};
use crate::services::{ServiceBuilding, ServiceType};
use crate::time_of_day::GameClock;

/// Time budget for pathfinding per tick (native). Processes as many paths as
/// fit within this duration, naturally handling commute bursts by doing more
/// work when paths are short/easy.
const PATH_BUDGET: Duration = Duration::from_millis(2);

/// Fallback count limit for WASM where Instant has poor resolution.
const MAX_PATHS_PER_TICK_WASM: usize = 256;

const CITIZEN_SPEED: f32 = 48.0; // pixels per second (at 10Hz that's ~4.8 px/tick)

// Duration limits (in ticks) for activities
const SHOPPING_DURATION: u32 = 30; // ~3 game minutes
const LEISURE_DURATION: u32 = 60; // ~6 game minutes
const SCHOOL_HOURS_START: u32 = 8;
const SCHOOL_HOURS_END: u32 = 15;

/// Per-citizen tick counter for activity durations
#[derive(Component, Debug, Clone, Default)]
pub struct ActivityTimer(pub u32);

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
        ),
        (With<Citizen>, Without<PathRequest>),
    >,
) {
    if clock.paused {
        return;
    }

    let hour = clock.hour_of_day();

    let shops = &dest_cache.shops;
    let leisure_spots = &dest_cache.leisure;
    let school_spots = &dest_cache.schools;

    for (entity, mut state, path, home, work, details, needs, mut timer, lod) in &mut query {
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
                    && (SCHOOL_HOURS_START..SCHOOL_HOURS_END).contains(&hour)
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
                if timer.0 >= SHOPPING_DURATION {
                    commands.entity(entity).insert(PathRequest {
                        from_gx: home.grid_x,
                        from_gy: home.grid_y,
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
                if timer.0 >= LEISURE_DURATION || hour >= 21 {
                    commands.entity(entity).insert(PathRequest {
                        from_gx: home.grid_x,
                        from_gy: home.grid_y,
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
                if hour >= SCHOOL_HOURS_END {
                    commands.entity(entity).insert(PathRequest {
                        from_gx: home.grid_x,
                        from_gy: home.grid_y,
                        to_gx: home.grid_x,
                        to_gy: home.grid_y,
                        target_state: CitizenState::CommutingHome,
                    });
                }
            }
        }
    }
}

/// Batch pathfinding: processes path requests within a time budget (native) or
/// count limit (WASM). This prevents frame spikes while allowing throughput to
/// scale with path complexity -- short paths process faster, so more fit per tick.
pub fn process_path_requests(
    mut commands: Commands,
    grid: Res<WorldGrid>,
    csr: Res<CsrGraph>,
    mut query: Query<(Entity, &PathRequest, &mut PathCache, &mut CitizenStateComp), With<Citizen>>,
) {
    let start = Instant::now();
    for (processed, (entity, request, mut path, mut state)) in query.iter_mut().enumerate() {
        if cfg!(target_arch = "wasm32") {
            if processed >= MAX_PATHS_PER_TICK_WASM {
                break;
            }
        } else if start.elapsed() >= PATH_BUDGET {
            break;
        }

        if let Some(route) = compute_route_csr(
            &grid,
            &csr,
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
}

#[allow(clippy::type_complexity)]
pub fn move_citizens(
    clock: Res<GameClock>,
    weather: Res<crate::weather::Weather>,
    fog: Res<crate::fog::FogState>,
    snow_stats: Res<crate::snow::SnowStats>,
    mut query: Query<
        (
            Entity,
            &CitizenStateComp,
            &mut Position,
            &mut Velocity,
            &mut PathCache,
            Option<&LodTier>,
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
    let speed_per_tick = (CITIZEN_SPEED / 10.0)
        * weather.travel_speed_multiplier_with_fog(fog.traffic_speed_modifier)
        * snow_mult;

    query
        .par_iter_mut()
        .for_each(|(entity, state, mut pos, mut vel, mut path, lod)| {
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

                if raw_dist < speed_per_tick {
                    pos.x = raw_tx;
                    pos.y = raw_ty;
                    vel.x = dx;
                    vel.y = dy;
                    path.advance();
                } else if dist > 0.001 {
                    let nx = dx / dist;
                    let ny = dy / dist;

                    // Per-entity lane offset: shift perpendicular to travel direction
                    let lane = (entity.index() % 3) as f32 - 1.0;
                    let lane_offset = lane * 2.5;
                    let perp_x = -ny;
                    let perp_y = nx;

                    pos.x += nx * speed_per_tick + perp_x * lane_offset * 0.02;
                    pos.y += ny * speed_per_tick + perp_y * lane_offset * 0.02;
                    vel.x = nx * speed_per_tick;
                    vel.y = ny * speed_per_tick;
                }
            } else {
                vel.x = 0.0;
                vel.y = 0.0;
            }
        });
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
    from_gx: usize,
    from_gy: usize,
    to_gx: usize,
    to_gy: usize,
) -> Option<Vec<RoadNode>> {
    let start = nearest_road_grid(grid, from_gx, from_gy)?;
    let goal = nearest_road_grid(grid, to_gx, to_gy)?;
    csr_find_path(csr, start, goal)
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
        app.init_resource::<DestinationCache>().add_systems(
            FixedUpdate,
            (
                invalidate_paths_on_road_removal,
                refresh_destination_cache,
                citizen_state_machine,
                bevy::ecs::schedule::apply_deferred,
                process_path_requests,
                move_citizens,
            )
                .chain()
                .after(crate::citizen_spawner::spawn_citizens),
        );
    }
}
