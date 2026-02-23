use bevy::prelude::*;

// ---------------------------------------------------------------------------
// City Attractiveness
// ---------------------------------------------------------------------------

/// Breakdown of city attractiveness factors, each scored 0.0-1.0.
#[derive(Resource, Debug, Clone)]
pub struct CityAttractiveness {
    pub overall_score: f32,
    pub employment_factor: f32,
    pub happiness_factor: f32,
    pub services_factor: f32,
    pub housing_factor: f32,
    pub tax_factor: f32,
}

impl Default for CityAttractiveness {
    fn default() -> Self {
        Self {
            overall_score: 50.0,
            employment_factor: 0.5,
            happiness_factor: 0.5,
            services_factor: 0.5,
            housing_factor: 0.5,
            tax_factor: 0.5,
        }
    }
}

// ---------------------------------------------------------------------------
// Immigration Statistics
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, Default)]
pub struct ImmigrationStats {
    pub immigrants_this_month: u32,
    pub emigrants_this_month: u32,
    pub net_migration: i32,
    /// Tick of last monthly reset
    pub(crate) last_reset_tick: u64,
}

// ---------------------------------------------------------------------------
// Weights for the attractiveness formula
// ---------------------------------------------------------------------------

pub(crate) const WEIGHT_EMPLOYMENT: f32 = 25.0;
pub(crate) const WEIGHT_HAPPINESS: f32 = 25.0;
pub(crate) const WEIGHT_SERVICES: f32 = 20.0;
pub(crate) const WEIGHT_HOUSING: f32 = 15.0;
pub(crate) const WEIGHT_TAX: f32 = 15.0;

/// Interval in ticks between attractiveness recomputation.
pub(crate) const ATTRACTIVENESS_INTERVAL: u64 = 50;
/// Interval in ticks between immigration wave checks.
pub(crate) const IMMIGRATION_INTERVAL: u64 = 100;
/// Monthly stats reset interval (roughly 1000 ticks ~ 100 seconds).
pub(crate) const MONTHLY_RESET_INTERVAL: u64 = 1000;
