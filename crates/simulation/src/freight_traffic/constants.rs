//! Constants for the freight traffic system.

/// Vehicle equivalence factor: each truck contributes this many "car equivalents"
/// to traffic density, road wear, and congestion calculations.
pub(crate) const TRUCK_EQUIVALENCE_FACTOR: f32 = 2.5;

/// Extra road degradation per truck waypoint visit (on top of normal traffic wear).
pub(crate) const TRUCK_WEAR_PER_VISIT: u8 = 1;

/// How often to generate new freight trips (every N ticks). At 10Hz, 20 ticks = 2s.
pub(crate) const FREIGHT_GENERATION_INTERVAL: u64 = 20;

/// How often to move freight trucks (every N ticks). Same cadence as traffic updates.
pub(crate) const FREIGHT_MOVE_INTERVAL: u64 = 5;

/// Maximum number of concurrent freight trucks in the city.
pub(crate) const MAX_FREIGHT_TRUCKS: usize = 200;

/// Maximum freight trips generated per cycle.
pub(crate) const MAX_TRIPS_PER_CYCLE: usize = 10;

/// Freight demand per occupant in an industrial building (outbound goods).
pub(crate) const INDUSTRIAL_FREIGHT_RATE: f32 = 0.02;

/// Freight demand per occupant in a commercial building (inbound goods).
pub(crate) const COMMERCIAL_FREIGHT_RATE: f32 = 0.015;

/// Maximum search distance for matching freight origin to destination (grid cells).
pub(crate) const MAX_FREIGHT_DISTANCE: i32 = 60;

/// Truck movement speed in grid cells per move tick.
pub(crate) const TRUCK_SPEED: usize = 2;
