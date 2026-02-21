//! Auto-Grid Road Tool UI (TRAF-010)
//!
//! Shows a floating config panel when the auto-grid tool is active,
//! allowing the player to adjust block size and road type.
//! Also shows cost preview near the cursor when placing the second corner.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::input::{ActiveTool, CursorGridPos};
use simulation::auto_grid_road::{
    self, AutoGridConfig, AutoGridPhase, AutoGridState, MAX_BLOCK_SIZE, MIN_BLOCK_SIZE,
};
use simulation::economy::CityBudget;
use simulation::grid::{RoadType, WorldGrid};

/// Display the auto-grid configuration panel and cost preview.
#[allow(clippy::too_many_arguments)]
pub fn auto_grid_config_ui(
    mut contexts: EguiContexts,
    tool: Res<ActiveTool>,
    mut config: ResMut<AutoGridConfig>,
    state: Res<AutoGridState>,
    cursor: Res<CursorGridPos>,
    grid: Res<WorldGrid>,
    budget: Res<CityBudget>,
) {
    if *tool != ActiveTool::AutoGrid {
        return;
    }

    let ctx = contexts.ctx_mut();

    // Configuration panel (fixed position, bottom-left area)
    egui::Window::new("Auto-Grid Settings")
        .id(egui::Id::new("auto_grid_config"))
        .fixed_pos(egui::pos2(10.0, 500.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(true)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("Road Grid Tool").strong().size(14.0));
            ui.add_space(4.0);

            // Block size slider
            let mut block_size = config.block_size as i32;
            ui.horizontal(|ui| {
                ui.label("Block size:");
                ui.add(
                    egui::Slider::new(
                        &mut block_size,
                        MIN_BLOCK_SIZE as i32..=MAX_BLOCK_SIZE as i32,
                    )
                    .suffix(" cells"),
                );
            });
            config.block_size =
                block_size.clamp(MIN_BLOCK_SIZE as i32, MAX_BLOCK_SIZE as i32) as u8;

            // Road type selector
            ui.horizontal(|ui| {
                ui.label("Road type:");
                egui::ComboBox::from_id_salt("auto_grid_road_type")
                    .selected_text(road_type_label(config.road_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut config.road_type, RoadType::Local, "Local ($10)");
                        ui.selectable_value(
                            &mut config.road_type,
                            RoadType::Avenue,
                            "Avenue ($20)",
                        );
                        ui.selectable_value(
                            &mut config.road_type,
                            RoadType::Boulevard,
                            "Boulevard ($30)",
                        );
                    });
            });

            ui.add_space(4.0);

            // Instructions
            match state.phase {
                AutoGridPhase::Idle => {
                    ui.label("Click to place first corner");
                }
                AutoGridPhase::PlacedFirstCorner => {
                    ui.label("Click second corner to place grid");
                    ui.label("Right-click or Esc to cancel");
                }
            }
        });

    // Cost preview near cursor when placing second corner
    if state.phase == AutoGridPhase::PlacedFirstCorner && cursor.valid {
        let corner2 = (cursor.grid_x as usize, cursor.grid_y as usize);
        let plan = auto_grid_road::compute_grid_plan(state.corner1, corner2, &config, &grid);

        if let Some(pointer_pos) = ctx.pointer_hover_pos() {
            let offset = egui::vec2(16.0, -40.0);
            let label_pos = pointer_pos + offset;

            egui::Area::new(egui::Id::new("auto_grid_cost_preview"))
                .fixed_pos(egui::pos2(label_pos.x, label_pos.y))
                .interactable(false)
                .order(egui::Order::Tooltip)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style())
                        .fill(egui::Color32::from_rgba_premultiplied(20, 20, 20, 200))
                        .show(ui, |ui| {
                            let affordable = budget.treasury >= plan.total_cost;
                            let cost_color = if affordable {
                                egui::Color32::from_rgb(80, 220, 80)
                            } else {
                                egui::Color32::from_rgb(220, 60, 60)
                            };

                            ui.label(
                                egui::RichText::new(format!(
                                    "{} cells  ${:.0}",
                                    plan.total_cells, plan.total_cost
                                ))
                                .color(cost_color)
                                .strong()
                                .size(13.0),
                            );

                            if !affordable {
                                ui.label(
                                    egui::RichText::new("Not enough money!")
                                        .color(egui::Color32::from_rgb(220, 60, 60))
                                        .size(11.0),
                                );
                            }
                        });
                });
        }
    }
}

fn road_type_label(rt: RoadType) -> &'static str {
    match rt {
        RoadType::Local => "Local",
        RoadType::Avenue => "Avenue",
        RoadType::Boulevard => "Boulevard",
        RoadType::Highway => "Highway",
        RoadType::OneWay => "One-Way",
        RoadType::Path => "Path",
    }
}

pub struct AutoGridUiPlugin;

impl Plugin for AutoGridUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, auto_grid_config_ui);
    }
}
