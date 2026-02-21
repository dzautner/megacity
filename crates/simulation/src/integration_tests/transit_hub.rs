use crate::land_value::LandValueGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// =============================================================================
// Transit Hub / Multi-Modal Stations (TRAF-015)
// =============================================================================

/// Test that transit hub resources are initialized when SimulationPlugin starts.
#[test]
fn test_transit_hub_resources_exist() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::transit_hub::TransitHubs>();
    city.assert_resource_exists::<crate::transit_hub::TransitHubStats>();
}

/// Test that co-located bus depot and subway station form a BusMetroHub.
#[test]
fn test_transit_hub_creation_bus_metro() {
    use crate::transit_hub::{TransitHubType, TransitHubs};

    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::BusDepot)
        .with_service(51, 50, ServiceType::SubwayStation);

    // Run a slow cycle so update_transit_hubs fires
    city.tick_slow_cycle();

    let hubs = city.world_mut().resource::<TransitHubs>();
    assert!(
        !hubs.hubs.is_empty(),
        "Expected at least one transit hub from co-located bus + subway"
    );
    assert_eq!(
        hubs.hubs[0].hub_type,
        TransitHubType::BusMetroHub,
        "Expected BusMetroHub type"
    );
}

/// Test that co-located train station and subway station form a TrainMetroHub.
#[test]
fn test_transit_hub_creation_train_metro() {
    use crate::transit_hub::{TransitHubType, TransitHubs};

    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::TrainStation)
        .with_service(51, 51, ServiceType::SubwayStation);

    city.tick_slow_cycle();

    let hubs = city.world_mut().resource::<TransitHubs>();
    assert!(
        !hubs.hubs.is_empty(),
        "Expected at least one transit hub from co-located train + subway"
    );
    assert_eq!(hubs.hubs[0].hub_type, TransitHubType::TrainMetroHub);
}

/// Test that 3+ transit modes co-located form a MultiModalHub.
#[test]
fn test_transit_hub_creation_multi_modal() {
    use crate::transit_hub::{TransitHubType, TransitHubs};

    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::BusDepot)
        .with_service(51, 50, ServiceType::SubwayStation)
        .with_service(50, 51, ServiceType::TrainStation);

    city.tick_slow_cycle();

    let hubs = city.world_mut().resource::<TransitHubs>();
    assert!(
        !hubs.hubs.is_empty(),
        "Expected a multi-modal hub from 3 co-located transit types"
    );
    assert_eq!(hubs.hubs[0].hub_type, TransitHubType::MultiModalHub);
}

/// Test that isolated transit stops do NOT form hubs.
#[test]
fn test_transit_hub_no_hub_for_isolated_stops() {
    use crate::transit_hub::TransitHubs;

    let mut city = TestCity::new()
        .with_service(10, 10, ServiceType::BusDepot)
        .with_service(100, 100, ServiceType::SubwayStation);

    city.tick_slow_cycle();

    let hubs = city.world_mut().resource::<TransitHubs>();
    assert!(
        hubs.hubs.is_empty(),
        "Isolated transit stops should not form a hub"
    );
}

/// Test that transfer penalty is reduced at hub locations.
#[test]
fn test_transit_hub_transfer_penalty_reduction() {
    use crate::transit_hub::{
        TransitHubs, TransitMode, DEFAULT_TRANSFER_PENALTY_MINUTES, HUB_TRANSFER_PENALTY_MINUTES,
    };

    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::BusDepot)
        .with_service(51, 50, ServiceType::SubwayStation);

    city.tick_slow_cycle();

    let hubs = city.world_mut().resource::<TransitHubs>();

    // At hub location: reduced penalty
    let penalty_at_hub = hubs.transfer_penalty_at(50, 50, TransitMode::Bus, TransitMode::Metro);
    assert!(
        (penalty_at_hub - HUB_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON,
        "Transfer penalty at hub should be {HUB_TRANSFER_PENALTY_MINUTES}, got {penalty_at_hub}"
    );

    // Away from hub: default penalty
    let penalty_away = hubs.transfer_penalty_at(200, 200, TransitMode::Bus, TransitMode::Metro);
    assert!(
        (penalty_away - DEFAULT_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON,
        "Transfer penalty away from hub should be {DEFAULT_TRANSFER_PENALTY_MINUTES}, got {penalty_away}"
    );
}

/// Test that hub land value boost is higher than individual station boost.
#[test]
fn test_transit_hub_land_value_boost() {
    use crate::transit_hub::{HUB_LAND_VALUE_MULTIPLIER, TRANSIT_STATION_BASE_BOOST};

    // Place a hub (bus + subway co-located) and measure land value nearby.
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::BusDepot)
        .with_service(129, 128, ServiceType::SubwayStation);

    city.tick_slow_cycle();

    let lv = city.world_mut().resource::<LandValueGrid>();
    let hub_lv = lv.get(128, 128);

    // Also create a city with just a single bus depot (no hub).
    let mut city_single = TestCity::new().with_service(128, 128, ServiceType::BusDepot);

    city_single.tick_slow_cycle();

    let lv_single = city_single.world_mut().resource::<LandValueGrid>();
    let single_lv = lv_single.get(128, 128);

    // The hub location should have at least as much land value as the single station.
    // The hub provides an additional boost via transit_hub_land_value system.
    assert!(
        hub_lv >= single_lv,
        "Hub land value ({hub_lv}) should be >= single station land value ({single_lv})"
    );

    // Verify the hub boost constant is correct
    let hub_boost = (TRANSIT_STATION_BASE_BOOST as f32 * HUB_LAND_VALUE_MULTIPLIER) as i32;
    assert!(
        hub_boost > TRANSIT_STATION_BASE_BOOST,
        "Hub boost ({hub_boost}) must exceed individual station boost ({TRANSIT_STATION_BASE_BOOST})"
    );
}

/// Test that hub stats are updated correctly.
#[test]
fn test_transit_hub_stats_update() {
    use crate::transit_hub::TransitHubStats;

    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::BusDepot)
        .with_service(51, 50, ServiceType::SubwayStation)
        .with_service(100, 100, ServiceType::TrainStation)
        .with_service(101, 100, ServiceType::SubwayStation);

    city.tick_slow_cycle();

    let stats = city.world_mut().resource::<TransitHubStats>();
    assert_eq!(
        stats.total_hubs, 2,
        "Expected 2 hubs, got {}",
        stats.total_hubs
    );
    assert!(stats.bus_metro_hubs >= 1, "Expected at least 1 BusMetroHub");
    assert!(
        stats.train_metro_hubs >= 1,
        "Expected at least 1 TrainMetroHub"
    );
}
