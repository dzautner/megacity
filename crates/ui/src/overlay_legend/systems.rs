//! Overlay legend UI system and helper functions.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::color_ramps::ColorRamp;
use rendering::overlay::{OverlayMode, OverlayState};
use simulation::colorblind::ColorblindSettings;

use super::metadata::legend_for_mode;
use super::types::{
    LegendKind, LegendTextureCache, GRADIENT_HEIGHT, GRADIENT_STEPS, GRADIENT_WIDTH, MARGIN,
};

// =============================================================================
// Systems
// =============================================================================

pub(crate) fn overlay_legend_ui(
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
            render_continuous_legend(
                ctx, &mut cache, mode, cb_mode, ramp, name, min_label, max_label, &screen,
            );
        }
        LegendKind::Binary {
            on_color,
            off_color,
            on_label,
            off_label,
        } => {
            render_binary_legend(
                ctx, &mut cache, mode, cb_mode, name, on_color, off_color, on_label, off_label,
                &screen,
            );
        }
        LegendKind::Directional { description } => {
            render_directional_legend(ctx, &mut cache, mode, cb_mode, name, description, &screen);
        }
    }
}

// =============================================================================
// Render helpers
// =============================================================================

#[allow(clippy::too_many_arguments)]
fn render_continuous_legend(
    ctx: &egui::Context,
    cache: &mut ResMut<LegendTextureCache>,
    mode: OverlayMode,
    cb_mode: simulation::colorblind::ColorblindMode,
    ramp: &'static ColorRamp,
    name: &str,
    min_label: &str,
    max_label: &str,
    screen: &egui::Rect,
) {
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
    let panel_height = GRADIENT_HEIGHT + 60.0;
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
                        if let Some(ref tex) = cache.texture {
                            let size = egui::vec2(GRADIENT_WIDTH, GRADIENT_HEIGHT);
                            ui.image(egui::load::SizedTexture::new(tex.id(), size));
                        }

                        ui.vertical(|ui| {
                            ui.set_height(GRADIENT_HEIGHT);
                            ui.label(
                                egui::RichText::new(max_label)
                                    .size(11.0)
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
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

#[allow(clippy::too_many_arguments)]
fn render_binary_legend(
    ctx: &egui::Context,
    cache: &mut ResMut<LegendTextureCache>,
    mode: OverlayMode,
    cb_mode: simulation::colorblind::ColorblindMode,
    name: &str,
    on_color: egui::Color32,
    off_color: egui::Color32,
    on_label: &str,
    off_label: &str,
    screen: &egui::Rect,
) {
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
                    ui.label(
                        egui::RichText::new(name)
                            .strong()
                            .size(13.0)
                            .color(egui::Color32::WHITE),
                    );
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, on_color);
                        ui.label(
                            egui::RichText::new(on_label)
                                .size(11.0)
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                    });

                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
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

fn render_directional_legend(
    ctx: &egui::Context,
    cache: &mut ResMut<LegendTextureCache>,
    mode: OverlayMode,
    cb_mode: simulation::colorblind::ColorblindMode,
    name: &str,
    description: &str,
    screen: &egui::Rect,
) {
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

// =============================================================================
// Conversion helpers
// =============================================================================

/// Convert a Bevy `Color` (sRGBA) to an egui `Color32`.
pub(crate) fn bevy_color_to_egui(color: bevy::prelude::Color) -> egui::Color32 {
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
