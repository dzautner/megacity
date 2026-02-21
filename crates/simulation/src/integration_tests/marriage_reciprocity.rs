use crate::grid::{WorldGrid, ZoneType};
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Marriage reciprocity invariant
// ---------------------------------------------------------------------------

/// After life simulation, verify all partnerships are reciprocal.
#[test]
fn test_marriage_reciprocity_invariant_after_life_simulation() {
    use crate::citizen::{
        Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation,
        Needs, PathCache, Personality, Position, Velocity,
    };
    use crate::movement::ActivityTimer;
    use std::collections::HashMap;

    let mut city = TestCity::new().with_building(50, 50, ZoneType::ResidentialLow, 3);

    let building_entity = city.grid().get(50, 50).building_id.unwrap();
    let (wx, wy) = WorldGrid::grid_to_world(50, 50);

    let world = city.world_mut();
    for i in 0..20 {
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
                grid_x: 50,
                grid_y: 50,
                building: building_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 25 + (i % 10) as u8,
                gender,
                education: 2,
                happiness: 80.0,
                health: 90.0,
                salary: 3500.0,
                savings: 7000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.7,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
        ));
    }

    city.tick(30_000);

    let world = city.world_mut();
    let mut query = world.query::<(bevy::prelude::Entity, &Family)>();
    let pairs: Vec<_> = query.iter(world).map(|(e, f)| (e, f.partner)).collect();

    let family_map: HashMap<_, _> = pairs.iter().map(|(e, p)| (*e, *p)).collect();

    let mut partnered_count = 0;
    for (entity, partner_opt) in &family_map {
        if let Some(partner) = partner_opt {
            partnered_count += 1;
            let partner_partner = family_map.get(partner).and_then(|p| *p);
            assert_eq!(
                partner_partner,
                Some(*entity),
                "Reciprocity violated: {:?} -> {:?}, but {:?} -> {:?}",
                entity,
                partner,
                partner,
                partner_partner
            );
        }
    }

    assert_eq!(
        partnered_count % 2,
        0,
        "Partnered citizen count should be even (pairs), got {}",
        partnered_count
    );
}
