use std::collections::HashSet;

use bevy::prelude::*;

use crate::buildings::Building;
use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, HomeLocation, Needs, PathCache,
    PathRequest, Position, WorkLocation,
};
use crate::game_params::GameParams;
use crate::grid::WorldGrid;
use crate::lod::LodTier;
use crate::roads::RoadNetwork;
use crate::services::{ServiceBuilding, ServiceType};
use crate::time_of_day::GameClock;

use super::pathfinding::ComputingPath;

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

    let removed = roads.drain_removed();

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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find the nearest destination within `max_dist` grid cells.
pub(crate) fn find_nearest(
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
