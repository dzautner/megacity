//! Main Menu Screen (PLAY-002) + New Game Options Dialog (PLAY-019).
//!
//! Renders the main menu when [`AppState::MainMenu`] is active. Provides
//! buttons for New Game, Continue (most recent save), Load Game, Settings,
//! and Quit (hidden on WASM).
//!
//! The load screen is extracted to `main_menu_load.rs` for modularity.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use save::{LoadGameEvent, NewGameEvent, PendingSavePath};
use simulation::app_state::AppState;
use simulation::new_game_config::{random_seed, NewGameConfig};
use simulation::save_slots::SaveSlotManager;
use simulation::PreLoadAppState;

use crate::main_menu_load::{discover_save_files, SaveFileEntry};
use crate::settings_menu::SettingsMenuOpen;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MainMenuState>();
        app.add_systems(
            Update,
            main_menu_ui.run_if(in_state(AppState::MainMenu)),
        );
        app.add_systems(OnEnter(AppState::MainMenu), refresh_save_list);
    }
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Which sub-screen the main menu is showing.
#[derive(Default, PartialEq, Eq)]
enum MenuScreen {
    #[default]
    Main,
    NewGame,
    LoadGame,
}

/// Tracks main menu UI state (sub-screen selection, cached save list).
#[derive(Resource, Default)]
struct MainMenuState {
    screen: MenuScreen,
    save_files: Vec<SaveFileEntry>,
    city_name_input: String,
    seed_input: String,
    seed_value: u64,
    confirm_delete: Option<u32>,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn refresh_save_list(mut state: ResMut<MainMenuState>) {
    state.screen = MenuScreen::Main;
    state.save_files = discover_save_files();
    state.confirm_delete = None;
    state.city_name_input = "New City".to_string();
    let seed = random_seed();
    state.seed_value = seed;
    state.seed_input = seed.to_string();
}

#[allow(clippy::too_many_arguments)]
fn main_menu_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<MainMenuState>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut new_game_events: EventWriter<NewGameEvent>,
    mut load_game_events: EventWriter<LoadGameEvent>,
    mut pending_path: ResMut<PendingSavePath>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
    mut settings_menu: ResMut<SettingsMenuOpen>,
    mut new_game_config: ResMut<NewGameConfig>,
    slot_manager: Res<SaveSlotManager>,
    mut delete_events: EventWriter<simulation::save_slots::DeleteSlotEvent>,
    mut pre_load: ResMut<PreLoadAppState>,
) {
    let ctx = contexts.ctx_mut();

    if settings_menu.open {
        return;
    }

    match state.screen {
        MenuScreen::LoadGame => {
            let save_files = state.save_files.clone();
            let mut back_clicked = false;
            crate::main_menu_load::render_load_screen(
                ctx,
                &save_files,
                &mut state.confirm_delete,
                &mut back_clicked,
                &mut next_app_state,
                &mut load_game_events,
                &mut pending_path,
                &slot_manager,
                &mut delete_events,
                &mut pre_load,
            );
            if back_clicked {
                state.screen = MenuScreen::Main;
            }
        }
        MenuScreen::NewGame => {
            render_new_game_dialog(
                ctx,
                &mut state,
                &mut next_app_state,
                &mut new_game_events,
                &mut new_game_config,
            );
        }
        MenuScreen::Main => {
            render_main_buttons(
                ctx,
                &mut state,
                &mut next_app_state,
                &mut load_game_events,
                &mut pending_path,
                &mut app_exit,
                &mut settings_menu,
                &slot_manager,
                &mut pre_load,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

const BUTTON_SIZE: egui::Vec2 = egui::Vec2 { x: 240.0, y: 44.0 };

#[allow(clippy::too_many_arguments)]
fn render_main_buttons(
    ctx: &egui::Context,
    state: &mut ResMut<MainMenuState>,
    next_app_state: &mut ResMut<NextState<AppState>>,
    load_game_events: &mut EventWriter<LoadGameEvent>,
    pending_path: &mut ResMut<PendingSavePath>,
    app_exit: &mut EventWriter<bevy::app::AppExit>,
    settings_menu: &mut ResMut<SettingsMenuOpen>,
    slot_manager: &Res<SaveSlotManager>,
    pre_load: &mut ResMut<PreLoadAppState>,
) {
    let has_saves = !state.save_files.is_empty() || slot_manager.slot_count() > 0;

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgba_premultiplied(20, 22, 30, 240)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let available = ui.available_height();
                ui.add_space(available * 0.25);

                ui.label(
                    egui::RichText::new("MEGACITY")
                        .size(64.0)
                        .strong()
                        .color(egui::Color32::from_rgb(100, 160, 220)),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("City Builder")
                        .size(18.0)
                        .color(egui::Color32::from_rgb(160, 170, 190)),
                );
                ui.add_space(48.0);

                if ui
                    .add_sized(
                        BUTTON_SIZE,
                        egui::Button::new(egui::RichText::new("New Game").size(18.0)),
                    )
                    .clicked()
                {
                    state.screen = MenuScreen::NewGame;
                }
                ui.add_space(8.0);

                let continue_response = ui.add_enabled(
                    has_saves,
                    egui::Button::new(egui::RichText::new("Continue").size(18.0))
                        .min_size(BUTTON_SIZE),
                );
                if !has_saves {
                    continue_response.on_disabled_hover_text("No save files found");
                } else if continue_response.clicked() {
                    let slot_saves = slot_manager.slots_by_recency();
                    if let Some(slot) = slot_saves.first() {
                        pending_path.0 = Some(slot.file_path());
                        pre_load.0 = Some(AppState::MainMenu);
                        load_game_events.send(LoadGameEvent);
                        next_app_state.set(AppState::Playing);
                    } else if let Some(entry) = state.save_files.first() {
                        pending_path.0 = Some(entry.path.clone());
                        pre_load.0 = Some(AppState::MainMenu);
                        load_game_events.send(LoadGameEvent);
                        next_app_state.set(AppState::Playing);
                    }
                }
                ui.add_space(8.0);

                let load_response = ui.add_enabled(
                    has_saves,
                    egui::Button::new(egui::RichText::new("Load Game").size(18.0))
                        .min_size(BUTTON_SIZE),
                );
                if !has_saves {
                    load_response.on_disabled_hover_text("No save files found");
                } else if load_response.clicked() {
                    state.screen = MenuScreen::LoadGame;
                }
                ui.add_space(8.0);

                if ui
                    .add_sized(
                        BUTTON_SIZE,
                        egui::Button::new(egui::RichText::new("Settings").size(18.0)),
                    )
                    .clicked()
                {
                    settings_menu.open = true;
                    settings_menu.from_main_menu = true;
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.add_space(8.0);
                    if ui
                        .add_sized(
                            BUTTON_SIZE,
                            egui::Button::new(egui::RichText::new("Quit").size(18.0)),
                        )
                        .clicked()
                    {
                        app_exit.send(bevy::app::AppExit::Success);
                    }
                }

                #[cfg(target_arch = "wasm32")]
                let _ = app_exit;
            });
        });
}

fn render_new_game_dialog(
    ctx: &egui::Context,
    state: &mut ResMut<MainMenuState>,
    next_app_state: &mut ResMut<NextState<AppState>>,
    new_game_events: &mut EventWriter<NewGameEvent>,
    new_game_config: &mut ResMut<NewGameConfig>,
) {
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgba_premultiplied(20, 22, 30, 240)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let available = ui.available_height();
                ui.add_space(available * 0.2);

                ui.label(
                    egui::RichText::new("New Game")
                        .size(36.0)
                        .strong()
                        .color(egui::Color32::from_rgb(100, 160, 220)),
                );
                ui.add_space(32.0);

                let field_width = 300.0;
                ui.label(
                    egui::RichText::new("City Name")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(180, 190, 210)),
                );
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::singleline(&mut state.city_name_input)
                        .desired_width(field_width)
                        .char_limit(40)
                        .font(egui::TextStyle::Body),
                );
                ui.add_space(20.0);

                ui.label(
                    egui::RichText::new("Map Seed")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(180, 190, 210)),
                );
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    let total_width = field_width + 8.0 + 120.0;
                    let avail = ui.available_width();
                    if avail > total_width {
                        ui.add_space((avail - total_width) / 2.0);
                    }

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut state.seed_input)
                            .desired_width(field_width)
                            .font(egui::TextStyle::Body),
                    );

                    if response.changed() {
                        if let Ok(parsed) = state.seed_input.trim().parse::<u64>() {
                            state.seed_value = parsed;
                        }
                    }

                    ui.add_space(8.0);

                    if ui
                        .add_sized(
                            egui::vec2(120.0, 24.0),
                            egui::Button::new(egui::RichText::new("Randomize").size(14.0)),
                        )
                        .clicked()
                    {
                        let new_seed = random_seed();
                        state.seed_value = new_seed;
                        state.seed_input = new_seed.to_string();
                    }
                });
                ui.add_space(32.0);

                ui.horizontal(|ui| {
                    let total_width = 240.0 + 16.0 + 240.0;
                    let avail = ui.available_width();
                    if avail > total_width {
                        ui.add_space((avail - total_width) / 2.0);
                    }

                    let name_valid = !state.city_name_input.trim().is_empty();

                    let start_btn = ui.add_enabled(
                        name_valid,
                        egui::Button::new(egui::RichText::new("Start").size(18.0))
                            .min_size(BUTTON_SIZE),
                    );
                    if !name_valid {
                        start_btn.on_disabled_hover_text("City name cannot be empty");
                    } else if start_btn.clicked() {
                        new_game_config.city_name = state.city_name_input.trim().to_string();
                        new_game_config.seed = state.seed_value;
                        new_game_events.send(NewGameEvent);
                        next_app_state.set(AppState::Playing);
                    }

                    ui.add_space(16.0);

                    if ui
                        .add_sized(
                            BUTTON_SIZE,
                            egui::Button::new(egui::RichText::new("Back").size(18.0)),
                        )
                        .clicked()
                    {
                        state.screen = MenuScreen::Main;
                    }
                });
            });
        });
}
