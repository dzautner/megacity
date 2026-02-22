use crate::buildings::Building;
use crate::grid::{WorldGrid, ZoneType};
use crate::test_harness::TestCity;

/// Despawn a building and prevent recovery: clear the grid zone so the
/// building spawner won't recreate it, and set negative savings so
/// `recover_from_homelessness` won't assign the citizen to any other building.
fn despawn_home_prevent_recovery(city: &mut TestCity, gx: usize, gy: usize) {
    let b = city.grid().get(gx, gy).building_id.expect("building");
    city.world_mut().despawn(b);
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        let cell = grid.get_mut(gx, gy);
        cell.building_id = None;
        cell.zone = ZoneType::None;
    }
    {
        let world = city.world_mut();
        let mut q = world.query::<&mut crate::citizen::CitizenDetails>();
        for mut d in q.iter_mut(world) {
            d.savings = -100.0;
        }
    }
}

// ====================================================================
// Homelessness system tests (TEST-059)
// ====================================================================

#[test]
fn test_homelessness_citizen_becomes_homeless_when_home_despawned() {
    // Spawn a citizen with a valid home building, then despawn the building.
    // After ticking past the CHECK_INTERVAL (50 ticks), the citizen should
    // gain the Homeless component.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    // Verify citizen exists and is not homeless yet
    {
        let world = city.world_mut();
        let homeless_count = world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count();
        assert_eq!(
            homeless_count, 0,
            "citizen should not be homeless initially"
        );
    }

    // Despawn the home building and prevent recovery
    despawn_home_prevent_recovery(&mut city, 50, 50);

    // Tick past the homelessness CHECK_INTERVAL (50 ticks)
    city.tick(50);

    // Citizen should now be homeless
    let homeless_count = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count()
    };
    assert_eq!(
        homeless_count, 1,
        "citizen should become homeless after home is despawned"
    );
}

#[test]
fn test_homelessness_stats_track_total_homeless() {
    // Create citizens and despawn their homes to produce homeless citizens.
    // Verify HomelessnessStats.total_homeless reflects the count.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50))
        .with_building(60, 60, ZoneType::ResidentialLow, 1)
        .with_citizen((60, 60), (60, 60));

    // Despawn both home buildings and prevent recovery
    despawn_home_prevent_recovery(&mut city, 50, 50);
    // Re-fetch second building (first despawn may not affect it)
    let b2 = city.grid().get(60, 60).building_id.expect("building 2");
    city.world_mut().despawn(b2);
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        let cell = grid.get_mut(60, 60);
        cell.building_id = None;
        cell.zone = ZoneType::None;
    }

    // Tick to trigger check_homelessness + seek_shelter
    city.tick(50);

    let stats = city.resource::<crate::homelessness::HomelessnessStats>();
    assert_eq!(
        stats.total_homeless, 2,
        "total_homeless should reflect both homeless citizens"
    );
}

#[test]
fn test_homelessness_recover_when_housing_available() {
    // Make a citizen homeless, then provide a residential building with capacity.
    // After ticking, the citizen should recover (Homeless component removed).
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    // Despawn the home building and clear zone to prevent spawner recreation
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        let cell = grid.get_mut(50, 50);
        cell.building_id = None;
        cell.zone = ZoneType::None;
    }

    // Tick to make citizen homeless
    city.tick(50);

    {
        let world = city.world_mut();
        let homeless_count = world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count();
        assert_eq!(homeless_count, 1, "citizen should be homeless");
    }

    // Now spawn a new residential building with capacity for the citizen to move into
    {
        let entity = city
            .world_mut()
            .spawn(Building {
                zone_type: ZoneType::ResidentialLow,
                level: 1,
                grid_x: 70,
                grid_y: 70,
                capacity: 5,
                occupants: 0,
            })
            .id();
        let mut grid = city.world_mut().resource_mut::<WorldGrid>();
        grid.get_mut(70, 70).building_id = Some(entity);
        grid.get_mut(70, 70).zone = ZoneType::ResidentialLow;
    }

    // Tick again to trigger recover_from_homelessness
    city.tick(50);

    let homeless_count = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count()
    };
    assert_eq!(
        homeless_count, 0,
        "citizen should recover from homelessness when housing is available"
    );
}

#[test]
fn test_homelessness_happiness_penalty_applied() {
    // When a citizen becomes homeless, their happiness should drop by HOMELESS_PENALTY (30.0).
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    // Record initial happiness
    let initial_happiness = {
        let world = city.world_mut();
        let details = world
            .query::<&crate::citizen::CitizenDetails>()
            .iter(world)
            .next()
            .expect("citizen should exist");
        details.happiness
    };

    // Despawn home building and prevent recovery
    despawn_home_prevent_recovery(&mut city, 50, 50);

    // Tick to make citizen homeless
    city.tick(50);

    let new_happiness = {
        let world = city.world_mut();
        let details = world
            .query::<&crate::citizen::CitizenDetails>()
            .iter(world)
            .next()
            .expect("citizen should exist");
        details.happiness
    };

    // Happiness should have dropped (by at least HOMELESS_PENALTY = 30.0,
    // though other systems may also affect it)
    assert!(
        new_happiness < initial_happiness,
        "happiness should decrease when homeless: was {initial_happiness}, now {new_happiness}"
    );
    // The penalty is exactly 30.0 in check_homelessness, but other systems running
    // concurrently may shift it slightly. Check it dropped by at least 20.
    assert!(
        initial_happiness - new_happiness >= 20.0,
        "happiness should drop significantly: was {initial_happiness}, now {new_happiness}"
    );
}

#[test]
fn test_homelessness_shelter_provides_shelter_to_homeless() {
    // Spawn a homeless citizen and a shelter. After ticking, the citizen
    // should become sheltered (Homeless.sheltered = true).
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    // Despawn home building and prevent recovery
    despawn_home_prevent_recovery(&mut city, 50, 50);

    // Tick to trigger check_homelessness (citizen becomes homeless)
    city.tick(50);

    // Now spawn a HomelessShelter component entity
    {
        city.world_mut()
            .spawn(crate::homelessness::HomelessShelter {
                grid_x: 55,
                grid_y: 55,
                capacity: 10,
                current_occupants: 0,
            });
    }

    // Tick again to trigger seek_shelter
    city.tick(50);

    // Citizen should now be sheltered
    let sheltered = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .any(|h| h.sheltered)
    };
    assert!(
        sheltered,
        "homeless citizen should become sheltered when shelter has capacity"
    );

    let stats = city.resource::<crate::homelessness::HomelessnessStats>();
    assert!(
        stats.sheltered > 0,
        "sheltered count in stats should be positive"
    );
}

#[test]
fn test_homelessness_shelter_capacity_respected() {
    // Create more homeless citizens than shelter capacity.
    // Only up to capacity should be sheltered.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50))
        .with_citizen((50, 50), (50, 50));

    // Despawn home building and prevent recovery
    despawn_home_prevent_recovery(&mut city, 50, 50);

    // Tick to make citizens homeless
    city.tick(50);

    // Spawn a shelter with capacity of 1
    {
        city.world_mut()
            .spawn(crate::homelessness::HomelessShelter {
                grid_x: 55,
                grid_y: 55,
                capacity: 1,
                current_occupants: 0,
            });
    }

    // Tick to trigger seek_shelter
    city.tick(50);

    let (sheltered_count, total_homeless) = {
        let world = city.world_mut();
        let homeless_list: Vec<&crate::homelessness::Homeless> = world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .collect();
        let sheltered = homeless_list.iter().filter(|h| h.sheltered).count();
        let total = homeless_list.len();
        (sheltered, total)
    };

    assert_eq!(
        sheltered_count, 1,
        "only 1 citizen should be sheltered (capacity=1), got {sheltered_count}"
    );
    assert_eq!(
        total_homeless, 3,
        "all 3 citizens should still be homeless, got {total_homeless}"
    );
}

#[test]
fn test_homelessness_citizen_placeholder_home_becomes_homeless() {
    // A citizen whose home building is Entity::PLACEHOLDER should be detected
    // as homeless by check_homelessness.
    use crate::citizen::*;
    use crate::mode_choice::ChosenTransportMode;
    use crate::movement::ActivityTimer;
    use bevy::prelude::Entity;

    let mut city = TestCity::new();

    // Manually spawn a citizen with PLACEHOLDER home building
    {
        let world = city.world_mut();
        world.spawn((
            Citizen,
            Position { x: 800.0, y: 800.0 },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: 50,
                grid_y: 50,
                building: Entity::PLACEHOLDER,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30,
                gender: Gender::Male,
                education: 2,
                happiness: 60.0,
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
        ));
    }

    // Tick to trigger check_homelessness
    city.tick(50);

    let homeless_count = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count()
    };
    assert_eq!(
        homeless_count, 1,
        "citizen with PLACEHOLDER home should become homeless"
    );
}

#[test]
fn test_homelessness_rent_unaffordable_becomes_homeless() {
    // A citizen with negative savings and low salary should become homeless
    // due to rent unaffordability.
    use crate::citizen::*;
    use crate::mode_choice::ChosenTransportMode;
    use crate::movement::ActivityTimer;

    let mut city = TestCity::new().with_building(50, 50, ZoneType::ResidentialLow, 1);

    let home_entity = city.grid().get(50, 50).building_id.expect("building");

    // Spawn citizen with negative savings and salary below threshold (1000.0)
    {
        let world = city.world_mut();
        world.spawn((
            Citizen,
            Position { x: 800.0, y: 800.0 },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: 50,
                grid_y: 50,
                building: home_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30,
                gender: Gender::Male,
                education: 0,
                happiness: 50.0,
                health: 80.0,
                salary: 500.0,   // below RENT_AFFORDABILITY_THRESHOLD (1000.0)
                savings: -100.0, // negative savings
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
        ));
    }

    // Tick to trigger check_homelessness
    city.tick(50);

    let homeless_count = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count()
    };
    assert_eq!(
        homeless_count, 1,
        "citizen with negative savings and low salary should become homeless"
    );
}

#[test]
fn test_homelessness_stats_zero_in_empty_city() {
    // An empty city should have zero homelessness stats.
    let mut city = TestCity::new();
    city.tick(50);

    let stats = city.resource::<crate::homelessness::HomelessnessStats>();
    assert_eq!(stats.total_homeless, 0, "no homeless in empty city");
    assert_eq!(stats.sheltered, 0, "no sheltered in empty city");
}

#[test]
fn test_homelessness_recovery_updates_stats() {
    // After a homeless citizen recovers, total_homeless should decrease.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    // Despawn home and clear zone to prevent building spawner recreation.
    // Don't set negative savings here — this test needs the citizen to be able to recover.
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        let cell = grid.get_mut(50, 50);
        cell.building_id = None;
        cell.zone = ZoneType::None;
    }

    city.tick(50);

    let homeless_before = city
        .resource::<crate::homelessness::HomelessnessStats>()
        .total_homeless;
    assert!(
        homeless_before > 0,
        "should have at least one homeless citizen"
    );

    // Provide new housing
    {
        let entity = city
            .world_mut()
            .spawn(Building {
                zone_type: ZoneType::ResidentialLow,
                level: 1,
                grid_x: 70,
                grid_y: 70,
                capacity: 5,
                occupants: 0,
            })
            .id();
        let mut grid = city.world_mut().resource_mut::<WorldGrid>();
        grid.get_mut(70, 70).building_id = Some(entity);
        grid.get_mut(70, 70).zone = ZoneType::ResidentialLow;
    }

    // Tick to recover
    city.tick(50);

    let homeless_after = city
        .resource::<crate::homelessness::HomelessnessStats>()
        .total_homeless;
    assert!(
        homeless_after < homeless_before,
        "total_homeless should decrease after recovery: before={homeless_before}, after={homeless_after}"
    );
}

#[test]
fn test_homelessness_ticks_homeless_increments() {
    // The ticks_homeless counter on the Homeless component should increment
    // each time check_homelessness runs.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    // Despawn home and prevent recovery
    despawn_home_prevent_recovery(&mut city, 50, 50);

    // Tick to make homeless (first check)
    city.tick(50);

    let ticks_after_first = {
        let world = city.world_mut();
        let homeless = world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .next()
            .expect("should be homeless");
        homeless.ticks_homeless
    };

    // Tick again (second check)
    city.tick(50);

    let ticks_after_second = {
        let world = city.world_mut();
        let homeless = world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .next()
            .expect("should still be homeless");
        homeless.ticks_homeless
    };

    assert!(
        ticks_after_second > ticks_after_first,
        "ticks_homeless should increment: first={ticks_after_first}, second={ticks_after_second}"
    );
}
