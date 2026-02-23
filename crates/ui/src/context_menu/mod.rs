//! Right-Click Context Menu (UX-012).
//!
//! Right-click release (without drag, < 5px movement) shows a context menu
//! for the entity under the cursor. Menu items vary by entity type:
//! - **Building**: Inspect, Bulldoze
//! - **Service Building**: Inspect, Bulldoze
//! - **Road**: Inspect, Bulldoze, Toggle One-Way
//! - **Citizen**: Follow, Details
//! - **Empty**: Zone, Place Service
//!
//! The menu closes on click outside, Escape, or when an item is selected.

mod actions;
mod detection;
mod types;
mod ui_render;

pub use types::{ContextMenuState, ContextTarget};

use bevy::prelude::*;

use types::PendingAction;

pub struct ContextMenuPlugin;

impl Plugin for ContextMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ContextMenuState>()
            .init_resource::<PendingAction>()
            .add_systems(
                Update,
                (
                    detection::detect_right_click_context_menu,
                    ui_render::context_menu_ui,
                    actions::execute_context_menu_action,
                )
                    .chain(),
            );
    }
}
