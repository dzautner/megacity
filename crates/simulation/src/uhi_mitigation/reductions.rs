//! Pure helper functions that compute per-cell UHI reductions from each
//! mitigation measure.

use crate::trees::TreeGrid;

use super::state::UhiMitigationState;

// =============================================================================
// Constants (reduction magnitudes and radii)
// =============================================================================

/// UHI reduction from a tree cell (Fahrenheit).
pub(crate) const TREE_UHI_REDUCTION: f32 = 1.5;

/// UHI reduction from a green roof (Fahrenheit).
pub(crate) const GREEN_ROOF_UHI_REDUCTION: f32 = 2.0;

/// UHI reduction from a cool (white) roof (Fahrenheit).
pub(crate) const COOL_ROOF_UHI_REDUCTION: f32 = 1.5;

/// UHI reduction from cool pavement on a road cell (Fahrenheit).
pub(crate) const COOL_PAVEMENT_UHI_REDUCTION: f32 = 1.0;

/// UHI reduction from a park cell in its radius (Fahrenheit).
pub(crate) const PARK_UHI_REDUCTION: f32 = 3.0;

/// Radius of park cooling effect (cells).
pub(crate) const PARK_RADIUS: i32 = 2;

/// UHI reduction from a water feature / fountain (Fahrenheit).
pub(crate) const WATER_FEATURE_UHI_REDUCTION: f32 = 2.0;

/// UHI reduction from permeable surfaces (Fahrenheit).
pub(crate) const PERMEABLE_SURFACE_UHI_REDUCTION: f32 = 0.5;

/// UHI reduction from district cooling in its radius (Fahrenheit).
pub(crate) const DISTRICT_COOLING_UHI_REDUCTION: f32 = 1.0;

/// Radius of district cooling effect (cells).
pub(crate) const DISTRICT_COOLING_RADIUS: i32 = 3;

// =============================================================================
// Pure helper functions
// =============================================================================

/// Compute the per-cell UHI reduction from tree planting.
/// Returns `TREE_UHI_REDUCTION` if the cell has a tree, 0.0 otherwise.
pub fn tree_uhi_reduction(has_tree: bool) -> f32 {
    if has_tree {
        TREE_UHI_REDUCTION
    } else {
        0.0
    }
}

/// Compute the city-wide average green roof UHI reduction.
/// Scales linearly with the fraction of buildings upgraded.
pub fn green_roof_reduction(upgraded_count: u32, total_buildings: u32) -> f32 {
    if total_buildings == 0 {
        return 0.0;
    }
    let fraction = (upgraded_count as f32 / total_buildings as f32).min(1.0);
    fraction * GREEN_ROOF_UHI_REDUCTION
}

/// Compute the city-wide average cool roof UHI reduction.
/// Scales linearly with the fraction of buildings upgraded.
pub fn cool_roof_reduction(upgraded_count: u32, total_buildings: u32) -> f32 {
    if total_buildings == 0 {
        return 0.0;
    }
    let fraction = (upgraded_count as f32 / total_buildings as f32).min(1.0);
    fraction * COOL_ROOF_UHI_REDUCTION
}

/// Compute the UHI reduction at cell `(x, y)` from all park cells within
/// `PARK_RADIUS`, returning the maximum reduction from any single park.
pub fn park_reduction_at(state: &UhiMitigationState, x: usize, y: usize) -> f32 {
    let mut max_reduction: f32 = 0.0;
    for dy in -PARK_RADIUS..=PARK_RADIUS {
        for dx in -PARK_RADIUS..=PARK_RADIUS {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0
                && ny >= 0
                && (nx as usize) < crate::config::GRID_WIDTH
                && (ny as usize) < crate::config::GRID_HEIGHT
                && state.has_park(nx as usize, ny as usize)
            {
                max_reduction = max_reduction.max(PARK_UHI_REDUCTION);
            }
        }
    }
    max_reduction
}

/// Compute the UHI reduction at cell `(x, y)` from nearby water features.
/// Returns the max reduction from any water feature within radius 1.
pub fn water_feature_reduction_at(state: &UhiMitigationState, x: usize, y: usize) -> f32 {
    for &(wx, wy) in &state.water_features {
        let dx = (x as i32 - wx as i32).unsigned_abs() as usize;
        let dy = (y as i32 - wy as i32).unsigned_abs() as usize;
        if dx <= 1 && dy <= 1 {
            return WATER_FEATURE_UHI_REDUCTION;
        }
    }
    0.0
}

/// Compute the UHI reduction at cell `(x, y)` from nearby district cooling
/// facilities. Returns the max reduction from any facility within
/// `DISTRICT_COOLING_RADIUS`.
pub fn district_cooling_reduction_at(state: &UhiMitigationState, x: usize, y: usize) -> f32 {
    for &(fx, fy) in &state.district_cooling_facilities {
        let dx = (x as i32 - fx as i32).abs();
        let dy = (y as i32 - fy as i32).abs();
        if dx <= DISTRICT_COOLING_RADIUS && dy <= DISTRICT_COOLING_RADIUS {
            return DISTRICT_COOLING_UHI_REDUCTION;
        }
    }
    0.0
}

/// Compute the total per-cell UHI reduction at `(x, y)` from all mitigation
/// measures. Reductions are additive.
pub fn total_cell_reduction(
    state: &UhiMitigationState,
    tree_grid: &TreeGrid,
    x: usize,
    y: usize,
    avg_green_roof_reduction: f32,
    avg_cool_roof_reduction: f32,
) -> f32 {
    let mut reduction: f32 = 0.0;

    // Tree planting
    reduction += tree_uhi_reduction(tree_grid.has_tree(x, y));

    // Building-level: green roofs + cool roofs (city-wide average applied per cell)
    reduction += avg_green_roof_reduction;
    reduction += avg_cool_roof_reduction;

    // Cool pavement
    if state.has_cool_pavement(x, y) {
        reduction += COOL_PAVEMENT_UHI_REDUCTION;
    }

    // Parks (radius effect)
    reduction += park_reduction_at(state, x, y);

    // Water features (radius effect)
    reduction += water_feature_reduction_at(state, x, y);

    // Permeable surfaces
    if state.has_permeable_surface(x, y) {
        reduction += PERMEABLE_SURFACE_UHI_REDUCTION;
    }

    // District cooling (radius effect)
    reduction += district_cooling_reduction_at(state, x, y);

    reduction
}
