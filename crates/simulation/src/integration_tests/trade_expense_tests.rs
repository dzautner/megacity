//! Integration tests verifying that trade deficit costs appear in
//! `monthly_expenses` and `ExpenseBreakdown` (issue #1984).

use crate::budget::ExtendedBudget;
use crate::economy::CityBudget;
use crate::production::CityGoods;
use crate::production_chain::DeepProductionChainState;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

/// Minutes per game day (matches the constant in economy.rs / income_projection.rs).
const TICKS_PER_DAY: f64 = 1440.0;
/// Days per collection month.
const DAYS_PER_MONTH: f64 = 30.0;

// ====================================================================
// Goods trade deficit appears in monthly_expenses
// ====================================================================

#[test]
fn test_trade_deficit_included_in_monthly_expenses() {
    let mut city = TestCity::new();

    // Set up a negative trade balance on CityGoods (per-tick deficit).
    let per_tick_deficit = -5.0; // importing $5/tick
    {
        let world = city.world_mut();
        world.resource_mut::<CityGoods>().trade_balance = per_tick_deficit;
    }

    // Advance the clock past the 30-day collection interval so collect_taxes fires.
    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 32;
        world.resource_mut::<CityBudget>().last_collection_day = 0;
    }

    // Run one tick so collect_taxes executes.
    city.tick(1);

    let budget = city.resource::<CityBudget>();
    let extended = city.resource::<ExtendedBudget>();

    let expected_trade_cost = 5.0 * TICKS_PER_DAY * DAYS_PER_MONTH; // 216,000

    // The trade deficit field should be populated.
    assert!(
        extended.expense_breakdown.trade_deficit > 0.0,
        "trade_deficit should be > 0 when city has negative trade balance; got {}",
        extended.expense_breakdown.trade_deficit
    );

    // The trade deficit should match the extrapolation formula.
    assert!(
        (extended.expense_breakdown.trade_deficit - expected_trade_cost).abs() < 1.0,
        "trade_deficit should be ~{expected_trade_cost}; got {}",
        extended.expense_breakdown.trade_deficit
    );

    // monthly_expenses should include the trade deficit.
    assert!(
        budget.monthly_expenses >= expected_trade_cost,
        "monthly_expenses ({}) should include trade deficit ({expected_trade_cost})",
        budget.monthly_expenses
    );
}

// ====================================================================
// Commodity trade deficit (deep chain) also included
// ====================================================================

#[test]
fn test_commodity_trade_deficit_included_in_monthly_expenses() {
    let mut city = TestCity::new();

    // Set a negative commodity trade balance (deep chain imports).
    let per_tick_deficit = -2.0;
    {
        let world = city.world_mut();
        world.resource_mut::<DeepProductionChainState>().commodity_trade_balance =
            per_tick_deficit;
    }

    // Trigger collection.
    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 32;
        world.resource_mut::<CityBudget>().last_collection_day = 0;
    }

    city.tick(1);

    let extended = city.resource::<ExtendedBudget>();
    let expected = 2.0 * TICKS_PER_DAY * DAYS_PER_MONTH; // 86,400

    assert!(
        (extended.expense_breakdown.trade_deficit - expected).abs() < 1.0,
        "trade_deficit should include commodity deficit ~{expected}; got {}",
        extended.expense_breakdown.trade_deficit
    );
}

// ====================================================================
// Both goods and commodity deficits combine
// ====================================================================

#[test]
fn test_combined_trade_deficits_sum_correctly() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        world.resource_mut::<CityGoods>().trade_balance = -3.0;
        world.resource_mut::<DeepProductionChainState>().commodity_trade_balance = -1.0;
    }

    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 32;
        world.resource_mut::<CityBudget>().last_collection_day = 0;
    }

    city.tick(1);

    let extended = city.resource::<ExtendedBudget>();
    let expected = (3.0 + 1.0) * TICKS_PER_DAY * DAYS_PER_MONTH; // 172,800

    assert!(
        (extended.expense_breakdown.trade_deficit - expected).abs() < 1.0,
        "combined trade_deficit should be ~{expected}; got {}",
        extended.expense_breakdown.trade_deficit
    );
}

// ====================================================================
// Positive trade balance contributes zero to expenses
// ====================================================================

#[test]
fn test_positive_trade_balance_zero_deficit() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        // Positive = exporting, should NOT add to expenses.
        world.resource_mut::<CityGoods>().trade_balance = 10.0;
        world.resource_mut::<DeepProductionChainState>().commodity_trade_balance = 5.0;
    }

    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 32;
        world.resource_mut::<CityBudget>().last_collection_day = 0;
    }

    city.tick(1);

    let extended = city.resource::<ExtendedBudget>();

    assert!(
        extended.expense_breakdown.trade_deficit.abs() < 0.001,
        "trade_deficit should be 0 with positive trade balance; got {}",
        extended.expense_breakdown.trade_deficit
    );
}

// ====================================================================
// Mixed: one positive, one negative
// ====================================================================

#[test]
fn test_mixed_trade_balances_only_negative_counted() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        world.resource_mut::<CityGoods>().trade_balance = 10.0; // exporting
        world.resource_mut::<DeepProductionChainState>().commodity_trade_balance = -4.0; // importing
    }

    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 32;
        world.resource_mut::<CityBudget>().last_collection_day = 0;
    }

    city.tick(1);

    let extended = city.resource::<ExtendedBudget>();
    let expected = 4.0 * TICKS_PER_DAY * DAYS_PER_MONTH; // only commodity deficit

    assert!(
        (extended.expense_breakdown.trade_deficit - expected).abs() < 1.0,
        "trade_deficit should only count negative balances; expected ~{expected}, got {}",
        extended.expense_breakdown.trade_deficit
    );
}
