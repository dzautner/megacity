//! Roundabout construction logic.
//!
//! Provides functions to compute ring cells and create roundabouts with
//! circular Bezier curve road segments.

use bevy::prelude::*;
use std::f32::consts::PI;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;

use super::{CirculationDirection, Roundabout, RoundaboutTrafficRule};

/// Minimum allowed roundabout radius (grid cells).
pub const MIN_RADIUS: usize = 2;
/// Maximum allowed roundabout radius (grid cells).
pub const MAX_RADIUS: usize = 5;

/// Number of Bezier arc segments used to approximate the circle.
pub(crate) const ARC_SEGMENT_COUNT: usize = 8;

/// Compute the ring cells for a roundabout by rasterizing a circle on the grid.
///
/// Returns grid cells that lie on the circle perimeter.
pub(crate) fn compute_ring_cells(
    center_x: usize,
    center_y: usize,
    radius: usize,
) -> Vec<(usize, usize)> {
    let mut cells = Vec::new();
    let cx = center_x as f32;
    let cy = center_y as f32;
    let r = radius as f32;

    // Sample points around the circle at fine granularity to get all ring cells.
    let sample_count = (2.0 * PI * r * 2.0).ceil() as usize;
    for i in 0..sample_count {
        let angle = 2.0 * PI * (i as f32) / (sample_count as f32);
        let gx = (cx + r * angle.cos()).round() as i32;
        let gy = (cy + r * angle.sin()).round() as i32;

        if gx < 0 || gy < 0 || gx >= GRID_WIDTH as i32 || gy >= GRID_HEIGHT as i32 {
            continue;
        }
        let cell = (gx as usize, gy as usize);
        if !cells.contains(&cell) {
            cells.push(cell);
        }
    }

    cells
}

/// Create a roundabout at the given center position with the specified radius.
///
/// This function:
/// 1. Computes the ring cells
/// 2. Creates Bezier curve road segments forming a circle
/// 3. Detects existing approach roads at the perimeter
/// 4. Returns the `Roundabout` definition
///
/// The caller is responsible for adding it to `RoundaboutRegistry`.
pub fn create_roundabout(
    center: (usize, usize),
    radius: usize,
    road_type: RoadType,
    direction: CirculationDirection,
    segments: &mut RoadSegmentStore,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
) -> Roundabout {
    let radius = radius.clamp(MIN_RADIUS, MAX_RADIUS);
    let (cx, cy) = center;

    // Compute ring cells for tracking
    let ring_cells = compute_ring_cells(cx, cy, radius);

    // Generate circular road segments using Bezier curves.
    // We split the circle into ARC_SEGMENT_COUNT arcs, each approximated by a
    // cubic Bezier curve using the standard circular arc approximation.
    let center_world = WorldGrid::grid_to_world(cx, cy);
    let center_vec = Vec2::new(center_world.0, center_world.1);
    let r_world = radius as f32 * crate::config::CELL_SIZE;

    let mut segment_ids: Vec<u32> = Vec::new();

    // Generate arc endpoint angles.
    let angle_step = 2.0 * PI / ARC_SEGMENT_COUNT as f32;

    // Pre-create nodes at each arc endpoint
    let mut arc_nodes = Vec::with_capacity(ARC_SEGMENT_COUNT);
    for i in 0..ARC_SEGMENT_COUNT {
        let angle = match direction {
            CirculationDirection::Clockwise => -(i as f32) * angle_step,
            CirculationDirection::Counterclockwise => (i as f32) * angle_step,
        };
        let pos = center_vec + Vec2::new(r_world * angle.cos(), r_world * angle.sin());
        let node_id = segments.find_or_create_node(pos, crate::config::CELL_SIZE * 0.5);
        arc_nodes.push((node_id, pos, angle));
    }

    // Create Bezier segments connecting consecutive arc points.
    // The "magic number" for approximating a circular arc with a cubic Bezier is:
    //   k = (4/3) * tan(theta/4) where theta is the arc angle.
    let theta = angle_step;
    let k = (4.0 / 3.0) * (theta / 4.0).tan();

    for i in 0..ARC_SEGMENT_COUNT {
        let j = (i + 1) % ARC_SEGMENT_COUNT;

        let (start_node, p0, angle0) = arc_nodes[i];
        let (end_node, p3, angle1) = arc_nodes[j];

        // Control points: perpendicular to the radius at each endpoint.
        // For a circular arc, the tangent direction is perpendicular to the radius.
        let tangent0 = match direction {
            CirculationDirection::Clockwise => Vec2::new(angle0.sin(), -angle0.cos()),
            CirculationDirection::Counterclockwise => Vec2::new(-angle0.sin(), angle0.cos()),
        };

        let tangent1 = match direction {
            CirculationDirection::Clockwise => Vec2::new(angle1.sin(), -angle1.cos()),
            CirculationDirection::Counterclockwise => Vec2::new(-angle1.sin(), angle1.cos()),
        };

        let p1 = p0 + tangent0 * r_world * k;
        let p2 = p3 - tangent1 * r_world * k;

        let seg_id =
            segments.add_segment(start_node, end_node, p0, p1, p2, p3, road_type, grid, roads);
        segment_ids.push(seg_id.0);
    }

    // Detect approach roads: existing road cells adjacent to ring cells
    let mut approach_connections = Vec::new();
    for &(rx, ry) in &ring_cells {
        // Check 4 cardinal neighbors
        for &(dx, dy) in &[(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
            let nx = rx as i32 + dx;
            let ny = ry as i32 + dy;
            if nx < 0 || ny < 0 || nx >= GRID_WIDTH as i32 || ny >= GRID_HEIGHT as i32 {
                continue;
            }
            let (nx, ny) = (nx as usize, ny as usize);
            // If neighbor is a road but not part of the ring, it's an approach road
            if grid.get(nx, ny).cell_type == CellType::Road
                && !ring_cells.contains(&(nx, ny))
                && !approach_connections.contains(&(nx, ny))
            {
                approach_connections.push((nx, ny));
            }
        }
    }

    Roundabout {
        center_x: cx,
        center_y: cy,
        radius,
        road_type,
        direction,
        traffic_rule: RoundaboutTrafficRule::YieldOnEntry,
        ring_cells,
        segment_ids,
        approach_connections,
    }
}
