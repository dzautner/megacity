//! Egui rendering for the right-click context menu.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::grid::ZoneType;

use super::types::{zone_label, ContextMenuAction, ContextMenuState, ContextTarget, PendingAction};

/// Render the context menu using egui.
pub(crate) fn context_menu_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<ContextMenuState>,
    mut pending: ResMut<PendingAction>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !state.open {
        return;
    }

    // Close on Escape
    if keys.just_pressed(KeyCode::Escape) {
        state.open = false;
        state.target = None;
        return;
    }

    let Some(target) = state.target.clone() else {
        state.open = false;
        return;
    };

    let pos = state.screen_pos;

    let mut close = false;

    egui::Area::new(egui::Id::new("context_menu_area"))
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
        .show(contexts.ctx_mut(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(140.0);

                match &target {
                    ContextTarget::Building {
                        zone_type, level, ..
                    } => {
                        ui.label(
                            egui::RichText::new(format!("{} (L{})", zone_label(*zone_type), level))
                                .strong(),
                        );
                        ui.separator();

                        if ui.button("Inspect").clicked() {
                            pending.0 = Some(ContextMenuAction::Inspect);
                            close = true;
                        }
                        if ui.button("Bulldoze").clicked() {
                            pending.0 = Some(ContextMenuAction::Bulldoze);
                            close = true;
                        }
                    }
                    ContextTarget::Service { name, .. } => {
                        ui.label(egui::RichText::new(name.as_str()).strong());
                        ui.separator();

                        if ui.button("Inspect").clicked() {
                            pending.0 = Some(ContextMenuAction::Inspect);
                            close = true;
                        }
                        if ui.button("Bulldoze").clicked() {
                            pending.0 = Some(ContextMenuAction::Bulldoze);
                            close = true;
                        }
                    }
                    ContextTarget::Road { segment_id, .. } => {
                        ui.label(egui::RichText::new("Road").strong());
                        ui.separator();

                        if ui.button("Inspect").clicked() {
                            pending.0 = Some(ContextMenuAction::Inspect);
                            close = true;
                        }
                        if ui.button("Bulldoze").clicked() {
                            pending.0 = Some(ContextMenuAction::Bulldoze);
                            close = true;
                        }
                        if let Some(seg_id) = segment_id {
                            if ui.button("Toggle One-Way").clicked() {
                                pending.0 = Some(ContextMenuAction::ToggleOneWay(*seg_id));
                                close = true;
                            }
                        }
                    }
                    ContextTarget::Citizen { entity } => {
                        ui.label(egui::RichText::new("Citizen").strong());
                        ui.separator();

                        if ui.button("Follow").clicked() {
                            pending.0 = Some(ContextMenuAction::FollowCitizen(*entity));
                            close = true;
                        }
                        if ui.button("Details").clicked() {
                            pending.0 = Some(ContextMenuAction::CitizenDetails(*entity));
                            close = true;
                        }
                    }
                    ContextTarget::Empty { zone_type, .. } => {
                        let label = if *zone_type == ZoneType::None {
                            "Empty Cell"
                        } else {
                            "Zoned Cell"
                        };
                        ui.label(egui::RichText::new(label).strong());
                        ui.separator();

                        if ui.button("Zone").clicked() {
                            pending.0 =
                                Some(ContextMenuAction::SetToolZone(ZoneType::ResidentialLow));
                            close = true;
                        }
                        if ui.button("Place Service").clicked() {
                            pending.0 = Some(ContextMenuAction::SetToolPlaceService);
                            close = true;
                        }
                    }
                }
            });
        });

    // Close on click outside: check if mouse is pressed and not over the menu
    let ctx = contexts.ctx_mut();
    if ctx.input(|i| i.pointer.any_pressed()) && !ctx.is_pointer_over_area() {
        close = true;
    }

    if close {
        state.open = false;
        state.target = None;
    }
}
