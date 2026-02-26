use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::time_of_day::GameClock;
use simulation::tutorial::{TutorialState, TutorialStep};
use simulation::tutorial_hints::TutorialUiHint;

/// Renders the tutorial overlay window when the tutorial is active.
///
/// Features:
/// - Progress indicator with step counter and progress bar
/// - Styled step title, description, and hint text
/// - Pulsing toolbar highlight indicator (e.g. ">>> Roads <<<")
/// - Back / Next / Skip buttons
/// - Rounded corner frame styling with polished colors
#[allow(clippy::too_many_arguments)]
pub fn tutorial_ui(
    mut contexts: EguiContexts,
    mut tutorial: ResMut<TutorialState>,
    mut clock: ResMut<GameClock>,
    hint: Res<TutorialUiHint>,
    time: Res<Time>,
) {
    if !tutorial.active {
        return;
    }

    let step = tutorial.current_step;
    let step_index = step.index();
    let total = TutorialStep::total_steps();
    let elapsed = time.elapsed_secs_f64();

    let ctx = contexts.ctx_mut();

    // Custom frame with rounded corners and polished styling
    let frame = egui::Frame::new()
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(16))
        .fill(egui::Color32::from_rgba_unmultiplied(25, 30, 45, 240))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 70, 100)));

    egui::Window::new("Tutorial")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(440.0)
        .frame(frame)
        .title_bar(false)
        .show(ctx, |ui| {
            render_progress(ui, step, step_index, total);
            render_title(ui, step);
            render_description(ui, step);
            render_hint(ui, step);
            render_highlight_indicator(ui, &hint, elapsed);
            ui.add_space(12.0);
            render_buttons(ui, &mut tutorial, &mut clock, step);
        });
}

/// Draw the progress bar and step counter.
fn render_progress(ui: &mut egui::Ui, step: TutorialStep, step_index: usize, total: usize) {
    if step != TutorialStep::Completed {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("Step {}/{}", step_index, total))
                    .small()
                    .color(egui::Color32::from_rgb(160, 160, 180)),
            );

            let progress = step_index as f32 / total as f32;
            ui.add(
                egui::ProgressBar::new(progress)
                    .desired_width(200.0)
                    .show_percentage(),
            );
        });
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);
    }
}

/// Draw the step title.
fn render_title(ui: &mut egui::Ui, step: TutorialStep) {
    ui.heading(
        egui::RichText::new(step.title())
            .strong()
            .size(18.0)
            .color(egui::Color32::from_rgb(100, 200, 255)),
    );
    ui.add_space(8.0);
}

/// Draw the step description text.
fn render_description(ui: &mut egui::Ui, step: TutorialStep) {
    ui.label(
        egui::RichText::new(step.description())
            .size(14.0)
            .color(egui::Color32::from_rgb(220, 220, 230)),
    );
    ui.add_space(8.0);
}

/// Draw the step hint text.
fn render_hint(ui: &mut egui::Ui, step: TutorialStep) {
    ui.label(
        egui::RichText::new(step.hint())
            .italics()
            .size(13.0)
            .color(egui::Color32::from_rgb(180, 210, 140)),
    );
}

/// Draw a pulsing indicator pointing to the relevant toolbar category.
fn render_highlight_indicator(ui: &mut egui::Ui, hint: &TutorialUiHint, elapsed: f64) {
    if let Some(target) = hint.highlight_target {
        ui.add_space(8.0);

        // Pulsing alpha based on elapsed time
        let pulse = ((elapsed * 3.0).sin() * 0.5 + 0.5) as f32;
        let alpha = (140.0 + pulse * 115.0) as u8;
        let color = egui::Color32::from_rgba_unmultiplied(255, 200, 60, alpha);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!(">>> {} <<<", target)).strong().size(15.0).color(color));
        });
    }
}

/// Draw the Back / Next / Skip buttons.
fn render_buttons(
    ui: &mut egui::Ui,
    tutorial: &mut ResMut<TutorialState>,
    clock: &mut ResMut<GameClock>,
    step: TutorialStep,
) {
    ui.horizontal(|ui| {
        // Back button (disabled on Welcome step)
        let can_go_back = step != TutorialStep::Welcome
            && step != TutorialStep::Completed
            && !tutorial.completed;

        if ui
            .add_enabled(
                can_go_back,
                egui::Button::new(
                    egui::RichText::new("Back")
                        .color(egui::Color32::from_rgb(180, 180, 200)),
                ),
            )
            .clicked()
        {
            tutorial.go_back();
        }

        // Next / Close button for manual steps
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
                if tutorial.paused_by_tutorial {
                    clock.paused = false;
                }
                tutorial.skip();
            }
        });
    });
}
