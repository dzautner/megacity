#[cfg(test)]
mod tests {
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::grid::{CellType, WorldGrid, ZoneType};
    use crate::trees::TreeGrid;
    use crate::urban_heat_island::calculations::{
        is_nighttime, local_green_fraction, surface_heat_factor,
    };
    use crate::urban_heat_island::constants::*;
    use crate::urban_heat_island::types::{effective_temperature, UhiGrid};

    #[test]
    fn test_uhi_grid_default() {
        let grid = UhiGrid::default();
        assert_eq!(grid.cells.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!((grid.get(0, 0)).abs() < f32::EPSILON);
        assert!((grid.get(128, 128)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_uhi_grid_get_set() {
        let mut grid = UhiGrid::default();
        grid.set(10, 20, 3.5);
        assert!((grid.get(10, 20) - 3.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_uhi_grid_out_of_bounds() {
        let grid = UhiGrid::default();
        assert!((grid.get(9999, 9999)).abs() < f32::EPSILON);

        let mut grid2 = UhiGrid::default();
        grid2.set(9999, 9999, 10.0); // should not panic
    }

    #[test]
    fn test_effective_temperature() {
        let mut grid = UhiGrid::default();
        grid.set(5, 5, 4.0);
        let eff = grid.effective_temperature(70.0, 5, 5);
        assert!((eff - 74.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_effective_temperature_standalone() {
        let mut grid = UhiGrid::default();
        grid.set(3, 3, -2.0);
        let eff = effective_temperature(&grid, 70.0, 3, 3);
        assert!((eff - 68.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_nighttime() {
        assert!(is_nighttime(20));
        assert!(is_nighttime(23));
        assert!(is_nighttime(0));
        assert!(is_nighttime(3));
        assert!(is_nighttime(5));
        assert!(!is_nighttime(6));
        assert!(!is_nighttime(12));
        assert!(!is_nighttime(19));
    }

    #[test]
    fn test_surface_heat_factor_road() {
        let factor = surface_heat_factor(CellType::Road, ZoneType::None, false);
        assert!((factor - SURFACE_ASPHALT).abs() < f32::EPSILON);
    }

    #[test]
    fn test_surface_heat_factor_water() {
        let factor = surface_heat_factor(CellType::Water, ZoneType::None, false);
        assert!((factor - SURFACE_WATER).abs() < f32::EPSILON);
    }

    #[test]
    fn test_surface_heat_factor_vegetation() {
        // Undeveloped grass with tree
        let factor = surface_heat_factor(CellType::Grass, ZoneType::None, true);
        assert!((factor - SURFACE_VEGETATION).abs() < f32::EPSILON);

        // Undeveloped grass without tree (also vegetation)
        let factor2 = surface_heat_factor(CellType::Grass, ZoneType::None, false);
        assert!((factor2 - SURFACE_VEGETATION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_surface_heat_factor_industrial_dark_roof() {
        let factor = surface_heat_factor(CellType::Grass, ZoneType::Industrial, false);
        assert!((factor - SURFACE_ASPHALT).abs() < f32::EPSILON);
    }

    #[test]
    fn test_surface_heat_factor_light_roof() {
        let factor = surface_heat_factor(CellType::Grass, ZoneType::ResidentialLow, false);
        assert!((factor - SURFACE_LIGHT_ROOF).abs() < f32::EPSILON);
    }

    #[test]
    fn test_surface_heat_factor_concrete_roof() {
        let factor = surface_heat_factor(CellType::Grass, ZoneType::ResidentialHigh, false);
        assert!((factor - SURFACE_CONCRETE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_surface_heat_factor_tree_overrides_zone() {
        // A tree on an industrial cell should still count as vegetation
        let factor = surface_heat_factor(CellType::Grass, ZoneType::Industrial, true);
        assert!((factor - SURFACE_VEGETATION).abs() < f32::EPSILON);
    }

    #[test]
    fn test_local_green_fraction_all_green() {
        // An empty world grid with all grass, no buildings, no roads -> fully green
        let world = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let trees = TreeGrid::default();
        let frac = local_green_fraction(&world, &trees, 128, 128);
        assert!(
            (frac - 1.0).abs() < 0.01,
            "fully undeveloped should be ~1.0 green, got {}",
            frac
        );
    }

    #[test]
    fn test_local_green_fraction_all_road() {
        let mut world = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Fill a 5x5 area with roads
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let nx = (50i32 + dx) as usize;
                let ny = (50i32 + dy) as usize;
                world.get_mut(nx, ny).cell_type = CellType::Road;
            }
        }
        let trees = TreeGrid::default();
        let frac = local_green_fraction(&world, &trees, 50, 50);
        assert!(
            frac < 0.01,
            "all-road area should be ~0.0 green, got {}",
            frac
        );
    }

    #[test]
    fn test_local_green_fraction_mixed() {
        let mut world = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Center cell (50, 50) and neighbors: half are roads
        let mut count = 0;
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let nx = (50i32 + dx) as usize;
                let ny = (50i32 + dy) as usize;
                if count % 2 == 0 {
                    world.get_mut(nx, ny).cell_type = CellType::Road;
                }
                count += 1;
            }
        }
        let trees = TreeGrid::default();
        let frac = local_green_fraction(&world, &trees, 50, 50);
        // About half should be green (undeveloped grass)
        assert!(
            frac > 0.3 && frac < 0.7,
            "half-road area should be ~0.5 green, got {}",
            frac
        );
    }

    #[test]
    fn test_vegetation_deficit_drives_uhi() {
        // In a fully developed area (green_frac = 0.0):
        //   deficit = 0.6 - 0.0 = 0.6
        //   contribution = 0.6 * 8.0 = 4.8 F
        let deficit = (RURAL_GREEN_BASELINE - 0.0).max(0.0);
        let contribution = deficit * VEGETATION_DEFICIT_SCALE;
        assert!((contribution - 4.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vegetation_no_deficit_when_green() {
        // In a fully green area (green_frac = 1.0):
        //   deficit = (0.6 - 1.0).max(0.0) = 0.0
        let deficit = (RURAL_GREEN_BASELINE - 1.0).max(0.0);
        assert!(deficit.abs() < f32::EPSILON);
    }

    #[test]
    fn test_canyon_effect_low_building() {
        // Building with 3 stories (below threshold): no canyon effect
        let levels: u8 = 3;
        let canyon = if levels > CANYON_STORIES_THRESHOLD {
            (levels - CANYON_STORIES_THRESHOLD) as f32 * CANYON_EFFECT_SCALE
        } else {
            0.0
        };
        assert!(canyon.abs() < f32::EPSILON);
    }

    #[test]
    fn test_canyon_effect_tall_building() {
        // Building with 5 stories: (5 - 4) * 1.5 = 1.5 F
        let levels: u8 = 5;
        let canyon = if levels > CANYON_STORIES_THRESHOLD {
            (levels - CANYON_STORIES_THRESHOLD) as f32 * CANYON_EFFECT_SCALE
        } else {
            0.0
        };
        assert!((canyon - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_nighttime_amplification_positive_only() {
        // Positive value should be doubled at night
        let mut val = 3.0_f32;
        if val > 0.0 {
            val *= NIGHTTIME_AMPLIFICATION;
        }
        assert!((val - 6.0).abs() < f32::EPSILON);

        // Negative value should NOT be doubled
        let mut neg = -1.5_f32;
        if neg > 0.0 {
            neg *= NIGHTTIME_AMPLIFICATION;
        }
        assert!((neg - (-1.5)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_smoothing_uniform_field() {
        // If all raw values are the same, smoothing should not change them
        let val = 5.0_f32;
        let width = 4;
        let height = 4;
        let raw = vec![val; width * height];
        let mut smoothed = vec![0.0_f32; width * height];
        for y in 0..height {
            for x in 0..width {
                let mut sum: f32 = 0.0;
                let mut count: u32 = 0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && ny >= 0 && (nx as usize) < width && (ny as usize) < height {
                            sum += raw[ny as usize * width + nx as usize];
                            count += 1;
                        }
                    }
                }
                smoothed[y * width + x] = sum / count as f32;
            }
        }
        for v in &smoothed {
            assert!(
                (*v - val).abs() < f32::EPSILON,
                "uniform smoothing should preserve value"
            );
        }
    }

    #[test]
    fn test_smoothing_reduces_spike() {
        // A single spike at the center of a zero field should be reduced
        let width = 5;
        let height = 5;
        let mut raw = vec![0.0_f32; width * height];
        raw[2 * width + 2] = 9.0; // spike at center

        let mut smoothed = vec![0.0_f32; width * height];
        for y in 0..height {
            for x in 0..width {
                let mut sum: f32 = 0.0;
                let mut count: u32 = 0;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && ny >= 0 && (nx as usize) < width && (ny as usize) < height {
                            sum += raw[ny as usize * width + nx as usize];
                            count += 1;
                        }
                    }
                }
                smoothed[y * width + x] = sum / count as f32;
            }
        }
        // Center should be 9/9 = 1.0 (spike spread over 9 neighbors)
        assert!(
            (smoothed[2 * width + 2] - 1.0).abs() < f32::EPSILON,
            "spike should be averaged: got {}",
            smoothed[2 * width + 2]
        );
    }

    #[test]
    fn test_waste_heat_proportional_to_occupants() {
        let occupants = 200u32;
        let waste_heat = occupants as f32 * 0.005;
        assert!((waste_heat - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_road_contributes_positive_uhi() {
        // Road surface (asphalt=+2.0) in developed area with 0 green fraction:
        //   surface=2.0, veg_deficit=0.6*8.0=4.8, waste=0, canyon=0
        //   total = 6.8 before smoothing
        let surface = SURFACE_ASPHALT;
        let veg = RURAL_GREEN_BASELINE * VEGETATION_DEFICIT_SCALE;
        let total = surface + veg;
        assert!(
            total > 0.0,
            "roads in developed area should have positive UHI"
        );
    }

    #[test]
    fn test_water_contributes_negative_uhi() {
        // Water surface: -2.0, and green fraction includes water so deficit should be low
        let surface = SURFACE_WATER;
        assert!(surface < 0.0, "water should have negative (cooling) UHI");
    }

    #[test]
    fn test_uhi_grid_serialize_deserialize() {
        let mut grid = UhiGrid::default();
        grid.set(10, 10, 5.5);
        grid.set(20, 20, -1.0);

        // Round-trip through serde_json
        let json = serde_json::to_string(&grid).expect("serialize");
        let restored: UhiGrid = serde_json::from_str(&json).expect("deserialize");
        assert!((restored.get(10, 10) - 5.5).abs() < f32::EPSILON);
        assert!((restored.get(20, 20) - (-1.0)).abs() < f32::EPSILON);
    }
}
