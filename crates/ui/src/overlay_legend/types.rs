//! Types and constants for the overlay legend.

use bevy::prelude::*;
use bevy_egui::egui;
use rendering::color_ramps::ColorRamp;
use rendering::overlay::OverlayMode;

// =============================================================================
// Constants
// =============================================================================

/// Height of the gradient bar in pixels.
pub(crate) const GRADIENT_HEIGHT: f32 = 150.0;
/// Width of the gradient bar in pixels.
pub(crate) const GRADIENT_WIDTH: f32 = 20.0;
/// Number of vertical steps used to render the gradient texture.
pub(crate) const GRADIENT_STEPS: usize = 64;
/// Margin from the bottom-left corner of the screen.
pub(crate) const MARGIN: f32 = 16.0;

// =============================================================================
// Resources
// =============================================================================

/// Cached gradient texture to avoid regenerating every frame.
#[derive(Resource, Default)]
pub struct LegendTextureCache {
    /// The overlay mode the cached texture was generated for.
    pub(crate) cached_mode: Option<OverlayMode>,
    /// Whether colorblind mode was active when the texture was generated.
    pub(crate) cached_cb_mode: Option<simulation::colorblind::ColorblindMode>,
    /// The egui texture handle for the gradient.
    pub(crate) texture: Option<egui::TextureHandle>,
}

// =============================================================================
// Legend kind
// =============================================================================

/// Describes how to render the legend for a given overlay.
pub(crate) enum LegendKind {
    /// Continuous color ramp with min/max labels.
    Continuous {
        ramp: &'static ColorRamp,
        min_label: &'static str,
        max_label: &'static str,
    },
    /// Binary on/off overlay with two color swatches.
    Binary {
        on_color: egui::Color32,
        off_color: egui::Color32,
        on_label: &'static str,
        off_label: &'static str,
    },
    /// Directional overlay with informational description (no color ramp).
    Directional { description: &'static str },
}
