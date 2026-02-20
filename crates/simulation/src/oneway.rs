use bevy::prelude::*;
use std::collections::HashMap;

use crate::road_graph_csr::CsrGraph;
use crate::road_segments::{RoadSegmentStore, SegmentId};
use crate::roads::{RoadNetwork, RoadNode};
use crate::Saveable;

/// Direction of traffic flow on a one-way segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OneWayDirection {
    /// Traffic flows from start_node to end_node.
    Forward,
    /// Traffic flows from end_node to start_node.
    Reverse,
}

/// Stores one-way direction overrides for road segments.
///
/// By default, all segments are bidirectional. When a segment is added to this map,
/// it becomes one-way in the specified direction.
#[derive(Resource, Default, Debug)]
pub struct OneWayDirectionMap {
    /// Map from segment ID to one-way direction.
    pub directions: HashMap<u32, OneWayDirection>,
    /// Incremented every time the map changes, so systems can detect changes cheaply.
    pub generation: u32,
}

impl OneWayDirectionMap {
    /// Get the one-way direction for a segment, if any.
    pub fn get(&self, id: SegmentId) -> Option<OneWayDirection> {
        self.directions.get(&id.0).copied()
    }

    /// Set a segment to one-way in the given direction.
    pub fn set(&mut self, id: SegmentId, direction: OneWayDirection) {
        self.directions.insert(id.0, direction);
        self.generation = self.generation.wrapping_add(1);
    }

    /// Remove one-way restriction (make bidirectional again).
    pub fn remove(&mut self, id: SegmentId) {
        self.directions.remove(&id.0);
        self.generation = self.generation.wrapping_add(1);
    }

    /// Toggle through: None -> Forward -> Reverse -> None
    pub fn toggle(&mut self, id: SegmentId) {
        match self.get(id) {
            None => self.set(id, OneWayDirection::Forward),
            Some(OneWayDirection::Forward) => self.set(id, OneWayDirection::Reverse),
            Some(OneWayDirection::Reverse) => self.remove(id),
        }
    }

    /// Check if a segment is one-way.
    pub fn is_oneway(&self, id: SegmentId) -> bool {
        self.directions.contains_key(&id.0)
    }
}

/// Event fired when user toggles one-way direction on a segment.
#[derive(Event)]
pub struct ToggleOneWayEvent {
    pub segment_id: SegmentId,
}

/// Handle toggle events by cycling through direction states.
fn handle_toggle_oneway(
    mut events: EventReader<ToggleOneWayEvent>,
    mut oneway_map: ResMut<OneWayDirectionMap>,
) {
    for event in events.read() {
        oneway_map.toggle(event.segment_id);
    }
}

/// Rebuild the CSR graph incorporating one-way direction constraints.
///
/// For bidirectional segments, edges go both ways (A->B and B->A).
/// For one-way Forward segments, only start_node->end_node edges exist.
/// For one-way Reverse segments, only end_node->start_node edges exist.
///
/// This replaces the default `rebuild_csr_on_road_change` when one-way
/// directions are active.
pub fn rebuild_csr_with_oneway(
    roads: Res<RoadNetwork>,
    segments: Res<RoadSegmentStore>,
    oneway_map: Res<OneWayDirectionMap>,
    mut csr: ResMut<CsrGraph>,
    mut last_gen: Local<u32>,
) {
    // Only rebuild if something changed
    if !roads.is_changed() && *last_gen == oneway_map.generation {
        return;
    }
    *last_gen = oneway_map.generation;

    // If no one-way directions exist, just use the standard builder
    if oneway_map.directions.is_empty() {
        *csr = CsrGraph::from_road_network(&roads);
        return;
    }

    // Build a set of directed edge restrictions from segments
    // For each one-way segment, identify which grid cells are rasterized and
    // restrict edges between consecutive rasterized cells to the allowed direction.
    let mut blocked_edges: std::collections::HashSet<(RoadNode, RoadNode)> =
        std::collections::HashSet::new();

    for segment in &segments.segments {
        let Some(direction) = oneway_map.get(segment.id) else {
            continue;
        };

        let cells = &segment.rasterized_cells;
        if cells.len() < 2 {
            continue;
        }

        // For each pair of consecutive rasterized cells, block the reverse direction
        for window in cells.windows(2) {
            let a = RoadNode(window[0].0, window[0].1);
            let b = RoadNode(window[1].0, window[1].1);

            match direction {
                OneWayDirection::Forward => {
                    // Allow A->B, block B->A
                    blocked_edges.insert((b, a));
                }
                OneWayDirection::Reverse => {
                    // Allow B->A, block A->B
                    blocked_edges.insert((a, b));
                }
            }
        }
    }

    // Build CSR graph from road network, filtering out blocked edges
    *csr = CsrGraph::from_road_network_filtered(&roads, &blocked_edges);
}

impl CsrGraph {
    /// Build CSR graph from road network, excluding blocked directed edges.
    pub fn from_road_network_filtered(
        network: &RoadNetwork,
        blocked: &std::collections::HashSet<(RoadNode, RoadNode)>,
    ) -> Self {
        let mut nodes: Vec<RoadNode> = network.edges.keys().copied().collect();
        nodes.sort_by(|a, b| (a.1, a.0).cmp(&(b.1, b.0)));

        let node_index: std::collections::HashMap<RoadNode, u32> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (*n, i as u32))
            .collect();

        let mut node_offsets = Vec::with_capacity(nodes.len() + 1);
        let mut edges = Vec::new();
        let mut weights = Vec::new();

        for node in &nodes {
            node_offsets.push(edges.len() as u32);
            if let Some(neighbors) = network.edges.get(node) {
                for neighbor in neighbors {
                    // Skip blocked edges
                    if blocked.contains(&(*node, *neighbor)) {
                        continue;
                    }
                    if let Some(&idx) = node_index.get(neighbor) {
                        edges.push(idx);
                        weights.push(1);
                    }
                }
            }
        }
        node_offsets.push(edges.len() as u32);

        Self {
            nodes,
            node_offsets,
            edges,
            weights,
        }
    }
}

// ---------------------------------------------------------------------------
// Save / Load
// ---------------------------------------------------------------------------

impl Saveable for OneWayDirectionMap {
    const SAVE_KEY: &'static str = "oneway_direction_map";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.directions.is_empty() {
            return None;
        }

        let mut buf = Vec::new();

        // Entry count (4 bytes)
        let count = self.directions.len() as u32;
        buf.extend_from_slice(&count.to_le_bytes());

        // Each entry: segment_id (4 bytes) + direction (1 byte)
        for (&seg_id, &direction) in &self.directions {
            buf.extend_from_slice(&seg_id.to_le_bytes());
            buf.push(match direction {
                OneWayDirection::Forward => 0,
                OneWayDirection::Reverse => 1,
            });
        }

        // Generation (4 bytes)
        buf.extend_from_slice(&self.generation.to_le_bytes());

        Some(buf)
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < 4 {
            return Self::default();
        }

        let count = u32::from_le_bytes(bytes[0..4].try_into().unwrap_or([0; 4])) as usize;
        let mut directions = HashMap::new();

        let mut offset = 4;
        for _ in 0..count {
            if offset + 5 > bytes.len() {
                break;
            }
            let seg_id =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
            let dir_byte = bytes[offset + 4];
            let direction = match dir_byte {
                0 => OneWayDirection::Forward,
                _ => OneWayDirection::Reverse,
            };
            directions.insert(seg_id, direction);
            offset += 5;
        }

        let generation = if offset + 4 <= bytes.len() {
            u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]))
        } else {
            0
        };

        Self {
            directions,
            generation,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct OneWayPlugin;

impl Plugin for OneWayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OneWayDirectionMap>()
            .add_event::<ToggleOneWayEvent>()
            .add_systems(Update, handle_toggle_oneway)
            .add_systems(
                Update,
                rebuild_csr_with_oneway.after(handle_toggle_oneway),
            );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<OneWayDirectionMap>();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::WorldGrid;
    use crate::road_graph_csr::csr_find_path;
    use crate::road_segments::RoadSegmentStore;
    use crate::roads::RoadNetwork;

    #[test]
    fn test_toggle_cycles_directions() {
        let mut map = OneWayDirectionMap::default();
        let id = SegmentId(42);

        assert_eq!(map.get(id), None);

        map.toggle(id);
        assert_eq!(map.get(id), Some(OneWayDirection::Forward));

        map.toggle(id);
        assert_eq!(map.get(id), Some(OneWayDirection::Reverse));

        map.toggle(id);
        assert_eq!(map.get(id), None);
    }

    #[test]
    fn test_generation_increments_on_change() {
        let mut map = OneWayDirectionMap::default();
        let id = SegmentId(1);

        assert_eq!(map.generation, 0);
        map.set(id, OneWayDirection::Forward);
        assert_eq!(map.generation, 1);
        map.remove(id);
        assert_eq!(map.generation, 2);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut map = OneWayDirectionMap::default();
        map.set(SegmentId(10), OneWayDirection::Forward);
        map.set(SegmentId(20), OneWayDirection::Reverse);

        let bytes = map.save_to_bytes().unwrap();
        let loaded = OneWayDirectionMap::load_from_bytes(&bytes);

        assert_eq!(loaded.get(SegmentId(10)), Some(OneWayDirection::Forward));
        assert_eq!(loaded.get(SegmentId(20)), Some(OneWayDirection::Reverse));
        assert_eq!(loaded.directions.len(), 2);
    }

    #[test]
    fn test_saveable_empty_returns_none() {
        let map = OneWayDirectionMap::default();
        assert!(map.save_to_bytes().is_none());
    }

    #[test]
    fn test_oneway_forward_blocks_reverse_path() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();

        // Build a straight road from (5,10) to (15,10) using segments
        let from = Vec2::new(5.0 * CELL_SIZE + 8.0, 10.0 * CELL_SIZE + 8.0);
        let to = Vec2::new(15.0 * CELL_SIZE + 8.0, 10.0 * CELL_SIZE + 8.0);
        let (seg_id, _cells) =
            store.add_straight_segment(from, to, crate::grid::RoadType::Local, 24.0, &mut grid, &mut roads);

        // Without one-way: path should exist in both directions
        let csr_bidir = CsrGraph::from_road_network(&roads);
        let forward_path = csr_find_path(&csr_bidir, RoadNode(5, 10), RoadNode(15, 10));
        let reverse_path = csr_find_path(&csr_bidir, RoadNode(15, 10), RoadNode(5, 10));
        assert!(forward_path.is_some(), "Forward path should exist bidirectional");
        assert!(reverse_path.is_some(), "Reverse path should exist bidirectional");

        // Set one-way Forward (start -> end, i.e. 5,10 -> 15,10)
        let mut oneway_map = OneWayDirectionMap::default();
        oneway_map.set(seg_id, OneWayDirection::Forward);

        // Build blocked edges
        let mut blocked = std::collections::HashSet::new();
        for segment in &store.segments {
            if let Some(direction) = oneway_map.get(segment.id) {
                let cells = &segment.rasterized_cells;
                for window in cells.windows(2) {
                    let a = RoadNode(window[0].0, window[0].1);
                    let b = RoadNode(window[1].0, window[1].1);
                    match direction {
                        OneWayDirection::Forward => {
                            blocked.insert((b, a));
                        }
                        OneWayDirection::Reverse => {
                            blocked.insert((a, b));
                        }
                    }
                }
            }
        }

        let csr_oneway = CsrGraph::from_road_network_filtered(&roads, &blocked);

        // Forward path should still exist
        let forward_path = csr_find_path(&csr_oneway, RoadNode(5, 10), RoadNode(15, 10));
        assert!(forward_path.is_some(), "Forward path should exist with one-way forward");

        // Reverse path should be blocked
        let reverse_path = csr_find_path(&csr_oneway, RoadNode(15, 10), RoadNode(5, 10));
        assert!(reverse_path.is_none(), "Reverse path should be blocked with one-way forward");
    }

    #[test]
    fn test_oneway_reverse_blocks_forward_path() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();

        let from = Vec2::new(5.0 * CELL_SIZE + 8.0, 10.0 * CELL_SIZE + 8.0);
        let to = Vec2::new(15.0 * CELL_SIZE + 8.0, 10.0 * CELL_SIZE + 8.0);
        let (seg_id, _cells) =
            store.add_straight_segment(from, to, crate::grid::RoadType::Local, 24.0, &mut grid, &mut roads);

        let mut oneway_map = OneWayDirectionMap::default();
        oneway_map.set(seg_id, OneWayDirection::Reverse);

        let mut blocked = std::collections::HashSet::new();
        for segment in &store.segments {
            if let Some(direction) = oneway_map.get(segment.id) {
                let cells = &segment.rasterized_cells;
                for window in cells.windows(2) {
                    let a = RoadNode(window[0].0, window[0].1);
                    let b = RoadNode(window[1].0, window[1].1);
                    match direction {
                        OneWayDirection::Forward => {
                            blocked.insert((b, a));
                        }
                        OneWayDirection::Reverse => {
                            blocked.insert((a, b));
                        }
                    }
                }
            }
        }

        let csr_oneway = CsrGraph::from_road_network_filtered(&roads, &blocked);

        // Forward path should be blocked
        let forward_path = csr_find_path(&csr_oneway, RoadNode(5, 10), RoadNode(15, 10));
        assert!(forward_path.is_none(), "Forward path should be blocked with one-way reverse");

        // Reverse path should exist
        let reverse_path = csr_find_path(&csr_oneway, RoadNode(15, 10), RoadNode(5, 10));
        assert!(reverse_path.is_some(), "Reverse path should exist with one-way reverse");
    }

    #[test]
    fn test_removing_oneway_restores_bidirectional() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();

        let from = Vec2::new(5.0 * CELL_SIZE + 8.0, 10.0 * CELL_SIZE + 8.0);
        let to = Vec2::new(15.0 * CELL_SIZE + 8.0, 10.0 * CELL_SIZE + 8.0);
        let (seg_id, _cells) =
            store.add_straight_segment(from, to, crate::grid::RoadType::Local, 24.0, &mut grid, &mut roads);

        let mut oneway_map = OneWayDirectionMap::default();
        oneway_map.set(seg_id, OneWayDirection::Forward);
        oneway_map.remove(seg_id);

        // No blocked edges since we removed the one-way
        let blocked = std::collections::HashSet::new();
        let csr = CsrGraph::from_road_network_filtered(&roads, &blocked);

        let forward = csr_find_path(&csr, RoadNode(5, 10), RoadNode(15, 10));
        let reverse = csr_find_path(&csr, RoadNode(15, 10), RoadNode(5, 10));
        assert!(forward.is_some(), "Forward path should exist after removing one-way");
        assert!(reverse.is_some(), "Reverse path should exist after removing one-way");
    }
}
