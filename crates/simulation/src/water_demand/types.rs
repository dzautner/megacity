use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Per-building water demand rates (gallons per day)
// =============================================================================

/// Residential: 150 gallons per capita per day (GPCD).
pub(crate) const RESIDENTIAL_GPCD: f32 = 150.0;

/// Commercial: 100 gallons per building occupant per day (GPB).
pub(crate) const COMMERCIAL_GPB: f32 = 100.0;

/// Industrial: 500 gallons per building occupant per day (GPB).
pub(crate) const INDUSTRIAL_GPB: f32 = 500.0;

/// Hospital: flat 300 gallons per day base.
pub(crate) const HOSPITAL_GPD: f32 = 300.0;

/// School: 25 gallons per student per day.
pub(crate) const SCHOOL_PER_STUDENT_GPD: f32 = 25.0;

/// Park: 500 gallons per cell per day (irrigation).
pub(crate) const PARK_PER_CELL_GPD: f32 = 500.0;

// =============================================================================
// Components and resources
// =============================================================================

/// Component attached to each building entity tracking its freshwater demand.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WaterDemand {
    /// Current water demand in gallons per day, including seasonal modifier.
    pub demand_gpd: f32,
    /// Whether this building is currently receiving water service.
    pub has_water_service: bool,
}

impl Default for WaterDemand {
    fn default() -> Self {
        Self {
            demand_gpd: 0.0,
            has_water_service: false,
        }
    }
}

/// City-wide water supply and demand tracking resource.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct WaterSupply {
    /// Total city-wide water demand in gallons per day.
    pub total_demand_gpd: f32,
    /// Total city-wide water supply capacity in gallons per day.
    /// Derived from water utility infrastructure (WaterTower, PumpingStation, etc.).
    pub total_supply_gpd: f32,
    /// Number of buildings currently served (have water coverage).
    pub buildings_served: u32,
    /// Number of buildings without water service.
    pub buildings_unserved: u32,
    /// Ratio of supply to demand (>1.0 means surplus).
    pub supply_ratio: f32,
}
