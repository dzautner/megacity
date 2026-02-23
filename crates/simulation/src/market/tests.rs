#[cfg(test)]
mod tests {
    use crate::market::events::{ActiveMarketEvent, MarketEvent};
    use crate::market::pricing::{pseudo_random, sine_approx};
    use crate::market::types::{MarketPrices, PriceEntry};
    use crate::natural_resources::ResourceType;
    use crate::production::GoodsType;

    #[test]
    fn test_market_prices_default() {
        let market = MarketPrices::default();
        for &g in GoodsType::all() {
            let entry = &market.goods_prices[&g];
            assert_eq!(entry.base_price, g.export_price());
            assert_eq!(entry.current_price, g.export_price());
            assert!((entry.multiplier() - 1.0).abs() < f64::EPSILON);
        }
        for &rt in &[
            ResourceType::FertileLand,
            ResourceType::Forest,
            ResourceType::Ore,
            ResourceType::Oil,
        ] {
            assert!(market.resource_prices.contains_key(&rt));
            let entry = &market.resource_prices[&rt];
            assert!((entry.multiplier() - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_price_entry_trend() {
        let mut entry = PriceEntry::new(10.0);
        assert!((entry.trend()).abs() < f64::EPSILON);
        entry.previous_price = 10.0;
        entry.current_price = 12.0;
        assert!((entry.trend() - 2.0).abs() < f64::EPSILON);
        entry.previous_price = 12.0;
        entry.current_price = 9.0;
        assert!((entry.trend() - (-3.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_market_event_effects_non_empty() {
        for &event in MarketEvent::ALL {
            assert!(!event.name().is_empty());
            assert!(event.duration_slow_ticks() > 0);
            // At least one of goods or resource effects should be present
            assert!(
                !event.price_effects().is_empty() || !event.resource_effects().is_empty(),
                "Event {:?} has no effects",
                event,
            );
        }
    }

    #[test]
    fn test_pseudo_random_range() {
        for seed in 0..1000u64 {
            let val = pseudo_random(seed);
            assert!(val >= 0.0 && val < 1.0, "pseudo_random({}) = {}", seed, val);
        }
    }

    #[test]
    fn test_pseudo_random_deterministic() {
        let a = pseudo_random(42);
        let b = pseudo_random(42);
        assert_eq!(a, b);
    }

    #[test]
    fn test_sine_approx_range() {
        for tick in 0..200 {
            let val = sine_approx(tick, 50);
            assert!(
                val >= -1.01 && val <= 1.01,
                "sine_approx({}, 50) = {}",
                tick,
                val
            );
        }
    }

    #[test]
    fn test_sine_approx_period() {
        let period = 100u32;
        // Value at tick 0 and tick `period` should be very close (both sin(0) ~ 0)
        let v0 = sine_approx(0, period);
        let v_period = sine_approx(period, period);
        assert!(
            (v0 - v_period).abs() < 0.01,
            "Period not respected: v0={}, v_period={}",
            v0,
            v_period
        );
    }

    #[test]
    fn test_goods_multiplier_default() {
        let market = MarketPrices::default();
        for &g in GoodsType::all() {
            assert!((market.goods_multiplier(g) - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_goods_price_returns_current() {
        let mut market = MarketPrices::default();
        // Manually adjust a price
        if let Some(entry) = market.goods_prices.get_mut(&GoodsType::Fuel) {
            entry.current_price = 15.0;
        }
        assert!((market.goods_price(GoodsType::Fuel) - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_active_event_tick_down() {
        let mut events = vec![
            ActiveMarketEvent {
                event: MarketEvent::OilShock,
                remaining_ticks: 3,
            },
            ActiveMarketEvent {
                event: MarketEvent::Recession,
                remaining_ticks: 1,
            },
        ];

        // Tick down
        events
            .iter_mut()
            .for_each(|ae| ae.remaining_ticks = ae.remaining_ticks.saturating_sub(1));
        events.retain(|ae| ae.remaining_ticks > 0);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event, MarketEvent::OilShock);
        assert_eq!(events[0].remaining_ticks, 2);
    }

    #[test]
    fn test_price_clamping() {
        let mut entry = PriceEntry::new(10.0);
        // Simulate extreme price
        entry.current_price = 50.0;
        let clamped = entry
            .current_price
            .clamp(entry.base_price * 0.3, entry.base_price * 3.0);
        assert!((clamped - 30.0).abs() < f64::EPSILON);

        entry.current_price = 0.5;
        let clamped = entry
            .current_price
            .clamp(entry.base_price * 0.3, entry.base_price * 3.0);
        assert!((clamped - 3.0).abs() < f64::EPSILON);
    }
}
