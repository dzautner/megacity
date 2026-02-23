//! Types and constants for the water supply dashboard.

use bevy::prelude::*;

/// Resource controlling whether the water dashboard window is visible.
/// Toggle with 'W' key.
#[derive(Resource, Default)]
pub struct WaterDashboardVisible(pub bool);

/// Conversion constant: 1 MGD = 1,000,000 gallons per day.
pub const MGD_TO_GPD: f32 = 1_000_000.0;

/// Aggregated water source data by type.
pub struct SourceAggregation {
    pub well_supply_mgd: f32,
    pub surface_supply_mgd: f32,
    pub reservoir_supply_mgd: f32,
    pub desal_supply_mgd: f32,
    pub well_count: u32,
    pub surface_count: u32,
    pub reservoir_source_count: u32,
    pub desal_count: u32,
    pub total_source_operating_cost: f64,
}

impl SourceAggregation {
    /// Returns true if any sources exist.
    pub fn has_sources(&self) -> bool {
        self.well_count > 0
            || self.surface_count > 0
            || self.reservoir_source_count > 0
            || self.desal_count > 0
    }
}
