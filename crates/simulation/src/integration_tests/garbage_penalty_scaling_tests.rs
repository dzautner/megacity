//! Integration tests verifying that garbage happiness penalties scale gradually
//! instead of using binary thresholds (issue #1961).

use crate::garbage::GarbageGrid;
use crate::test_harness::TestCity;
use crate::waste_effects::WasteAccumulation;

// ====================================================================
// waste_happiness_penalty scaling (pure function tests)
// ====================================================================

#[test]
fn test_waste_happiness_penalty_zero_waste_gives_zero_penalty() {
    let penalty = crate::waste_effects::waste_happiness_penalty(0.0);
    assert_eq!(penalty, 0.0, "Zero waste should give zero penalty");
}

#[test]
fn test_waste_happiness_penalty_low_waste_gives_partial_penalty() {
    // 100 lbs out of 500 max => 20% of full penalty => -1.0
    let penalty = crate::waste_effects::waste_happiness_penalty(100.0);
    assert!(
        penalty > -2.0 && penalty < 0.0,
        "100 lbs should give a small partial penalty, got {}",
        penalty
    );
    assert!(
        (penalty - (-1.0)).abs() < 0.01,
        "100 lbs should give roughly -1.0, got {}",
        penalty
    );
}

#[test]
fn test_waste_happiness_penalty_high_waste_gives_full_penalty() {
    let penalty = crate::waste_effects::waste_happiness_penalty(500.0);
    assert_eq!(penalty, -5.0, "500+ lbs should give full -5 penalty");

    let penalty_over = crate::waste_effects::waste_happiness_penalty(1000.0);
    assert_eq!(
        penalty_over, -5.0,
        "Over 500 lbs should still cap at -5 penalty"
    );
}

// ====================================================================
// GarbageGrid and WasteAccumulation resource defaults
// ====================================================================

#[test]
fn test_garbage_grid_default_is_zero() {
    let city = TestCity::new();
    let grid = city.resource::<GarbageGrid>();
    assert_eq!(
        grid.get(50, 50),
        0,
        "Default garbage grid should be zero"
    );
}

#[test]
fn test_waste_accumulation_default_is_zero() {
    let city = TestCity::new();
    let acc = city.resource::<WasteAccumulation>();
    assert_eq!(
        acc.get(50, 50),
        0.0,
        "Default waste accumulation should be zero"
    );
}

// ====================================================================
// Scaling linearity verification
// ====================================================================

#[test]
fn test_waste_penalty_scales_linearly() {
    // Verify that the penalty at 250 lbs is half the penalty at 500 lbs
    let half = crate::waste_effects::waste_happiness_penalty(250.0);
    let full = crate::waste_effects::waste_happiness_penalty(500.0);
    assert!(
        (half - full / 2.0).abs() < 0.01,
        "Penalty at 250 lbs ({}) should be half of penalty at 500 lbs ({})",
        half,
        full
    );
}

#[test]
fn test_waste_penalty_monotonically_increasing() {
    let mut prev = crate::waste_effects::waste_happiness_penalty(0.0);
    for lbs in (50..=500).step_by(50) {
        let current = crate::waste_effects::waste_happiness_penalty(lbs as f32);
        assert!(
            current <= prev,
            "Penalty should get more negative: at {} lbs got {}, prev was {}",
            lbs,
            current,
            prev
        );
        prev = current;
    }
}
