//! Regression test for issue #1606: marriage matching must not pair
//! the same entity with multiple partners in a single tick.
//!
//! The bug allowed one female to be matched with several males
//! (or vice-versa) because the pairing loop did not track
//! already-matched entities. The fix adds a `HashSet<Entity>`
//! guard so each citizen can appear in at most one new marriage
//! per tick.

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity,
};
use crate::grid::{WorldGrid, ZoneType};
use crate::immigration::CityAttractiveness;
use crate::movement::ActivityTimer;
use crate::test_harness::TestCity;
use std::collections::HashMap;

/// Spawns many males and one female in the same building.
/// Without the HashSet guard, the single female could be paired
/// with multiple males. After ticking, we assert that at most one
/// male is partnered with her, and that all partner links are
/// reciprocal.
#[test]
fn test_marriage_single_female_many_males_no_double_match() {
    let mut city = TestCity::new().with_building(60, 60, ZoneType::ResidentialLow, 3);

    let building_entity = city.grid().get(60, 60).building_id.unwrap();
    let (wx, wy) = WorldGrid::grid_to_world(60, 60);

    let world = city.world_mut();

    // Spawn 10 males and 1 female, all at prime marriage age with
    // very high happiness to maximise the chance of a match.
    for i in 0..11 {
        let gender = if i == 0 { Gender::Female } else { Gender::Male };
        world.spawn((
            Citizen,
            Position { x: wx, y: wy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: 60,
                grid_y: 60,
                building: building_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 25 + (i % 5) as u8,
                gender,
                education: 2,
                happiness: 99.0,
                health: 100.0,
                salary: 4000.0,
                savings: 60000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.9,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
        ));
    }

    // Prevent emigration.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 90.0;
    }

    // Run many ticks to give the random 5% probability time to fire.
    city.tick(50_000);

    let world = city.world_mut();
    let mut query = world.query::<(bevy::prelude::Entity, &Family)>();
    let families: Vec<_> = query.iter(world).map(|(e, f)| (e, f.partner)).collect();

    let family_map: HashMap<_, _> = families.iter().map(|(e, p)| (*e, *p)).collect();

    // Verify reciprocity: if A -> B then B -> A
    for (entity, partner_opt) in &family_map {
        if let Some(partner) = partner_opt {
            let reverse = family_map.get(partner).and_then(|p| *p);
            assert_eq!(
                reverse,
                Some(*entity),
                "Reciprocity violated (issue #1606): {:?} -> {:?}, but {:?} -> {:?}",
                entity,
                partner,
                partner,
                reverse
            );
        }
    }

    // Count how many entities are partnered — must be even (complete pairs).
    let partnered: Vec<_> = family_map
        .iter()
        .filter(|(_, p)| p.is_some())
        .collect();

    assert_eq!(
        partnered.len() % 2,
        0,
        "Partnered count must be even, got {}",
        partnered.len()
    );

    // A citizen should have at most one partner. Because we check
    // reciprocity above, this is already implied, but let's be
    // explicit: no entity should appear as the partner of more
    // than one other entity.
    let mut partner_target_count: HashMap<bevy::prelude::Entity, usize> = HashMap::new();
    for (_, partner_opt) in &family_map {
        if let Some(partner) = partner_opt {
            *partner_target_count.entry(*partner).or_insert(0) += 1;
        }
    }
    for (target, count) in &partner_target_count {
        assert_eq!(
            *count, 1,
            "Entity {:?} is the partner of {} citizens (should be exactly 1)",
            target, count
        );
    }
}

/// Spawns many eligible citizens across multiple buildings and verifies
/// the global invariant that every partner link is reciprocal.
#[test]
fn test_marriage_multi_building_partner_consistency() {
    let mut city = TestCity::new()
        .with_building(40, 40, ZoneType::ResidentialLow, 3)
        .with_building(45, 45, ZoneType::ResidentialLow, 3)
        .with_building(50, 50, ZoneType::ResidentialLow, 3);

    let buildings = [
        (40_usize, 40_usize),
        (45, 45),
        (50, 50),
    ];

    for &(gx, gy) in &buildings {
        let building_entity = city.grid().get(gx, gy).building_id.unwrap();
        let (wx, wy) = WorldGrid::grid_to_world(gx, gy);

        let world = city.world_mut();
        for i in 0..8 {
            let gender = if i % 2 == 0 {
                Gender::Male
            } else {
                Gender::Female
            };
            world.spawn((
                Citizen,
                Position { x: wx, y: wy },
                Velocity { x: 0.0, y: 0.0 },
                HomeLocation {
                    grid_x: gx,
                    grid_y: gy,
                    building: building_entity,
                },
                CitizenStateComp(CitizenState::AtHome),
                PathCache::new(Vec::new()),
                CitizenDetails {
                    age: 22 + (i * 2) as u8,
                    gender,
                    education: 2,
                    happiness: 95.0,
                    health: 100.0,
                    salary: 3500.0,
                    savings: 40000.0,
                },
                Personality {
                    ambition: 0.5,
                    sociability: 0.8,
                    materialism: 0.5,
                    resilience: 0.5,
                },
                Needs::default(),
                Family::default(),
                ActivityTimer::default(),
            ));
        }
    }

    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 85.0;
    }

    city.tick(40_000);

    let world = city.world_mut();
    let mut query = world.query::<(bevy::prelude::Entity, &Family)>();
    let families: Vec<_> = query.iter(world).map(|(e, f)| (e, f.partner)).collect();

    let family_map: HashMap<_, _> = families.iter().map(|(e, p)| (*e, *p)).collect();

    for (entity, partner_opt) in &family_map {
        if let Some(partner) = partner_opt {
            let reverse = family_map.get(partner).and_then(|p| *p);
            assert_eq!(
                reverse,
                Some(*entity),
                "Multi-building reciprocity violated: {:?} -> {:?}, but {:?} -> {:?}",
                entity,
                partner,
                partner,
                reverse
            );
        }
    }

    // No entity should be the target of more than one partner link.
    let mut target_counts: HashMap<bevy::prelude::Entity, usize> = HashMap::new();
    for (_, partner_opt) in &family_map {
        if let Some(partner) = partner_opt {
            *target_counts.entry(*partner).or_insert(0) += 1;
        }
    }
    for (target, count) in &target_counts {
        assert_eq!(
            *count, 1,
            "Entity {:?} claimed by {} partners (issue #1606)",
            target, count
        );
    }
}
