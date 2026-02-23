//! Flood protection types, constants, and state resource.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Saveable;

// =============================================================================
// Constants
// =============================================================================

/// Default design height for levees in feet.
pub(crate) const LEVEE_DESIGN_HEIGHT: f32 = 10.0;

/// Default design height for seawalls in feet.
pub(crate) const SEAWALL_DESIGN_HEIGHT: f32 = 15.0;

/// Default design height for closed floodgates in feet.
pub(crate) const FLOODGATE_DESIGN_HEIGHT: f32 = 12.0;

/// Annual maintenance cost per protection cell in dollars.
pub(crate) const MAINTENANCE_COST_PER_CELL_PER_YEAR: f64 = 2_000.0;

/// Number of game days per year for maintenance cost calculation.
pub(crate) const DAYS_PER_YEAR: u32 = 360;

/// Condition degradation rate per slow tick when maintenance is not funded.
/// A fully maintained structure stays at condition 1.0.
pub(crate) const DEGRADATION_RATE_PER_TICK: f32 = 0.002;

/// Condition recovery rate per slow tick when maintenance IS funded.
pub(crate) const RECOVERY_RATE_PER_TICK: f32 = 0.001;

/// Base failure probability per tick for a structure at full condition.
pub(crate) const BASE_FAILURE_PROB: f32 = 0.0001;

/// Additional failure probability per year of age.
pub(crate) const AGE_FAILURE_FACTOR: f32 = 0.00005;

/// Failure probability multiplier when condition is low (scales as 1/condition).
/// At condition 0.5, failure prob is doubled; at 0.25, quadrupled.
pub(crate) const CONDITION_FAILURE_EXPONENT: f32 = 2.0;

/// When a levee fails due to overtopping, flood depth is amplified by this factor.
pub(crate) const OVERTOPPING_AMPLIFICATION: f32 = 1.5;

// =============================================================================
// Types
// =============================================================================

/// Type of flood protection structure.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, bitcode::Encode, bitcode::Decode,
)]
pub enum ProtectionType {
    /// River levee: prevents flooding up to design height.
    Levee,
    /// Coastal seawall: prevents coastal surge.
    Seawall,
    /// Floodgate: controllable water barrier.
    Floodgate,
}

impl ProtectionType {
    /// Returns the design protection height in feet.
    pub fn design_height(self) -> f32 {
        match self {
            ProtectionType::Levee => LEVEE_DESIGN_HEIGHT,
            ProtectionType::Seawall => SEAWALL_DESIGN_HEIGHT,
            ProtectionType::Floodgate => FLOODGATE_DESIGN_HEIGHT,
        }
    }
}

/// A single flood protection structure placed on the grid.
#[derive(Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct ProtectionStructure {
    /// Grid X coordinate.
    pub grid_x: u16,
    /// Grid Y coordinate.
    pub grid_y: u16,
    /// Type of protection.
    pub protection_type: ProtectionType,
    /// Structural condition (0.0 = destroyed, 1.0 = perfect).
    pub condition: f32,
    /// Age in game days since placement.
    pub age_days: u32,
    /// Whether this structure has failed (overtopped or collapsed).
    pub failed: bool,
    /// Whether the floodgate is currently open (only relevant for Floodgate type).
    pub gate_open: bool,
}

impl ProtectionStructure {
    /// Create a new protection structure at the given grid position.
    pub fn new(grid_x: usize, grid_y: usize, protection_type: ProtectionType) -> Self {
        Self {
            grid_x: grid_x as u16,
            grid_y: grid_y as u16,
            protection_type,
            condition: 1.0,
            age_days: 0,
            failed: false,
            gate_open: false,
        }
    }

    /// Effective protection height accounting for condition degradation.
    pub fn effective_height(&self) -> f32 {
        if self.failed {
            return 0.0;
        }
        if self.protection_type == ProtectionType::Floodgate && self.gate_open {
            return 0.0;
        }
        self.protection_type.design_height() * self.condition
    }

    /// Calculate failure probability for this tick based on age and condition.
    pub fn failure_probability(&self) -> f32 {
        if self.failed {
            return 0.0; // Already failed
        }
        let age_factor = 1.0 + self.age_days as f32 * AGE_FAILURE_FACTOR;
        let condition_factor = if self.condition > 0.01 {
            (1.0 / self.condition).powf(CONDITION_FAILURE_EXPONENT)
        } else {
            100.0 // Essentially guaranteed failure
        };
        (BASE_FAILURE_PROB * age_factor * condition_factor).min(1.0)
    }
}

// =============================================================================
// FloodProtectionState resource
// =============================================================================

/// Resource tracking all flood protection infrastructure in the city.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct FloodProtectionState {
    /// All placed protection structures.
    pub structures: Vec<ProtectionStructure>,
    /// Total number of active (non-failed) protection cells.
    pub active_count: u32,
    /// Total number of failed protection cells.
    pub failed_count: u32,
    /// Total annual maintenance cost for all structures.
    pub annual_maintenance_cost: f64,
    /// Whether maintenance is currently funded from the budget.
    pub maintenance_funded: bool,
    /// Total flood damage prevented this period (estimated).
    pub damage_prevented: f64,
    /// Last game day that yearly maintenance was charged.
    pub last_maintenance_day: u32,
    /// Number of overtopping events this period.
    pub overtopping_events: u32,
}

impl Default for FloodProtectionState {
    fn default() -> Self {
        Self {
            structures: Vec::new(),
            active_count: 0,
            failed_count: 0,
            annual_maintenance_cost: 0.0,
            maintenance_funded: true,
            damage_prevented: 0.0,
            last_maintenance_day: 0,
            overtopping_events: 0,
        }
    }
}

impl Saveable for FloodProtectionState {
    const SAVE_KEY: &'static str = "flood_protection";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.structures.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
