//! Pause menu UI (PLAY-006).
//!
//! Pressing ESC toggles between `AppState::Playing` and `AppState::Paused`.
//! While paused, a semi-transparent overlay with action buttons is shown.
//!
//! "Save Game" and "Load Game" buttons now open the save-slot picker dialogs
//! from `save_slot_ui` instead of directly triggering save/load events.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::app_state::AppState;
use simulation::time_of_day::GameClock;

use crate::save_slot_ui::SaveSlotUiState;
use crate::settings_menu::SettingsMenuOpen;

// =============================================================================
// Resources
// =============================================================================

/// Tracks whether the "Return to Main Menu" confirmation is showing.
#[derive(Resource, Default)]
struct MainMenuConfirm(bool);

// =============================================================================
// Plugin
// =============================================================================

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MainMenuConfirm>();
        app.add_systems(Update, toggle_pause);
        app.add_systems(
            Update,
            pause_menu_ui.run_if(in_state(AppState::Paused)),
        );
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Toggles between Playing and Paused when ESC is pressed.
fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_clock: ResMut<GameClock>,
    mut contexts: EguiContexts,
    mut confirm: ResMut<MainMenuConfirm>,
    mut settings_menu: ResMut<SettingsMenuOpen>,
) {
    // Don't intercept ESC if egui is consuming keyboard input (e.g. text fields).
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    // If settings menu is open, close it instead of toggling pause.
    if settings_menu.open {
        settings_menu.open = false;
        return;
    }

    match app_state.get() {
        AppState::Playing => {
            game_clock.paused = true;
            next_state.set(AppState::Paused);
        }
        AppState::Paused => {
            // Reset confirmation state when unpausing via ESC.
            confirm.0 = false;
            game_clock.paused = false;
            next_state.set(AppState::Playing);
        }
        AppState::MainMenu => {
            // ESC does nothing on the main menu.
        }
    }
}

/// Renders the pause menu overlay and buttons.
#[allow(clippy::too_many_arguments)]
fn pause_menu_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_clock: ResMut<GameClock>,
    mut confirm: ResMut<MainMenuConfirm>,
    mut settings_menu: ResMut<SettingsMenuOpen>,
    mut slot_ui: ResMut<SaveSlotUiState>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: EventWriter<AppExit>,
) {
    // Don't render pause menu buttons when settings menu is open.
    if settings_menu.open {
        return;
    }

    // Don't render pause menu buttons when a save/load dialog is open.
    if slot_ui.save_dialog_open || slot_ui.load_dialog_open {
        return;
    }

    let ctx = contexts.ctx_mut();

    // Semi-transparent dark overlay covering the entire screen.
    let screen_rect = ctx.screen_rect();
    egui::Area::new(egui::Id::new("pause_overlay"))
        .fixed_pos(screen_rect.min)
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            let painter = ui.painter();
            painter.rect_filled(
                screen_rect,
                egui::CornerRadius::ZERO,
                egui::Color32::from_black_alpha(160),
            );
            // Allocate the full rect so the area has the right size.
            ui.allocate_rect(screen_rect, egui::Sense::hover());
        });

    // Centered panel with action buttons.
    egui::Window::new("Paused")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(240.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 10.0;

                ui.add_space(8.0);
                ui.heading("Paused");
                ui.add_space(8.0);

                let button_size = egui::Vec2::new(200.0, 36.0);

                // Resume
                if ui
                    .add_sized(button_size, egui::Button::new("Resume"))
                    .clicked()
                {
                    confirm.0 = false;
                    game_clock.paused = false;
                    next_state.set(AppState::Playing);
                }

                // Save Game — opens save slot picker
                if ui
                    .add_sized(button_size, egui::Button::new("Save Game"))
                    .clicked()
                {
                    slot_ui.save_dialog_open = true;
                    slot_ui.save_name_input = String::new();
                    slot_ui.confirm_overwrite = None;
                    slot_ui.confirm_delete = None;
                }

                // Load Game — opens load slot picker
                if ui
                    .add_sized(button_size, egui::Button::new("Load Game"))
                    .clicked()
                {
                    slot_ui.load_dialog_open = true;
                    slot_ui.confirm_delete = None;
                }

                // Settings
                if ui
                    .add_sized(button_size, egui::Button::new("Settings"))
                    .clicked()
                {
                    settings_menu.open = true;
                    settings_menu.from_main_menu = false;
                }

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // Main Menu — with confirmation step
                if !confirm.0 {
                    if ui
                        .add_sized(button_size, egui::Button::new("Main Menu"))
                        .clicked()
                    {
                        confirm.0 = true;
                    }
                } else {
                    ui.label("Unsaved progress will be lost.");
                    ui.horizontal(|ui| {
                        if ui.button("Confirm").clicked() {
                            confirm.0 = false;
                            game_clock.paused = false;
                            next_state.set(AppState::MainMenu);
                        }
                        if ui.button("Cancel").clicked() {
                            confirm.0 = false;
                        }
                    });
                }

                // Quit — hidden on WASM (browsers don't support app exit)
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if ui
                        .add_sized(button_size, egui::Button::new("Quit"))
                        .clicked()
                    {
                        exit.send(AppExit::Success);
                    }
                }

                ui.add_space(8.0);
            });
        });
}
