//! Resources for district inspection panel state.

use bevy::prelude::*;

/// Resource tracking the currently selected district index.
#[derive(Resource, Default)]
pub struct SelectedDistrict(pub Option<usize>);

/// Whether the district inspection panel is open (controls boundary highlighting).
#[derive(Resource, Default)]
pub struct DistrictPanelOpen(pub bool);

/// Cached district statistics for the selected district, refreshed each frame.
#[derive(Resource, Default)]
pub struct DistrictInspectCache {
    pub name: String,
    pub color: [f32; 4],
    pub population: u32,
    pub commercial_jobs: u32,
    pub industrial_jobs: u32,
    pub office_jobs: u32,
    pub avg_happiness: f32,
    pub cell_count: usize,
    // Service coverage counts
    pub fire_services: u32,
    pub police_services: u32,
    pub health_services: u32,
    pub education_services: u32,
    pub park_services: u32,
    pub transport_services: u32,
    pub valid: bool,
}
