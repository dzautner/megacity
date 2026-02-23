//! NIMBY/YIMBY citizen mechanics (ZONE-007).
//!
//! When the player rezones land or places a high-impact building, nearby
//! citizens generate opinions (opposition or support) based on personality,
//! property values, and the type of development.
//!
//! **NIMBY factors** (increase opposition):
//! - Density increase (e.g. low-res to high-res or industrial)
//! - Industrial adjacency to residential
//! - Traffic increase from commercial/industrial zones
//! - Income mismatch (high-income residents oppose low-density changes)
//!
//! **YIMBY factors** (increase support):
//! - Amenity addition (parks, transit coverage)
//! - Job creation (commercial/office near unemployed residents)
//! - Housing need (when residential vacancy is very low)
//!
//! **Effects**:
//! - Net opposition reduces happiness of nearby citizens
//! - Opposition strength scales with land value (wealthy areas oppose more)
//! - High opposition slows construction (increases `UnderConstruction` ticks)
//! - High opposition reduces building upgrade speed
//! - Protests are logged to the `EventJournal` as visual events
//! - Eminent Domain policy overrides opposition at a global happiness cost

pub mod opinion;
pub mod systems;
pub mod types;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_state;

// Re-export all public items for backward compatibility.
pub use opinion::{
    calculate_opinion, construction_slowdown, is_residential, nimby_happiness_penalty,
    zone_density_score,
};
pub use systems::{
    apply_construction_slowdown, apply_nimby_happiness, detect_zone_changes, update_nimby_opinions,
};
pub use types::{NimbyState, ZoneChangeEvent, ZoneSnapshot, EMINENT_DOMAIN_HAPPINESS_PENALTY};

use crate::SaveableRegistry;
use bevy::prelude::*;

// =============================================================================
// Plugin
// =============================================================================

pub struct NimbyPlugin;

impl Plugin for NimbyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NimbyState>()
            .init_resource::<ZoneSnapshot>()
            .add_systems(
                FixedUpdate,
                (
                    detect_zone_changes,
                    update_nimby_opinions,
                    apply_nimby_happiness,
                    apply_construction_slowdown,
                )
                    .chain()
                    .after(crate::zones::update_zone_demand)
                    .in_set(crate::SimulationSet::Simulation),
            );
        app.init_resource::<SaveableRegistry>();
        app.world_mut()
            .resource_mut::<SaveableRegistry>()
            .register::<NimbyState>();
    }
}
