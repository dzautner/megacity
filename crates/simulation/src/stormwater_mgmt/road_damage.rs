//! Flood damage to roads.
//!
//! When flood depth exceeds a threshold on road cells, the road condition
//! degrades. Prolonged flooding causes cumulative damage.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::flood_simulation::FloodGrid;
use crate::grid::{CellType, WorldGrid};
use crate::road_maintenance::RoadConditionGrid;

/// Minimum flood depth (feet) that causes road damage.
pub(crate) const ROAD_DAMAGE_DEPTH_THRESHOLD: f32 = 0.5;

/// Road condition points lost per slow tick per foot of flood depth.
/// At 2 ft depth, a road loses 6 condition points per tick (out of 255 max).
pub(crate) const ROAD_DAMAGE_PER_FOOT_PER_TICK: f32 = 3.0;

/// Maximum condition points that can be lost in a single tick.
pub(crate) const MAX_ROAD_DAMAGE_PER_TICK: u8 = 30;

/// Apply flood damage to road cells. Returns the number of road cells damaged.
pub(crate) fn apply_flood_road_damage(
    flood_grid: &FloodGrid,
    world_grid: &WorldGrid,
    road_condition: &mut RoadConditionGrid,
) -> u32 {
    let mut damaged_roads = 0u32;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if world_grid.get(x, y).cell_type != CellType::Road {
                continue;
            }

            let depth = flood_grid.get(x, y);
            if depth < ROAD_DAMAGE_DEPTH_THRESHOLD {
                continue;
            }

            let damage_raw = (depth * ROAD_DAMAGE_PER_FOOT_PER_TICK) as u8;
            let damage = damage_raw.min(MAX_ROAD_DAMAGE_PER_TICK);

            let current = road_condition.get(x, y);
            road_condition.set(x, y, current.saturating_sub(damage));
            damaged_roads += 1;
        }
    }

    damaged_roads
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    fn make_grids() -> (FloodGrid, WorldGrid, RoadConditionGrid) {
        let flood = FloodGrid::default();
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let condition = RoadConditionGrid::default();
        (flood, grid, condition)
    }

    #[test]
    fn test_no_flooding_no_damage() {
        let (flood, mut grid, mut condition) = make_grids();
        grid.get_mut(10, 10).cell_type = CellType::Road;
        condition.set(10, 10, 200);

        let damaged = apply_flood_road_damage(&flood, &grid, &mut condition);

        assert_eq!(damaged, 0);
        assert_eq!(condition.get(10, 10), 200);
    }

    #[test]
    fn test_shallow_flood_damages_road() {
        let (mut flood, mut grid, mut condition) = make_grids();
        grid.get_mut(10, 10).cell_type = CellType::Road;
        condition.set(10, 10, 200);
        flood.set(10, 10, 1.0); // 1 foot of flooding

        let damaged = apply_flood_road_damage(&flood, &grid, &mut condition);

        assert_eq!(damaged, 1);
        // 1.0 * 3.0 = 3 points of damage
        assert_eq!(condition.get(10, 10), 197);
    }

    #[test]
    fn test_deep_flood_more_damage() {
        let (mut flood, mut grid, mut condition) = make_grids();
        grid.get_mut(10, 10).cell_type = CellType::Road;
        condition.set(10, 10, 200);
        flood.set(10, 10, 5.0); // 5 feet of flooding

        let damaged = apply_flood_road_damage(&flood, &grid, &mut condition);

        assert_eq!(damaged, 1);
        // 5.0 * 3.0 = 15 points of damage
        assert_eq!(condition.get(10, 10), 185);
    }

    #[test]
    fn test_damage_capped_at_max() {
        let (mut flood, mut grid, mut condition) = make_grids();
        grid.get_mut(10, 10).cell_type = CellType::Road;
        condition.set(10, 10, 200);
        flood.set(10, 10, 20.0); // Very deep flooding

        apply_flood_road_damage(&flood, &grid, &mut condition);

        // 20.0 * 3.0 = 60, capped at MAX_ROAD_DAMAGE_PER_TICK = 30
        assert_eq!(condition.get(10, 10), 170);
    }

    #[test]
    fn test_condition_cannot_go_below_zero() {
        let (mut flood, mut grid, mut condition) = make_grids();
        grid.get_mut(10, 10).cell_type = CellType::Road;
        condition.set(10, 10, 5);
        flood.set(10, 10, 10.0);

        apply_flood_road_damage(&flood, &grid, &mut condition);

        // saturating_sub prevents underflow
        assert_eq!(condition.get(10, 10), 0);
    }

    #[test]
    fn test_non_road_cells_unaffected() {
        let (mut flood, grid, mut condition) = make_grids();
        // Grass cell (default)
        condition.set(10, 10, 200);
        flood.set(10, 10, 5.0);

        let damaged = apply_flood_road_damage(&flood, &grid, &mut condition);

        assert_eq!(damaged, 0);
        assert_eq!(condition.get(10, 10), 200);
    }

    #[test]
    fn test_below_threshold_no_damage() {
        let (mut flood, mut grid, mut condition) = make_grids();
        grid.get_mut(10, 10).cell_type = CellType::Road;
        condition.set(10, 10, 200);
        flood.set(10, 10, 0.3); // Below threshold

        let damaged = apply_flood_road_damage(&flood, &grid, &mut condition);

        assert_eq!(damaged, 0);
        assert_eq!(condition.get(10, 10), 200);
    }
}
