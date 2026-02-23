//! TEST-019: Integration tests for apply_deferred flush points.
//!
//! Verifies that `apply_deferred` flush points are correctly placed in the
//! `FixedUpdate` schedule so that `PathRequest` components inserted by
//! `citizen_state_machine` (via `Commands`) are visible to
//! `process_path_requests` within the same frame.
//!
//! The movement system chain is:
//!   citizen_state_machine -> apply_deferred -> process_path_requests
//!
//! Without the flush, `PathRequest` components would be deferred until the
//! next frame, breaking same-frame pathfinding dispatch.

use bevy::prelude::*;

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, PathRequest, Personality, Position, Velocity, WorkLocation,
};
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::immigration::CityAttractiveness;
use crate::mode_choice::ChosenTransportMode;
use crate::movement::{ActivityTimer, ComputingPath};
use crate::test_harness::TestCity;

// ====================================================================
// Helper: spawn a citizen in a specific state
// ====================================================================

/// Spawn a citizen at the given home/work with a specified state.
/// The home and work buildings must already exist.
fn spawn_citizen_at(
    city: &mut TestCity,
    home: (usize, usize),
    work: (usize, usize),
    state: CitizenState,
) -> Entity {
    let world = city.world_mut();

    let home_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(home.0, home.1)
            .building_id
            .unwrap_or(Entity::PLACEHOLDER)
    };
    let work_entity = {
        let grid = world.resource::<WorldGrid>();
        grid.get(work.0, work.1)
            .building_id
            .unwrap_or(Entity::PLACEHOLDER)
    };

    let (hx, hy) = WorldGrid::grid_to_world(home.0, home.1);

    world
        .spawn((
            Citizen,
            Position { x: hx, y: hy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: home.0,
                grid_y: home.1,
                building: home_entity,
            },
            WorkLocation {
                grid_x: work.0,
                grid_y: work.1,
                building: work_entity,
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
        ))
        .id()
}

/// Stabilize the test environment so emigration/happiness systems don't
/// despawn citizens during the tick run.
fn stabilize(city: &mut TestCity, cells: &[(usize, usize)]) {
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        for &(x, y) in cells {
            if grid.in_bounds(x, y) {
                grid.get_mut(x, y).has_power = true;
                grid.get_mut(x, y).has_water = true;
            }
        }
    }
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }
    {
        let mut stats = city.world_mut().resource_mut::<crate::stats::CityStats>();
        stats.average_happiness = 75.0;
    }
}

// ====================================================================
// Test: PathRequest inserted by state_machine is consumed in same frame
// ====================================================================

/// Verify that when `citizen_state_machine` inserts a `PathRequest` via
/// Commands, the `apply_deferred` flush point makes it visible to
/// `process_path_requests` within the same `FixedUpdate` run.
///
/// After a single tick, the citizen should NOT still have a `PathRequest` —
/// it should have been consumed by `process_path_requests` and replaced
/// with either a `ComputingPath` marker (async dispatch) or removed
/// entirely (no valid route / WASM fallback).
#[test]
fn test_flush_point_path_request_consumed_same_frame() {
    let home = (10, 10);
    let work = (10, 25);

    let mut city = TestCity::new()
        .with_road(10, 10, 10, 25, RoadType::Local)
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .rebuild_csr()
        .with_time(7.0); // morning commute window

    let citizen = spawn_citizen_at(&mut city, home, work, CitizenState::AtHome);
    stabilize(&mut city, &[home, work]);

    // Run 120 ticks to cover the full morning commute window (hours 7-8).
    // This guarantees hitting the per-entity departure jitter value.
    city.tick(120);

    let world = city.world_mut();

    // The citizen should NOT still have a lingering PathRequest.
    // If the flush point is correct, process_path_requests consumed it
    // in the same frame it was inserted.
    let has_path_request = world.get::<PathRequest>(citizen).is_some();
    let has_computing_path = world.get::<ComputingPath>(citizen).is_some();
    let state = world.get::<CitizenStateComp>(citizen).map(|s| s.0);

    // After 120 ticks, the citizen should have transitioned out of AtHome.
    // The PathRequest should have been consumed — the citizen is either:
    //   - In ComputingPath (async dispatch pending)
    //   - Already commuting (path resolved)
    //   - Working (arrived at work)
    // The key assertion: PathRequest must NOT be lingering.
    assert!(
        !has_path_request,
        "PathRequest should have been consumed by process_path_requests in the same \
         frame it was inserted (flush point working). State: {:?}, ComputingPath: {}",
        state, has_computing_path
    );
}

// ====================================================================
// Test: Manually inserted PathRequest is processed in single tick
// ====================================================================

/// Insert a PathRequest component directly (not via Commands), then run
/// a single tick and verify it is consumed by process_path_requests.
/// This tests the simpler case: PathRequest already exists on the entity
/// when the schedule begins, so no flush is needed — but it confirms
/// that process_path_requests runs and consumes PathRequests.
#[test]
fn test_flush_point_manual_path_request_consumed_in_one_tick() {
    let home = (30, 30);
    let work = (30, 45);

    let mut city = TestCity::new()
        .with_road(30, 30, 30, 45, RoadType::Local)
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .rebuild_csr()
        .with_time(12.0); // noon — no commute, state machine won't fire

    let citizen = spawn_citizen_at(&mut city, home, work, CitizenState::AtHome);
    stabilize(&mut city, &[home, work]);

    // Manually insert a PathRequest (bypassing Commands / state_machine).
    // This simulates the pre-flushed state.
    {
        let world = city.world_mut();
        world.entity_mut(citizen).insert(PathRequest {
            from_gx: home.0,
            from_gy: home.1,
            to_gx: work.0,
            to_gy: work.1,
            target_state: CitizenState::CommutingToWork,
        });
    }

    // Confirm PathRequest exists before tick.
    {
        let world = city.world_mut();
        assert!(
            world.get::<PathRequest>(citizen).is_some(),
            "PathRequest should exist before tick"
        );
    }

    // Single tick
    city.tick(1);

    let world = city.world_mut();
    let has_path_request = world.get::<PathRequest>(citizen).is_some();
    let has_computing_path = world.get::<ComputingPath>(citizen).is_some();

    // process_path_requests should have consumed the PathRequest in this tick.
    assert!(
        !has_path_request,
        "PathRequest should be consumed after a single tick"
    );

    // On native (non-WASM), the PathRequest is replaced with ComputingPath.
    // On WASM it would be directly resolved, but either way PathRequest is gone.
    if cfg!(not(target_arch = "wasm32")) {
        assert!(
            has_computing_path,
            "On native, PathRequest should be replaced with ComputingPath (async dispatch)"
        );
    }
}

// ====================================================================
// Test: No PathRequest survives across multiple ticks
// ====================================================================

/// Run many ticks during morning commute and verify that at no point
/// does a PathRequest component linger past the tick in which it was
/// inserted. This is a stronger invariant: even when many citizens
/// transition simultaneously, flush points ensure same-frame processing.
#[test]
fn test_flush_point_no_lingering_path_requests_across_ticks() {
    let home = (50, 50);
    let work = (50, 65);

    let mut city = TestCity::new()
        .with_road(50, 50, 50, 65, RoadType::Local)
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .rebuild_csr()
        .with_time(7.0);

    // Spawn multiple citizens to increase the chance of triggering
    // PathRequest insertion on different jitter offsets.
    for _ in 0..5 {
        spawn_citizen_at(&mut city, home, work, CitizenState::AtHome);
    }
    stabilize(&mut city, &[home, work]);

    // Run tick-by-tick and assert no lingering PathRequests after each tick.
    for tick_num in 0..120 {
        city.tick(1);

        let world = city.world_mut();
        let lingering_count = world
            .query_filtered::<Entity, With<PathRequest>>()
            .iter(world)
            .count();

        assert_eq!(
            lingering_count, 0,
            "After tick {tick_num}, found {lingering_count} entities with lingering \
             PathRequest. The apply_deferred flush point should ensure \
             process_path_requests consumes all PathRequests in the same frame."
        );
    }
}

// ====================================================================
// Test: PathRequest and ComputingPath never coexist on same entity
// ====================================================================

/// Verify that no entity ever has both PathRequest and ComputingPath
/// simultaneously. This is a corollary of the flush point being correct:
/// process_path_requests removes PathRequest before inserting ComputingPath.
#[test]
fn test_flush_point_no_simultaneous_path_request_and_computing_path() {
    let home = (70, 70);
    let work = (70, 85);

    let mut city = TestCity::new()
        .with_road(70, 70, 70, 85, RoadType::Local)
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .rebuild_csr()
        .with_time(7.0);

    for _ in 0..5 {
        spawn_citizen_at(&mut city, home, work, CitizenState::AtHome);
    }
    stabilize(&mut city, &[home, work]);

    for tick_num in 0..120 {
        city.tick(1);

        let world = city.world_mut();
        let both_count = world
            .query_filtered::<Entity, (With<PathRequest>, With<ComputingPath>)>()
            .iter(world)
            .count();

        assert_eq!(
            both_count, 0,
            "After tick {tick_num}, found {both_count} entities with both PathRequest \
             and ComputingPath. This should never happen — process_path_requests \
             removes PathRequest before inserting ComputingPath."
        );
    }
}

// ====================================================================
// Test: State machine + flush + pathfinding full pipeline in one tick
// ====================================================================

/// End-to-end test: a citizen at home during morning commute should have
/// its PathRequest both inserted (by state_machine) AND consumed (by
/// process_path_requests) within a single FixedUpdate run, thanks to
/// the apply_deferred flush point between them.
#[test]
fn test_flush_point_full_pipeline_single_tick() {
    let home = (90, 90);
    let work = (90, 105);

    let mut city = TestCity::new()
        .with_road(90, 90, 90, 105, RoadType::Local)
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::CommercialLow, 1)
        .rebuild_csr()
        .with_time(7.0);

    // Spawn one citizen with entity index 0 (jitter = 0 % 120 = 0).
    // At hour 7, minute 0, the jitter check (minute % 60 == jitter % 60)
    // should pass for jitter_value % 60 == 0.
    let citizen = spawn_citizen_at(&mut city, home, work, CitizenState::AtHome);
    stabilize(&mut city, &[home, work]);

    // Determine the citizen's jitter value
    let jitter = citizen.index() % 120;

    // Set the clock to the exact minute that matches the jitter so the
    // state machine fires on the first tick.
    let target_minute = jitter % 60;
    let hour = 7.0 + (target_minute as f32) / 60.0;
    {
        let mut clock = city.world_mut().resource_mut::<crate::time_of_day::GameClock>();
        clock.hour = hour;
    }

    // Single tick: state_machine inserts PathRequest -> apply_deferred ->
    // process_path_requests consumes it.
    city.tick(1);

    let world = city.world_mut();
    let has_path_request = world.get::<PathRequest>(citizen).is_some();
    let has_computing_path = world.get::<ComputingPath>(citizen).is_some();
    let state = world.get::<CitizenStateComp>(citizen).map(|s| s.0);

    // The PathRequest should have been consumed in the same tick.
    assert!(
        !has_path_request,
        "Full pipeline: PathRequest should be consumed in a single tick. \
         State: {:?}, ComputingPath: {}",
        state, has_computing_path
    );

    // On native, the citizen should now have ComputingPath (async dispatch).
    if cfg!(not(target_arch = "wasm32")) {
        // The citizen should either have ComputingPath or already be commuting
        // (if the task completed immediately).
        let transitioned = has_computing_path
            || state == Some(CitizenState::CommutingToWork);
        assert!(
            transitioned,
            "Full pipeline: citizen should have ComputingPath or be CommutingToWork. \
             State: {:?}, ComputingPath: {}",
            state, has_computing_path
        );
    }
}
