use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::input::{ActiveTool, CursorGridPos, DrawPhase, RoadDrawState};
use simulation::config::CELL_SIZE;
use simulation::economy::CityBudget;

/// Shows estimated road cost near the cursor during road placement.
/// Green text if affordable, red text if over budget.
pub fn road_cost_display_ui(
    mut contexts: EguiContexts,
    tool: Res<ActiveTool>,
    draw_state: Res<RoadDrawState>,
    cursor: Res<CursorGridPos>,
    budget: Res<CityBudget>,
) {
    // Only display when actively drawing a road (start placed, waiting for end)
    if draw_state.phase != DrawPhase::PlacedStart {
        return;
    }

    // Only for road tools
    let Some(road_type) = tool.road_type() else {
        return;
    };

    if !cursor.valid {
        return;
    }

    // Calculate segment length and cost
    let start = draw_state.start_pos;
    let end = cursor.world_pos;
    let length = (end - start).length();

    // Minimum length check - don't show for very short segments
    if length < CELL_SIZE * 0.5 {
        return;
    }

    let approx_cells = (length / CELL_SIZE).ceil() as usize;
    let cost_per_cell = road_type.cost();
    let total_cost = cost_per_cell * approx_cells as f64;
    let affordable = budget.treasury >= total_cost;

    // Get cursor screen position from egui
    let ctx = contexts.ctx_mut();
    let Some(pointer_pos) = ctx.pointer_hover_pos() else {
        return;
    };

    // Position the label slightly offset from cursor (below-right)
    let offset = egui::vec2(16.0, 20.0);
    let label_pos = pointer_pos + offset;

    let color = if affordable {
        egui::Color32::from_rgb(80, 220, 80) // green
    } else {
        egui::Color32::from_rgb(220, 60, 60) // red
    };

    let cost_text = format!("${:.0}", total_cost);

    egui::Area::new(egui::Id::new("road_cost_display"))
        .fixed_pos(egui::pos2(label_pos.x, label_pos.y))
        .interactable(false)
        .order(egui::Order::Tooltip)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_rgba_premultiplied(20, 20, 20, 200))
                .show(ui, |ui| {
                    ui.colored_label(color, egui::RichText::new(cost_text).strong().size(14.0));
                });
        });
}
