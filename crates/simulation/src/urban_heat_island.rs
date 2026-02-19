use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::time_of_day::GameClock;
use crate::trees::TreeGrid;
use crate::weather::Weather;
use crate::TickCounter;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// UHI update frequency in simulation ticks.
const UHI_UPDATE_INTERVAL: u64 = 30;

/// Rural baseline green fraction (fraction of cells that are vegetated in
/// undeveloped areas).  The deficit between actual local green fraction and
/// this baseline drives the vegetation-deficit UHI contribution.
const RURAL_GREEN_BASELINE: f32 = 0.6;

/// Maximum vegetation-deficit contribution in degrees Fahrenheit.
const VEGETATION_DEFICIT_SCALE: f32 = 8.0;

/// Canyon-effect scale: building levels (stories) above 4 contribute to UHI
/// proportional to a height-to-width ratio approximation.
const CANYON_STORIES_THRESHOLD: u8 = 4;
const CANYON_EFFECT_SCALE: f32 = 1.5;

/// Nighttime amplification factor (UHI is doubled at night).
const NIGHTTIME_AMPLIFICATION: f32 = 2.0;

/// Hours considered nighttime for UHI amplification.
/// Night: 20:00 - 05:59 (inclusive).
const NIGHT_START_HOUR: u32 = 20;
const NIGHT_END_HOUR: u32 = 5;

// ---------------------------------------------------------------------------
// Surface heat factors (Fahrenheit)
// ---------------------------------------------------------------------------

/// Asphalt / dark roof surface heat factor.
const SURFACE_ASPHALT: f32 = 2.0;
/// Concrete surface heat factor.
const SURFACE_CONCRETE: f32 = 1.5;
/// Light roof surface heat factor.
const SURFACE_LIGHT_ROOF: f32 = 0.5;
/// Water surface heat factor (strong cooling).
const SURFACE_WATER: f32 = -2.0;
/// Vegetation surface heat factor (cooling).
const SURFACE_VEGETATION: f32 = -1.5;

// ---------------------------------------------------------------------------
// UhiGrid resource
// ---------------------------------------------------------------------------

/// Per-cell temperature increment grid (in Fahrenheit). A positive value means
/// the cell is warmer than the rural baseline; negative values indicate cooling
/// (e.g. parks, water).
///
/// The final effective temperature for any cell is:
///   `base_weather_temperature + uhi_grid.cells[idx]`
#[derive(Resource, Serialize, Deserialize)]
pub struct UhiGrid {
    pub cells: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl Default for UhiGrid {
    fn default() -> Self {
        Self {
            cells: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl UhiGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x]
        } else {
            0.0
        }
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = val;
        }
    }

    /// Compute the effective temperature at a specific cell by adding the UHI
    /// increment to the base weather temperature.
    pub fn effective_temperature(&self, base_temp: f32, x: usize, y: usize) -> f32 {
        base_temp + self.get(x, y)
    }
}

// ---------------------------------------------------------------------------
// Helper: effective temperature (standalone for external callers)
// ---------------------------------------------------------------------------

/// Convenience function returning the final cell temperature given the base
/// weather temperature and the UHI grid value at `(x, y)`.
pub fn effective_temperature(uhi: &UhiGrid, base_temp: f32, x: usize, y: usize) -> f32 {
    uhi.effective_temperature(base_temp, x, y)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `true` when the given hour of day is considered nighttime for UHI
/// amplification purposes.
fn is_nighttime(hour: u32) -> bool {
    hour >= NIGHT_START_HOUR || hour <= NIGHT_END_HOUR
}

/// Compute the surface heat factor for a cell based on its type, zone, and
/// tree coverage.
fn surface_heat_factor(cell_type: CellType, zone: ZoneType, has_tree: bool) -> f32 {
    match cell_type {
        CellType::Water => SURFACE_WATER,
        CellType::Road => SURFACE_ASPHALT,
        CellType::Grass => {
            if has_tree {
                SURFACE_VEGETATION
            } else {
                match zone {
                    // Buildings with different roof types based on zone density.
                    ZoneType::Industrial => SURFACE_ASPHALT, // dark roofs
                    ZoneType::ResidentialHigh
                    | ZoneType::CommercialHigh
                    | ZoneType::Office
                    | ZoneType::MixedUse => SURFACE_CONCRETE, // concrete/mixed
                    ZoneType::ResidentialLow | ZoneType::CommercialLow => SURFACE_LIGHT_ROOF,
                    ZoneType::ResidentialMedium => SURFACE_CONCRETE,
                    ZoneType::None => {
                        // Undeveloped grass -- slightly negative (vegetation)
                        SURFACE_VEGETATION
                    }
                }
            }
        }
    }
}

/// Compute the local green fraction in a 5x5 neighborhood centered on `(cx, cy)`.
/// Green cells include trees and undeveloped grass (no building, no road).
fn local_green_fraction(grid: &WorldGrid, tree_grid: &TreeGrid, cx: usize, cy: usize) -> f32 {
    let mut green_count: u32 = 0;
    let mut total: u32 = 0;

    let radius = 2i32; // 5x5 neighbourhood
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                continue;
            }
            let ux = nx as usize;
            let uy = ny as usize;
            total += 1;

            let cell = grid.get(ux, uy);
            if tree_grid.has_tree(ux, uy)
                || (cell.cell_type == CellType::Grass
                    && cell.zone == ZoneType::None
                    && cell.building_id.is_none())
                || cell.cell_type == CellType::Water
            {
                green_count += 1;
            }
        }
    }

    if total == 0 {
        0.0
    } else {
        green_count as f32 / total as f32
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Recomputes the UHI grid every `UHI_UPDATE_INTERVAL` ticks.
///
/// Contributions per cell:
/// 1. **Surface type** -- roads=asphalt, industrial=dark roof, etc.
/// 2. **Vegetation deficit** -- comparison of local green fraction to rural
///    baseline; deficit * 8.0 F.
/// 3. **Waste heat** -- proportional to building occupancy (energy demand proxy).
/// 4. **Canyon effect** -- buildings > 4 stories add `(levels - 4) * 1.5 F`.
/// 5. **Nighttime amplification** -- UHI *= 2.0 between 20:00 and 05:59.
/// 6. **3x3 smoothing** -- averages each cell with its 3x3 neighbors.
#[allow(clippy::too_many_arguments)]
pub fn update_uhi_grid(
    tick: Res<TickCounter>,
    mut uhi: ResMut<UhiGrid>,
    grid: Res<WorldGrid>,
    tree_grid: Res<TreeGrid>,
    clock: Res<GameClock>,
    _weather: Res<Weather>,
    buildings: Query<&Building>,
) {
    if !tick.0.is_multiple_of(UHI_UPDATE_INTERVAL) {
        return;
    }

    let width = GRID_WIDTH;
    let height = GRID_HEIGHT;
    let total = width * height;

    // --- Phase 0: Build a lookup of building levels per cell ---
    // Also accumulate per-cell occupants for waste-heat contribution.
    let mut building_levels: Vec<u8> = vec![0; total];
    let mut building_occupants: Vec<u32> = vec![0; total];
    for building in &buildings {
        let bx = building.grid_x;
        let by = building.grid_y;
        if bx < width && by < height {
            let idx = by * width + bx;
            building_levels[idx] = building.level;
            building_occupants[idx] = building.occupants;
        }
    }

    // --- Phase 1: Raw UHI contribution per cell ---
    let mut raw: Vec<f32> = vec![0.0; total];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let cell = grid.get(x, y);
            let has_tree = tree_grid.has_tree(x, y);

            // 1. Surface heat factor
            let surface = if cell.building_id.is_some() {
                // Cell has a building -- use building zone for roof type
                surface_heat_factor(CellType::Grass, cell.zone, has_tree)
            } else {
                surface_heat_factor(cell.cell_type, cell.zone, has_tree)
            };

            // 2. Vegetation deficit
            let green_frac = local_green_fraction(&grid, &tree_grid, x, y);
            let veg_deficit = (RURAL_GREEN_BASELINE - green_frac).max(0.0);
            let veg_contribution = veg_deficit * VEGETATION_DEFICIT_SCALE;

            // 3. Waste heat (proportional to occupancy as energy demand proxy)
            // Each 100 occupants contributes ~0.5 F.
            let waste_heat = building_occupants[idx] as f32 * 0.005;

            // 4. Canyon effect
            let levels = building_levels[idx];
            let canyon = if levels > CANYON_STORIES_THRESHOLD {
                (levels - CANYON_STORIES_THRESHOLD) as f32 * CANYON_EFFECT_SCALE
            } else {
                0.0
            };

            raw[idx] = surface + veg_contribution + waste_heat + canyon;
        }
    }

    // --- Phase 2: Nighttime amplification ---
    let hour = clock.hour_of_day();
    if is_nighttime(hour) {
        for val in raw.iter_mut() {
            // Only amplify positive (warming) contributions; negative values
            // (water, vegetation) stay as-is to preserve cooling at night.
            if *val > 0.0 {
                *val *= NIGHTTIME_AMPLIFICATION;
            }
        }
    }

    // --- Phase 3: 3x3 smoothing ---
    let mut smoothed: Vec<f32> = vec![0.0; total];
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

    uhi.cells = smoothed;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Cell, CellType, WorldGrid, ZoneType};

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
