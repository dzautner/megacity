//! Data types, constants, and components for storm drainage infrastructure.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Constants
// =============================================================================

/// Runoff capacity removed per storm drain, in inches/hr.
pub(crate) const DRAIN_CAPACITY_PER_DRAIN: f32 = 0.5;

/// Maximum gallons a single retention pond can store.
pub(crate) const RETENTION_POND_CAPACITY: f32 = 500_000.0;

/// Fraction of stored retention water released per slow tick.
/// Retention ponds slowly release stored water between storms.
pub(crate) const RETENTION_RELEASE_RATE: f32 = 0.05;

/// Conversion factor from stormwater grid runoff units to inches/hr equivalent.
/// The stormwater grid stores runoff as `rainfall_intensity * imperviousness * CELL_AREA`.
/// We normalise to inches/hr for comparison with drain capacity.
pub(crate) const RUNOFF_TO_INCHES_PER_HR: f32 = 0.01;

/// Conversion factor from stormwater grid runoff units to gallons for retention storage.
pub(crate) const RUNOFF_TO_GALLONS: f32 = 100.0;

/// Fraction of a rain garden's 4 cardinal neighbors' runoff that it absorbs.
pub(crate) const RAIN_GARDEN_NEIGHBOR_ABSORB: f32 = 0.50;

/// Runoff threshold (inches/hr equivalent) above which a cell is considered flooding
/// when drainage capacity is exceeded.
pub(crate) const FLOOD_THRESHOLD: f32 = 0.1;

// =============================================================================
// Infrastructure type enum
// =============================================================================

/// The kind of storm drainage infrastructure placed in the city.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StormDrainageType {
    /// Storm drain placed along roads. Removes 0.5 in/hr capacity.
    StormDrain,
    /// 4x4 retention pond. Stores up to 500,000 gallons, slowly releases.
    RetentionPond,
    /// 1x1 rain garden. Absorbs 100% of local cell runoff + 50% from 4 neighbors.
    RainGarden,
}

// =============================================================================
// Storm drainage infrastructure component
// =============================================================================

/// Component attached to entities representing storm drainage infrastructure.
/// Used to query all drains, ponds, and gardens in the ECS world.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct StormDrainageInfrastructure {
    /// What kind of drainage infrastructure this is.
    pub drainage_type: StormDrainageType,
    /// Grid X position of this infrastructure.
    pub grid_x: usize,
    /// Grid Y position of this infrastructure.
    pub grid_y: usize,
}

// =============================================================================
// Storm drainage state resource
// =============================================================================

/// City-wide storm drainage state, tracking infrastructure counts, capacity, and overflow.
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct StormDrainageState {
    /// Total drain capacity in inches/hr removed by all storm drains.
    pub total_drain_capacity: f32,
    /// Total retention pond storage capacity in gallons.
    pub total_retention_capacity: f32,
    /// Gallons of stormwater currently stored in retention ponds.
    pub current_retention_stored: f32,
    /// Number of storm drains placed in the city.
    pub drain_count: u32,
    /// Number of retention ponds placed in the city.
    pub retention_pond_count: u32,
    /// Number of rain gardens placed in the city.
    pub rain_garden_count: u32,
    /// Number of cells where runoff exceeds drainage capacity (flooding).
    pub overflow_cells: u32,
    /// Fraction of road cells that have at least one drain (0.0..=1.0).
    pub drainage_coverage: f32,
}
