//! Parking constants, ratios, and per-zone cost tables.

use crate::grid::ZoneType;

// =============================================================================
// Constants
// =============================================================================

/// Cost per required parking space for low-density zones (surface lots).
pub const PARKING_COST_LOW_DENSITY: f64 = 5_000.0;

/// Cost per required parking space for medium-density zones.
pub const PARKING_COST_MEDIUM_DENSITY: f64 = 10_000.0;

/// Cost per required parking space for high-density zones (structured parking).
pub const PARKING_COST_HIGH_DENSITY: f64 = 20_000.0;

/// Cost per required parking space for industrial zones.
pub const PARKING_COST_INDUSTRIAL: f64 = 5_000.0;

/// Cost per required parking space for office zones.
pub const PARKING_COST_OFFICE: f64 = 15_000.0;

/// Parking maximum cap as a fraction of the minimum ratio (e.g., 0.5 means
/// maximum is half the minimum requirement).
pub const PARKING_MAXIMUM_FRACTION: f32 = 0.5;

// =============================================================================
// Parking ratios per zone type
// =============================================================================

/// Returns the parking spaces required per unit/1000sqft for a given zone type.
/// - Residential Low: 1.0 per unit
/// - Residential Medium: 1.5 per unit
/// - Residential High: 2.0 per unit
/// - Commercial Low: 3.3 per 1000 sqft (1 per 300 sqft)
/// - Commercial High: 3.3 per 1000 sqft
/// - Industrial: 2.0 per 1000 sqft (1 per 500 sqft)
/// - Office: 2.5 per 1000 sqft (1 per 400 sqft)
/// - MixedUse: 2.5 (weighted average of residential and commercial)
pub fn parking_ratio(zone: ZoneType) -> f32 {
    match zone {
        ZoneType::ResidentialLow => 1.0,
        ZoneType::ResidentialMedium => 1.5,
        ZoneType::ResidentialHigh => 2.0,
        ZoneType::CommercialLow => 3.3,
        ZoneType::CommercialHigh => 3.3,
        ZoneType::Industrial => 2.0,
        ZoneType::Office => 2.5,
        ZoneType::MixedUse => 2.5,
        ZoneType::None => 0.0,
    }
}

/// Returns the cost per required parking space for a given zone type.
pub fn parking_cost_per_space(zone: ZoneType) -> f64 {
    match zone {
        ZoneType::ResidentialLow => PARKING_COST_LOW_DENSITY,
        ZoneType::ResidentialMedium => PARKING_COST_MEDIUM_DENSITY,
        ZoneType::ResidentialHigh => PARKING_COST_HIGH_DENSITY,
        ZoneType::CommercialLow => PARKING_COST_LOW_DENSITY,
        ZoneType::CommercialHigh => PARKING_COST_HIGH_DENSITY,
        ZoneType::Industrial => PARKING_COST_INDUSTRIAL,
        ZoneType::Office => PARKING_COST_OFFICE,
        ZoneType::MixedUse => PARKING_COST_HIGH_DENSITY,
        ZoneType::None => 0.0,
    }
}
