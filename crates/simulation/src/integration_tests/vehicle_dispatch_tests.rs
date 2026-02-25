//! Integration tests for SVC-003: Service Vehicle Dispatch System.

use crate::buildings::Building;
use crate::fire::OnFire;
use crate::grid::RoadType;
use crate::service_vehicle_dispatch::{
    DispatchMetrics, IncidentRequest, PendingIncidents, ServiceVehicle, VehiclePool,
    VehicleType,
};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Helper: city with a road, a fire station, and a burning building
// ---------------------------------------------------------------------------

fn city_with_fire_station_and_fire() -> TestCity {
    // Road from x=90..=110 at y=100
    // Fire station at (90, 99) — adjacent to road
    // Burning building at (110, 99) — adjacent to road
    let mut city = TestCity::new()
        .with_road(90, 100, 110, 100, RoadType::Local)
        .with_service(90, 99, ServiceType::FireStation);

    // Spawn a building that's on fire
    let entity = city
        .world_mut()
        .spawn((
            Building {
                zone_type: crate::grid::ZoneType::Industrial,
                level: 1,
                grid_x: 110,
                grid_y: 99,
                capacity: 10,
                occupants: 5,
            },
            OnFire {
                intensity: 50.0,
                ticks_burning: 0,
            },
        ))
        .id();
    {
        let mut grid = city.world_mut().resource_mut::<crate::grid::WorldGrid>();
        if grid.in_bounds(110, 99) {
            grid.get_mut(110, 99).building_id = Some(entity);
        }
    }

    city
}

// ===========================================================================
// 1. Vehicle pool attached to service buildings
// ===========================================================================

#[test]
fn test_vehicle_pool_attached_on_spawn() {
    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::FireStation);

    // Run a tick so the attach_vehicle_pools system runs
    city.tick(1);

    let world = city.world_mut();
    let count = world
        .query::<&VehiclePool>()
        .iter(world)
        .count();
    assert_eq!(count, 1, "Fire station should have a VehiclePool");

    let pool = world.query::<&VehiclePool>().iter(world).next().unwrap();
    assert_eq!(pool.total_vehicles, 3, "FireStation should have 3 vehicles");
    assert_eq!(pool.dispatched_count, 0);
}

// ===========================================================================
// 2. Nearest vehicle dispatched to incident
// ===========================================================================

#[test]
fn test_nearest_vehicle_dispatched() {
    let mut city = TestCity::new()
        .with_road(80, 100, 120, 100, RoadType::Local)
        .with_service(80, 99, ServiceType::FireStation)
        .with_service(120, 99, ServiceType::FireStation);

    city.tick(1); // attach pools

    // Inject incident near (110, 99) — closer to the station at (120, 99)
    {
        let mut pending = city.world_mut().resource_mut::<PendingIncidents>();
        pending.requests.push(IncidentRequest {
            vehicle_type: VehicleType::FireTruck,
            target_x: 110,
            target_y: 99,
        });
    }

    city.tick(1); // dispatch runs

    let world = city.world_mut();
    let vehicles: Vec<&ServiceVehicle> = world
        .query::<&ServiceVehicle>()
        .iter(world)
        .collect();
    assert_eq!(vehicles.len(), 1, "One vehicle should be dispatched");
    assert_eq!(vehicles[0].station_x, 120, "Nearest station (120,99) should dispatch");
}

// ===========================================================================
// 3. All vehicles occupied = no response
// ===========================================================================

#[test]
fn test_all_vehicles_occupied_fails_dispatch() {
    let mut city = TestCity::new()
        .with_road(90, 100, 110, 100, RoadType::Local)
        .with_service(90, 99, ServiceType::FireHouse); // only 1 vehicle

    city.tick(1); // attach pools

    // Dispatch first incident
    {
        let mut pending = city.world_mut().resource_mut::<PendingIncidents>();
        pending.requests.push(IncidentRequest {
            vehicle_type: VehicleType::FireTruck,
            target_x: 100,
            target_y: 99,
        });
    }
    city.tick(1);

    // Now all vehicles are busy; dispatch another
    {
        let mut pending = city.world_mut().resource_mut::<PendingIncidents>();
        pending.requests.push(IncidentRequest {
            vehicle_type: VehicleType::FireTruck,
            target_x: 105,
            target_y: 99,
        });
    }
    city.tick(1);

    let metrics = city.resource::<DispatchMetrics>();
    assert_eq!(metrics.fire_dispatches, 1, "Only one dispatch should succeed");
    assert_eq!(metrics.failed_dispatches, 1, "Second dispatch should fail");
}

// ===========================================================================
// 4. Response time based on road distance
// ===========================================================================

#[test]
fn test_response_time_based_on_distance() {
    let mut city = TestCity::new()
        .with_road(90, 100, 110, 100, RoadType::Local)
        .with_service(90, 99, ServiceType::FireStation);

    city.tick(1); // attach pools

    // Dispatch to (110, 99) — ~20 cells road distance
    {
        let mut pending = city.world_mut().resource_mut::<PendingIncidents>();
        pending.requests.push(IncidentRequest {
            vehicle_type: VehicleType::FireTruck,
            target_x: 110,
            target_y: 99,
        });
    }
    city.tick(1);

    let metrics = city.resource::<DispatchMetrics>();
    assert_eq!(metrics.fire_dispatches, 1);
    // Response ticks should be > 0 and proportional to distance
    assert!(
        metrics.fire_total_response_ticks > 0,
        "Response time should be positive"
    );
}

// ===========================================================================
// 5. Vehicle lifecycle: respond -> scene -> return -> despawn
// ===========================================================================

#[test]
fn test_vehicle_lifecycle() {
    let mut city = TestCity::new()
        .with_road(100, 100, 105, 100, RoadType::Local)
        .with_service(100, 99, ServiceType::FireStation);

    city.tick(1); // attach pools

    // Dispatch with a short distance
    {
        let mut pending = city.world_mut().resource_mut::<PendingIncidents>();
        pending.requests.push(IncidentRequest {
            vehicle_type: VehicleType::FireTruck,
            target_x: 105,
            target_y: 99,
        });
    }
    city.tick(1); // dispatch

    // Vehicle should exist
    let world = city.world_mut();
    let count = world
        .query::<&ServiceVehicle>()
        .iter(world)
        .count();
    assert!(count > 0, "Vehicle should be spawned");

    // Tick enough for travel + scene + return (generous upper bound)
    city.tick(200);

    // Vehicle should be despawned after completing lifecycle
    let world = city.world_mut();
    let remaining = world
        .query::<&ServiceVehicle>()
        .iter(world)
        .count();
    assert_eq!(remaining, 0, "Vehicle should despawn after returning");

    // Pool should be restored
    let pool = world.query::<&VehiclePool>().iter(world).next().unwrap();
    assert_eq!(pool.dispatched_count, 0, "Pool should be restored after return");
}

// ===========================================================================
// 6. Dispatched vehicles reduce station capacity
// ===========================================================================

#[test]
fn test_dispatched_vehicles_reduce_pool() {
    let mut city = TestCity::new()
        .with_road(90, 100, 110, 100, RoadType::Local)
        .with_service(90, 99, ServiceType::FireStation); // 3 vehicles

    city.tick(1); // attach pools

    // Dispatch two incidents
    {
        let mut pending = city.world_mut().resource_mut::<PendingIncidents>();
        pending.requests.push(IncidentRequest {
            vehicle_type: VehicleType::FireTruck,
            target_x: 100,
            target_y: 99,
        });
        pending.requests.push(IncidentRequest {
            vehicle_type: VehicleType::FireTruck,
            target_x: 105,
            target_y: 99,
        });
    }
    city.tick(1);

    let world = city.world_mut();
    let pool = world.query::<&VehiclePool>().iter(world).next().unwrap();
    assert_eq!(pool.dispatched_count, 2, "Two vehicles should be dispatched");
    assert_eq!(pool.available(), 1, "One vehicle should remain available");
}

// ===========================================================================
// 7. Fire incident scan creates dispatch requests
// ===========================================================================

#[test]
fn test_fire_scan_dispatches_fire_truck() {
    let mut city = city_with_fire_station_and_fire();

    // Run enough ticks for scan + dispatch (scan runs every 5 ticks)
    city.tick(10);

    let metrics = city.resource::<DispatchMetrics>();
    assert!(
        metrics.fire_dispatches > 0,
        "Fire scan should trigger at least one dispatch"
    );
}
