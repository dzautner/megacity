//! Water Treatment Plant Level System (WATER-003).
//!
//! Split into sub-modules:
//! - `types`: Treatment level enum, plant/city state, helper functions
//! - `systems`: ECS system and plugin registration

mod systems;
mod types;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_risk_demand;

pub use systems::{update_water_treatment, WaterTreatmentPlugin};
pub use types::{
    calculate_disease_risk, calculate_effluent_quality, estimate_demand_mgd, PlantState,
    TreatmentLevel, WaterTreatmentState,
};
