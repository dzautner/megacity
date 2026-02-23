//! Two-key tool shortcuts (UX-014).
//!
//! Press a category key (R for roads, Z for zones, etc.) to open a numbered
//! popup listing the sub-tools for that category.  Then press a digit key
//! (1-9, 0) to select the corresponding sub-tool.  The popup auto-closes
//! after a 2-second timeout or when Escape is pressed.

mod categories;
mod input;
mod popup_ui;

use bevy::prelude::*;

pub use self::input::TwoKeyShortcutState;

/// Plugin that registers the two-key shortcut system.
pub struct TwoKeyShortcutPlugin;

impl Plugin for TwoKeyShortcutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TwoKeyShortcutState>().add_systems(
            Update,
            (input::two_key_input_system, popup_ui::two_key_popup_ui),
        );
    }
}
