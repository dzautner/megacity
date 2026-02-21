//! Road upgrade system: upgrade existing road segments to higher-tier road types
//! without bulldozing and rebuilding.
//!
//! Upgrade path: Path -> Local -> Avenue -> Boulevard
//! OneWay -> Avenue
//! Highway and Boulevard have no further upgrade.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::road_segments::{RoadSegmentStore, SegmentId};
use crate::roads::RoadNetwork;

/// Event fired when a road segment should be upgraded.
#[derive(Event)]
pub struct RoadUpgradeEvent {
    pub segment_id: SegmentId,
}

/// Upgrade a road segment to the next tier in-place.
///
/// This changes the segment's `road_type`, updates all rasterized grid cells
/// to the new type, and deducts the upgrade cost from the city budget.
///
/// Returns `Ok(new_road_type)` on success, or `Err(reason)` on failure.
pub fn upgrade_segment(
    segment_id: SegmentId,
    segments: &mut RoadSegmentStore,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
    budget: &mut CityBudget,
) -> Result<RoadType, &'static str> {
    // Find the segment
    let seg_idx = segments
        .segments
        .iter()
        .position(|s| s.id == segment_id)
        .ok_or("Segment not found")?;

    let current_type = segments.segments[seg_idx].road_type;

    // Determine upgrade tier
    let new_type = current_type
        .upgrade_tier()
        .ok_or("Already at maximum road tier")?;

    // Calculate cost: difference between new tier and current tier, multiplied by
    // the number of rasterized cells
    let cell_count = segments.segments[seg_idx].rasterized_cells.len();
    let cost_per_cell = new_type.cost() - current_type.cost();
    let total_cost = cost_per_cell * cell_count as f64;

    if budget.treasury < total_cost {
        return Err("Not enough money");
    }

    // Deduct cost
    budget.treasury -= total_cost;

    // Update the segment's road type
    segments.segments[seg_idx].road_type = new_type;

    // Update all rasterized grid cells to the new road type
    let cells: Vec<(usize, usize)> = segments.segments[seg_idx].rasterized_cells.clone();
    for &(gx, gy) in &cells {
        if gx < GRID_WIDTH && gy < GRID_HEIGHT {
            let cell = grid.get(gx, gy);
            if cell.cell_type == CellType::Road {
                grid.get_mut(gx, gy).road_type = new_type;
            }
        }
    }

    // Update road network edges -- the connectivity doesn't change,
    // but the road type stored in the grid cells is what pathfinding uses
    // for speed/capacity lookups, so we don't need to rebuild the graph.

    // Re-rasterize the segment to account for any width changes
    // (currently all road types are 1 cell wide, but this future-proofs it)
    let segment = &segments.segments[seg_idx];
    let sample_count = ((segment.arc_length / 8.0).ceil() as usize).max(4);
    let points = segment.sample_uniform(sample_count);
    let mut new_cells: Vec<(usize, usize)> = Vec::new();

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
        if new_cells.contains(&(gx, gy)) {
            continue;
        }
        new_cells.push((gx, gy));

        let cell = grid.get(gx, gy);
        if cell.cell_type != CellType::Water && cell.cell_type != CellType::Road {
            roads.place_road_typed(grid, gx, gy, new_type);
        } else if cell.cell_type == CellType::Road {
            grid.get_mut(gx, gy).road_type = new_type;
        }
    }

    segments.segments[seg_idx].rasterized_cells = new_cells;

    Ok(new_type)
}

/// Find the segment closest to a world position, if any.
/// Returns `(SegmentId, distance)` for the closest segment within `max_dist`.
pub fn find_segment_near(
    world_pos: bevy::math::Vec2,
    segments: &RoadSegmentStore,
    max_dist: f32,
) -> Option<SegmentId> {
    let mut best_id: Option<SegmentId> = None;
    let mut best_dist = max_dist;

    for segment in &segments.segments {
        // Sample several points along the segment and find closest
        let sample_count = ((segment.arc_length / 16.0).ceil() as usize).max(4);
        for i in 0..=sample_count {
            let t = i as f32 / sample_count as f32;
            let pt = segment.evaluate(t);
            let dist = (pt - world_pos).length();
            if dist < best_dist {
                best_dist = dist;
                best_id = Some(segment.id);
            }
        }
    }

    best_id
}

pub struct RoadUpgradePlugin;

impl Plugin for RoadUpgradePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RoadUpgradeEvent>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CELL_SIZE;

    #[test]
    fn test_upgrade_local_to_avenue() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();
        let mut budget = CityBudget {
            treasury: 10000.0,
            ..Default::default()
        };

        let from = bevy::math::Vec2::new(128.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let to = bevy::math::Vec2::new(132.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let (seg_id, _cells) =
            store.add_straight_segment(from, to, RoadType::Local, 24.0, &mut grid, &mut roads);

        let result = upgrade_segment(seg_id, &mut store, &mut grid, &mut roads, &mut budget);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RoadType::Avenue);

        // Verify segment type changed
        let seg = store.get_segment(seg_id).unwrap();
        assert_eq!(seg.road_type, RoadType::Avenue);

        // Verify grid cells updated
        for &(gx, gy) in &seg.rasterized_cells {
            assert_eq!(grid.get(gx, gy).road_type, RoadType::Avenue);
        }

        // Verify cost was deducted
        assert!(budget.treasury < 10000.0);
    }

    #[test]
    fn test_upgrade_path_to_local() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();
        let mut budget = CityBudget {
            treasury: 10000.0,
            ..Default::default()
        };

        let from = bevy::math::Vec2::new(128.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let to = bevy::math::Vec2::new(132.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let (seg_id, _) =
            store.add_straight_segment(from, to, RoadType::Path, 24.0, &mut grid, &mut roads);

        let result = upgrade_segment(seg_id, &mut store, &mut grid, &mut roads, &mut budget);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RoadType::Local);
    }

    #[test]
    fn test_upgrade_boulevard_fails_already_max() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();
        let mut budget = CityBudget {
            treasury: 10000.0,
            ..Default::default()
        };

        let from = bevy::math::Vec2::new(128.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let to = bevy::math::Vec2::new(132.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let (seg_id, _) =
            store.add_straight_segment(from, to, RoadType::Boulevard, 24.0, &mut grid, &mut roads);

        let result = upgrade_segment(seg_id, &mut store, &mut grid, &mut roads, &mut budget);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Already at maximum road tier");
    }

    #[test]
    fn test_upgrade_highway_fails_already_max() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();
        let mut budget = CityBudget {
            treasury: 10000.0,
            ..Default::default()
        };

        let from = bevy::math::Vec2::new(128.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let to = bevy::math::Vec2::new(132.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let (seg_id, _) =
            store.add_straight_segment(from, to, RoadType::Highway, 24.0, &mut grid, &mut roads);

        let result = upgrade_segment(seg_id, &mut store, &mut grid, &mut roads, &mut budget);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Already at maximum road tier");
    }

    #[test]
    fn test_upgrade_insufficient_funds() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();
        let mut budget = CityBudget {
            treasury: 0.0,
            ..Default::default()
        };

        let from = bevy::math::Vec2::new(128.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let to = bevy::math::Vec2::new(132.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let (seg_id, _) =
            store.add_straight_segment(from, to, RoadType::Local, 24.0, &mut grid, &mut roads);

        let result = upgrade_segment(seg_id, &mut store, &mut grid, &mut roads, &mut budget);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not enough money");
    }

    #[test]
    fn test_upgrade_cost_correct() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();
        let initial_treasury = 10000.0;
        let mut budget = CityBudget {
            treasury: initial_treasury,
            ..Default::default()
        };

        let from = bevy::math::Vec2::new(128.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let to = bevy::math::Vec2::new(132.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let (seg_id, _) =
            store.add_straight_segment(from, to, RoadType::Local, 24.0, &mut grid, &mut roads);

        let cell_count = store.get_segment(seg_id).unwrap().rasterized_cells.len();
        let expected_cost = (RoadType::Avenue.cost() - RoadType::Local.cost()) * cell_count as f64;

        let result = upgrade_segment(seg_id, &mut store, &mut grid, &mut roads, &mut budget);
        assert!(result.is_ok());
        assert!((budget.treasury - (initial_treasury - expected_cost)).abs() < 0.01);
    }

    #[test]
    fn test_upgrade_oneway_to_avenue() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();
        let mut budget = CityBudget {
            treasury: 10000.0,
            ..Default::default()
        };

        let from = bevy::math::Vec2::new(128.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let to = bevy::math::Vec2::new(132.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let (seg_id, _) =
            store.add_straight_segment(from, to, RoadType::OneWay, 24.0, &mut grid, &mut roads);

        let result = upgrade_segment(seg_id, &mut store, &mut grid, &mut roads, &mut budget);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RoadType::Avenue);
    }

    #[test]
    fn test_find_segment_near() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();

        let from = bevy::math::Vec2::new(128.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let to = bevy::math::Vec2::new(132.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let (seg_id, _) =
            store.add_straight_segment(from, to, RoadType::Local, 24.0, &mut grid, &mut roads);

        // Point very close to the segment midpoint
        let midpoint = bevy::math::Vec2::new(130.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let found = find_segment_near(midpoint, &store, 32.0);
        assert_eq!(found, Some(seg_id));

        // Point far away
        let far = bevy::math::Vec2::new(10.0 * CELL_SIZE, 10.0 * CELL_SIZE);
        let found = find_segment_near(far, &store, 32.0);
        assert_eq!(found, None);
    }

    #[test]
    fn test_sequential_upgrades() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();
        let mut budget = CityBudget {
            treasury: 100000.0,
            ..Default::default()
        };

        let from = bevy::math::Vec2::new(128.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let to = bevy::math::Vec2::new(132.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let (seg_id, _) =
            store.add_straight_segment(from, to, RoadType::Path, 24.0, &mut grid, &mut roads);

        // Path -> Local
        let result = upgrade_segment(seg_id, &mut store, &mut grid, &mut roads, &mut budget);
        assert_eq!(result.unwrap(), RoadType::Local);

        // Local -> Avenue
        let result = upgrade_segment(seg_id, &mut store, &mut grid, &mut roads, &mut budget);
        assert_eq!(result.unwrap(), RoadType::Avenue);

        // Avenue -> Boulevard
        let result = upgrade_segment(seg_id, &mut store, &mut grid, &mut roads, &mut budget);
        assert_eq!(result.unwrap(), RoadType::Boulevard);

        // Boulevard -> should fail
        let result = upgrade_segment(seg_id, &mut store, &mut grid, &mut roads, &mut budget);
        assert!(result.is_err());
    }

    #[test]
    fn test_upgrade_preserves_connections() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();
        let mut store = RoadSegmentStore::default();
        let mut budget = CityBudget {
            treasury: 100000.0,
            ..Default::default()
        };

        // Create two connected segments sharing a node
        let a = bevy::math::Vec2::new(128.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let b = bevy::math::Vec2::new(132.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);
        let c = bevy::math::Vec2::new(136.0 * CELL_SIZE + 8.0, 128.0 * CELL_SIZE + 8.0);

        let (seg1_id, _) =
            store.add_straight_segment(a, b, RoadType::Local, 24.0, &mut grid, &mut roads);
        let (seg2_id, _) =
            store.add_straight_segment(b, c, RoadType::Local, 24.0, &mut grid, &mut roads);

        // Upgrade first segment
        let result = upgrade_segment(seg1_id, &mut store, &mut grid, &mut roads, &mut budget);
        assert_eq!(result.unwrap(), RoadType::Avenue);

        // Verify second segment is still intact
        let seg2 = store.get_segment(seg2_id).unwrap();
        assert_eq!(seg2.road_type, RoadType::Local);

        // Verify shared node still connects both segments
        let seg1 = store.get_segment(seg1_id).unwrap();
        let shared_node_id = seg1.end_node;
        let shared_node = store.get_node(shared_node_id).unwrap();
        assert!(shared_node.connected_segments.contains(&seg1_id));
        assert!(shared_node.connected_segments.contains(&seg2_id));
    }
}
