//! Types for hazardous waste management.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Treatment type enum
// =============================================================================

/// Treatment method used by a hazardous waste facility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TreatmentType {
    /// Chemical treatment: neutralization, oxidation, reduction.
    #[default]
    Chemical,
    /// Thermal treatment: incineration at high temperatures.
    Thermal,
    /// Biological treatment: bioremediation using microorganisms.
    Biological,
    /// Stabilization/solidification: encapsulating waste in concrete or polymers.
    Stabilization,
}

impl TreatmentType {
    /// Efficiency multiplier for treatment capacity.
    /// Higher efficiency means more waste can be treated per unit capacity.
    pub fn efficiency(&self) -> f32 {
        match self {
            TreatmentType::Chemical => 1.0,
            TreatmentType::Thermal => 1.2,
            TreatmentType::Biological => 0.8,
            TreatmentType::Stabilization => 0.9,
        }
    }

    /// Cost multiplier relative to base operating cost.
    pub fn cost_multiplier(&self) -> f64 {
        match self {
            TreatmentType::Chemical => 1.0,
            TreatmentType::Thermal => 1.5,
            TreatmentType::Biological => 0.7,
            TreatmentType::Stabilization => 0.8,
        }
    }
}

// =============================================================================
// HazardousWasteState resource
// =============================================================================

/// City-wide hazardous waste management state.
///
/// Tracks generation, treatment capacity, overflow, contamination, and fines.
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct HazardousWasteState {
    /// Total hazardous waste generated per day (tons/day).
    pub total_generation: f32,
    /// Total treatment capacity available (tons/day).
    pub treatment_capacity: f32,
    /// Untreated waste overflow (tons) — resets each tick but accumulates effects.
    pub overflow: f32,
    /// Cumulative count of illegal dump events.
    pub illegal_dump_events: u32,
    /// Accumulated ground contamination level (0.0 = clean).
    pub contamination_level: f32,
    /// Accumulated federal fines in dollars.
    pub federal_fines: f64,
    /// Number of active hazardous waste treatment facilities.
    pub facility_count: u32,
    /// Daily operating cost for all facilities.
    pub daily_operating_cost: f64,
    /// Breakdown of waste by treatment type (tons processed per type).
    pub chemical_treated: f32,
    /// Tons treated via thermal methods.
    pub thermal_treated: f32,
    /// Tons treated via biological methods.
    pub biological_treated: f32,
    /// Tons treated via stabilization methods.
    pub stabilization_treated: f32,
}
