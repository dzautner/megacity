//! Urban flooding simulation and depth-damage curves (FLOOD-961).
//!
//! When stormwater runoff exceeds storm drainage capacity, excess water pools on
//! the surface and spreads via a simplified shallow-water model. The `FloodGrid`
//! resource tracks per-cell flood depth (in feet) while the `FloodState` resource
//! provides aggregate statistics (total flooded cells, cumulative damage, maximum
//! depth).
//!
//! Depth-damage curves translate flood depth into a fractional damage value for
//! each zone type (Residential, Commercial, Industrial). Damage is applied to
//! buildings based on their estimated property value.
//!
//! The `update_flood_simulation` system runs every slow tick and performs:
//!   1. Checks if flooding conditions exist (storm drainage overflow > threshold)
//!   2. Initializes the FloodGrid from stormwater overflow
//!   3. Runs 5 iterations of water spreading (high elevation to low, 4-connected)
//!   4. Applies drainage rates (natural drain + enhanced drain for cells with drains)
//!   5. Calculates building damage using depth-damage curves
//!   6. Updates FloodState with aggregate statistics
//!   7. Clears FloodGrid when flooding subsides

pub mod damage_curves;
pub mod resources;
pub mod systems;

#[cfg(test)]
mod system_tests;

// Re-export all public items for backward compatibility.
pub use damage_curves::{depth_damage_fraction, interpolate_damage};
pub use resources::{FloodGrid, FloodState};
pub use systems::{update_flood_simulation, FloodSimulationPlugin};
