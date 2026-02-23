use bevy::prelude::*;

use crate::buildings::{Building, UnderConstruction};
use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::grid::WorldGrid;
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::virtual_population::VirtualPopulation;
use crate::TickCounter;
use crate::TestSafetyNet;

use super::random::tick_pseudo_random;
use super::types::{
    CityAttractiveness, ImmigrationStats, IMMIGRATION_INTERVAL, MONTHLY_RESET_INTERVAL,
};

// ---------------------------------------------------------------------------
// System: immigration_wave
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn immigration_wave(
    tick: Res<TickCounter>,
    attractiveness: Res<CityAttractiveness>,
    mut commands: Commands,
    mut buildings: Query<(Entity, &mut Building), Without<UnderConstruction>>,
    citizens: Query<(Entity, &CitizenDetails, &HomeLocation), With<Citizen>>,
    mut virtual_pop: ResMut<VirtualPopulation>,
    mut imm_stats: ResMut<ImmigrationStats>,
    safety_net: Option<Res<TestSafetyNet>>,
) {
    if !tick.0.is_multiple_of(IMMIGRATION_INTERVAL) {
        return;
    }

    // Reset monthly stats periodically
    if tick.0.wrapping_sub(imm_stats.last_reset_tick) >= MONTHLY_RESET_INTERVAL {
        imm_stats.immigrants_this_month = 0;
        imm_stats.emigrants_this_month = 0;
        imm_stats.net_migration = 0;
        imm_stats.last_reset_tick = tick.0;
    }

    let score = attractiveness.overall_score;

    // Tick-based pseudo-random: hash the tick counter to get varied spawn counts
    let pseudo_rand = tick_pseudo_random(tick.0);

    if score > 60.0 {
        // Immigration wave
        let (min_families, max_families) = if score > 80.0 {
            (3u32, 10u32) // Boom times
        } else {
            (1u32, 5u32) // Normal attraction
        };
        let range = max_families - min_families + 1;
        let family_count = min_families + (pseudo_rand % range);

        spawn_immigrant_families(
            family_count,
            tick.0,
            &mut commands,
            &mut buildings,
            &mut virtual_pop,
            &mut imm_stats,
        );
    } else if score < 30.0 && safety_net.is_none() {
        // Emigration wave
        let (min_leave, max_leave) = if score < 15.0 {
            (5u32, 10u32) // Mass exodus
        } else {
            (1u32, 3u32) // Mild emigration
        };
        let range = max_leave - min_leave + 1;
        let leave_count = min_leave + (pseudo_rand % range);

        remove_unhappiest_citizens(
            leave_count,
            &mut commands,
            &citizens,
            &mut buildings,
            &mut virtual_pop,
            &mut imm_stats,
        );
    }
}

// ---------------------------------------------------------------------------
// Spawn immigrant families
// ---------------------------------------------------------------------------

fn spawn_immigrant_families(
    family_count: u32,
    tick: u64,
    commands: &mut Commands,
    buildings: &mut Query<(Entity, &mut Building), Without<UnderConstruction>>,
    virtual_pop: &mut ResMut<VirtualPopulation>,
    imm_stats: &mut ResMut<ImmigrationStats>,
) {
    // Collect residential buildings with capacity (including MixedUse)
    let homes_with_capacity: Vec<Entity> = buildings
        .iter()
        .filter(|(_, b)| {
            (b.zone_type.is_residential() || b.zone_type.is_mixed_use()) && b.occupants < b.capacity
        })
        .map(|(e, _)| e)
        .collect();

    if homes_with_capacity.is_empty() {
        return;
    }

    // Collect workplaces with capacity
    let workplaces: Vec<(Entity, usize, usize)> = buildings
        .iter()
        .filter(|(_, b)| b.zone_type.is_job_zone() && b.occupants < b.capacity)
        .map(|(e, b)| (e, b.grid_x, b.grid_y))
        .collect();

    if workplaces.is_empty() {
        return;
    }

    let mut spawned = 0u32;

    for i in 0..family_count {
        if homes_with_capacity.is_empty() {
            break;
        }

        // Pick a home using tick-based pseudo-random
        let home_idx = tick_pseudo_random(tick.wrapping_add(i as u64 * 7)) as usize
            % homes_with_capacity.len();
        let home_entity = homes_with_capacity[home_idx];

        // Check if this home still has capacity
        let (home_gx, home_gy, has_capacity) = {
            if let Ok((_, b)) = buildings.get(home_entity) {
                (b.grid_x, b.grid_y, b.occupants < b.capacity)
            } else {
                continue;
            }
        };

        if !has_capacity {
            continue;
        }

        // Pick a workplace
        let work_idx =
            tick_pseudo_random(tick.wrapping_add(i as u64 * 13 + 3)) as usize % workplaces.len();
        let (work_entity, work_gx, work_gy) = workplaces[work_idx];

        let (home_wx, home_wy) = WorldGrid::grid_to_world(home_gx, home_gy);

        // Generate citizen attributes from tick-based pseudo-random
        let seed = tick.wrapping_add(i as u64 * 31);
        let age = 18 + (tick_pseudo_random(seed) % 47) as u8;
        let gender = if tick_pseudo_random(seed.wrapping_add(1)).is_multiple_of(2) {
            Gender::Male
        } else {
            Gender::Female
        };
        let edu = match age {
            18..=22 => (tick_pseudo_random(seed.wrapping_add(2)) % 2) as u8,
            23..=30 => (tick_pseudo_random(seed.wrapping_add(2)) % 3).min(2) as u8,
            _ => (tick_pseudo_random(seed.wrapping_add(2)) % 4).min(3) as u8,
        };
        let salary = CitizenDetails::base_salary_for_education(edu)
            * (1.0 + age.saturating_sub(18) as f32 * 0.01);

        let pr = |offset: u64| -> f32 {
            (tick_pseudo_random(seed.wrapping_add(offset)) % 90 + 10) as f32 / 100.0
        };

        // Check real citizen cap
        if virtual_pop.max_real_citizens > 0 {
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
                    happiness: 55.0, // Immigrants start slightly above neutral
                    health: 80.0 + (tick_pseudo_random(seed.wrapping_add(10)) % 20) as f32,
                    salary,
                    savings: salary
                        * (1.0 + (tick_pseudo_random(seed.wrapping_add(11)) % 30) as f32 / 10.0),
                },
                Personality {
                    ambition: pr(20),
                    sociability: pr(21),
                    materialism: pr(22),
                    resilience: pr(23),
                },
                Needs::default(),
                Family::default(),
                ActivityTimer::default(),
                ChosenTransportMode::default(),
            ));
        }

        // Update building occupancy
        if let Ok((_, mut home_b)) = buildings.get_mut(home_entity) {
            home_b.occupants += 1;
        }
        if let Ok((_, mut work_b)) = buildings.get_mut(work_entity) {
            work_b.occupants += 1;
        }

        spawned += 1;
    }

    imm_stats.immigrants_this_month += spawned;
    imm_stats.net_migration += spawned as i32;
}

// ---------------------------------------------------------------------------
// Remove unhappiest citizens (emigration)
// ---------------------------------------------------------------------------

fn remove_unhappiest_citizens(
    count: u32,
    commands: &mut Commands,
    citizens: &Query<(Entity, &CitizenDetails, &HomeLocation), With<Citizen>>,
    buildings: &mut Query<(Entity, &mut Building), Without<UnderConstruction>>,
    virtual_pop: &mut ResMut<VirtualPopulation>,
    imm_stats: &mut ResMut<ImmigrationStats>,
) {
    // Collect all citizens sorted by happiness ascending (unhappiest first)
    let mut sorted_citizens: Vec<(Entity, f32, Entity)> = citizens
        .iter()
        .map(|(entity, details, home)| (entity, details.happiness, home.building))
        .collect();

    sorted_citizens.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut removed = 0u32;

    for (entity, _happiness, home_building) in sorted_citizens.iter() {
        if removed >= count {
            break;
        }

        if let Ok((_, mut building)) = buildings.get_mut(*home_building) {
            building.occupants = building.occupants.saturating_sub(1);
        }

        virtual_pop.total_virtual = virtual_pop.total_virtual.saturating_sub(1);
        commands.entity(*entity).despawn();
        removed += 1;
    }

    imm_stats.emigrants_this_month += removed;
    imm_stats.net_migration -= removed as i32;
}
