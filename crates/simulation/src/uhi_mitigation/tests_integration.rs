//! Integration-style and saveable tests for UHI mitigation.

use crate::config::GRID_WIDTH;
use crate::trees::TreeGrid;

use super::reductions::*;
use super::state::UhiMitigationState;

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
    use super::*;
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
