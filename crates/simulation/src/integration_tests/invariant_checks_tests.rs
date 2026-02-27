//! Integration tests for runtime invariant guards (budget, population, happiness).

use crate::economy::CityBudget;
use crate::invariant_checks::CoreInvariantViolations;
use crate::stats::CityStats;
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Budget / treasury invariant tests
// ---------------------------------------------------------------------------

#[test]
fn test_nan_treasury_is_caught_and_fixed() {
    let mut city = TestCity::new();

    // Inject NaN into treasury
    {
        let world = city.world_mut();
        let mut budget = world.resource_mut::<CityBudget>();
        budget.treasury = f64::NAN;
    }

    // Run a slow cycle so the invariant check fires
    city.tick_slow_cycle();

    let budget = city.budget();
    assert!(
        budget.treasury.is_finite(),
        "Treasury should be finite after invariant check, got {}",
        budget.treasury
    );
    assert!(
        (budget.treasury - 0.0).abs() < f64::EPSILON,
        "NaN treasury should be reset to 0.0, got {}",
        budget.treasury
    );

    let violations = city.resource::<CoreInvariantViolations>();
    assert!(
        violations.budget_treasury > 0,
        "Should have detected at least one treasury violation"
    );
}

#[test]
fn test_infinite_treasury_is_caught_and_fixed() {
    let mut city = TestCity::new();

    // Inject positive infinity into treasury
    {
        let world = city.world_mut();
        let mut budget = world.resource_mut::<CityBudget>();
        budget.treasury = f64::INFINITY;
    }

    city.tick_slow_cycle();

    let budget = city.budget();
    assert!(
        budget.treasury.is_finite(),
        "Infinite treasury should be corrected"
    );

    let violations = city.resource::<CoreInvariantViolations>();
    assert!(
        violations.budget_treasury > 0,
        "Should have detected treasury infinity violation"
    );
}

#[test]
fn test_extreme_negative_treasury_is_caught_and_fixed() {
    let mut city = TestCity::new();

    // Inject extreme negative value (below the -1 billion floor)
    {
        let world = city.world_mut();
        let mut budget = world.resource_mut::<CityBudget>();
        budget.treasury = -2_000_000_000.0;
    }

    city.tick_slow_cycle();

    let budget = city.budget();
    assert!(
        budget.treasury >= -1_000_000_000.0,
        "Extreme negative treasury should be corrected, got {}",
        budget.treasury
    );

    let violations = city.resource::<CoreInvariantViolations>();
    assert!(
        violations.budget_treasury > 0,
        "Should have detected extreme negative treasury violation"
    );
}

#[test]
fn test_nan_monthly_income_is_caught_and_fixed() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut budget = world.resource_mut::<CityBudget>();
        budget.monthly_income = f64::NAN;
    }

    city.tick_slow_cycle();

    let budget = city.budget();
    assert!(
        budget.monthly_income.is_finite() && budget.monthly_income >= 0.0,
        "NaN monthly_income should be reset to 0.0, got {}",
        budget.monthly_income
    );

    let violations = city.resource::<CoreInvariantViolations>();
    assert!(
        violations.budget_income > 0,
        "Should have detected income NaN violation"
    );
}

#[test]
fn test_nan_monthly_expenses_is_caught_and_fixed() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut budget = world.resource_mut::<CityBudget>();
        budget.monthly_expenses = f64::NEG_INFINITY;
    }

    city.tick_slow_cycle();

    let budget = city.budget();
    assert!(
        budget.monthly_expenses.is_finite() && budget.monthly_expenses >= 0.0,
        "Negative infinity expenses should be reset, got {}",
        budget.monthly_expenses
    );

    let violations = city.resource::<CoreInvariantViolations>();
    assert!(
        violations.budget_expenses > 0,
        "Should have detected expenses violation"
    );
}

// ---------------------------------------------------------------------------
// Happiness invariant tests
// ---------------------------------------------------------------------------

#[test]
fn test_nan_happiness_is_caught_and_fixed() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.average_happiness = f32::NAN;
    }

    city.tick_slow_cycle();

    let stats = city.resource::<CityStats>();
    assert!(
        stats.average_happiness.is_finite(),
        "NaN happiness should be corrected"
    );

    let violations = city.resource::<CoreInvariantViolations>();
    assert!(
        violations.happiness > 0,
        "Should have detected happiness NaN violation"
    );
}

#[test]
fn test_out_of_range_happiness_is_clamped() {
    let mut city = TestCity::new();

    // Inject happiness above 100
    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.average_happiness = 150.0;
    }

    city.tick_slow_cycle();

    let stats = city.resource::<CityStats>();
    assert!(
        stats.average_happiness <= 100.0,
        "Happiness above 100 should be clamped, got {}",
        stats.average_happiness
    );

    let violations = city.resource::<CoreInvariantViolations>();
    assert!(
        violations.happiness > 0,
        "Should have detected out-of-range happiness violation"
    );
}

#[test]
fn test_negative_happiness_is_clamped() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut stats = world.resource_mut::<CityStats>();
        stats.average_happiness = -10.0;
    }

    city.tick_slow_cycle();

    let stats = city.resource::<CityStats>();
    assert!(
        stats.average_happiness >= 0.0,
        "Negative happiness should be clamped to 0, got {}",
        stats.average_happiness
    );
}

// ---------------------------------------------------------------------------
// Healthy state — no false positives
// ---------------------------------------------------------------------------

#[test]
fn test_no_violations_on_healthy_state() {
    let mut city = TestCity::new().with_budget(50_000.0);

    city.tick_slow_cycle();

    let violations = city.resource::<CoreInvariantViolations>();
    assert_eq!(
        violations.budget_treasury, 0,
        "No treasury violations expected on healthy state"
    );
    assert_eq!(
        violations.budget_income, 0,
        "No income violations expected on healthy state"
    );
    assert_eq!(
        violations.budget_expenses, 0,
        "No expense violations expected on healthy state"
    );
    assert_eq!(
        violations.happiness, 0,
        "No happiness violations expected on healthy state"
    );
    assert_eq!(
        violations.population, 0,
        "No population violations expected on healthy state"
    );
}
