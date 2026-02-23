//! Types and constants for the undo/redo system.

use bevy::prelude::*;

use crate::grid::RoadType;
use crate::grid::ZoneType;
use crate::road_segments::{SegmentId, SegmentNodeId};
use crate::services::ServiceType;
use crate::utilities::UtilityType;

/// Maximum number of actions kept in the undo stack.
pub const MAX_HISTORY: usize = 100;

// ---------------------------------------------------------------------------
// CityAction enum — each variant stores enough data to reverse the action
// ---------------------------------------------------------------------------

/// A single undoable/redoable player action.
#[derive(Debug, Clone, Event)]
pub enum CityAction {
    /// A road segment was placed via the freeform drawing tool.
    PlaceRoadSegment {
        segment_id: SegmentId,
        start_node: SegmentNodeId,
        end_node: SegmentNodeId,
        p0: Vec2,
        p1: Vec2,
        p2: Vec2,
        p3: Vec2,
        road_type: RoadType,
        rasterized_cells: Vec<(usize, usize)>,
        cost: f64,
    },
    /// A road cell was placed via the legacy grid-snap tool.
    PlaceGridRoad {
        x: usize,
        y: usize,
        road_type: RoadType,
        cost: f64,
    },
    /// One or more zone cells were painted.
    PlaceZone {
        cells: Vec<(usize, usize, ZoneType)>,
        cost: f64,
    },
    /// A service building was placed.
    PlaceService {
        service_type: ServiceType,
        grid_x: usize,
        grid_y: usize,
        cost: f64,
    },
    /// A utility building was placed.
    PlaceUtility {
        utility_type: UtilityType,
        grid_x: usize,
        grid_y: usize,
        cost: f64,
    },
    /// A road cell was bulldozed.
    BulldozeRoad {
        x: usize,
        y: usize,
        road_type: RoadType,
        refund: f64,
    },
    /// A zone cell was bulldozed (cleared to None).
    BulldozeZone { x: usize, y: usize, zone: ZoneType },
    /// A service building was bulldozed.
    BulldozeService {
        service_type: ServiceType,
        grid_x: usize,
        grid_y: usize,
        refund: f64,
    },
    /// A utility building was bulldozed.
    BulldozeUtility {
        utility_type: UtilityType,
        grid_x: usize,
        grid_y: usize,
        refund: f64,
    },
    /// Multiple actions grouped as one (e.g., a drag operation).
    Composite(Vec<CityAction>),
}
