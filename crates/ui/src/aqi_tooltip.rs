//! AQI (Air Quality Index) tooltip enhancement for the pollution overlay.
//!
//! When the Pollution overlay is active and the cursor hovers a cell,
//! this module adds AQI-specific information to the tooltip:
//! - AQI numeric value
//! - Tier name and color
//! - Health advisory text
//!
//! This is a standalone system that renders its own tooltip panel,
//! positioned below the main cell tooltip.

use bevy::prelude::*;
use simulation::app_state::AppState;
use bevy_egui::{egui, EguiContexts};

use rendering::aqi_colors;
use rendering::input::CursorGridPos;
use rendering::overlay::{OverlayMode, OverlayState};
use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::pollution::PollutionGrid;

use crate::cell_tooltip::CellHoverState;

/// How long the cursor must hover before showing the AQI tooltip (seconds).
/// Matches the main cell tooltip delay.
const AQI_HOVER_DELAY: f32 = 0.5;

/// Pixel offset from the cursor.
const AQI_TOOLTIP_OFFSET_X: f32 = 20.0;
const AQI_TOOLTIP_OFFSET_Y: f32 = 160.0;

pub struct AqiTooltipPlugin;

impl Plugin for AqiTooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, aqi_tooltip_ui.run_if(in_state(AppState::Playing)));
    }
}

fn aqi_tooltip_ui(
    mut contexts: EguiContexts,
    cursor: Res<CursorGridPos>,
    overlay: Res<OverlayState>,
    pollution: Res<PollutionGrid>,
    hover: Res<CellHoverState>,
) {
    // Only show when pollution overlay is active
    if overlay.mode != OverlayMode::Pollution {
        return;
    }

    if !cursor.valid {
        return;
    }

    // Wait for hover delay
    if hover.elapsed < AQI_HOVER_DELAY {
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    if gx >= GRID_WIDTH || gy >= GRID_HEIGHT {
        return;
    }

    let concentration = pollution.get(gx, gy);
    let aqi = aqi_colors::concentration_to_aqi(concentration);
    let tier = aqi_colors::aqi_to_tier(aqi);
    let tier_rgb = aqi_colors::tier_color_rgb(tier);
    let tier_egui_color = egui::Color32::from_rgb(
        (tier_rgb[0] * 255.0) as u8,
        (tier_rgb[1] * 255.0) as u8,
        (tier_rgb[2] * 255.0) as u8,
    );

    let ctx = contexts.ctx_mut();

    let Some(pointer_pos) = ctx.pointer_hover_pos() else {
        return;
    };

    let offset = egui::vec2(AQI_TOOLTIP_OFFSET_X, AQI_TOOLTIP_OFFSET_Y);
    let label_pos = pointer_pos + offset;

    egui::Area::new(egui::Id::new("aqi_tooltip"))
        .fixed_pos(egui::pos2(label_pos.x, label_pos.y))
        .interactable(false)
        .order(egui::Order::Tooltip)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 220))
                .show(ui, |ui| {
                    ui.set_max_width(240.0);

                    // AQI header with colored tier indicator
                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, tier_egui_color);

                        ui.label(
                            egui::RichText::new(format!("AQI: {aqi}"))
                                .strong()
                                .size(13.0)
                                .color(egui::Color32::WHITE),
                        );
                    });

                    ui.separator();

                    // Tier name
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Level:")
                                .size(11.0)
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                        ui.label(
                            egui::RichText::new(tier.label())
                                .size(11.0)
                                .color(tier_egui_color),
                        );
                    });

                    // Concentration value
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Concentration:")
                                .size(11.0)
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                        ui.label(
                            egui::RichText::new(format!("{concentration}"))
                                .size(11.0)
                                .color(egui::Color32::WHITE),
                        );
                    });

                    ui.add_space(4.0);

                    // Health advisory
                    ui.label(
                        egui::RichText::new(tier.health_advisory())
                            .size(10.0)
                            .color(egui::Color32::from_rgb(180, 180, 180))
                            .italics(),
                    );
                });
        });
}
