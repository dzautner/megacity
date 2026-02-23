//! Unit tests for UHI mitigation.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::trees::TreeGrid;

use super::reductions::*;
use super::state::UhiMitigationState;

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
