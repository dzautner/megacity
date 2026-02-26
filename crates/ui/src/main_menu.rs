//! Main Menu Screen (PLAY-002).
//!
//! Renders the main menu when [`AppState::MainMenu`] is active. Provides
//! buttons for New Game, Continue (most recent save), Load Game (save list),
//! and Quit (hidden on WASM).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use save::{LoadGameEvent, NewGameEvent, PendingSavePath, SaveMetadata};
use simulation::app_state::AppState;

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

/// Tracks main menu UI state (sub-screen selection, cached save list).
#[derive(Resource, Default)]
struct MainMenuState {
    /// When `true`, show the save file list instead of the main buttons.
    show_load_screen: bool,
    /// Cached list of discovered save files (native only).
    save_files: Vec<SaveFileEntry>,
}

/// A discovered save file on disk.
#[derive(Clone)]
struct SaveFileEntry {
    /// File path (relative or absolute).
    path: String,
    /// Display name derived from file name.
    display_name: String,
    /// Optional metadata read from the file header.
    metadata: Option<SaveMetadata>,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Refresh the list of save files when entering the main menu.
fn refresh_save_list(mut state: ResMut<MainMenuState>) {
    state.show_load_screen = false;
    state.save_files = discover_save_files();
}

/// The main menu UI system. Renders a centered egui panel with game actions.
#[allow(clippy::too_many_arguments)]
fn main_menu_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<MainMenuState>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut new_game_events: EventWriter<NewGameEvent>,
    mut load_game_events: EventWriter<LoadGameEvent>,
    mut pending_path: ResMut<PendingSavePath>,
    mut app_exit: EventWriter<bevy::app::AppExit>,
) {
    let ctx = contexts.ctx_mut();

    if state.show_load_screen {
        render_load_screen(
            ctx,
            &mut state,
            &mut next_app_state,
            &mut load_game_events,
            &mut pending_path,
        );
        return;
    }

    render_main_buttons(
        ctx,
        &mut state,
        &mut next_app_state,
        &mut new_game_events,
        &mut load_game_events,
        &mut pending_path,
        &mut app_exit,
    );
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

/// Standard button size for main menu items.
const BUTTON_SIZE: egui::Vec2 = egui::Vec2 { x: 240.0, y: 44.0 };

/// Render the primary main menu buttons.
#[allow(clippy::too_many_arguments)]
fn render_main_buttons(
    ctx: &egui::Context,
    state: &mut ResMut<MainMenuState>,
    next_app_state: &mut ResMut<NextState<AppState>>,
    new_game_events: &mut EventWriter<NewGameEvent>,
    load_game_events: &mut EventWriter<LoadGameEvent>,
    pending_path: &mut ResMut<PendingSavePath>,
    app_exit: &mut EventWriter<bevy::app::AppExit>,
) {
    let has_saves = !state.save_files.is_empty();

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgba_premultiplied(20, 22, 30, 240)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let available = ui.available_height();
                ui.add_space(available * 0.25);

                // Title
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

                // New Game
                if ui
                    .add_sized(
                        BUTTON_SIZE,
                        egui::Button::new(egui::RichText::new("New Game").size(18.0)),
                    )
                    .clicked()
                {
                    new_game_events.send(NewGameEvent);
                    next_app_state.set(AppState::Playing);
                }
                ui.add_space(8.0);

                // Continue (most recent save)
                let continue_response = ui.add_enabled(
                    has_saves,
                    egui::Button::new(egui::RichText::new("Continue").size(18.0))
                        .min_size(BUTTON_SIZE),
                );
                if !has_saves {
                    continue_response.on_disabled_hover_text("No save files found");
                } else if continue_response.clicked() {
                    if let Some(entry) = state.save_files.first() {
                        pending_path.0 = Some(entry.path.clone());
                        load_game_events.send(LoadGameEvent);
                        next_app_state.set(AppState::Playing);
                    }
                }
                ui.add_space(8.0);

                // Load Game
                let load_response = ui.add_enabled(
                    has_saves,
                    egui::Button::new(egui::RichText::new("Load Game").size(18.0))
                        .min_size(BUTTON_SIZE),
                );
                if !has_saves {
                    load_response.on_disabled_hover_text("No save files found");
                } else if load_response.clicked() {
                    state.show_load_screen = true;
                }

                // Quit (hidden on WASM)
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

                // Suppress unused warning on WASM
                #[cfg(target_arch = "wasm32")]
                let _ = app_exit;
            });
        });
}

/// Render the load-game sub-screen with a list of save files.
fn render_load_screen(
    ctx: &egui::Context,
    state: &mut ResMut<MainMenuState>,
    next_app_state: &mut ResMut<NextState<AppState>>,
    load_game_events: &mut EventWriter<LoadGameEvent>,
    pending_path: &mut ResMut<PendingSavePath>,
) {
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgba_premultiplied(20, 22, 30, 240)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let available = ui.available_height();
                ui.add_space(available * 0.15);

                ui.label(
                    egui::RichText::new("Load Game")
                        .size(36.0)
                        .strong()
                        .color(egui::Color32::from_rgb(100, 160, 220)),
                );
                ui.add_space(24.0);

                let entry_size = egui::vec2(360.0, 40.0);

                // Clone save files to avoid borrow conflicts with state
                let save_files = state.save_files.clone();
                for entry in &save_files {
                    let label = format_save_entry(entry);
                    if ui
                        .add_sized(
                            entry_size,
                            egui::Button::new(egui::RichText::new(label).size(14.0)),
                        )
                        .clicked()
                    {
                        pending_path.0 = Some(entry.path.clone());
                        load_game_events.send(LoadGameEvent);
                        next_app_state.set(AppState::Playing);
                    }
                    ui.add_space(4.0);
                }

                ui.add_space(24.0);

                if ui
                    .add_sized(
                        egui::vec2(240.0, 36.0),
                        egui::Button::new(egui::RichText::new("Back").size(16.0)),
                    )
                    .clicked()
                {
                    state.show_load_screen = false;
                }
            });
        });
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Format a save entry for display in the load screen.
fn format_save_entry(entry: &SaveFileEntry) -> String {
    if let Some(meta) = &entry.metadata {
        let hours = (meta.play_time_seconds / 3600.0) as u32;
        let mins = ((meta.play_time_seconds % 3600.0) / 60.0) as u32;
        format!(
            "{} - {} pop, Day {}, {}h{}m played",
            meta.city_name, meta.population, meta.day, hours, mins,
        )
    } else {
        entry.display_name.clone()
    }
}

/// Discover save files on disk. Returns entries sorted by modification time
/// (most recent first). On WASM this returns an empty list since we cannot
/// enumerate IndexedDB entries synchronously.
fn discover_save_files() -> Vec<SaveFileEntry> {
    #[cfg(target_arch = "wasm32")]
    {
        Vec::new()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        discover_save_files_native()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn discover_save_files_native() -> Vec<SaveFileEntry> {
    let mut entries = Vec::new();

    let Ok(dir) = std::fs::read_dir(".") else {
        return entries;
    };

    for item in dir.flatten() {
        let path = item.path();
        let Some(ext) = path.extension() else {
            continue;
        };
        if ext != "bin" {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Skip hidden/crash sentinel files
        if file_name.starts_with('.') {
            continue;
        }

        let metadata = std::fs::read(&path)
            .ok()
            .and_then(|bytes| save::read_metadata_only(&bytes).ok().flatten());

        entries.push(SaveFileEntry {
            path: path.to_string_lossy().to_string(),
            display_name: file_name,
            metadata,
        });
    }

    // Sort by modification time, most recent first
    entries.sort_by(|a, b| {
        let time_a = std::fs::metadata(&a.path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let time_b = std::fs::metadata(&b.path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        time_b.cmp(&time_a)
    });

    entries
}
