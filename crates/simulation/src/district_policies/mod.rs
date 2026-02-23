//! Per-District Zone Policies (ZONE-015).
//!
//! Allows policies to be applied per-district instead of city-wide only.
//! Each district can have different tax rate overrides, building height limits,
//! heavy industry bans, service budget multipliers, and more.
//!
//! Supported per-district policies:
//! - **Tax rate overrides**: residential, commercial, industrial, office tax rates
//! - **High-rise ban**: caps building level at 2 in the district
//! - **Heavy industry ban**: prevents industrial zoning in the district
//! - **Small business incentive**: boosts commercial demand in the district
//! - **Noise ordinance**: reduces happiness penalty from noise in the district
//! - **Green space mandate**: boosts park effectiveness in the district
//! - **Service budget multiplier**: scales service effectiveness in the district
//!
//! The system runs on the slow tick timer to compute per-district effective
//! policy values that other systems can query via `DistrictPolicyLookup`.

pub mod lookup;
pub mod systems;
pub mod types;

mod tests;
mod tests_lookup;

// Re-export all public items so callers can use `crate::district_policies::*`.
pub use lookup::*;
pub use systems::*;
pub use types::*;
