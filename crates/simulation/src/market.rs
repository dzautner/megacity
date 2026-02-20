use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::economy::CityBudget;
use crate::natural_resources::ResourceType;
use crate::production::{CityGoods, GoodsType};
use crate::SlowTickTimer;
use crate::TickCounter;

// =============================================================================
// Market Events
// =============================================================================

/// Global market events that temporarily shift prices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketEvent {
    /// Oil prices spike due to geopolitical tensions.
    OilShock,
    /// A trade embargo restricts imports, raising all prices.
    TradeEmbargo,
    /// Technological breakthrough lowers electronics prices.
    TechBoom,
    /// Agricultural blight reduces food supply.
    FoodCrisis,
    /// Global recession reduces demand for consumer goods.
    Recession,
    /// Construction boom drives up raw material prices.
    ConstructionBoom,
}

impl MarketEvent {
    pub fn name(self) -> &'static str {
        match self {
            Self::OilShock => "Oil Shock",
            Self::TradeEmbargo => "Trade Embargo",
            Self::TechBoom => "Tech Boom",
            Self::FoodCrisis => "Food Crisis",
            Self::Recession => "Recession",
            Self::ConstructionBoom => "Construction Boom",
        }
    }

    /// Duration in slow ticks (each slow tick = 100 game ticks).
    pub fn duration_slow_ticks(self) -> u32 {
        match self {
            Self::OilShock => 15,
            Self::TradeEmbargo => 20,
            Self::TechBoom => 12,
            Self::FoodCrisis => 10,
            Self::Recession => 25,
            Self::ConstructionBoom => 18,
        }
    }

    /// Price multiplier adjustments for each goods type during this event.
    /// Returns (GoodsType, multiplier_delta). A positive delta means higher prices.
    pub fn price_effects(self) -> &'static [(GoodsType, f32)] {
        match self {
            Self::OilShock => &[
                (GoodsType::Fuel, 0.6),
                (GoodsType::Electronics, 0.2),
                (GoodsType::ConsumerGoods, 0.1),
            ],
            Self::TradeEmbargo => &[
                (GoodsType::RawFood, 0.3),
                (GoodsType::ProcessedFood, 0.3),
                (GoodsType::Steel, 0.3),
                (GoodsType::Electronics, 0.4),
                (GoodsType::ConsumerGoods, 0.2),
            ],
            Self::TechBoom => &[(GoodsType::Electronics, -0.3), (GoodsType::Steel, 0.1)],
            Self::FoodCrisis => &[(GoodsType::RawFood, 0.5), (GoodsType::ProcessedFood, 0.4)],
            Self::Recession => &[
                (GoodsType::ConsumerGoods, -0.2),
                (GoodsType::Electronics, -0.2),
                (GoodsType::Lumber, -0.15),
            ],
            Self::ConstructionBoom => &[
                (GoodsType::Steel, 0.4),
                (GoodsType::Lumber, 0.35),
                (GoodsType::ConsumerGoods, 0.1),
            ],
        }
    }

    /// Resource type price effects (multiplier delta).
    pub fn resource_effects(self) -> &'static [(ResourceType, f32)] {
        match self {
            Self::OilShock => &[(ResourceType::Oil, 0.5)],
            Self::TradeEmbargo => &[(ResourceType::Ore, 0.2), (ResourceType::Oil, 0.2)],
            Self::TechBoom => &[(ResourceType::Ore, 0.15)],
            Self::FoodCrisis => &[(ResourceType::FertileLand, 0.3)],
            Self::Recession => &[(ResourceType::Ore, -0.15), (ResourceType::Forest, -0.1)],
            Self::ConstructionBoom => &[(ResourceType::Ore, 0.3), (ResourceType::Forest, 0.25)],
        }
    }

    /// All possible market events.
    pub const ALL: &'static [MarketEvent] = &[
        Self::OilShock,
        Self::TradeEmbargo,
        Self::TechBoom,
        Self::FoodCrisis,
        Self::Recession,
        Self::ConstructionBoom,
    ];
}

// =============================================================================
// Price Entry
// =============================================================================

/// Tracks the current price, base price, and trend for a single commodity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceEntry {
    /// Base price (the GoodsType::export_price or resource base).
    pub base_price: f64,
    /// Current market price after supply/demand and events.
    pub current_price: f64,
    /// Previous price for computing trend direction.
    pub previous_price: f64,
}

impl PriceEntry {
    pub fn new(base: f64) -> Self {
        Self {
            base_price: base,
            current_price: base,
            previous_price: base,
        }
    }

    /// Returns the price multiplier relative to base (1.0 = at base price).
    pub fn multiplier(&self) -> f64 {
        if self.base_price > 0.0 {
            self.current_price / self.base_price
        } else {
            1.0
        }
    }

    /// Trend: positive = rising, negative = falling.
    pub fn trend(&self) -> f64 {
        self.current_price - self.previous_price
    }
}

// =============================================================================
// Active Event tracking
// =============================================================================

/// An active market event with remaining duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveMarketEvent {
    pub event: MarketEvent,
    pub remaining_ticks: u32,
}

// =============================================================================
// MarketPrices Resource
// =============================================================================

/// City-wide resource tracking global market prices for goods and resources.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrices {
    /// Current prices for each goods type.
    pub goods_prices: HashMap<GoodsType, PriceEntry>,
    /// Current prices for each resource type.
    pub resource_prices: HashMap<ResourceType, PriceEntry>,
    /// Currently active market events.
    pub active_events: Vec<ActiveMarketEvent>,
    /// Internal cycle counter for boom/bust sine wave.
    pub cycle_counter: u32,
}

impl Default for MarketPrices {
    fn default() -> Self {
        let mut goods_prices = HashMap::new();
        for &g in GoodsType::all() {
            goods_prices.insert(g, PriceEntry::new(g.export_price()));
        }

        let mut resource_prices = HashMap::new();
        let resource_base = |rt: ResourceType| -> f64 {
            match rt {
                ResourceType::FertileLand => 4.0,
                ResourceType::Forest => 5.0,
                ResourceType::Ore => 8.0,
                ResourceType::Oil => 10.0,
            }
        };
        for &rt in &[
            ResourceType::FertileLand,
            ResourceType::Forest,
            ResourceType::Ore,
            ResourceType::Oil,
        ] {
            resource_prices.insert(rt, PriceEntry::new(resource_base(rt)));
        }

        Self {
            goods_prices,
            resource_prices,
            active_events: Vec::new(),
            cycle_counter: 0,
        }
    }
}

impl MarketPrices {
    /// Get the current price multiplier for a goods type (1.0 = base price).
    pub fn goods_multiplier(&self, goods: GoodsType) -> f64 {
        self.goods_prices
            .get(&goods)
            .map(|e| e.multiplier())
            .unwrap_or(1.0)
    }

    /// Get the current price for a goods type.
    pub fn goods_price(&self, goods: GoodsType) -> f64 {
        self.goods_prices
            .get(&goods)
            .map(|e| e.current_price)
            .unwrap_or(goods.export_price())
    }

    /// Get the current price multiplier for a resource type.
    pub fn resource_multiplier(&self, resource: ResourceType) -> f64 {
        self.resource_prices
            .get(&resource)
            .map(|e| e.multiplier())
            .unwrap_or(1.0)
    }
}

// =============================================================================
// Deterministic pseudo-random helpers
// =============================================================================

/// Deterministic pseudo-random using wrapping multiplication (no rand crate).
const PRIME_A: u64 = 6364136223846793005;
const PRIME_B: u64 = 1442695040888963407;

/// Returns a deterministic pseudo-random value in [0.0, 1.0) for a given seed.
fn pseudo_random(seed: u64) -> f32 {
    let hash = seed.wrapping_mul(PRIME_A).wrapping_add(PRIME_B);
    // Take bits 16..48 for better distribution
    let bits = ((hash >> 16) & 0xFFFF_FFFF) as u32;
    (bits % 10000) as f32 / 10000.0
}

/// Simple sine approximation using integer ticks (avoids f64 libm dependency issues).
/// Returns value in [-1.0, 1.0].
fn sine_approx(tick: u32, period: u32) -> f32 {
    let phase = (tick % period) as f32 / period as f32;
    let x = phase * 2.0 * std::f32::consts::PI;
    x.sin()
}

// =============================================================================
// System: update_market_prices
// =============================================================================

/// Updates market prices based on supply/demand balance, market cycles, and events.
/// Runs every SlowTickTimer interval (100 ticks).
/// Also adjusts trade_balance in CityGoods to reflect market prices.
pub fn update_market_prices(
    slow_timer: Res<SlowTickTimer>,
    tick: Res<TickCounter>,
    mut market: ResMut<MarketPrices>,
    mut city_goods: ResMut<CityGoods>,
    mut budget: ResMut<CityBudget>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let cycle = market.cycle_counter;
    market.cycle_counter = cycle.wrapping_add(1);

    // -----------------------------------------------------------------
    // 1. Maybe trigger a new market event (deterministic, ~5% chance per slow tick)
    // -----------------------------------------------------------------
    let event_roll = pseudo_random(tick.0.wrapping_mul(7919));
    if event_roll < 0.05 && market.active_events.len() < 2 {
        let event_idx = (tick.0.wrapping_mul(6271) % MarketEvent::ALL.len() as u64) as usize;
        let new_event = MarketEvent::ALL[event_idx];

        // Don't duplicate an already active event
        let already_active = market.active_events.iter().any(|ae| ae.event == new_event);
        if !already_active {
            market.active_events.push(ActiveMarketEvent {
                event: new_event,
                remaining_ticks: new_event.duration_slow_ticks(),
            });
        }
    }

    // -----------------------------------------------------------------
    // 2. Compute event-based price adjustments
    // -----------------------------------------------------------------
    let mut goods_event_delta: HashMap<GoodsType, f32> = HashMap::new();
    let mut resource_event_delta: HashMap<ResourceType, f32> = HashMap::new();

    for active in &market.active_events {
        for &(goods, delta) in active.event.price_effects() {
            *goods_event_delta.entry(goods).or_insert(0.0) += delta;
        }
        for &(resource, delta) in active.event.resource_effects() {
            *resource_event_delta.entry(resource).or_insert(0.0) += delta;
        }
    }

    // Tick down event durations and remove expired
    market
        .active_events
        .iter_mut()
        .for_each(|ae| ae.remaining_ticks = ae.remaining_ticks.saturating_sub(1));
    market.active_events.retain(|ae| ae.remaining_ticks > 0);

    // -----------------------------------------------------------------
    // 3. Update goods prices based on supply/demand + cycle + events
    // -----------------------------------------------------------------
    for &g in GoodsType::all() {
        let entry = market
            .goods_prices
            .entry(g)
            .or_insert_with(|| PriceEntry::new(g.export_price()));
        entry.previous_price = entry.current_price;

        let base = entry.base_price;

        // Supply/demand factor: if city produces more than it consumes, price drops slightly
        let prod = city_goods.production_rate.get(&g).copied().unwrap_or(0.0);
        let cons = city_goods.consumption_rate.get(&g).copied().unwrap_or(0.0);
        let net = prod - cons;

        // Supply/demand multiplier: surplus pushes price down, deficit pushes up
        // Range: 0.7 to 1.5 based on net balance relative to consumption
        let sd_factor = if cons > 0.1 {
            let ratio = net / cons;
            // ratio > 0 means surplus, < 0 means deficit
            (1.0 - ratio * 0.3).clamp(0.7, 1.5)
        } else if net > 0.0 {
            0.85 // Surplus with no consumption
        } else {
            1.0
        };

        // Market cycle: sine wave with different periods per goods type
        // Creates boom/bust cycles of varying lengths
        let cycle_period = match g {
            GoodsType::RawFood => 40,
            GoodsType::ProcessedFood => 50,
            GoodsType::Lumber => 60,
            GoodsType::Steel => 45,
            GoodsType::Fuel => 35,
            GoodsType::Electronics => 55,
            GoodsType::ConsumerGoods => 65,
        };
        let cycle_amplitude = 0.12; // +/- 12% swing
        let cycle_factor = 1.0 + sine_approx(cycle, cycle_period) * cycle_amplitude;

        // Noise: small random perturbation per tick
        let noise_seed = tick.0.wrapping_mul(31).wrapping_add(g as u64 * 997);
        let noise = (pseudo_random(noise_seed) - 0.5) * 0.06; // +/- 3%
        let noise_factor = 1.0 + noise;

        // Event factor
        let event_delta = goods_event_delta.get(&g).copied().unwrap_or(0.0);
        let event_factor = 1.0 + event_delta;

        // Combine all factors
        let new_price = base
            * sd_factor as f64
            * cycle_factor as f64
            * noise_factor as f64
            * event_factor as f64;

        // Clamp to reasonable range: 30% to 300% of base
        entry.current_price = new_price.clamp(base * 0.3, base * 3.0);
    }

    // -----------------------------------------------------------------
    // 4. Update resource prices
    // -----------------------------------------------------------------
    for (&rt, entry) in market.resource_prices.iter_mut() {
        entry.previous_price = entry.current_price;
        let base = entry.base_price;

        // Resource cycle (longer period than goods)
        let cycle_period = match rt {
            ResourceType::FertileLand => 80,
            ResourceType::Forest => 70,
            ResourceType::Ore => 50,
            ResourceType::Oil => 40,
        };
        let cycle_factor = 1.0 + sine_approx(cycle, cycle_period) * 0.15;

        // Noise
        let noise_seed = tick.0.wrapping_mul(43).wrapping_add(rt as u64 * 1009);
        let noise_factor = 1.0 + (pseudo_random(noise_seed) - 0.5) * 0.05;

        // Event factor
        let event_delta = resource_event_delta.get(&rt).copied().unwrap_or(0.0);
        let event_factor = 1.0 + event_delta;

        let new_price = base * cycle_factor as f64 * noise_factor as f64 * event_factor as f64;
        entry.current_price = new_price.clamp(base * 0.3, base * 3.0);
    }

    // -----------------------------------------------------------------
    // 5. Adjust trade balance using market prices instead of fixed prices
    // -----------------------------------------------------------------
    let mut market_trade_balance = 0.0f64;
    for &g in GoodsType::all() {
        let stock = city_goods.available.get(&g).copied().unwrap_or(0.0);
        let market_price = market
            .goods_prices
            .get(&g)
            .map(|e| e.current_price)
            .unwrap_or(g.export_price());

        // Export surplus above 100 units at current market price
        if stock > 100.0 {
            let surplus = (stock - 100.0) as f64;
            market_trade_balance += surplus * market_price * 0.01;
        }

        // Import deficit at 1.8x current market price
        let net = city_goods.net(g);
        if net < -0.1 {
            let deficit = (-net).min(50.0) as f64;
            market_trade_balance -= deficit * market_price * 1.8 * 0.01;
        }
    }

    // Apply the market-adjusted trade balance delta (difference from base)
    // The production system already applies base trade_balance, so we apply
    // an additional delta for market price fluctuations
    let base_trade = city_goods.trade_balance;
    let market_delta = market_trade_balance - base_trade;
    budget.treasury += market_delta;
    city_goods.trade_balance = market_trade_balance;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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

pub struct MarketPlugin;

impl Plugin for MarketPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MarketPrices>().add_systems(
            FixedUpdate,
            update_market_prices.after(crate::production::update_production_chains),
        );
    }
}
