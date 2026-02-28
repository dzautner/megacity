//! Minimal watch-only UI for replay viewer mode.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::app_state::AppState;
use simulation::replay::ReplayViewerInfo;
use simulation::time_of_day::GameClock;
use simulation::TickCounter;

pub struct ReplayViewerUiPlugin;

impl Plugin for ReplayViewerUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            replay_viewer_ui.run_if(in_state(AppState::Playing)),
        );
    }
}

fn replay_viewer_ui(
    mut contexts: EguiContexts,
    mut clock: ResMut<GameClock>,
    tick: Res<TickCounter>,
    info: Option<Res<ReplayViewerInfo>>,
) {
    let ctx = contexts.ctx_mut();
    egui::TopBottomPanel::top("replay_viewer_controls").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("Replay Viewer");
            ui.separator();

            let play_label = if clock.paused { "Play" } else { "Pause" };
            if ui.button(play_label).clicked() {
                clock.paused = !clock.paused;
            }
            if ui.button("1x").clicked() {
                clock.paused = false;
                clock.speed = 1.0;
            }
            if ui.button("2x").clicked() {
                clock.paused = false;
                clock.speed = 2.0;
            }
            if ui.button("4x").clicked() {
                clock.paused = false;
                clock.speed = 4.0;
            }

            #[cfg(target_arch = "wasm32")]
            if ui.button("Reload").clicked() {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().reload();
                }
            }

            ui.separator();
            ui.label(format!("Tick: {}", tick.0));
            ui.label(if clock.paused {
                "State: Paused".to_string()
            } else {
                format!("State: Playing @ {:.0}x", clock.speed)
            });

            if let Some(info) = info {
                let denom = info.end_tick.max(1);
                let progress = (tick.0 as f64 / denom as f64).clamp(0.0, 1.0) as f32;
                ui.add(
                    egui::ProgressBar::new(progress)
                        .desired_width(180.0)
                        .show_percentage(),
                );
                ui.label(format!("End tick: {}", info.end_tick));
                ui.label(format!("Entries: {}", info.entry_count));
                ui.label(format!("Replay: {}", info.source));
            }
        });

        ui.small("Camera controls: WASD/Arrow pan, drag to pan/orbit, wheel zoom.");
    });
}
