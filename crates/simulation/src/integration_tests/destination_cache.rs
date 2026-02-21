use crate::buildings::Building;
use crate::grid::ZoneType;
use crate::movement::DestinationCache;
use crate::services::{ServiceBuilding, ServiceType};
use crate::test_harness::TestCity;

#[test]
fn test_destination_cache_removes_demolished_building() {
    // Build a city with a commercial building (which will appear in shops cache)
    let mut city = TestCity::new().with_building(10, 10, ZoneType::CommercialLow, 1);

    // Tick to let the destination cache populate
    city.tick(2);

    // Verify the building is in the shops cache
    {
        let cache = city.resource::<DestinationCache>();
        assert!(
            cache.shops.contains(&(10, 10)),
            "commercial building should be in shops cache after ticking"
        );
    }

    // Find and despawn the building entity (simulating bulldoze)
    let building_entity = {
        let world = city.world_mut();
        let mut query = world.query::<(bevy::prelude::Entity, &Building)>();
        let (entity, _) = query.iter(world).next().expect("should have a building");
        entity
    };
    city.world_mut().despawn(building_entity);

    // Tick again so that RemovedComponents fires and cache rebuilds
    city.tick(2);

    // Verify the building is no longer in the shops cache
    let cache = city.resource::<DestinationCache>();
    assert!(
        !cache.shops.contains(&(10, 10)),
        "demolished building should NOT be in shops cache"
    );
}

#[test]
fn test_destination_cache_removes_demolished_service() {
    // Build a city with a leisure service (park)
    let mut city = TestCity::new().with_service(15, 15, ServiceType::SmallPark);

    // Tick to populate destination cache
    city.tick(2);

    // Verify the service is in the leisure cache
    {
        let cache = city.resource::<DestinationCache>();
        assert!(
            cache.leisure.contains(&(15, 15)),
            "park should be in leisure cache after ticking"
        );
    }

    // Find and despawn the service entity
    let service_entity = {
        let world = city.world_mut();
        let mut query = world.query::<(bevy::prelude::Entity, &ServiceBuilding)>();
        let (entity, _) = query
            .iter(world)
            .next()
            .expect("should have a service building");
        entity
    };
    city.world_mut().despawn(service_entity);

    // Tick again so RemovedComponents fires
    city.tick(2);

    // Verify the service is no longer in the leisure cache
    let cache = city.resource::<DestinationCache>();
    assert!(
        !cache.leisure.contains(&(15, 15)),
        "demolished park should NOT be in leisure cache"
    );
}
