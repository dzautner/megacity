//! TRAF-014: Tram/Light Rail Transit System
//!
//! Implements a tram/light rail system that shares road space with other traffic.
//! Trams have higher capacity than buses (90 passengers) and run on fixed lines
//! between stops placed on road cells.
//!
//! ## Data model
//! - `TramStop`: a stop placed on a road cell (grid coords, passenger queue)
//! - `TramLine`: a named sequence of tram stops with active flag
//! - `TramVehicle`: a tram entity traveling along a line, picking up/dropping off
//! - `TramTransitState`: top-level resource storing all stops, lines, vehicles, and stats
//!
//! ## Costs
//! - $600/week per active line
//! - $200/week per tram depot
//! - Fare revenue: $2.50 per ride
//!
//! ## Depot requirement
//! A tram line is only active if at least one `TramDepot` service building
//! exists within coverage radius of any stop on that line.

pub mod state;
pub mod systems;
#[cfg(test)]
mod tests;

// Re-export all public items for backward compatibility.
pub use state::*;
pub use systems::*;
