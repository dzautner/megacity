//! TRAF-012: Train/Rail Transit System
//!
//! Implements a surface-level train/rail transit system for high-capacity
//! intercity and commuter rail service. Trains run on dedicated rail lines
//! separate from roads, connecting train stations placed on the grid.
//!
//! ## Data model
//! - `TrainStation`: a station placed on a grid cell (capacity, passenger queue)
//! - `TrainLine`: a named route connecting stations in sequence
//! - `TrainInstance`: a train entity traveling along a line
//! - `TrainTransitState`: top-level resource storing all stations, lines, trains, and stats
//!
//! ## Costs
//! - $2000/week per active line + $800/week per station
//! - Fare revenue: $3 per ride
//!
//! ## Key differences from metro
//! - Surface-level rail (marks grid cells as rail track)
//! - Higher capacity (200 passengers) but lower frequency
//! - Different cost structure (weekly operating costs)
//! - Larger land value boost radius (commuter rail effect)
//!
//! The `TrainTransitState` resource is the source of truth and is persisted
//! via the `Saveable` extension map.

mod state;
mod systems;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public items so external code sees the same API.
pub use systems::TrainTransitPlugin;
pub use systems::{train_station_land_value, update_train_costs, update_train_lines};
pub use types::*;
