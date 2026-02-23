//! Water Treatment Plant Level System (WATER-003).
//!
//! Treatment plants have upgradeable treatment levels that determine effluent
//! quality, treatment cost, and capacity. Higher treatment levels produce cleaner
//! output water but cost more to operate.
//!
//! Treatment levels:
//! - None: 0% removal (raw sewage bypass)
//! - Primary: 60% removal ($1K/MG) - physical settling
//! - Secondary: 85% removal ($2K/MG) - biological treatment
//! - Tertiary: 95% removal ($5K/MG) - nutrient removal
//! - Advanced: 99% removal ($10K/MG) - membrane filtration

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Treatment level enum
// =============================================================================

/// Treatment level for a water treatment plant, determining removal effectiveness
/// and operational cost.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TreatmentLevel {
    /// No treatment (bypass). 0% contaminant removal.
    #[default]
    None,
    /// Primary treatment (physical settling). 60% removal.
    Primary,
    /// Secondary treatment (biological). 85% removal.
    Secondary,
    /// Tertiary treatment (nutrient removal). 95% removal.
    Tertiary,
    /// Advanced treatment (membrane filtration). 99% removal.
    Advanced,
}

impl TreatmentLevel {
    /// Fraction of contaminants removed (0.0 = none, 1.0 = perfect).
    pub fn removal_efficiency(&self) -> f32 {
        match self {
            TreatmentLevel::None => 0.0,
            TreatmentLevel::Primary => 0.60,
            TreatmentLevel::Secondary => 0.85,
            TreatmentLevel::Tertiary => 0.95,
            TreatmentLevel::Advanced => 0.99,
        }
    }

    /// Treatment cost per million gallons processed (in dollars).
    pub fn cost_per_million_gallons(&self) -> f64 {
        match self {
            TreatmentLevel::None => 0.0,
            TreatmentLevel::Primary => 1_000.0,
            TreatmentLevel::Secondary => 2_000.0,
            TreatmentLevel::Tertiary => 5_000.0,
            TreatmentLevel::Advanced => 10_000.0,
        }
    }

    /// Cost to upgrade from the current level to the next level.
    /// Returns `None` if already at Advanced (max level).
    pub fn upgrade_cost(&self) -> Option<f64> {
        match self {
            TreatmentLevel::None => Some(25_000.0),
            TreatmentLevel::Primary => Some(50_000.0),
            TreatmentLevel::Secondary => Some(100_000.0),
            TreatmentLevel::Tertiary => Some(200_000.0),
            TreatmentLevel::Advanced => Option::None,
        }
    }

    /// Returns the next treatment level, or `None` if already at max.
    pub fn next_level(&self) -> Option<TreatmentLevel> {
        match self {
            TreatmentLevel::None => Some(TreatmentLevel::Primary),
            TreatmentLevel::Primary => Some(TreatmentLevel::Secondary),
            TreatmentLevel::Secondary => Some(TreatmentLevel::Tertiary),
            TreatmentLevel::Tertiary => Some(TreatmentLevel::Advanced),
            TreatmentLevel::Advanced => Option::None,
        }
    }

    /// Base capacity in million gallons per day (MGD) for a plant at this level.
    /// Higher-level plants tend to have lower throughput due to more processing stages.
    pub fn base_capacity_mgd(&self) -> f32 {
        match self {
            TreatmentLevel::None => 0.0,
            TreatmentLevel::Primary => 10.0,
            TreatmentLevel::Secondary => 8.0,
            TreatmentLevel::Tertiary => 5.0,
            TreatmentLevel::Advanced => 3.0,
        }
    }

    /// Display name for the treatment level.
    pub fn name(&self) -> &'static str {
        match self {
            TreatmentLevel::None => "None",
            TreatmentLevel::Primary => "Primary",
            TreatmentLevel::Secondary => "Secondary",
            TreatmentLevel::Tertiary => "Tertiary",
            TreatmentLevel::Advanced => "Advanced",
        }
    }
}

// =============================================================================
// Per-plant tracking
// =============================================================================

/// State for a single treatment plant entity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlantState {
    /// Current treatment level.
    pub level: TreatmentLevel,
    /// Maximum capacity in million gallons per day.
    pub capacity_mgd: f32,
    /// Current flow being processed in MGD.
    pub current_flow_mgd: f32,
    /// Effluent quality (0.0 = fully contaminated, 1.0 = pure).
    pub effluent_quality: f32,
    /// Treatment cost incurred this period (dollars).
    pub period_cost: f64,
}

impl PlantState {
    /// Create a new plant state at the given treatment level.
    pub fn new(level: TreatmentLevel) -> Self {
        Self {
            level,
            capacity_mgd: level.base_capacity_mgd(),
            current_flow_mgd: 0.0,
            effluent_quality: 0.0,
            period_cost: 0.0,
        }
    }
}

// =============================================================================
// City-wide water treatment state resource
// =============================================================================

/// City-wide water treatment state, tracking all treatment plants and aggregate metrics.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct WaterTreatmentState {
    /// Map of treatment plant entity IDs to their state.
    pub plants: HashMap<Entity, PlantState>,
    /// Total treatment capacity across all plants (MGD).
    pub total_capacity_mgd: f32,
    /// Total flow currently being processed (MGD).
    pub total_flow_mgd: f32,
    /// Weighted average effluent quality across all plants (0.0-1.0).
    pub avg_effluent_quality: f32,
    /// Total treatment cost this period (dollars).
    pub total_period_cost: f64,
    /// City-wide water demand in MGD (derived from population).
    pub city_demand_mgd: f32,
    /// Fraction of demand being treated (0.0-1.0).
    pub treatment_coverage: f32,
    /// Average input water quality before treatment (0.0-1.0).
    pub avg_input_quality: f32,
    /// Disease risk factor from untreated/poorly treated water (0.0 = safe, 1.0 = critical).
    pub disease_risk: f32,
}

impl Default for WaterTreatmentState {
    fn default() -> Self {
        Self {
            plants: HashMap::new(),
            total_capacity_mgd: 0.0,
            total_flow_mgd: 0.0,
            avg_effluent_quality: 0.0,
            total_period_cost: 0.0,
            city_demand_mgd: 0.0,
            treatment_coverage: 0.0,
            avg_input_quality: 0.5,
            disease_risk: 0.0,
        }
    }
}

impl WaterTreatmentState {
    /// Attempt to upgrade a plant to the next treatment level.
    /// Returns the upgrade cost if successful, or `None` if already at max or entity not found.
    pub fn upgrade_plant(&mut self, entity: Entity) -> Option<f64> {
        let plant = self.plants.get_mut(&entity)?;
        let cost = plant.level.upgrade_cost()?;
        let next = plant.level.next_level()?;
        plant.level = next;
        plant.capacity_mgd = next.base_capacity_mgd();
        Some(cost)
    }

    /// Register a new treatment plant entity at the given level.
    /// If the entity already exists, its state is replaced.
    pub fn register_plant(&mut self, entity: Entity, level: TreatmentLevel) {
        self.plants.insert(entity, PlantState::new(level));
    }

    /// Remove a treatment plant entity (e.g. when demolished).
    pub fn remove_plant(&mut self, entity: Entity) {
        self.plants.remove(&entity);
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Calculate effluent quality given input quality and treatment level.
///
/// Effluent = 1.0 - (1.0 - input_quality) * (1.0 - removal_efficiency)
///
/// For example, with input quality 0.3 (30% clean) and Primary treatment (60% removal):
///   effluent = 1.0 - (0.7 * 0.4) = 1.0 - 0.28 = 0.72
pub fn calculate_effluent_quality(input_quality: f32, level: TreatmentLevel) -> f32 {
    let contamination = 1.0 - input_quality.clamp(0.0, 1.0);
    let remaining = contamination * (1.0 - level.removal_efficiency());
    (1.0 - remaining).clamp(0.0, 1.0)
}

/// Calculate disease risk from drinking water quality.
///
/// Quality 0.0 (fully contaminated) = risk 1.0
/// Quality 0.6 = moderate risk
/// Quality >= 0.85 = negligible risk (< 0.05)
/// Quality >= 0.95 = essentially zero risk
pub fn calculate_disease_risk(drinking_water_quality: f32) -> f32 {
    let q = drinking_water_quality.clamp(0.0, 1.0);
    if q >= 0.95 {
        0.0
    } else {
        // Quadratic increase as quality drops: deficit² curve
        // At 0.95: 0.0025, at 0.85: 0.0225, at 0.5: 0.25, at 0.0: 1.0
        let deficit = 1.0 - q;
        (deficit * deficit).min(1.0)
    }
}

/// Estimate city water demand in MGD from population count.
///
/// Uses 150 gallons per capita per day (GPCD), converted to MGD.
pub fn estimate_demand_mgd(population: u32) -> f32 {
    const GPCD: f32 = 150.0;
    (population as f32 * GPCD) / 1_000_000.0
}
