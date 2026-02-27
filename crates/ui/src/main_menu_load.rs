//! Main menu load screen — shows save slots and loose save files.
//!
//! Extracted from `main_menu.rs` to stay within the 500-line limit.
//! Called by the main menu when `MenuScreen::LoadGame` is active.

use bevy::prelude::*;
use bevy_egui::egui;

use save::{LoadGameEvent, PendingSavePath, SaveMetadata};
use simulation::app_state::AppState;
use simulation::save_slots::{SaveSlotInfo, SaveSlotManager};
use simulation::PreLoadAppState;

use crate::save_slot_format::format_slot_details;

// =============================================================================
// Data Types (re-exported for main_menu)
// =============================================================================

/// A discovered save file on disk.
#[derive(Clone)]
pub struct SaveFileEntry {
    /// File path (relative or absolute).
    pub path: String,
    /// Display name derived from file name.
    pub display_name: String,
    /// Optional metadata read from the file header.
    pub metadata: Option<SaveMetadata>,
}

// =============================================================================
// Load Screen Renderer
// =============================================================================

/// Render the load-game sub-screen showing save slots and loose files.
#[allow(clippy::too_many_arguments)]
pub fn render_load_screen(
    ctx: &egui::Context,
    save_files: &[SaveFileEntry],
    confirm_delete: &mut Option<u32>,
    back_clicked: &mut bool,
    next_app_state: &mut ResMut<NextState<AppState>>,
    load_game_events: &mut EventWriter<LoadGameEvent>,
    pending_path: &mut ResMut<PendingSavePath>,
    slot_manager: &Res<SaveSlotManager>,
    delete_events: &mut EventWriter<simulation::save_slots::DeleteSlotEvent>,
    pre_load: &mut ResMut<PreLoadAppState>,
) {
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgba_premultiplied(20, 22, 30, 240)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let available = ui.available_height();
                ui.add_space(available * 0.1);

                ui.label(
                    egui::RichText::new("Load Game")
                        .size(36.0)
                        .strong()
                        .color(egui::Color32::from_rgb(100, 160, 220)),
                );
                ui.add_space(24.0);

                let entry_size = egui::vec2(440.0, 48.0);

                // -- Save Slots Section --
                let slots: Vec<SaveSlotInfo> =
                    slot_manager.slots_by_recency().into_iter().cloned().collect();

                if !slots.is_empty() {
                    ui.label(
                        egui::RichText::new("Save Slots")
                            .size(16.0)
                            .strong()
                            .color(egui::Color32::from_rgb(180, 190, 210)),
                    );
                    ui.add_space(8.0);

                    for slot in &slots {
                        render_slot_row(
                            ui, slot, entry_size, confirm_delete,
                            next_app_state, load_game_events,
                            pending_path, delete_events, pre_load,
                        );
                    }
                    ui.add_space(8.0);
                }

                // -- Loose Save Files Section --
                if !save_files.is_empty() {
                    if !slots.is_empty() {
                        ui.separator();
                        ui.add_space(8.0);
                    }
                    ui.label(
                        egui::RichText::new("Other Save Files")
                            .size(16.0)
                            .strong()
                            .color(egui::Color32::from_rgb(180, 190, 210)),
                    );
                    ui.add_space(8.0);

                    for entry in save_files {
                        let label = format_save_entry(entry);
                        if ui
                            .add_sized(
                                entry_size,
                                egui::Button::new(egui::RichText::new(label).size(13.0)),
                            )
                            .clicked()
                        {
                            pending_path.0 = Some(entry.path.clone());
                            pre_load.0 = Some(AppState::MainMenu);
                            load_game_events.send(LoadGameEvent);
                            next_app_state.set(AppState::Playing);
                        }
                        ui.add_space(4.0);
                    }
                }

                ui.add_space(24.0);

                if ui
                    .add_sized(
                        egui::vec2(240.0, 36.0),
                        egui::Button::new(egui::RichText::new("Back").size(16.0)),
                    )
                    .clicked()
                {
                    *back_clicked = true;
                    *confirm_delete = None;
                }
            });
        });
}

// =============================================================================
// Helpers
// =============================================================================

#[allow(clippy::too_many_arguments)]
fn render_slot_row(
    ui: &mut egui::Ui,
    slot: &SaveSlotInfo,
    entry_size: egui::Vec2,
    confirm_delete: &mut Option<u32>,
    next_app_state: &mut ResMut<NextState<AppState>>,
    load_game_events: &mut EventWriter<LoadGameEvent>,
    pending_path: &mut ResMut<PendingSavePath>,
    delete_events: &mut EventWriter<simulation::save_slots::DeleteSlotEvent>,
    pre_load: &mut ResMut<PreLoadAppState>,
) {
    let is_confirming = *confirm_delete == Some(slot.slot_index);

    ui.horizontal(|ui| {
        let total_width = entry_size.x + 80.0;
        let avail = ui.available_width();
        if avail > total_width {
            ui.add_space((avail - total_width) / 2.0);
        }

        let label = format_slot_load_entry(slot);
        if ui
            .add_sized(
                entry_size,
                egui::Button::new(egui::RichText::new(label).size(13.0)),
            )
            .clicked()
        {
            pending_path.0 = Some(slot.file_path());
            pre_load.0 = Some(AppState::MainMenu);
            load_game_events.send(LoadGameEvent);
            next_app_state.set(AppState::Playing);
        }

        if is_confirming {
            if ui
                .add_sized(
                    egui::vec2(36.0, entry_size.y),
                    egui::Button::new(
                        egui::RichText::new("Yes")
                            .size(11.0)
                            .color(crate::theme::ERROR),
                    ),
                )
                .clicked()
            {
                delete_events.send(simulation::save_slots::DeleteSlotEvent {
                    slot_index: slot.slot_index,
                });
                *confirm_delete = None;
            }
            if ui
                .add_sized(
                    egui::vec2(36.0, entry_size.y),
                    egui::Button::new(egui::RichText::new("No").size(11.0)),
                )
                .clicked()
            {
                *confirm_delete = None;
            }
        } else if ui
            .add_sized(
                egui::vec2(36.0, entry_size.y),
                egui::Button::new(
                    egui::RichText::new("X")
                        .size(11.0)
                        .color(crate::theme::TEXT_MUTED),
                ),
            )
            .on_hover_text("Delete this save")
            .clicked()
        {
            *confirm_delete = Some(slot.slot_index);
        }
    });
    ui.add_space(4.0);
}

/// Format a slot entry for the main menu load screen.
fn format_slot_load_entry(slot: &SaveSlotInfo) -> String {
    let details = format_slot_details(slot);
    format!("{} - {}", slot.display_name, details)
}

/// Format a save entry for display in the load screen.
pub fn format_save_entry(entry: &SaveFileEntry) -> String {
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
/// (most recent first). On WASM this returns an empty list.
pub fn discover_save_files() -> Vec<SaveFileEntry> {
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
