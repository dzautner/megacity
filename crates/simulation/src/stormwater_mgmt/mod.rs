//! Stormwater management and flooding integration (SVC-022).
//!
//! Ties together the stormwater, storm drainage, and flood simulation systems
//! with additional features:
//!
//! - **Flood risk overlay**: per-cell risk score combining elevation,
//!   imperviousness fraction, and drainage coverage.
//! - **Flood road damage**: reduces `RoadConditionGrid` condition for
//!   flooded road cells each slow tick.
//! - **Citizen displacement**: citizens in buildings on flooded cells
//!   receive happiness/health penalties.
//! - **Green infrastructure**: trees and parks reduce effective stormwater
//!   runoff in their vicinity.

mod flood_risk;
mod green_infra;
mod road_damage;
mod state;
mod systems;

#[cfg(test)]
mod tests;

pub use flood_risk::FloodRiskGrid;
pub use state::{StormwaterMgmtState, StormwaterMgmtPlugin};
