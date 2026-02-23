//! Constants for heat wave mitigation measures.

/// Mortality reduction factor when cooling centers are active (50%).
pub(crate) const COOLING_CENTER_MORTALITY_REDUCTION: f32 = 0.50;

/// Daily operating cost of cooling centers during heat waves.
pub(crate) const COOLING_CENTER_DAILY_COST: f64 = 10_000.0;

/// Temperature reduction per 20% tree coverage (Fahrenheit).
pub(crate) const GREEN_CANOPY_TEMP_REDUCTION_PER_20PCT: f32 = 5.0;

/// Temperature reduction from light-colored roofs (Fahrenheit).
pub(crate) const LIGHT_ROOF_TEMP_REDUCTION: f32 = 3.0;

/// One-time cost per building for light-colored roof upgrade.
/// Used by UI/policy layer; tested below.
#[allow(dead_code)]
pub(crate) const LIGHT_ROOF_UPGRADE_COST: f64 = 5_000.0;

/// Perceived temperature reduction from misting stations (Fahrenheit).
pub(crate) const MISTING_STATION_TEMP_REDUCTION: f32 = 10.0;

/// Daily operating cost per misting station during heat waves.
pub(crate) const MISTING_STATION_DAILY_COST: f64 = 2_000.0;

/// Daily operating cost of emergency water distribution during heat waves.
pub(crate) const EMERGENCY_WATER_DAILY_COST: f64 = 8_000.0;

/// Slow tick interval divider: update costs roughly once per game day
/// (slow tick runs every ~100 ticks; we apply daily costs each slow tick
/// scaled by the fraction of a day it represents).
pub(crate) const COST_TICKS_PER_DAY: f32 = 10.0;
