//! Heat wave mitigation measures (WEATHER-013).
//!
//! Provides several mitigation options that cities can deploy to reduce the
//! impact of heat waves on population, infrastructure, and services:
//!
//! - **Cooling centers**: Public buildings open as shelters, reducing heat
//!   mortality by 50%. Cost: $10,000/day during heat waves.
//! - **Green canopy**: Tree coverage provides local temperature reduction of
//!   5F per 20% tree coverage in a radius. Passive; derived from tree grid.
//! - **Light-colored roofs**: Building upgrade that reduces roof temperature
//!   by 3F. Cost: $5,000 per building (one-time upgrade).
//! - **Misting stations**: Placeable infrastructure that reduces perceived
//!   temperature by 10F in public spaces. Cost: $2,000/day during heat waves.
//! - **Emergency water distribution**: Policy toggle that prevents dehydration
//!   deaths during heat waves. Cost: $8,000/day during heat waves.
//!
//! Each mitigation has a cost and activation condition (only during heat waves).
//! The `HeatMitigationState` resource tracks which measures are active and
//! computes the aggregate effects.

mod calculations;
mod constants;
mod state;
mod systems;

#[cfg(test)]
mod tests;

// Re-export all public items for backwards compatibility.
pub use calculations::*;
pub use state::*;
pub use systems::*;
