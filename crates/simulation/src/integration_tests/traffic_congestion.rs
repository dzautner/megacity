use crate::citizen::{Citizen, CitizenState, CitizenStateComp, PathCache, Position};
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::roads::RoadNode;
use crate::test_harness::TestCity;
use crate::traffic_congestion::TrafficCongestion;
use crate::immigration::CityAttractiveness;
// ====================================================================
// Traffic congestion tests
// ====================================================================

#[test]
fn test_traffic_congestion_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<TrafficCongestion>();
}

#[test]
fn test_traffic_congestion_defaults_to_free_flow() {
    let city = TestCity::new();
    let congestion = city.resource::<TrafficCongestion>();
    assert!(
        (congestion.get(10, 10) - 1.0).abs() < f32::EPSILON,
        "Default congestion multiplier should be 1.0"
    );
}

#[test]
fn test_citizens_move_slower_on_congested_roads() {
    use crate::traffic::TrafficGrid;

    let mut city = TestCity::new()
        .with_road(50, 50, 80, 50, RoadType::Local)
        .with_building(48, 50, ZoneType::ResidentialLow, 1)
        .with_building(82, 50, ZoneType::CommercialLow, 1)
        .with_citizen((48, 50), (82, 50))
        .with_time(7.0);

    // Prevent emigration during tick runs.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }
    // Manually set citizen to commuting state with a path
    {
        let world = city.world_mut();
        let waypoints: Vec<RoadNode> = (50..=80).map(|x| RoadNode(x, 50)).collect();
        let (wx, wy) = WorldGrid::grid_to_world(50, 50);
        let mut q = world.query_filtered::<(
            &mut CitizenStateComp,
            &mut PathCache,
            &mut Position,
        ), bevy::prelude::With<Citizen>>();
        for (mut state, mut path, mut pos) in q.iter_mut(world) {
            state.0 = CitizenState::CommutingToWork;
            *path = PathCache::new(waypoints.clone());
            pos.x = wx;
            pos.y = wy;
        }
    }

    let start_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };

    // Run ticks at free flow
    city.tick(10);

    let free_flow_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };
    let free_flow_dist =
        ((free_flow_pos.0 - start_pos.0).powi(2) + (free_flow_pos.1 - start_pos.1).powi(2)).sqrt();

    // Reset citizen and inject congestion on BOTH TrafficGrid and TrafficCongestion
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        for x in 50..=80 {
            traffic.set(x, 50, 20); // at Local capacity
        }
        let mut congestion = world.resource_mut::<TrafficCongestion>();
        for x in 50..=80 {
            congestion.set(x, 50, 0.1);
        }
        let waypoints: Vec<RoadNode> = (50..=80).map(|x| RoadNode(x, 50)).collect();
        let (wx, wy) = WorldGrid::grid_to_world(50, 50);
        let mut q = world.query_filtered::<(
            &mut CitizenStateComp,
            &mut PathCache,
            &mut Position,
        ), bevy::prelude::With<Citizen>>();
        for (mut state, mut path, mut pos) in q.iter_mut(world) {
            state.0 = CitizenState::CommutingToWork;
            *path = PathCache::new(waypoints.clone());
            pos.x = wx;
            pos.y = wy;
        }
    }

    city.tick(10);

    let congested_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };
    let congested_dist =
        ((congested_pos.0 - start_pos.0).powi(2) + (congested_pos.1 - start_pos.1).powi(2)).sqrt();

    assert!(
        free_flow_dist > 1.0,
        "Citizen should have moved during free flow, dist={}",
        free_flow_dist
    );
    assert!(
        congested_dist < free_flow_dist,
        "Citizen should move slower under congestion. Free flow dist={}, congested dist={}",
        free_flow_dist,
        congested_dist
    );
}

#[test]
fn test_speed_returns_to_normal_when_congestion_clears() {
    use crate::traffic::TrafficGrid;

    let mut city = TestCity::new()
        .with_road(50, 50, 120, 50, RoadType::Local)
        .with_building(48, 50, ZoneType::ResidentialLow, 1)
        .with_building(122, 50, ZoneType::CommercialLow, 1)
        .with_citizen((48, 50), (122, 50))
        .with_time(7.0);

    // Prevent emigration during tick runs.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }
    // Set citizen to commuting with congestion
    {
        let world = city.world_mut();
        let waypoints: Vec<RoadNode> = (50..=120).map(|x| RoadNode(x, 50)).collect();
        let (wx, wy) = WorldGrid::grid_to_world(50, 50);
        let mut q = world.query_filtered::<(
            &mut CitizenStateComp,
            &mut PathCache,
            &mut Position,
        ), bevy::prelude::With<Citizen>>();
        for (mut state, mut path, mut pos) in q.iter_mut(world) {
            state.0 = CitizenState::CommutingToWork;
            *path = PathCache::new(waypoints.clone());
            pos.x = wx;
            pos.y = wy;
        }
        let mut traffic = world.resource_mut::<TrafficGrid>();
        for x in 50..=120 {
            traffic.set(x, 50, 20);
        }
        let mut congestion = world.resource_mut::<TrafficCongestion>();
        for x in 50..=120 {
            congestion.set(x, 50, 0.1);
        }
    }

    let start_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };

    city.tick(10);

    let congested_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };
    let congested_dist =
        ((congested_pos.0 - start_pos.0).powi(2) + (congested_pos.1 - start_pos.1).powi(2)).sqrt();

    // Clear congestion and reset citizen position + path for the free-flow phase.
    // We reset position to the same starting point as the congested phase so the
    // distance comparison is fair (both phases start from grid cell 50,50).
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        for x in 50..=120 {
            traffic.set(x, 50, 0);
        }
        let mut congestion = world.resource_mut::<TrafficCongestion>();
        for x in 50..=120 {
            congestion.set(x, 50, 1.0);
        }
        let waypoints: Vec<RoadNode> = (50..=120).map(|x| RoadNode(x, 50)).collect();
        let (wx, wy) = WorldGrid::grid_to_world(50, 50);
        let mut q = world.query_filtered::<(
            &mut CitizenStateComp,
            &mut PathCache,
            &mut Position,
        ), bevy::prelude::With<Citizen>>();
        for (mut state, mut path, mut pos) in q.iter_mut(world) {
            state.0 = CitizenState::CommutingToWork;
            *path = PathCache::new(waypoints.clone());
            pos.x = wx;
            pos.y = wy;
        }
    }

    let mid_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };

    city.tick(10);

    let free_flow_pos = {
        let world = city.world_mut();
        let mut q = world.query_filtered::<&Position, bevy::prelude::With<Citizen>>();
        let pos = q.iter(world).next().expect("should have a citizen");
        (pos.x, pos.y)
    };
    let free_flow_dist =
        ((free_flow_pos.0 - mid_pos.0).powi(2) + (free_flow_pos.1 - mid_pos.1).powi(2)).sqrt();

    assert!(
        congested_dist > 0.1,
        "Citizen should move even under congestion (min speed floor), dist={}",
        congested_dist
    );
    assert!(
        free_flow_dist > congested_dist,
        "Speed should return to normal after congestion clears. \
         Congested dist={}, free flow dist={}",
        congested_dist,
        free_flow_dist
    );
}

#[test]
fn test_higher_capacity_roads_congest_less() {
    use crate::traffic_congestion::congestion_speed_multiplier;

    let local_ratio = 15.0 / 20.0;
    let local_mult = congestion_speed_multiplier(local_ratio);
    let highway_ratio = 15.0 / 80.0;
    let highway_mult = congestion_speed_multiplier(highway_ratio);

    assert!(
        highway_mult > local_mult,
        "Highway should be less congested than Local at same volume. Highway={}, Local={}",
        highway_mult,
        local_mult
    );
    assert!(
        local_mult < 0.5,
        "Local at 75% capacity should have multiplier < 0.5, got {}",
        local_mult
    );
    assert!(
        highway_mult > 0.9,
        "Highway at ~19% capacity should have multiplier > 0.9, got {}",
        highway_mult
    );
}

// Async pathfinding tests
// ===========================================================================

#[test]
fn test_async_pathfinding_snapshot_initialized() {
    let city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);
    city.assert_resource_exists::<crate::movement::PathfindingSnapshot>();
}

#[test]
fn test_async_pathfinding_citizen_gets_path() {
    use crate::movement::ComputingPath;

    let mut city = TestCity::new()
        .with_road(5, 10, 25, 10, RoadType::Local)
        .with_building(5, 9, ZoneType::ResidentialLow, 1)
        .with_building(25, 9, ZoneType::CommercialLow, 1)
        .rebuild_csr()
        .with_citizen((5, 9), (25, 9))
        .with_time(7.0); // start of morning commute window
    // Prevent emigration during the long tick run.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    // Run 120 ticks (= 2 in-game hours) to cover the full morning commute window
    // (hours 7-8). This guarantees hitting any per-entity departure jitter value.
    // Async tasks run on background threads and are polled each tick via
    // `block_on(poll_once(...))`. A yield_now() in the test harness tick loop
    // gives background threads a chance to complete.
    city.tick(120);

    let world = city.world_mut();
    let not_at_home = world
        .query::<&crate::citizen::CitizenStateComp>()
        .iter(world)
        .filter(|s| s.0 != CitizenState::AtHome)
        .count();
    let computing = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<ComputingPath>>()
        .iter(world)
        .count();

    // Citizen should have started pathfinding (ComputingPath) or transitioned
    assert!(
        not_at_home > 0 || computing > 0,
        "citizen should have started pathfinding or left home (not_at_home={not_at_home}, computing={computing})"
    );
}

#[test]
fn test_async_pathfinding_no_road_no_crash() {
    // Citizens with no road connectivity should not crash the async system
    let mut city = TestCity::new()
        .with_building(5, 5, ZoneType::ResidentialLow, 1)
        .with_building(50, 50, ZoneType::CommercialLow, 1)
        .with_citizen((5, 5), (50, 50))
        .with_time(7.0);
    // Prevent emigration during the long tick run.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    // Should not panic even with no roads
    city.tick(120);

    // Citizen should still exist (spawner may add more during ticks)
    assert!(
        city.citizen_count() >= 1,
        "original citizen should still exist"
    );
}

#[test]
fn test_async_pathfinding_computing_path_prevents_requeue() {
    use crate::citizen::PathRequest;
    use crate::movement::ComputingPath;
    use bevy::prelude::Entity;

    let mut city = TestCity::new()
        .with_road(5, 10, 25, 10, RoadType::Local)
        .with_building(5, 9, ZoneType::ResidentialLow, 1)
        .with_building(25, 9, ZoneType::CommercialLow, 1)
        .rebuild_csr()
        .with_citizen((5, 9), (25, 9))
        .with_time(7.0);
    // Prevent emigration during the long tick run.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    // Run enough ticks for the state machine to fire and pathfinding to dispatch.
    city.tick(120);

    // Verify no entity has BOTH PathRequest and ComputingPath simultaneously.
    // This would indicate the state machine re-queued a citizen that is already
    // being processed by the async pathfinding system.
    let world = city.world_mut();
    let double_queued = world
        .query_filtered::<Entity, (
            bevy::prelude::With<PathRequest>,
            bevy::prelude::With<ComputingPath>,
        )>()
        .iter(world)
        .count();

    assert_eq!(
        double_queued, 0,
        "no entity should have both PathRequest and ComputingPath"
    );
}

#[test]
fn test_async_pathfinding_multiple_citizens() {
    use crate::movement::ComputingPath;

    let mut city = TestCity::new()
        .with_road(5, 10, 30, 10, RoadType::Local)
        .with_building(5, 9, ZoneType::ResidentialLow, 1)
        .with_building(30, 9, ZoneType::CommercialLow, 1)
        .rebuild_csr()
        .with_citizen((5, 9), (30, 9))
        .with_citizen((5, 9), (30, 9))
        .with_citizen((5, 9), (30, 9))
        .with_time(7.0);
    // Prevent emigration during the long tick run.
    {
        let mut attr = city.world_mut().resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    // Run 120 ticks to cover the full morning commute window (hours 7-8),
    // ensuring all citizens hit their departure jitter.
    city.tick(120);

    let world = city.world_mut();
    let not_at_home = world
        .query::<&crate::citizen::CitizenStateComp>()
        .iter(world)
        .filter(|s| s.0 != CitizenState::AtHome)
        .count();
    let computing = world
        .query_filtered::<bevy::prelude::Entity, bevy::prelude::With<ComputingPath>>()
        .iter(world)
        .count();

    // At least some citizens should have started pathfinding or transitioned
    assert!(
        not_at_home > 0 || computing > 0,
        "some citizens should be pathfinding or have left home (not_at_home={not_at_home}, computing={computing})"
    );
}

#[test]
fn test_async_pathfinding_snapshot_updates_on_road_change() {
    let mut city = TestCity::new().with_road(5, 10, 15, 10, RoadType::Local);

    let v1 = city
        .resource::<crate::movement::PathfindingSnapshot>()
        .version;

    // Add more road and tick to trigger CSR rebuild + snapshot update
    city = city.with_road(15, 10, 25, 10, RoadType::Local);
    city.tick(2);

    let v2 = city
        .resource::<crate::movement::PathfindingSnapshot>()
        .version;
    assert!(
        v2 > v1,
        "snapshot version should increase after road network change (v1={}, v2={})",
        v1,
        v2
    );
}
