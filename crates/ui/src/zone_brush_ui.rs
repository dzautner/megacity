use bevy::prelude::*;
use simulation::app_state::AppState;
use bevy_egui::{egui, EguiContexts};

use rendering::input::{ActiveTool, CursorGridPos};
use rendering::zone_brush_preview::{
    brush_cells, is_cell_valid_for_zone, ZoneBrushSize, ZONE_COST_PER_CELL,
};
use simulation::grid::WorldGrid;
use simulation::urban_growth_boundary::UrbanGrowthBoundary;

/// Display zone brush info and total cost near the cursor via egui.
pub fn zone_brush_cost_ui(
    mut contexts: EguiContexts,
    tool: Res<ActiveTool>,
    brush: Res<ZoneBrushSize>,
    cursor: Res<CursorGridPos>,
    grid: Res<WorldGrid>,
    ugb: Res<UrbanGrowthBoundary>,
) {
    let Some(zone) = tool.zone_type() else {
        return;
    };
    if !cursor.valid {
        return;
    }

    let cells = brush_cells(cursor.grid_x, cursor.grid_y, brush.half_extent, &grid);
    let valid_count = cells
        .iter()
        .filter(|(gx, gy)| is_cell_valid_for_zone(&grid, *gx, *gy, zone, &ugb))
        .count();
    let total_cost = valid_count as f64 * ZONE_COST_PER_CELL;

    let ctx = contexts.ctx_mut();
    let Some(pointer_pos) = ctx.pointer_hover_pos() else {
        return;
    };

    let offset = egui::vec2(16.0, -40.0);
    let label_pos = pointer_pos + offset;

    egui::Area::new(egui::Id::new("zone_brush_info"))
        .fixed_pos(egui::pos2(label_pos.x, label_pos.y))
        .interactable(false)
        .order(egui::Order::Tooltip)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_rgba_premultiplied(20, 20, 20, 200))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("Brush: {}", brush.label()))
                            .color(egui::Color32::WHITE)
                            .size(12.0),
                    );
                    if valid_count > 0 {
                        let cost_text = format!("{} cells  ${:.0}", valid_count, total_cost);
                        ui.label(
                            egui::RichText::new(cost_text)
                                .color(egui::Color32::from_rgb(80, 220, 80))
                                .strong()
                                .size(13.0),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("No valid cells")
                                .color(egui::Color32::from_rgb(220, 60, 60))
                                .size(12.0),
                        );
                    }
                    if brush.half_extent < 2 {
                        ui.label(
                            egui::RichText::new("[/] resize brush")
                                .color(egui::Color32::from_rgb(160, 160, 160))
                                .size(10.0),
                        );
                    }
                });
        });
}

pub struct ZoneBrushUiPlugin;

impl Plugin for ZoneBrushUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, zone_brush_cost_ui.run_if(in_state(AppState::Playing)));
    }
}
