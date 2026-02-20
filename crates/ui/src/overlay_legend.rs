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

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::color_ramps::{
    ColorRamp, CIVIDIS, GROUNDWATER_LEVEL, GROUNDWATER_QUALITY, INFERNO, VIRIDIS,
};
use rendering::colorblind_palette;
use rendering::overlay::{OverlayMode, OverlayState};
use simulation::colorblind::ColorblindSettings;

// =============================================================================
// Constants
// =============================================================================

/// Height of the gradient bar in pixels.
const GRADIENT_HEIGHT: f32 = 150.0;
/// Width of the gradient bar in pixels.
const GRADIENT_WIDTH: f32 = 20.0;
/// Number of vertical steps used to render the gradient texture.
const GRADIENT_STEPS: usize = 64;
/// Margin from the bottom-left corner of the screen.
const MARGIN: f32 = 16.0;

// =============================================================================
// Plugin
// =============================================================================

pub struct OverlayLegendPlugin;

impl Plugin for OverlayLegendPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LegendTextureCache>()
            .add_systems(Update, overlay_legend_ui);
    }
}

// =============================================================================
// Resources
// =============================================================================

/// Cached gradient texture to avoid regenerating every frame.
#[derive(Resource, Default)]
struct LegendTextureCache {
    /// The overlay mode the cached texture was generated for.
    cached_mode: Option<OverlayMode>,
    /// Whether colorblind mode was active when the texture was generated.
    cached_cb_mode: Option<simulation::colorblind::ColorblindMode>,
    /// The egui texture handle for the gradient.
    texture: Option<egui::TextureHandle>,
}

// =============================================================================
// Overlay metadata
// =============================================================================

/// Describes how to render the legend for a given overlay.
enum LegendKind {
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

fn legend_for_mode(
    mode: OverlayMode,
    cb_mode: simulation::colorblind::ColorblindMode,
) -> Option<(&'static str, LegendKind)> {
    match mode {
        OverlayMode::None => None,
        OverlayMode::Power => {
            let palette = colorblind_palette::power_palette(cb_mode);
            let on = bevy_color_to_egui(palette.on);
            let off = bevy_color_to_egui(palette.off);
            Some((
                "Power",
                LegendKind::Binary {
                    on_color: on,
                    off_color: off,
                    on_label: "Powered",
                    off_label: "No Power",
                },
            ))
        }
        OverlayMode::Water => {
            let palette = colorblind_palette::water_palette(cb_mode);
            let on = bevy_color_to_egui(palette.on);
            let off = bevy_color_to_egui(palette.off);
            Some((
                "Water",
                LegendKind::Binary {
                    on_color: on,
                    off_color: off,
                    on_label: "Connected",
                    off_label: "No Water",
                },
            ))
        }
        OverlayMode::Traffic => Some((
            "Traffic",
            LegendKind::Continuous {
                ramp: &INFERNO,
                min_label: "Free Flow",
                max_label: "Gridlock",
            },
        )),
        OverlayMode::Pollution => Some((
            "Pollution",
            LegendKind::Continuous {
                ramp: &INFERNO,
                min_label: "Clean",
                max_label: "Polluted",
            },
        )),
        OverlayMode::LandValue => Some((
            "Land Value",
            LegendKind::Continuous {
                ramp: &CIVIDIS,
                min_label: "Low",
                max_label: "High",
            },
        )),
        OverlayMode::Education => Some((
            "Education",
            LegendKind::Continuous {
                ramp: &VIRIDIS,
                min_label: "None",
                max_label: "University",
            },
        )),
        OverlayMode::Garbage => Some((
            "Garbage",
            LegendKind::Continuous {
                ramp: &INFERNO,
                min_label: "Clean",
                max_label: "Full",
            },
        )),
        OverlayMode::Noise => Some((
            "Noise",
            LegendKind::Continuous {
                ramp: &INFERNO,
                min_label: "Quiet",
                max_label: "Loud",
            },
        )),
        OverlayMode::WaterPollution => Some((
            "Water Pollution",
            LegendKind::Continuous {
                ramp: &VIRIDIS,
                min_label: "Polluted",
                max_label: "Clean",
            },
        )),
        OverlayMode::GroundwaterLevel => Some((
            "Groundwater Level",
            LegendKind::Continuous {
                ramp: &GROUNDWATER_LEVEL,
                min_label: "Dry",
                max_label: "Saturated",
            },
        )),
        OverlayMode::GroundwaterQuality => Some((
            "Groundwater Quality",
            LegendKind::Continuous {
                ramp: &GROUNDWATER_QUALITY,
                min_label: "Contaminated",
                max_label: "Clean",
            },
        )),
        OverlayMode::Wind => Some((
            "Wind",
            LegendKind::Directional {
                description: "Arrows show wind direction and speed",
            },
        )),
    }
}

// =============================================================================
// Systems
// =============================================================================

fn overlay_legend_ui(
    mut contexts: EguiContexts,
    overlay: Res<OverlayState>,
    cb_settings: Res<ColorblindSettings>,
    mut cache: ResMut<LegendTextureCache>,
) {
    let cb_mode = cb_settings.mode;
    let mode = overlay.mode;

    let Some((name, kind)) = legend_for_mode(mode, cb_mode) else {
        // No overlay active -- clear cache and return
        if cache.cached_mode.is_some() {
            cache.cached_mode = None;
            cache.cached_cb_mode = None;
            cache.texture = None;
        }
        return;
    };

    let ctx = contexts.ctx_mut();
    let screen = ctx.screen_rect();

    match kind {
        LegendKind::Continuous {
            ramp,
            min_label,
            max_label,
        } => {
            // Regenerate texture if mode or colorblind setting changed
            let needs_regen = cache.cached_mode != Some(mode)
                || cache.cached_cb_mode != Some(cb_mode)
                || cache.texture.is_none();
            if needs_regen {
                let texture = generate_gradient_texture(ctx, ramp, mode);
                cache.texture = Some(texture);
                cache.cached_mode = Some(mode);
                cache.cached_cb_mode = Some(cb_mode);
            }

            // Position: bottom-left corner with margin
            // The window should sit above the bottom edge
            let panel_height = GRADIENT_HEIGHT + 60.0; // gradient + labels + name
            let pos = egui::pos2(MARGIN, screen.max.y - panel_height - MARGIN);

            egui::Area::new(egui::Id::new("overlay_legend"))
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgba_unmultiplied(35, 37, 48, 220))
                        .corner_radius(egui::CornerRadius::same(6))
                        .inner_margin(egui::Margin::same(8))
                        .show(ui, |ui| {
                            ui.set_width(GRADIENT_WIDTH + 60.0);

                            // Overlay name
                            ui.label(
                                egui::RichText::new(name)
                                    .strong()
                                    .size(13.0)
                                    .color(egui::Color32::WHITE),
                            );
                            ui.add_space(4.0);

                            // Gradient bar with min/max labels
                            ui.horizontal(|ui| {
                                // Gradient bar
                                if let Some(ref tex) = cache.texture {
                                    let size = egui::vec2(GRADIENT_WIDTH, GRADIENT_HEIGHT);
                                    ui.image(egui::load::SizedTexture::new(tex.id(), size));
                                }

                                // Labels on the right side
                                ui.vertical(|ui| {
                                    ui.set_height(GRADIENT_HEIGHT);
                                    // Max label at top
                                    ui.label(
                                        egui::RichText::new(max_label)
                                            .size(11.0)
                                            .color(egui::Color32::LIGHT_GRAY),
                                    );
                                    // Push min label to bottom
                                    ui.add_space(ui.available_height() - 16.0);
                                    ui.label(
                                        egui::RichText::new(min_label)
                                            .size(11.0)
                                            .color(egui::Color32::LIGHT_GRAY),
                                    );
                                });
                            });
                        });
                });
        }
        LegendKind::Binary {
            on_color,
            off_color,
            on_label,
            off_label,
        } => {
            // Clear continuous ramp cache
            if cache.cached_mode != Some(mode) || cache.cached_cb_mode != Some(cb_mode) {
                cache.texture = None;
                cache.cached_mode = Some(mode);
                cache.cached_cb_mode = Some(cb_mode);
            }

            let panel_height = 80.0;
            let pos = egui::pos2(MARGIN, screen.max.y - panel_height - MARGIN);

            egui::Area::new(egui::Id::new("overlay_legend"))
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgba_unmultiplied(35, 37, 48, 220))
                        .corner_radius(egui::CornerRadius::same(6))
                        .inner_margin(egui::Margin::same(8))
                        .show(ui, |ui| {
                            // Overlay name
                            ui.label(
                                egui::RichText::new(name)
                                    .strong()
                                    .size(13.0)
                                    .color(egui::Color32::WHITE),
                            );
                            ui.add_space(4.0);

                            // On swatch + label
                            ui.horizontal(|ui| {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(16.0, 16.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(rect, 2.0, on_color);
                                ui.label(
                                    egui::RichText::new(on_label)
                                        .size(11.0)
                                        .color(egui::Color32::LIGHT_GRAY),
                                );
                            });

                            // Off swatch + label
                            ui.horizontal(|ui| {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(16.0, 16.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(rect, 2.0, off_color);
                                ui.label(
                                    egui::RichText::new(off_label)
                                        .size(11.0)
                                        .color(egui::Color32::LIGHT_GRAY),
                                );
                            });
                        });
                });
        }
        LegendKind::Directional { description } => {
            // Clear any cached gradient texture
            if cache.cached_mode != Some(mode) || cache.cached_cb_mode != Some(cb_mode) {
                cache.texture = None;
                cache.cached_mode = Some(mode);
                cache.cached_cb_mode = Some(cb_mode);
            }

            let panel_height = 60.0;
            let pos = egui::pos2(MARGIN, screen.max.y - panel_height - MARGIN);

            egui::Area::new(egui::Id::new("overlay_legend"))
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgba_unmultiplied(35, 37, 48, 220))
                        .corner_radius(egui::CornerRadius::same(6))
                        .inner_margin(egui::Margin::same(8))
                        .show(ui, |ui| {
                            // Overlay name
                            ui.label(
                                egui::RichText::new(name)
                                    .strong()
                                    .size(13.0)
                                    .color(egui::Color32::WHITE),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(description)
                                    .size(11.0)
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                        });
                });
        }
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Convert a Bevy `Color` (sRGBA) to an egui `Color32`.
fn bevy_color_to_egui(color: bevy::prelude::Color) -> egui::Color32 {
    let s = color.to_srgba();
    egui::Color32::from_rgba_unmultiplied(
        (s.red * 255.0) as u8,
        (s.green * 255.0) as u8,
        (s.blue * 255.0) as u8,
        (s.alpha * 255.0) as u8,
    )
}

/// Generate a vertical gradient texture for a continuous color ramp.
/// The texture has the max value (t=1) at the top and min value (t=0) at the bottom.
fn generate_gradient_texture(
    ctx: &egui::Context,
    ramp: &ColorRamp,
    mode: OverlayMode,
) -> egui::TextureHandle {
    let width = 1; // single column, egui stretches it
    let height = GRADIENT_STEPS;
    let mut pixels = Vec::with_capacity(width * height);

    for row in 0..height {
        // Row 0 is top of image = t=1 (max), row (height-1) is bottom = t=0 (min)
        let t = 1.0 - (row as f32 / (height - 1) as f32);
        let rgba = ramp.sample_rgba(t);
        pixels.push(egui::Color32::from_rgb(
            (rgba[0] * 255.0) as u8,
            (rgba[1] * 255.0) as u8,
            (rgba[2] * 255.0) as u8,
        ));
    }

    let color_image = egui::ColorImage {
        size: [width, height],
        pixels,
    };

    let tex_name = format!("overlay_legend_{:?}", mode);
    ctx.load_texture(tex_name, color_image, egui::TextureOptions::LINEAR)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::colorblind::ColorblindMode;

    #[test]
    fn legend_returns_none_for_no_overlay() {
        assert!(legend_for_mode(OverlayMode::None, ColorblindMode::Normal).is_none());
    }

    #[test]
    fn legend_returns_some_for_all_active_overlays() {
        let modes = [
            OverlayMode::Power,
            OverlayMode::Water,
            OverlayMode::Traffic,
            OverlayMode::Pollution,
            OverlayMode::LandValue,
            OverlayMode::Education,
            OverlayMode::Garbage,
            OverlayMode::Noise,
            OverlayMode::WaterPollution,
            OverlayMode::GroundwaterLevel,
            OverlayMode::GroundwaterQuality,
            OverlayMode::Wind,
        ];
        for mode in modes {
            let result = legend_for_mode(mode, ColorblindMode::Normal);
            assert!(
                result.is_some(),
                "legend_for_mode should return Some for {:?}",
                mode
            );
            let (name, _) = result.unwrap();
            assert!(
                !name.is_empty(),
                "Legend name should not be empty for {:?}",
                mode
            );
        }
    }

    #[test]
    fn legend_works_with_all_colorblind_modes() {
        for cb_mode in ColorblindMode::ALL {
            // Power and Water are binary and change palette per colorblind mode
            let (name, kind) = legend_for_mode(OverlayMode::Power, cb_mode).unwrap();
            assert_eq!(name, "Power");
            assert!(matches!(kind, LegendKind::Binary { .. }));

            let (name, kind) = legend_for_mode(OverlayMode::Water, cb_mode).unwrap();
            assert_eq!(name, "Water");
            assert!(matches!(kind, LegendKind::Binary { .. }));

            // Continuous overlays should work too
            let (name, kind) = legend_for_mode(OverlayMode::Traffic, cb_mode).unwrap();
            assert_eq!(name, "Traffic");
            assert!(matches!(kind, LegendKind::Continuous { .. }));
        }
    }

    #[test]
    fn wind_overlay_returns_directional_legend() {
        let (name, kind) = legend_for_mode(OverlayMode::Wind, ColorblindMode::Normal).unwrap();
        assert_eq!(name, "Wind");
        assert!(matches!(kind, LegendKind::Directional { .. }));
    }

    #[test]
    fn binary_overlays_have_distinct_on_off_colors() {
        for cb_mode in ColorblindMode::ALL {
            for mode in [OverlayMode::Power, OverlayMode::Water] {
                let (_, kind) = legend_for_mode(mode, cb_mode).unwrap();
                if let LegendKind::Binary {
                    on_color,
                    off_color,
                    ..
                } = kind
                {
                    assert_ne!(
                        on_color, off_color,
                        "On/off colors should be distinct for {:?} in {:?} mode",
                        mode, cb_mode
                    );
                }
            }
        }
    }

    #[test]
    fn bevy_color_to_egui_converts_correctly() {
        let bevy_color = bevy::prelude::Color::srgba(1.0, 0.5, 0.0, 0.8);
        let egui_color = bevy_color_to_egui(bevy_color);
        assert_eq!(egui_color.r(), 255);
        assert_eq!(egui_color.g(), 127); // 0.5 * 255 = 127.5, truncated to 127
        assert_eq!(egui_color.b(), 0);
        assert_eq!(egui_color.a(), 204); // 0.8 * 255 = 204
    }
}
