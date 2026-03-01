//! Integration tests for trade deficit cap and TradeDeficit warning (#1962).

use crate::city_observation::CityWarning;
use crate::economy::CityBudget;
use crate::observation_builder::CurrentObservation;
use crate::production::types::{CityGoods, GoodsType};
use crate::test_harness::TestCity;

/// With no industrial production, the city imports goods to meet population
/// demand. After the fix, the treasury should not collapse catastrophically.
#[test]
fn test_trade_deficit_does_not_collapse_treasury() {
    let mut city = TestCity::new().with_budget(50_000.0);

    // Run 200 ticks (~20 production cycles at PRODUCTION_INTERVAL=10).
    // Without the cap fix, this would drain hundreds of thousands.
    city.tick(200);

    let treasury = city.resource::<CityBudget>().treasury;

    // Treasury should not have fallen below -10,000 from trade costs alone.
    // With no population and no production, there should be zero trade costs.
    // Even with population, the capped deficit (10.0 * 1.2x * 0.01) per goods
    // per tick is manageable.
    assert!(
        treasury > -10_000.0,
        "Treasury should not collapse from trade deficit; got {treasury}"
    );
}

/// With population but no production, trade deficit should be moderate.
/// Manually inject negative net rates to simulate a consuming city.
#[test]
fn test_trade_deficit_capped_with_consumption() {
    let mut city = TestCity::new().with_budget(50_000.0);

    // Manually set consumption rates high to simulate a large population
    // consuming goods with no production.
    {
        let world = city.world_mut();
        let mut goods = world.resource_mut::<CityGoods>();
        for &g in GoodsType::all() {
            goods.consumption_rate.insert(g, 100.0);
            goods.production_rate.insert(g, 0.0);
        }
    }

    // Run 100 production cycles (1000 ticks)
    city.tick(1000);

    let treasury = city.resource::<CityBudget>().treasury;

    // With the cap (10.0 per goods type * 7 types * avg_import ~7.2 * 0.01
    // = ~5.04 per tick, applied every 10 ticks = ~0.504 per tick average),
    // over 1000 ticks that's ~504. Starting from 50K, treasury should stay
    // well above -50K.
    assert!(
        treasury > -50_000.0,
        "Trade deficit should be capped; treasury={treasury}, should be > -50000"
    );
}

/// When trade_balance is significantly negative, the TradeDeficit warning
/// should appear in the city observation.
#[test]
fn test_trade_deficit_warning_appears() {
    let mut city = TestCity::new();

    // Inject a negative trade balance directly to trigger the warning.
    {
        let world = city.world_mut();
        let mut goods = world.resource_mut::<CityGoods>();
        goods.trade_balance = -5.0; // well below the -0.5 threshold
    }

    // Tick so the observation builder runs and picks up the trade balance.
    city.tick(1);

    let obs = &city.resource::<CurrentObservation>().observation;
    assert!(
        obs.warnings.contains(&CityWarning::TradeDeficit),
        "TradeDeficit warning should appear when trade_balance is negative; warnings={:?}",
        obs.warnings,
    );
}

/// When trade_balance is zero or positive, no TradeDeficit warning.
#[test]
fn test_no_trade_deficit_warning_when_balanced() {
    let mut city = TestCity::new();

    // Ensure trade balance is non-negative.
    {
        let world = city.world_mut();
        let mut goods = world.resource_mut::<CityGoods>();
        goods.trade_balance = 0.0;
    }

    city.tick(1);

    let obs = &city.resource::<CurrentObservation>().observation;
    assert!(
        !obs.warnings.contains(&CityWarning::TradeDeficit),
        "TradeDeficit warning should NOT appear when trade_balance >= 0; warnings={:?}",
        obs.warnings,
    );
}

/// The import price multiplier should be 1.2x (not the old 1.8x).
#[test]
fn test_import_price_is_1_2x_export() {
    for &g in GoodsType::all() {
        let export = g.export_price();
        let import = g.import_price();
        let ratio = import / export;
        assert!(
            (ratio - 1.2).abs() < 0.001,
            "{:?} import/export ratio should be 1.2, got {ratio}",
            g,
        );
    }
}
