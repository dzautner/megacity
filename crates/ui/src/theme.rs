//! Centralized UI theme for Megacity.
//!
//! Provides a unified color palette, font sizes, and spacing constants
//! for all egui-based UI panels. The `ThemePlugin` applies the theme on
//! startup so every panel inherits consistent styling automatically.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

// =============================================================================
// Color Palette — dark-theme city-builder appropriate
// =============================================================================

/// Deep background for panels and windows.
pub const BG_DARK: egui::Color32 = egui::Color32::from_rgb(25, 27, 35);
/// Standard panel/window background.
pub const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(35, 37, 48);
/// Slightly lighter surface for cards or nested sections.
pub const BG_SURFACE: egui::Color32 = egui::Color32::from_rgb(45, 48, 60);
/// Faint background used for input fields and code areas.
pub const BG_FAINT: egui::Color32 = egui::Color32::from_rgb(40, 42, 52);

/// Primary accent — cyan-blue (buttons, highlights, links).
pub const PRIMARY: egui::Color32 = egui::Color32::from_rgb(70, 160, 230);
/// Secondary accent — teal/green-blue.
pub const SECONDARY: egui::Color32 = egui::Color32::from_rgb(60, 190, 180);

/// Default text color.
pub const TEXT: egui::Color32 = egui::Color32::from_rgb(220, 220, 230);
/// Muted/secondary text color.
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(140, 145, 160);
/// Heading text color — slightly brighter than body text.
pub const TEXT_HEADING: egui::Color32 = egui::Color32::from_rgb(240, 240, 250);

/// Success / positive indicator.
pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(50, 200, 80);
/// Warning indicator.
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(230, 180, 50);
/// Error / negative indicator.
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(220, 60, 60);

/// Widget states — inactive, hover, active.
pub const WIDGET_INACTIVE: egui::Color32 = egui::Color32::from_rgb(50, 55, 65);
pub const WIDGET_HOVER: egui::Color32 = egui::Color32::from_rgb(65, 75, 95);
pub const WIDGET_ACTIVE: egui::Color32 = egui::Color32::from_rgb(80, 140, 210);

/// Separator / border color.
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(60, 65, 80);

// =============================================================================
// Font Sizes
// =============================================================================

/// Large heading (panel titles).
pub const FONT_HEADING: f32 = 18.0;
/// Sub-heading (section titles).
pub const FONT_SUBHEADING: f32 = 15.0;
/// Body text.
pub const FONT_BODY: f32 = 13.0;
/// Small / caption text.
pub const FONT_SMALL: f32 = 11.0;

// =============================================================================
// Spacing
// =============================================================================

/// Standard inner padding for windows/panels.
pub const PADDING: f32 = 8.0;
/// Space between sections.
pub const SECTION_SPACING: f32 = 12.0;
/// Space between items in a list.
pub const ITEM_SPACING: f32 = 4.0;
/// Standard window corner radius.
pub const CORNER_RADIUS: u8 = 8;
/// Widget corner radius.
pub const WIDGET_CORNER_RADIUS: u8 = 6;

// =============================================================================
// Theme application
// =============================================================================

/// Apply the Megacity dark theme to the egui context.
///
/// Called once at startup. Configures `Visuals`, widget styles, and colors
/// so all panels inherit a consistent look without per-panel styling.
pub fn setup_megacity_theme(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    let mut style = (*ctx.style()).clone();

    // Window / panel backgrounds
    style.visuals.window_fill = BG_PANEL;
    style.visuals.panel_fill = BG_PANEL;
    style.visuals.extreme_bg_color = BG_DARK;
    style.visuals.faint_bg_color = BG_FAINT;

    // Widget colors
    style.visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    style.visuals.widgets.inactive.bg_fill = WIDGET_INACTIVE;
    style.visuals.widgets.inactive.weak_bg_fill = WIDGET_INACTIVE;
    style.visuals.widgets.hovered.bg_fill = WIDGET_HOVER;
    style.visuals.widgets.hovered.weak_bg_fill = WIDGET_HOVER;
    style.visuals.widgets.active.bg_fill = WIDGET_ACTIVE;
    style.visuals.widgets.active.weak_bg_fill = WIDGET_ACTIVE;

    // Text colors per widget state
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT);
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT);
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT_HEADING);
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT_HEADING);

    // Border strokes
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(0.5, BORDER);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, PRIMARY);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, PRIMARY);

    // Selection
    style.visuals.selection.bg_fill = WIDGET_ACTIVE;
    style.visuals.selection.stroke = egui::Stroke::new(1.0, PRIMARY);

    // Rounding
    let window_rounding = egui::CornerRadius::same(CORNER_RADIUS);
    let widget_rounding = egui::CornerRadius::same(WIDGET_CORNER_RADIUS);

    style.visuals.window_corner_radius = window_rounding;
    style.visuals.widgets.noninteractive.corner_radius = widget_rounding;
    style.visuals.widgets.inactive.corner_radius = widget_rounding;
    style.visuals.widgets.hovered.corner_radius = widget_rounding;
    style.visuals.widgets.active.corner_radius = widget_rounding;

    // Window stroke
    style.visuals.window_stroke = egui::Stroke::new(1.0, BORDER);

    ctx.set_style(style);
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that applies the Megacity UI theme at startup.
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_megacity_theme);
    }
}
