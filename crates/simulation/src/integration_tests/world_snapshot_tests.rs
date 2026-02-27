//! Integration tests for WorldSnapshot spatial state serialization (#1903).

use crate::grid::{RoadType, ZoneType};
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;
use crate::world_snapshot::build_world_snapshot;
use crate::world_snapshot_format::format_buildings;

#[test]
fn test_empty_city_snapshot_is_empty() {
    let mut city = TestCity::new();
    let snapshot = build_world_snapshot(city.world_mut());

    assert!(snapshot.buildings.is_empty(), "No buildings in empty city");
    assert!(snapshot.services.is_empty(), "No services in empty city");
    assert!(snapshot.utilities.is_empty(), "No utilities in empty city");
    assert!(
        snapshot.road_cells.is_empty(),
        "No road cells in empty city"
    );
    assert!(
        snapshot.zone_regions.is_empty(),
        "No zone regions in empty city"
    );
}

#[test]
fn test_road_cells_captured() {
    let mut city = TestCity::new().with_road(10, 10, 15, 10, RoadType::Local);

    let snapshot = build_world_snapshot(city.world_mut());

    assert!(
        !snapshot.road_cells.is_empty(),
        "Should have road cells after placing roads"
    );

    // All road cells should be Local type
    for rc in &snapshot.road_cells {
        assert_eq!(rc.road_type, RoadType::Local);
    }
}

#[test]
fn test_zone_regions_detected() {
    let mut city = TestCity::new().with_zone_rect(5, 5, 8, 8, ZoneType::ResidentialLow);

    let snapshot = build_world_snapshot(city.world_mut());

    assert!(
        !snapshot.zone_regions.is_empty(),
        "Should detect zone regions"
    );

    // The 4x4 zone should be captured as a single region
    let region = &snapshot.zone_regions[0];
    assert_eq!(region.zone_type, ZoneType::ResidentialLow);
    assert_eq!(region.min, (5, 5));
    assert_eq!(region.max, (8, 8));
}

#[test]
fn test_building_in_snapshot() {
    let mut city =
        TestCity::new().with_building(20, 20, ZoneType::CommercialLow, 2);

    let snapshot = build_world_snapshot(city.world_mut());

    assert_eq!(snapshot.buildings.len(), 1);
    let b = &snapshot.buildings[0];
    assert_eq!(b.pos, (20, 20));
    assert_eq!(b.zone_type, ZoneType::CommercialLow);
    assert_eq!(b.level, 2);
    assert!(b.capacity > 0, "Building should have non-zero capacity");
    assert_eq!(b.occupancy, 0, "New building should have 0 occupants");
}

#[test]
fn test_service_in_snapshot() {
    let mut city = TestCity::new().with_service(30, 30, ServiceType::Hospital);

    let snapshot = build_world_snapshot(city.world_mut());

    assert_eq!(snapshot.services.len(), 1);
    let s = &snapshot.services[0];
    assert_eq!(s.pos, (30, 30));
    assert_eq!(s.service_type, ServiceType::Hospital);
    assert!(s.radius > 0.0, "Hospital should have non-zero radius");
}

#[test]
fn test_utility_in_snapshot() {
    let mut city = TestCity::new().with_utility(40, 40, UtilityType::PowerPlant);

    let snapshot = build_world_snapshot(city.world_mut());

    assert_eq!(snapshot.utilities.len(), 1);
    let u = &snapshot.utilities[0];
    assert_eq!(u.pos, (40, 40));
    assert_eq!(u.utility_type, UtilityType::PowerPlant);
    assert!(u.range > 0, "Power plant should have non-zero range");
}

#[test]
fn test_format_buildings_readable() {
    let mut city =
        TestCity::new().with_building(10, 10, ZoneType::ResidentialHigh, 3);

    let snapshot = build_world_snapshot(city.world_mut());
    let output = format_buildings(&snapshot.buildings);

    assert!(!output.is_empty(), "Format output should not be empty");
    assert!(
        output.contains("Buildings"),
        "Output should contain header"
    );
    assert!(
        output.contains("1 total"),
        "Output should mention count"
    );
}
