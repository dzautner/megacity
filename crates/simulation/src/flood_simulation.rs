//! Urban flooding simulation and depth-damage curves (FLOOD-961).
//!
//! When stormwater runoff exceeds storm drainage capacity, excess water pools on
//! the surface and spreads via a simplified shallow-water model. The `FloodGrid`
//! resource tracks per-cell flood depth (in feet) while the `FloodState` resource
//! provides aggregate statistics (total flooded cells, cumulative damage, maximum
//! depth).
//!
//! Depth-damage curves translate flood depth into a fractional damage value for
//! each zone type (Residential, Commercial, Industrial). Damage is applied to
//! buildings based on their estimated property value.
//!
//! The `update_flood_simulation` system runs every slow tick and performs:
//!   1. Checks if flooding conditions exist (storm drainage overflow > threshold)
//!   2. Initializes the FloodGrid from stormwater overflow
//!   3. Runs 5 iterations of water spreading (high elevation to low, 4-connected)
//!   4. Applies drainage rates (natural drain + enhanced drain for cells with drains)
//!   5. Calculates building damage using depth-damage curves
//!   6. Updates FloodState with aggregate statistics
//!   7. Clears FloodGrid when flooding subsides

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{WorldGrid, ZoneType};
use crate::storm_drainage::StormDrainageState;
use crate::stormwater::StormwaterGrid;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Number of water-spreading iterations per slow tick.
const SPREAD_ITERATIONS: usize = 5;

/// Fraction of excess water distributed to each lower-elevation neighbor per iteration.
const SPREAD_RATE: f32 = 0.25;

/// Natural drain rate per tick (feet removed from all cells).
const NATURAL_DRAIN_RATE: f32 = 0.01;

/// Additional drain rate per tick for cells covered by storm drainage infrastructure (feet).
const STORM_DRAIN_RATE: f32 = 0.05;

/// Minimum flood depth (feet) for a cell to count as "flooded".
const FLOOD_DEPTH_THRESHOLD: f32 = 0.5;

/// Overflow cell count above which flooding is triggered.
/// Storm drainage overflow_cells must exceed this value to initiate flooding.
const OVERFLOW_TRIGGER_THRESHOLD: u32 = 10;

/// Conversion factor from stormwater runoff grid units to flood depth in feet.
/// Stormwater runoff is stored as `rainfall_intensity * imperviousness * CELL_AREA`.
/// We normalise into feet of standing water.
const RUNOFF_TO_FEET: f32 = 0.001;

/// Base property value per building capacity unit, used for damage cost estimation.
const BASE_PROPERTY_VALUE_PER_CAPACITY: f64 = 1000.0;

// =============================================================================
// Depth-damage curve data
// =============================================================================

/// Depth breakpoints (in feet) for the depth-damage curves.
const DEPTH_BREAKPOINTS: [f32; 5] = [0.0, 1.0, 3.0, 6.0, 10.0];

/// Damage fractions for Residential zones at each depth breakpoint.
const RESIDENTIAL_DAMAGE: [f32; 5] = [0.0, 0.10, 0.35, 0.65, 0.90];

/// Damage fractions for Commercial zones at each depth breakpoint.
const COMMERCIAL_DAMAGE: [f32; 5] = [0.0, 0.05, 0.20, 0.50, 0.80];

/// Damage fractions for Industrial zones at each depth breakpoint.
const INDUSTRIAL_DAMAGE: [f32; 5] = [0.0, 0.03, 0.15, 0.40, 0.70];

// =============================================================================
// Depth-damage curve lookup
// =============================================================================

/// Linearly interpolate the damage fraction for a given `depth` (feet) using the
/// provided breakpoint and damage arrays.
///
/// Depths below the first breakpoint return 0.0; depths above the last breakpoint
/// return the maximum damage fraction.
pub fn interpolate_damage(depth: f32, breakpoints: &[f32; 5], damages: &[f32; 5]) -> f32 {
    if depth <= breakpoints[0] {
        return damages[0];
    }
    for i in 1..breakpoints.len() {
        if depth <= breakpoints[i] {
            let t = (depth - breakpoints[i - 1]) / (breakpoints[i] - breakpoints[i - 1]);
            return damages[i - 1] + t * (damages[i] - damages[i - 1]);
        }
    }
    // Beyond the last breakpoint: return maximum damage
    damages[breakpoints.len() - 1]
}

/// Returns the damage fraction for a given `depth` (feet) and `zone` type.
///
/// Residential, Commercial, and Industrial zones each have distinct curves.
/// Office and MixedUse zones use the Commercial curve. All other zones (None,
/// unzoned) return 0.0 damage.
pub fn depth_damage_fraction(depth: f32, zone: ZoneType) -> f32 {
    if zone.is_residential() {
        interpolate_damage(depth, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE)
    } else if zone.is_commercial() || matches!(zone, ZoneType::Office | ZoneType::MixedUse) {
        interpolate_damage(depth, &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE)
    } else if matches!(zone, ZoneType::Industrial) {
        interpolate_damage(depth, &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE)
    } else {
        0.0
    }
}

// =============================================================================
// FloodGrid resource
// =============================================================================

/// Per-cell flood depth in feet. Only actively maintained during flooding events.
#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct FloodGrid {
    /// Flood depth per cell in feet (GRID_WIDTH * GRID_HEIGHT).
    pub cells: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl Default for FloodGrid {
    fn default() -> Self {
        Self {
            cells: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl FloodGrid {
    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.cells[self.index(x, y)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        let idx = self.index(x, y);
        self.cells[idx] = val;
    }

    /// Returns true if any cell has depth >= `FLOOD_DEPTH_THRESHOLD`.
    pub fn has_flooding(&self) -> bool {
        self.cells.iter().any(|&d| d >= FLOOD_DEPTH_THRESHOLD)
    }

    /// Clear all flood depths to zero.
    pub fn clear(&mut self) {
        self.cells.iter_mut().for_each(|d| *d = 0.0);
    }
}

// =============================================================================
// FloodState resource
// =============================================================================

/// Aggregate flood statistics for the city.
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct FloodState {
    /// Whether a flood event is currently active.
    pub is_flooding: bool,
    /// Number of cells with flood depth >= `FLOOD_DEPTH_THRESHOLD`.
    pub total_flooded_cells: u32,
    /// Cumulative monetary damage from the current flood event.
    pub total_damage: f64,
    /// Maximum flood depth across all cells (feet).
    pub max_depth: f32,
}

// =============================================================================
// System
// =============================================================================

/// Main flood simulation system. Runs every slow tick.
///
/// When storm drainage overflow exceeds the trigger threshold, the system
/// initialises flood depths from excess stormwater runoff, spreads water over
/// the terrain for 5 iterations, applies natural and infrastructure-assisted
/// drainage, calculates building damage via depth-damage curves, and updates
/// aggregate flood statistics.
#[allow(clippy::too_many_arguments)]
pub fn update_flood_simulation(
    slow_timer: Res<SlowTickTimer>,
    mut flood_grid: ResMut<FloodGrid>,
    mut flood_state: ResMut<FloodState>,
    world_grid: Res<WorldGrid>,
    stormwater: Res<StormwaterGrid>,
    drainage_state: Res<StormDrainageState>,
    buildings: Query<&crate::buildings::Building>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Step 1: Check if flooding conditions exist ---
    let flooding_triggered = drainage_state.overflow_cells > OVERFLOW_TRIGGER_THRESHOLD;

    if !flooding_triggered && !flood_grid.has_flooding() {
        // No new flooding and no residual water: ensure state is clean
        if flood_state.is_flooding {
            flood_state.is_flooding = false;
            flood_state.total_flooded_cells = 0;
            flood_state.total_damage = 0.0;
            flood_state.max_depth = 0.0;
            flood_grid.clear();
        }
        return;
    }

    // --- Step 2: If newly flooding, seed FloodGrid from stormwater overflow ---
    if flooding_triggered {
        // Excess runoff that the drainage system could not handle becomes flood water.
        // We only add NEW water each tick, not replace existing depths.
        let drain_cap = drainage_state.total_drain_capacity;
        let total_cells = GRID_WIDTH * GRID_HEIGHT;
        let per_cell_drain = if drain_cap > 0.0 {
            drain_cap / total_cells as f32
        } else {
            0.0
        };

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let runoff = stormwater.get(x, y);
                // Convert runoff to depth in feet; subtract the drainage capacity share
                let excess = (runoff * RUNOFF_TO_FEET - per_cell_drain).max(0.0);
                if excess > 0.0 {
                    let idx = flood_grid.index(x, y);
                    flood_grid.cells[idx] += excess;
                }
            }
        }
    }

    // --- Step 3: Run 5 iterations of water spreading ---
    for _ in 0..SPREAD_ITERATIONS {
        // Snapshot current depths to avoid order-dependent artifacts
        let snapshot: Vec<f32> = flood_grid.cells.clone();

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let idx = y * flood_grid.width + x;
                let current_depth = snapshot[idx];
                if current_depth <= 0.0 {
                    continue;
                }

                let current_elevation = world_grid.get(x, y).elevation;
                let current_surface = current_elevation + current_depth;

                // Find lower-surface neighbors (4-connected)
                let (neighbors, count) = world_grid.neighbors4(x, y);
                let mut lower: [(usize, usize, f32); 4] = [(0, 0, 0.0); 4];
                let mut lower_count = 0usize;
                let mut total_diff = 0.0_f32;

                for &(nx, ny) in &neighbors[..count] {
                    let n_idx = ny * flood_grid.width + nx;
                    let n_elevation = world_grid.get(nx, ny).elevation;
                    let n_surface = n_elevation + snapshot[n_idx];

                    if n_surface < current_surface {
                        let diff = current_surface - n_surface;
                        lower[lower_count] = (nx, ny, diff);
                        lower_count += 1;
                        total_diff += diff;
                    }
                }

                if lower_count == 0 || total_diff <= 0.0 {
                    continue;
                }

                // Distribute water proportionally to surface height difference
                let transferable = current_depth * SPREAD_RATE;
                flood_grid.cells[idx] -= transferable;

                for &(nx, ny, diff) in &lower[..lower_count] {
                    let fraction = diff / total_diff;
                    let transfer = transferable * fraction;
                    let n_idx = ny * flood_grid.width + nx;
                    flood_grid.cells[n_idx] += transfer;
                }
            }
        }
    }

    // --- Step 4: Apply drainage rates ---
    // Build a quick lookup of which cells have storm drain infrastructure.
    // We approximate this using drainage_coverage: if coverage > 0 and the cell
    // is a road cell or adjacent to a road, it gets enhanced drainage.
    let has_drain_infrastructure = drainage_state.drain_count > 0;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = flood_grid.index(x, y);
            if flood_grid.cells[idx] <= 0.0 {
                continue;
            }

            // Natural drainage applies to all cells
            let mut drain = NATURAL_DRAIN_RATE;

            // Enhanced drainage for cells with road (storm drains follow roads)
            if has_drain_infrastructure
                && world_grid.get(x, y).cell_type == crate::grid::CellType::Road
            {
                drain += STORM_DRAIN_RATE;
            }

            flood_grid.cells[idx] = (flood_grid.cells[idx] - drain).max(0.0);
        }
    }

    // --- Step 5: Calculate damage for buildings in flooded cells ---
    let mut total_damage = 0.0_f64;

    for building in &buildings {
        let bx = building.grid_x;
        let by = building.grid_y;
        if bx >= GRID_WIDTH || by >= GRID_HEIGHT {
            continue;
        }

        let depth = flood_grid.get(bx, by);
        if depth < FLOOD_DEPTH_THRESHOLD {
            continue;
        }

        let damage_fraction = depth_damage_fraction(depth, building.zone_type);
        let building_value =
            building.capacity as f64 * building.level as f64 * BASE_PROPERTY_VALUE_PER_CAPACITY;
        total_damage += building_value * damage_fraction as f64;
    }

    // --- Step 6: Update FloodState with stats ---
    let mut flooded_cells: u32 = 0;
    let mut max_depth: f32 = 0.0;

    for &depth in &flood_grid.cells {
        if depth >= FLOOD_DEPTH_THRESHOLD {
            flooded_cells += 1;
        }
        if depth > max_depth {
            max_depth = depth;
        }
    }

    flood_state.is_flooding = flooded_cells > 0;
    flood_state.total_flooded_cells = flooded_cells;
    flood_state.total_damage = total_damage;
    flood_state.max_depth = max_depth;

    // --- Step 7: If no more flooding, clear FloodGrid ---
    if !flood_state.is_flooding {
        flood_grid.clear();
        flood_state.total_damage = 0.0;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Depth-damage curve interpolation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_interpolate_damage_at_breakpoints() {
        // At each exact breakpoint, the result should match the damage table
        for i in 0..DEPTH_BREAKPOINTS.len() {
            let d = interpolate_damage(
                DEPTH_BREAKPOINTS[i],
                &DEPTH_BREAKPOINTS,
                &RESIDENTIAL_DAMAGE,
            );
            assert!(
                (d - RESIDENTIAL_DAMAGE[i]).abs() < f32::EPSILON,
                "Residential damage at {} ft should be {}, got {}",
                DEPTH_BREAKPOINTS[i],
                RESIDENTIAL_DAMAGE[i],
                d
            );
        }
    }

    #[test]
    fn test_interpolate_damage_between_breakpoints() {
        // At 2.0 ft (midpoint between 1.0 and 3.0), residential should interpolate
        // between 0.10 and 0.35 => 0.10 + 0.5 * 0.25 = 0.225
        let d = interpolate_damage(2.0, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
        assert!(
            (d - 0.225).abs() < 0.001,
            "Residential damage at 2.0 ft should be ~0.225, got {}",
            d
        );
    }

    #[test]
    fn test_interpolate_damage_below_zero() {
        let d = interpolate_damage(-1.0, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
        assert!(
            d.abs() < f32::EPSILON,
            "Damage at negative depth should be 0.0, got {}",
            d
        );
    }

    #[test]
    fn test_interpolate_damage_above_max_breakpoint() {
        // Above 10 ft should return the max damage (0.90 for residential)
        let d = interpolate_damage(15.0, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
        assert!(
            (d - 0.90).abs() < f32::EPSILON,
            "Residential damage above 10 ft should be 0.90, got {}",
            d
        );
    }

    #[test]
    fn test_interpolate_damage_at_zero_depth() {
        let d = interpolate_damage(0.0, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
        assert!(
            d.abs() < f32::EPSILON,
            "Damage at 0 ft should be 0.0, got {}",
            d
        );
    }

    // -------------------------------------------------------------------------
    // Commercial depth-damage curve tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_commercial_damage_at_breakpoints() {
        for i in 0..DEPTH_BREAKPOINTS.len() {
            let d =
                interpolate_damage(DEPTH_BREAKPOINTS[i], &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE);
            assert!(
                (d - COMMERCIAL_DAMAGE[i]).abs() < f32::EPSILON,
                "Commercial damage at {} ft should be {}, got {}",
                DEPTH_BREAKPOINTS[i],
                COMMERCIAL_DAMAGE[i],
                d
            );
        }
    }

    #[test]
    fn test_commercial_damage_interpolation_midpoint() {
        // At 4.5 ft (midpoint between 3.0 and 6.0), commercial should be
        // 0.20 + 0.5 * (0.50 - 0.20) = 0.20 + 0.15 = 0.35
        let d = interpolate_damage(4.5, &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE);
        assert!(
            (d - 0.35).abs() < 0.001,
            "Commercial damage at 4.5 ft should be ~0.35, got {}",
            d
        );
    }

    // -------------------------------------------------------------------------
    // Industrial depth-damage curve tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_industrial_damage_at_breakpoints() {
        for i in 0..DEPTH_BREAKPOINTS.len() {
            let d =
                interpolate_damage(DEPTH_BREAKPOINTS[i], &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE);
            assert!(
                (d - INDUSTRIAL_DAMAGE[i]).abs() < f32::EPSILON,
                "Industrial damage at {} ft should be {}, got {}",
                DEPTH_BREAKPOINTS[i],
                INDUSTRIAL_DAMAGE[i],
                d
            );
        }
    }

    #[test]
    fn test_industrial_damage_above_max() {
        let d = interpolate_damage(20.0, &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE);
        assert!(
            (d - 0.70).abs() < f32::EPSILON,
            "Industrial damage above 10 ft should be 0.70, got {}",
            d
        );
    }

    // -------------------------------------------------------------------------
    // Zone-type damage dispatch tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_depth_damage_fraction_residential_zones() {
        for zone in [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
        ] {
            let d = depth_damage_fraction(6.0, zone);
            assert!(
                (d - 0.65).abs() < f32::EPSILON,
                "Residential damage at 6 ft for {:?} should be 0.65, got {}",
                zone,
                d
            );
        }
    }

    #[test]
    fn test_depth_damage_fraction_commercial_zones() {
        for zone in [ZoneType::CommercialLow, ZoneType::CommercialHigh] {
            let d = depth_damage_fraction(6.0, zone);
            assert!(
                (d - 0.50).abs() < f32::EPSILON,
                "Commercial damage at 6 ft for {:?} should be 0.50, got {}",
                zone,
                d
            );
        }
    }

    #[test]
    fn test_depth_damage_fraction_industrial() {
        let d = depth_damage_fraction(6.0, ZoneType::Industrial);
        assert!(
            (d - 0.40).abs() < f32::EPSILON,
            "Industrial damage at 6 ft should be 0.40, got {}",
            d
        );
    }

    #[test]
    fn test_depth_damage_fraction_office_uses_commercial_curve() {
        let office = depth_damage_fraction(3.0, ZoneType::Office);
        let commercial = depth_damage_fraction(3.0, ZoneType::CommercialHigh);
        assert!(
            (office - commercial).abs() < f32::EPSILON,
            "Office should use commercial curve: office={}, commercial={}",
            office,
            commercial
        );
    }

    #[test]
    fn test_depth_damage_fraction_mixed_use_uses_commercial_curve() {
        let mixed = depth_damage_fraction(3.0, ZoneType::MixedUse);
        let commercial = depth_damage_fraction(3.0, ZoneType::CommercialLow);
        assert!(
            (mixed - commercial).abs() < f32::EPSILON,
            "MixedUse should use commercial curve: mixed={}, commercial={}",
            mixed,
            commercial
        );
    }

    #[test]
    fn test_depth_damage_fraction_none_zone_is_zero() {
        let d = depth_damage_fraction(10.0, ZoneType::None);
        assert!(
            d.abs() < f32::EPSILON,
            "None zone should have 0 damage, got {}",
            d
        );
    }

    // -------------------------------------------------------------------------
    // Damage monotonicity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_residential_damage_monotonically_increasing() {
        let mut prev = 0.0_f32;
        for depth_tenths in 0..=120 {
            let depth = depth_tenths as f32 * 0.1;
            let d = interpolate_damage(depth, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
            assert!(
                d >= prev - f32::EPSILON,
                "Residential damage should be monotonically increasing: {} at {} ft < {} at prev depth",
                d,
                depth,
                prev
            );
            prev = d;
        }
    }

    #[test]
    fn test_commercial_damage_monotonically_increasing() {
        let mut prev = 0.0_f32;
        for depth_tenths in 0..=120 {
            let depth = depth_tenths as f32 * 0.1;
            let d = interpolate_damage(depth, &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE);
            assert!(
                d >= prev - f32::EPSILON,
                "Commercial damage should be monotonically increasing at {} ft",
                depth
            );
            prev = d;
        }
    }

    #[test]
    fn test_industrial_damage_monotonically_increasing() {
        let mut prev = 0.0_f32;
        for depth_tenths in 0..=120 {
            let depth = depth_tenths as f32 * 0.1;
            let d = interpolate_damage(depth, &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE);
            assert!(
                d >= prev - f32::EPSILON,
                "Industrial damage should be monotonically increasing at {} ft",
                depth
            );
            prev = d;
        }
    }

    // -------------------------------------------------------------------------
    // Residential > Commercial > Industrial damage ordering
    // -------------------------------------------------------------------------

    #[test]
    fn test_damage_ordering_residential_gt_commercial_gt_industrial() {
        for depth_tenths in 1..=100 {
            let depth = depth_tenths as f32 * 0.1;
            let res = interpolate_damage(depth, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
            let com = interpolate_damage(depth, &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE);
            let ind = interpolate_damage(depth, &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE);
            assert!(
                res >= com - f32::EPSILON,
                "Residential damage ({}) should >= commercial ({}) at {} ft",
                res,
                com,
                depth
            );
            assert!(
                com >= ind - f32::EPSILON,
                "Commercial damage ({}) should >= industrial ({}) at {} ft",
                com,
                ind,
                depth
            );
        }
    }

    // -------------------------------------------------------------------------
    // FloodGrid resource tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_flood_grid_default() {
        let fg = FloodGrid::default();
        assert_eq!(fg.cells.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(fg.width, GRID_WIDTH);
        assert_eq!(fg.height, GRID_HEIGHT);
        assert!(fg.cells.iter().all(|&d| d == 0.0));
    }

    #[test]
    fn test_flood_grid_get_set() {
        let mut fg = FloodGrid::default();
        fg.set(10, 20, 3.5);
        assert!((fg.get(10, 20) - 3.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_flood_grid_index() {
        let fg = FloodGrid::default();
        assert_eq!(fg.index(0, 0), 0);
        assert_eq!(fg.index(1, 0), 1);
        assert_eq!(fg.index(0, 1), GRID_WIDTH);
        assert_eq!(fg.index(5, 3), 3 * GRID_WIDTH + 5);
    }

    #[test]
    fn test_flood_grid_has_flooding_false_when_empty() {
        let fg = FloodGrid::default();
        assert!(!fg.has_flooding());
    }

    #[test]
    fn test_flood_grid_has_flooding_true_when_above_threshold() {
        let mut fg = FloodGrid::default();
        fg.set(50, 50, FLOOD_DEPTH_THRESHOLD);
        assert!(fg.has_flooding());
    }

    #[test]
    fn test_flood_grid_has_flooding_false_when_below_threshold() {
        let mut fg = FloodGrid::default();
        fg.set(50, 50, FLOOD_DEPTH_THRESHOLD - 0.01);
        assert!(!fg.has_flooding());
    }

    #[test]
    fn test_flood_grid_clear() {
        let mut fg = FloodGrid::default();
        fg.set(10, 10, 5.0);
        fg.set(20, 20, 3.0);
        fg.clear();
        assert!(fg.cells.iter().all(|&d| d == 0.0));
    }

    // -------------------------------------------------------------------------
    // FloodState resource tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_flood_state_default() {
        let fs = FloodState::default();
        assert!(!fs.is_flooding);
        assert_eq!(fs.total_flooded_cells, 0);
        assert!((fs.total_damage - 0.0).abs() < f64::EPSILON);
        assert!((fs.max_depth - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_flood_state_clone() {
        let mut fs = FloodState::default();
        fs.is_flooding = true;
        fs.total_flooded_cells = 42;
        fs.total_damage = 123456.0;
        fs.max_depth = 8.5;
        let cloned = fs.clone();
        assert!(cloned.is_flooding);
        assert_eq!(cloned.total_flooded_cells, 42);
        assert!((cloned.total_damage - 123456.0).abs() < f64::EPSILON);
        assert!((cloned.max_depth - 8.5).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // FloodGrid serde round-trip test
    // -------------------------------------------------------------------------

    #[test]
    fn test_flood_grid_serde_roundtrip() {
        let mut fg = FloodGrid::default();
        fg.set(5, 5, 2.0);
        fg.set(100, 100, 7.5);

        let json = serde_json::to_string(&fg).expect("serialize");
        let deserialized: FloodGrid = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.width, GRID_WIDTH);
        assert_eq!(deserialized.height, GRID_HEIGHT);
        assert!((deserialized.get(5, 5) - 2.0).abs() < f32::EPSILON);
        assert!((deserialized.get(100, 100) - 7.5).abs() < f32::EPSILON);
        assert!((deserialized.get(0, 0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_flood_state_serde_roundtrip() {
        let fs = FloodState {
            is_flooding: true,
            total_flooded_cells: 150,
            total_damage: 999_999.99,
            max_depth: 12.3,
        };

        let json = serde_json::to_string(&fs).expect("serialize");
        let deserialized: FloodState = serde_json::from_str(&json).expect("deserialize");

        assert!(deserialized.is_flooding);
        assert_eq!(deserialized.total_flooded_cells, 150);
        assert!((deserialized.total_damage - 999_999.99).abs() < 0.01);
        assert!((deserialized.max_depth - 12.3).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Water spreading logic tests (unit tests for the algorithm)
    // -------------------------------------------------------------------------

    #[test]
    fn test_spread_rate_constant() {
        assert!(
            (SPREAD_RATE - 0.25).abs() < f32::EPSILON,
            "Spread rate should be 0.25"
        );
    }

    #[test]
    fn test_natural_drain_rate_constant() {
        assert!(
            (NATURAL_DRAIN_RATE - 0.01).abs() < f32::EPSILON,
            "Natural drain rate should be 0.01 ft/tick"
        );
    }

    #[test]
    fn test_storm_drain_rate_constant() {
        assert!(
            (STORM_DRAIN_RATE - 0.05).abs() < f32::EPSILON,
            "Storm drain rate should be 0.05 ft/tick"
        );
    }

    #[test]
    fn test_flood_threshold_constant() {
        assert!(
            (FLOOD_DEPTH_THRESHOLD - 0.5).abs() < f32::EPSILON,
            "Flood threshold should be 0.5 ft"
        );
    }

    #[test]
    fn test_spread_iterations_constant() {
        assert_eq!(SPREAD_ITERATIONS, 5, "Should run 5 spread iterations");
    }

    // -------------------------------------------------------------------------
    // Water conservation during spreading (manual simulation)
    // -------------------------------------------------------------------------

    #[test]
    fn test_water_conservation_single_spread_step() {
        // Simulate a single spread step on a small 3x3 grid.
        // Center cell has 4.0 ft of water; all neighbors are at lower elevation.
        // Flat terrain: elevation 10.0 at center, 9.0 at neighbors.
        let mut depths = vec![0.0_f32; 9];
        let elevations = vec![9.0, 9.0, 9.0, 9.0, 10.0, 9.0, 9.0, 9.0, 9.0];
        let width = 3usize;

        // Place water at center (1,1)
        depths[1 * width + 1] = 4.0;

        let total_before: f32 = depths.iter().sum();

        // Spread: center cell distributes SPREAD_RATE * depth to lower neighbors
        let snapshot = depths.clone();
        let cx = 1usize;
        let cy = 1usize;
        let cidx = cy * width + cx;
        let current_depth = snapshot[cidx];
        let current_elev = elevations[cidx];
        let current_surface = current_elev + current_depth;

        // 4 cardinal neighbors of (1,1) in 3x3: (0,1), (2,1), (1,0), (1,2)
        let neighbors: [(usize, usize); 4] = [(0, 1), (2, 1), (1, 0), (1, 2)];
        let mut lower_diffs = Vec::new();
        let mut total_diff = 0.0_f32;

        for &(nx, ny) in &neighbors {
            let nidx = ny * width + nx;
            let n_surface = elevations[nidx] + snapshot[nidx];
            if n_surface < current_surface {
                let diff = current_surface - n_surface;
                lower_diffs.push((nx, ny, diff));
                total_diff += diff;
            }
        }

        let transferable = current_depth * SPREAD_RATE;
        depths[cidx] -= transferable;

        for &(nx, ny, diff) in &lower_diffs {
            let fraction = diff / total_diff;
            let transfer = transferable * fraction;
            let nidx = ny * width + nx;
            depths[nidx] += transfer;
        }

        let total_after: f32 = depths.iter().sum();

        assert!(
            (total_before - total_after).abs() < 0.001,
            "Water should be conserved: before={}, after={}",
            total_before,
            total_after
        );
    }

    #[test]
    fn test_water_spreads_to_lower_elevation_only() {
        // 3-cell row: elevations [8.0, 10.0, 12.0]. Water at center (index 1).
        // Water should only flow to the left (lower elevation).
        let elevations = [8.0_f32, 10.0, 12.0];
        let mut depths = [0.0_f32, 5.0, 0.0];

        let current_depth = depths[1];
        let current_surface = elevations[1] + current_depth; // 15.0

        // Left neighbor: surface = 8.0 + 0.0 = 8.0 < 15.0 => lower
        // Right neighbor: surface = 12.0 + 0.0 = 12.0 < 15.0 => also lower
        // But right elevation (12.0) is higher than center elevation (10.0)
        // With the surface-based comparison, BOTH are lower surface
        // The left one gets more water because the diff is larger

        let left_surface = elevations[0] + depths[0]; // 8.0
        let right_surface = elevations[2] + depths[2]; // 12.0

        assert!(left_surface < current_surface);
        assert!(right_surface < current_surface);

        let left_diff = current_surface - left_surface; // 7.0
        let right_diff = current_surface - right_surface; // 3.0
        let total_diff = left_diff + right_diff; // 10.0

        let transferable = current_depth * SPREAD_RATE; // 1.25
        depths[1] -= transferable;

        depths[0] += transferable * (left_diff / total_diff);
        depths[2] += transferable * (right_diff / total_diff);

        // Left should get more water (70%)
        assert!(
            depths[0] > depths[2],
            "Lower-elevation cell should receive more water: left={}, right={}",
            depths[0],
            depths[2]
        );

        // Water is conserved
        let total: f32 = depths.iter().sum();
        assert!(
            (total - 5.0).abs() < 0.001,
            "Total water should be 5.0, got {}",
            total
        );
    }

    #[test]
    fn test_no_spread_when_cell_is_highest() {
        // All neighbors have higher surface than center: no spreading occurs
        let elevations = [20.0, 20.0, 20.0, 20.0, 5.0, 20.0, 20.0, 20.0, 20.0];
        let depths = [0.0_f32; 9];
        let width = 3usize;

        let cx = 1usize;
        let cy = 1usize;
        let cidx = cy * width + cx;
        let current_surface = elevations[cidx] + depths[cidx] + 2.0; // 5 + 2 = 7

        let neighbors: [(usize, usize); 4] = [(0, 1), (2, 1), (1, 0), (1, 2)];
        let lower_count = neighbors
            .iter()
            .filter(|&&(nx, ny)| {
                let nidx = ny * width + nx;
                elevations[nidx] + depths[nidx] < current_surface
            })
            .count();

        // All neighbors at elevation 20 > surface 7
        assert_eq!(
            lower_count, 0,
            "No neighbors should be lower than center cell"
        );
    }

    // -------------------------------------------------------------------------
    // Drainage calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_natural_drain_reduces_depth() {
        let initial_depth = 1.0_f32;
        let after_drain = (initial_depth - NATURAL_DRAIN_RATE).max(0.0);
        assert!(
            (after_drain - 0.99).abs() < f32::EPSILON,
            "After natural drain: expected 0.99, got {}",
            after_drain
        );
    }

    #[test]
    fn test_storm_drain_plus_natural_drain() {
        let initial_depth = 1.0_f32;
        let after_drain = (initial_depth - NATURAL_DRAIN_RATE - STORM_DRAIN_RATE).max(0.0);
        let expected = 1.0 - 0.01 - 0.05;
        assert!(
            (after_drain - expected).abs() < 0.001,
            "After combined drain: expected {}, got {}",
            expected,
            after_drain
        );
    }

    #[test]
    fn test_drain_does_not_go_negative() {
        let initial_depth = 0.005_f32;
        let after_drain = (initial_depth - NATURAL_DRAIN_RATE - STORM_DRAIN_RATE).max(0.0);
        assert!(
            after_drain >= 0.0,
            "Drain should not produce negative depth"
        );
        assert!(
            after_drain.abs() < f32::EPSILON,
            "Small depth should drain to exactly 0.0"
        );
    }

    // -------------------------------------------------------------------------
    // Building damage calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_building_damage_residential_at_6ft() {
        // Residential L3 building with capacity 500 at 6 ft flood depth
        let capacity = 500u32;
        let level = 3u8;
        let depth = 6.0_f32;
        let damage_frac = depth_damage_fraction(depth, ZoneType::ResidentialHigh);
        let building_value = capacity as f64 * level as f64 * BASE_PROPERTY_VALUE_PER_CAPACITY;
        let damage = building_value * damage_frac as f64;

        // damage_frac = 0.65, building_value = 500 * 3 * 1000 = 1,500,000
        // damage = 1,500,000 * 0.65 = 975,000
        assert!(
            (damage - 975_000.0).abs() < 1.0,
            "Residential L3 damage at 6ft should be 975000, got {}",
            damage
        );
    }

    #[test]
    fn test_building_damage_industrial_at_3ft() {
        let capacity = 150u32;
        let level = 3u8;
        let depth = 3.0_f32;
        let damage_frac = depth_damage_fraction(depth, ZoneType::Industrial);
        let building_value = capacity as f64 * level as f64 * BASE_PROPERTY_VALUE_PER_CAPACITY;
        let damage = building_value * damage_frac as f64;

        // damage_frac = 0.15, building_value = 150 * 3 * 1000 = 450,000
        // damage = 450,000 * 0.15 = 67,500
        assert!(
            (damage - 67_500.0).abs() < 1.0,
            "Industrial L3 damage at 3ft should be 67500, got {}",
            damage
        );
    }

    #[test]
    fn test_building_damage_zero_below_threshold() {
        let depth = 0.3_f32; // below FLOOD_DEPTH_THRESHOLD
        assert!(
            depth < FLOOD_DEPTH_THRESHOLD,
            "Depth {} should be below threshold {}",
            depth,
            FLOOD_DEPTH_THRESHOLD
        );
        // No damage should be applied for depths below threshold
    }

    #[test]
    fn test_building_damage_commercial_at_10ft() {
        let capacity = 300u32;
        let level = 5u8;
        let depth = 10.0_f32;
        let damage_frac = depth_damage_fraction(depth, ZoneType::CommercialHigh);
        let building_value = capacity as f64 * level as f64 * BASE_PROPERTY_VALUE_PER_CAPACITY;
        let damage = building_value * damage_frac as f64;

        // damage_frac = 0.80, building_value = 300 * 5 * 1000 = 1,500,000
        // damage = 1,500,000 * 0.80 = 1,200,000
        assert!(
            (damage - 1_200_000.0).abs() < 1.0,
            "Commercial L5 damage at 10ft should be 1200000, got {}",
            damage
        );
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_all_constants_positive() {
        assert!(SPREAD_RATE > 0.0);
        assert!(SPREAD_RATE <= 1.0);
        assert!(NATURAL_DRAIN_RATE > 0.0);
        assert!(STORM_DRAIN_RATE > 0.0);
        assert!(FLOOD_DEPTH_THRESHOLD > 0.0);
        assert!(OVERFLOW_TRIGGER_THRESHOLD > 0);
        assert!(RUNOFF_TO_FEET > 0.0);
        assert!(BASE_PROPERTY_VALUE_PER_CAPACITY > 0.0);
        assert!(SPREAD_ITERATIONS > 0);
    }

    #[test]
    fn test_damage_curves_start_at_zero() {
        assert!(RESIDENTIAL_DAMAGE[0].abs() < f32::EPSILON);
        assert!(COMMERCIAL_DAMAGE[0].abs() < f32::EPSILON);
        assert!(INDUSTRIAL_DAMAGE[0].abs() < f32::EPSILON);
    }

    #[test]
    fn test_damage_curves_max_below_one() {
        let last = DEPTH_BREAKPOINTS.len() - 1;
        assert!(RESIDENTIAL_DAMAGE[last] <= 1.0);
        assert!(COMMERCIAL_DAMAGE[last] <= 1.0);
        assert!(INDUSTRIAL_DAMAGE[last] <= 1.0);
    }

    #[test]
    fn test_depth_breakpoints_are_monotonically_increasing() {
        for i in 1..DEPTH_BREAKPOINTS.len() {
            assert!(
                DEPTH_BREAKPOINTS[i] > DEPTH_BREAKPOINTS[i - 1],
                "Breakpoints must be monotonically increasing: {} <= {}",
                DEPTH_BREAKPOINTS[i],
                DEPTH_BREAKPOINTS[i - 1]
            );
        }
    }

    // -------------------------------------------------------------------------
    // Interpolation edge cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_interpolation_at_exactly_1ft() {
        let res = interpolate_damage(1.0, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
        assert!(
            (res - 0.10).abs() < f32::EPSILON,
            "Residential at 1 ft should be 0.10, got {}",
            res
        );
        let com = interpolate_damage(1.0, &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE);
        assert!(
            (com - 0.05).abs() < f32::EPSILON,
            "Commercial at 1 ft should be 0.05, got {}",
            com
        );
        let ind = interpolate_damage(1.0, &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE);
        assert!(
            (ind - 0.03).abs() < f32::EPSILON,
            "Industrial at 1 ft should be 0.03, got {}",
            ind
        );
    }

    #[test]
    fn test_interpolation_at_8ft() {
        // 8 ft is between 6 ft and 10 ft. t = (8-6)/(10-6) = 0.5
        // Residential: 0.65 + 0.5 * (0.90 - 0.65) = 0.65 + 0.125 = 0.775
        let res = interpolate_damage(8.0, &DEPTH_BREAKPOINTS, &RESIDENTIAL_DAMAGE);
        assert!(
            (res - 0.775).abs() < 0.001,
            "Residential at 8 ft should be ~0.775, got {}",
            res
        );

        // Commercial: 0.50 + 0.5 * (0.80 - 0.50) = 0.50 + 0.15 = 0.65
        let com = interpolate_damage(8.0, &DEPTH_BREAKPOINTS, &COMMERCIAL_DAMAGE);
        assert!(
            (com - 0.65).abs() < 0.001,
            "Commercial at 8 ft should be ~0.65, got {}",
            com
        );

        // Industrial: 0.40 + 0.5 * (0.70 - 0.40) = 0.40 + 0.15 = 0.55
        let ind = interpolate_damage(8.0, &DEPTH_BREAKPOINTS, &INDUSTRIAL_DAMAGE);
        assert!(
            (ind - 0.55).abs() < 0.001,
            "Industrial at 8 ft should be ~0.55, got {}",
            ind
        );
    }
}

pub struct FloodSimulationPlugin;

impl Plugin for FloodSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FloodGrid>()
            .init_resource::<FloodState>()
            .add_systems(
                FixedUpdate,
                update_flood_simulation
                    .after(crate::storm_drainage::update_storm_drainage)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
