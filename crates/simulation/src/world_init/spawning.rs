// =============================================================================
// Spawning: buildings, utilities, services, and citizens for Tel Aviv.
// =============================================================================

use bevy::prelude::*;

use crate::buildings::{Building, MixedUseBuilding};
use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::lod::LodTier;
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::services::{ServiceBuilding, ServiceType};
use crate::utilities::{UtilitySource, UtilityType};

use super::find_free_grass_cell;

// =============================================================================
// Buildings
// =============================================================================

pub fn spawn_tel_aviv_buildings(
    commands: &mut Commands,
    grid: &mut WorldGrid,
) -> Vec<(Entity, ZoneType, usize, usize, u32)> {
    let mut building_entities: Vec<(Entity, ZoneType, usize, usize, u32)> = Vec::new();

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let zone = grid.get(x, y).zone;
            let cell_type = grid.get(x, y).cell_type;
            if zone == ZoneType::None || cell_type != CellType::Grass {
                continue;
            }
            if grid.get(x, y).building_id.is_some() {
                continue;
            }

            // Building setback: skip cells directly adjacent to road cells
            let (n4, n4c) = grid.neighbors4(x, y);
            let adjacent_to_road = n4[..n4c]
                .iter()
                .any(|&(nx, ny)| grid.get(nx, ny).cell_type == CellType::Road);
            if adjacent_to_road {
                continue;
            }

            let hash = x.wrapping_mul(7).wrapping_add(y.wrapping_mul(13));
            let fill_pct = match zone {
                ZoneType::CommercialHigh | ZoneType::Office => 90,
                ZoneType::CommercialLow => 85,
                ZoneType::ResidentialHigh => 82,
                ZoneType::ResidentialMedium => 80,
                ZoneType::Industrial => 78,
                ZoneType::ResidentialLow => 70,
                _ => 65,
            };
            if hash % 100 > fill_pct {
                continue;
            }

            let xf = x as f32;
            let yf = y as f32;
            // Building level based on neighborhood
            let level: u8 = if xf > 100.0 && xf < 150.0 && yf > 90.0 && yf < 115.0 {
                // Azrieli area: tall
                if hash % 3 == 0 {
                    2
                } else {
                    3
                }
            } else if xf > 70.0 && xf < 140.0 && yf > 70.0 && yf < 160.0 {
                // White City: medium-tall
                match hash % 4 {
                    0 => 1,
                    1..=2 => 2,
                    _ => 3,
                }
            } else if yf < 70.0 && xf < 80.0 {
                // Jaffa: low
                if hash % 4 == 0 {
                    2
                } else {
                    1
                }
            } else if yf > 192.0 {
                // Ramat Aviv: medium
                match hash % 3 {
                    0 => 1,
                    1 => 2,
                    _ => 1,
                }
            } else {
                match hash % 3 {
                    0 => 1,
                    1 => 2,
                    _ => 1,
                }
            };

            let capacity = Building::capacity_for_level(zone, level);

            let entity = if zone.is_mixed_use() {
                let (comm_cap, res_cap) = MixedUseBuilding::capacities_for_level(level);
                commands
                    .spawn((
                        Building {
                            zone_type: zone,
                            level,
                            grid_x: x,
                            grid_y: y,
                            capacity,
                            occupants: 0,
                        },
                        MixedUseBuilding {
                            commercial_capacity: comm_cap,
                            commercial_occupants: 0,
                            residential_capacity: res_cap,
                            residential_occupants: 0,
                        },
                    ))
                    .id()
            } else {
                commands
                    .spawn(Building {
                        zone_type: zone,
                        level,
                        grid_x: x,
                        grid_y: y,
                        capacity,
                        occupants: 0,
                    })
                    .id()
            };

            grid.get_mut(x, y).building_id = Some(entity);
            building_entities.push((entity, zone, x, y, capacity));
        }
    }

    building_entities
}

// =============================================================================
// Utilities
// =============================================================================

pub fn spawn_tel_aviv_utilities(commands: &mut Commands, grid: &mut WorldGrid) {
    let positions = [
        (UtilityType::PowerPlant, 200usize, 50usize),
        (UtilityType::PowerPlant, 200, 150),
        (UtilityType::PowerPlant, 200, 220),
        (UtilityType::PowerPlant, 120, 30),
        (UtilityType::WaterTower, 90, 90),
        (UtilityType::WaterTower, 130, 130),
        (UtilityType::WaterTower, 80, 160),
        (UtilityType::WaterTower, 110, 210),
        (UtilityType::WaterTower, 160, 80),
        (UtilityType::WaterTower, 160, 160),
    ];

    for &(utype, ux, uy) in &positions {
        if let Some((px, py)) = find_free_grass_cell(grid, ux, uy, 10) {
            let range = match utype {
                UtilityType::PowerPlant => 120,
                UtilityType::WaterTower => 90,
                _ => 50,
            };
            let entity = commands
                .spawn(UtilitySource {
                    utility_type: utype,
                    grid_x: px,
                    grid_y: py,
                    range,
                })
                .id();
            grid.get_mut(px, py).building_id = Some(entity);
        }
    }
}

// =============================================================================
// Services
// =============================================================================

pub fn spawn_tel_aviv_services(commands: &mut Commands, grid: &mut WorldGrid) {
    let positions = [
        // Fire stations
        (ServiceType::FireStation, 85usize, 55usize),
        (ServiceType::FireStation, 130, 100),
        (ServiceType::FireStation, 80, 145),
        (ServiceType::FireStation, 120, 210),
        // Police
        (ServiceType::PoliceStation, 65, 48), // Jaffa
        (ServiceType::PoliceStation, 110, 90),
        (ServiceType::PoliceStation, 90, 135),
        (ServiceType::PoliceStation, 130, 160),
        // Hospitals
        (ServiceType::Hospital, 95, 80), // Ichilov area
        (ServiceType::Hospital, 150, 130),
        // Schools
        (ServiceType::ElementarySchool, 78, 80),
        (ServiceType::ElementarySchool, 115, 130),
        (ServiceType::ElementarySchool, 88, 210),
        (ServiceType::HighSchool, 105, 95),
        (ServiceType::HighSchool, 90, 150),
        (ServiceType::University, 110, 215), // Tel Aviv University area
        // Parks
        (ServiceType::LargePark, 80, 180), // Yarkon Park
        (ServiceType::LargePark, 105, 180),
        (ServiceType::SmallPark, 95, 88), // Rothschild gardens
        (ServiceType::SmallPark, 110, 105),
        (ServiceType::SmallPark, 130, 140),
        (ServiceType::SmallPark, 70, 50), // Jaffa garden
        (ServiceType::Plaza, 100, 135),   // Dizengoff Square area
        (ServiceType::Plaza, 118, 108),   // Habima area
        // Culture & civic
        (ServiceType::Museum, 112, 100), // Art museum area
        (ServiceType::CityHall, 115, 95),
        (ServiceType::Library, 105, 110),
        // Transport
        (ServiceType::TrainStation, 145, 95), // HaShalom station
        (ServiceType::TrainStation, 145, 155), // Arlozorov station
        (ServiceType::BusDepot, 100, 105),
        (ServiceType::SubwayStation, 110, 85),
        (ServiceType::SubwayStation, 115, 120),
    ];

    for &(stype, sx, sy) in &positions {
        if let Some((px, py)) = find_free_grass_cell(grid, sx, sy, 10) {
            let entity = commands
                .spawn(ServiceBuilding {
                    service_type: stype,
                    grid_x: px,
                    grid_y: py,
                    radius: ServiceBuilding::coverage_radius(stype),
                })
                .id();
            grid.get_mut(px, py).building_id = Some(entity);
        }
    }
}

// =============================================================================
// Citizens
// =============================================================================

pub fn spawn_tel_aviv_citizens(
    commands: &mut Commands,
    _grid: &WorldGrid,
    building_entities: &[(Entity, ZoneType, usize, usize, u32)],
) {
    let work_buildings: Vec<(Entity, usize, usize)> = building_entities
        .iter()
        .filter(|(_, zt, _, _, _)| zt.is_job_zone())
        .map(|(e, _, x, y, _)| (*e, *x, *y))
        .collect();

    // MixedUse buildings also provide residential capacity
    let residential_buildings: Vec<(Entity, usize, usize, u32)> = building_entities
        .iter()
        .filter(|(_, zt, _, _, _)| zt.is_residential() || zt.is_mixed_use())
        .map(|(e, _, x, y, cap)| (*e, *x, *y, *cap))
        .collect();

    if work_buildings.is_empty() {
        return;
    }

    let work_caps: Vec<u32> = building_entities
        .iter()
        .filter(|(_, zt, _, _, _)| zt.is_job_zone())
        .map(|(_, _, _, _, cap)| *cap)
        .collect();

    let mut work_idx = 0usize;
    let mut work_occupancy: Vec<u32> = vec![0; work_buildings.len()];
    let mut citizen_count = 0u32;
    // Reduce citizen count on WASM to prevent WebGL2 OOM/context loss
    let target_pop: u32 = if cfg!(target_arch = "wasm32") {
        2_000
    } else {
        10_000
    };
    let mut age_counter = 0u8;

    for (home_entity, hx, hy, cap) in &residential_buildings {
        if citizen_count >= target_pop {
            break;
        }
        let fill = (*cap as f32 * 0.9).ceil() as u32;
        for _ in 0..fill {
            if citizen_count >= target_pop {
                break;
            }

            let start_idx = work_idx;
            loop {
                if work_occupancy[work_idx] < work_caps[work_idx] {
                    break;
                }
                work_idx = (work_idx + 1) % work_buildings.len();
                if work_idx == start_idx {
                    break;
                }
            }

            let (work_entity, wx, wy) = work_buildings[work_idx];
            work_occupancy[work_idx] += 1;
            work_idx = (work_idx + 1) % work_buildings.len();

            let (home_wx, home_wy) = WorldGrid::grid_to_world(*hx, *hy);
            age_counter = age_counter.wrapping_add(7);
            let age = 18 + (age_counter % 47);

            let gender = if citizen_count.is_multiple_of(2) {
                Gender::Male
            } else {
                Gender::Female
            };
            let edu = match age {
                18..=22 => (age_counter % 3).min(1),
                23..=30 => (age_counter % 4).min(2),
                _ => (age_counter % 5).min(3),
            };
            let salary = CitizenDetails::base_salary_for_education(edu)
                * (1.0 + age.saturating_sub(18) as f32 * 0.01);

            commands.spawn((
                Citizen,
                Position {
                    x: home_wx,
                    y: home_wy,
                },
                Velocity { x: 0.0, y: 0.0 },
                HomeLocation {
                    grid_x: *hx,
                    grid_y: *hy,
                    building: *home_entity,
                },
                WorkLocation {
                    grid_x: wx,
                    grid_y: wy,
                    building: work_entity,
                },
                CitizenStateComp(CitizenState::AtHome),
                PathCache::new(Vec::new()),
                CitizenDetails {
                    age,
                    gender,
                    education: edu,
                    happiness: 60.0,
                    health: 90.0,
                    salary,
                    savings: salary * 2.0,
                },
                Personality {
                    ambition: ((age_counter.wrapping_mul(3)) % 100) as f32 / 100.0,
                    sociability: ((age_counter.wrapping_mul(7)) % 100) as f32 / 100.0,
                    materialism: ((age_counter.wrapping_mul(11)) % 100) as f32 / 100.0,
                    resilience: ((age_counter.wrapping_mul(13)) % 100) as f32 / 100.0,
                },
                Needs::default(),
                Family::default(),
                ActivityTimer::default(),
                LodTier::default(),
                ChosenTransportMode::default(),
            ));

            citizen_count += 1;
        }
    }
}
