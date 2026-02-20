//! Day/Night cycle visual controls UI panel (UX-069).
//!
//! Provides an egui window with:
//! - Time-of-day slider (0..24h)
//! - Lock/unlock toggle to freeze the visual hour
//! - Cycle speed selector (Normal / Fast / Disabled)
//! - Keybind (N) to toggle the panel

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::day_night_controls::{CycleSpeed, DayNightControls};

// =============================================================================
// Resources
// =============================================================================

/// Whether the day/night controls panel is visible.
#[derive(Resource, Default)]
pub struct DayNightPanelVisible(pub bool);

// =============================================================================
// Systems
// =============================================================================

/// Renders the day/night controls window.
pub fn day_night_panel_ui(
    mut contexts: EguiContexts,
    mut visible: ResMut<DayNightPanelVisible>,
    mut controls: ResMut<DayNightControls>,
) {
    if !visible.0 {
        return;
    }

    let mut open = true;
    egui::Window::new("Day/Night Controls")
        .open(&mut open)
        .resizable(false)
        .default_width(260.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;

            // --- Current visual hour display ---
            let effective = controls.effective_hour();
            let h = effective as u32;
            let m = ((effective - h as f32) * 60.0) as u32;
            let period = time_period_label(effective);
            ui.heading(format!("{:02}:{:02} ({})", h, m, period));

            ui.separator();

            // --- Time-of-day slider ---
            ui.label("Time of Day:");
            let mut slider_hour = controls.effective_hour();
            let slider_response = ui.add(
                egui::Slider::new(&mut slider_hour, 0.0..=23.99)
                    .text("hour")
                    .custom_formatter(|v, _| {
                        let hh = v as u32;
                        let mm = ((v - hh as f64) * 60.0) as u32;
                        format!("{:02}:{:02}", hh, mm)
                    }),
            );
            if slider_response.changed() {
                // When the player manually drags the slider, lock to that hour
                controls.locked_hour = Some(slider_hour);
                controls.visual_hour = slider_hour;
            }

            ui.separator();

            // --- Lock time toggle ---
            let is_locked = controls.locked_hour.is_some();
            let lock_label = if is_locked {
                "Locked (click to unlock)"
            } else {
                "Unlocked (click to lock at current time)"
            };
            if ui.selectable_label(is_locked, lock_label).clicked() {
                if is_locked {
                    controls.locked_hour = None;
                } else {
                    controls.locked_hour = Some(controls.visual_hour);
                }
            }

            ui.separator();

            // --- Cycle speed ---
            ui.label("Cycle Speed:");
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(controls.cycle_speed == CycleSpeed::Normal, "Normal")
                    .clicked()
                {
                    controls.cycle_speed = CycleSpeed::Normal;
                    // Unlock when changing speed (user wants to see the cycle)
                    if controls.locked_hour.is_some() {
                        controls.locked_hour = None;
                    }
                }
                if ui
                    .selectable_label(controls.cycle_speed == CycleSpeed::Fast, "Fast")
                    .clicked()
                {
                    controls.cycle_speed = CycleSpeed::Fast;
                    if controls.locked_hour.is_some() {
                        controls.locked_hour = None;
                    }
                }
                if ui
                    .selectable_label(controls.cycle_speed == CycleSpeed::Disabled, "Disabled")
                    .clicked()
                {
                    controls.cycle_speed = CycleSpeed::Disabled;
                }
            });

            // --- Quick presets ---
            ui.separator();
            ui.label("Quick Presets:");
            ui.horizontal(|ui| {
                if ui.button("Dawn (6:00)").clicked() {
                    controls.locked_hour = Some(6.0);
                    controls.visual_hour = 6.0;
                }
                if ui.button("Noon (12:00)").clicked() {
                    controls.locked_hour = Some(12.0);
                    controls.visual_hour = 12.0;
                }
                if ui.button("Dusk (18:00)").clicked() {
                    controls.locked_hour = Some(18.0);
                    controls.visual_hour = 18.0;
                }
                if ui.button("Night (0:00)").clicked() {
                    controls.locked_hour = Some(0.0);
                    controls.visual_hour = 0.0;
                }
            });
        });

    if !open {
        visible.0 = false;
    }
}

/// Returns a human-readable label for the time period.
fn time_period_label(hour: f32) -> &'static str {
    if (5.0..7.0).contains(&hour) {
        "Dawn"
    } else if (7.0..12.0).contains(&hour) {
        "Morning"
    } else if (12.0..14.0).contains(&hour) {
        "Midday"
    } else if (14.0..17.0).contains(&hour) {
        "Afternoon"
    } else if (17.0..19.0).contains(&hour) {
        "Dusk"
    } else if (19.0..22.0).contains(&hour) {
        "Evening"
    } else {
        "Night"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_period_labels() {
        assert_eq!(time_period_label(5.5), "Dawn");
        assert_eq!(time_period_label(9.0), "Morning");
        assert_eq!(time_period_label(12.5), "Midday");
        assert_eq!(time_period_label(15.0), "Afternoon");
        assert_eq!(time_period_label(18.0), "Dusk");
        assert_eq!(time_period_label(20.0), "Evening");
        assert_eq!(time_period_label(23.0), "Night");
        assert_eq!(time_period_label(2.0), "Night");
    }
}
