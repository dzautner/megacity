//! Blueprint / Template System (UX-041).
//!
//! Provides a `BlueprintLibrary` resource that stores reusable road+zone layouts.
//! Players can capture a rectangular area of the map as a blueprint, then stamp
//! copies of that blueprint at different locations.
//!
//! Blueprints store road segments and zone cells relative to an origin, making
//! them position-independent and reusable.

pub mod blueprint;
pub mod library;
pub mod plugin;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export all public items so callers don't need to change their imports.
pub use blueprint::*;
pub use library::*;
pub use plugin::*;
pub use types::*;
