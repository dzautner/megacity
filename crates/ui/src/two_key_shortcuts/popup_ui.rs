//! Popup UI rendering for the two-key shortcut system.
//!
//! Draws a numbered sub-tool popup when a category key is pending, including
//! a countdown timer bar and the list of available sub-tools.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::categories::build_shortcut_categories;
use super::input::{TwoKeyShortcutState, TIMEOUT_SECS};

/// Draws the numbered sub-tool popup when a category key is pending.
pub(crate) fn two_key_popup_ui(state: Res<TwoKeyShortcutState>, mut contexts: EguiContexts) {
    let cat_idx = match state.pending_category {
        Some(idx) => idx,
        None => return,
    };

    let categories = build_shortcut_categories();
    if cat_idx >= categories.len() {
        return;
    }
    let cat = &categories[cat_idx];

    let screen = contexts.ctx_mut().screen_rect();

    egui::Area::new(egui::Id::new("two_key_shortcut_popup"))
        .fixed_pos(egui::pos2(
            screen.center().x - 120.0,
            screen.center().y - 100.0,
        ))
        .order(egui::Order::Foreground)
        .show(contexts.ctx_mut(), |ui| {
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 40, 240))
                .corner_radius(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.set_min_width(240.0);

                    // Header with category name and key hint
                    ui.horizontal(|ui| {
                        ui.heading(
                            egui::RichText::new(format!("[{}] {}", cat.key_hint, cat.label))
                                .strong()
                                .color(egui::Color32::from_rgb(140, 200, 255)),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Timer indicator
                            let pct = (state.timer / TIMEOUT_SECS).clamp(0.0, 1.0);
                            let bar_color = if pct > 0.3 {
                                egui::Color32::from_rgb(100, 200, 100)
                            } else {
                                egui::Color32::from_rgb(220, 100, 60)
                            };
                            let (rect, _) =
                                ui.allocate_exact_size(egui::vec2(40.0, 6.0), egui::Sense::hover());
                            ui.painter()
                                .rect_filled(rect, 3.0, egui::Color32::from_gray(50));
                            let filled = egui::Rect::from_min_size(
                                rect.min,
                                egui::vec2(rect.width() * pct, rect.height()),
                            );
                            ui.painter().rect_filled(filled, 3.0, bar_color);
                        });
                    });

                    ui.separator();

                    // Numbered sub-tool list
                    for (i, item) in cat.items.iter().enumerate() {
                        let number = if i < 9 { i + 1 } else { 0 }; // 1-9, then 0
                        if i >= 10 {
                            break; // max 10 items (digits 1-9 + 0)
                        }

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("[{}]", number))
                                    .strong()
                                    .color(egui::Color32::from_rgb(255, 220, 100))
                                    .monospace(),
                            );
                            ui.label(
                                egui::RichText::new(item.name)
                                    .color(egui::Color32::from_rgb(220, 220, 220)),
                            );
                        });
                    }

                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Press number to select, Esc to cancel")
                            .small()
                            .color(egui::Color32::from_gray(140)),
                    );
                });
        });
}
