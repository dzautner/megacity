use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::weather::Season;

// =============================================================================
// Constants
// =============================================================================

/// Temperature threshold (Fahrenheit) below which crops cannot grow.
pub(crate) const GROWING_TEMP_THRESHOLD_F: f32 = 50.0;

/// Frost risk threshold: if frost risk exceeds this, the growing season is inactive.
pub(crate) const FROST_RISK_THRESHOLD: f32 = 0.10;

/// Adequate rainfall range (inches per year).
pub(crate) const RAINFALL_ADEQUATE_LOW: f32 = 20.0;
pub(crate) const RAINFALL_ADEQUATE_HIGH: f32 = 40.0;

/// Rainfall adequacy multiplier for excess rainfall (>40 in/yr).
pub(crate) const RAINFALL_EXCESS_MULTIPLIER: f32 = 0.8;

/// Rainfall adequacy multiplier for deficit rainfall (<20 in/yr).
pub(crate) const RAINFALL_DEFICIT_MULTIPLIER: f32 = 0.6;

/// Base soil quality for fertile land deposits.
pub(crate) const BASE_SOIL_QUALITY: f32 = 0.8;

/// Fertilizer bonus when irrigation infrastructure is present.
pub(crate) const IRRIGATION_FERTILIZER_BONUS: f32 = 1.15;

/// Frost damage probability in Spring (early frost).
pub(crate) const SPRING_FROST_BASE_RISK: f32 = 0.15;

/// Frost damage probability in Autumn (late frost).
pub(crate) const AUTUMN_FROST_BASE_RISK: f32 = 0.12;

/// Fraction of crop yield destroyed by a frost event.
pub(crate) const FROST_DAMAGE_FRACTION: f32 = 0.3;

/// Irrigation coverage radius (in grid cells) from an irrigation building.
pub(crate) const IRRIGATION_RADIUS: u32 = 12;

// =============================================================================
// Frost event
// =============================================================================

/// Event fired when a frost event damages crops.
#[derive(Event, Debug, Clone)]
pub struct FrostEvent {
    /// Fraction of total crop yield destroyed (0.0 to 1.0).
    pub damage_fraction: f32,
    /// Season during which the frost occurred.
    pub season: Season,
}

// =============================================================================
// Resource
// =============================================================================

/// City-wide agricultural growing season and crop yield tracking.
///
/// Updated every slow tick based on weather, rainfall, soil quality, and
/// irrigation infrastructure. Affects food production in `ResourceBalance`.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct AgricultureState {
    /// Whether the growing season is currently active.
    pub growing_season_active: bool,
    /// Current crop yield modifier (0.0 to ~1.5).
    /// `rainfall_adequacy * temperature_suitability * soil_quality * fertilizer_bonus`
    pub crop_yield_modifier: f32,
    /// Rainfall adequacy factor (0.6, 0.8, or 1.0 for deficit/excess/adequate).
    pub rainfall_adequacy: f32,
    /// Temperature suitability factor (0.0 to 1.0).
    pub temperature_suitability: f32,
    /// Average soil quality across agricultural zones (0.0 to 1.0).
    pub soil_quality: f32,
    /// Fertilizer/irrigation bonus multiplier (1.0 = none, 1.15 = irrigated).
    pub fertilizer_bonus: f32,
    /// Current frost risk (0.0 to 1.0), based on season and temperature.
    pub frost_risk: f32,
    /// Number of frost events that have occurred this year.
    pub frost_events_this_year: u32,
    /// Total crop damage from frost this year (as a fraction of production lost).
    pub frost_damage_total: f32,
    /// Whether irrigation infrastructure is present in the city.
    pub has_irrigation: bool,
    /// Number of agricultural buildings (industrial on fertile land).
    pub farm_count: u32,
    /// Accumulated annual rainfall estimate (inches), derived from daily precipitation.
    pub annual_rainfall_estimate: f32,
    /// Last game day that checked for frost events.
    pub last_frost_check_day: u32,
    /// Last game day that updated rainfall accumulation.
    pub last_rainfall_day: u32,
}

impl Default for AgricultureState {
    fn default() -> Self {
        Self {
            growing_season_active: false,
            crop_yield_modifier: 1.0,
            rainfall_adequacy: 1.0,
            temperature_suitability: 1.0,
            soil_quality: BASE_SOIL_QUALITY,
            fertilizer_bonus: 1.0,
            frost_risk: 0.0,
            frost_events_this_year: 0,
            frost_damage_total: 0.0,
            has_irrigation: false,
            farm_count: 0,
            annual_rainfall_estimate: 30.0, // Default to adequate range
            last_frost_check_day: 0,
            last_rainfall_day: 0,
        }
    }
}
