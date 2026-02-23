//! Types, constants, and saveable implementations for transit hubs.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::services::ServiceType;

// =============================================================================
// Constants
// =============================================================================

/// Default transfer penalty between transit modes (in minutes).
pub const DEFAULT_TRANSFER_PENALTY_MINUTES: f32 = 3.0;

/// Reduced transfer penalty at hub locations (in minutes).
pub const HUB_TRANSFER_PENALTY_MINUTES: f32 = 1.0;

/// Land value boost multiplier for hubs relative to individual stations.
/// Individual transit stations give a base boost; hubs multiply it by this factor.
pub const HUB_LAND_VALUE_MULTIPLIER: f32 = 1.5;

/// Base land value boost for an individual transit station (in raw land value units).
pub const TRANSIT_STATION_BASE_BOOST: i32 = 10;

/// Radius (in cells) within which a hub boosts land value.
pub const HUB_LAND_VALUE_RADIUS: i32 = 8;

/// Radius (in cells) within which co-located transit stops form a hub.
pub const HUB_DETECTION_RADIUS: i32 = 2;

// =============================================================================
// Transit Mode
// =============================================================================

/// Individual transit modes that can be combined at a hub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum TransitMode {
    Bus,
    Metro,
    Train,
    Tram,
    Ferry,
}

impl TransitMode {
    /// Convert a service type to a transit mode, if applicable.
    pub fn from_service_type(st: ServiceType) -> Option<Self> {
        match st {
            ServiceType::BusDepot => Some(TransitMode::Bus),
            ServiceType::SubwayStation => Some(TransitMode::Metro),
            ServiceType::TrainStation => Some(TransitMode::Train),
            ServiceType::TramDepot => Some(TransitMode::Tram),
            ServiceType::FerryPier => Some(TransitMode::Ferry),
            _ => None,
        }
    }
}

// =============================================================================
// Hub Type
// =============================================================================

/// Type of multi-modal transit hub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum TransitHubType {
    /// Combined bus stop and metro station.
    BusMetroHub,
    /// Combined train and metro station.
    TrainMetroHub,
    /// All transit types at one location.
    MultiModalHub,
}

impl TransitHubType {
    /// Returns the set of transit modes supported by this hub type.
    pub fn supported_modes(&self) -> Vec<TransitMode> {
        match self {
            TransitHubType::BusMetroHub => vec![TransitMode::Bus, TransitMode::Metro],
            TransitHubType::TrainMetroHub => vec![TransitMode::Train, TransitMode::Metro],
            TransitHubType::MultiModalHub => vec![
                TransitMode::Bus,
                TransitMode::Metro,
                TransitMode::Train,
                TransitMode::Tram,
                TransitMode::Ferry,
            ],
        }
    }

    /// Determine the hub type from a set of co-located transit modes.
    /// Returns `None` if fewer than 2 modes are present.
    pub fn from_modes(modes: &[TransitMode]) -> Option<Self> {
        if modes.len() < 2 {
            return None;
        }

        let has_bus = modes.contains(&TransitMode::Bus);
        let has_metro = modes.contains(&TransitMode::Metro);
        let has_train = modes.contains(&TransitMode::Train);

        // If 3+ modes, it's a multi-modal hub
        if modes.len() >= 3 {
            return Some(TransitHubType::MultiModalHub);
        }

        // 2 modes: check specific combinations
        if has_bus && has_metro {
            Some(TransitHubType::BusMetroHub)
        } else if has_train && has_metro {
            Some(TransitHubType::TrainMetroHub)
        } else {
            // Any other combination of 2 modes is still a valid hub
            // (e.g., Bus+Train). Classify as MultiModalHub for simplicity.
            Some(TransitHubType::MultiModalHub)
        }
    }

    /// The transfer penalty reduction factor for this hub type.
    /// Penalty = DEFAULT * (1 - reduction). For standard hubs the penalty
    /// drops from 3min to 1min, so the reduction is ~0.667.
    pub fn transfer_penalty_reduction(&self) -> f32 {
        1.0 - (HUB_TRANSFER_PENALTY_MINUTES / DEFAULT_TRANSFER_PENALTY_MINUTES)
    }
}

// =============================================================================
// Component: TransitHub
// =============================================================================

/// ECS component marking an entity as a transit hub.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct TransitHub {
    pub hub_type: TransitHubType,
    pub supported_modes: Vec<TransitMode>,
    /// Reduction applied to transfer penalty (0.0 = no reduction, 1.0 = free transfer).
    pub transfer_penalty_reduction: f32,
    /// Grid coordinates of the hub center.
    pub grid_x: usize,
    pub grid_y: usize,
}

impl TransitHub {
    /// Create a new transit hub at the given location.
    pub fn new(hub_type: TransitHubType, grid_x: usize, grid_y: usize) -> Self {
        Self {
            supported_modes: hub_type.supported_modes(),
            transfer_penalty_reduction: hub_type.transfer_penalty_reduction(),
            hub_type,
            grid_x,
            grid_y,
        }
    }

    /// Get the effective transfer penalty in minutes when transferring
    /// between two modes at this hub. Returns the default penalty if one
    /// of the modes isn't supported by this hub.
    pub fn effective_transfer_penalty(&self, from: TransitMode, to: TransitMode) -> f32 {
        if self.supported_modes.contains(&from) && self.supported_modes.contains(&to) {
            HUB_TRANSFER_PENALTY_MINUTES
        } else {
            DEFAULT_TRANSFER_PENALTY_MINUTES
        }
    }
}

// =============================================================================
// Resource: TransitHubs (registry)
// =============================================================================

/// Registry tracking all transit hub locations and their detected modes.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct TransitHubs {
    /// Hub entries keyed by (grid_x, grid_y).
    pub hubs: Vec<TransitHubEntry>,
}

/// A single hub entry in the registry.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct TransitHubEntry {
    pub grid_x: usize,
    pub grid_y: usize,
    pub hub_type: TransitHubType,
    pub modes: Vec<TransitMode>,
}

impl TransitHubs {
    /// Find a hub at or near the given coordinates (within detection radius).
    pub fn find_hub_near(&self, x: usize, y: usize) -> Option<&TransitHubEntry> {
        self.hubs.iter().find(|h| {
            let dx = (h.grid_x as i32 - x as i32).unsigned_abs() as usize;
            let dy = (h.grid_y as i32 - y as i32).unsigned_abs() as usize;
            dx <= HUB_DETECTION_RADIUS as usize && dy <= HUB_DETECTION_RADIUS as usize
        })
    }

    /// Get the transfer penalty between two modes at a location.
    /// Returns the hub-reduced penalty if a hub exists, otherwise the default.
    pub fn transfer_penalty_at(
        &self,
        x: usize,
        y: usize,
        from: TransitMode,
        to: TransitMode,
    ) -> f32 {
        if let Some(hub) = self.find_hub_near(x, y) {
            if hub.modes.contains(&from) && hub.modes.contains(&to) {
                return HUB_TRANSFER_PENALTY_MINUTES;
            }
        }
        DEFAULT_TRANSFER_PENALTY_MINUTES
    }
}

// =============================================================================
// Resource: TransitHubStats
// =============================================================================

/// Aggregated statistics about transit hub usage.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct TransitHubStats {
    /// Total number of detected hubs.
    pub total_hubs: u32,
    /// Number of BusMetroHub hubs.
    pub bus_metro_hubs: u32,
    /// Number of TrainMetroHub hubs.
    pub train_metro_hubs: u32,
    /// Number of MultiModalHub hubs.
    pub multi_modal_hubs: u32,
    /// Total transfers tracked at hubs this cycle.
    pub total_transfers: u64,
    /// Average transfer penalty across all hubs (in minutes).
    pub avg_transfer_penalty: f32,
}

// =============================================================================
// Saveable implementations
// =============================================================================

impl crate::Saveable for TransitHubs {
    const SAVE_KEY: &'static str = "transit_hubs";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.hubs.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

impl crate::Saveable for TransitHubStats {
    const SAVE_KEY: &'static str = "transit_hub_stats";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.total_hubs == 0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
