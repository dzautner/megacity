use bevy::prelude::*;
use rand::Rng;

use crate::buildings::Building;
use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::grid::WorldGrid;
use crate::mode_choice::ChosenTransportMode;
use crate::time_of_day::GameClock;

use super::LifeSimTimer;

// ---------------------------------------------------------------------------
// System: life_events
// Handles marriage, births, and major life changes.
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn life_events(
    clock: Res<GameClock>,
    mut timer: ResMut<LifeSimTimer>,
    mut commands: Commands,
    mut citizens: Query<
        (
            Entity,
            &mut CitizenDetails,
            &mut Family,
            &HomeLocation,
            &Personality,
        ),
        With<Citizen>,
    >,
    mut buildings: Query<&mut Building>,
) {
    if clock.paused {
        return;
    }
    timer.life_event_tick += 1;
    if timer.life_event_tick < super::LIFE_EVENT_INTERVAL {
        return;
    }
    timer.life_event_tick = 0;

    let mut rng = rand::thread_rng();

    // Collect single adults in the same building for potential marriage
    let mut singles_by_building: std::collections::HashMap<Entity, Vec<(Entity, Gender, u8, f32)>> =
        std::collections::HashMap::new();

    for (entity, details, family, home, _personality) in &citizens {
        if family.partner.is_some() {
            continue;
        }
        if details.age < 20 || details.age > 55 {
            continue;
        }
        singles_by_building.entry(home.building).or_default().push((
            entity,
            details.gender,
            details.age,
            details.happiness,
        ));
    }

    // --- Marriage ---
    // Track matched entities to enforce one-to-one pairing within a single tick.
    // Without this, the same female could be paired with multiple males.
    let mut matched: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    let mut marriages: Vec<(Entity, Entity)> = Vec::new();
    for singles in singles_by_building.values() {
        let males: Vec<_> = singles.iter().filter(|s| s.1 == Gender::Male).collect();
        let females: Vec<_> = singles.iter().filter(|s| s.1 == Gender::Female).collect();

        for &&(m_entity, _, m_age, m_happy) in &males {
            if matched.contains(&m_entity) {
                continue;
            }
            if m_happy < 30.0 {
                continue; // too unhappy to marry
            }
            for &&(f_entity, _, f_age, f_happy) in &females {
                if matched.contains(&f_entity) {
                    continue;
                }
                if f_happy < 30.0 {
                    continue;
                }
                // Age compatibility: within 10 years
                let age_diff = (m_age as i32 - f_age as i32).unsigned_abs();
                if age_diff > 10 {
                    continue;
                }
                // Marriage probability
                if rng.gen::<f32>() < 0.05 {
                    matched.insert(m_entity);
                    matched.insert(f_entity);
                    marriages.push((m_entity, f_entity));
                    break; // one marriage per male per cycle
                }
            }
        }
    }

    for (m, f) in &marriages {
        if let Ok([(_, _, mut m_family, _, _), (_, _, mut f_family, _, _)]) =
            citizens.get_many_mut([*m, *f])
        {
            m_family.partner = Some(*f);
            f_family.partner = Some(*m);
        }
    }

    // --- Births ---
    // Collect couples where female is 20-38, have < 3 children
    let mut births: Vec<(Entity, Entity, usize, usize)> = Vec::new(); // (parent_entity, home_building, gx, gy)

    for (entity, details, family, home, personality) in &citizens {
        if details.gender != Gender::Female {
            continue;
        }
        if family.partner.is_none() {
            continue;
        }
        if details.age < 20 || details.age > 38 {
            continue;
        }
        if family.children.len() >= 3 {
            continue;
        }

        // Birth probability: influenced by sociability and age
        let age_factor = 1.0 - ((details.age as f32 - 25.0).abs() / 15.0).min(1.0);
        let prob = 0.02 * personality.sociability * age_factor;
        if rng.gen::<f32>() < prob {
            births.push((entity, home.building, home.grid_x, home.grid_y));
        }
    }

    for (parent_entity, home_building, home_gx, home_gy) in &births {
        // Check building has capacity
        if let Ok(mut building) = buildings.get_mut(*home_building) {
            if building.occupants >= building.capacity {
                continue;
            }
            building.occupants += 1;
        } else {
            continue;
        }

        let gender = if rng.gen::<f32>() < 0.5 {
            Gender::Male
        } else {
            Gender::Female
        };

        let (wx, wy) = WorldGrid::grid_to_world(*home_gx, *home_gy);

        let child = commands
            .spawn((
                Citizen,
                Position { x: wx, y: wy },
                Velocity { x: 0.0, y: 0.0 },
                HomeLocation {
                    grid_x: *home_gx,
                    grid_y: *home_gy,
                    building: *home_building,
                },
                CitizenStateComp(CitizenState::AtHome),
                PathCache::new(Vec::new()),
                CitizenDetails {
                    age: 0,
                    gender,
                    education: 0,
                    happiness: 80.0,
                    health: 100.0,
                    salary: 0.0,
                    savings: 0.0,
                },
                Personality::random(&mut rng),
                Needs {
                    hunger: 100.0,
                    energy: 100.0,
                    social: 100.0,
                    fun: 100.0,
                    comfort: 80.0,
                },
                Family {
                    parent: Some(*parent_entity),
                    ..Default::default()
                },
                ChosenTransportMode::default(),
            ))
            .id();

        // Add child to parent's family
        if let Ok((_, _, mut parent_family, _, _)) = citizens.get_mut(*parent_entity) {
            parent_family.children.push(child);
        }
        // Also add to partner's family
        if let Ok((_, _, parent_family, _, _)) = citizens.get(*parent_entity) {
            if let Some(partner) = parent_family.partner {
                if let Ok((_, _, mut partner_family, _, _)) = citizens.get_mut(partner) {
                    partner_family.children.push(child);
                }
            }
        }
    }
}
