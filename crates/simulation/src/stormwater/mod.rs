//! Stormwater management simulation.
//!
//! Tracks rainfall runoff accumulation and drainage across the city grid,
//! taking into account surface imperviousness, soil infiltration, and
//! elevation-based flow.

mod calculations;
mod systems;
#[cfg(test)]
mod tests;
mod types;

pub use calculations::{imperviousness, infiltration, runoff};
pub use systems::update_stormwater;
pub use types::{StormwaterGrid, StormwaterPlugin};
