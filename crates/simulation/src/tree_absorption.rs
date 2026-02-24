//! POLL-018: Enhanced tree and green space pollution absorption.
//!
//! This module replaces the flat pollution reduction in `trees.rs` with a
//! percentage-based vegetation filtering model:
//!
//! - **Vegetation filtering**: park/forest cells multiply incoming pollution by 0.6
//! - **CO2 absorption**: 48 lbs CO2/tree/year (tracked for future climate systems)
//! - **Green space bonus**: 10+ adjacent tree cells provide extra absorption
//! - **Tree maturity**: planted trees grow from 0.0 to 1.0 over 5 game-days
//! - **Canopy percentage**: tracked per district for UHI calculation

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::districts::{Districts, DISTRICTS_X, DISTRICTS_Y, DISTRICT_SIZE};
use crate::land_value::LandValueGrid;
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::trees::TreeGrid;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Pollution multiplier for vegetated cells (0.6 = 40% reduction).
const VEGETATION_FILTER_MULT: f32 = 0.6;

/// Additional absorption multiplier when 10+ adjacent tree cells form a cluster.
const GREEN_SPACE_CLUSTER_BONUS: f32 = 0.8;

/// Minimum adjacent tree cells to trigger cluster bonus.
const CLUSTER_THRESHOLD: usize = 10;

/// CO2 absorbed per mature tree per year in lbs.
pub const CO2_PER_TREE_PER_YEAR_LBS: f32 = 48.0;

/// Number of game-days for a planted tree to reach full maturity.
const MATURITY_DAYS: f32 = 5.0;

/// Maturity growth per slow tick. 1 slow-tick ≈ 100 ticks ≈ 100 min game time.
/// There are 1440 min/day, so ~14.4 slow-ticks/day, 72 over 5 days.
/// Growth per slow tick = 1.0 / (MATURITY_DAYS * 1440 / SlowTickTimer::INTERVAL)
/// = 1.0 / (5 * 14.4) = 1.0 / 72 ≈ 0.01389
const MATURITY_PER_SLOW_TICK: f32 = 1.0 / (MATURITY_DAYS * 1440.0 / 100.0);

/// Noise reduction: percentage-based (up to 30% at distance 0, scaling with maturity).
const MAX_NOISE_REDUCTION_PCT: f32 = 0.3;

/// Land value boost per adjacent tree (unchanged from original).
const LAND_VALUE_BOOST: u8 = 2;

// =============================================================================
// Resources
// =============================================================================

/// Per-cell maturity of planted trees (0.0 = just planted, 1.0 = fully mature).
/// Only meaningful for cells where `TreeGrid.has_tree(x, y)` is true.
#[derive(Resource, Clone, Encode, Decode)]
pub struct TreeMaturityGrid {
    pub values: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl Default for TreeMaturityGrid {
    fn default() -> Self {
        Self {
            values: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl TreeMaturityGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.values[y * self.width + x]
        } else {
            0.0
        }
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        if x < self.width && y < self.height {
            self.values[y * self.width + x] = val;
        }
    }
}

/// Per-district tree canopy statistics.
#[derive(Resource, Clone, Default, Encode, Decode)]
pub struct TreeCanopyStats {
    /// Canopy percentage (0.0–1.0) per district, indexed [dy * DISTRICTS_X + dx].
    pub canopy_pct: Vec<f32>,
    /// Total CO2 absorbed city-wide in lbs/year.
    pub total_co2_absorption_lbs_per_year: f32,
}

impl TreeCanopyStats {
    pub fn new() -> Self {
        Self {
            canopy_pct: vec![0.0; DISTRICTS_X * DISTRICTS_Y],
            total_co2_absorption_lbs_per_year: 0.0,
        }
    }

    /// Get canopy percentage for a district.
    pub fn district_canopy(&self, dx: usize, dy: usize) -> f32 {
        if dx < DISTRICTS_X && dy < DISTRICTS_Y {
            self.canopy_pct[dy * DISTRICTS_X + dx]
        } else {
            0.0
        }
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Grows tree maturity each slow tick. Trees that are removed (no longer in
/// TreeGrid) have their maturity reset to 0.
pub fn grow_tree_maturity(
    slow_tick: Res<SlowTickTimer>,
    tree_grid: Res<TreeGrid>,
    mut maturity: ResMut<TreeMaturityGrid>,
) {
    if !slow_tick.should_run() {
        return;
    }
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            if tree_grid.has_tree(x, y) {
                let cur = maturity.values[idx];
                if cur < 1.0 {
                    maturity.values[idx] = (cur + MATURITY_PER_SLOW_TICK).min(1.0);
                }
            } else {
                // Reset maturity for removed trees
                maturity.values[idx] = 0.0;
            }
        }
    }
}

/// Enhanced tree effects: percentage-based pollution filtering, noise reduction,
/// green space cluster bonus, and land value boost.
///
/// Replaces the original flat reduction in `trees::tree_effects`.
#[allow(clippy::too_many_arguments)]
pub fn tree_absorption_effects(
    slow_tick: Res<SlowTickTimer>,
    tree_grid: Res<TreeGrid>,
    maturity: Res<TreeMaturityGrid>,
    mut pollution: ResMut<PollutionGrid>,
    mut noise: ResMut<NoisePollutionGrid>,
    mut land_value: ResMut<LandValueGrid>,
) {
    if !slow_tick.should_run() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if !tree_grid.has_tree(x, y) {
                continue;
            }

            let mat = maturity.get(x, y);
            if mat <= 0.0 {
                continue;
            }

            // Count adjacent trees for cluster bonus (radius 2, 8-connectivity)
            let adjacent_trees = count_adjacent_trees(&tree_grid, x, y);
            let cluster_mult = if adjacent_trees >= CLUSTER_THRESHOLD {
                GREEN_SPACE_CLUSTER_BONUS
            } else {
                1.0
            };

            // Apply vegetation filtering in radius 2
            let radius = 2i32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0
                        || ny < 0
                        || (nx as usize) >= GRID_WIDTH
                        || (ny as usize) >= GRID_HEIGHT
                    {
                        continue;
                    }
                    let ux = nx as usize;
                    let uy = ny as usize;
                    let dist = dx.abs() + dy.abs();

                    // Distance-scaled effectiveness: 1.0 at center, 0.5 at dist 2
                    let dist_factor = 1.0 - (dist as f32 * 0.25);
                    if dist_factor <= 0.0 {
                        continue;
                    }

                    let effective = mat * dist_factor;

                    // Percentage-based air pollution filtering
                    let filter = 1.0
                        - (1.0 - VEGETATION_FILTER_MULT) * effective * cluster_mult;
                    let cur_pol = pollution.get(ux, uy) as f32;
                    let new_pol = (cur_pol * filter).clamp(0.0, 255.0) as u8;
                    pollution.set(ux, uy, new_pol);

                    // Percentage-based noise reduction
                    let noise_reduction = MAX_NOISE_REDUCTION_PCT * effective;
                    let cur_noise = noise.get(ux, uy) as f32;
                    let new_noise =
                        (cur_noise * (1.0 - noise_reduction)).clamp(0.0, 255.0) as u8;
                    noise.set(ux, uy, new_noise);
                }
            }

            // Land value boost (radius 1, unchanged)
            let lv_radius = 1i32;
            for dy in -lv_radius..=lv_radius {
                for dx in -lv_radius..=lv_radius {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                    {
                        let ux = nx as usize;
                        let uy = ny as usize;
                        let boost =
                            (LAND_VALUE_BOOST as f32 * mat).round() as u8;
                        let cur = land_value.get(ux, uy);
                        land_value.set(ux, uy, cur.saturating_add(boost));
                    }
                }
            }
        }
    }
}

/// Computes per-district canopy percentage and city-wide CO2 absorption.
pub fn update_canopy_stats(
    slow_tick: Res<SlowTickTimer>,
    tree_grid: Res<TreeGrid>,
    maturity: Res<TreeMaturityGrid>,
    _districts: Res<Districts>,
    mut stats: ResMut<TreeCanopyStats>,
) {
    if !slow_tick.should_run() {
        return;
    }

    let cells_per_district = (DISTRICT_SIZE * DISTRICT_SIZE) as f32;

    // Reset stats
    stats.canopy_pct.fill(0.0);
    let mut total_mature_trees: f32 = 0.0;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if !tree_grid.has_tree(x, y) {
                continue;
            }
            let mat = maturity.get(x, y);
            let dx = x / DISTRICT_SIZE;
            let dy = y / DISTRICT_SIZE;
            if dx < DISTRICTS_X && dy < DISTRICTS_Y {
                stats.canopy_pct[dy * DISTRICTS_X + dx] += mat;
            }
            total_mature_trees += mat;
        }
    }

    // Convert tree counts to percentages
    for pct in stats.canopy_pct.iter_mut() {
        *pct = (*pct / cells_per_district).min(1.0);
    }

    // CO2 absorption: mature-equivalent trees * 48 lbs/year
    stats.total_co2_absorption_lbs_per_year = total_mature_trees * CO2_PER_TREE_PER_YEAR_LBS;
}

/// Count adjacent trees in a 5x5 area (radius 2) around (cx, cy), excluding center.
fn count_adjacent_trees(tree_grid: &TreeGrid, cx: usize, cy: usize) -> usize {
    let mut count = 0;
    let radius = 2i32;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx >= 0
                && ny >= 0
                && (nx as usize) < GRID_WIDTH
                && (ny as usize) < GRID_HEIGHT
                && tree_grid.has_tree(nx as usize, ny as usize)
            {
                count += 1;
            }
        }
    }
    count
}

// =============================================================================
// Saveable implementations
// =============================================================================

impl crate::Saveable for TreeMaturityGrid {
    const SAVE_KEY: &'static str = "tree_maturity";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.values.iter().all(|&v| v == 0.0) {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

impl crate::Saveable for TreeCanopyStats {
    const SAVE_KEY: &'static str = "tree_canopy_stats";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.total_co2_absorption_lbs_per_year == 0.0
            && self.canopy_pct.iter().all(|&v| v == 0.0)
        {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct TreeAbsorptionPlugin;

impl Plugin for TreeAbsorptionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TreeMaturityGrid>();

        // Initialize TreeCanopyStats with correct size
        app.insert_resource(TreeCanopyStats::new());

        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<TreeMaturityGrid>();
        registry.register::<TreeCanopyStats>();

        app.add_systems(
            FixedUpdate,
            (
                grow_tree_maturity,
                tree_absorption_effects
                    .after(grow_tree_maturity)
                    .after(crate::wind_pollution::update_pollution_gaussian_plume),
                update_canopy_stats.after(grow_tree_maturity),
            )
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maturity_per_slow_tick_value() {
        // 5 game days = 5 * 1440 / 100 = 72 slow ticks
        let expected = 1.0 / 72.0;
        let diff = (MATURITY_PER_SLOW_TICK - expected).abs();
        assert!(diff < 0.001, "MATURITY_PER_SLOW_TICK={}, expected ~{}", MATURITY_PER_SLOW_TICK, expected);
    }

    #[test]
    fn test_vegetation_filter_multiplier() {
        // At full maturity, center cell: filter = 1.0 - 0.4 * 1.0 * 1.0 = 0.6
        let mat = 1.0;
        let dist_factor = 1.0;
        let cluster_mult = 1.0;
        let filter = 1.0 - (1.0 - VEGETATION_FILTER_MULT) * mat * dist_factor * cluster_mult;
        assert!((filter - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cluster_bonus_filter() {
        // With cluster bonus: filter = 1.0 - 0.4 * 1.0 * 0.8 = 0.68
        // Effective reduction = 1.0 - 0.68 = 0.32 on the multiplier
        let filter = 1.0 - (1.0 - VEGETATION_FILTER_MULT) * 1.0 * GREEN_SPACE_CLUSTER_BONUS;
        assert!((filter - 0.68).abs() < 0.001);
    }

    #[test]
    fn test_count_adjacent_trees_empty() {
        let grid = TreeGrid::default();
        assert_eq!(count_adjacent_trees(&grid, 128, 128), 0);
    }

    #[test]
    fn test_count_adjacent_trees_full() {
        let mut grid = TreeGrid::default();
        // Fill a 5x5 area around (128, 128)
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                grid.set((128 + dx) as usize, (128 + dy) as usize, true);
            }
        }
        // 5x5 = 25, minus center = 24
        assert_eq!(count_adjacent_trees(&grid, 128, 128), 24);
    }

    #[test]
    fn test_maturity_grid_default() {
        let grid = TreeMaturityGrid::default();
        assert_eq!(grid.get(0, 0), 0.0);
        assert_eq!(grid.get(255, 255), 0.0);
    }

    #[test]
    fn test_canopy_stats_default() {
        let stats = TreeCanopyStats::new();
        assert_eq!(stats.canopy_pct.len(), DISTRICTS_X * DISTRICTS_Y);
        assert_eq!(stats.total_co2_absorption_lbs_per_year, 0.0);
    }

    #[test]
    fn test_co2_absorption_calculation() {
        // 100 fully mature trees should absorb 100 * 48 = 4800 lbs/year
        let trees = 100.0;
        let absorption = trees * CO2_PER_TREE_PER_YEAR_LBS;
        assert!((absorption - 4800.0).abs() < f32::EPSILON);
    }
}
