//! Parking minimum/maximum system (ZONE-011).
//!
//! Implements parking requirements as a zoning control. Parking minimums
//! consume land and increase construction costs, while parking maximums
//! encourage transit usage.
//!
//! ## Per-zone parking ratios
//! - Residential: 1-2 spaces per unit (low density=1, medium=1.5, high=2)
//! - Commercial: 1 space per 300 sqft (~3.3 per 1000 sqft)
//! - Industrial: 1 space per 500 sqft (~2.0 per 1000 sqft)
//! - Office: 1 space per 400 sqft (~2.5 per 1000 sqft)
//! - MixedUse: weighted average of residential + commercial ratios
//!
//! ## Cost impact
//! Each required parking space adds $5K-$20K to effective building cost
//! depending on zone type (surface lots are cheaper, structured parking
//! in dense areas is expensive).
//!
//! ## Policies
//! - **Eliminate parking minimums**: removes minimum requirements, reduces
//!   construction cost, increases transit dependency.
//! - **Parking maximum**: caps parking to a fraction of the minimum ratio,
//!   encouraging transit use and reducing land consumed by parking.

pub mod constants;
pub mod state;

#[cfg(test)]
mod tests_constants;
#[cfg(test)]
mod tests_state;

// Re-export all public items so callers don't need to change their imports.
pub use constants::*;
pub use state::*;
