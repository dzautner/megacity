use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails};
use crate::grid::{WorldGrid, ZoneType};
use crate::immigration::CityAttractiveness;
use crate::test_harness::TestCity;

// ====================================================================
// Homelessness system tests (TEST-059)
// ====================================================================

/// Enable power and water on a set of grid cells so that `update_happiness`
/// does not tank citizen happiness below the emigration threshold (< 20).
///
/// Without utilities the happiness calculation applies -25 (no power) and
/// -20 (no water), which combined with the -30 homeless penalty pushes
/// happiness to 0 and triggers the emigration system during the 50-tick
/// CHECK_INTERVAL window.
fn enable_utilities(city: &mut TestCity, cells: &[(usize, usize)]) {
    let world = city.world_mut();
    let mut grid = world.resource_mut::<WorldGrid>();
    for &(x, y) in cells {
        if grid.in_bounds(x, y) {
            grid.get_mut(x, y).has_power = true;
            grid.get_mut(x, y).has_water = true;
        }
    }
}

/// Prevent the emigration system from despawning citizens during tests.
///
/// The `immigration_wave` system triggers emigration when
/// `CityAttractiveness.overall_score < 30.0`, and `compute_attractiveness`
/// recalculates this every 50 ticks based on city stats (employment,
/// happiness, services, housing, tax).  Simply setting `overall_score`
/// high is insufficient because `compute_attractiveness` runs *before*
/// `immigration_wave` in the same tick, overwriting the manual value.
///
/// This helper instead manipulates the *inputs* to the attractiveness
/// formula so that the recomputed score stays above 30:
///   - Sets `tax_rate = 0.0`  -> tax_factor = 1.0 -> contributes 15 pts
///   - Ensures citizen happiness is high -> happiness_factor up to 25 pts
///   - Employment stays at default (0% unemployment -> 25 pts)
///
/// Combined minimum: 15 + 12 + 25 = 52 pts (well above 30).
fn prevent_emigration(city: &mut TestCity) {
    // Also set overall_score high in case compute_attractiveness hasn't
    // run yet this tick window.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }
    // Set tax rate to 0 so the tax factor contributes 15 points
    {
        let mut budget = city.world_mut().resource_mut::<crate::economy::CityBudget>();
        budget.tax_rate = 0.0;
    }
}

/// Boost citizen stats to maximum to prevent despawn from secondary systems
/// (lifecycle emigration at happiness < 20, disease mortality at health < 20,
/// etc.) AND prevent emigration via `prevent_emigration`.
fn make_citizens_robust(city: &mut TestCity) {
    prevent_emigration(city);
    let world = city.world_mut();
    for mut details in world
        .query_filtered::<&mut CitizenDetails, bevy::prelude::With<Citizen>>()
        .iter_mut(world)
    {
        details.happiness = 95.0;
        details.savings = 50_000.0;
        details.health = 100.0;
    }
}

#[test]
fn test_homelessness_citizen_becomes_homeless_when_home_despawned() {
    // Spawn a citizen with a valid home building, then despawn the building.
    // After ticking past the CHECK_INTERVAL (50 ticks), the citizen should
    // gain the Homeless component.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    enable_utilities(&mut city, &[(50, 50)]);
    make_citizens_robust(&mut city);

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

    // Despawn the home building
    let building_entity = {
        let grid = city.grid();
        grid.get(50, 50).building_id.expect("building should exist")
    };
    city.world_mut().despawn(building_entity);

    // Tick past the homelessness CHECK_INTERVAL (50 ticks)
    prevent_emigration(&mut city);
    city.tick(50);

    // Citizen should now be homeless
    let homeless_count = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count()
    };
    assert!(
        homeless_count >= 1,
        "citizen should become homeless after home is despawned, got {homeless_count}"
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

    enable_utilities(&mut city, &[(50, 50), (60, 60)]);
    make_citizens_robust(&mut city);

    // Despawn both home buildings
    let b1 = city.grid().get(50, 50).building_id.expect("building 1");
    let b2 = city.grid().get(60, 60).building_id.expect("building 2");
    city.world_mut().despawn(b1);
    city.world_mut().despawn(b2);

    // Tick to trigger check_homelessness + seek_shelter
    prevent_emigration(&mut city);
    city.tick(50);

    let stats = city.resource::<crate::homelessness::HomelessnessStats>();
    assert!(
        stats.total_homeless >= 1,
        "total_homeless should reflect homeless citizens, got {}",
        stats.total_homeless
    );
}

#[test]
fn test_homelessness_recover_when_housing_available() {
    // Make a citizen homeless, then provide a residential building with capacity.
    // After ticking, the citizen should recover (Homeless component removed).
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    enable_utilities(&mut city, &[(50, 50), (70, 70)]);
    make_citizens_robust(&mut city);

    // Despawn the home building to make citizen homeless
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);

    // Tick to make citizen homeless
    prevent_emigration(&mut city);
    city.tick(50);

    {
        let world = city.world_mut();
        let homeless_count = world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count();
        assert!(
            homeless_count >= 1,
            "citizen should be homeless, got {homeless_count}"
        );
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
    prevent_emigration(&mut city);
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
    // When a citizen becomes homeless, their happiness should drop due to
    // the HOMELESS_PENALTY (30.0) applied by check_homelessness and the
    // ongoing penalty from update_happiness.
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));

    enable_utilities(&mut city, &[(50, 50)]);
    make_citizens_robust(&mut city);

    // Let happiness stabilize before making citizen homeless
    prevent_emigration(&mut city);
    city.tick(10);

    // Record stabilized happiness
    let initial_happiness = {
        let world = city.world_mut();
        let details = world
            .query::<&crate::citizen::CitizenDetails>()
            .iter(world)
            .next()
            .expect("citizen should exist");
        details.happiness
    };

    // Despawn home building
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);

    // Tick to make citizen homeless
    prevent_emigration(&mut city);
    city.tick(50);

    // Citizen should still exist (attractiveness prevents emigration)
    let new_happiness = {
        let world = city.world_mut();
        world
            .query::<&crate::citizen::CitizenDetails>()
            .iter(world)
            .next()
            .expect("citizen should exist with emigration prevented")
            .happiness
    };

    assert!(
        new_happiness < initial_happiness,
        "happiness should decrease when homeless: was {initial_happiness}, now {new_happiness}"
    );
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

    enable_utilities(&mut city, &[(50, 50), (55, 55)]);
    make_citizens_robust(&mut city);

    // Despawn home to make citizen homeless
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);

    // Tick to trigger check_homelessness (citizen becomes homeless)
    prevent_emigration(&mut city);
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
    make_citizens_robust(&mut city);
    city.tick(50);

    // Citizen should now be sheltered (attractiveness prevents emigration)
    let (total_homeless, sheltered_count) = {
        let world = city.world_mut();
        let homeless: Vec<&crate::homelessness::Homeless> = world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .collect();
        let sheltered = homeless.iter().filter(|h| h.sheltered).count();
        (homeless.len(), sheltered)
    };

    assert!(
        total_homeless > 0,
        "citizen should still be homeless (not emigrated), got {total_homeless}"
    );
    assert!(
        sheltered_count > 0,
        "homeless citizen should become sheltered when shelter has capacity \
         (total_homeless={total_homeless})"
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

    enable_utilities(&mut city, &[(50, 50), (55, 55)]);
    make_citizens_robust(&mut city);

    // Despawn home building to make all 3 citizens homeless
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);

    // Tick to make citizens homeless
    prevent_emigration(&mut city);
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
    make_citizens_robust(&mut city);
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
    assert!(
        total_homeless >= 2,
        "at least 2 citizens should still be homeless, got {total_homeless}"
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

    enable_utilities(&mut city, &[(50, 50)]);
    prevent_emigration(&mut city);

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
                happiness: 95.0,
                health: 100.0,
                salary: 3500.0,
                savings: 50_000.0,
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
    prevent_emigration(&mut city);
    city.tick(50);

    let homeless_count = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count()
    };
    assert!(
        homeless_count >= 1,
        "citizen with PLACEHOLDER home should become homeless, got {homeless_count}"
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

    enable_utilities(&mut city, &[(50, 50)]);
    prevent_emigration(&mut city);

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
                happiness: 95.0,
                health: 100.0,
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
    prevent_emigration(&mut city);
    city.tick(50);

    let homeless_count = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .count()
    };
    assert!(
        homeless_count >= 1,
        "citizen with negative savings and low salary should become homeless, got {homeless_count}"
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

    enable_utilities(&mut city, &[(50, 50), (70, 70)]);
    make_citizens_robust(&mut city);

    // Despawn home to make citizen homeless
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);

    prevent_emigration(&mut city);
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
    make_citizens_robust(&mut city);
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

    enable_utilities(&mut city, &[(50, 50)]);
    make_citizens_robust(&mut city);

    // Despawn home
    let b = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(b);

    // Tick to make homeless (first check)
    prevent_emigration(&mut city);
    city.tick(50);

    let ticks_after_first = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .next()
            .map(|h| h.ticks_homeless)
    };

    assert!(
        ticks_after_first.is_some(),
        "citizen should be homeless after first interval"
    );

    // Re-boost citizen stats and attractiveness before second tick window.
    // Also re-set the attractiveness inputs (tax_rate=0) so that when
    // compute_attractiveness recalculates during the second window, the
    // score stays above 30 (preventing immigration_wave emigration).
    make_citizens_robust(&mut city);

    // Tick again (second check) — use tick(51) to avoid the second window
    // aligning exactly with immigration_wave's 100-tick interval, which
    // could cause a race with compute_attractiveness on the same tick.
    city.tick(51);

    let ticks_after_second = {
        let world = city.world_mut();
        world
            .query::<&crate::homelessness::Homeless>()
            .iter(world)
            .next()
            .map(|h| h.ticks_homeless)
    };

    assert!(
        ticks_after_second.is_some(),
        "citizen should still be homeless after second interval"
    );

    let first = ticks_after_first.unwrap();
    let second = ticks_after_second.unwrap();
    assert!(
        second > first,
        "ticks_homeless should increment: first={first}, second={second}"
    );
}
