//! PLAY-012: Settings Menu (Audio, Graphics, Controls).
//!
//! A full-screen settings menu accessible from both the main menu and
//! the pause menu. Provides audio volume sliders (master, music, SFX, UI)
//! and a mute toggle, with placeholder sections for graphics and controls.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::audio_settings::AudioSettings;

use crate::theme;

// =============================================================================
// Resources
// =============================================================================

/// Tracks whether the settings menu is open and where it was opened from.
#[derive(Resource, Default)]
pub struct SettingsMenuOpen {
    /// Whether the settings menu is currently visible.
    pub open: bool,
    /// `true` when opened from the main menu, `false` from the pause menu.
    pub from_main_menu: bool,
}

/// Which tab is active in the settings menu.
#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum SettingsTab {
    #[default]
    Audio,
    Graphics,
    Controls,
}

/// Tracks the currently selected tab.
#[derive(Resource, Default)]
struct SettingsTabState {
    tab: SettingsTab,
}

// =============================================================================
// Plugin
// =============================================================================

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsMenuOpen>();
        app.init_resource::<SettingsTabState>();
        app.add_systems(
            Update,
            settings_menu_ui.run_if(|menu: Res<SettingsMenuOpen>| menu.open),
        );
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Renders the settings menu with a dark overlay and centered window.
fn settings_menu_ui(
    mut contexts: EguiContexts,
    mut menu: ResMut<SettingsMenuOpen>,
    mut tab_state: ResMut<SettingsTabState>,
    mut audio: ResMut<AudioSettings>,
) {
    let ctx = contexts.ctx_mut();

    // Semi-transparent dark overlay covering the entire screen.
    let screen_rect = ctx.screen_rect();
    egui::Area::new(egui::Id::new("settings_overlay"))
        .fixed_pos(screen_rect.min)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            painter.rect_filled(
                screen_rect,
                egui::CornerRadius::ZERO,
                egui::Color32::from_black_alpha(180),
            );
            ui.allocate_rect(screen_rect, egui::Sense::click());
        });

    // Centered settings window.
    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(400.0)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                ui.add_space(8.0);

                // Title
                ui.label(
                    egui::RichText::new("Settings")
                        .size(28.0)
                        .strong()
                        .color(theme::TEXT_HEADING),
                );
                ui.add_space(12.0);

                // Tab bar
                render_tab_bar(ui, &mut tab_state);
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Tab content
                match tab_state.tab {
                    SettingsTab::Audio => render_audio_tab(ui, &mut audio),
                    SettingsTab::Graphics => render_graphics_tab(ui),
                    SettingsTab::Controls => render_controls_tab(ui),
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                // Back button
                let back_size = egui::Vec2::new(200.0, 36.0);
                if ui
                    .add_sized(back_size, egui::Button::new("Back"))
                    .clicked()
                {
                    menu.open = false;
                }

                ui.add_space(8.0);
            });
        });
}

// =============================================================================
// Rendering helpers
// =============================================================================

/// Renders the tab selection bar (Audio / Graphics / Controls).
fn render_tab_bar(ui: &mut egui::Ui, tab_state: &mut ResMut<SettingsTabState>) {
    ui.horizontal(|ui| {
        let tabs = [
            (SettingsTab::Audio, "Audio"),
            (SettingsTab::Graphics, "Graphics"),
            (SettingsTab::Controls, "Controls"),
        ];

        for (tab, label) in &tabs {
            let selected = tab_state.tab == *tab;
            let text = egui::RichText::new(*label)
                .size(theme::FONT_SUBHEADING)
                .color(if selected {
                    theme::PRIMARY
                } else {
                    theme::TEXT_MUTED
                });
            if ui.selectable_label(selected, text).clicked() {
                tab_state.tab = *tab;
            }
        }
    });
}

/// Renders the audio settings tab with volume sliders and mute toggle.
fn render_audio_tab(ui: &mut egui::Ui, audio: &mut ResMut<AudioSettings>) {
    ui.label(
        egui::RichText::new("Volume Controls")
            .size(theme::FONT_SUBHEADING)
            .color(theme::TEXT_HEADING),
    );
    ui.add_space(8.0);

    // Master Volume
    let mut master_pct = audio.master_volume * 100.0;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Master")
                .size(theme::FONT_BODY)
                .color(theme::TEXT),
        );
        ui.add(
            egui::Slider::new(&mut master_pct, 0.0..=100.0)
                .suffix("%")
                .fixed_decimals(0),
        );
    });
    audio.set_master_volume(master_pct / 100.0);

    // Music Volume
    let mut music_pct = audio.music_volume * 100.0;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Music")
                .size(theme::FONT_BODY)
                .color(theme::TEXT),
        );
        ui.add(
            egui::Slider::new(&mut music_pct, 0.0..=100.0)
                .suffix("%")
                .fixed_decimals(0),
        );
    });
    audio.set_music_volume(music_pct / 100.0);

    // SFX Volume
    let mut sfx_pct = audio.sfx_volume * 100.0;
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("SFX")
                .size(theme::FONT_BODY)
                .color(theme::TEXT),
        );
        ui.add(
            egui::Slider::new(&mut sfx_pct, 0.0..=100.0)
                .suffix("%")
                .fixed_decimals(0),
        );
    });
    audio.set_sfx_volume(sfx_pct / 100.0);

    // UI Volume
    let mut ui_pct = audio.ui_volume * 100.0;
    ui.horizontal(|inner_ui| {
        inner_ui.label(
            egui::RichText::new("UI")
                .size(theme::FONT_BODY)
                .color(theme::TEXT),
        );
        inner_ui.add(
            egui::Slider::new(&mut ui_pct, 0.0..=100.0)
                .suffix("%")
                .fixed_decimals(0),
        );
    });
    audio.set_ui_volume(ui_pct / 100.0);

    ui.add_space(8.0);

    // Mute toggle
    let mut muted = audio.muted;
    ui.checkbox(&mut muted, "Mute All Audio");
    if muted != audio.muted {
        audio.toggle_mute();
    }

    ui.add_space(4.0);

    // Effective volume display
    if audio.muted {
        ui.label(
            egui::RichText::new("All audio is muted")
                .size(theme::FONT_SMALL)
                .color(theme::WARNING),
        );
    } else {
        let eff_text = format!(
            "Effective: Music {:.0}% | SFX {:.0}% | UI {:.0}%",
            audio.effective_music_volume() * 100.0,
            audio.effective_sfx_volume() * 100.0,
            audio.effective_ui_volume() * 100.0,
        );
        ui.label(
            egui::RichText::new(eff_text)
                .size(theme::FONT_SMALL)
                .color(theme::TEXT_MUTED),
        );
    }
}

/// Renders the graphics settings tab (placeholder).
fn render_graphics_tab(ui: &mut egui::Ui) {
    ui.add_space(20.0);
    ui.label(
        egui::RichText::new("Graphics settings coming soon")
            .size(theme::FONT_BODY)
            .color(theme::TEXT_MUTED)
            .italics(),
    );
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(
            "Future options: resolution, quality presets, LOD distance, shadows, anti-aliasing.",
        )
        .size(theme::FONT_SMALL)
        .color(theme::TEXT_MUTED),
    );
    ui.add_space(20.0);
}

/// Renders the controls settings tab (placeholder).
fn render_controls_tab(ui: &mut egui::Ui) {
    ui.add_space(20.0);
    ui.label(
        egui::RichText::new("Control settings coming soon")
            .size(theme::FONT_BODY)
            .color(theme::TEXT_MUTED)
            .italics(),
    );
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(
            "Future options: camera sensitivity, scroll speed, key rebinding.",
        )
        .size(theme::FONT_SMALL)
        .color(theme::TEXT_MUTED),
    );
    ui.add_space(20.0);
}
