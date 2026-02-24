use bevy::prelude::*;
use bitcode::{Decode, Encode};
use rand::Rng;

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails, Family, HomeLocation, WorkLocation};
use crate::death_care::{DeathCareGrid, DeathCareStats};
use crate::time_of_day::GameClock;
use crate::virtual_population::VirtualPopulation;
use crate::{decode_or_warn, Saveable, TestSafetyNet};

const AGING_INTERVAL_DAYS: u32 = 365;
const MAX_AGE: u8 = 100;
#[derive(Resource, Default, Encode, Decode)]
pub struct LifecycleTimer {
    pub last_aging_day: u32,
    pub last_emigration_tick: u32,
}

// ---------------------------------------------------------------------------
// Saveable
// ---------------------------------------------------------------------------

impl Saveable for LifecycleTimer {
    const SAVE_KEY: &'static str = "lifecycle_timer";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn age_citizens(
    clock: Res<GameClock>,
    mut timer: ResMut<LifecycleTimer>,
    mut commands: Commands,
    mut citizens: Query<
        (
            Entity,
            &mut CitizenDetails,
            &HomeLocation,
            Option<&WorkLocation>,
            &Family,
        ),
        With<Citizen>,
    >,
    mut buildings: Query<&mut Building>,
    mut virtual_pop: ResMut<VirtualPopulation>,
    mut death_grid: ResMut<DeathCareGrid>,
    mut death_stats: ResMut<DeathCareStats>,
    safety_net: Option<Res<TestSafetyNet>>,
) {
    if safety_net.is_some() {
        return;
    }
    if clock.day < timer.last_aging_day + AGING_INTERVAL_DAYS {
        return;
    }
    timer.last_aging_day = clock.day;

    death_stats.total_deaths_this_month = 0;
    death_stats.processed_this_month = 0;

    let mut rng = rand::thread_rng();

    // (entity, work_building_entity, partner_entity)
    let mut to_despawn: Vec<(Entity, Option<Entity>, Option<Entity>)> = Vec::new();

    for (entity, mut details, home, work, family) in &mut citizens {
        details.age = details.age.saturating_add(1);

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
                death_grid.record_death(home.grid_x, home.grid_y);
                death_stats.total_deaths_this_month += 1;
                to_despawn.push((entity, work.map(|w| w.building), family.partner));
                continue;
            }
        }

        if details.age >= MAX_AGE {
            if let Ok(mut building) = buildings.get_mut(home.building) {
                building.occupants = building.occupants.saturating_sub(1);
            }
            virtual_pop.total_virtual = virtual_pop.total_virtual.saturating_sub(1);
            death_grid.record_death(home.grid_x, home.grid_y);
            death_stats.total_deaths_this_month += 1;
            to_despawn.push((entity, work.map(|w| w.building), family.partner));
        }
    }

    let despawn_set: std::collections::HashSet<Entity> =
        to_despawn.iter().map(|&(e, _, _)| e).collect();

    for &(entity, work_building, partner) in &to_despawn {
        if let Some(wb) = work_building {
            if let Ok(mut building) = buildings.get_mut(wb) {
                building.occupants = building.occupants.saturating_sub(1);
            }
        }
        if let Some(partner_entity) = partner {
            // Skip if partner is also being despawned (avoids inserting on a dead entity)
            if !despawn_set.contains(&partner_entity) {
                if let Ok((_, _, _, _, partner_family)) = citizens.get(partner_entity) {
                    let children = partner_family.children.clone();
                    let parent = partner_family.parent;
                    commands.entity(partner_entity).insert(Family {
                        partner: None,
                        children,
                        parent,
                    });
                }
            }
        }
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::type_complexity)]
pub fn emigration(
    mut commands: Commands,
    mut timer: ResMut<LifecycleTimer>,
    citizens: Query<
        (
            Entity,
            &CitizenDetails,
            &HomeLocation,
            Option<&WorkLocation>,
            &Family,
        ),
        With<Citizen>,
    >,
    mut buildings: Query<&mut Building>,
    mut virtual_pop: ResMut<VirtualPopulation>,
    safety_net: Option<Res<TestSafetyNet>>,
) {
    if safety_net.is_some() {
        return;
    }
    timer.last_emigration_tick += 1;
    if timer.last_emigration_tick < 30 {
        return;
    }
    timer.last_emigration_tick = 0;

    let mut rng = rand::thread_rng();
    let mut to_despawn: Vec<(Entity, Option<Entity>, Option<Entity>)> = Vec::new();

    for (entity, details, home, work, family) in &citizens {
        if details.happiness < 20.0 {
            let leave_chance = (20.0 - details.happiness) / 100.0;
            if rng.gen::<f32>() < leave_chance {
                if let Ok(mut building) = buildings.get_mut(home.building) {
                    building.occupants = building.occupants.saturating_sub(1);
                }
                virtual_pop.total_virtual = virtual_pop.total_virtual.saturating_sub(1);
                to_despawn.push((entity, work.map(|w| w.building), family.partner));
            }
        }
    }

    let despawn_set: std::collections::HashSet<Entity> =
        to_despawn.iter().map(|&(e, _, _)| e).collect();

    for &(entity, work_building, partner) in &to_despawn {
        if let Some(wb) = work_building {
            if let Ok(mut building) = buildings.get_mut(wb) {
                building.occupants = building.occupants.saturating_sub(1);
            }
        }
        if let Some(partner_entity) = partner {
            // Skip if partner is also being despawned (avoids inserting on a dead entity)
            if !despawn_set.contains(&partner_entity) {
                if let Ok((_, _, _, _, partner_family)) = citizens.get(partner_entity) {
                    let children = partner_family.children.clone();
                    let parent = partner_family.parent;
                    commands.entity(partner_entity).insert(Family {
                        partner: None,
                        children,
                        parent,
                    });
                }
            }
        }
        commands.entity(entity).despawn();
    }
}

pub struct LifecyclePlugin;

impl Plugin for LifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LifecycleTimer>();

        // Register for save/load.
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<LifecycleTimer>();

        app.add_systems(
            FixedUpdate,
            (age_citizens, emigration)
                .chain()
                .after(crate::districts::district_stats)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
