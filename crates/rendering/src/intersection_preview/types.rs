//! Types and constants for the intersection preview system.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Snap distance matching `RoadSegmentStore::find_or_create_node` default.
pub(crate) const NODE_SNAP_DIST: f32 = 24.0;

/// Height above ground for gizmo rendering (slightly above the road preview).
pub(crate) const GIZMO_Y: f32 = 0.6;

/// Minimum distance between two detected intersection markers to avoid clutter.
pub(crate) const DEDUP_RADIUS: f32 = 8.0;

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
