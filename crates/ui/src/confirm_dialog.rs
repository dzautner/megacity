//! Confirmation dialogs for destructive actions during active gameplay.
//!
//! When the player clicks "New Game" or presses Ctrl+N while playing, a
//! confirmation dialog appears instead of immediately discarding the current
//! city. Quick-load (F9) also shows confirmation before replacing the city.
//!
//! This module exposes a [`PendingConfirmAction`] resource that other systems
//! write to in order to request confirmation. The dialog system reads this
//! resource and, upon confirmation, fires the appropriate save-crate event.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use save::{LoadGameEvent, NewGameEvent};
use simulation::app_state::AppState;
use simulation::PreLoadAppState;

// =============================================================================
// Types
// =============================================================================

/// The destructive action awaiting player confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Start a brand-new game, discarding the current city.
    NewGame,
    /// Quick-load a saved game, replacing the current city.
    QuickLoad,
}

/// Resource that holds a pending destructive action requiring confirmation.
///
/// Other UI systems set this to `Some(action)` instead of directly firing
/// the destructive event. The [`confirm_dialog_ui`] system renders the dialog
/// and, on confirmation, fires the event and clears this resource.
#[derive(Resource, Default)]
pub struct PendingConfirmAction(pub Option<ConfirmAction>);

// =============================================================================
// Plugin
// =============================================================================

pub struct ConfirmDialogPlugin;

impl Plugin for ConfirmDialogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingConfirmAction>();
        app.add_systems(
            Update,
            confirm_dialog_ui
                .run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
        );
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Renders a modal confirmation dialog when a destructive action is pending.
fn confirm_dialog_ui(
    mut contexts: EguiContexts,
    mut pending: ResMut<PendingConfirmAction>,
    mut new_game_events: EventWriter<NewGameEvent>,
    mut load_events: EventWriter<LoadGameEvent>,
    mut pre_load: ResMut<PreLoadAppState>,
) {
    let Some(action) = pending.0 else {
        return;
    };

    let ctx = contexts.ctx_mut();

    // Semi-transparent backdrop to block interaction with the game UI.
    let screen_rect = ctx.screen_rect();
    egui::Area::new(egui::Id::new("confirm_dialog_backdrop"))
        .fixed_pos(screen_rect.min)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            painter.rect_filled(
                screen_rect,
                egui::CornerRadius::ZERO,
                egui::Color32::from_black_alpha(120),
            );
            ui.allocate_rect(screen_rect, egui::Sense::click());
        });

    let (title, description) = match action {
        ConfirmAction::NewGame => (
            "New Game",
            "Start a new game? Unsaved progress will be lost.",
        ),
        ConfirmAction::QuickLoad => (
            "Quick Load",
            "Load the quicksave? Unsaved progress will be lost.",
        ),
    };

    let mut should_clear = false;

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(320.0)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 10.0;
                ui.add_space(12.0);

                ui.heading(title);
                ui.add_space(4.0);
                ui.label(description);
                ui.add_space(12.0);

                let button_size = egui::Vec2::new(120.0, 32.0);

                ui.horizontal(|ui| {
                    // Center the two buttons
                    let total_width = button_size.x * 2.0 + 16.0;
                    let avail = ui.available_width();
                    if avail > total_width {
                        ui.add_space((avail - total_width) / 2.0);
                    }

                    if ui
                        .add_sized(button_size, egui::Button::new("Confirm"))
                        .clicked()
                    {
                        match action {
                            ConfirmAction::NewGame => {
                                new_game_events.send(NewGameEvent);
                            }
                            ConfirmAction::QuickLoad => {
                                pre_load.0 = Some(AppState::Playing);
                                load_events.send(LoadGameEvent);
                            }
                        }
                        should_clear = true;
                    }

                    ui.add_space(16.0);

                    if ui
                        .add_sized(button_size, egui::Button::new("Cancel"))
                        .clicked()
                    {
                        should_clear = true;
                    }
                });

                ui.add_space(12.0);
            });
        });

    if should_clear {
        pending.0 = None;
    }
}
