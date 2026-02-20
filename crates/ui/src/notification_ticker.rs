//! UX-008: Notification Ticker UI.
//!
//! Renders a horizontal scrolling ticker below the HUD toolbar bar showing
//! active notifications color-coded by priority. Clicking a notification
//! jumps the camera to its location. Emergency notifications can be dismissed
//! with a close button; lower-priority notifications auto-expire.
//!
//! Also provides a notification journal window (toggled via button) showing
//! the full history of past notifications.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::camera::OrbitCamera;
use simulation::notifications::{NotificationLog, NotificationPriority};

// =============================================================================
// Constants
// =============================================================================

/// Height of the ticker bar in pixels.
const TICKER_HEIGHT: f32 = 28.0;

/// Y offset from top (below the main toolbar).
const TICKER_Y_OFFSET: f32 = 36.0;

// =============================================================================
// Resources
// =============================================================================

/// Tracks visibility of the notification journal window.
#[derive(Resource, Default)]
pub struct NotificationJournalVisible(pub bool);

/// Horizontal scroll offset for the ticker (auto-scrolls).
#[derive(Resource)]
pub struct TickerScroll {
    pub offset: f32,
}

impl Default for TickerScroll {
    fn default() -> Self {
        Self { offset: 0.0 }
    }
}

// =============================================================================
// Color mapping
// =============================================================================

fn priority_color(priority: NotificationPriority) -> egui::Color32 {
    match priority {
        NotificationPriority::Emergency => egui::Color32::from_rgb(255, 60, 60),
        NotificationPriority::Warning => egui::Color32::from_rgb(255, 165, 0),
        NotificationPriority::Attention => egui::Color32::from_rgb(255, 220, 50),
        NotificationPriority::Info => egui::Color32::from_rgb(220, 220, 220),
        NotificationPriority::Positive => egui::Color32::from_rgb(80, 220, 80),
    }
}

fn priority_icon(priority: NotificationPriority) -> &'static str {
    match priority {
        NotificationPriority::Emergency => "[!]",
        NotificationPriority::Warning => "[W]",
        NotificationPriority::Attention => "[*]",
        NotificationPriority::Info => "[i]",
        NotificationPriority::Positive => "[+]",
    }
}

// =============================================================================
// Ticker System
// =============================================================================

/// Renders the notification ticker bar below the HUD toolbar.
#[allow(clippy::too_many_arguments)]
pub fn notification_ticker_ui(
    mut contexts: EguiContexts,
    mut log: ResMut<NotificationLog>,
    mut orbit: ResMut<OrbitCamera>,
    mut journal_visible: ResMut<NotificationJournalVisible>,
    mut scroll: ResMut<TickerScroll>,
    time: Res<Time>,
) {
    let active_count = log.active.len();
    if active_count == 0 && !journal_visible.0 {
        // Nothing to show — reset scroll
        scroll.offset = 0.0;
        return;
    }

    // Auto-scroll the ticker
    scroll.offset += time.delta_secs() * 30.0;

    let mut jump_target: Option<(f32, f32)> = None;
    let mut dismiss_id: Option<u64> = None;

    // Render ticker bar as a top panel offset below the toolbar
    egui::Area::new(egui::Id::new("notification_ticker"))
        .fixed_pos(egui::pos2(0.0, TICKER_Y_OFFSET))
        .order(egui::Order::Middle)
        .show(contexts.ctx_mut(), |ui| {
            let screen_width = ui.ctx().screen_rect().width();

            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(20, 20, 30, 220))
                .inner_margin(egui::Margin::symmetric(6, 4))
                .show(ui, |ui| {
                    ui.set_min_width(screen_width);
                    ui.set_max_height(TICKER_HEIGHT);

                    ui.horizontal(|ui| {
                        // Journal toggle button
                        let journal_btn = if journal_visible.0 {
                            "Journal [v]"
                        } else {
                            "Journal [>]"
                        };
                        if ui.small_button(journal_btn).clicked() {
                            journal_visible.0 = !journal_visible.0;
                        }

                        ui.separator();

                        // Show active notification count
                        if active_count > 0 {
                            ui.label(
                                egui::RichText::new(format!("({})", active_count))
                                    .small()
                                    .color(egui::Color32::from_rgb(180, 180, 180)),
                            );
                        }

                        // Scrollable area for notifications
                        egui::ScrollArea::horizontal()
                            .scroll_offset(egui::vec2(scroll.offset, 0.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Sort: emergency first, then by creation (newest first)
                                    let mut sorted_indices: Vec<usize> =
                                        (0..log.active.len()).collect();
                                    sorted_indices.sort_by(|&a, &b| {
                                        let na = &log.active[a];
                                        let nb = &log.active[b];
                                        na.priority
                                            .cmp(&nb.priority)
                                            .then(nb.created_tick.cmp(&na.created_tick))
                                    });

                                    for &idx in &sorted_indices {
                                        let notif = &log.active[idx];
                                        let color = priority_color(notif.priority);
                                        let icon = priority_icon(notif.priority);

                                        let label_text = format!("{} {}", icon, notif.text);
                                        let response = ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(&label_text).color(color),
                                            )
                                            .sense(egui::Sense::click()),
                                        );

                                        if response.clicked() {
                                            if let Some(loc) = notif.location {
                                                jump_target = Some(loc);
                                            }
                                        }
                                        response.on_hover_text(format!(
                                            "{} — Day {} {:02}:{:02}{}",
                                            notif.priority.label(),
                                            notif.day,
                                            notif.hour as u32,
                                            ((notif.hour.fract()) * 60.0) as u32,
                                            if notif.location.is_some() {
                                                " (click to jump)"
                                            } else {
                                                ""
                                            },
                                        ));

                                        // Dismiss button for emergency notifications
                                        if notif.priority == NotificationPriority::Emergency {
                                            if ui.small_button("x").clicked() {
                                                dismiss_id = Some(notif.id);
                                            }
                                        }

                                        ui.add_space(12.0);
                                    }
                                });
                            });
                    });
                });
        });

    // Apply camera jump
    if let Some((wx, wz)) = jump_target {
        orbit.focus.x = wx;
        orbit.focus.z = wz;
        orbit.distance = orbit.distance.min(400.0);
    }

    // Apply dismiss
    if let Some(id) = dismiss_id {
        log.dismiss(id);
    }
}

// =============================================================================
// Journal Window System
// =============================================================================

/// Renders the notification journal window showing full history.
pub fn notification_journal_ui(
    mut contexts: EguiContexts,
    log: Res<NotificationLog>,
    mut orbit: ResMut<OrbitCamera>,
    visible: Res<NotificationJournalVisible>,
) {
    if !visible.0 {
        return;
    }

    let mut jump_target: Option<(f32, f32)> = None;

    egui::Window::new("Notification Journal")
        .default_width(400.0)
        .default_height(350.0)
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(8.0, -8.0))
        .resizable(true)
        .collapsible(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(format!("{} entries", log.journal.len()));
            ui.separator();

            if log.journal.is_empty() {
                ui.label("No notifications recorded yet.");
                return;
            }

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    // Show most recent first
                    for entry in log.journal.iter().rev() {
                        let color = priority_color(entry.priority);
                        let icon = priority_icon(entry.priority);
                        let h = entry.hour as u32;
                        let m = ((entry.hour.fract()) * 60.0) as u32;
                        let time_str = format!("Day {} {:02}:{:02}", entry.day, h, m);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&time_str)
                                    .small()
                                    .color(egui::Color32::from_rgb(150, 150, 150)),
                            );
                            let label_text = format!("{} {}", icon, entry.text);
                            let response = ui.add(
                                egui::Label::new(egui::RichText::new(&label_text).color(color))
                                    .sense(egui::Sense::click()),
                            );

                            if response.clicked() {
                                if let Some(loc) = entry.location {
                                    jump_target = Some(loc);
                                }
                            }
                            if entry.location.is_some() {
                                response.on_hover_text("Click to jump to location");
                            }
                        });
                        ui.add_space(1.0);
                    }
                });
        });

    if let Some((wx, wz)) = jump_target {
        orbit.focus.x = wx;
        orbit.focus.z = wz;
        orbit.distance = orbit.distance.min(400.0);
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct NotificationTickerPlugin;

impl Plugin for NotificationTickerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationJournalVisible>()
            .init_resource::<TickerScroll>()
            .add_systems(Update, (notification_ticker_ui, notification_journal_ui));
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_colors_distinct() {
        let colors = [
            priority_color(NotificationPriority::Emergency),
            priority_color(NotificationPriority::Warning),
            priority_color(NotificationPriority::Attention),
            priority_color(NotificationPriority::Info),
            priority_color(NotificationPriority::Positive),
        ];
        // Ensure all are distinct
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j], "Priority colors must be distinct");
            }
        }
    }

    #[test]
    fn test_priority_icons_distinct() {
        let icons = [
            priority_icon(NotificationPriority::Emergency),
            priority_icon(NotificationPriority::Warning),
            priority_icon(NotificationPriority::Attention),
            priority_icon(NotificationPriority::Info),
            priority_icon(NotificationPriority::Positive),
        ];
        for i in 0..icons.len() {
            for j in (i + 1)..icons.len() {
                assert_ne!(icons[i], icons[j], "Priority icons must be distinct");
            }
        }
    }

    #[test]
    fn test_journal_visible_default() {
        let visible = NotificationJournalVisible::default();
        assert!(!visible.0);
    }

    #[test]
    fn test_ticker_scroll_default() {
        let scroll = TickerScroll::default();
        assert_eq!(scroll.offset, 0.0);
    }
}
