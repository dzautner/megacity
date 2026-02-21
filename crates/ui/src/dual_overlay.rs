//! UI panel for dual-overlay blending / split-screen (UX-029).
//!
//! Provides a floating panel that allows the player to:
//! - Select a secondary overlay to display alongside the primary
//! - Toggle between Blend and Split display modes
//! - Adjust the blend factor (in Blend mode)

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use rendering::overlay::{
    DualOverlayMode, DualOverlayState, OverlayMode, OverlayState, OVERLAY_CHOICES,
};

pub struct DualOverlayPlugin;

impl Plugin for DualOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, dual_overlay_ui);
    }
}

#[allow(clippy::too_many_arguments)]
fn dual_overlay_ui(
    mut contexts: EguiContexts,
    overlay: Res<OverlayState>,
    mut dual: ResMut<DualOverlayState>,
) {
    // Only show the dual overlay toggle button when a primary overlay is active
    if overlay.mode == OverlayMode::None {
        // If primary goes away, clear secondary
        if dual.secondary != OverlayMode::None {
            dual.secondary = OverlayMode::None;
        }
        return;
    }

    // Draw a small button to toggle the dual overlay panel,
    // positioned at the right side of the screen
    let screen_rect = contexts.ctx_mut().screen_rect();
    let btn_pos = egui::pos2(screen_rect.right() - 160.0, 42.0);

    egui::Area::new(egui::Id::new("dual_overlay_toggle"))
        .fixed_pos(btn_pos)
        .show(contexts.ctx_mut(), |ui| {
            let label = if dual.secondary != OverlayMode::None {
                format!("Compare: {} [x]", dual.secondary.label())
            } else {
                "Compare...".to_string()
            };
            let btn = ui.button(
                egui::RichText::new(&label)
                    .size(11.0)
                    .color(egui::Color32::from_rgb(180, 220, 255)),
            );
            if btn.clicked() {
                dual.panel_open = !dual.panel_open;
            }
        });

    if !dual.panel_open {
        return;
    }

    // Draw the dual overlay configuration panel
    let panel_pos = egui::pos2(screen_rect.right() - 240.0, 66.0);

    egui::Area::new(egui::Id::new("dual_overlay_panel"))
        .fixed_pos(panel_pos)
        .show(contexts.ctx_mut(), |ui| {
            egui::Frame::popup(ui.style())
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.set_min_width(220.0);

                    ui.label(
                        egui::RichText::new("Dual Overlay")
                            .strong()
                            .size(13.0)
                            .color(egui::Color32::from_rgb(180, 220, 255)),
                    );
                    ui.separator();

                    // Primary overlay (read-only display)
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Primary:")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(200, 200, 200)),
                        );
                        ui.label(
                            egui::RichText::new(overlay.mode.label())
                                .size(11.0)
                                .strong(),
                        );
                    });

                    // Secondary overlay selection
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Secondary:")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(200, 200, 200)),
                        );

                        egui::ComboBox::from_id_salt("secondary_overlay")
                            .selected_text(if dual.secondary == OverlayMode::None {
                                "None"
                            } else {
                                dual.secondary.label()
                            })
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                // "None" option to disable dual overlay
                                ui.selectable_value(&mut dual.secondary, OverlayMode::None, "None");
                                ui.separator();
                                // All overlay options except the primary
                                for &choice in &OVERLAY_CHOICES {
                                    if choice != overlay.mode {
                                        ui.selectable_value(
                                            &mut dual.secondary,
                                            choice,
                                            choice.label(),
                                        );
                                    }
                                }
                            });
                    });

                    // Mode and blend controls only shown when secondary is active
                    if dual.secondary != OverlayMode::None {
                        ui.add_space(4.0);
                        ui.separator();

                        // Mode toggle: Blend / Split
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Mode:")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(200, 200, 200)),
                            );

                            let blend_active = dual.mode == DualOverlayMode::Blend;
                            let split_active = dual.mode == DualOverlayMode::Split;

                            if ui
                                .selectable_label(
                                    blend_active,
                                    egui::RichText::new("Blend").size(11.0),
                                )
                                .clicked()
                            {
                                dual.mode = DualOverlayMode::Blend;
                            }
                            if ui
                                .selectable_label(
                                    split_active,
                                    egui::RichText::new("Split").size(11.0),
                                )
                                .clicked()
                            {
                                dual.mode = DualOverlayMode::Split;
                            }
                        });

                        // Blend factor slider (only in Blend mode)
                        if dual.mode == DualOverlayMode::Blend {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Mix:")
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(200, 200, 200)),
                                );
                                ui.add(
                                    egui::Slider::new(&mut dual.blend_factor, 0.0..=1.0)
                                        .text("")
                                        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                                );
                            });

                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}: {:.0}%",
                                        overlay.mode.label(),
                                        (1.0 - dual.blend_factor) * 100.0
                                    ))
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(160, 160, 160)),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}: {:.0}%",
                                        dual.secondary.label(),
                                        dual.blend_factor * 100.0
                                    ))
                                    .size(10.0)
                                    .color(egui::Color32::from_rgb(160, 160, 160)),
                                );
                            });
                        } else {
                            // Split mode info
                            ui.label(
                                egui::RichText::new(format!(
                                    "Left: {}  |  Right: {}",
                                    overlay.mode.label(),
                                    dual.secondary.label()
                                ))
                                .size(10.0)
                                .color(egui::Color32::from_rgb(160, 160, 160)),
                            );
                        }

                        // Clear button
                        ui.add_space(4.0);
                        if ui
                            .button(
                                egui::RichText::new("Clear Secondary")
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(220, 100, 100)),
                            )
                            .clicked()
                        {
                            dual.secondary = OverlayMode::None;
                            dual.panel_open = false;
                        }
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dual_overlay_state_default_values() {
        let state = DualOverlayState::default();
        assert_eq!(state.secondary, OverlayMode::None);
        assert_eq!(state.mode, DualOverlayMode::Blend);
        assert!((state.blend_factor - 0.5).abs() < f32::EPSILON);
        assert!(!state.panel_open);
    }

    #[test]
    fn dual_overlay_active_requires_both_non_none() {
        let state = DualOverlayState {
            secondary: OverlayMode::Traffic,
            ..Default::default()
        };
        assert!(state.is_active(OverlayMode::Pollution));
        assert!(!state.is_active(OverlayMode::None));
    }
}
