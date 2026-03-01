//! Integration tests for #1969: trade deficit should not drain treasury
//! when population is zero.

use crate::city_observation::CityWarning;
use crate::economy::CityBudget;
use crate::observation_builder::CurrentObservation;
use crate::production::types::CityGoods;
use crate::test_harness::TestCity;

/// With 0 population, production and trade should be skipped entirely,
/// so the treasury should remain unchanged from trade activity.
#[test]
fn test_trade_deficit_no_drain_with_zero_population() {
    let initial_treasury = 50_000.0;
    let mut city = TestCity::new().with_budget(initial_treasury);

    // Run 500 ticks (50 production cycles). With the bug, imports would
    // drain the treasury even though nobody lives in the city.
    city.tick(500);

    let treasury = city.resource::<CityBudget>().treasury;

    // Treasury should not have decreased significantly from trade costs.
    // Allow a small epsilon for floating-point rounding in other systems.
    assert!(
        treasury > initial_treasury - 100.0,
        "Treasury should not drain from trade with 0 population; \
         started at {initial_treasury}, now {treasury}"
    );
}

/// With 0 population, trade_balance should remain at zero (no imports),
/// so the TradeDeficit warning must not appear.
#[test]
fn test_no_trade_deficit_warning_with_zero_population() {
    let mut city = TestCity::new();

    // Run enough ticks for the observation builder to run.
    city.tick(100);

    let trade_balance = city.resource::<CityGoods>().trade_balance;
    assert!(
        trade_balance >= -0.5,
        "Trade balance should not be significantly negative with 0 pop; got {trade_balance}"
    );

    let obs = &city.resource::<CurrentObservation>().observation;
    assert!(
        !obs.warnings.contains(&CityWarning::TradeDeficit),
        "TradeDeficit warning should not appear with 0 population; warnings={:?}",
        obs.warnings
    );
}
