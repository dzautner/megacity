//! Water Pressure Zones and Distribution (WATER-010).
//!
//! Higher elevation areas require pumping stations to maintain adequate water
//! pressure. Buildings above a pressure zone's effective elevation lose water
//! service or receive reduced service quality.
//!
//! - Base pressure zone serves buildings up to elevation 50.
//! - Booster pump station: extends pressure zone by +30 elevation, $200K, 1x1.
//! - Buildings above the pressure zone elevation have reduced water pressure
//!   (lower service quality).
//! - No pressure = no water service for high-elevation buildings.
//! - Multiple pump stations can chain (each adds +30 elevation to zone).

mod state;
mod systems;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public items so external callers don't break.
pub use state::WaterPressureState;
pub use systems::{update_water_pressure, WaterPressurePlugin};
pub use types::{
    classify_pressure, effective_pressure_elevation, pressure_factor, BoosterPumpStation,
    PressureCategory, BASE_PRESSURE_ELEVATION, BOOSTER_ELEVATION_GAIN, BOOSTER_PUMP_COST,
    PRESSURE_FALLOFF_RANGE,
};
