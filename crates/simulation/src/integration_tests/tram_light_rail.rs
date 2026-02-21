use crate::grid::{RoadType, WorldGrid};
use crate::roads::RoadNetwork;

// ====================================================================
// Tram / light rail transit tests (issue #865)
// ====================================================================

#[test]
fn test_tram_transit_add_stops_and_line() {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::tram_transit::TramTransitState;

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    for x in 95..125 {
        roads.place_road_typed(&mut grid, x, 100, RoadType::Local);
    }

    let mut state = TramTransitState::default();
    let s0 = state.add_stop(&grid, 100, 100).expect("stop on road");
    let s1 = state.add_stop(&grid, 120, 100).expect("stop on road");
    assert_eq!(state.stops.len(), 2);

    state.add_line("Green".into(), vec![s0, s1]);
    assert_eq!(state.lines.len(), 1);
    assert_eq!(state.lines[0].stop_ids.len(), 2);
}

#[test]
fn test_tram_stop_on_non_road_cell_fails() {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::tram_transit::TramTransitState;

    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut state = TramTransitState::default();
    let result = state.add_stop(&grid, 5, 5);
    assert!(result.is_none(), "should not place tram stop on grass");
    assert_eq!(state.stops.len(), 0);
}

#[test]
fn test_tram_transit_saveable_roundtrip() {
    use crate::tram_transit::TramTransitState;
    use crate::Saveable;

    let mut state = TramTransitState::default();
    state.stops.push(crate::tram_transit::TramStop {
        id: 0,
        grid_x: 10,
        grid_y: 20,
        waiting: 0,
    });
    state.stops.push(crate::tram_transit::TramStop {
        id: 1,
        grid_x: 30,
        grid_y: 40,
        waiting: 0,
    });
    state.next_stop_id = 2;
    state.add_line("Red".into(), vec![0, 1]);

    let bytes = state.save_to_bytes().expect("should serialize");
    let restored = TramTransitState::load_from_bytes(&bytes);
    assert_eq!(restored.stops.len(), 2);
    assert_eq!(restored.lines.len(), 1);
    assert_eq!(restored.stops[0].grid_x, 10);
}

#[test]
fn test_tram_capacity_is_90() {
    use crate::tram_transit::TRAM_CAPACITY;
    assert_eq!(TRAM_CAPACITY, 90, "tram capacity should be 90 passengers");
}
