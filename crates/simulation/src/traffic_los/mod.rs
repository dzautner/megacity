//! Traffic Level of Service (LOS) grading system.
//!
//! Computes LOS grades A through F for road cells and road segments based on
//! traffic density relative to road capacity. LOS A represents free flow,
//! while LOS F represents gridlock conditions.
//!
//! The grades follow the Highway Capacity Manual (HCM) convention:
//! - A: Free flow (V/C < 0.35)
//! - B: Stable flow (V/C 0.35-0.55)
//! - C: Stable flow, some restriction (V/C 0.55-0.77)
//! - D: Approaching unstable (V/C 0.77-0.93)
//! - E: Unstable flow (V/C 0.93-1.00)
//! - F: Forced flow / breakdown (V/C >= 1.00)

pub mod grades;
pub mod grid;
pub mod plugin;
pub mod segment_los;
pub mod systems;

#[cfg(test)]
mod tests;

// Re-export public items so callers don't need to change their imports.
pub use grades::LosGrade;
pub use grid::TrafficLosGrid;
pub use plugin::TrafficLosPlugin;
pub use segment_los::{LosDistribution, TrafficLosState};
pub use systems::{update_segment_los, update_traffic_los};
