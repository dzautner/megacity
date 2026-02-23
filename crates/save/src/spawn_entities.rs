use bevy::prelude::*;

use crate::serialization::{
    restore_water_source, u8_to_service_type, u8_to_utility_type, u8_to_zone_type, SaveData,
};

use simulation::buildings::{Building, MixedUseBuilding};
use simulation::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use simulation::grid::WorldGrid;
use simulation::lod::LodTier;
use simulation::movement::ActivityTimer;
use simulation::roads::RoadNode;
use simulation::services::ServiceBuilding;
use simulation::utilities::UtilitySource;

/// Spawns all game entities from a parsed SaveData using direct world access.
pub(crate) fn spawn_entities_from_save(world: &mut World, save: &SaveData) {
    // Spawn buildings
    for sb in &save.buildings {
        let zone = u8_to_zone_type(sb.zone_type);
        let building = Building {
            zone_type: zone,
            level: sb.level,
            grid_x: sb.grid_x,
            grid_y: sb.grid_y,
            capacity: sb.capacity,
            occupants: sb.occupants,
        };
        let entity = if zone.is_mixed_use() {
            let (comm_cap, res_cap) = if sb.commercial_capacity > 0 || sb.residential_capacity > 0 {
                (sb.commercial_capacity, sb.residential_capacity)
            } else {
                MixedUseBuilding::capacities_for_level(sb.level)
            };
            world
                .spawn((
                    building,
                    MixedUseBuilding {
                        commercial_capacity: comm_cap,
                        commercial_occupants: sb.commercial_occupants,
                        residential_capacity: res_cap,
                        residential_occupants: sb.residential_occupants,
                    },
                ))
                .id()
        } else {
            world.spawn(building).id()
        };
        let mut grid = world.resource_mut::<WorldGrid>();
        if grid.in_bounds(sb.grid_x, sb.grid_y) {
            grid.get_mut(sb.grid_x, sb.grid_y).building_id = Some(entity);
        }
    }

    // Spawn utility sources
    for su in &save.utility_sources {
        let ut = u8_to_utility_type(su.utility_type);
        world.spawn(UtilitySource {
            utility_type: ut,
            grid_x: su.grid_x,
            grid_y: su.grid_y,
            range: su.range,
        });
    }

    // Spawn service buildings
    for ss in &save.service_buildings {
        if let Some(service_type) = u8_to_service_type(ss.service_type) {
            let radius = ServiceBuilding::coverage_radius(service_type);
            let entity = world
                .spawn(ServiceBuilding {
                    service_type,
                    grid_x: ss.grid_x,
                    grid_y: ss.grid_y,
                    radius,
                })
                .id();
            let mut grid = world.resource_mut::<WorldGrid>();
            if grid.in_bounds(ss.grid_x, ss.grid_y) {
                grid.get_mut(ss.grid_x, ss.grid_y).building_id = Some(entity);
            }
        }
    }

    // Spawn water sources
    if let Some(ref saved_water_sources) = save.water_sources {
        for sws in saved_water_sources {
            if let Some(ws) = restore_water_source(sws) {
                let entity = world.spawn(ws).id();
                let mut grid = world.resource_mut::<WorldGrid>();
                if grid.in_bounds(sws.grid_x, sws.grid_y) {
                    grid.get_mut(sws.grid_x, sws.grid_y).building_id = Some(entity);
                }
            }
        }
    }

    // Spawn citizens
    let mut citizen_entities: Vec<Entity> = Vec::with_capacity(save.citizens.len());

    // Pre-compute all citizen data in an inner scope so the grid borrow
    // ends before we call world.spawn().
    let citizen_spawn_data: Vec<_> = {
        let grid = world.resource::<WorldGrid>();
        save.citizens
            .iter()
            .map(|sc| {
                let state = match sc.state {
                    1 => CitizenState::CommutingToWork,
                    2 => CitizenState::Working,
                    3 => CitizenState::CommutingHome,
                    4 => CitizenState::CommutingToShop,
                    5 => CitizenState::Shopping,
                    6 => CitizenState::CommutingToLeisure,
                    7 => CitizenState::AtLeisure,
                    8 => CitizenState::CommutingToSchool,
                    9 => CitizenState::AtSchool,
                    _ => CitizenState::AtHome,
                };

                let home_building = if grid.in_bounds(sc.home_x, sc.home_y) {
                    grid.get(sc.home_x, sc.home_y)
                        .building_id
                        .unwrap_or(Entity::PLACEHOLDER)
                } else {
                    Entity::PLACEHOLDER
                };

                let work_building = if grid.in_bounds(sc.work_x, sc.work_y) {
                    grid.get(sc.work_x, sc.work_y)
                        .building_id
                        .unwrap_or(Entity::PLACEHOLDER)
                } else {
                    Entity::PLACEHOLDER
                };

                let (pos_x, pos_y) = if sc.pos_x != 0.0 || sc.pos_y != 0.0 {
                    (sc.pos_x, sc.pos_y)
                } else {
                    WorldGrid::grid_to_world(sc.home_x, sc.home_y)
                };

                let (path_cache, restored_state) = {
                    let waypoints: Vec<RoadNode> = sc
                        .path_waypoints
                        .iter()
                        .map(|&(x, y)| RoadNode(x, y))
                        .collect();

                    let all_valid = waypoints.iter().all(|n| grid.in_bounds(n.0, n.1));

                    if !waypoints.is_empty() && all_valid {
                        let mut pc = PathCache::new(waypoints);
                        pc.current_index = sc.path_current_index;
                        (pc, state)
                    } else if state.is_commuting() {
                        (PathCache::new(vec![]), CitizenState::AtHome)
                    } else {
                        (PathCache::new(vec![]), state)
                    }
                };

                let velocity = Velocity {
                    x: sc.velocity_x,
                    y: sc.velocity_y,
                };

                let gender = if sc.gender == 1 {
                    Gender::Female
                } else {
                    Gender::Male
                };

                let salary = if sc.salary != 0.0 {
                    sc.salary
                } else {
                    CitizenDetails::base_salary_for_education(sc.education)
                };

                let savings = if sc.savings != 0.0 {
                    sc.savings
                } else {
                    salary * 2.0
                };

                (
                    Citizen,
                    CitizenDetails {
                        age: sc.age,
                        gender,
                        happiness: sc.happiness,
                        health: sc.health,
                        education: sc.education,
                        salary,
                        savings,
                    },
                    CitizenStateComp(restored_state),
                    HomeLocation {
                        grid_x: sc.home_x,
                        grid_y: sc.home_y,
                        building: home_building,
                    },
                    WorkLocation {
                        grid_x: sc.work_x,
                        grid_y: sc.work_y,
                        building: work_building,
                    },
                    Position { x: pos_x, y: pos_y },
                    velocity,
                    path_cache,
                    Personality {
                        ambition: sc.ambition,
                        sociability: sc.sociability,
                        materialism: sc.materialism,
                        resilience: sc.resilience,
                    },
                    Needs {
                        hunger: sc.need_hunger,
                        energy: sc.need_energy,
                        social: sc.need_social,
                        fun: sc.need_fun,
                        comfort: sc.need_comfort,
                    },
                    Family::default(),
                    ActivityTimer(sc.activity_timer),
                    LodTier::default(),
                )
            })
            .collect()
    }; // grid borrow ends here

    for data in citizen_spawn_data {
        let entity = world.spawn(data).id();
        citizen_entities.push(entity);
    }

    // Second pass: restore family relationships using saved citizen indices.
    let num_citizens = citizen_entities.len();
    for (i, sc) in save.citizens.iter().enumerate() {
        let mut family = Family::default();
        if (sc.family_partner as usize) < num_citizens {
            family.partner = Some(citizen_entities[sc.family_partner as usize]);
        }
        for &child_idx in &sc.family_children {
            if (child_idx as usize) < num_citizens {
                family.children.push(citizen_entities[child_idx as usize]);
            }
        }
        if (sc.family_parent as usize) < num_citizens {
            family.parent = Some(citizen_entities[sc.family_parent as usize]);
        }
        if family.partner.is_some() || !family.children.is_empty() || family.parent.is_some() {
            if let Ok(mut entity_mut) = world.get_entity_mut(citizen_entities[i]) {
                entity_mut.insert(family);
            }
        }
    }
}
