//! Integration tests for SAVE-039: Serialize Groundwater State.
//!
//! Verifies that all groundwater-related state (grid levels, water quality /
//! contamination, and depletion tracking) persists across save/load cycles
//! so the groundwater overlay matches its pre-save state.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::groundwater::{GroundwaterGrid, WaterQualityGrid};
use crate::groundwater_depletion::GroundwaterDepletionState;
use crate::test_harness::TestCity;
use crate::Saveable;
use crate::SaveableRegistry;

// ====================================================================
// Roundtrip helper
// ====================================================================

/// Save all registered saveables, reset them to defaults, then restore
/// from the saved bytes. Operates entirely through `world_mut()`.
fn roundtrip(city: &mut TestCity) {
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();

    let extensions = registry.save_all(world);
    registry.reset_all(world);
    registry.load_all(world, &extensions);

    world.insert_resource(registry);
}

// ====================================================================
// GroundwaterGrid roundtrip with varied levels
// ====================================================================

#[test]
fn test_groundwater_grid_varied_levels_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<GroundwaterGrid>();
        // Set a gradient of values
        grid.set(0, 0, 0); // fully dry
        grid.set(10, 10, 64);
        grid.set(50, 50, 128); // mid-level
        grid.set(100, 100, 192);
        grid.set(200, 200, 255); // fully saturated
    }

    roundtrip(&mut city);

    let grid = city.resource::<GroundwaterGrid>();
    assert_eq!(grid.get(0, 0), 0, "dry cell not preserved");
    assert_eq!(grid.get(10, 10), 64);
    assert_eq!(grid.get(50, 50), 128);
    assert_eq!(grid.get(100, 100), 192);
    assert_eq!(grid.get(200, 200), 255, "saturated cell not preserved");
}

// ====================================================================
// WaterQualityGrid (contamination state) roundtrip
// ====================================================================

#[test]
fn test_water_quality_contamination_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut quality = world.resource_mut::<WaterQualityGrid>();
        // Simulate contamination: low quality near industrial area
        quality.set(30, 30, 10); // heavily contaminated
        quality.set(31, 30, 25); // moderately contaminated
        quality.set(32, 30, 49); // just below contamination threshold
        quality.set(33, 30, 50); // at threshold
        quality.set(100, 100, 255); // pure water
    }

    roundtrip(&mut city);

    let quality = city.resource::<WaterQualityGrid>();
    assert_eq!(quality.get(30, 30), 10, "heavy contamination not preserved");
    assert_eq!(quality.get(31, 30), 25, "moderate contamination not preserved");
    assert_eq!(quality.get(32, 30), 49, "near-threshold contamination not preserved");
    assert_eq!(quality.get(33, 30), 50, "threshold value not preserved");
    assert_eq!(quality.get(100, 100), 255, "pure water not preserved");
    // Unmodified cells should retain the default value of 200
    assert_eq!(quality.get(0, 0), 200, "default quality not preserved");
}

// ====================================================================
// GroundwaterDepletionState roundtrip — scalar fields
// ====================================================================

#[test]
fn test_depletion_state_scalar_fields_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<GroundwaterDepletionState>();
        state.extraction_rate = 1500.0;
        state.recharge_rate = 800.0;
        state.sustainability_ratio = 800.0 / 1500.0;
        state.critical_depletion = true;
        state.subsidence_cells = 42;
        state.well_yield_modifier = 0.65;
        state.recharge_basin_count = 3;
        state.avg_groundwater_level = 45.0;
        state.cells_at_risk = 100;
        state.over_extracted_cells = 500;
    }

    roundtrip(&mut city);

    let state = city.resource::<GroundwaterDepletionState>();
    assert!(
        (state.extraction_rate - 1500.0).abs() < f32::EPSILON,
        "extraction_rate not preserved: {}",
        state.extraction_rate,
    );
    assert!(
        (state.recharge_rate - 800.0).abs() < f32::EPSILON,
        "recharge_rate not preserved",
    );
    assert!(
        (state.sustainability_ratio - 800.0 / 1500.0).abs() < 0.001,
        "sustainability_ratio not preserved: {}",
        state.sustainability_ratio,
    );
    assert!(state.critical_depletion, "critical_depletion not preserved");
    assert_eq!(state.subsidence_cells, 42, "subsidence_cells not preserved");
    assert!(
        (state.well_yield_modifier - 0.65).abs() < f32::EPSILON,
        "well_yield_modifier not preserved",
    );
    assert_eq!(
        state.recharge_basin_count, 3,
        "recharge_basin_count not preserved",
    );
    assert!(
        (state.avg_groundwater_level - 45.0).abs() < f32::EPSILON,
        "avg_groundwater_level not preserved",
    );
    assert_eq!(state.cells_at_risk, 100, "cells_at_risk not preserved");
    assert_eq!(
        state.over_extracted_cells, 500,
        "over_extracted_cells not preserved",
    );
}

// ====================================================================
// GroundwaterDepletionState roundtrip — subsidence ticks per cell
// ====================================================================

#[test]
fn test_depletion_state_subsidence_ticks_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<GroundwaterDepletionState>();
        // Simulate some cells accumulating subsidence ticks
        state.ticks_below_threshold[0] = 10;
        state.ticks_below_threshold[100] = 25;
        state.ticks_below_threshold[5000] = 49; // one tick from subsidence
        state.ticks_below_threshold[10000] = 50; // already subsided
        // Also set a scalar to trigger save (not skip)
        state.extraction_rate = 1.0;
    }

    roundtrip(&mut city);

    let state = city.resource::<GroundwaterDepletionState>();
    assert_eq!(
        state.ticks_below_threshold[0], 10,
        "subsidence tick at cell 0 not preserved",
    );
    assert_eq!(
        state.ticks_below_threshold[100], 25,
        "subsidence tick at cell 100 not preserved",
    );
    assert_eq!(
        state.ticks_below_threshold[5000], 49,
        "near-subsidence tick not preserved",
    );
    assert_eq!(
        state.ticks_below_threshold[10000], 50,
        "subsided cell tick not preserved",
    );
    // Unmodified cells should be 0
    assert_eq!(state.ticks_below_threshold[1], 0);
}

// ====================================================================
// GroundwaterDepletionState roundtrip — previous_levels snapshot
// ====================================================================

#[test]
fn test_depletion_state_previous_levels_roundtrip() {
    let mut city = TestCity::new();

    let total = GRID_WIDTH * GRID_HEIGHT;
    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<GroundwaterDepletionState>();
        // Set a non-empty previous_levels snapshot
        state.previous_levels = vec![128; total];
        state.previous_levels[0] = 50;
        state.previous_levels[1000] = 200;
        state.previous_levels[total - 1] = 10;
        // Trigger save by marking as active
        state.extraction_rate = 1.0;
    }

    roundtrip(&mut city);

    let state = city.resource::<GroundwaterDepletionState>();
    assert_eq!(
        state.previous_levels.len(),
        total,
        "previous_levels length not preserved",
    );
    assert_eq!(state.previous_levels[0], 50);
    assert_eq!(state.previous_levels[1000], 200);
    assert_eq!(state.previous_levels[total - 1], 10);
    assert_eq!(state.previous_levels[500], 128);
}

// ====================================================================
// Default depletion state skips save (returns None)
// ====================================================================

#[test]
fn test_depletion_state_default_returns_none_on_save() {
    let state = GroundwaterDepletionState::default();
    assert!(
        state.save_to_bytes().is_none(),
        "default GroundwaterDepletionState should return None (skip saving)",
    );
}

// ====================================================================
// Corrupt bytes produce sensible default
// ====================================================================

#[test]
fn test_depletion_state_load_from_corrupt_bytes() {
    let garbage = vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB];
    let restored = GroundwaterDepletionState::load_from_bytes(&garbage);

    // Should fall back to Default
    assert_eq!(restored.extraction_rate, 0.0);
    assert_eq!(restored.recharge_rate, 0.0);
    assert!(!restored.critical_depletion);
    assert_eq!(restored.subsidence_cells, 0);
    assert_eq!(
        restored.well_yield_modifier, 1.0,
        "corrupt load should yield default well_yield_modifier",
    );
}

// ====================================================================
// Save key correctness
// ====================================================================

#[test]
fn test_depletion_state_save_key() {
    assert_eq!(GroundwaterDepletionState::SAVE_KEY, "groundwater_depletion");
}

// ====================================================================
// Key is registered in the SaveableRegistry
// ====================================================================

#[test]
fn test_groundwater_depletion_key_registered() {
    let city = TestCity::new();
    let registry = city.resource::<SaveableRegistry>();
    let registered: std::collections::HashSet<&str> =
        registry.entries.iter().map(|e| e.key.as_str()).collect();

    assert!(
        registered.contains("groundwater_depletion"),
        "groundwater_depletion key should be registered in SaveableRegistry",
    );
}

// ====================================================================
// Combined groundwater state roundtrip: grid + quality + depletion
// ====================================================================

#[test]
fn test_combined_groundwater_state_roundtrip() {
    let mut city = TestCity::new();

    // Set up non-trivial state across all three resources
    {
        let world = city.world_mut();

        // GroundwaterGrid: simulate depletion in one area
        let mut gw = world.resource_mut::<GroundwaterGrid>();
        gw.set(50, 50, 10); // nearly depleted
        gw.set(51, 50, 15);
        gw.set(52, 50, 5);

        // WaterQualityGrid: contamination near the depleted area
        let mut wq = world.resource_mut::<WaterQualityGrid>();
        wq.set(50, 50, 20); // contaminated
        wq.set(51, 50, 30);
        wq.set(52, 50, 15);

        // GroundwaterDepletionState: active depletion tracking
        let mut state = world.resource_mut::<GroundwaterDepletionState>();
        state.extraction_rate = 2000.0;
        state.recharge_rate = 500.0;
        state.critical_depletion = true;
        state.subsidence_cells = 3;
        state.well_yield_modifier = 0.4;
        // Simulate subsidence ticks at the depleted cells
        let idx_50_50 = 50 * GRID_WIDTH + 50;
        let idx_51_50 = 50 * GRID_WIDTH + 51;
        let idx_52_50 = 50 * GRID_WIDTH + 52;
        state.ticks_below_threshold[idx_50_50] = 45;
        state.ticks_below_threshold[idx_51_50] = 30;
        state.ticks_below_threshold[idx_52_50] = 50; // subsided
    }

    roundtrip(&mut city);

    // Verify all three resources survived the roundtrip
    let gw = city.resource::<GroundwaterGrid>();
    assert_eq!(gw.get(50, 50), 10);
    assert_eq!(gw.get(51, 50), 15);
    assert_eq!(gw.get(52, 50), 5);

    let wq = city.resource::<WaterQualityGrid>();
    assert_eq!(wq.get(50, 50), 20);
    assert_eq!(wq.get(51, 50), 30);
    assert_eq!(wq.get(52, 50), 15);

    let state = city.resource::<GroundwaterDepletionState>();
    assert!((state.extraction_rate - 2000.0).abs() < f32::EPSILON);
    assert!((state.recharge_rate - 500.0).abs() < f32::EPSILON);
    assert!(state.critical_depletion);
    assert_eq!(state.subsidence_cells, 3);
    assert!((state.well_yield_modifier - 0.4).abs() < f32::EPSILON);

    let idx_50_50 = 50 * GRID_WIDTH + 50;
    let idx_51_50 = 50 * GRID_WIDTH + 51;
    let idx_52_50 = 50 * GRID_WIDTH + 52;
    assert_eq!(state.ticks_below_threshold[idx_50_50], 45);
    assert_eq!(state.ticks_below_threshold[idx_51_50], 30);
    assert_eq!(state.ticks_below_threshold[idx_52_50], 50);
}

// ====================================================================
// Infinity sustainability_ratio survives roundtrip
// ====================================================================

#[test]
fn test_depletion_state_infinity_sustainability_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<GroundwaterDepletionState>();
        // No extraction => sustainability is INFINITY (default behaviour)
        state.sustainability_ratio = f32::INFINITY;
        // Set extraction_rate to trigger save
        state.recharge_rate = 100.0;
        state.extraction_rate = 0.0;
        // Need something non-default to actually trigger save
        state.critical_depletion = true;
    }

    roundtrip(&mut city);

    let state = city.resource::<GroundwaterDepletionState>();
    assert!(
        state.sustainability_ratio.is_infinite(),
        "f32::INFINITY sustainability_ratio should survive roundtrip, got {}",
        state.sustainability_ratio,
    );
}
