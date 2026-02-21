//! Intersection Auto-Detection Preview (UX-023)
//!
//! When drawing roads in freeform mode, this system detects where the preview
//! road crosses existing road segments and renders colored markers at each
//! intersection point:
//!
//! - **Green diamond**: A valid new intersection that will create a new node
//!   in the road network when the road is placed.
//! - **Yellow diamond**: The intersection is close to an existing node and will
//!   snap to it rather than creating a new one.
//!
//! This gives players visual feedback about how their road will connect to
//! the existing network before they commit to placing it.

use bevy::prelude::*;

use simulation::road_segments::RoadSegmentStore;

use crate::input::{ActiveTool, CursorGridPos, DrawPhase, IntersectionSnap, RoadDrawState};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Snap distance matching `RoadSegmentStore::find_or_create_node` default.
const NODE_SNAP_DIST: f32 = 24.0;

/// Height above ground for gizmo rendering (slightly above the road preview).
const GIZMO_Y: f32 = 0.6;

/// Minimum distance between two detected intersection markers to avoid clutter.
const DEDUP_RADIUS: f32 = 8.0;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Classification of a detected intersection point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntersectionKind {
    /// Will create a new node (no existing node nearby).
    NewNode,
    /// Close to an existing node; will snap to it.
    SnapToExisting,
}

/// A single detected intersection between the preview road and an existing segment.
#[derive(Debug, Clone)]
pub struct DetectedIntersection {
    /// World-space 2D position of the intersection.
    pub position: Vec2,
    /// Whether this is a new node or a snap to existing.
    pub kind: IntersectionKind,
}

/// Resource holding the intersection preview results for the current frame.
#[derive(Resource, Default)]
pub struct IntersectionPreviewState {
    /// Detected intersections for the current preview road.
    pub intersections: Vec<DetectedIntersection>,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct IntersectionPreviewPlugin;

impl Plugin for IntersectionPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IntersectionPreviewState>().add_systems(
            Update,
            (
                compute_intersection_preview
                    .after(crate::input::update_cursor_grid_pos)
                    .after(crate::angle_snap::update_angle_snap)
                    .after(crate::input::update_intersection_snap),
                draw_intersection_preview_markers.after(compute_intersection_preview),
            ),
        );
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Evaluate cubic Bezier at parameter t given four control points (in 2D).
fn bezier_eval(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    let uu = u * u;
    let tt = t * t;
    u * uu * p0 + 3.0 * uu * t * p1 + 3.0 * u * tt * p2 + t * tt * p3
}

/// 2D line-segment intersection test. Returns the intersection point if
/// the two segments (a1-a2) and (b1-b2) cross each other.
fn segment_intersection_2d(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<Vec2> {
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
fn find_classified_intersections(
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

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Each frame, compute intersection points between the preview road and
/// existing segments, classify them, and store the results.
#[allow(clippy::too_many_arguments)]
fn compute_intersection_preview(
    draw_state: Res<RoadDrawState>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    angle_snap: Res<crate::angle_snap::AngleSnapState>,
    snap: Res<IntersectionSnap>,
    segment_store: Res<RoadSegmentStore>,
    mut state: ResMut<IntersectionPreviewState>,
) {
    state.intersections.clear();

    // Only compute when actively drawing a road
    if draw_state.phase != DrawPhase::PlacedStart || !cursor.valid {
        return;
    }

    // Only for road tools
    if tool.road_type().is_none() {
        return;
    }

    // No segments to intersect with
    if segment_store.segments.is_empty() {
        return;
    }

    let start = draw_state.start_pos;
    // Use the same end-point logic as draw_bezier_preview
    let end = if let Some(snapped) = snap.snapped_pos {
        snapped
    } else if angle_snap.active {
        angle_snap.snapped_pos
    } else {
        cursor.world_pos
    };

    // Skip if road is too short
    if (end - start).length() < 1.0 {
        return;
    }

    // Build Bezier control points (straight line, same as draw_bezier_preview)
    let p0 = start;
    let p3 = end;
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    state.intersections = find_classified_intersections(p0, p1, p2, p3, &segment_store);
}

/// Draw diamond-shaped markers at each detected intersection point.
/// Green = new node, Yellow = snap to existing node.
fn draw_intersection_preview_markers(
    state: Res<IntersectionPreviewState>,
    draw_state: Res<RoadDrawState>,
    mut gizmos: Gizmos,
) {
    if draw_state.phase != DrawPhase::PlacedStart {
        return;
    }

    for intersection in &state.intersections {
        let pt = intersection.position;
        let pos = Vec3::new(pt.x, GIZMO_Y + 0.1, pt.y);

        let (fill_color, outline_color) = match intersection.kind {
            IntersectionKind::NewNode => (
                Color::srgba(0.1, 0.9, 0.2, 0.85), // Green fill
                Color::srgba(0.2, 1.0, 0.3, 1.0),  // Bright green outline
            ),
            IntersectionKind::SnapToExisting => (
                Color::srgba(1.0, 0.85, 0.1, 0.85), // Yellow fill
                Color::srgba(1.0, 0.95, 0.3, 1.0),  // Bright yellow outline
            ),
        };

        let diamond_size = 6.0;

        // Diamond shape (rotated square)
        let top = pos + Vec3::new(0.0, 0.0, -diamond_size);
        let right = pos + Vec3::new(diamond_size, 0.0, 0.0);
        let bottom = pos + Vec3::new(0.0, 0.0, diamond_size);
        let left = pos + Vec3::new(-diamond_size, 0.0, 0.0);

        // Outline
        gizmos.line(top, right, outline_color);
        gizmos.line(right, bottom, outline_color);
        gizmos.line(bottom, left, outline_color);
        gizmos.line(left, top, outline_color);

        // Inner diamond (smaller, simulates fill)
        let inner_size = diamond_size * 0.5;
        let i_top = pos + Vec3::new(0.0, 0.0, -inner_size);
        let i_right = pos + Vec3::new(inner_size, 0.0, 0.0);
        let i_bottom = pos + Vec3::new(0.0, 0.0, inner_size);
        let i_left = pos + Vec3::new(-inner_size, 0.0, 0.0);

        gizmos.line(i_top, i_right, fill_color);
        gizmos.line(i_right, i_bottom, fill_color);
        gizmos.line(i_bottom, i_left, fill_color);
        gizmos.line(i_left, i_top, fill_color);

        // Cross lines inside for extra visibility
        gizmos.line(top, bottom, fill_color);
        gizmos.line(left, right, fill_color);

        // Circle around diamond for emphasis
        gizmos.circle(
            Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            diamond_size * 1.3,
            outline_color,
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
        use simulation::grid::RoadType;
        use simulation::road_segments::{RoadSegment, SegmentId, SegmentNode, SegmentNodeId};

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
        use simulation::grid::RoadType;
        use simulation::road_segments::{RoadSegment, SegmentId, SegmentNode, SegmentNodeId};

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
        use simulation::grid::RoadType;
        use simulation::road_segments::{RoadSegment, SegmentId, SegmentNode, SegmentNodeId};

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
}
