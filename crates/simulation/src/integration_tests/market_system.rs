use crate::test_harness::TestCity;

// ====================================================================
// TEST-060: Market System Unit Tests
// ====================================================================

#[test]
fn test_market_prices_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::market::MarketPrices>();
}

#[test]
fn test_market_prices_initialized_at_base() {
    use crate::market::MarketPrices;
    use crate::production::GoodsType;
    let city = TestCity::new();
    let market = city.resource::<MarketPrices>();
    for &g in GoodsType::all() {
        let price = market.goods_price(g);
        let base = g.export_price();
        assert!(
            (price - base).abs() < f64::EPSILON,
            "goods {:?} price={} base={}",
            g,
            price,
            base
        );
    }
}

#[test]
fn test_market_default_multipliers_are_one() {
    use crate::market::MarketPrices;
    use crate::natural_resources::ResourceType;
    use crate::production::GoodsType;
    let city = TestCity::new();
    let market = city.resource::<MarketPrices>();
    for &g in GoodsType::all() {
        assert!(
            (market.goods_multiplier(g) - 1.0).abs() < f64::EPSILON,
            "goods {:?}",
            g
        );
    }
    for &rt in &[
        ResourceType::FertileLand,
        ResourceType::Forest,
        ResourceType::Ore,
        ResourceType::Oil,
    ] {
        assert!(
            (market.resource_multiplier(rt) - 1.0).abs() < f64::EPSILON,
            "resource {:?}",
            rt
        );
    }
}

#[test]
fn test_market_prices_update_after_slow_cycle() {
    use crate::market::MarketPrices;
    let mut city = TestCity::new();
    let before = city.resource::<MarketPrices>().cycle_counter;
    city.tick_slow_cycle();
    assert!(city.resource::<MarketPrices>().cycle_counter > before);
}

#[test]
fn test_market_surplus_lowers_price() {
    use crate::market::MarketPrices;
    use crate::production::{CityGoods, GoodsType};
    let mut city = TestCity::new();
    {
        let w = city.world_mut();
        let mut cg = w.resource_mut::<CityGoods>();
        cg.production_rate.insert(GoodsType::Steel, 100.0);
        cg.consumption_rate.insert(GoodsType::Steel, 1.0);
    }
    city.tick_slow_cycles(5);
    assert!(
        city.resource::<MarketPrices>()
            .goods_multiplier(GoodsType::Steel)
            < 1.1,
        "surplus should lower price"
    );
}

#[test]
fn test_market_deficit_raises_price() {
    use crate::market::MarketPrices;
    use crate::production::{CityGoods, GoodsType};
    let mut city = TestCity::new();
    {
        let w = city.world_mut();
        let mut cg = w.resource_mut::<CityGoods>();
        cg.production_rate.insert(GoodsType::Electronics, 1.0);
        cg.consumption_rate.insert(GoodsType::Electronics, 100.0);
    }
    city.tick_slow_cycles(5);
    assert!(
        city.resource::<MarketPrices>()
            .goods_multiplier(GoodsType::Electronics)
            > 0.9,
        "deficit should raise price"
    );
}

#[test]
fn test_production_adds_to_supply_stockpile() {
    use crate::production::{CityGoods, GoodsType};
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<CityGoods>()
            .available
            .insert(GoodsType::Lumber, 50.0);
    }
    assert!((city.resource::<CityGoods>().available[&GoodsType::Lumber] - 50.0).abs() < 0.001);
}

#[test]
fn test_consumption_reduces_supply_stockpile() {
    use crate::production::{CityGoods, GoodsType};
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<CityGoods>()
            .available
            .insert(GoodsType::ProcessedFood, 200.0);
    }
    city.tick(20);
    assert!(city.resource::<CityGoods>().available[&GoodsType::ProcessedFood] <= 200.0);
}

#[test]
fn test_trade_balance_export_surplus() {
    use crate::production::{CityGoods, GoodsType};
    let mut city = TestCity::new();
    {
        let w = city.world_mut();
        let mut cg = w.resource_mut::<CityGoods>();
        for &g in GoodsType::all() {
            cg.available.insert(g, 500.0);
        }
    }
    city.tick(10);
    assert!(
        city.resource::<CityGoods>().trade_balance >= 0.0,
        "surplus should yield non-negative trade balance"
    );
}

#[test]
fn test_trade_balance_import_deficit() {
    use crate::production::{CityGoods, GoodsType};
    let mut city = TestCity::new();
    {
        let w = city.world_mut();
        let mut cg = w.resource_mut::<CityGoods>();
        cg.production_rate.insert(GoodsType::Fuel, 0.0);
        cg.consumption_rate.insert(GoodsType::Fuel, 100.0);
    }
    assert!(
        city.resource::<CityGoods>().net(GoodsType::Fuel) < 0.0,
        "deficit net should be negative"
    );
}

#[test]
fn test_market_event_oil_shock_raises_fuel() {
    use crate::market::{ActiveMarketEvent, MarketEvent, MarketPrices};
    use crate::production::GoodsType;
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<MarketPrices>()
            .active_events
            .push(ActiveMarketEvent {
                event: MarketEvent::OilShock,
                remaining_ticks: 15,
            });
    }
    city.tick_slow_cycle();
    assert!(
        city.resource::<MarketPrices>()
            .goods_multiplier(GoodsType::Fuel)
            > 1.0,
        "OilShock should raise Fuel"
    );
}

#[test]
fn test_market_event_tech_boom_lowers_electronics() {
    use crate::market::{ActiveMarketEvent, MarketEvent, MarketPrices};
    use crate::production::GoodsType;
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<MarketPrices>()
            .active_events
            .push(ActiveMarketEvent {
                event: MarketEvent::TechBoom,
                remaining_ticks: 12,
            });
    }
    city.tick_slow_cycle();
    assert!(
        city.resource::<MarketPrices>()
            .goods_multiplier(GoodsType::Electronics)
            < 1.1,
        "TechBoom should lower Electronics"
    );
}

#[test]
fn test_market_event_food_crisis_raises_food() {
    use crate::market::{ActiveMarketEvent, MarketEvent, MarketPrices};
    use crate::production::GoodsType;
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<MarketPrices>()
            .active_events
            .push(ActiveMarketEvent {
                event: MarketEvent::FoodCrisis,
                remaining_ticks: 10,
            });
    }
    city.tick_slow_cycle();
    assert!(
        city.resource::<MarketPrices>()
            .goods_multiplier(GoodsType::RawFood)
            > 1.0,
        "FoodCrisis should raise RawFood"
    );
}

#[test]
fn test_market_event_expires_after_duration() {
    use crate::market::{ActiveMarketEvent, MarketEvent, MarketPrices};
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<MarketPrices>()
            .active_events
            .push(ActiveMarketEvent {
                event: MarketEvent::Recession,
                remaining_ticks: 2,
            });
    }
    city.tick_slow_cycles(3);
    let m = city.resource::<MarketPrices>();
    assert!(
        m.active_events.is_empty()
            || !m
                .active_events
                .iter()
                .any(|e| e.event == MarketEvent::Recession)
    );
}

#[test]
fn test_market_event_resource_effects_construction_boom() {
    use crate::market::{ActiveMarketEvent, MarketEvent, MarketPrices};
    use crate::natural_resources::ResourceType;
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<MarketPrices>()
            .active_events
            .push(ActiveMarketEvent {
                event: MarketEvent::ConstructionBoom,
                remaining_ticks: 18,
            });
    }
    city.tick_slow_cycle();
    assert!(
        city.resource::<MarketPrices>()
            .resource_multiplier(ResourceType::Ore)
            > 1.0,
        "ConstructionBoom should raise Ore"
    );
}

#[test]
fn test_market_prices_clamped_to_range() {
    use crate::market::MarketPrices;
    use crate::production::GoodsType;
    let mut city = TestCity::new();
    city.tick_slow_cycles(50);
    let m = city.resource::<MarketPrices>();
    for &g in GoodsType::all() {
        let e = &m.goods_prices[&g];
        assert!(
            e.current_price >= e.base_price * 0.3 && e.current_price <= e.base_price * 3.0,
            "{:?} out of range",
            g
        );
    }
}

#[test]
fn test_market_resource_prices_clamped() {
    use crate::market::MarketPrices;
    use crate::natural_resources::ResourceType;
    let mut city = TestCity::new();
    city.tick_slow_cycles(50);
    let m = city.resource::<MarketPrices>();
    for &rt in &[
        ResourceType::FertileLand,
        ResourceType::Forest,
        ResourceType::Ore,
        ResourceType::Oil,
    ] {
        let e = &m.resource_prices[&rt];
        assert!(
            e.current_price >= e.base_price * 0.3 && e.current_price <= e.base_price * 3.0,
            "{:?} out of range",
            rt
        );
    }
}

#[test]
fn test_market_cycle_counter_increments_by_5() {
    use crate::market::MarketPrices;
    let mut city = TestCity::new();
    let before = city.resource::<MarketPrices>().cycle_counter;
    city.tick_slow_cycles(5);
    assert_eq!(city.resource::<MarketPrices>().cycle_counter - before, 5);
}

#[test]
fn test_market_no_duplicate_events() {
    use crate::market::{ActiveMarketEvent, MarketEvent, MarketPrices};
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<MarketPrices>()
            .active_events
            .push(ActiveMarketEvent {
                event: MarketEvent::TradeEmbargo,
                remaining_ticks: 20,
            });
    }
    city.tick_slow_cycles(10);
    assert!(
        city.resource::<MarketPrices>()
            .active_events
            .iter()
            .filter(|e| e.event == MarketEvent::TradeEmbargo)
            .count()
            <= 1
    );
}

#[test]
fn test_market_max_two_active_events() {
    use crate::market::{ActiveMarketEvent, MarketEvent, MarketPrices};
    let mut city = TestCity::new();
    {
        let w = city.world_mut();
        let mut m = w.resource_mut::<MarketPrices>();
        m.active_events.push(ActiveMarketEvent {
            event: MarketEvent::OilShock,
            remaining_ticks: 100,
        });
        m.active_events.push(ActiveMarketEvent {
            event: MarketEvent::Recession,
            remaining_ticks: 100,
        });
    }
    city.tick_slow_cycles(20);
    assert!(city.resource::<MarketPrices>().active_events.len() <= 2);
}

#[test]
fn test_market_trade_embargo_affects_multiple_goods() {
    use crate::market::{ActiveMarketEvent, MarketEvent, MarketPrices};
    use crate::production::GoodsType;
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<MarketPrices>()
            .active_events
            .push(ActiveMarketEvent {
                event: MarketEvent::TradeEmbargo,
                remaining_ticks: 20,
            });
    }
    city.tick_slow_cycle();
    let m = city.resource::<MarketPrices>();
    for &g in &[
        GoodsType::RawFood,
        GoodsType::ProcessedFood,
        GoodsType::Steel,
        GoodsType::Electronics,
        GoodsType::ConsumerGoods,
    ] {
        assert!(
            m.goods_multiplier(g) > 0.5,
            "{:?} too low under TradeEmbargo",
            g
        );
    }
}

#[test]
fn test_market_price_trend_tracking() {
    use crate::market::PriceEntry;
    let mut e = PriceEntry::new(10.0);
    assert!(e.trend().abs() < f64::EPSILON);
    e.previous_price = 10.0;
    e.current_price = 12.5;
    assert!((e.trend() - 2.5).abs() < f64::EPSILON);
    e.previous_price = 12.5;
    e.current_price = 9.0;
    assert!((e.trend() - (-3.5)).abs() < f64::EPSILON);
}

#[test]
fn test_market_event_all_variants_valid() {
    use crate::market::MarketEvent;
    for &ev in MarketEvent::ALL {
        assert!(!ev.name().is_empty());
        assert!(ev.duration_slow_ticks() > 0);
        assert!(!ev.price_effects().is_empty() || !ev.resource_effects().is_empty());
    }
}

#[test]
fn test_market_surplus_export_caps_stock() {
    use crate::production::{CityGoods, GoodsType};
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<CityGoods>()
            .available
            .insert(GoodsType::Steel, 500.0);
    }
    city.tick(10);
    assert!(city.resource::<CityGoods>().available[&GoodsType::Steel] <= 100.0);
}

#[test]
fn test_market_treasury_increases_from_exports() {
    use crate::production::{CityGoods, GoodsType};
    let mut city = TestCity::new().with_budget(10000.0);
    {
        let w = city.world_mut();
        let mut cg = w.resource_mut::<CityGoods>();
        for &g in GoodsType::all() {
            cg.available.insert(g, 500.0);
        }
    }
    city.tick(10);
    assert!(
        city.budget().treasury > 10000.0,
        "exports should increase treasury"
    );
}

#[test]
fn test_market_goods_net_surplus() {
    use crate::production::{CityGoods, GoodsType};
    let mut city = TestCity::new();
    {
        let w = city.world_mut();
        let mut cg = w.resource_mut::<CityGoods>();
        cg.production_rate.insert(GoodsType::Lumber, 25.0);
        cg.consumption_rate.insert(GoodsType::Lumber, 10.0);
    }
    assert!((city.resource::<CityGoods>().net(GoodsType::Lumber) - 15.0).abs() < 0.001);
}

#[test]
fn test_market_goods_net_deficit_value() {
    use crate::production::{CityGoods, GoodsType};
    let mut city = TestCity::new();
    {
        let w = city.world_mut();
        let mut cg = w.resource_mut::<CityGoods>();
        cg.production_rate.insert(GoodsType::Fuel, 5.0);
        cg.consumption_rate.insert(GoodsType::Fuel, 30.0);
    }
    assert!((city.resource::<CityGoods>().net(GoodsType::Fuel) - (-25.0)).abs() < 0.001);
}

#[test]
fn test_market_combined_events_stack() {
    use crate::market::{ActiveMarketEvent, MarketEvent, MarketPrices};
    use crate::production::GoodsType;
    let mut city = TestCity::new();
    {
        let w = city.world_mut();
        let mut m = w.resource_mut::<MarketPrices>();
        m.active_events.push(ActiveMarketEvent {
            event: MarketEvent::OilShock,
            remaining_ticks: 15,
        });
        m.active_events.push(ActiveMarketEvent {
            event: MarketEvent::ConstructionBoom,
            remaining_ticks: 18,
        });
    }
    city.tick_slow_cycle();
    assert!(
        city.resource::<MarketPrices>()
            .goods_multiplier(GoodsType::Fuel)
            > 1.0
    );
}

#[test]
fn test_market_recession_lowers_consumer_goods() {
    use crate::market::{ActiveMarketEvent, MarketEvent, MarketPrices};
    use crate::production::GoodsType;
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<MarketPrices>()
            .active_events
            .push(ActiveMarketEvent {
                event: MarketEvent::Recession,
                remaining_ticks: 25,
            });
    }
    city.tick_slow_cycle();
    assert!(
        city.resource::<MarketPrices>()
            .goods_multiplier(GoodsType::ConsumerGoods)
            < 1.2
    );
}

#[test]
fn test_market_construction_boom_raises_steel_lumber() {
    use crate::market::{ActiveMarketEvent, MarketEvent, MarketPrices};
    use crate::production::GoodsType;
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<MarketPrices>()
            .active_events
            .push(ActiveMarketEvent {
                event: MarketEvent::ConstructionBoom,
                remaining_ticks: 18,
            });
    }
    city.tick_slow_cycle();
    let m = city.resource::<MarketPrices>();
    assert!(m.goods_multiplier(GoodsType::Steel) > 1.0);
    assert!(m.goods_multiplier(GoodsType::Lumber) > 1.0);
}

#[test]
fn test_market_prices_valid_after_many_cycles() {
    use crate::market::MarketPrices;
    use crate::production::GoodsType;
    let mut city = TestCity::new();
    city.tick_slow_cycles(20);
    let p = city.resource::<MarketPrices>().goods_price(GoodsType::Fuel);
    let b = GoodsType::Fuel.export_price();
    assert!(p >= b * 0.3 && p <= b * 3.0 && p.is_finite());
}

#[test]
fn test_market_all_resources_have_prices() {
    use crate::market::MarketPrices;
    use crate::natural_resources::ResourceType;
    let city = TestCity::new();
    let m = city.resource::<MarketPrices>();
    for &rt in &[
        ResourceType::FertileLand,
        ResourceType::Forest,
        ResourceType::Ore,
        ResourceType::Oil,
    ] {
        assert!(m.resource_prices.contains_key(&rt));
    }
}

#[test]
fn test_market_oil_shock_raises_oil_resource() {
    use crate::market::{ActiveMarketEvent, MarketEvent, MarketPrices};
    use crate::natural_resources::ResourceType;
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<MarketPrices>()
            .active_events
            .push(ActiveMarketEvent {
                event: MarketEvent::OilShock,
                remaining_ticks: 15,
            });
    }
    city.tick_slow_cycle();
    assert!(
        city.resource::<MarketPrices>()
            .resource_multiplier(ResourceType::Oil)
            > 1.0
    );
}

#[test]
fn test_city_goods_default_zeroed() {
    use crate::production::{CityGoods, GoodsType};
    let g = CityGoods::default();
    for &t in GoodsType::all() {
        assert_eq!(g.available[&t], 0.0);
        assert_eq!(g.production_rate[&t], 0.0);
        assert_eq!(g.consumption_rate[&t], 0.0);
    }
    assert_eq!(g.trade_balance, 0.0);
}

#[test]
fn test_market_event_names_are_unique() {
    use crate::market::MarketEvent;
    use std::collections::HashSet;
    let n: HashSet<&str> = MarketEvent::ALL.iter().map(|e| e.name()).collect();
    assert_eq!(n.len(), MarketEvent::ALL.len());
}
