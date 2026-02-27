//! UI-015: Crash Recovery Prompt.
//!
//! On startup, checks if `CrashRecoveryState::detected` is true (meaning the
//! previous session ended abnormally). If a valid autosave is available for
//! recovery, displays a modal prompt offering the player to restore it or
//! start fresh.
//!
//! The prompt only appears once per session and auto-dismisses if the player
//! has already started playing.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use save::{CrashRecoveryState, LoadGameEvent, PendingSavePath};
use simulation::app_state::AppState;

// =============================================================================
// Resources
// =============================================================================

/// Tracks whether the crash recovery prompt has been shown/dismissed.
#[derive(Resource, Default)]
struct CrashRecoveryUiState {
    /// Whether the prompt has been dismissed (either by choosing an action
    /// or clicking "Ignore").
    dismissed: bool,
}

// =============================================================================
// Plugin
// =============================================================================

pub struct CrashRecoveryUiPlugin;

impl Plugin for CrashRecoveryUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CrashRecoveryUiState>();
        app.add_systems(
            Update,
            crash_recovery_prompt.run_if(in_state(AppState::MainMenu)),
        );
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Displays the crash recovery prompt when crash artifacts were detected.
fn crash_recovery_prompt(
    mut contexts: EguiContexts,
    crash_state: Res<CrashRecoveryState>,
    mut ui_state: ResMut<CrashRecoveryUiState>,
    mut load_game_events: EventWriter<LoadGameEvent>,
    mut pending_path: ResMut<PendingSavePath>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    // Skip if already dismissed or no crash detected
    if ui_state.dismissed || !crash_state.detected {
        return;
    }

    // Only show if we have a recovery path
    let recovery_path = match &crash_state.recovery_path {
        Some(path) => path.clone(),
        None => {
            // Crash detected but no valid autosave — nothing to offer.
            // Auto-dismiss so the prompt doesn't block the main menu.
            ui_state.dismissed = true;
            return;
        }
    };

    let ctx = contexts.ctx_mut();

    // Semi-transparent overlay behind the modal
    let screen_rect = ctx.screen_rect();
    egui::Area::new(egui::Id::new("crash_recovery_overlay"))
        .fixed_pos(screen_rect.min)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            painter.rect_filled(
                screen_rect,
                egui::CornerRadius::ZERO,
                egui::Color32::from_black_alpha(180),
            );
            ui.allocate_rect(screen_rect, egui::Sense::hover());
        });

    egui::Window::new("Session Recovery")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(400.0)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);

                // Warning icon (text-based)
                ui.label(
                    egui::RichText::new("Session Recovery")
                        .size(24.0)
                        .strong()
                        .color(crate::theme::WARNING),
                );

                ui.add_space(12.0);

                ui.label(
                    egui::RichText::new(
                        "It looks like your previous session ended unexpectedly.",
                    )
                    .size(crate::theme::FONT_BODY)
                    .color(crate::theme::TEXT),
                );

                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new(
                        "An autosave was found that may contain your recent progress.",
                    )
                    .size(crate::theme::FONT_BODY)
                    .color(crate::theme::TEXT_MUTED),
                );

                // Show recovery details
                if crash_state.corrupted_slots > 0 {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "({} corrupted save(s) were skipped)",
                            crash_state.corrupted_slots
                        ))
                        .size(crate::theme::FONT_SMALL)
                        .color(crate::theme::TEXT_MUTED),
                    );
                }

                ui.add_space(20.0);

                let button_size = egui::vec2(180.0, 36.0);

                // Restore button
                if ui
                    .add_sized(
                        button_size,
                        egui::Button::new(
                            egui::RichText::new("Restore Session")
                                .size(16.0)
                                .color(crate::theme::TEXT_HEADING),
                        ),
                    )
                    .clicked()
                {
                    pending_path.0 =
                        Some(recovery_path.to_string_lossy().to_string());
                    load_game_events.send(LoadGameEvent);
                    next_app_state.set(AppState::Playing);
                    ui_state.dismissed = true;
                }

                ui.add_space(8.0);

                // Ignore button
                if ui
                    .add_sized(
                        button_size,
                        egui::Button::new(
                            egui::RichText::new("Ignore")
                                .size(14.0)
                                .color(crate::theme::TEXT_MUTED),
                        ),
                    )
                    .clicked()
                {
                    ui_state.dismissed = true;
                }

                ui.add_space(12.0);
            });
        });
}
