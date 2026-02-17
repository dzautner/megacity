use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::roads::RoadNetwork;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SegmentNodeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SegmentId(pub u32);

#[derive(Debug, Clone)]
pub struct SegmentNode {
    pub id: SegmentNodeId,
    pub position: Vec2,
    pub connected_segments: Vec<SegmentId>,
}

#[derive(Debug, Clone)]
pub struct RoadSegment {
    pub id: SegmentId,
    pub start_node: SegmentNodeId,
    pub end_node: SegmentNodeId,
    pub p0: Vec2,
    pub p1: Vec2,
    pub p2: Vec2,
    pub p3: Vec2,
    pub road_type: RoadType,
    pub arc_length: f32,
    pub rasterized_cells: Vec<(usize, usize)>,
}

impl RoadSegment {
    /// Evaluate cubic Bezier at parameter t in [0, 1]
    pub fn evaluate(&self, t: f32) -> Vec2 {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        let uu = u * u;
        let tt = t * t;
        u * uu * self.p0 + 3.0 * uu * t * self.p1 + 3.0 * u * tt * self.p2 + t * tt * self.p3
    }

    /// Tangent (first derivative) at parameter t
    pub fn tangent(&self, t: f32) -> Vec2 {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        3.0 * u * u * (self.p1 - self.p0)
            + 6.0 * u * t * (self.p2 - self.p1)
            + 3.0 * t * t * (self.p3 - self.p2)
    }

    /// Compute approximate arc length by sampling
    pub fn compute_arc_length(&self) -> f32 {
        let steps = 64;
        let mut length = 0.0_f32;
        let mut prev = self.p0;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let pt = self.evaluate(t);
            length += (pt - prev).length();
            prev = pt;
        }
        length
    }

    /// Sample n uniformly-spaced points along the curve
    pub fn sample_uniform(&self, n: usize) -> Vec<Vec2> {
        if n == 0 {
            return vec![];
        }
        if n == 1 {
            return vec![self.evaluate(0.5)];
        }

        let lut_steps = 128;
        let mut lut: Vec<(f32, f32)> = Vec::with_capacity(lut_steps + 1);
        let mut prev = self.p0;
        let mut cumulative = 0.0_f32;
        lut.push((0.0, 0.0));
        for i in 1..=lut_steps {
            let t = i as f32 / lut_steps as f32;
            let pt = self.evaluate(t);
            cumulative += (pt - prev).length();
            lut.push((cumulative, t));
            prev = pt;
        }

        let total = cumulative;
        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            let target_dist = (i as f32 / (n - 1) as f32) * total;
            let idx = lut
                .partition_point(|&(d, _)| d < target_dist)
                .min(lut.len() - 1)
                .max(1);
            let (d0, t0) = lut[idx - 1];
            let (d1, t1) = lut[idx];
            let frac = if (d1 - d0).abs() < 1e-6 {
                0.0
            } else {
                (target_dist - d0) / (d1 - d0)
            };
            let t = t0 + frac * (t1 - t0);
            points.push(self.evaluate(t));
        }
        points
    }
}

/// Rasterize a segment onto the grid, returning the affected cells
fn rasterize_segment(
    segment: &RoadSegment,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
) -> Vec<(usize, usize)> {
    let sample_count = ((segment.arc_length / 8.0).ceil() as usize).max(4);
    let points = segment.sample_uniform(sample_count);
    let mut cells: Vec<(usize, usize)> = Vec::new();

    for pt in &points {
        let (gx, gy) = WorldGrid::world_to_grid(pt.x, pt.y);
        if gx < 0 || gy < 0 {
            continue;
        }
        let gx = gx as usize;
        let gy = gy as usize;
        if gx >= GRID_WIDTH || gy >= GRID_HEIGHT {
            continue;
        }
        if cells.contains(&(gx, gy)) {
            continue;
        }
        cells.push((gx, gy));

        let cell = grid.get(gx, gy);
        if cell.cell_type != CellType::Water && cell.cell_type != CellType::Road {
            roads.place_road_typed(grid, gx, gy, segment.road_type);
        }
    }

    cells
}

#[derive(Resource, Default)]
pub struct RoadSegmentStore {
    pub nodes: Vec<SegmentNode>,
    pub segments: Vec<RoadSegment>,
    next_node_id: u32,
    next_segment_id: u32,
}

impl RoadSegmentStore {
    /// Create a store from pre-built nodes and segments (used for deserialization).
    pub fn from_parts(nodes: Vec<SegmentNode>, segments: Vec<RoadSegment>) -> Self {
        let mut store = Self {
            nodes,
            segments,
            next_node_id: 0,
            next_segment_id: 0,
        };
        store.rebuild_counters();
        store
    }

    /// Rebuild internal ID counters from loaded data.
    pub fn rebuild_counters(&mut self) {
        self.next_node_id = self.nodes.iter().map(|n| n.id.0 + 1).max().unwrap_or(0);
        self.next_segment_id = self.segments.iter().map(|s| s.id.0 + 1).max().unwrap_or(0);
    }

    /// Get a segment by ID.
    pub fn get_segment(&self, id: SegmentId) -> Option<&RoadSegment> {
        self.segments.iter().find(|s| s.id == id)
    }

    /// Find an existing node within `snap_dist` world units, or create a new one
    pub fn find_or_create_node(&mut self, pos: Vec2, snap_dist: f32) -> SegmentNodeId {
        for node in &self.nodes {
            if (node.position - pos).length() < snap_dist {
                return node.id;
            }
        }
        let id = SegmentNodeId(self.next_node_id);
        self.next_node_id += 1;
        self.nodes.push(SegmentNode {
            id,
            position: pos,
            connected_segments: Vec::new(),
        });
        id
    }

    pub fn get_node(&self, id: SegmentNodeId) -> Option<&SegmentNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Add a segment with explicit control points and rasterize onto grid
    #[allow(clippy::too_many_arguments)]
    pub fn add_segment(
        &mut self,
        start: SegmentNodeId,
        end: SegmentNodeId,
        p0: Vec2,
        p1: Vec2,
        p2: Vec2,
        p3: Vec2,
        road_type: RoadType,
        grid: &mut WorldGrid,
        roads: &mut RoadNetwork,
    ) -> SegmentId {
        let id = SegmentId(self.next_segment_id);
        self.next_segment_id += 1;

        let mut segment = RoadSegment {
            id,
            start_node: start,
            end_node: end,
            p0,
            p1,
            p2,
            p3,
            road_type,
            arc_length: 0.0,
            rasterized_cells: Vec::new(),
        };
        segment.arc_length = segment.compute_arc_length();
        segment.rasterized_cells = rasterize_segment(&segment, grid, roads);

        // Connect nodes
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == start) {
            node.connected_segments.push(id);
        }
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == end) {
            node.connected_segments.push(id);
        }

        self.segments.push(segment);
        id
    }

    /// Add a straight segment (convenience). Returns (SegmentId, rasterized cells).
    pub fn add_straight_segment(
        &mut self,
        from: Vec2,
        to: Vec2,
        road_type: RoadType,
        snap_dist: f32,
        grid: &mut WorldGrid,
        roads: &mut RoadNetwork,
    ) -> (SegmentId, Vec<(usize, usize)>) {
        let start_node = self.find_or_create_node(from, snap_dist);
        let end_node = self.find_or_create_node(to, snap_dist);
        let p1 = from + (to - from) / 3.0;
        let p2 = from + (to - from) * 2.0 / 3.0;
        let id = self.add_segment(start_node, end_node, from, p1, p2, to, road_type, grid, roads);
        let cells = self.segments.iter().find(|s| s.id == id)
            .map(|s| s.rasterized_cells.clone())
            .unwrap_or_default();
        (id, cells)
    }

    /// Remove a segment and un-rasterize it from the grid
    pub fn remove_segment(
        &mut self,
        id: SegmentId,
        grid: &mut WorldGrid,
        roads: &mut RoadNetwork,
    ) {
        if let Some(idx) = self.segments.iter().position(|s| s.id == id) {
            let segment = self.segments.remove(idx);

            for &(gx, gy) in &segment.rasterized_cells {
                let covered_by_other = self
                    .segments
                    .iter()
                    .any(|s| s.rasterized_cells.contains(&(gx, gy)));
                if !covered_by_other {
                    roads.remove_road(grid, gx, gy);
                }
            }

            // Disconnect from nodes
            for node in &mut self.nodes {
                node.connected_segments.retain(|&sid| sid != id);
            }
        }
    }

    /// Re-rasterize all segments (used after load)
    pub fn rasterize_all(&mut self, grid: &mut WorldGrid, roads: &mut RoadNetwork) {
        for segment in &mut self.segments {
            segment.rasterized_cells = rasterize_segment(segment, grid, roads);
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};

    #[test]
    fn test_bezier_evaluate_endpoints() {
        let seg = RoadSegment {
            id: SegmentId(0),
            start_node: SegmentNodeId(0),
            end_node: SegmentNodeId(1),
            p0: Vec2::new(0.0, 0.0),
            p1: Vec2::new(100.0, 0.0),
            p2: Vec2::new(200.0, 100.0),
            p3: Vec2::new(300.0, 100.0),
            road_type: RoadType::Local,
            arc_length: 0.0,
            rasterized_cells: Vec::new(),
        };
        let start = seg.evaluate(0.0);
        let end = seg.evaluate(1.0);
        assert!((start - seg.p0).length() < 0.01);
        assert!((end - seg.p3).length() < 0.01);
    }

    #[test]
    fn test_arc_length_straight_line() {
        let seg = RoadSegment {
            id: SegmentId(0),
            start_node: SegmentNodeId(0),
            end_node: SegmentNodeId(1),
            p0: Vec2::new(0.0, 0.0),
            p1: Vec2::new(100.0, 0.0),
            p2: Vec2::new(200.0, 0.0),
            p3: Vec2::new(300.0, 0.0),
            road_type: RoadType::Local,
            arc_length: 0.0,
            rasterized_cells: Vec::new(),
        };
        let len = seg.compute_arc_length();
        assert!((len - 300.0).abs() < 1.0);
    }

    #[test]
    fn test_sample_uniform() {
        let seg = RoadSegment {
            id: SegmentId(0),
            start_node: SegmentNodeId(0),
            end_node: SegmentNodeId(1),
            p0: Vec2::new(0.0, 0.0),
            p1: Vec2::new(100.0, 0.0),
            p2: Vec2::new(200.0, 0.0),
            p3: Vec2::new(300.0, 0.0),
            road_type: RoadType::Local,
            arc_length: 300.0,
            rasterized_cells: Vec::new(),
        };
        let pts = seg.sample_uniform(4);
        assert_eq!(pts.len(), 4);
        assert!((pts[0] - Vec2::new(0.0, 0.0)).length() < 1.0);
        assert!((pts[3] - Vec2::new(300.0, 0.0)).length() < 1.0);
    }

    #[test]
    fn test_rasterize_straight_segment() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();

        let from = Vec2::new(128.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let to = Vec2::new(132.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        store.add_straight_segment(from, to, RoadType::Local, 24.0, &mut grid, &mut roads);

        assert_eq!(store.segments.len(), 1);
        assert!(!store.segments[0].rasterized_cells.is_empty());
        // Check that at least some cells became roads
        let road_cells = store.segments[0]
            .rasterized_cells
            .iter()
            .filter(|&&(gx, gy)| grid.get(gx, gy).cell_type == CellType::Road)
            .count();
        assert!(road_cells > 0);
    }
}
