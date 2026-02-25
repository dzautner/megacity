//! Integration tests verifying family relationship consistency for save/load.
//!
//! The save system serializes `Family` component entity references as citizen
//! array indices (entity-to-index mapping) and restores them via a two-pass
//! load (spawn all citizens first, then resolve indices to entities). These
//! tests verify the simulation-side invariants that the save pipeline depends on:
//!
//! - Partner relationships are bidirectional (A.partner == B && B.partner == A)
//! - Parent-child references are consistent (parent.children contains child, child.parent == parent)
//! - Family entity references point to valid Citizen entities
//! - Dangling references (e.g. from despawned citizens) are handled gracefully

use bevy::prelude::*;

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity,
};
use crate::grid::{WorldGrid, ZoneType};
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Spawn a citizen with a specific gender and age, returning the Entity.
fn spawn_citizen(
    world: &mut World,
    building: Entity,
    gx: usize,
    gy: usize,
    age: u8,
    gender: Gender,
) -> Entity {
    let (wx, wy) = WorldGrid::grid_to_world(gx, gy);
    world
        .spawn((
            Citizen,
            Position { x: wx, y: wy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: gx,
                grid_y: gy,
                building,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age,
                gender,
                education: 2,
                happiness: 70.0,
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
            ChosenTransportMode::default(),
        ))
        .id()
}

/// Set up a basic test city with a residential building at (50,50).
fn setup_city() -> (TestCity, Entity) {
    let city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 10)
        .with_utility(52, 52, UtilityType::PowerPlant)
        .with_utility(54, 54, UtilityType::WaterTower);
    let building = city.grid().get(50, 50).building_id.unwrap();
    (city, building)
}

// ---------------------------------------------------------------------------
// Test: partner relationship is bidirectional
// ---------------------------------------------------------------------------

#[test]
fn test_family_partner_bidirectional() {
    let (mut city, building) = setup_city();
    let husband = spawn_citizen(city.world_mut(), building, 50, 50, 30, Gender::Male);
    let wife = spawn_citizen(city.world_mut(), building, 50, 50, 28, Gender::Female);

    // Establish marriage
    {
        let world = city.world_mut();
        world.get_mut::<Family>(husband).unwrap().partner = Some(wife);
        world.get_mut::<Family>(wife).unwrap().partner = Some(husband);
    }

    // Verify bidirectional
    let world = city.world_mut();
    let h_family = world.get::<Family>(husband).unwrap();
    let w_family = world.get::<Family>(wife).unwrap();

    assert_eq!(h_family.partner, Some(wife), "Husband's partner should be wife");
    assert_eq!(w_family.partner, Some(husband), "Wife's partner should be husband");
}

// ---------------------------------------------------------------------------
// Test: parent-child references are consistent
// ---------------------------------------------------------------------------

#[test]
fn test_family_parent_child_consistency() {
    let (mut city, building) = setup_city();
    let mother = spawn_citizen(city.world_mut(), building, 50, 50, 30, Gender::Female);
    let father = spawn_citizen(city.world_mut(), building, 50, 50, 32, Gender::Male);
    let child1 = spawn_citizen(city.world_mut(), building, 50, 50, 5, Gender::Male);
    let child2 = spawn_citizen(city.world_mut(), building, 50, 50, 3, Gender::Female);

    // Set up family relationships (matching life_events.rs pattern)
    {
        let world = city.world_mut();
        // Parents are partners
        world.get_mut::<Family>(mother).unwrap().partner = Some(father);
        world.get_mut::<Family>(father).unwrap().partner = Some(mother);
        // Children reference mother as parent
        world.get_mut::<Family>(child1).unwrap().parent = Some(mother);
        world.get_mut::<Family>(child2).unwrap().parent = Some(mother);
        // Mother and father both list children
        world.get_mut::<Family>(mother).unwrap().children = vec![child1, child2];
        world.get_mut::<Family>(father).unwrap().children = vec![child1, child2];
    }

    // Verify consistency
    let world = city.world_mut();
    let m_family = world.get::<Family>(mother).unwrap();
    let f_family = world.get::<Family>(father).unwrap();
    let c1_family = world.get::<Family>(child1).unwrap();
    let c2_family = world.get::<Family>(child2).unwrap();

    // Both parents list both children
    assert_eq!(m_family.children.len(), 2);
    assert!(m_family.children.contains(&child1));
    assert!(m_family.children.contains(&child2));
    assert_eq!(f_family.children.len(), 2);
    assert!(f_family.children.contains(&child1));
    assert!(f_family.children.contains(&child2));

    // Children reference their parent
    assert_eq!(c1_family.parent, Some(mother));
    assert_eq!(c2_family.parent, Some(mother));

    // Children have no partner or children of their own
    assert!(c1_family.partner.is_none());
    assert!(c1_family.children.is_empty());
    assert!(c2_family.partner.is_none());
    assert!(c2_family.children.is_empty());
}

// ---------------------------------------------------------------------------
// Test: family entity references point to valid citizens
// ---------------------------------------------------------------------------

#[test]
fn test_family_references_point_to_valid_citizens() {
    let (mut city, building) = setup_city();
    let parent = spawn_citizen(city.world_mut(), building, 50, 50, 35, Gender::Female);
    let partner = spawn_citizen(city.world_mut(), building, 50, 50, 37, Gender::Male);
    let child = spawn_citizen(city.world_mut(), building, 50, 50, 8, Gender::Female);

    {
        let world = city.world_mut();
        world.get_mut::<Family>(parent).unwrap().partner = Some(partner);
        world.get_mut::<Family>(parent).unwrap().children = vec![child];
        world.get_mut::<Family>(partner).unwrap().partner = Some(parent);
        world.get_mut::<Family>(partner).unwrap().children = vec![child];
        world.get_mut::<Family>(child).unwrap().parent = Some(parent);
    }

    // Verify all references resolve to valid Citizen entities
    let world = city.world_mut();
    let p_family = world.get::<Family>(parent).unwrap().clone();

    if let Some(partner_e) = p_family.partner {
        assert!(
            world.get::<Citizen>(partner_e).is_some(),
            "Partner entity should be a valid Citizen"
        );
    }
    for child_e in &p_family.children {
        assert!(
            world.get::<Citizen>(*child_e).is_some(),
            "Child entity should be a valid Citizen"
        );
    }

    let c_family = world.get::<Family>(child).unwrap();
    if let Some(parent_e) = c_family.parent {
        assert!(
            world.get::<Citizen>(parent_e).is_some(),
            "Parent entity should be a valid Citizen"
        );
    }
}

// ---------------------------------------------------------------------------
// Test: default Family has no relationships
// ---------------------------------------------------------------------------

#[test]
fn test_family_default_has_no_relationships() {
    let (mut city, building) = setup_city();
    let citizen = spawn_citizen(city.world_mut(), building, 50, 50, 25, Gender::Male);

    let world = city.world_mut();
    let family = world.get::<Family>(citizen).unwrap();

    assert!(family.partner.is_none(), "Default family should have no partner");
    assert!(family.children.is_empty(), "Default family should have no children");
    assert!(family.parent.is_none(), "Default family should have no parent");
}

// ---------------------------------------------------------------------------
// Test: entity-to-index mapping produces correct indices for families
// ---------------------------------------------------------------------------

#[test]
fn test_family_entity_to_index_mapping() {
    // This test simulates the save-side entity-to-index mapping that
    // entity_stage.rs uses to serialize family relationships as citizen
    // array indices.
    let (mut city, building) = setup_city();
    let e0 = spawn_citizen(city.world_mut(), building, 50, 50, 30, Gender::Male);
    let e1 = spawn_citizen(city.world_mut(), building, 50, 50, 28, Gender::Female);
    let e2 = spawn_citizen(city.world_mut(), building, 50, 50, 5, Gender::Male);

    // Set up family: e0 and e1 are partners, e2 is their child
    {
        let world = city.world_mut();
        world.get_mut::<Family>(e0).unwrap().partner = Some(e1);
        world.get_mut::<Family>(e0).unwrap().children = vec![e2];
        world.get_mut::<Family>(e1).unwrap().partner = Some(e0);
        world.get_mut::<Family>(e1).unwrap().children = vec![e2];
        world.get_mut::<Family>(e2).unwrap().parent = Some(e1);
    }

    // Build entity-to-index map (same logic as entity_stage.rs)
    let citizen_entities = vec![e0, e1, e2];
    let entity_to_idx: std::collections::HashMap<Entity, u32> = citizen_entities
        .iter()
        .enumerate()
        .map(|(i, &e)| (e, i as u32))
        .collect();

    let world = city.world_mut();

    // Verify mapping for e0 (index 0)
    let f0 = world.get::<Family>(e0).unwrap();
    let partner_idx = f0.partner.and_then(|e| entity_to_idx.get(&e).copied());
    assert_eq!(partner_idx, Some(1), "e0's partner (e1) should map to index 1");
    let child_indices: Vec<u32> = f0
        .children
        .iter()
        .filter_map(|e| entity_to_idx.get(e).copied())
        .collect();
    assert_eq!(child_indices, vec![2], "e0's child (e2) should map to index 2");

    // Verify mapping for e1 (index 1)
    let f1 = world.get::<Family>(e1).unwrap();
    let partner_idx = f1.partner.and_then(|e| entity_to_idx.get(&e).copied());
    assert_eq!(partner_idx, Some(0), "e1's partner (e0) should map to index 0");

    // Verify mapping for e2 (index 2)
    let f2 = world.get::<Family>(e2).unwrap();
    let parent_idx = f2.parent.and_then(|e| entity_to_idx.get(&e).copied());
    assert_eq!(parent_idx, Some(1), "e2's parent (e1) should map to index 1");
}

// ---------------------------------------------------------------------------
// Test: index-to-entity resolution (simulating load-side two-pass)
// ---------------------------------------------------------------------------

#[test]
fn test_family_index_to_entity_resolution() {
    // This test simulates the load-side two-pass approach from
    // spawn_entities.rs: spawn citizens first, then resolve family
    // indices to entities.
    let (mut city, building) = setup_city();

    // Pass 1: spawn citizens with default (empty) families
    let e0 = spawn_citizen(city.world_mut(), building, 50, 50, 30, Gender::Male);
    let e1 = spawn_citizen(city.world_mut(), building, 50, 50, 28, Gender::Female);
    let e2 = spawn_citizen(city.world_mut(), building, 50, 50, 5, Gender::Male);
    let citizen_entities = vec![e0, e1, e2];

    // Simulated saved data (indices)
    let saved_partner_0: u32 = 1;
    let saved_partner_1: u32 = 0;
    let saved_children_0: Vec<u32> = vec![2];
    let saved_children_1: Vec<u32> = vec![2];
    let saved_parent_2: u32 = 1;

    // Pass 2: resolve indices to entities and set Family components
    {
        let world = city.world_mut();
        let num = citizen_entities.len();

        // Resolve e0
        let mut family_0 = Family::default();
        if (saved_partner_0 as usize) < num {
            family_0.partner = Some(citizen_entities[saved_partner_0 as usize]);
        }
        for &idx in &saved_children_0 {
            if (idx as usize) < num {
                family_0.children.push(citizen_entities[idx as usize]);
            }
        }
        *world.get_mut::<Family>(e0).unwrap() = family_0;

        // Resolve e1
        let mut family_1 = Family::default();
        if (saved_partner_1 as usize) < num {
            family_1.partner = Some(citizen_entities[saved_partner_1 as usize]);
        }
        for &idx in &saved_children_1 {
            if (idx as usize) < num {
                family_1.children.push(citizen_entities[idx as usize]);
            }
        }
        *world.get_mut::<Family>(e1).unwrap() = family_1;

        // Resolve e2
        let mut family_2 = Family::default();
        if (saved_parent_2 as usize) < num {
            family_2.parent = Some(citizen_entities[saved_parent_2 as usize]);
        }
        *world.get_mut::<Family>(e2).unwrap() = family_2;
    }

    // Verify the resolved family relationships
    let world = city.world_mut();
    let f0 = world.get::<Family>(e0).unwrap();
    assert_eq!(f0.partner, Some(e1));
    assert_eq!(f0.children, vec![e2]);

    let f1 = world.get::<Family>(e1).unwrap();
    assert_eq!(f1.partner, Some(e0));
    assert_eq!(f1.children, vec![e2]);

    let f2 = world.get::<Family>(e2).unwrap();
    assert_eq!(f2.parent, Some(e1));
    assert!(f2.partner.is_none());
    assert!(f2.children.is_empty());
}

// ---------------------------------------------------------------------------
// Test: out-of-bounds indices are skipped during resolution
// ---------------------------------------------------------------------------

#[test]
fn test_family_out_of_bounds_index_skipped() {
    // Simulates loading a save where a family reference index is >= citizen
    // count (e.g. citizen was removed between save and load).
    let (mut city, building) = setup_city();
    let e0 = spawn_citizen(city.world_mut(), building, 50, 50, 30, Gender::Male);
    let citizen_entities = vec![e0];

    // Saved data references index 5, but only 1 citizen exists
    let saved_partner: u32 = 5;
    let saved_children: Vec<u32> = vec![10, 20];
    let saved_parent: u32 = 99;

    {
        let world = city.world_mut();
        let num = citizen_entities.len();
        let mut family = Family::default();
        if (saved_partner as usize) < num {
            family.partner = Some(citizen_entities[saved_partner as usize]);
        }
        for &idx in &saved_children {
            if (idx as usize) < num {
                family.children.push(citizen_entities[idx as usize]);
            }
        }
        if (saved_parent as usize) < num {
            family.parent = Some(citizen_entities[saved_parent as usize]);
        }
        *world.get_mut::<Family>(e0).unwrap() = family;
    }

    let world = city.world_mut();
    let f = world.get::<Family>(e0).unwrap();
    assert!(f.partner.is_none(), "Out-of-bounds partner index should produce None");
    assert!(f.children.is_empty(), "Out-of-bounds child indices should be skipped");
    assert!(f.parent.is_none(), "Out-of-bounds parent index should produce None");
}

// ---------------------------------------------------------------------------
// Test: u32::MAX sentinel means no relationship
// ---------------------------------------------------------------------------

#[test]
fn test_family_u32_max_sentinel_means_no_relationship() {
    let (mut city, building) = setup_city();
    let e0 = spawn_citizen(city.world_mut(), building, 50, 50, 30, Gender::Male);
    let citizen_entities = vec![e0];

    // u32::MAX is the sentinel for "no relationship" in saved data
    let saved_partner: u32 = u32::MAX;
    let saved_parent: u32 = u32::MAX;

    {
        let world = city.world_mut();
        let num = citizen_entities.len();
        let mut family = Family::default();
        if (saved_partner as usize) < num {
            family.partner = Some(citizen_entities[saved_partner as usize]);
        }
        if (saved_parent as usize) < num {
            family.parent = Some(citizen_entities[saved_parent as usize]);
        }
        *world.get_mut::<Family>(e0).unwrap() = family;
    }

    let world = city.world_mut();
    let f = world.get::<Family>(e0).unwrap();
    assert!(f.partner.is_none(), "u32::MAX partner should resolve to None");
    assert!(f.parent.is_none(), "u32::MAX parent should resolve to None");
}
