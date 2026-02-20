use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::oneway::{OneWayDirection, OneWayDirectionMap, ToggleOneWayEvent};
use simulation::road_segments::{RoadSegmentStore, SegmentId};

/// Currently selected road segment for the context menu.
#[derive(Resource, Default)]
pub struct SelectedSegment(pub Option<SegmentId>);

/// Road segment context menu UI.
///
/// When a segment is selected, shows a small window with one-way toggle controls.
pub fn road_segment_context_menu(
    mut contexts: EguiContexts,
    mut selected: ResMut<SelectedSegment>,
    store: Res<RoadSegmentStore>,
    oneway_map: Res<OneWayDirectionMap>,
    mut toggle_events: EventWriter<ToggleOneWayEvent>,
) {
    let Some(seg_id) = selected.0 else {
        return;
    };

    let Some(segment) = store.get_segment(seg_id) else {
        // Segment was removed, clear selection
        selected.0 = None;
        return;
    };

    let current_direction = oneway_map.get(seg_id);

    let direction_label = match current_direction {
        None => "Two-Way",
        Some(OneWayDirection::Forward) => "One-Way \u{2192}",
        Some(OneWayDirection::Reverse) => "One-Way \u{2190}",
    };

    let road_type = format!("{:?}", segment.road_type);
    let arc_length = segment.arc_length;

    let mut open = true;
    egui::Window::new("Road Segment")
        .open(&mut open)
        .default_width(220.0)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 40.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Road Properties");
            ui.separator();

            egui::Grid::new("road_segment_props")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Type:");
                    ui.label(&road_type);
                    ui.end_row();

                    ui.label("Length:");
                    ui.label(format!("{:.0}m", arc_length));
                    ui.end_row();

                    ui.label("Direction:");
                    ui.label(direction_label);
                    ui.end_row();
                });

            ui.separator();

            ui.horizontal(|ui| {
                if ui
                    .button("Toggle Direction")
                    .on_hover_text(
                        "Cycle: Two-Way \u{2192} One-Way Forward \u{2192} One-Way Reverse \u{2192} Two-Way",
                    )
                    .clicked()
                {
                    toggle_events.send(ToggleOneWayEvent {
                        segment_id: seg_id,
                    });
                }

                let status_color = if current_direction.is_some() {
                    egui::Color32::from_rgb(50, 200, 100)
                } else {
                    egui::Color32::from_rgb(150, 150, 150)
                };
                ui.colored_label(status_color, direction_label);
            });
        });

    if !open {
        selected.0 = None;
    }
}

/// Detect right-clicks on road segments to open the context menu.
pub fn select_road_segment_on_click(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<rendering::input::CursorGridPos>,
    store: Res<RoadSegmentStore>,
    mut selected: ResMut<SelectedSegment>,
) {
    if !buttons.just_pressed(MouseButton::Right) || !cursor.valid {
        return;
    }

    // Find which segment the cursor is over by checking rasterized cells
    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    for segment in &store.segments {
        if segment.rasterized_cells.contains(&(gx, gy)) {
            selected.0 = Some(segment.id);
            return;
        }
    }

    // No segment under cursor, clear selection
    selected.0 = None;
}

pub struct OneWayUiPlugin;

impl Plugin for OneWayUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSegment>().add_systems(
            Update,
            (select_road_segment_on_click, road_segment_context_menu).chain(),
        );
    }
}
