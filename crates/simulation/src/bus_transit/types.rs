//! Data types and constants for the bus transit system.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

// =============================================================================
// Constants
// =============================================================================

/// Maximum passengers per bus.
pub const BUS_CAPACITY: u32 = 30;

/// Monthly cost per bus route.
pub const ROUTE_MONTHLY_COST: f64 = 400.0;

/// Monthly cost per active bus.
pub const BUS_MONTHLY_COST: f64 = 100.0;

/// Fare per ride (revenue).
pub const FARE_PER_RIDE: f64 = 2.0;

/// Number of buses spawned per route.
pub const BUSES_PER_ROUTE: u32 = 2;

/// Maximum number of stops per route.
pub const MAX_STOPS_PER_ROUTE: usize = 20;

/// Maximum walking distance (in grid cells) to consider a bus stop reachable.
pub const MAX_WALK_DISTANCE: u32 = 10;

/// Average wait time at a stop in ticks (simulated).
pub const AVERAGE_WAIT_TICKS: u32 = 20;

/// Bus speed in grid cells per tick (slower than cars, affected by traffic).
pub const BUS_SPEED_CELLS_PER_TICK: f32 = 0.3;

// =============================================================================
// Data structures
// =============================================================================

/// Unique identifier for a bus stop.
pub type BusStopId = u32;

/// Unique identifier for a bus route.
pub type BusRouteId = u32;

/// A bus stop placed on a road cell.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BusStop {
    pub id: BusStopId,
    pub grid_x: usize,
    pub grid_y: usize,
    /// Number of citizens currently waiting at this stop.
    pub waiting: u32,
}

/// A bus route connecting an ordered sequence of bus stops.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BusRoute {
    pub id: BusRouteId,
    /// Human-readable name for the route.
    pub name: String,
    /// Ordered list of bus stop IDs along this route.
    pub stop_ids: Vec<BusStopId>,
    /// Whether this route is currently active (has a depot nearby).
    pub active: bool,
    /// Total lifetime ridership on this route.
    pub total_ridership: u64,
    /// Ridership in the current month.
    pub monthly_ridership: u32,
}

/// A bus entity traveling along a route.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct BusInstance {
    pub route_id: BusRouteId,
    /// Index into the route's stop_ids for the next stop.
    pub next_stop_index: usize,
    /// Current grid position (fractional for smooth movement).
    pub grid_x: f32,
    pub grid_y: f32,
    /// Number of passengers currently on board.
    pub passengers: u32,
    /// Ticks remaining at current stop (for dwell time).
    pub dwell_ticks: u32,
    /// Whether the bus is currently dwelling at a stop.
    pub at_stop: bool,
}

// =============================================================================
// Resource: BusTransitState
// =============================================================================

/// Top-level resource for the bus transit system.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct BusTransitState {
    /// All bus stops, keyed by ID.
    pub stops: Vec<BusStop>,
    /// All bus routes.
    pub routes: Vec<BusRoute>,
    /// All active bus instances.
    pub buses: Vec<BusInstance>,
    /// Next ID for bus stops.
    pub next_stop_id: BusStopId,
    /// Next ID for bus routes.
    pub next_route_id: BusRouteId,
    /// Total fare revenue this month.
    pub monthly_fare_revenue: f64,
    /// Total operating cost this month.
    pub monthly_operating_cost: f64,
    /// Last day costs were applied.
    pub last_cost_day: u32,
}
