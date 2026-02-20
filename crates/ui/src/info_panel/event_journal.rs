use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::events::{ActiveCityEffects, EventJournal};

use super::JournalVisible;

/// Returns a color associated with a given event type for visual distinction.
fn event_type_color(event_type: &simulation::events::CityEventType) -> egui::Color32 {
    use simulation::events::CityEventType;
    match event_type {
        CityEventType::MilestoneReached(_) => egui::Color32::from_rgb(50, 200, 255),
        CityEventType::BuildingFire(_, _) => egui::Color32::from_rgb(255, 100, 30),
        CityEventType::DisasterStrike(_) => egui::Color32::from_rgb(255, 50, 50),
        CityEventType::NewPolicy(_) => egui::Color32::from_rgb(100, 200, 100),
        CityEventType::BudgetCrisis => egui::Color32::from_rgb(255, 50, 50),
        CityEventType::PopulationBoom => egui::Color32::from_rgb(50, 200, 255),
        CityEventType::Epidemic => egui::Color32::from_rgb(200, 50, 200),
        CityEventType::Festival => egui::Color32::from_rgb(255, 215, 0),
        CityEventType::EconomicBoom => egui::Color32::from_rgb(50, 220, 50),
        CityEventType::ResourceDepleted(_) => egui::Color32::from_rgb(200, 150, 50),
    }
}

/// Displays the Event Journal as a collapsible egui window.
/// Shows the last 10 events with day/hour and description.
/// Also shows active effects status.
pub fn event_journal_ui(
    mut contexts: EguiContexts,
    journal: Res<EventJournal>,
    effects: Res<ActiveCityEffects>,
    visible: Res<JournalVisible>,
) {
    if !visible.0 {
        return;
    }

    egui::Window::new("Event Journal")
        .default_open(true)
        .default_width(350.0)
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(8.0, -8.0))
        .show(contexts.ctx_mut(), |ui| {
            // Active effects section
            let has_effects = effects.festival_ticks > 0
                || effects.economic_boom_ticks > 0
                || effects.epidemic_ticks > 0;

            if has_effects {
                ui.heading("Active Effects");
                if effects.festival_ticks > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 215, 0),
                        format!("Festival ({} ticks remaining)", effects.festival_ticks),
                    );
                }
                if effects.economic_boom_ticks > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(50, 200, 50),
                        format!(
                            "Economic Boom ({} ticks remaining)",
                            effects.economic_boom_ticks
                        ),
                    );
                }
                if effects.epidemic_ticks > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 50, 50),
                        format!("Epidemic ({} ticks remaining)", effects.epidemic_ticks),
                    );
                }
                ui.separator();
            }

            // Recent events
            ui.heading("Recent Events");
            ui.small("Press [J] to toggle");
            ui.separator();

            if journal.events.is_empty() {
                ui.label("No events recorded yet.");
            } else {
                // Show last 10 events, most recent first
                let start = journal.events.len().saturating_sub(10);
                let recent = &journal.events[start..];

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for event in recent.iter().rev() {
                            let h = event.hour as u32;
                            let m = ((event.hour - h as f32) * 60.0) as u32;
                            let time_str = format!("Day {} {:02}:{:02}", event.day, h, m);

                            let event_color = event_type_color(&event.event_type);

                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::from_rgb(150, 150, 150), &time_str);
                                ui.colored_label(event_color, &event.description);
                            });
                            ui.add_space(2.0);
                        }
                    });
            }
        });
}
