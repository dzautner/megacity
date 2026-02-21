use crate::grid::{WorldGrid, ZoneType};
use crate::test_harness::TestCity;

#[test]
fn test_keybindings_default_resource_exists() {
    let city = TestCity::new();
    let bindings = city.resource::<crate::keybindings::KeyBindings>();
    assert_eq!(
        bindings.toggle_pause.key,
        bevy::prelude::KeyCode::Space,
        "default pause key should be Space"
    );
}

#[test]
fn test_keybindings_rebind_and_conflict_detection() {
    use crate::keybindings::{BindableAction, KeyBinding, KeyBindings};
    use bevy::prelude::KeyCode;

    let mut kb = KeyBindings::default();
    kb.set(
        BindableAction::TogglePause,
        KeyBinding::simple(KeyCode::KeyX),
    );
    assert_eq!(kb.get(BindableAction::TogglePause).key, KeyCode::KeyX);

    let same_key = KeyBinding::simple(KeyCode::KeyQ);
    kb.set(BindableAction::ToolRoad, same_key);
    kb.set(BindableAction::ToolBulldoze, same_key);
    let conflicts = kb.find_conflicts();
    assert!(
        conflicts.iter().any(|(a, b)| {
            (*a == BindableAction::ToolRoad && *b == BindableAction::ToolBulldoze)
                || (*a == BindableAction::ToolBulldoze && *b == BindableAction::ToolRoad)
        }),
        "should detect conflict"
    );
}

#[test]
fn test_keybindings_saveable_roundtrip() {
    use crate::keybindings::{BindableAction, KeyBinding, KeyBindings};
    use crate::Saveable;
    use bevy::prelude::KeyCode;

    assert!(
        KeyBindings::default().save_to_bytes().is_none(),
        "default should skip save"
    );

    let mut kb = KeyBindings::default();
    kb.set(BindableAction::Screenshot, KeyBinding::simple(KeyCode::F11));
    let bytes = kb.save_to_bytes().expect("modified should save");
    let restored = KeyBindings::load_from_bytes(&bytes);
    assert_eq!(restored.get(BindableAction::Screenshot).key, KeyCode::F11);
    assert_eq!(
        restored.get(BindableAction::TogglePause).key,
        KeyCode::Space
    );
}

#[test]
fn test_keybindings_reset_to_defaults() {
    use crate::keybindings::{BindableAction, KeyBinding, KeyBindings};
    use bevy::prelude::KeyCode;

    let mut kb = KeyBindings::default();
    kb.set(
        BindableAction::TogglePause,
        KeyBinding::simple(KeyCode::KeyX),
    );
    assert_eq!(kb.get(BindableAction::TogglePause).key, KeyCode::KeyX);

    kb = KeyBindings::default();
    assert_eq!(kb.get(BindableAction::TogglePause).key, KeyCode::Space);
}

#[test]
fn test_marriage_matching_one_to_one_no_duplicate_partners() {
    use crate::citizen::{
        Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation,
        Needs, PathCache, Personality, Position, Velocity,
    };
    use crate::movement::ActivityTimer;
    use std::collections::HashMap;

    // Create a city with a residential building
    let mut city = TestCity::new().with_building(50, 50, ZoneType::ResidentialLow, 3);

    // Get the building entity
    let building_entity = city.grid().get(50, 50).building_id.unwrap();

    let (wx, wy) = WorldGrid::grid_to_world(50, 50);

    // Spawn 10 eligible males and 2 eligible females in the same building.
    // Without the fix, both females could be matched to multiple males in
    // a single tick, creating non-reciprocal partnerships.
    let world = city.world_mut();
    for _ in 0..10 {
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
                age: 30,
                gender: Gender::Male,
                education: 2,
                happiness: 80.0,
                health: 90.0,
                salary: 3500.0,
                savings: 7000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
        ));
    }
    for _ in 0..2 {
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
                age: 28,
                gender: Gender::Female,
                education: 2,
                happiness: 80.0,
                health: 90.0,
                salary: 3500.0,
                savings: 7000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
        ));
    }

    // Run many life-event cycles to give marriage matching many chances to fire.
    // LIFE_EVENT_INTERVAL is 600, so 600 * 50 = 30000 ticks gives ~50 cycles.
    city.tick(30_000);

    // Verify 1:1 matching: every citizen with a partner must have that partner
    // point back at them (reciprocal), and no entity appears as a partner of
    // more than one other entity.
    let world = city.world_mut();
    let mut partner_of: HashMap<bevy::prelude::Entity, bevy::prelude::Entity> = HashMap::new();

    let mut query = world.query::<(bevy::prelude::Entity, &Family)>();
    let pairs: Vec<_> = query.iter(world).map(|(e, f)| (e, f.partner)).collect();

    for (entity, partner_opt) in &pairs {
        if let Some(partner) = partner_opt {
            // Check no entity is claimed as partner by more than one citizen
            if let Some(&prev_claimer) = partner_of.get(partner) {
                panic!(
                    "Entity {:?} is partner of both {:?} and {:?} -- duplicate pairing!",
                    partner, prev_claimer, entity
                );
            }
            partner_of.insert(*partner, *entity);
        }
    }

    // Check reciprocity: if A's partner is B, then B's partner must be A
    let family_map: HashMap<_, _> = pairs.iter().map(|(e, p)| (*e, *p)).collect();
    for (entity, partner_opt) in &family_map {
        if let Some(partner) = partner_opt {
            let partner_partner = family_map.get(partner).and_then(|p| *p);
            assert_eq!(
                partner_partner,
                Some(*entity),
                "Non-reciprocal partnership: {:?} -> {:?}, but {:?} -> {:?}",
                entity,
                partner,
                partner,
                partner_partner
            );
        }
    }
}
