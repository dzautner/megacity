use std::collections::{BTreeMap, BTreeSet, HashSet};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::grid::{CellType, RoadType, WorldGrid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RoadNode(pub usize, pub usize);

/// The road network graph.
///
/// **Deterministic collections**: `edges` uses `BTreeMap<RoadNode, BTreeSet<RoadNode>>`
/// to guarantee deterministic iteration order. This is critical because:
/// - `CsrGraph::from_road_network` iterates edges to build the CSR graph
/// - `neighbors()` returns nodes in a deterministic order for pathfinding
/// - Simulation invariant checks iterate the edge map
///
/// `intersections` remains a `HashSet` because it is only used for membership
/// checks, never iterated in an order-dependent way.
#[derive(Resource, Default, Serialize, Deserialize)]
pub struct RoadNetwork {
    /// Adjacency list using ordered collections for deterministic iteration.
    pub edges: BTreeMap<RoadNode, BTreeSet<RoadNode>>,
    pub intersections: HashSet<RoadNode>,
    /// Nodes removed since the last drain. Movement systems drain this to
    /// invalidate stale `PathCache` entries that reference deleted roads.
    #[serde(skip)]
    pub recently_removed: Vec<RoadNode>,
}

impl RoadNetwork {
    pub fn place_road(&mut self, grid: &mut WorldGrid, x: usize, y: usize) -> bool {
        self.place_road_typed(grid, x, y, RoadType::Local)
    }

    pub fn place_road_typed(
        &mut self,
        grid: &mut WorldGrid,
        x: usize,
        y: usize,
        road_type: RoadType,
    ) -> bool {
        if !grid.in_bounds(x, y) {
            return false;
        }
        let cell = grid.get(x, y);
        if cell.cell_type == CellType::Water {
            return false;
        }
        if cell.cell_type == CellType::Road {
            return false; // already a road
        }
        if cell.building_id.is_some() {
            return false;
        }

        grid.get_mut(x, y).cell_type = CellType::Road;
        grid.get_mut(x, y).road_type = road_type;

        let node = RoadNode(x, y);
        self.edges.entry(node).or_default();

        // Connect to adjacent road cells
        let (neighbors, ncount) = grid.neighbors4(x, y);
        for &(nx, ny) in &neighbors[..ncount] {
            if grid.get(nx, ny).cell_type == CellType::Road {
                let neighbor_node = RoadNode(nx, ny);
                self.edges.entry(node).or_default().insert(neighbor_node);
                self.edges.entry(neighbor_node).or_default().insert(node);
            }
        }

        // Update intersection status for this node and neighbors
        self.update_intersection(node);
        for &(nx, ny) in &neighbors[..ncount] {
            if grid.get(nx, ny).cell_type == CellType::Road {
                self.update_intersection(RoadNode(nx, ny));
            }
        }

        true
    }

    pub fn remove_road(&mut self, grid: &mut WorldGrid, x: usize, y: usize) -> bool {
        if !grid.in_bounds(x, y) || grid.get(x, y).cell_type != CellType::Road {
            return false;
        }

        let node = RoadNode(x, y);

        // Record the removed node so movement systems can invalidate caches
        self.recently_removed.push(node);

        // Remove edges from neighbors pointing to this node
        if let Some(neighbors) = self.edges.remove(&node) {
            for neighbor in &neighbors {
                if let Some(nset) = self.edges.get_mut(neighbor) {
                    nset.remove(&node);
                }
                self.update_intersection(*neighbor);
            }
        }
        self.intersections.remove(&node);

        grid.get_mut(x, y).cell_type = CellType::Grass;
        grid.get_mut(x, y).zone = crate::grid::ZoneType::None;
        grid.get_mut(x, y).building_id = None;

        true
    }

    /// Drain all recently-removed nodes. Returns the removed set for checking
    /// against cached paths.
    pub fn drain_removed(&mut self) -> HashSet<RoadNode> {
        self.recently_removed.drain(..).collect()
    }

    fn update_intersection(&mut self, node: RoadNode) {
        let edge_count = self.edges.get(&node).map_or(0, |e| e.len());
        if edge_count >= 3 {
            self.intersections.insert(node);
        } else {
            self.intersections.remove(&node);
        }
    }

    pub fn is_road(&self, x: usize, y: usize) -> bool {
        self.edges.contains_key(&RoadNode(x, y))
    }

    /// Returns neighbors in deterministic order (BTreeSet iteration).
    pub fn neighbors(&self, node: &RoadNode) -> Vec<RoadNode> {
        self.edges
            .get(node)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    #[test]
    fn test_place_road_creates_edges() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();

        assert!(roads.place_road(&mut grid, 10, 10));
        assert!(roads.place_road(&mut grid, 11, 10));
        assert!(roads.place_road(&mut grid, 12, 10));

        let node = RoadNode(11, 10);
        let neighbors = roads.neighbors(&node);
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&RoadNode(10, 10)));
        assert!(neighbors.contains(&RoadNode(12, 10)));
    }

    #[test]
    fn test_no_road_on_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(5, 5).cell_type = CellType::Water;
        let mut roads = RoadNetwork::default();

        assert!(!roads.place_road(&mut grid, 5, 5));
    }

    #[test]
    fn test_intersection_detection() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();

        // Create a cross: center at (10,10)
        roads.place_road(&mut grid, 10, 10);
        roads.place_road(&mut grid, 9, 10);
        roads.place_road(&mut grid, 11, 10);
        assert!(!roads.intersections.contains(&RoadNode(10, 10)));

        roads.place_road(&mut grid, 10, 9);
        assert!(roads.intersections.contains(&RoadNode(10, 10)));
    }

    #[test]
    fn test_remove_road() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();

        roads.place_road(&mut grid, 10, 10);
        roads.place_road(&mut grid, 11, 10);
        roads.place_road(&mut grid, 12, 10);

        roads.remove_road(&mut grid, 11, 10);
        assert!(!roads.is_road(11, 10));
        assert_eq!(roads.neighbors(&RoadNode(10, 10)).len(), 0);
        assert_eq!(roads.neighbors(&RoadNode(12, 10)).len(), 0);
    }

    #[test]
    fn test_no_duplicate_road() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();

        assert!(roads.place_road(&mut grid, 10, 10));
        assert!(!roads.place_road(&mut grid, 10, 10)); // already road
    }

    #[test]
    fn test_remove_road_tracks_removed_nodes() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();

        roads.place_road(&mut grid, 10, 10);
        roads.place_road(&mut grid, 11, 10);
        roads.place_road(&mut grid, 12, 10);

        assert!(roads.recently_removed.is_empty());
        roads.remove_road(&mut grid, 11, 10);
        assert_eq!(roads.recently_removed.len(), 1);
        assert_eq!(roads.recently_removed[0], RoadNode(11, 10));

        let removed = roads.drain_removed();
        assert!(removed.contains(&RoadNode(11, 10)));
        assert!(roads.recently_removed.is_empty());
    }

    #[test]
    fn test_neighbors_are_deterministically_ordered() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();

        // Create a cross: center at (10,10) with 4 neighbors
        roads.place_road(&mut grid, 10, 10);
        roads.place_road(&mut grid, 9, 10);
        roads.place_road(&mut grid, 11, 10);
        roads.place_road(&mut grid, 10, 9);
        roads.place_road(&mut grid, 10, 11);

        // Verify neighbors are returned in deterministic (sorted) order
        let neighbors = roads.neighbors(&RoadNode(10, 10));
        assert_eq!(neighbors.len(), 4);
        // BTreeSet iteration yields RoadNode in Ord order: (x, y) ascending
        assert_eq!(neighbors[0], RoadNode(9, 10));
        assert_eq!(neighbors[1], RoadNode(10, 9));
        assert_eq!(neighbors[2], RoadNode(10, 11));
        assert_eq!(neighbors[3], RoadNode(11, 10));

        // Verify repeated calls return the same order
        let neighbors2 = roads.neighbors(&RoadNode(10, 10));
        assert_eq!(neighbors, neighbors2);
    }
}
