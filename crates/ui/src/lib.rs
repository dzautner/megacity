use bevy::prelude::*;

// Auto-discover all public modules from src/ directory.
// plugin_registration is declared manually below because it is private.
automod_dir::dir!(pub "src" exclude "plugin_registration");

mod plugin_registration;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Register all UI systems and plugins (extracted for conflict-free additions)
        plugin_registration::register_ui_systems(app);
    }
}
