use bevy::prelude::*;

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::time_of_day::GameClock;
use crate::trees::TreeGrid;
use crate::weather::Weather;
use crate::TickCounter;

use super::calculations::{is_nighttime, local_green_fraction, surface_heat_factor};
use super::constants::*;
use super::types::UhiGrid;

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
// Plugin
// ---------------------------------------------------------------------------

pub struct UrbanHeatIslandPlugin;

impl Plugin for UrbanHeatIslandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UhiGrid>().add_systems(
            FixedUpdate,
            update_uhi_grid
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
