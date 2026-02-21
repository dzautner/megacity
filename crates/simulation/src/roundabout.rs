//! Roundabout builder tool (TRAF-011).
//!
//! Provides a `RoundaboutRegistry` resource that tracks all roundabouts in the
//! city, a builder function to create circular one-way roads using Bezier curves,
//! and systems for yield-on-entry traffic rules and throughput tracking.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::road_graph_csr::CsrGraph;
use crate::road_segments::RoadSegmentStore;
use crate::roads::{RoadNetwork, RoadNode};
use crate::traffic::TrafficGrid;
use crate::Saveable;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Saveable implementation (manual byte serialization)
// ---------------------------------------------------------------------------

/// Encode a `RoadType` as a single byte.
fn road_type_to_u8(rt: RoadType) -> u8 {
    match rt {
        RoadType::Local => 0,
        RoadType::Avenue => 1,
        RoadType::Boulevard => 2,
        RoadType::Highway => 3,
        RoadType::OneWay => 4,
        RoadType::Path => 5,
    }
}

/// Decode a `RoadType` from a single byte.
fn road_type_from_u8(b: u8) -> RoadType {
    match b {
        0 => RoadType::Local,
        1 => RoadType::Avenue,
        2 => RoadType::Boulevard,
        3 => RoadType::Highway,
        4 => RoadType::OneWay,
        5 => RoadType::Path,
        _ => RoadType::Local,
    }
}

impl Saveable for RoundaboutRegistry {
    const SAVE_KEY: &'static str = "roundabout_registry";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.roundabouts.is_empty() {
            return None;
        }

        let mut buf = Vec::new();

        // Number of roundabouts (4 bytes)
        let count = self.roundabouts.len() as u32;
        buf.extend_from_slice(&count.to_le_bytes());

        for rb in &self.roundabouts {
            // center_x, center_y, radius (4 bytes each = 12 bytes)
            buf.extend_from_slice(&(rb.center_x as u32).to_le_bytes());
            buf.extend_from_slice(&(rb.center_y as u32).to_le_bytes());
            buf.extend_from_slice(&(rb.radius as u32).to_le_bytes());

            // road_type (1 byte)
            buf.push(road_type_to_u8(rb.road_type));

            // direction (1 byte: 0 = Clockwise, 1 = Counterclockwise)
            buf.push(match rb.direction {
                CirculationDirection::Clockwise => 0,
                CirculationDirection::Counterclockwise => 1,
            });

            // traffic_rule (1 byte: 0 = YieldOnEntry, 1 = PriorityOnRoundabout)
            buf.push(match rb.traffic_rule {
                RoundaboutTrafficRule::YieldOnEntry => 0,
                RoundaboutTrafficRule::PriorityOnRoundabout => 1,
            });

            // ring_cells count + data
            let rc_count = rb.ring_cells.len() as u32;
            buf.extend_from_slice(&rc_count.to_le_bytes());
            for &(x, y) in &rb.ring_cells {
                buf.extend_from_slice(&(x as u16).to_le_bytes());
                buf.extend_from_slice(&(y as u16).to_le_bytes());
            }

            // segment_ids count + data
            let seg_count = rb.segment_ids.len() as u32;
            buf.extend_from_slice(&seg_count.to_le_bytes());
            for &sid in &rb.segment_ids {
                buf.extend_from_slice(&sid.to_le_bytes());
            }

            // approach_connections count + data
            let ac_count = rb.approach_connections.len() as u32;
            buf.extend_from_slice(&ac_count.to_le_bytes());
            for &(x, y) in &rb.approach_connections {
                buf.extend_from_slice(&(x as u16).to_le_bytes());
                buf.extend_from_slice(&(y as u16).to_le_bytes());
            }
        }

        Some(buf)
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let mut pos = 0;

        let read_u32 = |bytes: &[u8], pos: &mut usize| -> u32 {
            if *pos + 4 > bytes.len() {
                return 0;
            }
            let val = u32::from_le_bytes(bytes[*pos..*pos + 4].try_into().unwrap_or([0; 4]));
            *pos += 4;
            val
        };

        let read_u16 = |bytes: &[u8], pos: &mut usize| -> u16 {
            if *pos + 2 > bytes.len() {
                return 0;
            }
            let val = u16::from_le_bytes(bytes[*pos..*pos + 2].try_into().unwrap_or([0; 2]));
            *pos += 2;
            val
        };

        let read_u8 = |bytes: &[u8], pos: &mut usize| -> u8 {
            if *pos >= bytes.len() {
                return 0;
            }
            let val = bytes[*pos];
            *pos += 1;
            val
        };

        let count = read_u32(bytes, &mut pos) as usize;
        let mut roundabouts = Vec::with_capacity(count);

        for _ in 0..count {
            let center_x = read_u32(bytes, &mut pos) as usize;
            let center_y = read_u32(bytes, &mut pos) as usize;
            let radius = read_u32(bytes, &mut pos) as usize;
            let road_type = road_type_from_u8(read_u8(bytes, &mut pos));
            let direction = match read_u8(bytes, &mut pos) {
                0 => CirculationDirection::Clockwise,
                _ => CirculationDirection::Counterclockwise,
            };
            let traffic_rule = match read_u8(bytes, &mut pos) {
                0 => RoundaboutTrafficRule::YieldOnEntry,
                _ => RoundaboutTrafficRule::PriorityOnRoundabout,
            };

            let rc_count = read_u32(bytes, &mut pos) as usize;
            let mut ring_cells = Vec::with_capacity(rc_count);
            for _ in 0..rc_count {
                let x = read_u16(bytes, &mut pos) as usize;
                let y = read_u16(bytes, &mut pos) as usize;
                ring_cells.push((x, y));
            }

            let seg_count = read_u32(bytes, &mut pos) as usize;
            let mut segment_ids = Vec::with_capacity(seg_count);
            for _ in 0..seg_count {
                segment_ids.push(read_u32(bytes, &mut pos));
            }

            let ac_count = read_u32(bytes, &mut pos) as usize;
            let mut approach_connections = Vec::with_capacity(ac_count);
            for _ in 0..ac_count {
                let x = read_u16(bytes, &mut pos) as usize;
                let y = read_u16(bytes, &mut pos) as usize;
                approach_connections.push((x, y));
            }

            roundabouts.push(Roundabout {
                center_x,
                center_y,
                radius,
                road_type,
                direction,
                traffic_rule,
                ring_cells,
                segment_ids,
                approach_connections,
            });
        }

        Self {
            roundabouts,
            stats: Vec::new(), // stats are transient, not saved
        }
    }
}

// ---------------------------------------------------------------------------
// Builder logic
// ---------------------------------------------------------------------------

/// Minimum allowed roundabout radius (grid cells).
pub const MIN_RADIUS: usize = 2;
/// Maximum allowed roundabout radius (grid cells).
pub const MAX_RADIUS: usize = 5;

/// Number of Bezier arc segments used to approximate the circle.
const ARC_SEGMENT_COUNT: usize = 8;

/// Compute the ring cells for a roundabout by rasterizing a circle on the grid.
///
/// Returns grid cells that lie on the circle perimeter.
fn compute_ring_cells(center_x: usize, center_y: usize, radius: usize) -> Vec<(usize, usize)> {
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

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Weight multiplier applied to edges entering a roundabout (yield-on-entry).
/// Higher values discourage entry when the roundabout is busy.
const YIELD_ENTRY_WEIGHT_MULTIPLIER: u32 = 3;

/// Update traffic weights in the CSR graph based on roundabout traffic rules.
///
/// For each roundabout:
/// - Edges from approach roads INTO the roundabout ring get increased weight
///   (yield-on-entry), scaled by current roundabout traffic density.
/// - Edges within the roundabout ring maintain normal weight (priority).
fn update_roundabout_traffic(
    registry: Res<RoundaboutRegistry>,
    mut csr: ResMut<CsrGraph>,
    traffic: Res<TrafficGrid>,
    timer: Res<SlowTickTimer>,
) {
    // Only run every few ticks (matching slow tick)
    if !timer.should_run() {
        return;
    }

    if registry.roundabouts.is_empty() {
        return;
    }

    // Build a set of roundabout ring nodes for fast lookup
    let mut ring_node_set = std::collections::HashSet::new();
    for roundabout in &registry.roundabouts {
        for &(rx, ry) in &roundabout.ring_cells {
            ring_node_set.insert(RoadNode(rx, ry));
        }
    }

    // Adjust weights for edges entering the roundabout
    for node_idx in 0..csr.node_count() {
        let node = csr.nodes[node_idx];
        let is_ring_node = ring_node_set.contains(&node);

        let start = csr.node_offsets[node_idx] as usize;
        let end = csr.node_offsets[node_idx + 1] as usize;

        for edge_pos in start..end {
            let neighbor_idx = csr.edges[edge_pos] as usize;
            let neighbor = csr.nodes[neighbor_idx];
            let neighbor_is_ring = ring_node_set.contains(&neighbor);

            if !is_ring_node && neighbor_is_ring {
                // Edge entering the roundabout: apply yield-on-entry penalty
                let ring_traffic = traffic.get(neighbor.0, neighbor.1);
                let penalty = if ring_traffic > 0 {
                    YIELD_ENTRY_WEIGHT_MULTIPLIER * (1 + ring_traffic as u32 / 5)
                } else {
                    YIELD_ENTRY_WEIGHT_MULTIPLIER
                };
                csr.weights[edge_pos] = csr.weights[edge_pos].max(1) * penalty;
            }
            // Edges within the ring or exiting the ring keep default weight (priority)
        }
    }
}

/// Track roundabout throughput statistics.
///
/// For each roundabout, sums the traffic density on its ring cells and updates
/// rolling average statistics.
fn roundabout_efficiency(
    mut registry: ResMut<RoundaboutRegistry>,
    traffic: Res<TrafficGrid>,
    timer: Res<SlowTickTimer>,
) {
    if !timer.should_run() {
        return;
    }

    let roundabout_count = registry.roundabouts.len();

    // Ensure stats vec matches roundabout count
    registry
        .stats
        .resize_with(roundabout_count, Default::default);

    for i in 0..roundabout_count {
        let throughput: u32 = registry.roundabouts[i]
            .ring_cells
            .iter()
            .map(|&(rx, ry)| {
                if rx < GRID_WIDTH && ry < GRID_HEIGHT {
                    traffic.get(rx, ry) as u32
                } else {
                    0
                }
            })
            .sum();

        let stats = &mut registry.stats[i];
        stats.current_throughput = throughput;
        stats.sample_count += 1;

        // Exponential moving average (alpha = 0.1)
        let alpha = 0.1_f32;
        stats.average_throughput =
            alpha * throughput as f32 + (1.0 - alpha) * stats.average_throughput;
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct RoundaboutPlugin;

impl Plugin for RoundaboutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoundaboutRegistry>()
            .add_systems(FixedUpdate, update_roundabout_traffic)
            .add_systems(FixedUpdate, roundabout_efficiency);

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<RoundaboutRegistry>();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
}
