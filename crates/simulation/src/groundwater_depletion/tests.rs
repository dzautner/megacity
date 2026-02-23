//! Unit tests for groundwater depletion mechanics.

use super::*;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::groundwater::GroundwaterGrid;

/// Helper: create a `GroundwaterGrid` with all cells set to a uniform level.
fn uniform_grid(level: u8) -> GroundwaterGrid {
    GroundwaterGrid {
        levels: vec![level; GRID_WIDTH * GRID_HEIGHT],
        width: GRID_WIDTH,
        height: GRID_HEIGHT,
    }
}

/// Helper: create a default depletion state with previous_levels initialized.
fn state_with_previous(prev_level: u8) -> GroundwaterDepletionState {
    let mut state = GroundwaterDepletionState::default();
    state.previous_levels = vec![prev_level; GRID_WIDTH * GRID_HEIGHT];
    state
}

// -------------------------------------------------------------------------
// Well yield modifier tests
// -------------------------------------------------------------------------

#[test]
fn test_well_yield_at_full_level() {
    // Average level >= 50 should give full yield (1.0)
    assert!((compute_well_yield_modifier(128.0) - 1.0).abs() < f32::EPSILON);
    assert!((compute_well_yield_modifier(50.0) - 1.0).abs() < f32::EPSILON);
    assert!((compute_well_yield_modifier(255.0) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_well_yield_at_zero_level() {
    assert!((compute_well_yield_modifier(0.0)).abs() < f32::EPSILON);
}

#[test]
fn test_well_yield_diminishing_returns() {
    // At level 25 (half of threshold 50), yield should be 0.5
    let modifier = compute_well_yield_modifier(25.0);
    assert!((modifier - 0.5).abs() < f32::EPSILON);

    // At level 10, yield should be 0.2
    let modifier = compute_well_yield_modifier(10.0);
    assert!((modifier - 0.2).abs() < f32::EPSILON);
}

#[test]
fn test_well_yield_monotonic() {
    // Yield should always increase with level
    let mut prev = 0.0_f32;
    for level in 0..=60 {
        let modifier = compute_well_yield_modifier(level as f32);
        assert!(
            modifier >= prev,
            "yield should be monotonically increasing: at level {} got {} but prev was {}",
            level,
            modifier,
            prev
        );
        prev = modifier;
    }
}

// -------------------------------------------------------------------------
// Sustainability ratio tests
// -------------------------------------------------------------------------

#[test]
fn test_sustainability_ratio_balanced() {
    let ratio = compute_sustainability_ratio(100.0, 100.0);
    assert!((ratio - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_sustainability_ratio_over_extraction() {
    let ratio = compute_sustainability_ratio(50.0, 100.0);
    assert!((ratio - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_sustainability_ratio_no_extraction() {
    let ratio = compute_sustainability_ratio(50.0, 0.0);
    assert!(ratio.is_infinite());
}

#[test]
fn test_sustainability_ratio_surplus_recharge() {
    let ratio = compute_sustainability_ratio(200.0, 100.0);
    assert!((ratio - 2.0).abs() < f32::EPSILON);
}

// -------------------------------------------------------------------------
// Critical depletion tests
// -------------------------------------------------------------------------

#[test]
fn test_critical_depletion_below_threshold() {
    assert!(is_critical_depletion(30.0));
    assert!(is_critical_depletion(0.0));
    assert!(is_critical_depletion(50.9));
}

#[test]
fn test_critical_depletion_at_or_above_threshold() {
    assert!(!is_critical_depletion(51.0));
    assert!(!is_critical_depletion(128.0));
    assert!(!is_critical_depletion(255.0));
}

// -------------------------------------------------------------------------
// Default state tests
// -------------------------------------------------------------------------

#[test]
fn test_default_state() {
    let state = GroundwaterDepletionState::default();
    assert_eq!(state.extraction_rate, 0.0);
    assert_eq!(state.recharge_rate, 0.0);
    assert!(state.sustainability_ratio.is_infinite());
    assert!(!state.critical_depletion);
    assert_eq!(state.subsidence_cells, 0);
    assert!((state.well_yield_modifier - 1.0).abs() < f32::EPSILON);
    assert_eq!(state.ticks_below_threshold.len(), GRID_WIDTH * GRID_HEIGHT);
    assert!(state.ticks_below_threshold.iter().all(|&v| v == 0));
    assert!(state.previous_levels.is_empty());
    assert_eq!(state.recharge_basin_count, 0);
    assert!((state.avg_groundwater_level - 128.0).abs() < f32::EPSILON);
    assert_eq!(state.cells_at_risk, 0);
    assert_eq!(state.over_extracted_cells, 0);
}

// -------------------------------------------------------------------------
// Extraction / recharge delta computation tests
// -------------------------------------------------------------------------

#[test]
fn test_extraction_rate_computed_from_level_drop() {
    // Simulate: previous levels were 100, current levels are 90.
    // Each cell dropped by 10, so total extraction = 10 * total_cells.
    let total = GRID_WIDTH * GRID_HEIGHT;
    let grid = uniform_grid(90);
    let state = state_with_previous(100);

    // Manually run the extraction/recharge delta logic
    let mut extraction: f32 = 0.0;
    let mut recharge: f32 = 0.0;
    for i in 0..total {
        let delta = grid.levels[i] as f32 - state.previous_levels[i] as f32;
        if delta < 0.0 {
            extraction -= delta;
        } else if delta > 0.0 {
            recharge += delta;
        }
    }

    assert!((extraction - (10.0 * total as f32)).abs() < 1.0);
    assert!(recharge.abs() < f32::EPSILON);
}

#[test]
fn test_recharge_rate_computed_from_level_rise() {
    let total = GRID_WIDTH * GRID_HEIGHT;
    let grid = uniform_grid(110);
    let state = state_with_previous(100);

    let mut extraction: f32 = 0.0;
    let mut recharge: f32 = 0.0;
    for i in 0..total {
        let delta = grid.levels[i] as f32 - state.previous_levels[i] as f32;
        if delta < 0.0 {
            extraction -= delta;
        } else if delta > 0.0 {
            recharge += delta;
        }
    }

    assert!(extraction.abs() < f32::EPSILON);
    assert!((recharge - (10.0 * total as f32)).abs() < 1.0);
}

#[test]
fn test_no_delta_means_zero_rates() {
    let total = GRID_WIDTH * GRID_HEIGHT;
    let grid = uniform_grid(100);
    let state = state_with_previous(100);

    let mut extraction: f32 = 0.0;
    let mut recharge: f32 = 0.0;
    for i in 0..total {
        let delta = grid.levels[i] as f32 - state.previous_levels[i] as f32;
        if delta < 0.0 {
            extraction -= delta;
        } else if delta > 0.0 {
            recharge += delta;
        }
    }

    assert!(extraction.abs() < f32::EPSILON);
    assert!(recharge.abs() < f32::EPSILON);
}

// -------------------------------------------------------------------------
// Subsidence tracking tests
// -------------------------------------------------------------------------

#[test]
fn test_subsidence_counter_increments_below_threshold() {
    let mut ticks = vec![0u16; 4];
    let levels: Vec<u8> = vec![10, 15, 5, 19]; // all < SUBSIDENCE_THRESHOLD (20)

    for i in 0..4 {
        if levels[i] < SUBSIDENCE_THRESHOLD {
            ticks[i] = ticks[i].saturating_add(1);
        } else {
            ticks[i] = 0;
        }
    }

    assert_eq!(ticks, vec![1, 1, 1, 1]);
}

#[test]
fn test_subsidence_counter_resets_above_threshold() {
    let mut ticks = vec![30u16; 4];
    let levels: Vec<u8> = vec![10, 15, 50, 100]; // index 0,1 below threshold; 2,3 above

    for i in 0..4 {
        if levels[i] < SUBSIDENCE_THRESHOLD {
            ticks[i] = ticks[i].saturating_add(1);
        } else {
            ticks[i] = 0;
        }
    }

    assert_eq!(ticks[0], 31); // still below, incremented
    assert_eq!(ticks[1], 31); // still below, incremented
    assert_eq!(ticks[2], 0); // above threshold, reset
    assert_eq!(ticks[3], 0); // above threshold, reset
}

#[test]
fn test_subsidence_triggers_at_threshold_ticks() {
    let mut ticks = vec![49u16; 1];
    let levels: Vec<u8> = vec![10]; // below threshold

    // Tick once more
    if levels[0] < SUBSIDENCE_THRESHOLD && ticks[0] < SUBSIDENCE_TICKS {
        ticks[0] = ticks[0].saturating_add(1);
    }

    assert_eq!(ticks[0], SUBSIDENCE_TICKS);

    // Count subsided
    let subsided = ticks.iter().filter(|&&t| t >= SUBSIDENCE_TICKS).count();
    assert_eq!(subsided, 1);
}

#[test]
fn test_subsidence_counter_freezes_after_trigger() {
    let mut ticks = vec![SUBSIDENCE_TICKS; 1];
    let levels: Vec<u8> = vec![10]; // still below threshold

    // Should not increment beyond SUBSIDENCE_TICKS
    if levels[0] < SUBSIDENCE_THRESHOLD && ticks[0] < SUBSIDENCE_TICKS {
        ticks[0] = ticks[0].saturating_add(1);
    }

    assert_eq!(ticks[0], SUBSIDENCE_TICKS); // unchanged, already at threshold
}

// -------------------------------------------------------------------------
// Over-extraction indicator tests
// -------------------------------------------------------------------------

#[test]
fn test_over_extracted_cells_counted() {
    let total = GRID_WIDTH * GRID_HEIGHT;
    let mut grid = uniform_grid(128);

    // Set first 100 cells to below critical level
    for i in 0..100 {
        grid.levels[i] = 30;
    }

    let mut over_extracted: u32 = 0;
    for i in 0..total {
        if grid.levels[i] < GROUNDWATER_CRITICAL_LEVEL {
            over_extracted += 1;
        }
    }

    assert_eq!(over_extracted, 100);
}

// -------------------------------------------------------------------------
// Average groundwater level tests
// -------------------------------------------------------------------------

#[test]
fn test_avg_groundwater_level_uniform() {
    let total = GRID_WIDTH * GRID_HEIGHT;
    let grid = uniform_grid(200);

    let sum: u64 = grid.levels.iter().map(|&v| v as u64).sum();
    let avg = sum as f32 / total as f32;

    assert!((avg - 200.0).abs() < f32::EPSILON);
}

#[test]
fn test_avg_groundwater_level_mixed() {
    let total = GRID_WIDTH * GRID_HEIGHT;
    let mut grid = uniform_grid(100);

    // Set first half to 0, second half stays 100
    let half = total / 2;
    for i in 0..half {
        grid.levels[i] = 0;
    }

    let sum: u64 = grid.levels.iter().map(|&v| v as u64).sum();
    let avg = sum as f32 / total as f32;

    // Average should be 50.0 (half at 0, half at 100)
    assert!((avg - 50.0).abs() < 0.1);
}

// -------------------------------------------------------------------------
// Constants sanity tests
// -------------------------------------------------------------------------

#[test]
fn test_constants_are_sensible() {
    // Critical level is 20% of 255
    assert_eq!(GROUNDWATER_CRITICAL_LEVEL, 51);
    // Subsidence threshold is below critical
    assert!(SUBSIDENCE_THRESHOLD < GROUNDWATER_CRITICAL_LEVEL);
    // Subsidence requires sustained depletion
    assert!(SUBSIDENCE_TICKS > 0);
    // Well yield reduction starts at a reasonable level
    assert!(WELL_YIELD_REDUCTION_THRESHOLD > SUBSIDENCE_THRESHOLD);
}

// -------------------------------------------------------------------------
// Integration-style test: full depletion cycle
// -------------------------------------------------------------------------

#[test]
fn test_full_depletion_cycle_simulation() {
    // Simulate a scenario where groundwater drops over multiple ticks.
    // Start at level 60, drop by 5 each tick for several ticks.
    let total = GRID_WIDTH * GRID_HEIGHT;

    let mut state = GroundwaterDepletionState::default();
    let mut current_level: u8 = 60;

    // Tick 0: initialize snapshot
    state.previous_levels = vec![current_level; total];

    // Simulate 10 ticks of extraction (drop by 5 each tick)
    for tick in 0..10 {
        let new_level = current_level.saturating_sub(5);
        let grid = uniform_grid(new_level);

        // Compute extraction
        let mut extraction: f32 = 0.0;
        let mut recharge: f32 = 0.0;
        for i in 0..total {
            let delta = grid.levels[i] as f32 - state.previous_levels[i] as f32;
            if delta < 0.0 {
                extraction -= delta;
            } else if delta > 0.0 {
                recharge += delta;
            }
        }
        state.extraction_rate = extraction;
        state.recharge_rate = recharge;
        state.sustainability_ratio = compute_sustainability_ratio(recharge, extraction);

        // Compute average
        let sum: u64 = grid.levels.iter().map(|&v| v as u64).sum();
        state.avg_groundwater_level = sum as f32 / total as f32;

        // Well yield
        state.well_yield_modifier = compute_well_yield_modifier(state.avg_groundwater_level);

        // Critical depletion
        state.critical_depletion = is_critical_depletion(state.avg_groundwater_level);

        // Subsidence tracking
        for i in 0..total {
            if grid.levels[i] < SUBSIDENCE_THRESHOLD {
                if state.ticks_below_threshold[i] < SUBSIDENCE_TICKS {
                    state.ticks_below_threshold[i] =
                        state.ticks_below_threshold[i].saturating_add(1);
                }
            } else {
                state.ticks_below_threshold[i] = 0;
            }
        }
        state.subsidence_cells = state
            .ticks_below_threshold
            .iter()
            .filter(|&&t| t >= SUBSIDENCE_TICKS)
            .count() as u32;

        state.previous_levels = grid.levels.clone();
        current_level = new_level;

        // Verify extraction is happening
        if tick < 10 {
            assert!(
                state.extraction_rate > 0.0,
                "tick {}: extraction should be positive",
                tick
            );
            assert!(
                state.sustainability_ratio < 1.0,
                "tick {}: ratio should indicate over-extraction",
                tick
            );
        }
    }

    // After 10 ticks: level went from 60 -> 10
    assert_eq!(current_level, 10);

    // At level 10, well yield should be reduced
    assert!(state.well_yield_modifier < 1.0);
    assert!((state.well_yield_modifier - 0.2).abs() < f32::EPSILON);

    // At level 10 (< 51), critical depletion should be active
    assert!(state.critical_depletion);

    // At level 10 (< 20), cells are at subsidence risk but only accumulated
    // a few ticks so far (not yet at SUBSIDENCE_TICKS = 50)
    assert_eq!(state.subsidence_cells, 0);
}
