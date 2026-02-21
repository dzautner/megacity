use crate::grid::{CellType, RoadType, WorldGrid};
use crate::outside_connections::{ConnectionType, OutsideConnections};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ====================================================================
// Outside Connections integration tests (TEST-070)
// ====================================================================

#[test]
fn test_outside_connections_resource_initialized() {
    let city = TestCity::new();
    city.assert_resource_exists::<OutsideConnections>();
}

#[test]
fn test_outside_connections_empty_city_has_none() {
    let mut city = TestCity::new();
    // Run enough ticks to trigger the update (UPDATE_INTERVAL=100)
    city.tick(101);

    let outside = city.resource::<OutsideConnections>();
    assert!(
        outside.connections.is_empty(),
        "Empty city should have no outside connections"
    );
}

#[test]
fn test_outside_connections_highway_at_edge_detected_via_system() {
    let mut city = TestCity::new();
    // Place a highway at the bottom edge using the grid directly
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        let cell = grid.get_mut(100, 0);
        cell.cell_type = CellType::Road;
        cell.road_type = RoadType::Highway;
    }

    // Run enough ticks for the system to detect connections
    city.tick(101);

    let outside = city.resource::<OutsideConnections>();
    assert!(
        outside.has_connection(ConnectionType::Highway),
        "Should detect highway at map edge"
    );
    assert_eq!(outside.count(ConnectionType::Highway), 1);
}

#[test]
fn test_outside_connections_airport_detected_via_service() {
    let mut city = TestCity::new().with_service(100, 100, ServiceType::InternationalAirport);

    city.tick(101);

    let outside = city.resource::<OutsideConnections>();
    assert!(
        outside.has_connection(ConnectionType::Airport),
        "Should detect InternationalAirport as airport connection"
    );
}

#[test]
fn test_outside_connections_train_station_at_edge_detected_as_railway() {
    // TrainStation near the edge (x=1 is within EDGE_PROXIMITY=3)
    let mut city = TestCity::new().with_service(1, 128, ServiceType::TrainStation);

    city.tick(101);

    let outside = city.resource::<OutsideConnections>();
    assert!(
        outside.has_connection(ConnectionType::Railway),
        "Should detect TrainStation at edge as railway connection"
    );
}

#[test]
fn test_outside_connections_train_station_interior_not_detected() {
    // TrainStation in the interior (x=128 is NOT near edge)
    let mut city = TestCity::new().with_service(128, 128, ServiceType::TrainStation);

    city.tick(101);

    let outside = city.resource::<OutsideConnections>();
    assert!(
        !outside.has_connection(ConnectionType::Railway),
        "TrainStation in interior should NOT be a railway connection"
    );
}

#[test]
fn test_outside_connections_effects_boost_tourism() {
    use crate::tourism::Tourism;

    let mut city = TestCity::new().with_service(100, 100, ServiceType::InternationalAirport);

    // Record initial tourism
    let initial_attractiveness = city.resource::<Tourism>().attractiveness;

    // Run system
    city.tick(101);

    let tourism = city.resource::<Tourism>();
    assert!(
        tourism.attractiveness > initial_attractiveness,
        "Airport connection should boost tourism attractiveness (was {}, now {})",
        initial_attractiveness,
        tourism.attractiveness
    );
}

#[test]
fn test_outside_connections_multiple_types_coexist() {
    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::InternationalAirport)
        .with_service(1, 128, ServiceType::TrainStation);

    // Place a highway at the bottom edge
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        let cell = grid.get_mut(100, 0);
        cell.cell_type = CellType::Road;
        cell.road_type = RoadType::Highway;
    }

    city.tick(101);

    let outside = city.resource::<OutsideConnections>();
    assert!(outside.has_connection(ConnectionType::Highway));
    assert!(outside.has_connection(ConnectionType::Railway));
    assert!(outside.has_connection(ConnectionType::Airport));
    // SeaPort requires FerryPier near water edge, which we didn't set up
    assert!(!outside.has_connection(ConnectionType::SeaPort));
}
