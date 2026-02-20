use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::overlay::{OverlayMode, OverlayState};
use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::services::ServiceBuilding;

/// Groundwater tooltip: shows per-cell groundwater level, quality, extraction
/// rate, and recharge rate when a groundwater overlay is active and the cursor
/// is over a valid cell.
#[allow(clippy::too_many_arguments)]
pub fn groundwater_tooltip_ui(
    mut contexts: EguiContexts,
    overlay: Res<OverlayState>,
    cursor: Res<rendering::input::CursorGridPos>,
    groundwater: Res<simulation::groundwater::GroundwaterGrid>,
    water_quality: Res<simulation::groundwater::WaterQualityGrid>,
    depletion: Res<simulation::groundwater_depletion::GroundwaterDepletionState>,
    services: Query<&ServiceBuilding>,
) {
    // Only show when a groundwater overlay is active
    if overlay.mode != OverlayMode::GroundwaterLevel
        && overlay.mode != OverlayMode::GroundwaterQuality
    {
        return;
    }

    if !cursor.valid {
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;
    if gx >= GRID_WIDTH || gy >= GRID_HEIGHT {
        return;
    }

    let level = groundwater.get(gx, gy);
    let quality = water_quality.get(gx, gy);
    let level_pct = level as f32 / 255.0 * 100.0;
    let quality_pct = quality as f32 / 255.0 * 100.0;

    // Check if there is a well pump at or near this cell
    let mut nearby_well = false;
    for service in &services {
        if service.service_type == simulation::services::ServiceType::WellPump {
            let dx = (service.grid_x as i32 - gx as i32).abs();
            let dy = (service.grid_y as i32 - gy as i32).abs();
            if dx <= 1 && dy <= 1 {
                nearby_well = true;
                break;
            }
        }
    }

    egui::Window::new("Groundwater Info")
        .fixed_pos(egui::pos2(
            contexts.ctx_mut().screen_rect().max.x - 240.0,
            contexts.ctx_mut().screen_rect().max.y - 180.0,
        ))
        .auto_sized()
        .title_bar(true)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Cell ({}, {})", gx, gy));
            ui.separator();

            egui::Grid::new("gw_tooltip_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Level:");
                    let level_color = if level < 76 {
                        egui::Color32::from_rgb(220, 80, 50)
                    } else if level < 128 {
                        egui::Color32::from_rgb(220, 180, 50)
                    } else {
                        egui::Color32::from_rgb(50, 180, 220)
                    };
                    ui.colored_label(level_color, format!("{}/255 ({:.0}%)", level, level_pct));
                    ui.end_row();

                    ui.label("Quality:");
                    let quality_color = if quality < 50 {
                        egui::Color32::from_rgb(200, 50, 50)
                    } else if quality < 128 {
                        egui::Color32::from_rgb(200, 150, 50)
                    } else {
                        egui::Color32::from_rgb(50, 200, 80)
                    };
                    ui.colored_label(
                        quality_color,
                        format!("{}/255 ({:.0}%)", quality, quality_pct),
                    );
                    ui.end_row();

                    ui.label("Extraction:");
                    ui.label(format!("{:.1} units/tick", depletion.extraction_rate));
                    ui.end_row();

                    ui.label("Recharge:");
                    ui.label(format!("{:.1} units/tick", depletion.recharge_rate));
                    ui.end_row();

                    if nearby_well {
                        ui.label("Well:");
                        ui.colored_label(
                            egui::Color32::from_rgb(100, 200, 255),
                            "Well pump nearby",
                        );
                        ui.end_row();
                    }

                    if level < 76 {
                        ui.label("Warning:");
                        ui.colored_label(egui::Color32::from_rgb(255, 100, 50), "Depletion risk!");
                        ui.end_row();
                    }
                });
        });
}
