//! Recycling commodity prices and market cycle economics.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Base commodity prices per ton for each recyclable material category.
/// These are the "neutral" prices before market cycle adjustment.
pub(crate) const BASE_PRICE_PAPER: f64 = 80.0; // $/ton
pub(crate) const BASE_PRICE_PLASTIC: f64 = 200.0;
pub(crate) const BASE_PRICE_GLASS: f64 = 25.0;
pub(crate) const BASE_PRICE_METAL: f64 = 300.0;
pub(crate) const BASE_PRICE_ORGANIC: f64 = 15.0; // compost value

/// Processing cost per ton (sorting, baling, transport to buyers).
pub(crate) const PROCESSING_COST_PER_TON: f64 = 50.0;

/// Collection cost per ton (trucks, fuel, labor).
pub(crate) const COLLECTION_COST_PER_TON: f64 = 40.0;

/// Commodity prices and market cycle for recyclable materials.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct RecyclingEconomics {
    /// Current price per ton for paper/cardboard.
    pub price_paper: f64,
    /// Current price per ton for plastics.
    pub price_plastic: f64,
    /// Current price per ton for glass.
    pub price_glass: f64,
    /// Current price per ton for metals.
    pub price_metal: f64,
    /// Current price per ton for organics (compost).
    pub price_organic: f64,

    /// Position in the market cycle (0.0..1.0).
    /// 0.0 = start of cycle, 0.5 = peak, wraps around.
    pub market_cycle_position: f64,

    /// Day when we last updated the market cycle.
    pub last_update_day: u32,
}

impl Default for RecyclingEconomics {
    fn default() -> Self {
        Self {
            price_paper: BASE_PRICE_PAPER,
            price_plastic: BASE_PRICE_PLASTIC,
            price_glass: BASE_PRICE_GLASS,
            price_metal: BASE_PRICE_METAL,
            price_organic: BASE_PRICE_ORGANIC,
            market_cycle_position: 0.0,
            last_update_day: 0,
        }
    }
}

impl RecyclingEconomics {
    /// Market cycle period in game days (~5 game years = 5 * 365 = 1825 days).
    const CYCLE_PERIOD_DAYS: f64 = 1825.0;

    /// Market price multiplier based on cycle position.
    ///
    /// Uses a sine wave: ranges from 0.3 (bust) to 1.5 (boom).
    /// The midpoint (1.0x) occurs at positions 0.0 and 0.5 on the way up/down.
    pub fn price_multiplier(&self) -> f64 {
        // sine wave: amplitude 0.6 around mean 0.9 => range [0.3, 1.5]
        let angle = self.market_cycle_position * std::f64::consts::TAU;
        0.9 + 0.6 * angle.sin()
    }

    /// Weighted-average revenue per ton of diverted material at current prices.
    ///
    /// Uses default MSW composition weights: paper 25%, plastics 13%, metals 9%,
    /// glass 4%, organics 34% (food+yard), other 15% (no value).
    pub fn revenue_per_ton(&self) -> f64 {
        let mult = self.price_multiplier();
        (self.price_paper * 0.25
            + self.price_plastic * 0.13
            + self.price_metal * 0.09
            + self.price_glass * 0.04
            + self.price_organic * 0.34)
            * mult
    }

    /// Net value per ton of recycled material (revenue minus costs).
    /// Can be negative when market prices are low.
    pub fn net_value_per_ton(&self) -> f64 {
        self.revenue_per_ton() - PROCESSING_COST_PER_TON - COLLECTION_COST_PER_TON
    }

    /// Advance the market cycle based on elapsed game days.
    pub fn update_market_cycle(&mut self, current_day: u32) {
        if current_day <= self.last_update_day {
            return;
        }
        let elapsed = (current_day - self.last_update_day) as f64;
        self.market_cycle_position += elapsed / Self::CYCLE_PERIOD_DAYS;
        // Keep position in [0, 1)
        self.market_cycle_position -= self.market_cycle_position.floor();
        self.last_update_day = current_day;
    }
}
