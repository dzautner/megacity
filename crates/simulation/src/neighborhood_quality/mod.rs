//! Neighborhood Quality Index (ZONE-014).
//!
//! Computes a composite quality index per statistical district combining:
//! - Walkability (20%): road connectivity, paths, and sidewalk density
//! - Service coverage (20%): health, education, police, fire, park coverage
//! - Environment quality (20%): inverse of pollution and noise levels
//! - Crime rate (15%): inverse of crime level
//! - Park access (15%): fraction of cells with park coverage
//! - Building quality average (10%): average building level in district
//!
//! Computed per district on the slow tick. Affects immigration attractiveness
//! at the district level -- high-quality neighborhoods attract higher-income citizens.

pub mod compute;
pub mod systems;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export all public items for backward compatibility.
pub use compute::{
    compute_composite_index, compute_environment_quality, compute_park_access, compute_safety,
    compute_service_coverage, compute_walkability,
};
pub use systems::update_neighborhood_quality;
pub use types::{
    DistrictQuality, NeighborhoodQualityIndex, WEIGHT_BUILDING_QUALITY, WEIGHT_CRIME,
    WEIGHT_ENVIRONMENT, WEIGHT_PARK_ACCESS, WEIGHT_SERVICE_COVERAGE, WEIGHT_WALKABILITY,
};

use bevy::prelude::*;

// =============================================================================
// Plugin
// =============================================================================

pub struct NeighborhoodQualityPlugin;

impl Plugin for NeighborhoodQualityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NeighborhoodQualityIndex>().add_systems(
            FixedUpdate,
            update_neighborhood_quality
                .after(crate::crime::update_crime)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<NeighborhoodQualityIndex>();
    }
}
