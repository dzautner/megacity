//! UI-014: Save Slot Picker for save and load operations.
//!
//! Provides egui windows for:
//! - **Save dialog**: Choose an existing slot to overwrite or create a new save
//! - **Load dialog**: Browse available saves with metadata and load one
//!
//! Integrates with the backend `SaveSlotManager` from simulation and the
//! `SaveGameEvent`/`LoadGameEvent` from the save crate.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use save::{LoadGameEvent, PendingSavePath, SaveGameEvent};
use simulation::app_state::AppState;
use simulation::notifications::{NotificationEvent, NotificationPriority};
use simulation::save_slots::{
    DeleteSlotEvent, SaveSlotInfo, SaveSlotManager, SaveToSlotEvent, MAX_SAVE_SLOTS,
};

pub use crate::save_slot_format::{format_population, format_slot_details, format_timestamp};

// =============================================================================
// Resources
// =============================================================================

/// Controls which save-slot dialog is currently open.
#[derive(Resource, Default)]
pub struct SaveSlotUiState {
    /// Whether the save-slot picker (for saving) is open.
    pub save_dialog_open: bool,
    /// Whether the load-slot picker (for loading) is open.
    pub load_dialog_open: bool,
    /// Text input for the new save name.
    pub save_name_input: String,
    /// Slot index pending delete confirmation, if any.
    pub confirm_delete: Option<u32>,
    /// Slot index pending overwrite confirmation, if any.
    pub confirm_overwrite: Option<u32>,
}

// =============================================================================
// Constants
// =============================================================================

const DIALOG_WIDTH: f32 = 480.0;
const SLOT_ROW_HEIGHT: f32 = 52.0;
const BUTTON_HEIGHT: f32 = 28.0;

// =============================================================================
// Plugin
// =============================================================================

pub struct SaveSlotUiPlugin;

impl Plugin for SaveSlotUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveSlotUiState>();
        app.add_systems(
            Update,
            (save_slot_dialog_system, load_slot_dialog_system)
                .run_if(in_state(AppState::Playing).or(in_state(AppState::Paused))),
        );
    }
}

// =============================================================================
// Save Dialog System
// =============================================================================

/// Renders the "Save Game" slot picker dialog.
#[allow(clippy::too_many_arguments)]
fn save_slot_dialog_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<SaveSlotUiState>,
    manager: Res<SaveSlotManager>,
    mut save_to_slot_events: EventWriter<SaveToSlotEvent>,
    mut save_game_events: EventWriter<SaveGameEvent>,
    mut pending_path: ResMut<PendingSavePath>,
    mut notifications: EventWriter<NotificationEvent>,
) {
    if !ui_state.save_dialog_open {
        return;
    }

    let ctx = contexts.ctx_mut();
    let mut should_close = false;

    egui::Window::new("Save Game")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(DIALOG_WIDTH)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 6.0;

                render_new_save_section(ui, &manager, &mut ui_state, &mut save_to_slot_events,
                    &mut save_game_events, &mut pending_path, &mut notifications,
                    &mut should_close);

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                render_overwrite_section(ui, &manager, &mut ui_state, &mut save_to_slot_events,
                    &mut save_game_events, &mut pending_path, &mut notifications,
                    &mut should_close);

                ui.add_space(8.0);
                render_cancel_button(ui, &mut should_close);
            });
        });

    if should_close {
        ui_state.save_dialog_open = false;
        ui_state.confirm_overwrite = None;
        ui_state.confirm_delete = None;
    }
}

// =============================================================================
// Load Dialog System
// =============================================================================

/// Renders the "Load Game" slot picker dialog.
#[allow(clippy::too_many_arguments)]
fn load_slot_dialog_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<SaveSlotUiState>,
    manager: Res<SaveSlotManager>,
    mut load_game_events: EventWriter<LoadGameEvent>,
    mut pending_path: ResMut<PendingSavePath>,
    mut delete_events: EventWriter<DeleteSlotEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if !ui_state.load_dialog_open {
        return;
    }

    let ctx = contexts.ctx_mut();
    let mut should_close = false;

    egui::Window::new("Load Game")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(DIALOG_WIDTH)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 6.0;

                if manager.slot_count() == 0 {
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("No save slots found.")
                            .size(crate::theme::FONT_BODY)
                            .color(crate::theme::TEXT_MUTED),
                    );
                    ui.add_space(16.0);
                } else {
                    ui.label(
                        egui::RichText::new("Select a save to load")
                            .size(crate::theme::FONT_SUBHEADING)
                            .color(crate::theme::TEXT_HEADING),
                    );
                    ui.add_space(4.0);

                    let slots: Vec<SaveSlotInfo> =
                        manager.slots_by_recency().into_iter().cloned().collect();

                    egui::ScrollArea::vertical()
                        .max_height(360.0)
                        .show(ui, |ui| {
                            for slot in &slots {
                                render_load_slot_row(
                                    ui, slot, &mut ui_state,
                                    &mut load_game_events, &mut pending_path,
                                    &mut delete_events, &mut next_app_state,
                                    &mut should_close,
                                );
                            }
                        });
                }

                ui.add_space(8.0);
                render_cancel_button(ui, &mut should_close);
            });
        });

    if should_close {
        ui_state.load_dialog_open = false;
        ui_state.confirm_delete = None;
    }
}

// =============================================================================
// Save Dialog Helpers
// =============================================================================

#[allow(clippy::too_many_arguments)]
fn render_new_save_section(
    ui: &mut egui::Ui,
    manager: &Res<SaveSlotManager>,
    ui_state: &mut ResMut<SaveSlotUiState>,
    save_to_slot_events: &mut EventWriter<SaveToSlotEvent>,
    save_game_events: &mut EventWriter<SaveGameEvent>,
    pending_path: &mut ResMut<PendingSavePath>,
    notifications: &mut EventWriter<NotificationEvent>,
    should_close: &mut bool,
) {
    ui.label(
        egui::RichText::new("Create New Save")
            .size(crate::theme::FONT_SUBHEADING)
            .color(crate::theme::TEXT_HEADING),
    );
    ui.add_space(2.0);

    let can_create = !manager.is_full();

    ui.horizontal(|ui| {
        let response = ui.add_enabled(
            can_create,
            egui::TextEdit::singleline(&mut ui_state.save_name_input)
                .desired_width(DIALOG_WIDTH - 120.0)
                .char_limit(40)
                .hint_text("Save name..."),
        );
        if !can_create {
            response.on_disabled_hover_text(format!(
                "Maximum {} save slots reached", MAX_SAVE_SLOTS
            ));
        }

        let name_valid = can_create && !ui_state.save_name_input.trim().is_empty();
        let save_btn = ui.add_enabled(
            name_valid,
            egui::Button::new("Save").min_size(egui::vec2(80.0, BUTTON_HEIGHT)),
        );
        if save_btn.clicked() {
            let name = ui_state.save_name_input.trim().to_string();
            if let Some(idx) = manager.next_available_index() {
                trigger_slot_save(idx, &name, save_to_slot_events, save_game_events, pending_path);
                notifications.send(NotificationEvent {
                    text: format!("Saved to slot {}: {}", idx + 1, name),
                    priority: NotificationPriority::Info,
                    location: None,
                });
                *should_close = true;
            }
        }
    });

    if manager.is_full() {
        ui.label(
            egui::RichText::new(format!(
                "All {} slots in use. Overwrite an existing save.", MAX_SAVE_SLOTS
            ))
            .size(crate::theme::FONT_SMALL)
            .color(crate::theme::WARNING),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_overwrite_section(
    ui: &mut egui::Ui,
    manager: &Res<SaveSlotManager>,
    ui_state: &mut ResMut<SaveSlotUiState>,
    save_to_slot_events: &mut EventWriter<SaveToSlotEvent>,
    save_game_events: &mut EventWriter<SaveGameEvent>,
    pending_path: &mut ResMut<PendingSavePath>,
    notifications: &mut EventWriter<NotificationEvent>,
    should_close: &mut bool,
) {
    if manager.slot_count() > 0 {
        ui.label(
            egui::RichText::new("Overwrite Existing")
                .size(crate::theme::FONT_SUBHEADING)
                .color(crate::theme::TEXT_HEADING),
        );
        ui.add_space(2.0);

        let slots: Vec<SaveSlotInfo> =
            manager.slots_by_recency().into_iter().cloned().collect();

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for slot in &slots {
                    render_save_slot_row(
                        ui, slot, ui_state, save_to_slot_events,
                        save_game_events, pending_path, notifications, should_close,
                    );
                }
            });
    } else {
        ui.label(
            egui::RichText::new("No existing saves.")
                .size(crate::theme::FONT_BODY)
                .color(crate::theme::TEXT_MUTED),
        );
    }
}

// =============================================================================
// Row Renderers
// =============================================================================

/// Render a single slot row in the save dialog (with overwrite confirmation).
#[allow(clippy::too_many_arguments)]
fn render_save_slot_row(
    ui: &mut egui::Ui,
    slot: &SaveSlotInfo,
    ui_state: &mut ResMut<SaveSlotUiState>,
    save_to_slot_events: &mut EventWriter<SaveToSlotEvent>,
    save_game_events: &mut EventWriter<SaveGameEvent>,
    pending_path: &mut ResMut<PendingSavePath>,
    notifications: &mut EventWriter<NotificationEvent>,
    should_close: &mut bool,
) {
    let is_confirming = ui_state.confirm_overwrite == Some(slot.slot_index);

    egui::Frame::NONE
        .fill(crate::theme::BG_SURFACE)
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin::same(6))
        .show(ui, |ui| {
            ui.set_min_height(SLOT_ROW_HEIGHT);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(DIALOG_WIDTH - 180.0);
                    ui.label(
                        egui::RichText::new(&slot.display_name)
                            .size(crate::theme::FONT_BODY)
                            .strong()
                            .color(crate::theme::TEXT_HEADING),
                    );
                    ui.label(
                        egui::RichText::new(format_slot_details(slot))
                            .size(crate::theme::FONT_SMALL)
                            .color(crate::theme::TEXT_MUTED),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if is_confirming {
                        if ui.add(egui::Button::new(
                            egui::RichText::new("Cancel").size(crate::theme::FONT_SMALL),
                        )).clicked() {
                            ui_state.confirm_overwrite = None;
                        }
                        if ui.add(egui::Button::new(
                            egui::RichText::new("Confirm").size(crate::theme::FONT_SMALL)
                                .color(crate::theme::WARNING),
                        )).clicked() {
                            let name = slot.display_name.clone();
                            trigger_slot_save(
                                slot.slot_index, &name,
                                save_to_slot_events, save_game_events, pending_path,
                            );
                            notifications.send(NotificationEvent {
                                text: format!("Overwrote slot {}: {}", slot.slot_index + 1, name),
                                priority: NotificationPriority::Info,
                                location: None,
                            });
                            *should_close = true;
                        }
                    } else if ui.add(egui::Button::new(
                        egui::RichText::new("Overwrite").size(crate::theme::FONT_SMALL),
                    )).clicked() {
                        ui_state.confirm_overwrite = Some(slot.slot_index);
                    }
                });
            });
        });
    ui.add_space(2.0);
}

/// Render a single slot row in the load dialog (with load/delete).
#[allow(clippy::too_many_arguments)]
fn render_load_slot_row(
    ui: &mut egui::Ui,
    slot: &SaveSlotInfo,
    ui_state: &mut ResMut<SaveSlotUiState>,
    load_game_events: &mut EventWriter<LoadGameEvent>,
    pending_path: &mut ResMut<PendingSavePath>,
    delete_events: &mut EventWriter<DeleteSlotEvent>,
    next_app_state: &mut ResMut<NextState<AppState>>,
    should_close: &mut bool,
) {
    let is_confirming_delete = ui_state.confirm_delete == Some(slot.slot_index);

    egui::Frame::NONE
        .fill(crate::theme::BG_SURFACE)
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin::same(6))
        .show(ui, |ui| {
            ui.set_min_height(SLOT_ROW_HEIGHT);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(DIALOG_WIDTH - 200.0);
                    ui.label(
                        egui::RichText::new(&slot.display_name)
                            .size(crate::theme::FONT_BODY)
                            .strong()
                            .color(crate::theme::TEXT_HEADING),
                    );
                    ui.label(
                        egui::RichText::new(format_slot_details(slot))
                            .size(crate::theme::FONT_SMALL)
                            .color(crate::theme::TEXT_MUTED),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if is_confirming_delete {
                        if ui.add(egui::Button::new(
                            egui::RichText::new("Cancel").size(crate::theme::FONT_SMALL),
                        )).clicked() {
                            ui_state.confirm_delete = None;
                        }
                        if ui.add(egui::Button::new(
                            egui::RichText::new("Delete").size(crate::theme::FONT_SMALL)
                                .color(crate::theme::ERROR),
                        )).clicked() {
                            delete_events.send(DeleteSlotEvent { slot_index: slot.slot_index });
                            ui_state.confirm_delete = None;
                        }
                    } else {
                        if ui.add(egui::Button::new(
                            egui::RichText::new("Delete").size(crate::theme::FONT_SMALL),
                        )).clicked() {
                            ui_state.confirm_delete = Some(slot.slot_index);
                        }
                        if ui.add(egui::Button::new(
                            egui::RichText::new("Load").size(crate::theme::FONT_SMALL)
                                .color(crate::theme::PRIMARY),
                        )).clicked() {
                            pending_path.0 = Some(slot.file_path());
                            load_game_events.send(LoadGameEvent);
                            next_app_state.set(AppState::Playing);
                            *should_close = true;
                        }
                    }
                });
            });
        });
    ui.add_space(2.0);
}

// =============================================================================
// Shared Helpers
// =============================================================================

fn render_cancel_button(ui: &mut egui::Ui, should_close: &mut bool) {
    ui.horizontal(|ui| {
        let avail = ui.available_width();
        ui.add_space((avail - 100.0).max(0.0) / 2.0);
        if ui
            .add_sized(egui::vec2(100.0, 32.0), egui::Button::new("Cancel"))
            .clicked()
        {
            *should_close = true;
        }
    });
}

/// Triggers a save to a specific slot: sends `SaveToSlotEvent` to update
/// metadata, sets `PendingSavePath`, and fires `SaveGameEvent`.
fn trigger_slot_save(
    slot_index: u32,
    name: &str,
    save_to_slot_events: &mut EventWriter<SaveToSlotEvent>,
    save_game_events: &mut EventWriter<SaveGameEvent>,
    pending_path: &mut ResMut<PendingSavePath>,
) {
    save_to_slot_events.send(SaveToSlotEvent {
        slot_index: Some(slot_index),
        display_name: name.to_string(),
    });
    pending_path.0 = Some(simulation::save_slots::slot_file_path(slot_index));
    save_game_events.send(SaveGameEvent);
}
