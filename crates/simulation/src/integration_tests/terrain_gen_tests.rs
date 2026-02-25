//! REND-002: Integration tests for procedural terrain generation.
//!
//! Tests the full terrain generation pipeline including fBm noise,
//! hydraulic erosion, river generation, and biome assignment.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::terrain_generation::{
    classify_biome, generate_procedural_terrain, Biome, BiomeGrid, TerrainConfig,
};
use crate::Saveable;

#[test]
fn test_terrain_gen_full_grid_has_varied_elevation() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let _biomes = generate_procedural_terrain(&mut grid, 42, 10_000);

    let n = grid.cells.len() as f32;
    let mean = grid.cells.iter().map(|c| c.elevation).sum::<f32>() / n;
    let variance = grid
        .cells
        .iter()
        .map(|c| (c.elevation - mean).powi(2))
        .sum::<f32>()
        / n;
    let stddev = variance.sqrt();

    assert!(
        stddev > 0.01,
        "elevation stddev {stddev} too low (terrain is flat)"
    );
    assert!(
        stddev < 0.4,
        "elevation stddev {stddev} too high (terrain is noisy)"
    );
}

#[test]
fn test_terrain_gen_elevations_in_bounds() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let _biomes = generate_procedural_terrain(&mut grid, 42, 10_000);
    for cell in &grid.cells {
        assert!(
            (0.0..=1.0).contains(&cell.elevation),
            "elevation {} out of [0,1]",
            cell.elevation
        );
    }
}

#[test]
fn test_terrain_gen_rivers_create_water_paths() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let _biomes = generate_procedural_terrain(&mut grid, 42, 10_000);

    let water_count = grid
        .cells
        .iter()
        .filter(|c| c.cell_type == CellType::Water)
        .count();
    let total = grid.cells.len();

    // Should have some water (at least 5%)
    assert!(
        water_count > total / 20,
        "too few water cells: {water_count}/{total}"
    );
    // Should have plenty of land for building (at least 30%)
    let land_count = total - water_count;
    assert!(
        land_count > total * 3 / 10,
        "too few land cells: {land_count}/{total}"
    );
}

#[test]
fn test_terrain_gen_biomes_include_multiple_types() {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let biomes = generate_procedural_terrain(&mut grid, 42, 10_000);

    let mut has = [false; 7];
    for &b in &biomes.biomes {
        match b {
            Biome::DeepWater => has[0] = true,
            Biome::ShallowWater => has[1] = true,
            Biome::Beach => has[2] = true,
            Biome::Grassland => has[3] = true,
            Biome::Forest => has[4] = true,
            Biome::Highland => has[5] = true,
            Biome::Mountain => has[6] = true,
        }
    }
    let count = has.iter().filter(|&&b| b).count();
    // At least 4 distinct biome types should be present
    assert!(
        count >= 4,
        "only {count} biome types present, expected at least 4"
    );
}

#[test]
fn test_terrain_gen_seed_replay_identical() {
    let mut g1 = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut g2 = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let b1 = generate_procedural_terrain(&mut g1, 777, 5000);
    let b2 = generate_procedural_terrain(&mut g2, 777, 5000);

    for i in 0..g1.cells.len() {
        assert_eq!(
            g1.cells[i].elevation, g2.cells[i].elevation,
            "elevation mismatch at cell {i}"
        );
        assert_eq!(
            g1.cells[i].cell_type, g2.cells[i].cell_type,
            "cell type mismatch at cell {i}"
        );
    }
    assert_eq!(b1.biomes, b2.biomes, "biome grids should be identical");
}

#[test]
fn test_terrain_gen_five_seeds_all_playable() {
    // Each seed must produce a map with at least 30% buildable land
    for seed in [1_u64, 42, 100, 999, 2025] {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let _biomes = generate_procedural_terrain(&mut grid, seed, 5000);

        let land = grid
            .cells
            .iter()
            .filter(|c| c.cell_type != CellType::Water)
            .count();
        let ratio = land as f32 / grid.cells.len() as f32;
        assert!(
            ratio > 0.3,
            "seed {seed}: land ratio {ratio} too low for playable map"
        );
    }
}

#[test]
fn test_terrain_gen_five_seeds_distinct() {
    let seeds: [u64; 5] = [1, 42, 100, 999, 2025];
    let grids: Vec<Vec<f32>> = seeds
        .iter()
        .map(|&s| {
            let mut grid = WorldGrid::new(64, 64);
            generate_procedural_terrain(&mut grid, s, 1000);
            grid.cells.iter().map(|c| c.elevation).collect()
        })
        .collect();

    for i in 0..grids.len() {
        for j in (i + 1)..grids.len() {
            let diff = grids[i]
                .iter()
                .zip(grids[j].iter())
                .filter(|(a, b)| (*a - *b).abs() > 0.01)
                .count();
            assert!(
                diff > 100,
                "seeds {} and {} too similar ({} cells differ)",
                seeds[i],
                seeds[j],
                diff
            );
        }
    }
}

#[test]
fn test_terrain_config_default() {
    let cfg = TerrainConfig::default();
    assert_eq!(cfg.seed, 0);
    assert_eq!(cfg.erosion_iterations, 0);
    assert!(!cfg.generated);
}

#[test]
fn test_terrain_config_saveable_roundtrip() {
    let config = TerrainConfig {
        seed: 12345,
        erosion_iterations: 50_000,
        generated: true,
    };
    let bytes = config.save_to_bytes().unwrap();
    let restored = TerrainConfig::load_from_bytes(&bytes);
    assert_eq!(restored.seed, 12345);
    assert_eq!(restored.erosion_iterations, 50_000);
    assert!(restored.generated);
}

#[test]
fn test_biome_grid_default_and_get() {
    let bg = BiomeGrid::default();
    assert_eq!(bg.get(0, 0), Biome::Grassland);
    assert_eq!(bg.get(128, 128), Biome::Grassland);
}

#[test]
fn test_classify_biome_thresholds() {
    assert_eq!(classify_biome(0.1, 0.5), Biome::DeepWater);
    assert_eq!(classify_biome(0.3, 0.5), Biome::ShallowWater);
    assert_eq!(classify_biome(0.38, 0.3), Biome::Beach);
    assert_eq!(classify_biome(0.5, 0.3), Biome::Grassland);
    assert_eq!(classify_biome(0.5, 0.7), Biome::Forest);
    assert_eq!(classify_biome(0.75, 0.3), Biome::Highland);
    assert_eq!(classify_biome(0.9, 0.3), Biome::Mountain);
}
