//! Intersection lamp posts -- spawns additional lamp posts at road intersections
//! (cells where 3+ road neighbours meet).

use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::grid::{CellType, WorldGrid};

use crate::building_meshes::BuildingModelCache;
use crate::props::{PropEntity, PropsSpawned, StreetLamp};

// =============================================================================
// Constants
// =============================================================================

/// Scale for intersection lamp models.
const INTERSECTION_LAMP_SCALE: f32 = 1.8;

// =============================================================================
// Components
// =============================================================================

/// Marker for intersection lamp post entities (separate from edge lamps).
#[derive(Component)]
pub struct IntersectionLamp;

// =============================================================================
// Resources
// =============================================================================

/// Tracks whether intersection lamps have been spawned (one-shot, like `PropsSpawned`).
#[derive(Resource, Default)]
pub struct IntersectionLampsSpawned(pub bool);

// =============================================================================
// Pure helper functions
// =============================================================================

/// Count how many orthogonal road-type neighbours a cell has.
/// A cell is an "intersection" if it has 3 or more road neighbours.
pub fn road_neighbour_count(grid: &WorldGrid, gx: usize, gy: usize) -> usize {
    let width = grid.width;
    let height = grid.height;
    let neighbours = [
        (gx.wrapping_sub(1), gy),
        (gx + 1, gy),
        (gx, gy.wrapping_sub(1)),
        (gx, gy + 1),
    ];
    neighbours
        .iter()
        .filter(|&&(nx, ny)| {
            nx < width && ny < height && grid.get(nx, ny).cell_type == CellType::Road
        })
        .count()
}

/// Returns true if the cell at (gx, gy) is a road intersection (3+ road neighbours).
pub fn is_intersection(grid: &WorldGrid, gx: usize, gy: usize) -> bool {
    road_neighbour_count(grid, gx, gy) >= 3
}

// =============================================================================
// Systems
// =============================================================================

/// Spawn lamp posts at road intersections. Runs once after the grid is ready.
pub fn spawn_intersection_lamps(
    mut commands: Commands,
    model_cache: Res<BuildingModelCache>,
    grid: Res<WorldGrid>,
    mut spawned: ResMut<IntersectionLampsSpawned>,
    props_spawned: Res<PropsSpawned>,
) {
    // Wait until the base props system has run so the grid is populated.
    if spawned.0 || !props_spawned.lamps_spawned || model_cache.props.is_empty() {
        return;
    }
    spawned.0 = true;

    let width = grid.width;
    let height = grid.height;

    for gy in 1..height.saturating_sub(1) {
        for gx in 1..width.saturating_sub(1) {
            let cell = grid.get(gx, gy);
            if cell.cell_type != CellType::Road {
                continue;
            }

            if !is_intersection(&grid, gx, gy) {
                continue;
            }

            // Deterministic hash to avoid placing a lamp at every single intersection
            let hash = gx.wrapping_mul(47).wrapping_add(gy.wrapping_mul(61)) % 100;
            if hash >= 80 {
                continue; // ~80% of intersections get a lamp
            }

            let (wx, _) = WorldGrid::grid_to_world(gx, gy);
            let wz = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;

            // Place the lamp at a slight offset toward the corner of the intersection.
            let off_x = if hash % 2 == 0 {
                CELL_SIZE * 0.35
            } else {
                -CELL_SIZE * 0.35
            };
            let off_z = if (hash / 10) % 2 == 0 {
                CELL_SIZE * 0.35
            } else {
                -CELL_SIZE * 0.35
            };

            // Prefer the double-light for intersections (index 2 in props if available).
            let scene_handle = if model_cache.props.len() > 2 {
                model_cache.props[2].clone() // detail-light-double
            } else {
                model_cache.get_prop(hash)
            };

            commands.spawn((
                PropEntity,
                StreetLamp,
                IntersectionLamp,
                SceneRoot(scene_handle),
                Transform::from_xyz(wx + off_x, 0.0, wz + off_z)
                    .with_scale(Vec3::splat(INTERSECTION_LAMP_SCALE)),
                Visibility::default(),
            ));
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::config::{GRID_HEIGHT, GRID_WIDTH};

    #[test]
    fn test_road_neighbour_count_no_roads() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert_eq!(road_neighbour_count(&grid, 5, 5), 0);
    }

    #[test]
    fn test_road_neighbour_count_single_road() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(5, 5).cell_type = CellType::Road;
        assert_eq!(road_neighbour_count(&grid, 5, 5), 0);
    }

    #[test]
    fn test_road_neighbour_count_cross() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Road;
        grid.get_mut(9, 10).cell_type = CellType::Road;
        grid.get_mut(11, 10).cell_type = CellType::Road;
        grid.get_mut(10, 9).cell_type = CellType::Road;
        grid.get_mut(10, 11).cell_type = CellType::Road;
        assert_eq!(road_neighbour_count(&grid, 10, 10), 4);
    }

    #[test]
    fn test_road_neighbour_count_t_junction() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Road;
        grid.get_mut(9, 10).cell_type = CellType::Road;
        grid.get_mut(11, 10).cell_type = CellType::Road;
        grid.get_mut(10, 11).cell_type = CellType::Road;
        assert_eq!(road_neighbour_count(&grid, 10, 10), 3);
    }

    #[test]
    fn test_is_intersection_cross() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Road;
        grid.get_mut(9, 10).cell_type = CellType::Road;
        grid.get_mut(11, 10).cell_type = CellType::Road;
        grid.get_mut(10, 9).cell_type = CellType::Road;
        grid.get_mut(10, 11).cell_type = CellType::Road;
        assert!(is_intersection(&grid, 10, 10));
    }

    #[test]
    fn test_is_intersection_straight_road_not_intersection() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Road;
        grid.get_mut(9, 10).cell_type = CellType::Road;
        grid.get_mut(11, 10).cell_type = CellType::Road;
        assert!(!is_intersection(&grid, 10, 10));
    }

    #[test]
    fn test_is_intersection_corner_not_intersection() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Road;
        grid.get_mut(11, 10).cell_type = CellType::Road;
        grid.get_mut(10, 11).cell_type = CellType::Road;
        assert!(!is_intersection(&grid, 10, 10));
    }

    #[test]
    fn test_road_neighbour_count_at_grid_edge() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(0, 0).cell_type = CellType::Road;
        grid.get_mut(1, 0).cell_type = CellType::Road;
        assert_eq!(road_neighbour_count(&grid, 0, 0), 1);
    }

    #[test]
    fn test_road_neighbour_count_at_max_edge() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let x = GRID_WIDTH - 1;
        let y = GRID_HEIGHT - 1;
        grid.get_mut(x, y).cell_type = CellType::Road;
        grid.get_mut(x - 1, y).cell_type = CellType::Road;
        grid.get_mut(x, y - 1).cell_type = CellType::Road;
        assert_eq!(road_neighbour_count(&grid, x, y), 2);
    }

    #[test]
    fn test_intersection_lamp_scale_positive() {
        assert!(INTERSECTION_LAMP_SCALE > 0.0);
    }
}
