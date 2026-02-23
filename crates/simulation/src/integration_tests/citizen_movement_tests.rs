//! TEST-051: Integration tests for citizen movement system.
//!
//! Covers: velocity application, path following, waypoint progression,
//! arrival detection, position clamping to grid bounds, and PathCache
//! index invariant.

use bevy::prelude::*;

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::roads::RoadNode;
use crate::test_harness::TestCity;

use crate::immigration::CityAttractiveness;// ====================================================================
// Helper: spawn a citizen in a specific state with an optional path
// ====================================================================

fn spawn_citizen_with_path(
    city: &mut TestCity,
    home: (usize, usize),
    work: (usize, usize),
    state: CitizenState,
    waypoints: Vec<RoadNode>,
    position: Option<(f32, f32)>,
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

    let (px, py) = position.unwrap_or_else(|| WorldGrid::grid_to_world(home.0, home.1));

    world
        .spawn((
            Citizen,
            Position { x: px, y: py },
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
            PathCache::new(waypoints),
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
            Family::default(),
            ActivityTimer::default(),
            ChosenTransportMode::default(),
        ))
        .id()
}

// ====================================================================
// Test: Velocity moves citizen position
// ====================================================================

#[test]
fn test_citizen_movement_velocity_changes_position() {
    // A commuting citizen with a path should have its position change
    // after ticking — the move_citizens system applies velocity.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 120, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 120, ZoneType::CommercialLow, 1)
        .rebuild_csr();

    // Place the citizen in a commuting state with waypoints far away.
    let waypoints = vec![
        RoadNode(100, 105),
        RoadNode(100, 110),
        RoadNode(100, 115),
        RoadNode(100, 120),
    ];
    let start_pos = WorldGrid::grid_to_world(100, 100);
    let entity = spawn_citizen_with_path(
        &mut city,
        (100, 100),
        (100, 120),
        CitizenState::CommutingToWork,
        waypoints,
        Some(start_pos),
    );

    // Record initial position
    let (initial_x, initial_y) = {
        let world = city.world_mut();
        let pos = world.get::<Position>(entity).unwrap();
        (pos.x, pos.y)
    };

    // Tick a few times so the movement system runs
    city.tick(5);

    // Position should have changed
    let (new_x, new_y) = {
        let world = city.world_mut();
        let pos = world.get::<Position>(entity).unwrap();
        (pos.x, pos.y)
    };

    let distance_moved =
        ((new_x - initial_x).powi(2) + (new_y - initial_y).powi(2)).sqrt();
    assert!(
        distance_moved > 0.1,
        "citizen position should change when commuting with a path, \
         moved only {distance_moved:.4} pixels"
    );
}

// ====================================================================
// Test: Velocity is zero when citizen is not commuting
// ====================================================================

#[test]
fn test_citizen_movement_velocity_zero_when_not_commuting() {
    // A citizen at home (not commuting) should have zero velocity.
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 110, ZoneType::CommercialLow, 1);

    let entity = spawn_citizen_with_path(
        &mut city,
        (100, 100),
        (100, 110),
        CitizenState::AtHome,
        Vec::new(),
        None,
    );

    city.tick(3);

    let (vx, vy) = {
        let world = city.world_mut();
        let vel = world.get::<Velocity>(entity).unwrap();
        (vel.x, vel.y)
    };

    assert!(
        vx.abs() < f32::EPSILON && vy.abs() < f32::EPSILON,
        "velocity should be zero for non-commuting citizen, got ({vx}, {vy})"
    );
}

// ====================================================================
// Test: Path following progresses through waypoints
// ====================================================================

#[test]
fn test_citizen_movement_path_following_progresses_waypoints() {
    // A commuting citizen should advance through waypoints over time.
    // We give a path with several waypoints placed close together so
    // that the citizen can reach them within a few ticks.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 106, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 106, ZoneType::CommercialLow, 1)
        .rebuild_csr();

    // Adjacent waypoints (16 pixels apart at CELL_SIZE=16) — citizen speed
    // is 48/10 = 4.8 px/tick, so each waypoint takes ~3-4 ticks to reach.
    let waypoints = vec![
        RoadNode(100, 101),
        RoadNode(100, 102),
        RoadNode(100, 103),
        RoadNode(100, 104),
        RoadNode(100, 105),
        RoadNode(100, 106),
    ];
    let start_pos = WorldGrid::grid_to_world(100, 100);
    let entity = spawn_citizen_with_path(
        &mut city,
        (100, 100),
        (100, 106),
        CitizenState::CommutingToWork,
        waypoints,
        Some(start_pos),
    );

    // Record initial path index
    let initial_index = {
        let world = city.world_mut();
        world.get::<PathCache>(entity).unwrap().current_index
    };

    // Tick enough for the citizen to advance at least 1 waypoint
    // (speed 4.8 px/tick, waypoint at 16px away => ~4 ticks per waypoint)
    city.tick(10);

    let new_index = {
        let world = city.world_mut();
        world.get::<PathCache>(entity).unwrap().current_index
    };

    assert!(
        new_index > initial_index,
        "path index should advance after movement ticks, \
         initial={initial_index}, current={new_index}"
    );
}

// ====================================================================
// Test: Arrival detection when close to waypoint
// ====================================================================

#[test]
fn test_citizen_movement_arrival_detection_advances_waypoint() {
    // Place a citizen very close to the first waypoint so that arrival
    // detection triggers immediately.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 103, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 103, ZoneType::CommercialLow, 1)
        .rebuild_csr();

    let first_waypoint = RoadNode(100, 101);
    let waypoints = vec![first_waypoint, RoadNode(100, 102), RoadNode(100, 103)];

    // Place citizen just 1 pixel away from the first waypoint center.
    let (wx, wy) = WorldGrid::grid_to_world(100, 101);
    let entity = spawn_citizen_with_path(
        &mut city,
        (100, 100),
        (100, 103),
        CitizenState::CommutingToWork,
        waypoints,
        Some((wx - 1.0, wy)),
    );

    // One tick should be enough for arrival detection (distance < arrival_dist).
    city.tick(1);

    let index_after = {
        let world = city.world_mut();
        world.get::<PathCache>(entity).unwrap().current_index
    };

    assert!(
        index_after >= 1,
        "citizen near a waypoint should advance past it, current_index={index_after}"
    );
}

// ====================================================================
// Test: Path completion sets velocity to zero
// ====================================================================

#[test]
fn test_citizen_movement_path_complete_velocity_zero() {
    // A commuting citizen whose path is already complete (empty waypoints)
    // should have zero velocity after a tick.
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 110, ZoneType::CommercialLow, 1);

    let entity = spawn_citizen_with_path(
        &mut city,
        (100, 100),
        (100, 110),
        CitizenState::CommutingToWork,
        Vec::new(), // empty path => complete
        None,
    );

    city.tick(1);

    let (vx, vy) = {
        let world = city.world_mut();
        let vel = world.get::<Velocity>(entity).unwrap();
        (vel.x, vel.y)
    };

    assert!(
        vx.abs() < f32::EPSILON && vy.abs() < f32::EPSILON,
        "velocity should be zero when path is complete, got ({vx}, {vy})"
    );
}

// ====================================================================
// Test: PathCache index never exceeds waypoints.len()
// ====================================================================

#[test]
fn test_citizen_movement_path_cache_index_invariant() {
    // After many ticks, the PathCache current_index must never exceed
    // waypoints.len(). This is the core invariant from PathCache::advance().
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 105, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 105, ZoneType::CommercialLow, 1)
        .rebuild_csr();

    let waypoints = vec![
        RoadNode(100, 101),
        RoadNode(100, 102),
        RoadNode(100, 103),
        RoadNode(100, 104),
        RoadNode(100, 105),
    ];
    let start_pos = WorldGrid::grid_to_world(100, 100);
    let entity = spawn_citizen_with_path(
        &mut city,
        (100, 100),
        (100, 105),
        CitizenState::CommutingToWork,
        waypoints.clone(),
        Some(start_pos),
    );

    // Tick many times — enough to complete the path and then some
    // Prevent emigration during the long tick run.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }
    city.tick(100);

    let (index, len) = {
        let world = city.world_mut();
        let path = world.get::<PathCache>(entity).unwrap();
        (path.current_index, path.waypoints.len())
    };

    assert!(
        index <= len,
        "PathCache.current_index ({index}) must be <= waypoints.len() ({len})"
    );
}

// ====================================================================
// Test: Citizen position stays within world bounds
// ====================================================================

#[test]
fn test_citizen_movement_position_within_world_bounds() {
    // Place a citizen near the edge of the grid and give it a path
    // that stays near the edge. After ticking, position should remain
    // within the world coordinate range.
    let mut city = TestCity::new()
        .with_road(253, 253, 253, 255, RoadType::Local)
        .with_building(253, 253, ZoneType::ResidentialLow, 1)
        .with_building(253, 255, ZoneType::CommercialLow, 1)
        .rebuild_csr();

    let waypoints = vec![RoadNode(253, 254), RoadNode(253, 255)];
    let start_pos = WorldGrid::grid_to_world(253, 253);
    let entity = spawn_citizen_with_path(
        &mut city,
        (253, 253),
        (253, 255),
        CitizenState::CommutingToWork,
        waypoints,
        Some(start_pos),
    );

    // Prevent emigration.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }
    city.tick(30);

    let (px, py) = {
        let world = city.world_mut();
        let pos = world.get::<Position>(entity).unwrap();
        (pos.x, pos.y)
    };

    // World bounds: 0.0 to GRID_SIZE * CELL_SIZE = 256 * 16 = 4096
    let world_max = 256.0 * 16.0;
    assert!(
        px >= -1.0 && px <= world_max + 1.0,
        "x position ({px}) should be within world bounds [0, {world_max}]"
    );
    assert!(
        py >= -1.0 && py <= world_max + 1.0,
        "y position ({py}) should be within world bounds [0, {world_max}]"
    );
}

// ====================================================================
// Test: Citizen moves toward waypoint (direction is correct)
// ====================================================================

#[test]
fn test_citizen_movement_direction_toward_waypoint() {
    // A citizen commuting along a vertical road (increasing y) should
    // have positive y velocity, confirming movement in the correct
    // direction toward the waypoint.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 120, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 120, ZoneType::CommercialLow, 1)
        .rebuild_csr();

    // Waypoints go from y=105 to y=120 (increasing y direction)
    let waypoints = vec![
        RoadNode(100, 105),
        RoadNode(100, 110),
        RoadNode(100, 115),
        RoadNode(100, 120),
    ];
    let start_pos = WorldGrid::grid_to_world(100, 100);
    let entity = spawn_citizen_with_path(
        &mut city,
        (100, 100),
        (100, 120),
        CitizenState::CommutingToWork,
        waypoints,
        Some(start_pos),
    );

    // Tick once for the movement system to set velocity
    city.tick(1);

    let vy = {
        let world = city.world_mut();
        world.get::<Velocity>(entity).unwrap().y
    };

    // The waypoint is at a higher y-coordinate, so the citizen should
    // move in the positive y direction. The smoothing may add a small
    // component, but the primary direction should be positive y.
    assert!(
        vy > 0.0,
        "citizen should move toward waypoint (positive y direction), got vy={vy}"
    );
}
