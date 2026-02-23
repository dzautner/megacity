//! Transport mode types, components, and statistics resources.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::constants::*;

// =============================================================================
// TransportMode enum
// =============================================================================

/// The available transport modes for citizen trips.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Encode, Decode,
)]
pub enum TransportMode {
    /// Walking: always available, slow, best for short distances.
    Walk,
    /// Bicycle: requires bike infrastructure (Path roads), medium speed.
    Bike,
    /// Car/Drive: requires vehicle-accessible road, fastest on uncongested roads.
    #[default]
    Drive,
    /// Public transit: requires transit stops nearby, reliable for medium/long trips.
    Transit,
}

impl TransportMode {
    /// Speed multiplier relative to the base citizen movement speed.
    pub fn speed_multiplier(self) -> f32 {
        match self {
            TransportMode::Walk => WALK_SPEED_MULTIPLIER,
            TransportMode::Bike => BIKE_SPEED_MULTIPLIER,
            TransportMode::Drive => DRIVE_SPEED_MULTIPLIER,
            TransportMode::Transit => TRANSIT_SPEED_MULTIPLIER,
        }
    }

    /// Comfort factor for perceived-time calculation.
    pub fn comfort_factor(self) -> f32 {
        match self {
            TransportMode::Walk => WALK_COMFORT,
            TransportMode::Bike => BIKE_COMFORT,
            TransportMode::Drive => DRIVE_COMFORT,
            TransportMode::Transit => TRANSIT_COMFORT,
        }
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            TransportMode::Walk => "Walking",
            TransportMode::Bike => "Bicycle",
            TransportMode::Drive => "Car",
            TransportMode::Transit => "Transit",
        }
    }
}

// =============================================================================
// Components
// =============================================================================

/// Component attached to each citizen indicating their current trip's transport mode.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ChosenTransportMode(pub TransportMode);

// =============================================================================
// Mode Share Statistics
// =============================================================================

/// City-wide mode share statistics, tracking the percentage of active trips
/// by each transport mode.
#[derive(Resource, Debug, Clone, Encode, Decode)]
pub struct ModeShareStats {
    /// Number of citizens currently using each mode.
    pub walk_count: u32,
    pub bike_count: u32,
    pub drive_count: u32,
    pub transit_count: u32,
    /// Percentage (0.0-100.0) of trips by each mode.
    pub walk_pct: f32,
    pub bike_pct: f32,
    pub drive_pct: f32,
    pub transit_pct: f32,
}

impl Default for ModeShareStats {
    fn default() -> Self {
        Self {
            walk_count: 0,
            bike_count: 0,
            drive_count: 0,
            transit_count: 0,
            walk_pct: 0.0,
            bike_pct: 0.0,
            drive_pct: 100.0,
            transit_pct: 0.0,
        }
    }
}

impl ModeShareStats {
    /// Total number of active trips.
    pub fn total(&self) -> u32 {
        self.walk_count + self.bike_count + self.drive_count + self.transit_count
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for ModeShareStats {
    const SAVE_KEY: &'static str = "mode_share_stats";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no trips recorded
        if self.total() == 0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Infrastructure cache
// =============================================================================

/// Cached positions of transit stops and bike-friendly roads, rebuilt when
/// services change. Avoids per-citizen iteration over all service buildings.
#[derive(Resource, Default)]
pub struct ModeInfrastructureCache {
    /// Positions of transit stops (bus depot, train station, subway, tram, ferry).
    pub transit_stops: Vec<(usize, usize)>,
    /// Positions of bike-friendly road cells (Path type).
    pub bike_paths: Vec<(usize, usize)>,
    /// Positions of vehicle-accessible road cells (any non-Path road).
    pub vehicle_roads: Vec<(usize, usize)>,
}
