//! Tests for intersection preview geometry and classification.

use bevy::prelude::*;

use simulation::grid::RoadType;
use simulation::road_segments::{
    RoadSegment, RoadSegmentStore, SegmentId, SegmentNode, SegmentNodeId,
};

use super::geometry::{bezier_eval, find_classified_intersections, segment_intersection_2d};
use super::types::IntersectionKind;

#[test]
fn test_segment_intersection_crossing() {
    // Two perpendicular line segments crossing at (1, 1)
    let a1 = Vec2::new(0.0, 0.0);
    let a2 = Vec2::new(2.0, 2.0);
    let b1 = Vec2::new(0.0, 2.0);
    let b2 = Vec2::new(2.0, 0.0);

    let result = segment_intersection_2d(a1, a2, b1, b2);
    assert!(result.is_some());
    let pt = result.unwrap();
    assert!((pt.x - 1.0).abs() < 0.01);
    assert!((pt.y - 1.0).abs() < 0.01);
}

#[test]
fn test_segment_intersection_parallel() {
    // Two parallel segments should not intersect
    let a1 = Vec2::new(0.0, 0.0);
    let a2 = Vec2::new(2.0, 0.0);
    let b1 = Vec2::new(0.0, 1.0);
    let b2 = Vec2::new(2.0, 1.0);

    let result = segment_intersection_2d(a1, a2, b1, b2);
    assert!(result.is_none());
}

#[test]
fn test_segment_intersection_non_crossing() {
    // Two segments that would intersect if extended, but don't actually cross
    let a1 = Vec2::new(0.0, 0.0);
    let a2 = Vec2::new(1.0, 0.0);
    let b1 = Vec2::new(2.0, -1.0);
    let b2 = Vec2::new(2.0, 1.0);

    let result = segment_intersection_2d(a1, a2, b1, b2);
    assert!(result.is_none());
}

#[test]
fn test_bezier_eval_endpoints() {
    let p0 = Vec2::new(0.0, 0.0);
    let p1 = Vec2::new(100.0, 0.0);
    let p2 = Vec2::new(200.0, 0.0);
    let p3 = Vec2::new(300.0, 0.0);

    let start = bezier_eval(p0, p1, p2, p3, 0.0);
    let end = bezier_eval(p0, p1, p2, p3, 1.0);

    assert!((start - p0).length() < 0.01);
    assert!((end - p3).length() < 0.01);
}

#[test]
fn test_classification_new_node() {
    // Create a store with one horizontal segment and no nodes near crossing point
    let store = RoadSegmentStore::from_parts(
        vec![
            SegmentNode {
                id: SegmentNodeId(0),
                position: Vec2::new(0.0, 100.0),
                connected_segments: vec![SegmentId(0)],
            },
            SegmentNode {
                id: SegmentNodeId(1),
                position: Vec2::new(300.0, 100.0),
                connected_segments: vec![SegmentId(0)],
            },
        ],
        vec![RoadSegment {
            id: SegmentId(0),
            start_node: SegmentNodeId(0),
            end_node: SegmentNodeId(1),
            p0: Vec2::new(0.0, 100.0),
            p1: Vec2::new(100.0, 100.0),
            p2: Vec2::new(200.0, 100.0),
            p3: Vec2::new(300.0, 100.0),
            road_type: RoadType::Local,
            arc_length: 300.0,
            rasterized_cells: Vec::new(),
        }],
    );

    // Preview road goes vertically through the horizontal road at x=150
    let preview_p0 = Vec2::new(150.0, 0.0);
    let preview_p3 = Vec2::new(150.0, 200.0);
    let preview_p1 = preview_p0 + (preview_p3 - preview_p0) / 3.0;
    let preview_p2 = preview_p0 + (preview_p3 - preview_p0) * 2.0 / 3.0;

    let results =
        find_classified_intersections(preview_p0, preview_p1, preview_p2, preview_p3, &store);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].kind, IntersectionKind::NewNode);
    // Intersection should be near (150, 100)
    assert!((results[0].position.x - 150.0).abs() < 5.0);
    assert!((results[0].position.y - 100.0).abs() < 5.0);
}

#[test]
fn test_classification_snap_to_existing() {
    // Create a store with a node at exactly (150, 100) where the crossing happens
    let store = RoadSegmentStore::from_parts(
        vec![
            SegmentNode {
                id: SegmentNodeId(0),
                position: Vec2::new(0.0, 100.0),
                connected_segments: vec![SegmentId(0)],
            },
            SegmentNode {
                id: SegmentNodeId(1),
                position: Vec2::new(150.0, 100.0), // Node at crossing point
                connected_segments: vec![SegmentId(0), SegmentId(1)],
            },
            SegmentNode {
                id: SegmentNodeId(2),
                position: Vec2::new(300.0, 100.0),
                connected_segments: vec![SegmentId(1)],
            },
        ],
        vec![
            RoadSegment {
                id: SegmentId(0),
                start_node: SegmentNodeId(0),
                end_node: SegmentNodeId(1),
                p0: Vec2::new(0.0, 100.0),
                p1: Vec2::new(50.0, 100.0),
                p2: Vec2::new(100.0, 100.0),
                p3: Vec2::new(150.0, 100.0),
                road_type: RoadType::Local,
                arc_length: 150.0,
                rasterized_cells: Vec::new(),
            },
            RoadSegment {
                id: SegmentId(1),
                start_node: SegmentNodeId(1),
                end_node: SegmentNodeId(2),
                p0: Vec2::new(150.0, 100.0),
                p1: Vec2::new(200.0, 100.0),
                p2: Vec2::new(250.0, 100.0),
                p3: Vec2::new(300.0, 100.0),
                road_type: RoadType::Local,
                arc_length: 150.0,
                rasterized_cells: Vec::new(),
            },
        ],
    );

    // Preview road goes vertically through the crossing at x=150
    let preview_p0 = Vec2::new(150.0, 0.0);
    let preview_p3 = Vec2::new(150.0, 200.0);
    let preview_p1 = preview_p0 + (preview_p3 - preview_p0) / 3.0;
    let preview_p2 = preview_p0 + (preview_p3 - preview_p0) * 2.0 / 3.0;

    let results =
        find_classified_intersections(preview_p0, preview_p1, preview_p2, preview_p3, &store);

    // Should detect intersection(s) near (150, 100), classified as SnapToExisting
    assert!(!results.is_empty());
    // At least one should be SnapToExisting since node is at (150, 100)
    let has_snap = results
        .iter()
        .any(|r| r.kind == IntersectionKind::SnapToExisting);
    assert!(has_snap);
}

#[test]
fn test_no_intersections_when_no_crossing() {
    let store = RoadSegmentStore::from_parts(
        vec![
            SegmentNode {
                id: SegmentNodeId(0),
                position: Vec2::new(0.0, 100.0),
                connected_segments: vec![SegmentId(0)],
            },
            SegmentNode {
                id: SegmentNodeId(1),
                position: Vec2::new(300.0, 100.0),
                connected_segments: vec![SegmentId(0)],
            },
        ],
        vec![RoadSegment {
            id: SegmentId(0),
            start_node: SegmentNodeId(0),
            end_node: SegmentNodeId(1),
            p0: Vec2::new(0.0, 100.0),
            p1: Vec2::new(100.0, 100.0),
            p2: Vec2::new(200.0, 100.0),
            p3: Vec2::new(300.0, 100.0),
            road_type: RoadType::Local,
            arc_length: 300.0,
            rasterized_cells: Vec::new(),
        }],
    );

    // Preview road is parallel, above the existing road
    let preview_p0 = Vec2::new(0.0, 200.0);
    let preview_p3 = Vec2::new(300.0, 200.0);
    let preview_p1 = preview_p0 + (preview_p3 - preview_p0) / 3.0;
    let preview_p2 = preview_p0 + (preview_p3 - preview_p0) * 2.0 / 3.0;

    let results =
        find_classified_intersections(preview_p0, preview_p1, preview_p2, preview_p3, &store);

    assert!(results.is_empty());
}
