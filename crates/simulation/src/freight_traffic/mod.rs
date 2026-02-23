//! TRAF-004: Freight/Goods Traffic on Road Network.
//!
//! Industrial buildings generate outbound freight (trucks) that deliver goods
//! to commercial buildings. Trucks are heavier than cars and contribute more
//! to congestion, road wear, and noise.
//!
//! Key behaviors:
//! - Industrial buildings generate outbound freight demand proportional to occupants
//! - Commercial buildings generate inbound freight demand proportional to occupants
//! - Freight vehicles (trucks) are spawned, routed via A*, and despawned on arrival
//! - Trucks have a vehicle equivalence factor of 2.5 (each truck = 2.5 cars for congestion)
//! - Trucks add to traffic density on the road grid via `TrafficGrid`
//! - Trucks increase road wear in `RoadConditionGrid`
//! - Heavy traffic ban per district blocks truck routing through those districts
//! - Freight satisfaction affects commercial/industrial productivity

mod constants;
mod plugin;
mod systems;
mod types;

#[cfg(test)]
mod tests;

pub use plugin::FreightTrafficPlugin;
pub use systems::{
    compute_freight_demand, generate_freight_trips, move_freight_trucks,
    update_freight_satisfaction,
};
pub use types::{FreightTrafficState, FreightTruck};
