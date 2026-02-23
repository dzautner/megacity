//! Groundwater depletion and sustainability mechanics (WATER-008).
//!
//! Tracks extraction vs recharge rates across the groundwater grid, computes
//! sustainability metrics, applies well yield reduction when groundwater is low,
//! detects critical depletion, and models land subsidence for cells that remain
//! depleted for extended periods.
//!
//! This module reads from the existing `GroundwaterGrid` resource and produces
//! a `GroundwaterDepletionState` resource that other systems (UI overlays,
//! well pump output, notifications) can consume.

mod systems;
#[cfg(test)]
mod tests;
mod types;

pub use systems::*;
pub use types::*;
