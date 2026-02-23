use std::collections::HashMap;

use bevy::prelude::*;

use crate::economy::CityBudget;
use crate::production::{CityGoods, GoodsType};
use crate::SlowTickTimer;
use crate::TickCounter;

use super::events::{ActiveMarketEvent, MarketEvent};
use super::types::{MarketPrices, PriceEntry};

// =============================================================================
// Deterministic pseudo-random helpers
// =============================================================================

/// Deterministic pseudo-random using wrapping multiplication (no rand crate).
const PRIME_A: u64 = 6364136223846793005;
const PRIME_B: u64 = 1442695040888963407;

/// Returns a deterministic pseudo-random value in [0.0, 1.0) for a given seed.
pub(crate) fn pseudo_random(seed: u64) -> f32 {
    let hash = seed.wrapping_mul(PRIME_A).wrapping_add(PRIME_B);
    // Take bits 16..48 for better distribution
    let bits = ((hash >> 16) & 0xFFFF_FFFF) as u32;
    (bits % 10000) as f32 / 10000.0
}

/// Simple sine approximation using integer ticks (avoids f64 libm dependency issues).
/// Returns value in [-1.0, 1.0].
pub(crate) fn sine_approx(tick: u32, period: u32) -> f32 {
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
    let mut resource_event_delta: HashMap<crate::natural_resources::ResourceType, f32> =
        HashMap::new();

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
    update_goods_prices(&mut market, &city_goods, cycle, &tick, &goods_event_delta);

    // -----------------------------------------------------------------
    // 4. Update resource prices
    // -----------------------------------------------------------------
    update_resource_prices(&mut market, cycle, &tick, &resource_event_delta);

    // -----------------------------------------------------------------
    // 5. Adjust trade balance using market prices instead of fixed prices
    // -----------------------------------------------------------------
    update_trade_balance(&market, &mut city_goods, &mut budget);
}

fn update_goods_prices(
    market: &mut MarketPrices,
    city_goods: &CityGoods,
    cycle: u32,
    tick: &TickCounter,
    goods_event_delta: &HashMap<GoodsType, f32>,
) {
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
}

fn update_resource_prices(
    market: &mut MarketPrices,
    cycle: u32,
    tick: &TickCounter,
    resource_event_delta: &HashMap<crate::natural_resources::ResourceType, f32>,
) {
    for (&rt, entry) in market.resource_prices.iter_mut() {
        entry.previous_price = entry.current_price;
        let base = entry.base_price;

        // Resource cycle (longer period than goods)
        let cycle_period = match rt {
            crate::natural_resources::ResourceType::FertileLand => 80,
            crate::natural_resources::ResourceType::Forest => 70,
            crate::natural_resources::ResourceType::Ore => 50,
            crate::natural_resources::ResourceType::Oil => 40,
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
}

fn update_trade_balance(
    market: &MarketPrices,
    city_goods: &mut CityGoods,
    budget: &mut CityBudget,
) {
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
