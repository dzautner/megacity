use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Industry Types
// =============================================================================

/// The kind of industry an industrial building operates as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndustryType {
    // Extraction (primary sector) -- harvest raw materials from ResourceGrid deposits
    Agriculture,
    Forestry,
    Mining,
    OilExtraction,
    // Processing (secondary sector) -- convert raw -> refined goods
    FoodProcessing,
    SawMill,
    Smelter,
    Refinery,
    // Manufacturing (tertiary production) -- refined -> consumer goods
    Manufacturing,
    TechAssembly,
}

impl IndustryType {
    /// Whether this industry extracts raw materials from the resource grid.
    pub fn is_extraction(self) -> bool {
        matches!(
            self,
            Self::Agriculture | Self::Forestry | Self::Mining | Self::OilExtraction
        )
    }

    /// Whether this industry is a processing stage.
    pub fn is_processing(self) -> bool {
        matches!(
            self,
            Self::FoodProcessing | Self::SawMill | Self::Smelter | Self::Refinery
        )
    }

    /// Whether this industry creates final consumer goods.
    pub fn is_manufacturing(self) -> bool {
        matches!(self, Self::Manufacturing | Self::TechAssembly)
    }

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Agriculture => "Agriculture",
            Self::Forestry => "Forestry",
            Self::Mining => "Mining",
            Self::OilExtraction => "Oil Extraction",
            Self::FoodProcessing => "Food Processing",
            Self::SawMill => "Saw Mill",
            Self::Smelter => "Smelter",
            Self::Refinery => "Refinery",
            Self::Manufacturing => "Manufacturing",
            Self::TechAssembly => "Tech Assembly",
        }
    }
}

// =============================================================================
// Goods Types
// =============================================================================

/// The tradeable goods that flow through production chains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GoodsType {
    RawFood,
    ProcessedFood,
    Lumber,
    Steel,
    Fuel,
    Electronics,
    ConsumerGoods,
}

impl GoodsType {
    pub fn name(self) -> &'static str {
        match self {
            Self::RawFood => "Raw Food",
            Self::ProcessedFood => "Processed Food",
            Self::Lumber => "Lumber",
            Self::Steel => "Steel",
            Self::Fuel => "Fuel",
            Self::Electronics => "Electronics",
            Self::ConsumerGoods => "Consumer Goods",
        }
    }

    /// All goods types, useful for iteration.
    pub fn all() -> &'static [GoodsType] {
        &[
            Self::RawFood,
            Self::ProcessedFood,
            Self::Lumber,
            Self::Steel,
            Self::Fuel,
            Self::Electronics,
            Self::ConsumerGoods,
        ]
    }

    /// Export price per unit of surplus.
    pub fn export_price(self) -> f64 {
        match self {
            Self::RawFood => 2.0,
            Self::ProcessedFood => 5.0,
            Self::Lumber => 3.0,
            Self::Steel => 6.0,
            Self::Fuel => 8.0,
            Self::Electronics => 12.0,
            Self::ConsumerGoods => 7.0,
        }
    }

    /// Import price per unit of deficit (more expensive than export).
    pub fn import_price(self) -> f64 {
        self.export_price() * 1.8
    }
}

// =============================================================================
// Production Chain definitions
// =============================================================================

/// A production chain describes the conversion of input goods to output goods
/// with a specific ratio. Each unit of output requires `input_ratio` units of
/// each input and produces `output_ratio` units of each output per worker per tick.
#[derive(Debug, Clone)]
pub struct ProductionChain {
    pub inputs: Vec<(GoodsType, f32)>, // (goods, amount consumed per unit of production)
    pub outputs: Vec<(GoodsType, f32)>, // (goods, amount produced per unit of production)
}

/// Returns the production chain for a given industry type.
pub(crate) fn chain_for(industry: IndustryType) -> ProductionChain {
    match industry {
        // Extraction: no goods input, produces raw materials
        IndustryType::Agriculture => ProductionChain {
            inputs: vec![],
            outputs: vec![(GoodsType::RawFood, 1.0)],
        },
        IndustryType::Forestry => ProductionChain {
            inputs: vec![],
            outputs: vec![(GoodsType::Lumber, 0.8)],
        },
        IndustryType::Mining => ProductionChain {
            inputs: vec![],
            outputs: vec![(GoodsType::Steel, 0.5)],
        },
        IndustryType::OilExtraction => ProductionChain {
            inputs: vec![],
            outputs: vec![(GoodsType::Fuel, 0.6)],
        },
        // Processing: raw -> refined
        IndustryType::FoodProcessing => ProductionChain {
            inputs: vec![(GoodsType::RawFood, 2.0)],
            outputs: vec![(GoodsType::ProcessedFood, 1.5)],
        },
        IndustryType::SawMill => ProductionChain {
            inputs: vec![(GoodsType::Lumber, 1.5)],
            outputs: vec![(GoodsType::ConsumerGoods, 1.0)],
        },
        IndustryType::Smelter => ProductionChain {
            inputs: vec![(GoodsType::Steel, 1.0)],
            outputs: vec![(GoodsType::ConsumerGoods, 0.8)],
        },
        IndustryType::Refinery => ProductionChain {
            inputs: vec![(GoodsType::Fuel, 1.0)],
            outputs: vec![(GoodsType::Fuel, 1.5)], // refining increases usable fuel
        },
        // Manufacturing: refined -> consumer/electronics
        IndustryType::Manufacturing => ProductionChain {
            inputs: vec![(GoodsType::Steel, 0.5), (GoodsType::Lumber, 0.3)],
            outputs: vec![(GoodsType::ConsumerGoods, 1.2)],
        },
        IndustryType::TechAssembly => ProductionChain {
            inputs: vec![(GoodsType::Steel, 0.8), (GoodsType::Fuel, 0.3)],
            outputs: vec![(GoodsType::Electronics, 0.6)],
        },
    }
}

// =============================================================================
// Components and Resources
// =============================================================================

/// Attached to industrial buildings to track their industry specialization and storage.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct IndustryBuilding {
    pub industry_type: IndustryType,
    pub input_storage: HashMap<GoodsType, f32>,
    pub output_storage: HashMap<GoodsType, f32>,
    pub workers: u32,
    pub efficiency: f32, // 0.0..1.0+, scales with worker count and education
}

impl IndustryBuilding {
    pub fn new(industry_type: IndustryType) -> Self {
        Self {
            industry_type,
            input_storage: HashMap::new(),
            output_storage: HashMap::new(),
            workers: 0,
            efficiency: 0.0,
        }
    }
}

/// City-wide tracking of available goods, production rates, and consumption rates.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CityGoods {
    /// Current stockpile of each goods type.
    pub available: HashMap<GoodsType, f32>,
    /// Production rate per tick (updated each production cycle).
    pub production_rate: HashMap<GoodsType, f32>,
    /// Consumption rate per tick (updated each production cycle).
    pub consumption_rate: HashMap<GoodsType, f32>,
    /// Trade balance from goods surplus/deficit (updated per cycle, applied monthly).
    pub trade_balance: f64,
}

impl Default for CityGoods {
    fn default() -> Self {
        let mut available = HashMap::new();
        let mut production_rate = HashMap::new();
        let mut consumption_rate = HashMap::new();
        for &g in GoodsType::all() {
            available.insert(g, 0.0);
            production_rate.insert(g, 0.0);
            consumption_rate.insert(g, 0.0);
        }
        Self {
            available,
            production_rate,
            consumption_rate,
            trade_balance: 0.0,
        }
    }
}

impl CityGoods {
    /// Net balance for a goods type (positive = surplus, negative = deficit).
    pub fn net(&self, goods: GoodsType) -> f32 {
        let prod = self.production_rate.get(&goods).copied().unwrap_or(0.0);
        let cons = self.consumption_rate.get(&goods).copied().unwrap_or(0.0);
        prod - cons
    }
}
