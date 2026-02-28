//! Integration tests for procedural terrain generation.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::procedural_terrain::generate_terrain;

fn count_water(grid: &WorldGrid) -> usize {
    (0..GRID_HEIGHT)
        .flat_map(|y| (0..GRID_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| grid.get(x, y).cell_type == CellType::Water)
        .count()
}

#[test]
fn test_terrain_deterministic() {
    let mut grid1 = WorldGrid::default();
    let mut grid2 = WorldGrid::default();
    generate_terrain(&mut grid1, 42);
    generate_terrain(&mut grid2, 42);
    let water1 = count_water(&grid1);
    let water2 = count_water(&grid2);
    assert_eq!(water1, water2);
    assert!(water1 > 0, "should have some water");
}

#[test]
fn test_terrain_different_seeds_differ() {
    let mut grid1 = WorldGrid::default();
    let mut grid2 = WorldGrid::default();
    generate_terrain(&mut grid1, 42);
    generate_terrain(&mut grid2, 99);
    let water1 = count_water(&grid1);
    let water2 = count_water(&grid2);
    assert_ne!(water1, water2);
}

#[test]
fn test_terrain_has_reasonable_water_ratio() {
    let mut grid = WorldGrid::default();
    generate_terrain(&mut grid, 12345);
    let water = count_water(&grid);
    let total = GRID_WIDTH * GRID_HEIGHT;
    let ratio = water as f64 / total as f64;
    assert!(
        ratio > 0.05,
        "should have at least 5% water: got {:.2}%",
        ratio * 100.0
    );
    assert!(
        ratio < 0.50,
        "should have at most 50% water: got {:.2}%",
        ratio * 100.0
    );
}

#[test]
fn test_terrain_water_cells_have_low_elevation() {
    let mut grid = WorldGrid::default();
    generate_terrain(&mut grid, 777);
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Water {
                assert!(
                    cell.elevation < 0.35,
                    "water cell ({x},{y}) has elevation {} >= 0.35",
                    cell.elevation
                );
            }
        }
    }
}

#[test]
fn test_terrain_many_seeds_produce_water() {
    // Every seed should produce some water (coastline is always generated)
    for seed in 0..20 {
        let mut grid = WorldGrid::default();
        generate_terrain(&mut grid, seed);
        let water = count_water(&grid);
        assert!(water > 0, "seed {seed} produced no water");
    }
}
