use fastnoise_lite::{FastNoiseLite, NoiseType};

use crate::config::WATER_THRESHOLD;
use crate::grid::{CellType, WorldGrid};

pub fn generate_terrain(grid: &mut WorldGrid, seed: i32) {
    let mut noise = FastNoiseLite::with_seed(seed);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise.set_frequency(Some(0.008));

    for y in 0..grid.height {
        for x in 0..grid.width {
            let raw = noise.get_noise_2d(x as f32, y as f32);
            let elevation = (raw + 1.0) * 0.5; // normalize to 0..1
            let cell = grid.get_mut(x, y);
            cell.elevation = elevation;
            if elevation < WATER_THRESHOLD {
                cell.cell_type = CellType::Water;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    #[test]
    fn test_elevation_bounds() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        generate_terrain(&mut grid, 42);
        for cell in &grid.cells {
            assert!(cell.elevation >= 0.0 && cell.elevation <= 1.0,
                "elevation {} out of bounds", cell.elevation);
        }
    }

    #[test]
    fn test_water_generation() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        generate_terrain(&mut grid, 42);
        let water_count = grid.cells.iter()
            .filter(|c| c.cell_type == CellType::Water)
            .count();
        // Should have some water but not all water
        assert!(water_count > 0, "should have some water cells");
        assert!(water_count < GRID_WIDTH * GRID_HEIGHT, "should not be all water");
    }

    #[test]
    fn test_deterministic() {
        let mut g1 = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut g2 = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        generate_terrain(&mut g1, 42);
        generate_terrain(&mut g2, 42);
        for (a, b) in g1.cells.iter().zip(g2.cells.iter()) {
            assert_eq!(a.elevation, b.elevation);
            assert_eq!(a.cell_type, b.cell_type);
        }
    }
}
