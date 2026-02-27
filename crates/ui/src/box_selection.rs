//! Box Selection UI (UX-011).
//!
//! Shows a persistent indicator in the top info bar when a box selection
//! drag is in progress, displaying the current selection dimensions.
//! Also shows the selected entity count after a box selection completes.

use bevy::prelude::*;
use simulation::app_state::AppState;
use bevy_egui::{egui, EguiContexts};

use rendering::box_selection::BoxSelectionState;
use simulation::multi_select::MultiSelectState;

// =============================================================================
// Systems
// =============================================================================

/// Show a floating indicator during an active box selection drag.
pub fn box_selection_indicator_ui(
    mut contexts: EguiContexts,
    box_state: Res<BoxSelectionState>,
    multi_select: Res<MultiSelectState>,
) {
    // During active box drag, show dimensions
    if box_state.active {
        let size = box_state.size();
        let cells_x = (size.x / simulation::config::CELL_SIZE).ceil() as usize;
        let cells_y = (size.y / simulation::config::CELL_SIZE).ceil() as usize;

        egui::Area::new(egui::Id::new("box_selection_indicator"))
            .fixed_pos(egui::pos2(
                contexts.ctx_mut().screen_rect().center().x - 80.0,
                42.0,
            ))
            .show(contexts.ctx_mut(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(20, 40, 60, 220))
                    .show(ui, |ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(100, 200, 255),
                            format!("Box Select: {}x{} cells", cells_x, cells_y),
                        );
                    });
            });
    }

    // When not dragging but items are selected, show count in a subtle indicator
    // (The multi_select_panel_ui already shows a full panel, so we just add
    // a compact indicator near the status bar area)
    if !box_state.active && !multi_select.is_empty() {
        let count = multi_select.count();
        let buildings = multi_select.building_count();
        let roads = multi_select.road_count();

        let label = if buildings > 0 && roads > 0 {
            format!("Selected: {} ({} bldg, {} road)", count, buildings, roads)
        } else if buildings > 0 {
            format!("Selected: {} building(s)", buildings)
        } else if roads > 0 {
            format!("Selected: {} road cell(s)", roads)
        } else {
            format!("Selected: {}", count)
        };

        egui::Area::new(egui::Id::new("box_selection_count"))
            .fixed_pos(egui::pos2(
                contexts.ctx_mut().screen_rect().right() - 250.0,
                42.0,
            ))
            .show(contexts.ctx_mut(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(20, 40, 60, 200))
                    .show(ui, |ui| {
                        ui.colored_label(egui::Color32::from_rgb(140, 220, 255), label);
                    });
            });
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct BoxSelectionUiPlugin;

impl Plugin for BoxSelectionUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, box_selection_indicator_ui.run_if(in_state(AppState::Playing)));
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_selection_state_default() {
        let state = BoxSelectionState::default();
        assert!(!state.active);
    }
}
