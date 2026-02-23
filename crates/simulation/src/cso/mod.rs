//! Combined Sewer Overflow (CSO) events (WATER-009).
//!
//! In cities with combined sewer systems, a single pipe carries both sewage and
//! stormwater. During heavy rain, the combined flow can exceed treatment plant
//! capacity, forcing untreated discharge (CSO) into waterways.
//!
//! This module tracks the sewer system type (combined vs. separated), calculates
//! combined flow from population sewage and stormwater runoff, detects overflow
//! conditions, and emits `CsoEvent` Bevy events when CSO occurs. Separated sewers
//! route stormwater to storm drains independently, preventing CSO entirely.

mod systems;
mod types;

#[cfg(test)]
mod overflow_tests;

pub use systems::{update_sewer_overflow, CsoPlugin};
pub use types::{
    CsoEvent, SewerSystemState, SewerType, BASE_COMBINED_CAPACITY_PER_CELL,
    GALLONS_PER_CAPITA_PER_DAY, POLLUTION_PER_GALLON_CSO, SEPARATION_COST_PER_CELL,
    STORMWATER_TO_SEWER_FACTOR,
};
