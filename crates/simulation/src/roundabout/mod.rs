//! Roundabout builder tool (TRAF-011).
//!
//! Provides a `RoundaboutRegistry` resource that tracks all roundabouts in the
//! city, a builder function to create circular one-way roads using Bezier curves,
//! and systems for yield-on-entry traffic rules and throughput tracking.

pub mod builder;
pub mod save;
pub mod systems;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export all public items so callers don't need to change their imports.
pub use builder::*;
pub use systems::*;
pub use types::*;
