//! Core landfill types: liner type, status, and individual site data.

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::constants::*;

// =============================================================================
// LandfillLinerType
// =============================================================================

/// Type of liner installed at a landfill, determining environmental impact.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode,
)]
pub enum LandfillLinerType {
    /// No liner: high groundwater pollution, large odor radius, worst land value impact.
    #[default]
    Unlined,
    /// Synthetic or clay liner: low groundwater pollution, moderate odor radius.
    Lined,
    /// Liner plus active gas collection system: minimal pollution, small odor radius.
    LinedWithCollection,
}

impl LandfillLinerType {
    /// Returns the groundwater pollution factor (0.0-1.0) for this liner type.
    pub fn groundwater_pollution_factor(self) -> f32 {
        match self {
            Self::Unlined => GROUNDWATER_POLLUTION_UNLINED,
            Self::Lined => GROUNDWATER_POLLUTION_LINED,
            Self::LinedWithCollection => GROUNDWATER_POLLUTION_LINED_COLLECTION,
        }
    }

    /// Returns the odor radius in grid cells for this liner type.
    pub fn odor_radius(self) -> u32 {
        match self {
            Self::Unlined => ODOR_RADIUS_UNLINED,
            Self::Lined => ODOR_RADIUS_LINED,
            Self::LinedWithCollection => ODOR_RADIUS_LINED_COLLECTION,
        }
    }

    /// Returns the land value penalty fraction (0.0-1.0) for this liner type.
    pub fn land_value_penalty(self) -> f32 {
        match self {
            Self::Unlined => LAND_VALUE_PENALTY_UNLINED,
            Self::Lined => LAND_VALUE_PENALTY_LINED,
            Self::LinedWithCollection => LAND_VALUE_PENALTY_LINED_COLLECTION,
        }
    }

    /// Returns whether this liner type includes gas collection.
    pub fn has_gas_collection(self) -> bool {
        matches!(self, Self::LinedWithCollection)
    }

    /// Returns a human-readable label for this liner type.
    pub fn label(self) -> &'static str {
        match self {
            Self::Unlined => "Unlined",
            Self::Lined => "Lined",
            Self::LinedWithCollection => "Lined + Gas Collection",
        }
    }
}

// =============================================================================
// LandfillStatus
// =============================================================================

/// Status of a landfill site throughout its lifecycle.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode,
)]
pub enum LandfillStatus {
    /// Actively receiving waste.
    #[default]
    Active,
    /// Full and capped, undergoing post-closure monitoring.
    Closed {
        /// Number of slow ticks (game days) since closure.
        days_since_closure: u32,
    },
    /// Post-closure monitoring complete (30+ years). Site converted to park.
    ConvertedToPark,
}

impl LandfillStatus {
    /// Returns a human-readable label for this status.
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Closed { .. } => "Closed (Monitoring)",
            Self::ConvertedToPark => "Converted to Park",
        }
    }

    /// Returns whether this landfill is actively receiving waste.
    pub fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns the number of years since closure, or 0 if not closed.
    pub fn years_since_closure(self) -> f32 {
        match self {
            Self::Closed { days_since_closure } => days_since_closure as f32 / DAYS_PER_YEAR,
            _ => 0.0,
        }
    }
}

// =============================================================================
// LandfillSite
// =============================================================================

/// Individual landfill site data.
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct LandfillSite {
    /// Unique identifier for this landfill site.
    pub id: u32,
    /// Grid X position of this landfill.
    pub grid_x: usize,
    /// Grid Y position of this landfill.
    pub grid_y: usize,
    /// Total capacity of this landfill in tons.
    pub total_capacity_tons: f64,
    /// Current fill level in tons.
    pub current_fill_tons: f64,
    /// Current daily input rate in tons/day (updated each tick).
    pub daily_input_tons: f64,
    /// Type of liner installed.
    pub liner_type: LandfillLinerType,
    /// Current operational status.
    pub status: LandfillStatus,
}

impl LandfillSite {
    /// Create a new active landfill site with default capacity.
    pub fn new(id: u32, grid_x: usize, grid_y: usize) -> Self {
        Self {
            id,
            grid_x,
            grid_y,
            total_capacity_tons: DEFAULT_LANDFILL_CAPACITY_TONS,
            current_fill_tons: 0.0,
            daily_input_tons: 0.0,
            liner_type: LandfillLinerType::default(),
            status: LandfillStatus::Active,
        }
    }

    /// Create a new landfill site with specified capacity and liner type.
    pub fn with_capacity_and_liner(
        id: u32,
        grid_x: usize,
        grid_y: usize,
        capacity: f64,
        liner_type: LandfillLinerType,
    ) -> Self {
        Self {
            id,
            grid_x,
            grid_y,
            total_capacity_tons: capacity,
            current_fill_tons: 0.0,
            daily_input_tons: 0.0,
            liner_type,
            status: LandfillStatus::Active,
        }
    }

    /// Returns remaining capacity in tons.
    pub fn remaining_capacity_tons(&self) -> f64 {
        (self.total_capacity_tons - self.current_fill_tons).max(0.0)
    }

    /// Returns remaining capacity as a percentage (0.0 to 100.0).
    pub fn remaining_capacity_pct(&self) -> f64 {
        if self.total_capacity_tons <= 0.0 {
            return 0.0;
        }
        (self.remaining_capacity_tons() / self.total_capacity_tons * 100.0).clamp(0.0, 100.0)
    }

    /// Returns estimated days remaining at current daily input rate.
    /// Returns `f32::INFINITY` if daily input is zero or negative.
    pub fn days_remaining(&self) -> f32 {
        if self.daily_input_tons <= 0.0 {
            return f32::INFINITY;
        }
        (self.remaining_capacity_tons() / self.daily_input_tons) as f32
    }

    /// Returns estimated years remaining at current daily input rate.
    pub fn years_remaining(&self) -> f32 {
        self.days_remaining() / DAYS_PER_YEAR
    }

    /// Returns whether this landfill is full (no remaining capacity).
    pub fn is_full(&self) -> bool {
        self.current_fill_tons >= self.total_capacity_tons
    }

    /// Returns the electricity generated in MW from landfill gas, if gas collection is enabled.
    /// Conversion: 1 MW per 1,000 tons/day of waste input.
    pub fn gas_electricity_mw(&self) -> f64 {
        if !self.liner_type.has_gas_collection() {
            return 0.0;
        }
        self.daily_input_tons * GAS_COLLECTION_MW_PER_1000_TONS_DAY / 1000.0
    }

    /// Advance fill by one day's input. Clamps at total capacity.
    /// If the landfill becomes full, transitions to Closed status.
    pub fn advance_fill(&mut self, daily_input: f64) {
        if !self.status.is_active() {
            return;
        }
        self.daily_input_tons = daily_input;
        self.current_fill_tons =
            (self.current_fill_tons + daily_input).min(self.total_capacity_tons);
        if self.is_full() {
            self.status = LandfillStatus::Closed {
                days_since_closure: 0,
            };
        }
    }

    /// Advance post-closure monitoring by one day.
    /// After POST_CLOSURE_MONITORING_YEARS * 365 days, transitions to ConvertedToPark.
    pub fn advance_closure(&mut self) {
        if let LandfillStatus::Closed { days_since_closure } = &mut self.status {
            *days_since_closure += 1;
            let monitoring_days = POST_CLOSURE_MONITORING_YEARS as f32 * DAYS_PER_YEAR;
            if *days_since_closure as f32 >= monitoring_days {
                self.status = LandfillStatus::ConvertedToPark;
            }
        }
    }
}
