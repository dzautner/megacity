//! TEST-009: Unit tests for the citizen state machine.
//!
//! Verifies that citizens transition through their daily cycle correctly:
//! AtHome -> CommutingToWork -> Working -> CommutingHome -> AtHome,
//! governed by time-of-day rules and path completion.

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Gender, HomeLocation, Needs,
    PathCache, PathRequest, Personality, WorkLocation,
};
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::immigration::CityAttractiveness;
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::roads::RoadNode;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use bevy::prelude::*;

/// Enable power and water on grid cells to prevent the happiness system from
/// tanking citizen happiness due to critical utility penalties, which can
/// trigger emigration and interfere with the test.
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

/// Stabilize test environment by enabling utilities on given cells and boosting
/// CityAttractiveness + CityStats. This prevents the happiness system + emigration
/// system from despawning citizens during multi-tick test runs.
fn stabilize_test(city: &mut TestCity, cells: &[(usize, usize)]) {
    enable_utilities(city, cells);
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 50.0;
    }
    {
        let mut stats = city
            .world_mut()
            .resource_mut::<crate::stats::CityStats>();
        stats.average_happiness = 75.0;
    }
    // Also boost all existing citizens' happiness and savings to ensure the
    // happiness formula + lifecycle emigration don't despawn them.
    {
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
}

// ====================================================================
// Helper: spawn a citizen manually with a specific state
// ====================================================================

/// Spawn a citizen at a given state with specified home/work grid positions.
/// Buildings at those positions must already exist.
fn spawn_citizen_in_state(
    city: &mut TestCity,
    home: (usize, usize),
    work: Option<(usize, usize)>,
    state: CitizenState,
) -> Entity {
    let world = city.world_mut();

    // Look up entities before spawning (to avoid borrow conflicts)
    let home_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(home.0, home.1)
            .building_id
            .unwrap_or(Entity::PLACEHOLDER)
    };
    let work_info = work.map(|w| {
        let grid = world.resource::<WorldGrid>();
        let entity = grid
            .get(w.0, w.1)
            .building_id
            .unwrap_or(Entity::PLACEHOLDER);
        (w, entity)
    });

    let (hx, hy) = WorldGrid::grid_to_world(home.0, home.1);

    let citizen_entity = world
        .spawn((
            Citizen,
            crate::citizen::Position { x: hx, y: hy },
            crate::citizen::Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: home.0,
                grid_y: home.1,
                building: home_entity,
            },
            CitizenStateComp(state),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30,
                gender: Gender::Male,
                education: 2,
                happiness: 95.0,
                health: 100.0,
                salary: 3500.0,
                savings: 50000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            crate::citizen::Family::default(),
            ActivityTimer::default(),
            ChosenTransportMode::default(),
        ))
        .id();

    if let Some((w, work_entity)) = work_info {
        world.entity_mut(citizen_entity).insert(WorkLocation {
            grid_x: w.0,
            grid_y: w.1,
            building: work_entity,
        });
    }

    // Enable utilities at home (and work if applicable) to prevent the
    // happiness system from applying critical utility penalties, which can
    // tank happiness below the lifecycle emigration threshold.
    {
        let mut cells = vec![home];
        if let Some(w) = work {
            cells.push(w);
        }
        stabilize_test(city, &cells);
    }

    citizen_entity
}

// ====================================================================
// Test: AtHome transitions to CommutingToWork at morning commute hour
// ====================================================================

#[test]
fn test_at_home_transitions_to_commuting_to_work_at_work_hour() {
    // Set up a city with a road connecting home and work, set clock to
    // morning commute window (7-8 AM), and verify that after ticking
    // the citizen either has a PathRequest or has transitioned to
    // CommutingToWork.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 115, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (100, 115))
        .with_time(7.0)
        .rebuild_csr();

    // Prevent emigration from despawning the citizen during the long tick run.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    // Run enough ticks to cover the jitter window (entity index % 120 minutes)
    // and for pathfinding to complete. 200 ticks should be plenty.
    city.tick(200);

    // The citizen should have left the AtHome state. They could be in
    // CommutingToWork (path found) or Working (if they arrived quickly).
    let commuting = city.citizens_in_state(CitizenState::CommutingToWork);
    let working = city.citizens_in_state(CitizenState::Working);

    assert!(
        commuting > 0 || working > 0,
        "citizen should have started commuting or arrived at work during morning hours, \
         but found: CommutingToWork={commuting}, Working={working}"
    );
}

// ====================================================================
// Test: CommutingToWork transitions to Working on arrival (path complete)
// ====================================================================

#[test]
fn test_commuting_to_work_transitions_to_working_on_arrival() {
    // Create a citizen in CommutingToWork state with an already-complete
    // (empty) path. After one tick the state machine should transition
    // them to Working.
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1);

    let citizen_entity = spawn_citizen_in_state(
        &mut city,
        (100, 100),
        Some((100, 115)),
        CitizenState::CommutingToWork,
    );

    // The path is empty (already complete), so the state machine should
    // transition to Working on the next tick.
    city.tick(1);

    let state = {
        let world = city.world_mut();
        world.get::<CitizenStateComp>(citizen_entity).map(|s| s.0)
    };
    assert_eq!(
        state,
        Some(CitizenState::Working),
        "citizen should transition to Working when path is complete"
    );
}

// ====================================================================
// Test: Working transitions to CommutingHome at evening commute
// ====================================================================

#[test]
fn test_working_transitions_to_commuting_home_at_end_of_day() {
    // Place a citizen in Working state and set the clock to the evening
    // commute window (17-18). After ticking, the citizen should have a
    // PathRequest for CommutingHome or already be in CommutingHome.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 115, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1)
        .with_time(17.0)
        .rebuild_csr();

    let citizen_entity = spawn_citizen_in_state(
        &mut city,
        (100, 100),
        Some((100, 115)),
        CitizenState::Working,
    );

    city.tick(5);

    let world = city.world_mut();
    let state = world.get::<CitizenStateComp>(citizen_entity).map(|s| s.0);
    let has_path_request = world.get::<PathRequest>(citizen_entity).is_some();

    assert!(
        state == Some(CitizenState::CommutingHome) || has_path_request,
        "working citizen should start commuting home or have a path request at evening time, \
         state={state:?}, has_path_request={has_path_request}"
    );
}

// ====================================================================
// Test: CommutingHome transitions to AtHome on arrival
// ====================================================================

#[test]
fn test_commuting_home_transitions_to_at_home_on_arrival() {
    // Create a citizen in CommutingHome state with an already-complete
    // (empty) path. After one tick they should transition to AtHome.
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1);

    let citizen_entity = spawn_citizen_in_state(
        &mut city,
        (100, 100),
        Some((100, 115)),
        CitizenState::CommutingHome,
    );

    city.tick(1);

    let state = {
        let world = city.world_mut();
        world.get::<CitizenStateComp>(citizen_entity).map(|s| s.0)
    };
    assert_eq!(
        state,
        Some(CitizenState::AtHome),
        "citizen should transition to AtHome when commuting home path is complete"
    );
}

// ====================================================================
// Test: Citizen without job stays AtHome during morning commute
// ====================================================================

#[test]
fn test_citizen_without_job_stays_at_home() {
    // An unemployed citizen (no WorkLocation) should remain AtHome even
    // during the morning commute window.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 115, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_unemployed_citizen((100, 100))
        .with_time(7.5)
        .rebuild_csr();

    // Prevent emigration from despawning the citizen during the long tick run.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    // Tick through the entire morning commute window
    city.tick(200);

    // The unemployed citizen should never enter CommutingToWork
    let commuting_to_work = city.citizens_in_state(CitizenState::CommutingToWork);

    assert_eq!(
        commuting_to_work, 0,
        "unemployed citizen should never enter CommutingToWork state"
    );
}

// ====================================================================
// Test: Citizen without home becomes homeless
// ====================================================================

#[test]
fn test_citizen_without_valid_home_becomes_homeless() {
    // A citizen whose home building no longer exists should be detected
    // by the homelessness system and gain the Homeless component.
    // (The codebase does not have a "Wandering" state; homelessness is
    // handled via the Homeless component from the homelessness module.)
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((50, 50), (50, 50));
    // Prevent emigration from despawning the citizen during the tick window.
    // Homeless citizens lose 30 happiness, so start high to stay above threshold.
    {
        let world = city.world_mut();
        for mut details in world
            .query_filtered::<&mut CitizenDetails, With<Citizen>>()
            .iter_mut(world)
        {
            details.happiness = 95.0;
            details.health = 100.0;
        }
        let mut attr = world.resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    // Enable utilities and boost attractiveness to prevent the emigration
    // system from despawning the citizen before homelessness triggers.
    stabilize_test(&mut city, &[(50, 50)]);
    {
        let world = city.world_mut();
        for mut details in world
            .query_filtered::<&mut CitizenDetails, bevy::prelude::With<Citizen>>()
            .iter_mut(world)
        {
            details.happiness = 95.0;
        }
    }

    // Despawn the home building
    let building_entity = city.grid().get(50, 50).building_id.expect("building");
    city.world_mut().despawn(building_entity);

    // Tick past the homelessness CHECK_INTERVAL (50 ticks)
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
        "citizen with despawned home should become homeless, got {homeless_count}"
    );
}

// ====================================================================
// Test: Full daily cycle (AtHome -> Work -> AtHome)
// Test: Full daily cycle - morning departure
// ====================================================================

#[test]
fn test_full_daily_commute_cycle_morning_departure() {
    // Verify the morning part of the commute cycle: a citizen at home
    // at 6 AM should leave for work during the 7-8 AM window.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 110, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (100, 110))
        .with_time(6.0)
        .rebuild_csr();

    // Enable utilities on both buildings to prevent critical utility penalties
    // from tanking happiness and triggering emigration.
    stabilize_test(&mut city, &[(100, 100), (100, 110)]);

    let citizen_entity = {
        let world = city.world_mut();
        world
            .query_filtered::<Entity, With<Citizen>>()
            .iter(world)
            .next()
            .expect("citizen should exist")
    };

    // Boost citizen happiness to survive the needs-decay over 200 ticks.
    {
        let world = city.world_mut();
        for mut details in world
            .query_filtered::<&mut CitizenDetails, bevy::prelude::With<Citizen>>()
            .iter_mut(world)
        {
            details.happiness = 95.0;
        }
    }

    // Advance past the morning commute window (7-8 AM).
    // 2.5 hours = 150 ticks from 6 AM should reach ~8:30 AM.
    city.tick(200);

    // The citizen should have left home by now (commuting or working)
    let state_after_morning = {
        let world = city.world_mut();
        world.get::<CitizenStateComp>(citizen_entity).map(|s| s.0)
    };
    // Citizen should be commuting to work or already working
    assert!(
        state_after_morning == Some(CitizenState::CommutingToWork)
            || state_after_morning == Some(CitizenState::Working),
        "citizen should be commuting or working after morning commute, got {state_after_morning:?}"
    );
}

// ====================================================================
// Test: Full daily cycle - evening departure
// ====================================================================

#[test]
fn test_full_daily_commute_cycle_evening_departure() {
    // Verify the evening part: a Working citizen at 16:30 should
    // start commuting home during the 17-18 window.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 110, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 110, ZoneType::CommercialLow, 1)
        .with_time(16.5)
        .rebuild_csr();

    let citizen_entity = spawn_citizen_in_state(
        &mut city,
        (100, 100),
        Some((100, 110)),
        CitizenState::Working,
    );

    // Tick enough to enter the evening commute window (17-18)
    // 1.5 hours = 90 ticks from 16:30
    city.tick(120);

    let state = {
        let world = city.world_mut();
        world.get::<CitizenStateComp>(citizen_entity).map(|s| s.0)
    };
    // Should have started heading home or arrived home (or detoured to shop/leisure)
    assert!(
        state == Some(CitizenState::CommutingHome)
            || state == Some(CitizenState::AtHome)
            || state == Some(CitizenState::CommutingToShop)
            || state == Some(CitizenState::CommutingToLeisure),
        "working citizen should leave work during evening commute, got {state:?}"
    );
}

#[test]
fn test_paused_clock_prevents_state_transitions() {
    // When the game clock is paused, the citizen state machine should
    // not process any transitions.
    // Track a specific citizen entity to avoid confusion with spawned citizens.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 115, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (100, 115))
        .with_time(7.5)
        .rebuild_csr();

    stabilize_test(&mut city, &[(100, 100), (100, 115)]);

    // Capture our citizen entity
    let citizen_entity = {
        let world = city.world_mut();
        world
            .query_filtered::<Entity, With<Citizen>>()
            .iter(world)
            .next()
            .expect("citizen should exist")
    };

    // Pause the clock
    {
        let mut clock = city.world_mut().resource_mut::<GameClock>();
        clock.paused = true;
    }

    city.tick(200);

    // Our specific citizen should remain AtHome because the clock is paused
    let state = {
        let world = city.world_mut();
        world.get::<CitizenStateComp>(citizen_entity).map(|s| s.0)
    };
    assert_eq!(
        state,
        Some(CitizenState::AtHome),
        "citizen should remain AtHome when clock is paused"
    );
}

// ====================================================================
// Test: Citizen stays AtHome outside commute hours
// ====================================================================

#[test]
fn test_citizen_stays_at_home_outside_commute_hours() {
    // At 3 AM (well outside commute window), a citizen with a job
    // should remain AtHome. Track the specific entity.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 115, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (100, 115))
        .with_time(3.0)
        .rebuild_csr();

    stabilize_test(&mut city, &[(100, 100), (100, 115)]);

    let citizen_entity = {
        let world = city.world_mut();
        world
            .query_filtered::<Entity, With<Citizen>>()
            .iter(world)
            .next()
            .expect("citizen should exist")
    };

    // Prevent emigration from despawning the citizen during the tick run.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }
    // Only tick a few times (not enough to advance the clock to commute hour)
    city.tick(50);

    let state = {
        let world = city.world_mut();
        world.get::<CitizenStateComp>(citizen_entity).map(|s| s.0)
    };
    assert_eq!(
        state,
        Some(CitizenState::AtHome),
        "citizen should stay AtHome at 3 AM (outside commute hours)"
    );
}

// ====================================================================
// Test: Working citizen does not leave before evening commute
// ====================================================================

#[test]
fn test_working_citizen_stays_at_work_before_evening() {
    // A citizen at work at noon should remain Working (not start commuting
    // home until 17-18).
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1)
        .with_time(12.0);

    let citizen_entity = spawn_citizen_in_state(
        &mut city,
        (100, 100),
        Some((100, 115)),
        CitizenState::Working,
    );
    // Prevent emigration from despawning the citizen during the tick run.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    // Run a few ticks (not enough to reach evening commute at 17:00)
    city.tick(30);

    let state = {
        let world = city.world_mut();
        world.get::<CitizenStateComp>(citizen_entity).map(|s| s.0)
    };
    assert_eq!(
        state,
        Some(CitizenState::Working),
        "citizen should remain Working at noon, not leave until evening commute"
    );
}

// ====================================================================
// Test: CommutingToWork with active path stays commuting
// ====================================================================

#[test]
fn test_commuting_to_work_with_active_path_stays_commuting() {
    // A citizen who is CommutingToWork with waypoints remaining should
    // stay in CommutingToWork state.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 115, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1);

    let citizen_entity = spawn_citizen_in_state(
        &mut city,
        (100, 100),
        Some((100, 115)),
        CitizenState::CommutingToWork,
    );

    // Give the citizen a long path with many waypoints
    {
        let world = city.world_mut();
        if let Some(mut path) = world.get_mut::<PathCache>(citizen_entity) {
            *path = PathCache::new(vec![
                RoadNode(100, 101),
                RoadNode(100, 102),
                RoadNode(100, 103),
                RoadNode(100, 104),
                RoadNode(100, 105),
                RoadNode(100, 106),
                RoadNode(100, 107),
                RoadNode(100, 108),
                RoadNode(100, 109),
                RoadNode(100, 110),
                RoadNode(100, 111),
                RoadNode(100, 112),
                RoadNode(100, 113),
                RoadNode(100, 114),
                RoadNode(100, 115),
            ]);
        }
    }

    // Tick once -- the citizen should still be commuting (path not yet complete)
    city.tick(1);

    let state = {
        let world = city.world_mut();
        world.get::<CitizenStateComp>(citizen_entity).map(|s| s.0)
    };
    assert_eq!(
        state,
        Some(CitizenState::CommutingToWork),
        "citizen should remain CommutingToWork while path has remaining waypoints"
    );
}

// ====================================================================
// Test: CommutingToShop transitions to Shopping on arrival
// ====================================================================

#[test]
fn test_commuting_to_shop_transitions_to_shopping_on_arrival() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 110, ZoneType::CommercialLow, 1);

    let citizen_entity = spawn_citizen_in_state(
        &mut city,
        (100, 100),
        Some((100, 110)),
        CitizenState::CommutingToShop,
    );

    // Empty path = already arrived
    city.tick(1);

    let state = {
        let world = city.world_mut();
        world.get::<CitizenStateComp>(citizen_entity).map(|s| s.0)
    };
    assert_eq!(
        state,
        Some(CitizenState::Shopping),
        "citizen should transition to Shopping when CommutingToShop path is complete"
    );
}

// ====================================================================
// Test: CommutingToLeisure transitions to AtLeisure on arrival
// ====================================================================

#[test]
fn test_commuting_to_leisure_transitions_to_at_leisure_on_arrival() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 110, ZoneType::CommercialLow, 1);

    let citizen_entity = spawn_citizen_in_state(
        &mut city,
        (100, 100),
        Some((100, 110)),
        CitizenState::CommutingToLeisure,
    );

    city.tick(1);

    let state = {
        let world = city.world_mut();
        world.get::<CitizenStateComp>(citizen_entity).map(|s| s.0)
    };
    assert_eq!(
        state,
        Some(CitizenState::AtLeisure),
        "citizen should transition to AtLeisure when CommutingToLeisure path is complete"
    );
}

// ====================================================================
// Test: CommutingToSchool transitions to AtSchool on arrival
// ====================================================================

#[test]
fn test_commuting_to_school_transitions_to_at_school_on_arrival() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 110, ZoneType::CommercialLow, 1);

    let citizen_entity = spawn_citizen_in_state(
        &mut city,
        (100, 100),
        Some((100, 110)),
        CitizenState::CommutingToSchool,
    );

    city.tick(1);

    let state = {
        let world = city.world_mut();
        world.get::<CitizenStateComp>(citizen_entity).map(|s| s.0)
    };
    assert_eq!(
        state,
        Some(CitizenState::AtSchool),
        "citizen should transition to AtSchool when CommutingToSchool path is complete"
    );
}

// ====================================================================
// Test: is_commuting helper returns true for all commuting states
// ====================================================================

#[test]
fn test_is_commuting_helper() {
    assert!(CitizenState::CommutingToWork.is_commuting());
    assert!(CitizenState::CommutingHome.is_commuting());
    assert!(CitizenState::CommutingToShop.is_commuting());
    assert!(CitizenState::CommutingToLeisure.is_commuting());
    assert!(CitizenState::CommutingToSchool.is_commuting());

    assert!(!CitizenState::AtHome.is_commuting());
    assert!(!CitizenState::Working.is_commuting());
    assert!(!CitizenState::Shopping.is_commuting());
    assert!(!CitizenState::AtLeisure.is_commuting());
    assert!(!CitizenState::AtSchool.is_commuting());
}

// ====================================================================
// Test: is_at_destination helper returns true for destination states
// ====================================================================

#[test]
fn test_is_at_destination_helper() {
    assert!(CitizenState::AtHome.is_at_destination());
    assert!(CitizenState::Working.is_at_destination());
    assert!(CitizenState::Shopping.is_at_destination());
    assert!(CitizenState::AtLeisure.is_at_destination());
    assert!(CitizenState::AtSchool.is_at_destination());

    assert!(!CitizenState::CommutingToWork.is_at_destination());
    assert!(!CitizenState::CommutingHome.is_at_destination());
    assert!(!CitizenState::CommutingToShop.is_at_destination());
    assert!(!CitizenState::CommutingToLeisure.is_at_destination());
    assert!(!CitizenState::CommutingToSchool.is_at_destination());
}

// ====================================================================
// Test: Multiple citizens can be in different states simultaneously
// ====================================================================

#[test]
fn test_multiple_citizens_different_states() {
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1);

    // Spawn citizens in different states and track their entities
    let home_citizen = spawn_citizen_in_state(
        &mut city,
        (100, 100),
        Some((100, 115)),
        CitizenState::AtHome,
    );
    let working_citizen = spawn_citizen_in_state(
        &mut city,
        (100, 100),
        Some((100, 115)),
        CitizenState::Working,
    );
    let commuting_citizen = spawn_citizen_in_state(
        &mut city,
        (100, 100),
        Some((100, 115)),
        CitizenState::CommutingHome,
    );

    // Verify each citizen is in the expected state (before ticking)
    let world = city.world_mut();
    assert_eq!(
        world.get::<CitizenStateComp>(home_citizen).map(|s| s.0),
        Some(CitizenState::AtHome),
        "first citizen should be AtHome"
    );
    assert_eq!(
        world.get::<CitizenStateComp>(working_citizen).map(|s| s.0),
        Some(CitizenState::Working),
        "second citizen should be Working"
    );
    assert_eq!(
        world
            .get::<CitizenStateComp>(commuting_citizen)
            .map(|s| s.0),
        Some(CitizenState::CommutingHome),
        "third citizen should be CommutingHome"
    );
}

// ====================================================================
// Test: Citizen with PLACEHOLDER home building becomes homeless
// ====================================================================

#[test]
fn test_citizen_with_placeholder_home_becomes_homeless() {
    // A citizen whose home building entity is Entity::PLACEHOLDER
    // (i.e., never had a valid home) should be detected as homeless.
    let mut city = TestCity::new();

    // Spawn citizen with no valid building at home position
    spawn_citizen_in_state(&mut city, (50, 50), None, CitizenState::AtHome);
    // Prevent emigration from despawning the citizen during the tick window.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    // Tick past the homelessness CHECK_INTERVAL
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
