use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::time_of_day::GameClock;
use simulation::tutorial::{TutorialState, TutorialStep};

/// Renders the tutorial overlay window when the tutorial is active.
pub fn tutorial_ui(
    mut contexts: EguiContexts,
    mut tutorial: ResMut<TutorialState>,
    mut clock: ResMut<GameClock>,
) {
    if !tutorial.active {
        return;
    }

    let step = tutorial.current_step;
    let step_index = step.index();
    let total = TutorialStep::total_steps();

    let ctx = contexts.ctx_mut();

    // Semi-transparent highlight overlay to draw attention to the tutorial window
    egui::Window::new("Tutorial")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(420.0)
        .show(ctx, |ui| {
            // Progress indicator
            if step != TutorialStep::Completed {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Step {}/{}", step_index, total))
                            .small()
                            .color(egui::Color32::from_rgb(160, 160, 160)),
                    );

                    // Progress bar
                    let progress = step_index as f32 / total as f32;
                    ui.add(
                        egui::ProgressBar::new(progress)
                            .desired_width(200.0)
                            .show_percentage(),
                    );
                });
                ui.separator();
            }

            // Step title
            ui.heading(
                egui::RichText::new(step.title())
                    .strong()
                    .color(egui::Color32::from_rgb(100, 200, 255)),
            );
            ui.add_space(8.0);

            // Step description
            ui.label(
                egui::RichText::new(step.description())
                    .color(egui::Color32::from_rgb(220, 220, 220)),
            );
            ui.add_space(8.0);

            // Hint text
            ui.label(
                egui::RichText::new(step.hint())
                    .italics()
                    .color(egui::Color32::from_rgb(180, 200, 140)),
            );
            ui.add_space(12.0);

            // Action buttons
            ui.horizontal(|ui| {
                // Next button for manual steps
                if tutorial.is_manual_step() {
                    let button_text = if step == TutorialStep::Completed {
                        "Close"
                    } else {
                        "Next"
                    };

                    if ui
                        .button(
                            egui::RichText::new(button_text)
                                .strong()
                                .color(egui::Color32::from_rgb(100, 255, 100)),
                        )
                        .clicked()
                    {
                        if step == TutorialStep::Completed {
                            tutorial.active = false;
                        } else {
                            // Unpause if we paused for this step
                            if tutorial.paused_by_tutorial {
                                clock.paused = false;
                                tutorial.paused_by_tutorial = false;
                            }
                            tutorial.advance();
                        }
                    }
                } else {
                    // For auto-advancing steps, show "waiting" status
                    ui.label(
                        egui::RichText::new("Waiting for action...")
                            .color(egui::Color32::from_rgb(200, 180, 100)),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Skip button (always available except when completed)
                    if step != TutorialStep::Completed
                        && ui
                            .button(
                                egui::RichText::new("Skip Tutorial")
                                    .color(egui::Color32::from_rgb(180, 180, 180)),
                            )
                            .clicked()
                    {
                        // Unpause if tutorial paused the sim
                        if tutorial.paused_by_tutorial {
                            clock.paused = false;
                        }
                        tutorial.skip();
                    }
                });
            });
        });
}
