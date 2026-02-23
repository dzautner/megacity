//! Unit tests for roundabout types, builder, and save/load.

use super::*;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{RoadType, WorldGrid};
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;
use crate::Saveable;

use super::builder::{compute_ring_cells, ARC_SEGMENT_COUNT};
use super::save::{road_type_from_u8, road_type_to_u8};

#[test]
fn test_compute_ring_cells_radius_3() {
    let cells = compute_ring_cells(128, 128, 3);
    assert!(!cells.is_empty(), "ring should have cells");
    // All cells should be approximately radius distance from center
    for &(x, y) in &cells {
        let dx = x as f32 - 128.0;
        let dy = y as f32 - 128.0;
        let dist = (dx * dx + dy * dy).sqrt();
        assert!(
            dist >= 2.0 && dist <= 4.0,
            "cell ({}, {}) at distance {} is out of range",
            x,
            y,
            dist,
        );
    }
}

#[test]
fn test_compute_ring_cells_no_duplicates() {
    let cells = compute_ring_cells(128, 128, 4);
    let mut sorted = cells.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        cells.len(),
        sorted.len(),
        "ring cells should have no duplicates"
    );
}

#[test]
fn test_radius_clamping() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut store = RoadSegmentStore::default();

    // Radius 1 should be clamped to MIN_RADIUS (2)
    let rb = create_roundabout(
        (128, 128),
        1,
        RoadType::Local,
        CirculationDirection::Clockwise,
        &mut store,
        &mut grid,
        &mut roads,
    );
    assert_eq!(rb.radius, MIN_RADIUS);

    // Radius 10 should be clamped to MAX_RADIUS (5)
    let rb2 = create_roundabout(
        (200, 200),
        10,
        RoadType::Local,
        CirculationDirection::Clockwise,
        &mut store,
        &mut grid,
        &mut roads,
    );
    assert_eq!(rb2.radius, MAX_RADIUS);
}

#[test]
fn test_create_roundabout_generates_segments() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut store = RoadSegmentStore::default();

    let rb = create_roundabout(
        (128, 128),
        3,
        RoadType::Local,
        CirculationDirection::Clockwise,
        &mut store,
        &mut grid,
        &mut roads,
    );

    assert_eq!(
        rb.segment_ids.len(),
        ARC_SEGMENT_COUNT,
        "should create {} arc segments",
        ARC_SEGMENT_COUNT,
    );
    assert_eq!(store.segments.len(), ARC_SEGMENT_COUNT);
    assert!(!rb.ring_cells.is_empty());
}

#[test]
fn test_roundabout_road_type() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut store = RoadSegmentStore::default();

    let rb = create_roundabout(
        (128, 128),
        3,
        RoadType::Avenue,
        CirculationDirection::Clockwise,
        &mut store,
        &mut grid,
        &mut roads,
    );

    assert_eq!(rb.road_type, RoadType::Avenue);
    // All generated segments should use the specified road type
    for seg in &store.segments {
        assert_eq!(seg.road_type, RoadType::Avenue);
    }
}

#[test]
fn test_roundabout_creates_road_cells() {
    use crate::grid::CellType;

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut store = RoadSegmentStore::default();

    let _rb = create_roundabout(
        (128, 128),
        3,
        RoadType::Local,
        CirculationDirection::Clockwise,
        &mut store,
        &mut grid,
        &mut roads,
    );

    // Some grid cells should now be roads
    let road_count = grid
        .cells
        .iter()
        .filter(|c| c.cell_type == CellType::Road)
        .count();
    assert!(road_count > 0, "roundabout should create road cells");
}

#[test]
fn test_roundabout_direction() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut store = RoadSegmentStore::default();

    let rb_cw = create_roundabout(
        (128, 128),
        3,
        RoadType::Local,
        CirculationDirection::Clockwise,
        &mut store,
        &mut grid,
        &mut roads,
    );
    assert_eq!(rb_cw.direction, CirculationDirection::Clockwise);

    let rb_ccw = create_roundabout(
        (200, 200),
        3,
        RoadType::Local,
        CirculationDirection::Counterclockwise,
        &mut store,
        &mut grid,
        &mut roads,
    );
    assert_eq!(rb_ccw.direction, CirculationDirection::Counterclockwise);
}

#[test]
fn test_roundabout_detects_approach_roads() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut store = RoadSegmentStore::default();

    // Place an approach road leading to where the roundabout ring will be.
    // For radius 3 centered at (128, 128), the ring passes through cells
    // at distance ~3 from center. Place a road leading from outside.
    let ring_cells = compute_ring_cells(128, 128, 3);
    if let Some(&(rx, ry)) = ring_cells.first() {
        // Place road cells leading away from the ring
        if rx + 1 < GRID_WIDTH && !ring_cells.contains(&(rx + 1, ry)) {
            roads.place_road_typed(&mut grid, rx + 1, ry, RoadType::Local);
            roads.place_road_typed(&mut grid, rx + 2, ry, RoadType::Local);
        }
    }

    let rb = create_roundabout(
        (128, 128),
        3,
        RoadType::Local,
        CirculationDirection::Clockwise,
        &mut store,
        &mut grid,
        &mut roads,
    );

    assert!(
        !rb.approach_connections.is_empty(),
        "should detect approach road connections"
    );
}

#[test]
fn test_registry_find_at_cell() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut store = RoadSegmentStore::default();

    let rb = create_roundabout(
        (128, 128),
        3,
        RoadType::Local,
        CirculationDirection::Clockwise,
        &mut store,
        &mut grid,
        &mut roads,
    );

    let ring_cell = rb.ring_cells[0];
    let mut registry = RoundaboutRegistry::default();
    registry.roundabouts.push(rb);

    assert_eq!(registry.find_at_cell(ring_cell.0, ring_cell.1), Some(0));
    assert_eq!(registry.find_at_cell(0, 0), None);
}

#[test]
fn test_registry_find_by_center() {
    let mut registry = RoundaboutRegistry::default();
    registry.roundabouts.push(Roundabout {
        center_x: 100,
        center_y: 100,
        radius: 3,
        road_type: RoadType::Local,
        direction: CirculationDirection::Clockwise,
        traffic_rule: RoundaboutTrafficRule::YieldOnEntry,
        ring_cells: vec![],
        segment_ids: vec![],
        approach_connections: vec![],
    });

    assert_eq!(registry.find_by_center(100, 100), Some(0));
    assert_eq!(registry.find_by_center(50, 50), None);
}

#[test]
fn test_saveable_roundtrip() {
    let mut registry = RoundaboutRegistry::default();
    registry.roundabouts.push(Roundabout {
        center_x: 100,
        center_y: 100,
        radius: 3,
        road_type: RoadType::Local,
        direction: CirculationDirection::Clockwise,
        traffic_rule: RoundaboutTrafficRule::YieldOnEntry,
        ring_cells: vec![(97, 100), (103, 100), (100, 97), (100, 103)],
        segment_ids: vec![1, 2, 3, 4],
        approach_connections: vec![(96, 100)],
    });

    let bytes = registry
        .save_to_bytes()
        .expect("should serialize non-empty registry");
    let loaded = RoundaboutRegistry::load_from_bytes(&bytes);

    assert_eq!(loaded.roundabouts.len(), 1);
    assert_eq!(loaded.roundabouts[0].center_x, 100);
    assert_eq!(loaded.roundabouts[0].center_y, 100);
    assert_eq!(loaded.roundabouts[0].radius, 3);
    assert_eq!(
        loaded.roundabouts[0].direction,
        CirculationDirection::Clockwise
    );
    assert_eq!(loaded.roundabouts[0].ring_cells.len(), 4);
    assert_eq!(loaded.roundabouts[0].segment_ids.len(), 4);
    assert_eq!(loaded.roundabouts[0].approach_connections.len(), 1);
    assert_eq!(loaded.roundabouts[0].road_type, RoadType::Local);
}

#[test]
fn test_saveable_empty_returns_none() {
    let registry = RoundaboutRegistry::default();
    assert!(
        registry.save_to_bytes().is_none(),
        "empty registry should not serialize"
    );
}

#[test]
fn test_traffic_rule_default() {
    let rule = RoundaboutTrafficRule::default();
    assert_eq!(rule, RoundaboutTrafficRule::YieldOnEntry);
}

#[test]
fn test_circulation_direction_default() {
    let dir = CirculationDirection::default();
    assert_eq!(dir, CirculationDirection::Clockwise);
}

#[test]
fn test_road_type_roundtrip_all_variants() {
    for (byte, expected) in [
        (0u8, RoadType::Local),
        (1, RoadType::Avenue),
        (2, RoadType::Boulevard),
        (3, RoadType::Highway),
        (4, RoadType::OneWay),
        (5, RoadType::Path),
    ] {
        assert_eq!(road_type_to_u8(expected), byte);
        assert_eq!(road_type_from_u8(byte), expected);
    }
    // Unknown byte falls back to Local
    assert_eq!(road_type_from_u8(255), RoadType::Local);
}
