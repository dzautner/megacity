//! Geometry helpers for Bezier curve evaluation and line-segment intersection.

use bevy::prelude::*;
use simulation::road_segments::RoadSegmentStore;

use super::types::{DetectedIntersection, IntersectionKind, DEDUP_RADIUS, NODE_SNAP_DIST};

/// Evaluate cubic Bezier at parameter t given four control points (in 2D).
pub(crate) fn bezier_eval(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    let uu = u * u;
    let tt = t * t;
    u * uu * p0 + 3.0 * uu * t * p1 + 3.0 * u * tt * p2 + t * tt * p3
}

/// 2D line-segment intersection test. Returns the intersection point if
/// the two segments (a1-a2) and (b1-b2) cross each other.
pub(crate) fn segment_intersection_2d(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<Vec2> {
    let d1 = a2 - a1;
    let d2 = b2 - b1;
    let cross = d1.x * d2.y - d1.y * d2.x;
    if cross.abs() < 1e-6 {
        return None; // parallel or coincident
    }
    let d = b1 - a1;
    let t = (d.x * d2.y - d.y * d2.x) / cross;
    let u = (d.x * d1.y - d.y * d1.x) / cross;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some(a1 + d1 * t)
    } else {
        None
    }
}

/// Find intersection points between a preview Bezier curve and all existing
/// road segments. Each intersection is classified as `NewNode` or
/// `SnapToExisting` depending on proximity to existing nodes.
pub(crate) fn find_classified_intersections(
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    store: &RoadSegmentStore,
) -> Vec<DetectedIntersection> {
    let mut results: Vec<DetectedIntersection> = Vec::new();
    let preview_samples = 48;

    // Pre-sample the preview curve
    let mut preview_points: Vec<Vec2> = Vec::with_capacity(preview_samples + 1);
    for i in 0..=preview_samples {
        let t = i as f32 / preview_samples as f32;
        preview_points.push(bezier_eval(p0, p1, p2, p3, t));
    }

    for segment in &store.segments {
        let seg_samples = 32;
        let mut seg_points: Vec<Vec2> = Vec::with_capacity(seg_samples + 1);
        for i in 0..=seg_samples {
            let t = i as f32 / seg_samples as f32;
            seg_points.push(segment.evaluate(t));
        }

        // Check for line-segment intersections between consecutive sample pairs
        for i in 0..preview_samples {
            let a1 = preview_points[i];
            let a2 = preview_points[i + 1];
            for j in 0..seg_samples {
                let b1 = seg_points[j];
                let b2 = seg_points[j + 1];
                if let Some(pt) = segment_intersection_2d(a1, a2, b1, b2) {
                    // Deduplicate: skip if too close to an already-detected point
                    let dominated = results
                        .iter()
                        .any(|existing| (existing.position - pt).length() < DEDUP_RADIUS);
                    if dominated {
                        continue;
                    }

                    // Classify: is this point near an existing node?
                    let near_existing_node = store
                        .nodes
                        .iter()
                        .any(|node| (node.position - pt).length() < NODE_SNAP_DIST);

                    let kind = if near_existing_node {
                        IntersectionKind::SnapToExisting
                    } else {
                        IntersectionKind::NewNode
                    };

                    results.push(DetectedIntersection { position: pt, kind });
                }
            }
        }
    }

    results
}
