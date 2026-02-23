//! Reservoir water-level tracking, warning tiers, and update systems.

pub mod systems;
pub mod types;

mod tests_simulation;
mod tests_unit;

pub use systems::{update_reservoir_levels, ReservoirPlugin};
pub use types::{
    warning_tier_from_fill, ReservoirState, ReservoirWarningEvent, ReservoirWarningTier,
};
