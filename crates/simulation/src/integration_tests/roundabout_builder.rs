use crate::grid::{RoadType, WorldGrid};
use crate::roads::RoadNetwork;

// ====================================================================
// TRAF-011: Roundabout builder
// ====================================================================

#[test]
fn test_roundabout_creation_produces_valid_struct() {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::roundabout::{create_roundabout, CirculationDirection};

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut segments = crate::road_segments::RoadSegmentStore::default();

    let rb = create_roundabout(
        (128, 128),
        3,
        RoadType::Avenue,
        CirculationDirection::Clockwise,
        &mut segments,
        &mut grid,
        &mut roads,
    );

    assert!(rb.ring_cells.len() > 4, "ring should have multiple cells");
    assert_eq!(rb.center_x, 128);
    assert_eq!(rb.center_y, 128);
    assert_eq!(rb.radius, 3);
    assert!(
        !rb.segment_ids.is_empty(),
        "roundabout should have Bezier segments"
    );
}

#[test]
fn test_roundabout_registry_find_at_cell() {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::roundabout::{
        create_roundabout, CirculationDirection, RoundaboutRegistry, RoundaboutStats,
    };

    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut roads = RoadNetwork::default();
    let mut segments = crate::road_segments::RoadSegmentStore::default();

    let rb = create_roundabout(
        (100, 100),
        2,
        RoadType::Local,
        CirculationDirection::Clockwise,
        &mut segments,
        &mut grid,
        &mut roads,
    );

    let first_ring_cell = rb.ring_cells[0];
    let mut registry = RoundaboutRegistry::default();
    registry.stats.push(RoundaboutStats::default());
    registry.roundabouts.push(rb);

    assert!(
        registry
            .find_at_cell(first_ring_cell.0, first_ring_cell.1)
            .is_some(),
        "should find roundabout at ring cell"
    );
    assert!(
        registry.find_at_cell(0, 0).is_none(),
        "should not find roundabout at (0,0)"
    );
}

#[test]
fn test_roundabout_saveable_roundtrip() {
    use crate::roundabout::{
        CirculationDirection, Roundabout, RoundaboutRegistry, RoundaboutTrafficRule,
    };
    use crate::Saveable;

    let mut registry = RoundaboutRegistry::default();
    registry.roundabouts.push(Roundabout {
        center_x: 50,
        center_y: 60,
        radius: 3,
        road_type: RoadType::Avenue,
        direction: CirculationDirection::Clockwise,
        traffic_rule: RoundaboutTrafficRule::YieldOnEntry,
        ring_cells: vec![(49, 60), (50, 61), (51, 60), (50, 59)],
        segment_ids: vec![10, 11, 12, 13],
        approach_connections: vec![(48, 60)],
    });

    let bytes = registry.save_to_bytes().expect("should serialize");
    let restored = RoundaboutRegistry::load_from_bytes(&bytes);
    assert_eq!(restored.roundabouts.len(), 1);
    assert_eq!(restored.roundabouts[0].center_x, 50);
    assert_eq!(restored.roundabouts[0].ring_cells.len(), 4);
}
