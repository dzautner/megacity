//! Types and resources for the search/filter feature.

use bevy::prelude::*;

/// Maximum number of results to display per category.
pub const MAX_RESULTS: usize = 50;

/// Tracks the search panel state.
#[derive(Resource)]
pub struct SearchState {
    /// Whether the search panel is visible.
    pub visible: bool,
    /// The current search query text.
    pub query: String,
    /// Whether to search buildings.
    pub search_buildings: bool,
    /// Whether to search citizens.
    pub search_citizens: bool,
    /// Cached building results: (entity, zone_label, level, status, grid_x, grid_y).
    pub building_results: Vec<BuildingResult>,
    /// Cached citizen results: (entity, name, age, education_label, grid_x, grid_y).
    pub citizen_results: Vec<CitizenResult>,
    /// Whether results need to be refreshed.
    pub dirty: bool,
    /// Track the previous query to detect changes.
    pub(crate) prev_query: String,
    /// Whether the text field should request focus on the next frame.
    pub(crate) request_focus: bool,
}

#[derive(Clone)]
pub struct BuildingResult {
    pub entity: Entity,
    pub zone_label: String,
    pub level: u8,
    pub status: &'static str,
    pub grid_x: usize,
    pub grid_y: usize,
}

#[derive(Clone)]
pub struct CitizenResult {
    pub entity: Entity,
    pub name: String,
    pub age: u8,
    pub education: &'static str,
    pub grid_x: f32,
    pub grid_y: f32,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            visible: false,
            query: String::new(),
            search_buildings: true,
            search_citizens: true,
            building_results: Vec::new(),
            citizen_results: Vec::new(),
            dirty: false,
            prev_query: String::new(),
            request_focus: false,
        }
    }
}
