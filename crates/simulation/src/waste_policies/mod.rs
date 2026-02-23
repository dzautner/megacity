//! Waste reduction policies (WASTE-008).
//!
//! Implements waste management policies that push the waste hierarchy
//! (reduce > reuse > recycle > energy recovery > landfill). Each policy is
//! individually toggleable and has specific costs, benefits, and impacts on
//! waste generation, recycling rates, composting diversion, and citizen happiness.
//!
//! Policies:
//! - **Plastic bag ban**: -5% overall waste generation, minor happiness impact
//! - **Deposit/return program**: +10% recycling rate, $500K infrastructure cost
//! - **Composting mandate**: +15% diversion to composting, happiness -2, $1M enforcement
//! - **WTE mandate**: waste diverted from landfill to WTE (incinerator) when available
//!
//! The system reads `WasteSystem.period_generated_tons` and modifies the
//! effective waste stream via a `WastePolicyEffects` resource that other
//! systems can query.

pub mod constants;
pub mod state;
pub mod systems;

#[cfg(test)]
mod tests_effects;
#[cfg(test)]
mod tests_state;

// Re-export all public items so callers don't need to change their imports.
pub use constants::*;
pub use state::*;
pub use systems::*;
