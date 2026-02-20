//! Urban Heat Island (UHI) mitigation measures (WEATHER-009).
//!
//! Provides several mitigation options that reduce `UhiGrid` values in affected
//! cells, lowering the urban heat island effect:
//!
//! - **Tree planting**: -1.5F UHI per tree cell (passive, from `TreeGrid`)
//! - **Green roofs**: -2.0F, building upgrade $15K/building
//! - **Cool (white) roofs**: -1.5F, building upgrade $3K/building
//! - **Cool pavement**: -1.0F, road upgrade $5K/cell
//! - **Parks**: -3.0F in radius 2, $10K/cell
//! - **Water features (fountains)**: -2.0F, placeable $8K each
//! - **Permeable surfaces**: -0.5F, $4K/cell
//! - **District cooling**: -1.0F in radius 3, large facility $50K each
//!
//! Each mitigation reduces `UhiGrid` values in affected cells after the base
//! UHI calculation runs.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::trees::TreeGrid;
use crate::urban_heat_island::UhiGrid;
use crate::TickCounter;

// =============================================================================
// Constants
// =============================================================================

/// UHI update frequency -- must match `urban_heat_island::UHI_UPDATE_INTERVAL`.
const UHI_MITIGATION_UPDATE_INTERVAL: u64 = 30;

/// UHI reduction from a tree cell (Fahrenheit).
const TREE_UHI_REDUCTION: f32 = 1.5;

/// UHI reduction from a green roof (Fahrenheit).
const GREEN_ROOF_UHI_REDUCTION: f32 = 2.0;

/// Cost per building for green roof upgrade.
pub const GREEN_ROOF_COST: f64 = 15_000.0;

/// UHI reduction from a cool (white) roof (Fahrenheit).
const COOL_ROOF_UHI_REDUCTION: f32 = 1.5;

/// Cost per building for cool roof upgrade.
pub const COOL_ROOF_COST: f64 = 3_000.0;

/// UHI reduction from cool pavement on a road cell (Fahrenheit).
const COOL_PAVEMENT_UHI_REDUCTION: f32 = 1.0;

/// Cost per cell for cool pavement upgrade.
pub const COOL_PAVEMENT_COST: f64 = 5_000.0;

/// UHI reduction from a park cell in its radius (Fahrenheit).
const PARK_UHI_REDUCTION: f32 = 3.0;

/// Radius of park cooling effect (cells).
const PARK_RADIUS: i32 = 2;

/// Cost per cell for park placement.
pub const PARK_COST: f64 = 10_000.0;

/// UHI reduction from a water feature / fountain (Fahrenheit).
const WATER_FEATURE_UHI_REDUCTION: f32 = 2.0;

/// Cost per water feature.
pub const WATER_FEATURE_COST: f64 = 8_000.0;

/// UHI reduction from permeable surfaces (Fahrenheit).
const PERMEABLE_SURFACE_UHI_REDUCTION: f32 = 0.5;

/// Cost per cell for permeable surfaces.
pub const PERMEABLE_SURFACE_COST: f64 = 4_000.0;

/// UHI reduction from district cooling in its radius (Fahrenheit).
const DISTRICT_COOLING_UHI_REDUCTION: f32 = 1.0;

/// Radius of district cooling effect (cells).
const DISTRICT_COOLING_RADIUS: i32 = 3;

/// Cost per district cooling facility.
pub const DISTRICT_COOLING_COST: f64 = 50_000.0;

// =============================================================================
// Resources
// =============================================================================

/// Tracks all UHI mitigation measures deployed across the city.
///
/// Grid-level mitigations (cool pavement, parks, permeable surfaces) are stored
/// as boolean grids. Building-level mitigations (green roofs, cool roofs) are
/// stored as counts. Point mitigations (water features, district cooling) are
/// stored as coordinate lists.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct UhiMitigationState {
    // --- Building-level mitigations ---
    /// Number of buildings upgraded with green roofs.
    pub green_roof_count: u32,
    /// Number of buildings upgraded with cool (white) roofs.
    pub cool_roof_count: u32,

    // --- Grid-level mitigations (per-cell booleans) ---
    /// Cells with cool pavement applied.
    pub cool_pavement_cells: Vec<bool>,
    /// Cells designated as parks for UHI mitigation.
    pub park_cells: Vec<bool>,
    /// Cells with permeable surfaces applied.
    pub permeable_surface_cells: Vec<bool>,

    // --- Point mitigations ---
    /// Locations of water features (fountains). Each entry is `(x, y)`.
    pub water_features: Vec<(usize, usize)>,
    /// Locations of district cooling facilities. Each entry is `(x, y)`.
    pub district_cooling_facilities: Vec<(usize, usize)>,

    // --- Cost tracking ---
    /// Total cumulative cost of all UHI mitigation measures.
    pub total_cost: f64,

    // --- Derived (computed each update) ---
    /// Total UHI reduction applied across all cells this tick (for stats/UI).
    pub total_cells_mitigated: u32,
}

impl Default for UhiMitigationState {
    fn default() -> Self {
        let grid_size = GRID_WIDTH * GRID_HEIGHT;
        Self {
            green_roof_count: 0,
            cool_roof_count: 0,
            cool_pavement_cells: vec![false; grid_size],
            park_cells: vec![false; grid_size],
            permeable_surface_cells: vec![false; grid_size],
            water_features: Vec::new(),
            district_cooling_facilities: Vec::new(),
            total_cost: 0.0,
            total_cells_mitigated: 0,
        }
    }
}

impl UhiMitigationState {
    /// Check if cool pavement is applied at `(x, y)`.
    #[inline]
    pub fn has_cool_pavement(&self, x: usize, y: usize) -> bool {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.cool_pavement_cells[y * GRID_WIDTH + x]
        } else {
            false
        }
    }

    /// Check if a park is placed at `(x, y)`.
    #[inline]
    pub fn has_park(&self, x: usize, y: usize) -> bool {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.park_cells[y * GRID_WIDTH + x]
        } else {
            false
        }
    }

    /// Check if permeable surfaces are applied at `(x, y)`.
    #[inline]
    pub fn has_permeable_surface(&self, x: usize, y: usize) -> bool {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.permeable_surface_cells[y * GRID_WIDTH + x]
        } else {
            false
        }
    }
}

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
                && (nx as usize) < GRID_WIDTH
                && (ny as usize) < GRID_HEIGHT
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

// =============================================================================
// System
// =============================================================================

/// System that applies UHI mitigation reductions to the `UhiGrid` after the
/// base UHI calculation has run.
///
/// Runs at the same interval as the UHI update system and must be scheduled
/// after `update_uhi_grid`.
pub fn apply_uhi_mitigation(
    tick: Res<TickCounter>,
    mut uhi: ResMut<UhiGrid>,
    tree_grid: Res<TreeGrid>,
    mitigation: Res<UhiMitigationState>,
    buildings: Query<&crate::buildings::Building>,
) {
    if !tick.0.is_multiple_of(UHI_MITIGATION_UPDATE_INTERVAL) {
        return;
    }

    let total_buildings = buildings.iter().count() as u32;
    let avg_green = green_roof_reduction(mitigation.green_roof_count, total_buildings);
    let avg_cool = cool_roof_reduction(mitigation.cool_roof_count, total_buildings);

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let reduction =
                total_cell_reduction(&mitigation, &tree_grid, x, y, avg_green, avg_cool);
            if reduction > 0.0 {
                let current = uhi.get(x, y);
                uhi.set(x, y, current - reduction);
            }
        }
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for UhiMitigationState {
    const SAVE_KEY: &'static str = "uhi_mitigation";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if nothing has been deployed
        if self.green_roof_count == 0
            && self.cool_roof_count == 0
            && self.water_features.is_empty()
            && self.district_cooling_facilities.is_empty()
            && self.total_cost == 0.0
            && !self.cool_pavement_cells.iter().any(|&v| v)
            && !self.park_cells.iter().any(|&v| v)
            && !self.permeable_surface_cells.iter().any(|&v| v)
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

pub struct UhiMitigationPlugin;

impl Plugin for UhiMitigationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UhiMitigationState>().add_systems(
            FixedUpdate,
            apply_uhi_mitigation.after(crate::urban_heat_island::update_uhi_grid),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<UhiMitigationState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Default state tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_state() {
        let state = UhiMitigationState::default();
        assert_eq!(state.green_roof_count, 0);
        assert_eq!(state.cool_roof_count, 0);
        assert!(state.water_features.is_empty());
        assert!(state.district_cooling_facilities.is_empty());
        assert_eq!(state.total_cost, 0.0);
        assert_eq!(state.total_cells_mitigated, 0);
        assert_eq!(state.cool_pavement_cells.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(state.park_cells.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(
            state.permeable_surface_cells.len(),
            GRID_WIDTH * GRID_HEIGHT
        );
    }

    // -------------------------------------------------------------------------
    // Tree UHI reduction tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_tree_uhi_reduction_with_tree() {
        let reduction = tree_uhi_reduction(true);
        assert!(
            (reduction - 1.5).abs() < f32::EPSILON,
            "tree should reduce UHI by 1.5F, got {}",
            reduction
        );
    }

    #[test]
    fn test_tree_uhi_reduction_no_tree() {
        let reduction = tree_uhi_reduction(false);
        assert!(
            reduction.abs() < f32::EPSILON,
            "no tree = no reduction, got {}",
            reduction
        );
    }

    // -------------------------------------------------------------------------
    // Green roof reduction tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_green_roof_no_buildings() {
        let reduction = green_roof_reduction(0, 0);
        assert!(
            reduction.abs() < f32::EPSILON,
            "no buildings = no reduction"
        );
    }

    #[test]
    fn test_green_roof_no_upgrades() {
        let reduction = green_roof_reduction(0, 100);
        assert!(reduction.abs() < f32::EPSILON, "no upgrades = no reduction");
    }

    #[test]
    fn test_green_roof_all_upgraded() {
        let reduction = green_roof_reduction(100, 100);
        assert!(
            (reduction - GREEN_ROOF_UHI_REDUCTION).abs() < f32::EPSILON,
            "all upgraded = full 2.0F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_green_roof_half_upgraded() {
        let reduction = green_roof_reduction(50, 100);
        assert!(
            (reduction - 1.0).abs() < f32::EPSILON,
            "50% upgraded = 1.0F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_green_roof_capped() {
        let reduction = green_roof_reduction(200, 100);
        assert!(
            (reduction - GREEN_ROOF_UHI_REDUCTION).abs() < f32::EPSILON,
            "capped at full reduction, got {}",
            reduction
        );
    }

    // -------------------------------------------------------------------------
    // Cool roof reduction tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cool_roof_no_buildings() {
        let reduction = cool_roof_reduction(0, 0);
        assert!(
            reduction.abs() < f32::EPSILON,
            "no buildings = no reduction"
        );
    }

    #[test]
    fn test_cool_roof_no_upgrades() {
        let reduction = cool_roof_reduction(0, 100);
        assert!(reduction.abs() < f32::EPSILON, "no upgrades = no reduction");
    }

    #[test]
    fn test_cool_roof_all_upgraded() {
        let reduction = cool_roof_reduction(100, 100);
        assert!(
            (reduction - COOL_ROOF_UHI_REDUCTION).abs() < f32::EPSILON,
            "all upgraded = full 1.5F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_cool_roof_half_upgraded() {
        let reduction = cool_roof_reduction(50, 100);
        assert!(
            (reduction - 0.75).abs() < f32::EPSILON,
            "50% upgraded = 0.75F reduction, got {}",
            reduction
        );
    }

    // -------------------------------------------------------------------------
    // Cool pavement tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cool_pavement_cell_check() {
        let mut state = UhiMitigationState::default();
        assert!(!state.has_cool_pavement(10, 10));
        state.cool_pavement_cells[10 * GRID_WIDTH + 10] = true;
        assert!(state.has_cool_pavement(10, 10));
    }

    #[test]
    fn test_cool_pavement_out_of_bounds() {
        let state = UhiMitigationState::default();
        assert!(!state.has_cool_pavement(9999, 9999));
    }

    // -------------------------------------------------------------------------
    // Park reduction tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_park_no_parks() {
        let state = UhiMitigationState::default();
        let reduction = park_reduction_at(&state, 50, 50);
        assert!(reduction.abs() < f32::EPSILON, "no parks = no reduction");
    }

    #[test]
    fn test_park_at_cell() {
        let mut state = UhiMitigationState::default();
        state.park_cells[50 * GRID_WIDTH + 50] = true;
        let reduction = park_reduction_at(&state, 50, 50);
        assert!(
            (reduction - PARK_UHI_REDUCTION).abs() < f32::EPSILON,
            "park at cell should give 3.0F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_park_within_radius() {
        let mut state = UhiMitigationState::default();
        state.park_cells[50 * GRID_WIDTH + 50] = true;
        // Cell 2 away (within PARK_RADIUS=2)
        let reduction = park_reduction_at(&state, 52, 50);
        assert!(
            (reduction - PARK_UHI_REDUCTION).abs() < f32::EPSILON,
            "within radius should get reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_park_outside_radius() {
        let mut state = UhiMitigationState::default();
        state.park_cells[50 * GRID_WIDTH + 50] = true;
        // Cell 3 away (outside PARK_RADIUS=2)
        let reduction = park_reduction_at(&state, 53, 50);
        assert!(
            reduction.abs() < f32::EPSILON,
            "outside radius = no reduction, got {}",
            reduction
        );
    }

    // -------------------------------------------------------------------------
    // Water feature reduction tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_water_feature_no_features() {
        let state = UhiMitigationState::default();
        let reduction = water_feature_reduction_at(&state, 50, 50);
        assert!(reduction.abs() < f32::EPSILON);
    }

    #[test]
    fn test_water_feature_at_cell() {
        let mut state = UhiMitigationState::default();
        state.water_features.push((50, 50));
        let reduction = water_feature_reduction_at(&state, 50, 50);
        assert!(
            (reduction - WATER_FEATURE_UHI_REDUCTION).abs() < f32::EPSILON,
            "water feature at cell = 2.0F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_water_feature_adjacent() {
        let mut state = UhiMitigationState::default();
        state.water_features.push((50, 50));
        let reduction = water_feature_reduction_at(&state, 51, 51);
        assert!(
            (reduction - WATER_FEATURE_UHI_REDUCTION).abs() < f32::EPSILON,
            "adjacent to water feature = 2.0F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_water_feature_too_far() {
        let mut state = UhiMitigationState::default();
        state.water_features.push((50, 50));
        let reduction = water_feature_reduction_at(&state, 52, 52);
        assert!(
            reduction.abs() < f32::EPSILON,
            "too far from water feature = no reduction, got {}",
            reduction
        );
    }

    // -------------------------------------------------------------------------
    // Permeable surface tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_permeable_surface_cell_check() {
        let mut state = UhiMitigationState::default();
        assert!(!state.has_permeable_surface(10, 10));
        state.permeable_surface_cells[10 * GRID_WIDTH + 10] = true;
        assert!(state.has_permeable_surface(10, 10));
    }

    // -------------------------------------------------------------------------
    // District cooling reduction tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_district_cooling_no_facilities() {
        let state = UhiMitigationState::default();
        let reduction = district_cooling_reduction_at(&state, 50, 50);
        assert!(reduction.abs() < f32::EPSILON);
    }

    #[test]
    fn test_district_cooling_at_facility() {
        let mut state = UhiMitigationState::default();
        state.district_cooling_facilities.push((50, 50));
        let reduction = district_cooling_reduction_at(&state, 50, 50);
        assert!(
            (reduction - DISTRICT_COOLING_UHI_REDUCTION).abs() < f32::EPSILON,
            "at facility = 1.0F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_district_cooling_within_radius() {
        let mut state = UhiMitigationState::default();
        state.district_cooling_facilities.push((50, 50));
        // Cell 3 away (within DISTRICT_COOLING_RADIUS=3)
        let reduction = district_cooling_reduction_at(&state, 53, 50);
        assert!(
            (reduction - DISTRICT_COOLING_UHI_REDUCTION).abs() < f32::EPSILON,
            "within radius = reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_district_cooling_outside_radius() {
        let mut state = UhiMitigationState::default();
        state.district_cooling_facilities.push((50, 50));
        // Cell 4 away (outside DISTRICT_COOLING_RADIUS=3)
        let reduction = district_cooling_reduction_at(&state, 54, 50);
        assert!(
            reduction.abs() < f32::EPSILON,
            "outside radius = no reduction, got {}",
            reduction
        );
    }

    // -------------------------------------------------------------------------
    // Total cell reduction tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_total_cell_reduction_empty() {
        let state = UhiMitigationState::default();
        let tree_grid = TreeGrid::default();
        let reduction = total_cell_reduction(&state, &tree_grid, 50, 50, 0.0, 0.0);
        assert!(
            reduction.abs() < f32::EPSILON,
            "no mitigations = no reduction"
        );
    }

    #[test]
    fn test_total_cell_reduction_all_mitigations() {
        let mut state = UhiMitigationState::default();
        let mut tree_grid = TreeGrid::default();

        // Tree at (50, 50)
        tree_grid.set(50, 50, true);
        // Cool pavement at (50, 50)
        state.cool_pavement_cells[50 * GRID_WIDTH + 50] = true;
        // Park at (50, 50)
        state.park_cells[50 * GRID_WIDTH + 50] = true;
        // Water feature at (50, 50)
        state.water_features.push((50, 50));
        // Permeable surface at (50, 50)
        state.permeable_surface_cells[50 * GRID_WIDTH + 50] = true;
        // District cooling at (50, 50)
        state.district_cooling_facilities.push((50, 50));

        // Green roof avg = 1.0F, cool roof avg = 0.5F
        let reduction = total_cell_reduction(&state, &tree_grid, 50, 50, 1.0, 0.5);

        // Expected: 1.5 (tree) + 1.0 (green roof avg) + 0.5 (cool roof avg)
        //         + 1.0 (cool pavement) + 3.0 (park) + 2.0 (water feature)
        //         + 0.5 (permeable) + 1.0 (district cooling) = 10.5
        let expected = 1.5 + 1.0 + 0.5 + 1.0 + 3.0 + 2.0 + 0.5 + 1.0;
        assert!(
            (reduction - expected).abs() < 0.01,
            "expected {} total reduction, got {}",
            expected,
            reduction
        );
    }

    #[test]
    fn test_total_cell_reduction_tree_only() {
        let state = UhiMitigationState::default();
        let mut tree_grid = TreeGrid::default();
        tree_grid.set(50, 50, true);

        let reduction = total_cell_reduction(&state, &tree_grid, 50, 50, 0.0, 0.0);
        assert!(
            (reduction - TREE_UHI_REDUCTION).abs() < f32::EPSILON,
            "tree only = 1.5F, got {}",
            reduction
        );
    }

    // -------------------------------------------------------------------------
    // Cost constant tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cost_constants() {
        assert!((GREEN_ROOF_COST - 15_000.0).abs() < f64::EPSILON);
        assert!((COOL_ROOF_COST - 3_000.0).abs() < f64::EPSILON);
        assert!((COOL_PAVEMENT_COST - 5_000.0).abs() < f64::EPSILON);
        assert!((PARK_COST - 10_000.0).abs() < f64::EPSILON);
        assert!((WATER_FEATURE_COST - 8_000.0).abs() < f64::EPSILON);
        assert!((PERMEABLE_SURFACE_COST - 4_000.0).abs() < f64::EPSILON);
        assert!((DISTRICT_COOLING_COST - 50_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reduction_constants() {
        assert!((TREE_UHI_REDUCTION - 1.5).abs() < f32::EPSILON);
        assert!((GREEN_ROOF_UHI_REDUCTION - 2.0).abs() < f32::EPSILON);
        assert!((COOL_ROOF_UHI_REDUCTION - 1.5).abs() < f32::EPSILON);
        assert!((COOL_PAVEMENT_UHI_REDUCTION - 1.0).abs() < f32::EPSILON);
        assert!((PARK_UHI_REDUCTION - 3.0).abs() < f32::EPSILON);
        assert!((WATER_FEATURE_UHI_REDUCTION - 2.0).abs() < f32::EPSILON);
        assert!((PERMEABLE_SURFACE_UHI_REDUCTION - 0.5).abs() < f32::EPSILON);
        assert!((DISTRICT_COOLING_UHI_REDUCTION - 1.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Saveable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let state = UhiMitigationState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "default state should skip saving"
        );
    }

    #[test]
    fn test_saveable_saves_when_modified() {
        use crate::Saveable;
        let mut state = UhiMitigationState::default();
        state.green_roof_count = 5;
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = UhiMitigationState::default();
        state.green_roof_count = 10;
        state.cool_roof_count = 20;
        state.cool_pavement_cells[100] = true;
        state.park_cells[200] = true;
        state.permeable_surface_cells[300] = true;
        state.water_features.push((50, 60));
        state.district_cooling_facilities.push((70, 80));
        state.total_cost = 500_000.0;

        let bytes = state
            .save_to_bytes()
            .expect("should serialize non-default state");
        let restored = UhiMitigationState::load_from_bytes(&bytes);

        assert_eq!(restored.green_roof_count, 10);
        assert_eq!(restored.cool_roof_count, 20);
        assert!(restored.cool_pavement_cells[100]);
        assert!(restored.park_cells[200]);
        assert!(restored.permeable_surface_cells[300]);
        assert_eq!(restored.water_features.len(), 1);
        assert_eq!(restored.water_features[0], (50, 60));
        assert_eq!(restored.district_cooling_facilities.len(), 1);
        assert_eq!(restored.district_cooling_facilities[0], (70, 80));
        assert!((restored.total_cost - 500_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_saveable_corrupted_bytes() {
        use crate::Saveable;
        let garbage = vec![0xFF, 0xFE, 0xFD];
        let restored = UhiMitigationState::load_from_bytes(&garbage);
        // Should produce default state on corrupt data
        assert_eq!(restored.green_roof_count, 0);
        assert_eq!(restored.cool_roof_count, 0);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(UhiMitigationState::SAVE_KEY, "uhi_mitigation");
    }

    // -------------------------------------------------------------------------
    // Integration-style tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_park_provides_area_cooling() {
        // A park should provide cooling to cells within its radius
        let mut state = UhiMitigationState::default();
        state.park_cells[50 * GRID_WIDTH + 50] = true;

        // Check cells at various distances
        for d in 0..=PARK_RADIUS {
            let reduction = park_reduction_at(&state, 50 + d as usize, 50);
            assert!(
                (reduction - PARK_UHI_REDUCTION).abs() < f32::EPSILON,
                "park radius {} should cool, got {}",
                d,
                reduction
            );
        }
        // Just outside radius
        let outside = park_reduction_at(&state, 50 + PARK_RADIUS as usize + 1, 50);
        assert!(
            outside.abs() < f32::EPSILON,
            "outside park radius should not cool"
        );
    }

    #[test]
    fn test_district_cooling_provides_area_cooling() {
        let mut state = UhiMitigationState::default();
        state.district_cooling_facilities.push((50, 50));

        for d in 0..=DISTRICT_COOLING_RADIUS {
            let reduction = district_cooling_reduction_at(&state, 50 + d as usize, 50);
            assert!(
                (reduction - DISTRICT_COOLING_UHI_REDUCTION).abs() < f32::EPSILON,
                "district cooling radius {} should cool, got {}",
                d,
                reduction
            );
        }
        let outside =
            district_cooling_reduction_at(&state, 50 + DISTRICT_COOLING_RADIUS as usize + 1, 50);
        assert!(
            outside.abs() < f32::EPSILON,
            "outside district cooling radius should not cool"
        );
    }

    #[test]
    fn test_multiple_water_features() {
        let mut state = UhiMitigationState::default();
        state.water_features.push((10, 10));
        state.water_features.push((50, 50));

        // Near first feature
        let r1 = water_feature_reduction_at(&state, 10, 10);
        assert!((r1 - WATER_FEATURE_UHI_REDUCTION).abs() < f32::EPSILON);

        // Near second feature
        let r2 = water_feature_reduction_at(&state, 50, 50);
        assert!((r2 - WATER_FEATURE_UHI_REDUCTION).abs() < f32::EPSILON);

        // Far from both
        let r3 = water_feature_reduction_at(&state, 30, 30);
        assert!(r3.abs() < f32::EPSILON);
    }
}
