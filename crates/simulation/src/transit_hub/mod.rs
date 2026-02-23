//! Transit Hub / Multi-Modal Stations (TRAF-015).
//!
//! Multi-modal transit stations combine multiple transit types at a single
//! location, serving as transfer points with reduced transfer penalties.
//!
//! ## Hub Types
//! - **BusMetroHub**: Combined bus stop and metro station
//! - **TrainMetroHub**: Combined train and metro station
//! - **MultiModalHub**: All transit types at one location
//!
//! ## Transfer Penalties
//! - Default transfer between modes: 3 minutes
//! - Hub reduces to 1 minute between co-located modes
//!
//! ## Land Value
//! Hubs provide a 1.5x land value boost compared to individual stations.

pub mod systems;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export all public items so callers don't need to change their imports.
pub use systems::*;
pub use types::*;
