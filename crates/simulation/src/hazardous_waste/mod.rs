//! Hazardous waste facility and industrial waste management (WASTE-007).
//!
//! Industrial and medical buildings generate hazardous waste that requires
//! specialized treatment at a HazardousWasteFacility. Without adequate
//! treatment capacity, overflow waste triggers illegal dumping events that
//! cause soil and groundwater contamination, plus federal fines.
//!
//! Key design points:
//! - HazardousWasteFacility: 20 tons/day capacity, $3M build, $5K/day operating
//! - Industrial buildings generate waste based on level (higher level = more waste)
//! - Medical buildings (Hospital, MedicalClinic, MedicalCenter) generate medical waste
//! - Without facility: illegal dumping causes contamination + federal fines
//! - Treatment types: chemical, thermal, biological, stabilization

pub mod constants;
pub mod systems;
mod tests;
pub mod types;

pub use constants::*;
pub use systems::*;
pub use types::*;

use bevy::prelude::*;

pub struct HazardousWastePlugin;

impl Plugin for HazardousWastePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HazardousWasteState>().add_systems(
            FixedUpdate,
            update_hazardous_waste
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
