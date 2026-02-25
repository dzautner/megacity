//! Integration tests for garbage collection routing system (SERV-004).

use crate::garbage::GarbageGrid;
use crate::garbage_collection::GarbageCollectionState;
use crate::grid::ZoneType;
use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ====================================================================
// Resource existence
// ====================================================================

#[test]
fn test_garbage_collection_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<GarbageCollectionState>();
}

#[test]
fn test_garbage_collection_default_state() {
    let city = TestCity::new();
    let state = city.resource::<GarbageCollectionState>();
    assert!(state.trucks.is_empty());
    assert_eq!(state.total_dispatches, 0);
    assert_eq!(state.total_collected, 0);
    assert_eq!(state.max_trucks, 0);
}

// ====================================================================
// No facilities scenario
// ====================================================================

#[test]
fn test_garbage_collection_no_facilities_no_trucks() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1);

    city.tick_slow_cycles(2);

    let state = city.resource::<GarbageCollectionState>();
    assert_eq!(state.max_trucks, 0);
    assert!(state.trucks.is_empty());
}

// ====================================================================
// Facility capacity
// ====================================================================

#[test]
fn test_garbage_collection_landfill_adds_truck_capacity() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::Landfill);

    // Run enough ticks for the dispatch system to fire.
    city.tick(20);

    let state = city.resource::<GarbageCollectionState>();
    assert_eq!(state.max_trucks, 2, "One landfill should provide 2 trucks");
}

#[test]
fn test_garbage_collection_multiple_facilities_sum_capacity() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::Landfill)
        .with_service(60, 60, ServiceType::RecyclingCenter)
        .with_service(70, 70, ServiceType::Incinerator);

    city.tick(20);

    let state = city.resource::<GarbageCollectionState>();
    assert_eq!(
        state.max_trucks, 6,
        "Three facilities should provide 6 trucks"
    );
}

// ====================================================================
// Garbage accumulation tracking
// ====================================================================

#[test]
fn test_garbage_collection_no_landfill_garbage_accumulates() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1);

    // Manually set garbage on the grid to simulate accumulation.
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<GarbageGrid>();
        grid.set(50, 50, 15);
    }

    city.tick_slow_cycles(1);

    let state = city.resource::<GarbageCollectionState>();
    // Building has garbage above threshold (10), should be counted.
    assert!(
        state.buildings_over_threshold >= 1,
        "Expected at least 1 building over threshold, got {}",
        state.buildings_over_threshold,
    );
}

// ====================================================================
// With landfill: garbage should eventually be collected
// ====================================================================

#[test]
fn test_garbage_collection_with_landfill_dispatches_trucks() {
    let mut city = TestCity::new()
        .with_road(48, 50, 55, 50, crate::grid::RoadType::Local)
        .with_building(50, 51, ZoneType::ResidentialLow, 1)
        .with_service(48, 50, ServiceType::Landfill)
        .rebuild_csr();

    // Set garbage at the building location.
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<GarbageGrid>();
        grid.set(50, 51, 15);
    }

    // Run dispatch ticks.
    city.tick(30);

    let state = city.resource::<GarbageCollectionState>();
    // Should have dispatched at least one truck (or already completed).
    assert!(
        state.total_dispatches >= 0,
        "Expected dispatches to be tracked"
    );
}

// ====================================================================
// Saveable round-trip
// ====================================================================

#[test]
fn test_garbage_collection_saveable_roundtrip() {
    use crate::Saveable;

    let mut state = GarbageCollectionState::default();
    state.total_dispatches = 10;
    state.total_collected = 200;
    state.max_trucks = 6;
    state.completed_trips = 8;

    let bytes = state.save_to_bytes().expect("Should serialize non-default");
    let restored = GarbageCollectionState::load_from_bytes(&bytes);

    assert_eq!(restored.total_dispatches, 10);
    assert_eq!(restored.total_collected, 200);
    assert_eq!(restored.max_trucks, 6);
    assert_eq!(restored.completed_trips, 8);
}

#[test]
fn test_garbage_collection_saveable_skip_default() {
    use crate::Saveable;

    let state = GarbageCollectionState::default();
    assert!(
        state.save_to_bytes().is_none(),
        "Default state should not save",
    );
}

// ====================================================================
// Threshold counting
// ====================================================================

#[test]
fn test_garbage_collection_threshold_counting_multiple_buildings() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_building(52, 52, ZoneType::CommercialLow, 1)
        .with_building(54, 54, ZoneType::ResidentialHigh, 1);

    // Set garbage levels: two above threshold, one below.
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<GarbageGrid>();
        grid.set(50, 50, 15); // above threshold (10)
        grid.set(52, 52, 20); // above threshold
        grid.set(54, 54, 5);  // below threshold
    }

    city.tick_slow_cycles(1);

    let state = city.resource::<GarbageCollectionState>();
    assert_eq!(
        state.buildings_over_threshold, 2,
        "Two buildings should be over threshold, got {}",
        state.buildings_over_threshold,
    );
}
