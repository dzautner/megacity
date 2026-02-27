//! UX-028: Overlay Legend (Color Ramp + Value Range)
//!
//! When an overlay is active, displays a vertical gradient bar (150px tall,
//! 20px wide) in the bottom-left corner with:
//! - Overlay name label at the top
//! - Color ramp gradient bar
//! - Min/max value labels
//!
//! Supports both continuous (color ramp) and binary (on/off) overlay types.
//! Respects colorblind palette adjustments.
//!
//! The Wind overlay uses gizmo streamlines (directional arrows) rather than
//! a color ramp, so it shows a simple informational label instead.

mod metadata;
mod systems;
#[cfg(test)]
mod tests;
mod types;

pub use types::LegendTextureCache;

use bevy::prelude::*;
use simulation::app_state::AppState;

pub struct OverlayLegendPlugin;

impl Plugin for OverlayLegendPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LegendTextureCache>()
            .add_systems(Update, systems::overlay_legend_ui.run_if(in_state(AppState::Playing)));
    }
}
