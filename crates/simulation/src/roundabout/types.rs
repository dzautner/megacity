//! Core data structures for roundabouts.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::grid::RoadType;

/// Direction of traffic flow around a roundabout.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CirculationDirection {
    /// Traffic flows clockwise (right-hand traffic convention).
    #[default]
    Clockwise,
    /// Traffic flows counterclockwise (left-hand traffic convention).
    Counterclockwise,
}

/// Traffic rule applied at roundabout entry points.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoundaboutTrafficRule {
    /// Vehicles entering must yield to vehicles already on the roundabout.
    #[default]
    YieldOnEntry,
    /// Vehicles on the roundabout have absolute priority.
    PriorityOnRoundabout,
}

/// A single roundabout in the city.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roundabout {
    /// Grid X coordinate of the roundabout center.
    pub center_x: usize,
    /// Grid Y coordinate of the roundabout center.
    pub center_y: usize,
    /// Radius in grid cells (2-5).
    pub radius: usize,
    /// Road type used for the roundabout circle.
    pub road_type: RoadType,
    /// Direction of traffic flow.
    pub direction: CirculationDirection,
    /// Traffic rule at entry points.
    pub traffic_rule: RoundaboutTrafficRule,
    /// Grid cells that are part of the roundabout ring.
    pub ring_cells: Vec<(usize, usize)>,
    /// Segment IDs of the circular road segments.
    pub segment_ids: Vec<u32>,
    /// Grid coordinates of approach road connection points.
    pub approach_connections: Vec<(usize, usize)>,
}

/// Throughput statistics for a single roundabout.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoundaboutStats {
    /// Sum of traffic density across all ring cells (snapshot).
    pub current_throughput: u32,
    /// Rolling average throughput over recent ticks.
    pub average_throughput: f32,
    /// Number of samples in the rolling average.
    pub sample_count: u32,
}

/// Registry of all roundabouts in the city.
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
pub struct RoundaboutRegistry {
    /// All roundabouts, indexed by position.
    pub roundabouts: Vec<Roundabout>,
    /// Per-roundabout throughput statistics.
    pub stats: Vec<RoundaboutStats>,
}

impl RoundaboutRegistry {
    /// Find a roundabout whose ring contains the given grid cell.
    pub fn find_at_cell(&self, x: usize, y: usize) -> Option<usize> {
        self.roundabouts
            .iter()
            .position(|r| r.ring_cells.contains(&(x, y)))
    }

    /// Find a roundabout by center position.
    pub fn find_by_center(&self, cx: usize, cy: usize) -> Option<usize> {
        self.roundabouts
            .iter()
            .position(|r| r.center_x == cx && r.center_y == cy)
    }

    /// Check if a grid cell is inside any roundabout (within its radius).
    pub fn is_inside_roundabout(&self, x: usize, y: usize) -> bool {
        self.roundabouts.iter().any(|r| {
            let dx = x as f32 - r.center_x as f32;
            let dy = y as f32 - r.center_y as f32;
            (dx * dx + dy * dy).sqrt() <= r.radius as f32 + 0.5
        })
    }
}
