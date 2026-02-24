//! Integration tests for road intersection mesh invalidation on segment removal.
//!
//! Verifies that `RoadSegmentStore::removed_segment_endpoints` correctly
//! records endpoint node IDs before stripping connectivity, allowing the
//! renderer to dirty the correct intersection meshes. (Issue #1607, #1239)

use crate::grid::RoadType;
use crate::road_segments::RoadSegmentStore;
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// 1. Single segment removal records both endpoint node IDs
// ---------------------------------------------------------------------------

#[test]
fn test_intersection_invalidation_single_segment_records_endpoints() {
    let mut city = TestCity::new().with_road(100, 128, 120, 128, RoadType::Local);
    let segments = city.road_segments();
    let start_node = segments.segments[0].start_node;
    let end_node = segments.segments[0].end_node;

    city.remove_segment_by_index(0);

    let segments = city.road_segments();
    assert_eq!(
        segments.removed_segment_endpoints.len(),
        2,
        "both endpoints should be recorded after removal"
    );
    assert!(segments.removed_segment_endpoints.contains(&start_node));
    assert!(segments.removed_segment_endpoints.contains(&end_node));
}

// ---------------------------------------------------------------------------
// 2. Junction invalidation: removing one arm of a T-junction
// ---------------------------------------------------------------------------

#[test]
fn test_intersection_invalidation_junction_node_recorded_on_arm_removal() {
    let mut city = TestCity::new()
        .with_road(100, 128, 120, 128, RoadType::Local)
        .with_road(120, 128, 140, 128, RoadType::Local)
        .with_road(120, 128, 120, 148, RoadType::Local);

    // Three segments share a node near (120, 128)
    let segments = city.road_segments();
    assert_eq!(segments.segments.len(), 3);

    // Find the shared junction node
    let shared_node = segments.segments[0].end_node;
    let node = segments.get_node(shared_node).unwrap();
    assert!(
        node.connected_segments.len() >= 2,
        "shared node should be a junction before removal"
    );

    // Remove the third segment (the vertical arm)
    city.remove_segment_by_index(2);

    let segments = city.road_segments();
    assert!(
        segments.removed_segment_endpoints.contains(&shared_node),
        "junction node must appear in removed_segment_endpoints"
    );
}

// ---------------------------------------------------------------------------
// 3. Drain clears the list for the next frame
// ---------------------------------------------------------------------------

#[test]
fn test_intersection_invalidation_drain_clears_endpoints() {
    let mut city = TestCity::new().with_road(100, 128, 120, 128, RoadType::Local);
    city.remove_segment_by_index(0);

    let world = city.world_mut();
    world.resource_scope(|_world, mut segments: bevy::prelude::Mut<RoadSegmentStore>| {
        let drained = segments.drain_removed_endpoints();
        assert_eq!(drained.len(), 2);
        assert!(segments.removed_segment_endpoints.is_empty());
    });
}

// ---------------------------------------------------------------------------
// 4. Multiple removals accumulate before drain
// ---------------------------------------------------------------------------

#[test]
fn test_intersection_invalidation_multiple_removals_accumulate() {
    let mut city = TestCity::new()
        .with_road(80, 128, 100, 128, RoadType::Local)
        .with_road(120, 128, 140, 128, RoadType::Local);

    // Remove both segments without draining in between
    city.remove_segment_by_index(1);
    city.remove_segment_by_index(0);

    let segments = city.road_segments();
    assert_eq!(
        segments.removed_segment_endpoints.len(),
        4,
        "two removals should produce 4 endpoint entries (2 per segment)"
    );
}

// ---------------------------------------------------------------------------
// 5. Node connectivity is stripped after recording endpoints
// ---------------------------------------------------------------------------

#[test]
fn test_intersection_invalidation_connectivity_stripped_after_recording() {
    let mut city = TestCity::new()
        .with_road(100, 128, 120, 128, RoadType::Local)
        .with_road(120, 128, 140, 128, RoadType::Local);

    let segments = city.road_segments();
    let shared_node = segments.segments[0].end_node;
    assert_eq!(
        segments.get_node(shared_node).unwrap().connected_segments.len(),
        2,
        "shared node should have 2 connections before removal"
    );

    // Remove one segment
    city.remove_segment_by_index(0);

    let segments = city.road_segments();
    // Endpoints were recorded
    assert!(segments.removed_segment_endpoints.contains(&shared_node));
    // But connectivity was also stripped — only 1 segment remains on the node
    assert_eq!(
        segments.get_node(shared_node).unwrap().connected_segments.len(),
        1,
        "shared node should have 1 connection after removal"
    );
}
