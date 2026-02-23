//! Cold snap detection, pipe damage, and effect systems.
//!
//! This module tracks consecutive cold days, classifies cold snap severity tiers,
//! simulates pipe bursts, and computes derived effects (heating demand, traffic
//! capacity, school closures, construction halt, homeless mortality).

mod detection;
mod pipe_damage;
mod systems;
mod types;

// Re-export all public items so external code sees the same API as before.
pub use detection::{
    cold_snap_tier, heating_demand_modifier, homeless_mortality, is_cold_day, seasonal_average_temp,
};
pub use pipe_damage::{
    calculate_pipe_bursts, estimate_water_main_miles, pipe_burst_probability,
    water_service_from_bursts,
};
pub use systems::{update_cold_snap, ColdSnapPlugin};
pub use types::{ColdSnapEvent, ColdSnapState, ColdSnapTier};
