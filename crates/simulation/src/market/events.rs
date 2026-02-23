use serde::{Deserialize, Serialize};

use crate::natural_resources::ResourceType;
use crate::production::GoodsType;

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
// Active Event tracking
// =============================================================================

/// An active market event with remaining duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveMarketEvent {
    pub event: MarketEvent,
    pub remaining_ticks: u32,
}
