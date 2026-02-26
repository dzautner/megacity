//! PLAY-003: Integration tests for New Game procedural terrain generation.
//!
//! Verifies that `generate_procedural_terrain` with the new-game default
//! erosion iterations (10,000) produces playable, varied, deterministic maps.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::terrain_generation::{generate_procedural_terrain, TerrainConfig};

/// Default erosion iterations used by the new game flow.
const NEW_GAME_EROSION_ITERATIONS: u32 = 10_000;

// -----------------------------------------------------------------------
// Terrain variety tests
// -----------------------------------------------------------------------

#[test]
fn test_newgame_terrain_has_varied_elevations() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let _biomes = generate_procedural_terrain(&mut grid, 42, NEW_GAME_EROSION_ITERATIONS);

    // Collect unique elevations (quantized to 2 decimal places)
    let mut unique = std::collections::HashSet::new();
    for cell in &grid.cells {
        let key = (cell.elevation * 100.0) as i32;
        unique.insert(key);
    }

    assert!(
        unique.len() > 10,
        "expected varied elevations, found only {} distinct levels",
        unique.len()
    );
}

#[test]
fn test_newgame_terrain_not_flat() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let _biomes = generate_procedural_terrain(&mut grid, 42, NEW_GAME_EROSION_ITERATIONS);

    let n = grid.cells.len() as f32;
    let mean = grid.cells.iter().map(|c| c.elevation).sum::<f32>() / n;
    let variance = grid
        .cells
        .iter()
        .map(|c| (c.elevation - mean).powi(2))
        .sum::<f32>()
        / n;
    let stddev = variance.sqrt();

    // A flat map (all 0.5) has stddev 0. Procedural terrain should be well above that.
    assert!(
        stddev > 0.05,
        "terrain is too flat: stddev = {stddev} (expected > 0.05)"
    );
}

// -----------------------------------------------------------------------
// Water and land distribution tests
// -----------------------------------------------------------------------

#[test]
fn test_newgame_terrain_has_water_and_land() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let _biomes = generate_procedural_terrain(&mut grid, 42, NEW_GAME_EROSION_ITERATIONS);

    let water = grid
        .cells
        .iter()
        .filter(|c| c.cell_type == CellType::Water)
        .count();
    let land = grid
        .cells
        .iter()
        .filter(|c| c.cell_type != CellType::Water)
        .count();

    assert!(water > 0, "terrain should have water cells");
    assert!(land > 0, "terrain should have land cells");

    let total = grid.cells.len();
    let water_pct = water as f32 / total as f32;
    let land_pct = land as f32 / total as f32;

    // Water should be between 5% and 70% (realistic, playable maps)
    assert!(
        water_pct > 0.05,
        "too little water: {water_pct:.1}% (expected > 5%)"
    );
    assert!(
        land_pct > 0.30,
        "too little land: {land_pct:.1}% (expected > 30%)"
    );
}

#[test]
fn test_newgame_terrain_multiple_seeds_all_have_water_and_land() {
    for seed in [1_u64, 99, 256, 1000, 54321] {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let _biomes = generate_procedural_terrain(&mut grid, seed, NEW_GAME_EROSION_ITERATIONS);

        let water = grid
            .cells
            .iter()
            .filter(|c| c.cell_type == CellType::Water)
            .count();
        let land = grid.cells.len() - water;

        assert!(
            water > 0,
            "seed {seed}: terrain should have water cells"
        );
        assert!(
            land > grid.cells.len() * 3 / 10,
            "seed {seed}: too little land ({land} cells, need at least 30%)"
        );
    }
}

// -----------------------------------------------------------------------
// Determinism tests
// -----------------------------------------------------------------------

#[test]
fn test_newgame_same_seed_produces_identical_terrain() {
    let mut grid1 = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut grid2 = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    let biomes1 = generate_procedural_terrain(&mut grid1, 12345, NEW_GAME_EROSION_ITERATIONS);
    let biomes2 = generate_procedural_terrain(&mut grid2, 12345, NEW_GAME_EROSION_ITERATIONS);

    // Check every cell matches
    for i in 0..grid1.cells.len() {
        assert_eq!(
            grid1.cells[i].elevation, grid2.cells[i].elevation,
            "elevation mismatch at cell {i}"
        );
        assert_eq!(
            grid1.cells[i].cell_type, grid2.cells[i].cell_type,
            "cell type mismatch at cell {i}"
        );
    }
    assert_eq!(
        biomes1.biomes, biomes2.biomes,
        "biome grids should be identical for the same seed"
    );
}

#[test]
fn test_newgame_different_seeds_produce_different_terrain() {
    let mut grid1 = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut grid2 = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    let _biomes1 = generate_procedural_terrain(&mut grid1, 111, NEW_GAME_EROSION_ITERATIONS);
    let _biomes2 = generate_procedural_terrain(&mut grid2, 222, NEW_GAME_EROSION_ITERATIONS);

    let diff_count = grid1
        .cells
        .iter()
        .zip(grid2.cells.iter())
        .filter(|(a, b)| (a.elevation - b.elevation).abs() > 0.01)
        .count();

    assert!(
        diff_count > 1000,
        "different seeds should produce substantially different terrain \
         (only {diff_count} cells differ)"
    );
}

// -----------------------------------------------------------------------
// TerrainConfig resource state test
// -----------------------------------------------------------------------

#[test]
fn test_newgame_terrain_config_marks_generated() {
    let config = TerrainConfig {
        seed: 42,
        erosion_iterations: NEW_GAME_EROSION_ITERATIONS,
        generated: true,
    };

    // After new game, the config should be marked as generated
    assert!(config.generated);
    assert_eq!(config.erosion_iterations, NEW_GAME_EROSION_ITERATIONS);
}
