//! Integration tests for PROG-009: Hope and Discontent Dual Meters.

use crate::economy::CityBudget;
use crate::hope_discontent::{CrisisState, HopeDiscontent};
use crate::test_harness::TestCity;
use crate::Saveable;

// -----------------------------------------------------------------------
// Initialization
// -----------------------------------------------------------------------

#[test]
fn test_hope_discontent_initialized_with_defaults() {
    let city = TestCity::new();
    let hd = city.resource::<HopeDiscontent>();
    assert!(
        (hd.hope - 0.5).abs() < f32::EPSILON,
        "Hope should default to 0.5, got {}",
        hd.hope
    );
    assert!(
        (hd.discontent - 0.2).abs() < f32::EPSILON,
        "Discontent should default to 0.2, got {}",
        hd.discontent
    );
    assert_eq!(hd.crisis_state, CrisisState::Normal);
}

// -----------------------------------------------------------------------
// Economy effects (budget persists across ticks — no other system resets it)
// -----------------------------------------------------------------------

#[test]
fn test_good_economy_increases_hope() {
    let mut city = TestCity::new().with_budget(50_000.0);

    {
        let world = city.world_mut();
        let mut budget = world.resource_mut::<CityBudget>();
        budget.monthly_income = 5000.0;
        budget.monthly_expenses = 1000.0;
    }

    let initial_hope = city.resource::<HopeDiscontent>().hope;
    city.tick_slow_cycles(3);
    let new_hope = city.resource::<HopeDiscontent>().hope;

    assert!(
        new_hope > initial_hope,
        "Hope should increase with budget surplus: initial={initial_hope}, new={new_hope}",
    );
}

#[test]
fn test_bad_economy_decreases_hope() {
    let mut city = TestCity::new().with_budget(1_000.0);

    {
        let world = city.world_mut();
        let mut budget = world.resource_mut::<CityBudget>();
        budget.monthly_income = 100.0;
        budget.monthly_expenses = 2000.0;
        let mut ext = world.resource_mut::<crate::budget::ExtendedBudget>();
        ext.loans.push(crate::budget::Loan::new(100_000.0, 0.05, 120));
    }

    let initial_hope = city.resource::<HopeDiscontent>().hope;
    city.tick_slow_cycles(3);
    let new_hope = city.resource::<HopeDiscontent>().hope;

    assert!(
        new_hope < initial_hope,
        "Hope should decrease with budget deficit: initial={initial_hope}, new={new_hope}",
    );
}

#[test]
fn test_bad_economy_increases_discontent() {
    let mut city = TestCity::new().with_budget(1_000.0);

    {
        let world = city.world_mut();
        let mut budget = world.resource_mut::<CityBudget>();
        budget.monthly_income = 100.0;
        budget.monthly_expenses = 2000.0;
        let mut ext = world.resource_mut::<crate::budget::ExtendedBudget>();
        ext.loans.push(crate::budget::Loan::new(100_000.0, 0.05, 120));
    }

    let initial_discontent = city.resource::<HopeDiscontent>().discontent;
    city.tick_slow_cycles(3);
    let new_discontent = city.resource::<HopeDiscontent>().discontent;

    assert!(
        new_discontent > initial_discontent,
        "Discontent should increase with bad economy: initial={initial_discontent}, new={new_discontent}",
    );
}

// -----------------------------------------------------------------------
// Crisis detection (unit logic, no ticking needed)
// -----------------------------------------------------------------------

#[test]
fn test_crisis_triggers_at_extreme_hope() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut hd = world.resource_mut::<HopeDiscontent>();
        hd.hope = 0.05;
        hd.update_crisis_state();
    }

    let hd = city.resource::<HopeDiscontent>();
    assert_eq!(
        hd.crisis_state,
        CrisisState::Crisis,
        "Crisis should trigger when hope < 0.1"
    );
}

#[test]
fn test_crisis_triggers_at_extreme_discontent() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut hd = world.resource_mut::<HopeDiscontent>();
        hd.discontent = 0.95;
        hd.update_crisis_state();
    }

    let hd = city.resource::<HopeDiscontent>();
    assert_eq!(
        hd.crisis_state,
        CrisisState::Crisis,
        "Crisis should trigger when discontent > 0.9"
    );
}

#[test]
fn test_warning_state_at_moderate_thresholds() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut hd = world.resource_mut::<HopeDiscontent>();
        hd.hope = 0.2;
        hd.discontent = 0.5;
        hd.update_crisis_state();
    }

    let hd = city.resource::<HopeDiscontent>();
    assert_eq!(
        hd.crisis_state,
        CrisisState::Warning,
        "Warning should trigger when hope < 0.25"
    );
}

#[test]
fn test_normal_state_when_values_healthy() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut hd = world.resource_mut::<HopeDiscontent>();
        hd.hope = 0.6;
        hd.discontent = 0.3;
        hd.update_crisis_state();
    }

    let hd = city.resource::<HopeDiscontent>();
    assert_eq!(
        hd.crisis_state,
        CrisisState::Normal,
        "Normal state when hope > 0.25 and discontent < 0.75"
    );
}

// -----------------------------------------------------------------------
// Saveable roundtrip
// -----------------------------------------------------------------------

#[test]
fn test_hope_discontent_save_load_roundtrip() {
    let original = HopeDiscontent {
        hope: 0.33,
        discontent: 0.67,
        crisis_state: CrisisState::Warning,
    };

    let bytes = original.save_to_bytes().unwrap();
    let restored = HopeDiscontent::load_from_bytes(&bytes);

    assert!(
        (restored.hope - 0.33).abs() < f32::EPSILON,
        "Hope should survive save/load"
    );
    assert!(
        (restored.discontent - 0.67).abs() < f32::EPSILON,
        "Discontent should survive save/load"
    );
    assert_eq!(restored.crisis_state, CrisisState::Warning);
}
