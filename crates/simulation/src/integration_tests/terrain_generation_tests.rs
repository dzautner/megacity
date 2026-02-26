//! PLAY-015: Integration tests for terrain generation on new game.
//!
//! Verifies that procedural terrain generation produces expected results:
//! - Varied elevation (not flat)
//! - Water bodies present
//! - Deterministic with the same seed
//! - Different seeds produce different terrain
//! - Terrain overwrites prior cell state (simulating new game reset)

use crate::config::{GRID_HEIGHT, GRID_WIDTH, WATER_THRESHOLD};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::terrain_generation::{generate_procedural_terrain, BiomeGrid, Biome};

/// Reduced erosion iterations for fast tests (full erosion is 10_000).
const TEST_EROSION_ITERATIONS: u32 = 100;

// ---------------------------------------------------------------------------
// Helper: generate a fresh grid with procedural terrain
// ---------------------------------------------------------------------------

fn generate_test_terrain(seed: u64) -> WorldGrid {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    generate_procedural_terrain(&mut grid, seed, TEST_EROSION_ITERATIONS);
    grid
}

// ===========================================================================
// 1. Terrain has varied elevation (not flat)
// ===========================================================================

#[test]
fn test_terrain_generation_produces_varied_elevation() {
    let grid = generate_test_terrain(12345);

    // Collect unique elevation values (rounded to 2 decimal places to
    // tolerate floating-point noise while still proving variety).
    let unique_elevations: std::collections::HashSet<i32> = grid
        .cells
        .iter()
        .map(|c| (c.elevation * 100.0) as i32)
        .collect();

    // A procedurally generated 256x256 grid should have far more than 10
    // distinct elevation levels. A flat grid would have exactly 1.
    assert!(
        unique_elevations.len() > 50,
        "Expected diverse elevations, got only {} distinct values",
        unique_elevations.len()
    );

    // Verify elevation range spans a meaningful portion of [0, 1].
    let min_elev = grid
        .cells
        .iter()
        .map(|c| c.elevation)
        .fold(f32::INFINITY, f32::min);
    let max_elev = grid
        .cells
        .iter()
        .map(|c| c.elevation)
        .fold(f32::NEG_INFINITY, f32::max);
    let range = max_elev - min_elev;

    assert!(
        range > 0.3,
        "Expected elevation range > 0.3, got {range:.3} (min={min_elev:.3}, max={max_elev:.3})"
    );
}

// ===========================================================================
// 2. Terrain has water bodies
// ===========================================================================

#[test]
fn test_terrain_generation_has_water() {
    let grid = generate_test_terrain(12345);

    let water_count = grid
        .cells
        .iter()
        .filter(|c| c.cell_type == CellType::Water)
        .count();

    let total_cells = GRID_WIDTH * GRID_HEIGHT;

    // There should be a non-trivial amount of water. With WATER_THRESHOLD=0.35,
    // a meaningful fraction of cells should be water.
    assert!(
        water_count > 100,
        "Expected at least 100 water cells, got {water_count}"
    );

    // Water cells should have elevation below the threshold.
    let invalid_water = grid
        .cells
        .iter()
        .filter(|c| c.cell_type == CellType::Water && c.elevation >= WATER_THRESHOLD)
        .count();

    // River cells get forced to below threshold, so all water should be below.
    assert_eq!(
        invalid_water, 0,
        "Found {invalid_water} water cells with elevation >= WATER_THRESHOLD ({WATER_THRESHOLD})"
    );

    // Log for debugging.
    let water_pct = (water_count as f64 / total_cells as f64) * 100.0;
    eprintln!("Water coverage: {water_count}/{total_cells} cells ({water_pct:.1}%)");
}

// ===========================================================================
// 3. Deterministic with same seed
// ===========================================================================

#[test]
fn test_terrain_generation_deterministic() {
    let seed = 99999_u64;

    let grid_a = generate_test_terrain(seed);
    let grid_b = generate_test_terrain(seed);

    // Every cell must have identical elevation and cell type.
    for i in 0..grid_a.cells.len() {
        assert_eq!(
            grid_a.cells[i].elevation, grid_b.cells[i].elevation,
            "Elevation mismatch at cell index {i}: {} vs {}",
            grid_a.cells[i].elevation, grid_b.cells[i].elevation
        );
        assert_eq!(
            grid_a.cells[i].cell_type, grid_b.cells[i].cell_type,
            "CellType mismatch at cell index {i}: {:?} vs {:?}",
            grid_a.cells[i].cell_type, grid_b.cells[i].cell_type
        );
    }
}

// ===========================================================================
// 4. Different seeds produce different terrain
// ===========================================================================

#[test]
fn test_terrain_generation_different_seeds_differ() {
    let grid_a = generate_test_terrain(11111);
    let grid_b = generate_test_terrain(22222);

    // Count cells where elevation differs.
    let differing_cells = grid_a
        .cells
        .iter()
        .zip(grid_b.cells.iter())
        .filter(|(a, b)| (a.elevation - b.elevation).abs() > 0.001)
        .count();

    let total = grid_a.cells.len();
    let diff_pct = (differing_cells as f64 / total as f64) * 100.0;

    // With different seeds, the vast majority of cells should differ.
    assert!(
        differing_cells > total / 2,
        "Expected >50% of cells to differ between seeds, \
         got {differing_cells}/{total} ({diff_pct:.1}%)"
    );
}

// ===========================================================================
// 5. Terrain overwrites prior cell state (simulates new game reset)
// ===========================================================================

#[test]
fn test_terrain_generation_overwrites_existing_grid_state() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    // Pollute the grid with non-default state: set zones and road cells.
    for y in 40..60 {
        for x in 40..60 {
            grid.get_mut(x, y).zone = ZoneType::ResidentialLow;
            grid.get_mut(x, y).cell_type = CellType::Road;
            grid.get_mut(x, y).elevation = 0.99;
        }
    }

    // Verify our setup took effect.
    assert_eq!(grid.get(50, 50).cell_type, CellType::Road);
    assert_eq!(grid.get(50, 50).zone, ZoneType::ResidentialLow);

    // Generate terrain over the existing grid.
    generate_procedural_terrain(&mut grid, 42, TEST_EROSION_ITERATIONS);

    // After generation, all cells should be either Grass or Water -- no
    // Road cells should remain because terrain gen overwrites cell_type.
    let road_count = grid
        .cells
        .iter()
        .filter(|c| c.cell_type == CellType::Road)
        .count();

    assert_eq!(
        road_count, 0,
        "Expected 0 road cells after terrain generation, got {road_count}"
    );

    // Elevation should be set by the noise function, not the hardcoded 0.99.
    // At least some of the previously-modified cells should have different
    // elevation now.
    let mut overwritten = 0;
    for y in 40..60 {
        for x in 40..60 {
            if (grid.get(x, y).elevation - 0.99).abs() > 0.01 {
                overwritten += 1;
            }
        }
    }

    assert!(
        overwritten > 100,
        "Expected terrain gen to overwrite most cells' elevation, \
         but only {overwritten}/400 cells changed from 0.99"
    );
}

// ===========================================================================
// 6. Biome grid is populated correctly
// ===========================================================================

#[test]
fn test_terrain_generation_produces_valid_biome_grid() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let biome_grid =
        generate_procedural_terrain(&mut grid, 54321, TEST_EROSION_ITERATIONS);

    assert_eq!(biome_grid.width, GRID_WIDTH);
    assert_eq!(biome_grid.height, GRID_HEIGHT);
    assert_eq!(biome_grid.biomes.len(), GRID_WIDTH * GRID_HEIGHT);

    // Water cells should have water biomes.
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            let biome = biome_grid.get(x, y);
            if cell.cell_type == CellType::Water {
                assert!(
                    matches!(biome, Biome::DeepWater | Biome::ShallowWater),
                    "Water cell at ({x},{y}) has non-water biome {biome:?}"
                );
            }
        }
    }

    // There should be multiple biome types present (not all one biome).
    let unique_biomes: std::collections::HashSet<Biome> =
        biome_grid.biomes.iter().copied().collect();
    assert!(
        unique_biomes.len() >= 3,
        "Expected at least 3 distinct biomes, got {}",
        unique_biomes.len()
    );
}
