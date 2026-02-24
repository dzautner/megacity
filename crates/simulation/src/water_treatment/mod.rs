//! Water Treatment and Quality System (SVC-017 / WATER-003).
//!
//! Split into sub-modules:
//! - `types`: Treatment level enum, plant/city state, helper functions, Saveable
//! - `systems`: Core ECS update system and plugin registration
//! - `effects`: Grid interaction systems (pollution reduction, quality boost, WellPump)

mod effects;
mod systems;
mod types;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_risk_demand;

pub use effects::{apply_treatment_grid_effects, apply_well_pump_effects};
pub use systems::{update_water_treatment, WaterTreatmentPlugin};
pub use types::{
    calculate_disease_risk, calculate_effluent_quality, estimate_demand_mgd, PlantState,
    TreatmentLevel, WaterTreatmentSaveData, WaterTreatmentState,
};
