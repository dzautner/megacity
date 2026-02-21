//! Auto-Grid Road Placement Tool (TRAF-010)
//!
//! Generates a grid of roads within a player-defined rectangular area.
//! The player defines two corners, chooses block size and road type,
//! and the tool auto-generates horizontal and vertical road segments.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::grid::{CellType, RoadType, WorldGrid};
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;

/// A grid-cell coordinate pair.
type GridPos = (usize, usize);

/// Return type for line-scan functions: (segments, new_cell_count).
type ScanResult = (Vec<(GridPos, GridPos)>, usize);

/// Configurable block size range (cells between roads).
pub const MIN_BLOCK_SIZE: u8 = 4;
pub const MAX_BLOCK_SIZE: u8 = 8;
pub const DEFAULT_BLOCK_SIZE: u8 = 6;

/// Auto-grid tool configuration resource.
#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
pub struct AutoGridConfig {
    /// Number of cells between roads (4-8).
    pub block_size: u8,
    /// Road type to place.
    pub road_type: RoadType,
}

impl Default for AutoGridConfig {
    fn default() -> Self {
        Self {
            block_size: DEFAULT_BLOCK_SIZE,
            road_type: RoadType::Local,
        }
    }
}

/// State machine for the auto-grid tool's two-click rectangle definition.
#[derive(Resource, Default, Debug, Clone)]
pub struct AutoGridState {
    /// Phase of the tool interaction.
    pub phase: AutoGridPhase,
    /// First corner (grid coordinates).
    pub corner1: (usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoGridPhase {
    /// Waiting for first corner click.
    #[default]
    Idle,
    /// First corner placed, waiting for second corner.
    PlacedFirstCorner,
}

/// Result of computing the auto-grid road plan (preview or commit).
#[derive(Debug, Clone)]
pub struct AutoGridPlan {
    /// Road segments to place as (start_grid, end_grid) pairs.
    pub segments: Vec<(GridPos, GridPos)>,
    /// Total number of road cells that will be placed.
    pub total_cells: usize,
    /// Total cost.
    pub total_cost: f64,
}

/// Collect horizontal road line Y-coordinates for a given area and block size.
fn horizontal_lines(min_y: usize, max_y: usize, block: usize) -> Vec<usize> {
    let mut lines = Vec::new();
    let mut y = min_y;
    while y <= max_y {
        lines.push(y);
        if y == min_y {
            y = min_y + block + 1;
        } else {
            y += block + 1;
        }
    }
    // Add boundary at max_y if not already covered
    if max_y > min_y && !lines.contains(&max_y) {
        lines.push(max_y);
    }
    lines
}

/// Collect vertical road line X-coordinates for a given area and block size.
fn vertical_lines(min_x: usize, max_x: usize, block: usize) -> Vec<usize> {
    let mut lines = Vec::new();
    let mut x = min_x;
    while x <= max_x {
        lines.push(x);
        if x == min_x {
            x = min_x + block + 1;
        } else {
            x += block + 1;
        }
    }
    // Add boundary at max_x if not already covered
    if max_x > min_x && !lines.contains(&max_x) {
        lines.push(max_x);
    }
    lines
}

/// Scan a horizontal line of cells and break into contiguous segments of
/// placeable or existing-road cells. Returns (segments, new_cell_count).
fn scan_horizontal_line(grid: &WorldGrid, y: usize, min_x: usize, max_x: usize) -> ScanResult {
    let mut segments = Vec::new();
    let mut new_cells = 0;
    let mut run_start: Option<usize> = None;

    for x in min_x..=max_x {
        let passable = can_place_road(grid, x, y) || is_existing_road(grid, x, y);

        if passable && run_start.is_none() {
            run_start = Some(x);
        }

        let at_end = x == max_x;
        let run_broken = !passable;

        if (run_broken || at_end) && run_start.is_some() {
            let start = run_start.unwrap();
            let end = if passable { x } else { x - 1 };
            if end > start {
                segments.push(((start, y), (end, y)));
                for cx in start..=end {
                    if can_place_road(grid, cx, y) {
                        new_cells += 1;
                    }
                }
            }
            run_start = None;
        }
    }
    (segments, new_cells)
}

/// Scan a vertical line and break into contiguous segments.
fn scan_vertical_line(
    grid: &WorldGrid,
    x: usize,
    min_y: usize,
    max_y: usize,
    h_lines: &[usize],
) -> ScanResult {
    let mut segments = Vec::new();
    let mut new_cells = 0;
    let mut run_start: Option<usize> = None;

    for y in min_y..=max_y {
        let passable = can_place_road(grid, x, y) || is_existing_road(grid, x, y);

        if passable && run_start.is_none() {
            run_start = Some(y);
        }

        let at_end = y == max_y;
        let run_broken = !passable;

        if (run_broken || at_end) && run_start.is_some() {
            let start = run_start.unwrap();
            let end = if passable { y } else { y - 1 };
            if end > start {
                segments.push(((x, start), (x, end)));
                for cy in start..=end {
                    // Avoid double-counting: skip cells on horizontal lines
                    if can_place_road(grid, x, cy) && !h_lines.contains(&cy) {
                        new_cells += 1;
                    }
                }
            }
            run_start = None;
        }
    }
    (segments, new_cells)
}

/// Compute the auto-grid road plan for a given rectangle and config.
/// Returns a plan with horizontal and vertical road segments.
pub fn compute_grid_plan(
    corner1: (usize, usize),
    corner2: (usize, usize),
    config: &AutoGridConfig,
    grid: &WorldGrid,
) -> AutoGridPlan {
    let min_x = corner1.0.min(corner2.0);
    let max_x = corner1.0.max(corner2.0);
    let min_y = corner1.1.min(corner2.1);
    let max_y = corner1.1.max(corner2.1);

    let block = config.block_size as usize;
    let mut segments = Vec::new();
    let mut total_cells: usize = 0;

    let h_lines = horizontal_lines(min_y, max_y, block);
    let v_lines = vertical_lines(min_x, max_x, block);

    // Horizontal roads
    for &y in &h_lines {
        let (segs, cells) = scan_horizontal_line(grid, y, min_x, max_x);
        segments.extend(segs);
        total_cells += cells;
    }

    // Vertical roads
    for &x in &v_lines {
        let (segs, cells) = scan_vertical_line(grid, x, min_y, max_y, &h_lines);
        segments.extend(segs);
        total_cells += cells;
    }

    let cost_per_cell = config.road_type.cost();
    let total_cost = total_cells as f64 * cost_per_cell;

    AutoGridPlan {
        segments,
        total_cells,
        total_cost,
    }
}

/// Execute the auto-grid plan: place road segments and deduct cost.
/// Returns the list of all newly placed road cells.
pub fn execute_grid_plan(
    plan: &AutoGridPlan,
    config: &AutoGridConfig,
    segments: &mut RoadSegmentStore,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
) -> Vec<(usize, usize)> {
    let mut all_cells = Vec::new();

    for &((x0, y0), (x1, y1)) in &plan.segments {
        let (wx0, wy0) = WorldGrid::grid_to_world(x0, y0);
        let (wx1, wy1) = WorldGrid::grid_to_world(x1, y1);
        let from = bevy::math::Vec2::new(wx0, wy0);
        let to = bevy::math::Vec2::new(wx1, wy1);

        let (_seg_id, cells) =
            segments.add_straight_segment(from, to, config.road_type, 16.0, grid, roads);
        all_cells.extend(cells);
    }

    all_cells
}

/// Check if a cell can have a road placed on it.
fn can_place_road(grid: &WorldGrid, x: usize, y: usize) -> bool {
    if !grid.in_bounds(x, y) {
        return false;
    }
    let cell = grid.get(x, y);
    cell.cell_type == CellType::Grass && cell.building_id.is_none()
}

/// Check if a cell already has a road (passable for segment continuity).
fn is_existing_road(grid: &WorldGrid, x: usize, y: usize) -> bool {
    grid.in_bounds(x, y) && grid.get(x, y).cell_type == CellType::Road
}

pub struct AutoGridRoadPlugin;

impl Plugin for AutoGridRoadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutoGridConfig>()
            .init_resource::<AutoGridState>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    #[test]
    fn test_compute_grid_plan_basic() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let config = AutoGridConfig {
            block_size: 6,
            road_type: RoadType::Local,
        };
        let plan = compute_grid_plan((10, 10), (30, 30), &config, &grid);
        assert!(!plan.segments.is_empty(), "should have segments");
        assert!(plan.total_cells > 0, "should have road cells to place");
        assert!(plan.total_cost > 0.0, "should have a cost");
    }

    #[test]
    fn test_compute_grid_plan_respects_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        for x in 10..=30 {
            grid.get_mut(x, 15).cell_type = CellType::Water;
        }
        let config = AutoGridConfig {
            block_size: 4,
            road_type: RoadType::Local,
        };
        let plan = compute_grid_plan((10, 10), (30, 30), &config, &grid);
        for &((x0, y0), (x1, y1)) in &plan.segments {
            if y0 == 15 && y1 == 15 {
                assert!(x1 - x0 < 20, "segment should not span full water row");
            }
        }
    }

    #[test]
    fn test_compute_grid_plan_small_block_size() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let config = AutoGridConfig {
            block_size: MIN_BLOCK_SIZE,
            road_type: RoadType::Avenue,
        };
        let plan = compute_grid_plan((50, 50), (70, 70), &config, &grid);
        assert!(!plan.segments.is_empty());
        let config_large = AutoGridConfig {
            block_size: MAX_BLOCK_SIZE,
            road_type: RoadType::Avenue,
        };
        let plan_large = compute_grid_plan((50, 50), (70, 70), &config_large, &grid);
        assert!(
            plan.total_cells >= plan_large.total_cells,
            "smaller block size should produce more or equal road cells"
        );
    }

    #[test]
    fn test_compute_grid_plan_cost_scales_with_road_type() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let config_local = AutoGridConfig {
            block_size: 6,
            road_type: RoadType::Local,
        };
        let config_avenue = AutoGridConfig {
            block_size: 6,
            road_type: RoadType::Avenue,
        };
        let plan_local = compute_grid_plan((10, 10), (30, 30), &config_local, &grid);
        let plan_avenue = compute_grid_plan((10, 10), (30, 30), &config_avenue, &grid);
        assert_eq!(plan_local.total_cells, plan_avenue.total_cells);
        assert!(
            plan_avenue.total_cost > plan_local.total_cost,
            "avenue should cost more than local"
        );
    }

    #[test]
    fn test_can_place_road_on_grass() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert!(can_place_road(&grid, 10, 10));
    }

    #[test]
    fn test_cannot_place_road_on_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Water;
        assert!(!can_place_road(&grid, 10, 10));
    }

    #[test]
    fn test_cannot_place_road_on_existing_road() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Road;
        assert!(!can_place_road(&grid, 10, 10));
    }

    #[test]
    fn test_auto_grid_config_default() {
        let config = AutoGridConfig::default();
        assert_eq!(config.block_size, DEFAULT_BLOCK_SIZE);
        assert_eq!(config.road_type, RoadType::Local);
    }
}
// TRAF-010 auto-grid road tool
