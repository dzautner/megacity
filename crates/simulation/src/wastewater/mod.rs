//! Wastewater and sewage collection system (WATER-007).
//!
//! Buildings generate sewage at 80% of their water consumption. Sewage treatment
//! plants provide treatment capacity. If total sewage exceeds capacity, overflow
//! discharges as raw sewage into nearby water bodies (increasing water pollution).
//! Uncollected sewage near residential areas applies a health penalty to citizens.

pub mod types;

mod systems;
mod tests;

// Re-export public items so external callers don't break.
pub use systems::{update_wastewater, wastewater_health_penalty, WastewaterPlugin};
pub use types::WastewaterState;
