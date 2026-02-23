//! 15-Minute City Walkability Scoring (ZONE-013).
//!
//! Each cell receives a walkability score (0-100) based on how many essential
//! service categories are reachable within walking distance. The scoring follows
//! the Walk Score methodology:
//!
//! - Full points within 400m (~25 cells at CELL_SIZE=16)
//! - Linear decay to 0 at 1600m (~100 cells)
//!
//! Categories and weights:
//! - Grocery/Commercial: 0.25
//! - School/Education:   0.15
//! - Healthcare:         0.20
//! - Park/Recreation:    0.15
//! - Transit:            0.15
//! - Employment:         0.10
//!
//! The composite score is a weighted average of per-category scores. It affects
//! citizen happiness, land value, and mode choice via the `WalkabilityGrid`
//! resource that other systems can read.
//!
//! Computed on the slow tick (every ~100 ticks) since scanning 65K cells is
//! expensive.

pub mod categories;
pub mod grid;
pub mod scoring;

#[cfg(test)]
mod tests;

// Re-export all public items so that `crate::walkability::Foo` continues to work.
pub use categories::{classify_service, classify_zone, WalkabilityCategory};
pub use grid::{WalkabilityGrid, WALKABILITY_HAPPINESS_BONUS, WALKABILITY_LAND_VALUE_BONUS};
pub use scoring::{distance_decay, update_walkability};

use bevy::prelude::*;

// =============================================================================
// Plugin
// =============================================================================

pub struct WalkabilityPlugin;

impl Plugin for WalkabilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WalkabilityGrid>().add_systems(
            FixedUpdate,
            update_walkability
                .after(crate::happiness::update_service_coverage)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<WalkabilityGrid>();
    }
}
