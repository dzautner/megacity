//! Tests for the freight traffic system.

use crate::roads::RoadNode;
use crate::Saveable;

use super::constants::TRUCK_EQUIVALENCE_FACTOR;
use super::systems::find_nearest_destination;
use super::types::{FreightTrafficState, FreightTruck};

#[test]
fn test_freight_truck_advance() {
    let truck = FreightTruck {
        route: vec![
            RoadNode(10, 10),
            RoadNode(11, 10),
            RoadNode(12, 10),
            RoadNode(13, 10),
            RoadNode(14, 10),
        ],
        current_index: 0,
        origin: (10, 10),
        destination: (14, 10),
    };

    let mut t = truck;
    assert_eq!(t.current_position(), Some(&RoadNode(10, 10)));
    assert!(!t.is_arrived());

    t.advance(2);
    assert_eq!(t.current_position(), Some(&RoadNode(12, 10)));
    assert!(!t.is_arrived());

    t.advance(3);
    assert!(t.is_arrived());
    assert_eq!(t.current_position(), None);
}

#[test]
fn test_freight_truck_advance_past_end() {
    let mut truck = FreightTruck {
        route: vec![RoadNode(10, 10), RoadNode(11, 10)],
        current_index: 0,
        origin: (10, 10),
        destination: (11, 10),
    };
    truck.advance(100);
    assert!(truck.is_arrived());
    assert_eq!(truck.current_index, 2); // clamped to route length
}

#[test]
fn test_default_freight_state() {
    let state = FreightTrafficState::default();
    assert!(state.trucks.is_empty());
    assert_eq!(state.industrial_demand, 0.0);
    assert_eq!(state.commercial_demand, 0.0);
    assert!((state.satisfaction - 1.0).abs() < f32::EPSILON);
    assert_eq!(state.trips_completed, 0);
    assert_eq!(state.trips_generated, 0);
    assert!(state.heavy_traffic_ban.is_empty());
}

#[test]
fn test_heavy_traffic_ban_toggle() {
    let mut state = FreightTrafficState::default();
    assert!(!state.is_heavy_traffic_banned(0));

    state.toggle_heavy_traffic_ban(0);
    assert!(state.is_heavy_traffic_banned(0));

    state.toggle_heavy_traffic_ban(0);
    assert!(!state.is_heavy_traffic_banned(0));
}

#[test]
fn test_heavy_traffic_ban_per_district() {
    let mut state = FreightTrafficState::default();
    state.toggle_heavy_traffic_ban(1);
    state.toggle_heavy_traffic_ban(3);

    assert!(!state.is_heavy_traffic_banned(0));
    assert!(state.is_heavy_traffic_banned(1));
    assert!(!state.is_heavy_traffic_banned(2));
    assert!(state.is_heavy_traffic_banned(3));
}

#[test]
fn test_find_nearest_destination_basic() {
    let dests = vec![(20, 20), (30, 30), (15, 15)];
    let result = find_nearest_destination(&dests, 10, 10, 60);
    assert_eq!(result, Some((15, 15)));
}

#[test]
fn test_find_nearest_destination_out_of_range() {
    let dests = vec![(200, 200)];
    let result = find_nearest_destination(&dests, 10, 10, 60);
    assert!(result.is_none());
}

#[test]
fn test_find_nearest_destination_empty() {
    let dests: Vec<(usize, usize)> = vec![];
    let result = find_nearest_destination(&dests, 10, 10, 60);
    assert!(result.is_none());
}

#[test]
fn test_truck_equivalence_factor() {
    // Verify the constant is reasonable (between 2.0 and 3.0 as per issue spec)
    assert!(TRUCK_EQUIVALENCE_FACTOR >= 2.0);
    assert!(TRUCK_EQUIVALENCE_FACTOR <= 3.0);
}

#[test]
fn test_saveable_roundtrip() {
    let mut state = FreightTrafficState::default();
    state.trips_completed = 42;
    state.trips_generated = 100;
    state.satisfaction = 0.75;
    state.toggle_heavy_traffic_ban(2);

    let bytes = state
        .save_to_bytes()
        .expect("should save non-default state");
    let loaded = FreightTrafficState::load_from_bytes(&bytes);

    assert_eq!(loaded.trips_completed, 42);
    assert_eq!(loaded.trips_generated, 100);
    assert!((loaded.satisfaction - 0.75).abs() < f32::EPSILON);
    assert!(loaded.is_heavy_traffic_banned(2));
    assert!(!loaded.is_heavy_traffic_banned(0));
}

#[test]
fn test_saveable_skip_default() {
    let state = FreightTrafficState::default();
    assert!(
        state.save_to_bytes().is_none(),
        "default state should skip save"
    );
}
