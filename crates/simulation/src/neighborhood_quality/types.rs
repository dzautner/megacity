//! Data types, constants, and resource definitions for neighborhood quality.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::districts::{Districts, DISTRICTS_X, DISTRICTS_Y};

// =============================================================================
// Weight constants (must sum to 1.0)
// =============================================================================

/// Walkability weight in the composite index.
pub const WEIGHT_WALKABILITY: f32 = 0.20;
/// Service coverage weight in the composite index.
pub const WEIGHT_SERVICE_COVERAGE: f32 = 0.20;
/// Environment quality (inverse pollution/noise) weight.
pub const WEIGHT_ENVIRONMENT: f32 = 0.20;
/// Crime rate (inverse) weight.
pub const WEIGHT_CRIME: f32 = 0.15;
/// Park access weight.
pub const WEIGHT_PARK_ACCESS: f32 = 0.15;
/// Building quality average weight.
pub const WEIGHT_BUILDING_QUALITY: f32 = 0.10;

/// Maximum building level used for normalization.
pub(crate) const MAX_BUILDING_LEVEL: f32 = 5.0;

// =============================================================================
// Per-district quality data
// =============================================================================

/// Quality index data for a single statistical district.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct DistrictQuality {
    /// Composite quality index (0.0 to 100.0).
    pub overall: f32,
    /// Walkability sub-score (0.0 to 1.0).
    pub walkability: f32,
    /// Service coverage sub-score (0.0 to 1.0).
    pub service_coverage: f32,
    /// Environment quality sub-score (0.0 to 1.0).
    pub environment: f32,
    /// Safety sub-score (inverse of crime, 0.0 to 1.0).
    pub safety: f32,
    /// Park access sub-score (0.0 to 1.0).
    pub park_access: f32,
    /// Building quality sub-score (0.0 to 1.0).
    pub building_quality: f32,
}

// =============================================================================
// Resource: neighborhood quality index per district
// =============================================================================

/// Resource holding the neighborhood quality index for every statistical district.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct NeighborhoodQualityIndex {
    /// One entry per statistical district (DISTRICTS_X * DISTRICTS_Y).
    pub districts: Vec<DistrictQuality>,
    /// City-wide average quality index (0.0 to 100.0).
    pub city_average: f32,
}

impl Default for NeighborhoodQualityIndex {
    fn default() -> Self {
        Self {
            districts: vec![DistrictQuality::default(); DISTRICTS_X * DISTRICTS_Y],
            city_average: 0.0,
        }
    }
}

impl NeighborhoodQualityIndex {
    /// Get the quality data for a given statistical district.
    pub fn get(&self, dx: usize, dy: usize) -> &DistrictQuality {
        &self.districts[dy * DISTRICTS_X + dx]
    }

    /// Get the quality index for the district containing a grid cell.
    pub fn quality_at_cell(&self, gx: usize, gy: usize) -> f32 {
        let (dx, dy) = Districts::district_for_grid(gx, gy);
        if dx < DISTRICTS_X && dy < DISTRICTS_Y {
            self.districts[dy * DISTRICTS_X + dx].overall
        } else {
            0.0
        }
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for NeighborhoodQualityIndex {
    const SAVE_KEY: &'static str = "neighborhood_quality";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if all districts are at default (overall == 0.0)
        let has_data = self.districts.iter().any(|d| d.overall > 0.0);
        if !has_data {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
