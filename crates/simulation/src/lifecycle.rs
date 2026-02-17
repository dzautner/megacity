use bevy::prelude::*;
use rand::Rng;

use crate::citizen::{Citizen, CitizenDetails, HomeLocation};
use crate::buildings::Building;
use crate::time_of_day::GameClock;
use crate::virtual_population::VirtualPopulation;

const AGING_INTERVAL_DAYS: u32 = 365;
const MAX_AGE: u8 = 100;
#[derive(Resource, Default)]
pub struct LifecycleTimer {
    pub last_aging_day: u32,
    pub last_emigration_tick: u32,
}

pub fn age_citizens(
    clock: Res<GameClock>,
    mut timer: ResMut<LifecycleTimer>,
    mut commands: Commands,
    mut citizens: Query<(Entity, &mut CitizenDetails, &HomeLocation), With<Citizen>>,
    mut buildings: Query<&mut Building>,
    mut virtual_pop: ResMut<VirtualPopulation>,
) {
    if clock.day < timer.last_aging_day + AGING_INTERVAL_DAYS {
        return;
    }
    timer.last_aging_day = clock.day;

    let mut rng = rand::thread_rng();

    for (entity, mut details, home) in &mut citizens {
        details.age = details.age.saturating_add(1);

        // Death check: increasing probability after age 70, amplified by poor health
        if details.age >= 70 || details.health < 5.0 {
            let age_factor = if details.age >= 70 {
                (details.age as f32 - 70.0) / 60.0
            } else {
                0.0
            };
            let health_factor = if details.health < 20.0 {
                (20.0 - details.health) / 40.0
            } else {
                0.0
            };
            let death_chance = (age_factor + health_factor).min(1.0);
            if rng.gen::<f32>() < death_chance {
                if let Ok(mut building) = buildings.get_mut(home.building) {
                    building.occupants = building.occupants.saturating_sub(1);
                }
                virtual_pop.total_virtual = virtual_pop.total_virtual.saturating_sub(1);
                commands.entity(entity).despawn();
                continue;
            }
        }

        // Max age death
        if details.age >= MAX_AGE {
            if let Ok(mut building) = buildings.get_mut(home.building) {
                building.occupants = building.occupants.saturating_sub(1);
            }
            virtual_pop.total_virtual = virtual_pop.total_virtual.saturating_sub(1);
            commands.entity(entity).despawn();
        }
    }
}

/// Citizens leave when unhappy
pub fn emigration(
    mut commands: Commands,
    mut timer: ResMut<LifecycleTimer>,
    citizens: Query<(Entity, &CitizenDetails, &HomeLocation), With<Citizen>>,
    mut buildings: Query<&mut Building>,
    mut virtual_pop: ResMut<VirtualPopulation>,
) {
    // Only check emigration every 30 ticks
    timer.last_emigration_tick += 1;
    if timer.last_emigration_tick < 30 {
        return;
    }
    timer.last_emigration_tick = 0;

    let mut rng = rand::thread_rng();

    for (entity, details, home) in &citizens {
        if details.happiness < 20.0 {
            let leave_chance = (20.0 - details.happiness) / 100.0;
            if rng.gen::<f32>() < leave_chance {
                if let Ok(mut building) = buildings.get_mut(home.building) {
                    building.occupants = building.occupants.saturating_sub(1);
                }
                virtual_pop.total_virtual = virtual_pop.total_virtual.saturating_sub(1);
                commands.entity(entity).despawn();
            }
        }
    }
}
