use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::advisors::AdvisorPanel;

use super::AdvisorVisible;

/// Displays the City Advisors as a standalone egui window.
/// Toggle visibility with the 'A' key.
pub fn advisor_window_ui(
    mut contexts: EguiContexts,
    advisor_panel: Res<AdvisorPanel>,
    visible: Res<AdvisorVisible>,
) {
    if !visible.0 {
        return;
    }

    egui::Window::new("City Advisors")
        .default_open(true)
        .default_width(350.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Press [A] to toggle");
            ui.separator();

            let messages = &advisor_panel.messages;
            if messages.is_empty() {
                ui.label("No advisor messages at this time.");
            } else {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for msg in messages {
                            let priority_color = match msg.priority {
                                5 => egui::Color32::from_rgb(220, 50, 50),   // red
                                4 => egui::Color32::from_rgb(230, 150, 30),  // orange
                                3 => egui::Color32::from_rgb(220, 200, 50),  // yellow
                                2 => egui::Color32::from_rgb(50, 130, 220),  // blue
                                _ => egui::Color32::from_rgb(150, 150, 150), // grey
                            };

                            ui.horizontal(|ui| {
                                let (dot_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(10.0, 10.0),
                                    egui::Sense::hover(),
                                );
                                let painter = ui.painter_at(dot_rect);
                                painter.circle_filled(dot_rect.center(), 4.0, priority_color);

                                ui.colored_label(priority_color, msg.advisor_type.name());
                            });

                            ui.label(&msg.message);
                            ui.small(&msg.suggestion);
                            ui.add_space(4.0);
                        }
                    });
            }
        });
}
