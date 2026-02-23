//! Constants for CO2 emission rates, climate thresholds, and related parameters.

/// CO2 emission rate for coal power plants (tons per MWh).
pub const CO2_COAL: f32 = 1.0;

/// CO2 emission rate for gas power plants (tons per MWh).
pub const CO2_GAS: f32 = 0.4;

/// CO2 emission rate for oil/generic power plants (tons per MWh).
pub const CO2_OIL: f32 = 0.8;

/// CO2 emission rate for biomass/renewable sources (tons per MWh, carbon neutral).
pub const CO2_BIOMASS: f32 = 0.0;

/// Base MWh production per power plant per assessment period.
/// Each utility source is assumed to generate this many MWh per yearly assessment.
pub(crate) const BASE_MWH_PER_PLANT: f32 = 1000.0;

/// Base CO2 from industrial buildings per assessment (tons per building).
pub(crate) const INDUSTRIAL_CO2_PER_BUILDING: f32 = 50.0;

/// Climate threshold: cumulative tons for +1F temperature increase.
pub(crate) const THRESHOLD_1F: f64 = 1_000_000.0;

/// Climate threshold: cumulative tons for +2F temperature increase.
pub(crate) const THRESHOLD_2F: f64 = 5_000_000.0;

/// Climate threshold: cumulative tons for +3F temperature increase.
pub(crate) const THRESHOLD_3F: f64 = 20_000_000.0;

/// Disaster frequency increase per 1F of warming (10% per degree F).
pub(crate) const DISASTER_FREQUENCY_INCREASE_PER_F: f32 = 0.10;

/// Number of game days per year (used for yearly assessments).
pub(crate) const DAYS_PER_YEAR: u32 = 360;

/// Elevation percentile threshold for sea level rise flooding.
/// At +3F, water-adjacent cells with elevation below this percentile flood permanently.
pub(crate) const SEA_LEVEL_RISE_ELEVATION_PERCENTILE: f32 = 0.15;
