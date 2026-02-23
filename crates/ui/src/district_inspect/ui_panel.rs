//! Egui rendering for the district inspection panel.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::helpers::{happiness_color, happiness_label};
use super::resources::DistrictInspectCache;

/// System that renders the District Inspection Panel using egui.
pub fn district_inspect_ui(mut contexts: EguiContexts, cache: Res<DistrictInspectCache>) {
    if !cache.valid {
        return;
    }

    egui::Window::new("District Info")
        .default_width(260.0)
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 80.0))
        .show(contexts.ctx_mut(), |ui| {
            // District name with color swatch
            ui.horizontal(|ui| {
                let c = cache.color;
                let swatch_color = egui::Color32::from_rgba_unmultiplied(
                    (c[0] * 255.0) as u8,
                    (c[1] * 255.0) as u8,
                    (c[2] * 255.0) as u8,
                    255,
                );
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, egui::CornerRadius::same(3), swatch_color);
                ui.heading(&cache.name);
            });
            ui.separator();

            // Overview stats
            egui::Grid::new("district_overview")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Population:");
                    ui.label(format!("{}", cache.population));
                    ui.end_row();

                    ui.label("Happiness:");
                    let h_color = happiness_color(cache.avg_happiness);
                    ui.colored_label(
                        h_color,
                        format!(
                            "{:.0}% ({})",
                            cache.avg_happiness,
                            happiness_label(cache.avg_happiness)
                        ),
                    );
                    ui.end_row();

                    ui.label("Cells:");
                    ui.label(format!("{}", cache.cell_count));
                    ui.end_row();
                });

            ui.separator();
            ui.heading("Jobs");
            egui::Grid::new("district_jobs")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Commercial:");
                    ui.label(format!("{}", cache.commercial_jobs));
                    ui.end_row();

                    ui.label("Industrial:");
                    ui.label(format!("{}", cache.industrial_jobs));
                    ui.end_row();

                    ui.label("Office:");
                    ui.label(format!("{}", cache.office_jobs));
                    ui.end_row();
                });

            ui.separator();
            ui.heading("Service Coverage");
            egui::Grid::new("district_services")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    service_row(ui, "Fire", cache.fire_services);
                    service_row(ui, "Police", cache.police_services);
                    service_row(ui, "Health", cache.health_services);
                    service_row(ui, "Education", cache.education_services);
                    service_row(ui, "Parks", cache.park_services);
                    service_row(ui, "Transport", cache.transport_services);
                });
        });
}

/// Helper to render a service coverage row with color indicator.
fn service_row(ui: &mut egui::Ui, label: &str, count: u32) {
    ui.label(format!("{}:", label));
    let color = if count >= 2 {
        egui::Color32::from_rgb(50, 200, 50) // well covered
    } else if count == 1 {
        egui::Color32::from_rgb(220, 220, 50) // minimal
    } else {
        egui::Color32::from_rgb(220, 50, 50) // none
    };
    ui.colored_label(color, format!("{}", count));
    ui.end_row();
}
