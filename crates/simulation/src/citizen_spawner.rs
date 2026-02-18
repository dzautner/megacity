use bevy::prelude::*;
use rand::Rng;

use crate::buildings::{Building, UnderConstruction};
use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::movement::ActivityTimer;
use crate::grid::WorldGrid;
use crate::virtual_population::VirtualPopulation;

const SPAWN_INTERVAL: u32 = 5; // sim ticks between spawn attempts
const MAX_SPAWN_PER_TICK: u32 = 200;
const BURST_SPAWN_PER_TICK: u32 = 5000; // fast spawning when below capacity

#[derive(Resource, Default)]
pub struct CitizenSpawnTimer(pub u32);

pub fn spawn_citizens(
    mut commands: Commands,
    _grid: Res<WorldGrid>,
    mut timer: ResMut<CitizenSpawnTimer>,
    mut buildings: Query<(Entity, &mut Building)>,
    under_construction: Query<Entity, With<UnderConstruction>>,
    mut virtual_pop: ResMut<VirtualPopulation>,
    citizens: Query<&crate::citizen::Citizen>,
) {
    timer.0 += 1;
    if timer.0 < SPAWN_INTERVAL {
        return;
    }
    timer.0 = 0;

    let real_count = citizens.iter().count() as u32;

    let mut rng = rand::thread_rng();

    // Collect available workplaces (immutable pass)
    // Skip buildings that are still under construction
    let available_work: Vec<(Entity, usize, usize)> = buildings
        .iter()
        .filter(|(e, b)| {
            b.zone_type.is_job_zone()
                && b.occupants < b.capacity
                && under_construction.get(*e).is_err()
        })
        .map(|(e, b)| (e, b.grid_x, b.grid_y))
        .collect();

    if available_work.is_empty() {
        return;
    }

    // Collect residential buildings that need citizens
    // Skip buildings that are still under construction
    let homes_to_fill: Vec<Entity> = buildings
        .iter()
        .filter(|(e, b)| {
            b.zone_type.is_residential()
                && b.occupants < b.capacity
                && under_construction.get(*e).is_err()
        })
        .map(|(e, _)| e)
        .collect();

    // Use burst mode when population is far below building capacity
    // Only count operational (non-construction) residential capacity
    let total_res_capacity: u32 = buildings.iter()
        .filter(|(e, b)| b.zone_type.is_residential() && under_construction.get(*e).is_err())
        .map(|(_, b)| b.capacity)
        .sum();
    let target_fill = (total_res_capacity as f32 * 0.5) as u32;
    let current_pop = real_count + virtual_pop.total_virtual;
    let max_this_tick = if current_pop < target_fill {
        BURST_SPAWN_PER_TICK
    } else {
        MAX_SPAWN_PER_TICK
    };

    let mut spawned = 0u32;

    for home_entity in homes_to_fill {
        if spawned >= max_this_tick {
            break;
        }

        let (home_gx, home_gy) = {
            let (_, b) = buildings.get(home_entity).unwrap();
            (b.grid_x, b.grid_y)
        };

        // Pick a random workplace
        let work_idx = rng.gen_range(0..available_work.len());
        let (work_entity, work_gx, work_gy) = available_work[work_idx];

        let (home_wx, home_wy) = WorldGrid::grid_to_world(home_gx, home_gy);

        if real_count + spawned >= virtual_pop.max_real_citizens {
            // Over the real-citizen cap: track virtually instead of spawning an entity
            virtual_pop.total_virtual += 1;
            buildings.get_mut(home_entity).unwrap().1.occupants += 1;
            if let Ok((_, mut work_b)) = buildings.get_mut(work_entity) {
                work_b.occupants += 1;
            }
            spawned += 1;
            continue;
        }

        let age: u8 = rng.gen_range(18..65);
        let gender = if rng.gen::<bool>() {
            Gender::Male
        } else {
            Gender::Female
        };
        let edu = rng.gen_range(0u8..=2);
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
                grid_x: home_gx,
                grid_y: home_gy,
                building: home_entity,
            },
            WorkLocation {
                grid_x: work_gx,
                grid_y: work_gy,
                building: work_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age,
                gender,
                education: edu,
                happiness: 50.0,
                health: 85.0 + rng.gen_range(0.0..15.0),
                salary,
                savings: salary * rng.gen_range(0.5..3.0),
            },
            Personality::random(&mut rng),
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
        ));

        buildings.get_mut(home_entity).unwrap().1.occupants += 1;
        if let Ok((_, mut work_b)) = buildings.get_mut(work_entity) {
            work_b.occupants += 1;
        }
        spawned += 1;
    }
}
