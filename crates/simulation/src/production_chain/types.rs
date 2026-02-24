//! Types for the deep production chain system (SERV-009).

use std::collections::HashMap;

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

// =============================================================================
// Commodity — fine-grained goods flowing through multi-stage chains
// =============================================================================

/// Commodities that flow through deep production chains.
/// These are more specific than `GoodsType` and represent intermediate and
/// final products in multi-stage manufacturing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum Commodity {
    // Stage 1: Raw materials (from extraction industries)
    Grain,
    Timber,
    CrudeOil,
    IronOre,

    // Stage 2: Processed goods (from processing industries)
    Flour,
    Lumber,
    Petroleum,
    Steel,

    // Stage 3: Consumer products (from manufacturing)
    Bread,
    Furniture,
    Plastics,
    Machinery,
}

impl Commodity {
    pub fn name(self) -> &'static str {
        match self {
            Self::Grain => "Grain",
            Self::Timber => "Timber",
            Self::CrudeOil => "Crude Oil",
            Self::IronOre => "Iron Ore",
            Self::Flour => "Flour",
            Self::Lumber => "Lumber",
            Self::Petroleum => "Petroleum",
            Self::Steel => "Steel",
            Self::Bread => "Bread",
            Self::Furniture => "Furniture",
            Self::Plastics => "Plastics",
            Self::Machinery => "Machinery",
        }
    }

    /// All commodity types.
    pub fn all() -> &'static [Commodity] {
        &[
            Self::Grain,
            Self::Timber,
            Self::CrudeOil,
            Self::IronOre,
            Self::Flour,
            Self::Lumber,
            Self::Petroleum,
            Self::Steel,
            Self::Bread,
            Self::Furniture,
            Self::Plastics,
            Self::Machinery,
        ]
    }

    /// Whether this is a raw material (stage 1).
    pub fn is_raw(self) -> bool {
        matches!(
            self,
            Self::Grain | Self::Timber | Self::CrudeOil | Self::IronOre
        )
    }

    /// Whether this is a processed good (stage 2).
    pub fn is_processed(self) -> bool {
        matches!(
            self,
            Self::Flour | Self::Lumber | Self::Petroleum | Self::Steel
        )
    }

    /// Whether this is a final consumer product (stage 3).
    pub fn is_final(self) -> bool {
        matches!(
            self,
            Self::Bread | Self::Furniture | Self::Plastics | Self::Machinery
        )
    }

    /// Base export price per unit.
    pub fn export_price(self) -> f64 {
        match self {
            Self::Grain => 1.5,
            Self::Timber => 2.0,
            Self::CrudeOil => 3.0,
            Self::IronOre => 2.5,
            Self::Flour => 4.0,
            Self::Lumber => 5.0,
            Self::Petroleum => 7.0,
            Self::Steel => 8.0,
            Self::Bread => 8.0,
            Self::Furniture => 12.0,
            Self::Plastics => 10.0,
            Self::Machinery => 15.0,
        }
    }

    /// Import price per unit (more expensive than export).
    pub fn import_price(self) -> f64 {
        self.export_price() * 1.6
    }

    /// Which chain this commodity belongs to (0=food, 1=forestry, 2=oil, 3=mining).
    pub fn chain_index(self) -> usize {
        match self {
            Self::Grain | Self::Flour | Self::Bread => 0,
            Self::Timber | Self::Lumber | Self::Furniture => 1,
            Self::CrudeOil | Self::Petroleum | Self::Plastics => 2,
            Self::IronOre | Self::Steel | Self::Machinery => 3,
        }
    }
}

// =============================================================================
// Production Stage — describes one step in a chain
// =============================================================================

/// A single production step: consume inputs to produce outputs.
#[derive(Debug, Clone)]
pub struct ProductionStage {
    /// Inputs consumed per production cycle.
    pub inputs: &'static [(Commodity, f32)],
    /// Outputs produced per production cycle.
    pub outputs: &'static [(Commodity, f32)],
    /// Which `IndustryType` runs this stage.
    pub industry: crate::production::IndustryType,
}

/// The 4 production chains, each with 3 stages.
pub fn all_chains() -> &'static [[ProductionStage; 3]; 4] {
    use crate::production::IndustryType;
    static CHAINS: [[ProductionStage; 3]; 4] = [
        // Chain 0: Food — Grain -> Flour -> Bread
        [
            ProductionStage {
                inputs: &[],
                outputs: &[(Commodity::Grain, 2.0)],
                industry: IndustryType::Agriculture,
            },
            ProductionStage {
                inputs: &[(Commodity::Grain, 2.0)],
                outputs: &[(Commodity::Flour, 1.5)],
                industry: IndustryType::FoodProcessing,
            },
            ProductionStage {
                inputs: &[(Commodity::Flour, 1.5)],
                outputs: &[(Commodity::Bread, 1.0)],
                industry: IndustryType::Manufacturing,
            },
        ],
        // Chain 1: Forestry — Timber -> Lumber -> Furniture
        [
            ProductionStage {
                inputs: &[],
                outputs: &[(Commodity::Timber, 1.5)],
                industry: IndustryType::Forestry,
            },
            ProductionStage {
                inputs: &[(Commodity::Timber, 1.5)],
                outputs: &[(Commodity::Lumber, 1.0)],
                industry: IndustryType::SawMill,
            },
            ProductionStage {
                inputs: &[(Commodity::Lumber, 1.0)],
                outputs: &[(Commodity::Furniture, 0.7)],
                industry: IndustryType::Manufacturing,
            },
        ],
        // Chain 2: Oil — CrudeOil -> Petroleum -> Plastics
        [
            ProductionStage {
                inputs: &[],
                outputs: &[(Commodity::CrudeOil, 1.2)],
                industry: IndustryType::OilExtraction,
            },
            ProductionStage {
                inputs: &[(Commodity::CrudeOil, 1.2)],
                outputs: &[(Commodity::Petroleum, 0.8)],
                industry: IndustryType::Refinery,
            },
            ProductionStage {
                inputs: &[(Commodity::Petroleum, 0.8)],
                outputs: &[(Commodity::Plastics, 0.5)],
                industry: IndustryType::Manufacturing,
            },
        ],
        // Chain 3: Mining — IronOre -> Steel -> Machinery
        [
            ProductionStage {
                inputs: &[],
                outputs: &[(Commodity::IronOre, 1.0)],
                industry: IndustryType::Mining,
            },
            ProductionStage {
                inputs: &[(Commodity::IronOre, 1.0)],
                outputs: &[(Commodity::Steel, 0.7)],
                industry: IndustryType::Smelter,
            },
            ProductionStage {
                inputs: &[(Commodity::Steel, 0.7)],
                outputs: &[(Commodity::Machinery, 0.4)],
                industry: IndustryType::TechAssembly,
            },
        ],
    ];
    &CHAINS
}

// =============================================================================
// ECS Components
// =============================================================================

/// Component marking a building as part of a deep production chain.
/// Tracks its chain index, stage, and local commodity storage.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct DeepChainBuilding {
    /// Which chain (0=food, 1=forestry, 2=oil, 3=mining).
    pub chain_index: usize,
    /// Which stage in the chain (0=extraction, 1=processing, 2=manufacturing).
    pub stage: usize,
    /// Local input buffer for this building.
    pub input_buffer: HashMap<Commodity, f32>,
    /// Local output buffer for this building.
    pub output_buffer: HashMap<Commodity, f32>,
    /// Whether this building is currently halted due to missing inputs.
    pub disrupted: bool,
    /// How many ticks this building has been disrupted.
    pub disruption_ticks: u32,
}

impl DeepChainBuilding {
    pub fn new(chain_index: usize, stage: usize) -> Self {
        Self {
            chain_index,
            stage,
            input_buffer: HashMap::new(),
            output_buffer: HashMap::new(),
            disrupted: false,
            disruption_ticks: 0,
        }
    }

    /// Maximum capacity per commodity in this building's buffers.
    pub const BUFFER_CAPACITY: f32 = 50.0;
}

/// Component for warehouse buildings that buffer goods between production stages.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseBuilding {
    /// Stored commodities and their amounts.
    pub storage: HashMap<Commodity, f32>,
    /// Maximum storage capacity per commodity.
    pub capacity: f32,
}

impl WarehouseBuilding {
    pub fn new(capacity: f32) -> Self {
        Self {
            storage: HashMap::new(),
            capacity,
        }
    }
}

// =============================================================================
// City-wide Production Chain State
// =============================================================================

/// City-wide state for the deep production chain system.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct DeepProductionChainState {
    /// Global commodity stockpile (shared pool for the city).
    pub stockpile: HashMap<Commodity, f32>,
    /// Production rate per tick for each commodity.
    pub production_rates: HashMap<Commodity, f32>,
    /// Consumption rate per tick for each commodity.
    pub consumption_rates: HashMap<Commodity, f32>,
    /// Per-chain disruption status (true = at least one stage is halted).
    pub chain_disrupted: [bool; 4],
    /// Total disrupted buildings count.
    pub disrupted_count: u32,
    /// Trade balance from commodity import/export (per tick contribution).
    pub commodity_trade_balance: f64,
    /// Total commodities exported since last reset.
    pub total_exported: f64,
    /// Total commodities imported since last reset.
    pub total_imported: f64,
}

impl Default for DeepProductionChainState {
    fn default() -> Self {
        let mut stockpile = HashMap::new();
        let mut production_rates = HashMap::new();
        let mut consumption_rates = HashMap::new();
        for &c in Commodity::all() {
            stockpile.insert(c, 0.0);
            production_rates.insert(c, 0.0);
            consumption_rates.insert(c, 0.0);
        }
        Self {
            stockpile,
            production_rates,
            consumption_rates,
            chain_disrupted: [false; 4],
            disrupted_count: 0,
            commodity_trade_balance: 0.0,
            total_exported: 0.0,
            total_imported: 0.0,
        }
    }
}

impl DeepProductionChainState {
    /// Net balance for a commodity (positive = surplus).
    pub fn net(&self, commodity: Commodity) -> f32 {
        let prod = self.production_rates.get(&commodity).copied().unwrap_or(0.0);
        let cons = self.consumption_rates.get(&commodity).copied().unwrap_or(0.0);
        prod - cons
    }

    /// Current stock of a commodity.
    pub fn stock(&self, commodity: Commodity) -> f32 {
        self.stockpile.get(&commodity).copied().unwrap_or(0.0)
    }
}

// =============================================================================
// Save/Load
// =============================================================================

#[derive(Debug, Clone, Default, Encode, Decode)]
pub(crate) struct ChainSaveData {
    pub(crate) stockpile: Vec<(u8, f32)>,
    pub(crate) chain_disrupted: [bool; 4],
    pub(crate) total_exported: f64,
    pub(crate) total_imported: f64,
}

impl crate::Saveable for DeepProductionChainState {
    const SAVE_KEY: &'static str = "production_chain";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        let stockpile: Vec<(u8, f32)> = self
            .stockpile
            .iter()
            .filter(|(_, &v)| v > 0.01)
            .map(|(&k, &v)| (k as u8, v))
            .collect();
        if stockpile.is_empty()
            && self.total_exported < 0.01
            && self.total_imported < 0.01
        {
            return None;
        }
        let data = ChainSaveData {
            stockpile,
            chain_disrupted: self.chain_disrupted,
            total_exported: self.total_exported,
            total_imported: self.total_imported,
        };
        Some(bitcode::encode(&data))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let data: ChainSaveData = crate::decode_or_warn(Self::SAVE_KEY, bytes);
        let mut state = Self::default();
        let all = Commodity::all();
        for (idx, amount) in data.stockpile {
            if let Some(&commodity) = all.get(idx as usize) {
                state.stockpile.insert(commodity, amount);
            }
        }
        state.chain_disrupted = data.chain_disrupted;
        state.total_exported = data.total_exported;
        state.total_imported = data.total_imported;
        state
    }
}
