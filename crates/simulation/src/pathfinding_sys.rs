use pathfinding::prelude::astar;

use crate::grid::{CellType, WorldGrid};
use crate::roads::{RoadNetwork, RoadNode};

pub fn find_path(network: &RoadNetwork, start: RoadNode, goal: RoadNode) -> Option<Vec<RoadNode>> {
    if start == goal {
        return Some(vec![start]);
    }

    let result = astar(
        &start,
        |node| network.neighbors(node).into_iter().map(|n| (n, 1u32)),
        |node| heuristic(node, &goal),
        |node| *node == goal,
    );

    result.map(|(path, _cost)| path)
}

fn heuristic(a: &RoadNode, b: &RoadNode) -> u32 {
    let dx = (a.0 as i32 - b.0 as i32).unsigned_abs();
    let dy = (a.1 as i32 - b.1 as i32).unsigned_abs();
    dx + dy
}

/// Find the nearest road node to a grid position.
/// Uses direct grid lookup + spiral search instead of linear scan over all roads.
/// Complexity: O(r^2) where r = search radius (max 3), vs O(R) for all road nodes.
pub fn nearest_road(network: &RoadNetwork, gx: usize, gy: usize) -> Option<RoadNode> {
    let target = RoadNode(gx, gy);
    if network.edges.contains_key(&target) {
        return Some(target);
    }

    // Spiral search in expanding Manhattan-distance rings (radius 1, 2, 3)
    for radius in 1..=3i32 {
        let mut best: Option<(RoadNode, u32)> = None;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let dist = dx.unsigned_abs() + dy.unsigned_abs();
                if dist != radius as u32 {
                    continue; // Only check the current ring, not inner
                }
                let nx = gx as i32 + dx;
                let ny = gy as i32 + dy;
                if nx < 0 || ny < 0 {
                    continue;
                }
                let node = RoadNode(nx as usize, ny as usize);
                if network.edges.contains_key(&node) {
                    match best {
                        None => best = Some((node, dist)),
                        Some((_, bd)) if dist < bd => best = Some((node, dist)),
                        _ => {}
                    }
                }
            }
        }
        if best.is_some() {
            return best.map(|(n, _)| n);
        }
    }
    None
}

/// Grid-accelerated nearest road lookup. Uses WorldGrid cell types for O(1) checks
/// instead of HashMap lookups. Faster when the grid is available.
pub fn nearest_road_grid(grid: &WorldGrid, gx: usize, gy: usize) -> Option<RoadNode> {
    // Direct check
    if grid.in_bounds(gx, gy) && grid.get(gx, gy).cell_type == CellType::Road {
        return Some(RoadNode(gx, gy));
    }

    // Spiral search using grid cell types (no hash lookups)
    for radius in 1..=3i32 {
        let mut best: Option<RoadNode> = None;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let dist = dx.unsigned_abs() + dy.unsigned_abs();
                if dist != radius as u32 {
                    continue;
                }
                let nx = gx as i32 + dx;
                let ny = gy as i32 + dy;
                if nx < 0 || ny < 0 {
                    continue;
                }
                let (ux, uy) = (nx as usize, ny as usize);
                if grid.in_bounds(ux, uy)
                    && grid.get(ux, uy).cell_type == CellType::Road
                    && best.is_none()
                {
                    best = Some(RoadNode(ux, uy));
                }
            }
        }
        if best.is_some() {
            return best;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::WorldGrid;

    #[test]
    fn test_pathfinding_straight_line() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut network = RoadNetwork::default();

        // Lay a straight road
        for x in 5..=15 {
            network.place_road(&mut grid, x, 10);
        }

        let path = find_path(&network, RoadNode(5, 10), RoadNode(15, 10));
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 11); // 5 to 15 inclusive
        assert_eq!(path[0], RoadNode(5, 10));
        assert_eq!(path[10], RoadNode(15, 10));
    }

    #[test]
    fn test_pathfinding_no_path() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut network = RoadNetwork::default();

        // Two disconnected road segments
        network.place_road(&mut grid, 5, 10);
        network.place_road(&mut grid, 6, 10);
        network.place_road(&mut grid, 20, 10);
        network.place_road(&mut grid, 21, 10);

        let path = find_path(&network, RoadNode(5, 10), RoadNode(20, 10));
        assert!(path.is_none());
    }

    #[test]
    fn test_nearest_road() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut network = RoadNetwork::default();
        network.place_road(&mut grid, 10, 10);

        assert_eq!(nearest_road(&network, 10, 10), Some(RoadNode(10, 10)));
        assert_eq!(nearest_road(&network, 11, 10), Some(RoadNode(10, 10)));
        assert_eq!(nearest_road(&network, 100, 100), None); // too far
    }
}
