//! UX-044: Search/Filter for Buildings and Citizens.
//!
//! Provides a search bar (toggled via Ctrl+F) to find buildings and citizens.
//! Buildings can be searched by zone type, level, or status (abandoned, under construction).
//! Citizens can be searched by name, age, or occupation (education level).
//! Results are displayed in a scrollable list; clicking a result jumps the camera.

mod helpers;
mod systems;
mod types;

#[cfg(test)]
mod tests;

pub use systems::{search_keybind, search_panel_ui, update_search_results};
pub use types::{BuildingResult, CitizenResult, SearchState};

use bevy::prelude::*;
use simulation::app_state::AppState;

pub struct SearchPlugin;

impl Plugin for SearchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SearchState>().add_systems(
            Update,
            (search_keybind, update_search_results, search_panel_ui)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}
