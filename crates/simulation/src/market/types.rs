use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::natural_resources::ResourceType;
use crate::production::GoodsType;

use super::events::ActiveMarketEvent;

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
