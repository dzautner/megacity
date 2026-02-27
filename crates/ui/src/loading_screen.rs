//! Loading and transition screen overlays (PLAY-011).
//!
//! Displays a full-screen semi-transparent overlay with a contextual message
//! during save, load, and new-game operations. An animated dots effect
//! provides visual feedback so the player knows the application has not frozen.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::SaveLoadState;

use crate::theme;

// =============================================================================
// Resources
// =============================================================================

/// Tracks the animated dots state for the loading message.
#[derive(Resource)]
pub struct LoadingAnimation {
    /// Number of dots currently shown (cycles 1 -> 2 -> 3 -> 1 ...).
    pub dots: usize,
    /// Timer controlling the animation speed.
    pub timer: Timer,
}

impl Default for LoadingAnimation {
    fn default() -> Self {
        Self {
            dots: 1,
            timer: Timer::from_seconds(0.4, TimerMode::Repeating),
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingAnimation>();
        app.add_systems(Update, loading_screen_ui);
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Renders a loading overlay when a save/load/new-game operation is active.
///
/// The system checks the current `SaveLoadState` and, if it is anything other
/// than `Idle`, draws a full-screen dark overlay with a centered message.
fn loading_screen_ui(
    mut contexts: EguiContexts,
    state: Res<State<SaveLoadState>>,
    time: Res<Time>,
    mut animation: ResMut<LoadingAnimation>,
) {
    let label = match state.get() {
        SaveLoadState::Idle => {
            // Reset animation so it starts fresh next time.
            animation.dots = 1;
            animation.timer.reset();
            return;
        }
        SaveLoadState::Saving => "Saving",
        SaveLoadState::Loading => "Loading",
        SaveLoadState::NewGame => "Generating World",
    };

    // Advance the dots animation.
    animation.timer.tick(time.delta());
    if animation.timer.just_finished() {
        animation.dots = animation.dots % 3 + 1;
    }

    let dots_str = ".".repeat(animation.dots);
    let display_text = format!("{label}{dots_str}");

    let ctx = contexts.ctx_mut();
    let screen_rect = ctx.screen_rect();

    // Full-screen semi-transparent overlay — sits above everything else.
    egui::Area::new(egui::Id::new("loading_overlay"))
        .fixed_pos(screen_rect.min)
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            let painter = ui.painter();
            painter.rect_filled(
                screen_rect,
                egui::CornerRadius::ZERO,
                egui::Color32::from_black_alpha(180),
            );
            // Allocate the full rect so the area consumes input.
            ui.allocate_rect(screen_rect, egui::Sense::click_and_drag());
        });

    // Centered message window (no title bar, no chrome).
    egui::Window::new("loading_screen_window")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(260.0)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new(display_text)
                        .size(theme::FONT_HEADING)
                        .color(theme::TEXT_HEADING),
                );
                ui.add_space(16.0);
            });
        });
}
