//! Recycling program tiers and economics (WASTE-004).
//!
//! Implements tiered recycling programs from "No program" (5% baseline diversion)
//! to "Zero waste goal" (60% diversion). Each tier specifies diversion rates,
//! participation rates, per-household costs, and contamination rates.
//!
//! Recycling economics tracks commodity prices per material type with market
//! cycles (~5 game-year period, 0.3x bust to 1.5x boom) and computes net
//! value per ton after subtracting collection and processing costs.

mod economics;
mod state;
mod systems;
mod tests;
mod tiers;

pub use economics::RecyclingEconomics;
pub use state::RecyclingState;
pub use systems::{update_recycling_economics, RecyclingPlugin};
pub use tiers::RecyclingTier;
