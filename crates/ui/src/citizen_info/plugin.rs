//! Plugin registration for the citizen info panel.

use bevy::prelude::*;

use super::resources::{FollowCitizen, SelectedCitizen};
use super::systems::{camera_follow_citizen, citizen_info_panel_ui, detect_citizen_selection};

pub struct CitizenInfoPlugin;

impl Plugin for CitizenInfoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedCitizen>()
            .init_resource::<FollowCitizen>()
            .add_systems(
                Update,
                (
                    detect_citizen_selection,
                    citizen_info_panel_ui,
                    camera_follow_citizen,
                )
                    .chain(),
            );
    }
}
