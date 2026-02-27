//! UX-008 / PLAY-023: Notification Toast System.
//!
//! Renders a vertical stack of toast notifications (up to 5) in the top-right
//! corner, color-coded by priority using theme colors. Toasts auto-dismiss
//! after a configurable real-time duration and can be dismissed early by
//! clicking. Emergency notifications stay longer. Clicking a notification
//! with a location also jumps the camera there.
//!
//! Also provides a notification journal window (toggled via button) showing
//! the full history of past notifications.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use simulation::app_state::AppState;

use crate::theme;
use rendering::camera::OrbitCamera;
use simulation::notifications::{NotificationLog, NotificationPriority};

const MAX_VISIBLE_TOASTS: usize = 5;
const TOAST_DURATION_DEFAULT: f32 = 5.0;
const TOAST_DURATION_CRITICAL: f32 = 15.0;
const TOAST_DURATION_WARNING: f32 = 8.0;
const TOAST_WIDTH: f32 = 360.0;
const TOAST_RIGHT_MARGIN: f32 = 16.0;
const TOAST_TOP_MARGIN: f32 = 48.0;
const TOAST_SPACING: f32 = 4.0;

/// Tracks visibility of the notification journal window.
#[derive(Resource, Default)]
pub struct NotificationJournalVisible(pub bool);

/// Per-toast real-time timer state, keyed by notification ID.
#[derive(Resource, Default)]
pub struct ToastTimers {
    pub timers: Vec<(u64, f32)>,
}

impl ToastTimers {
    /// Ensure a timer exists for the given notification; returns true if new.
    pub fn ensure_timer(&mut self, id: u64, duration: f32) -> bool {
        if self.timers.iter().any(|(tid, _)| *tid == id) {
            return false;
        }
        self.timers.push((id, duration));
        true
    }

    /// Tick all timers by `dt` seconds and return IDs that have expired.
    pub fn tick(&mut self, dt: f32) -> Vec<u64> {
        let mut expired = Vec::new();
        for (id, remaining) in &mut self.timers {
            *remaining -= dt;
            if *remaining <= 0.0 {
                expired.push(*id);
            }
        }
        self.timers.retain(|(_, r)| *r > 0.0);
        expired
    }

    /// Remove a timer by ID (e.g., when user dismisses).
    pub fn remove(&mut self, id: u64) {
        self.timers.retain(|(tid, _)| *tid != id);
    }
}

fn priority_color(priority: NotificationPriority) -> egui::Color32 {
    match priority {
        NotificationPriority::Emergency => theme::ERROR,
        NotificationPriority::Warning | NotificationPriority::Attention => theme::WARNING,
        NotificationPriority::Info => theme::TEXT,
        NotificationPriority::Positive => theme::SUCCESS,
    }
}

fn priority_bg(priority: NotificationPriority) -> egui::Color32 {
    match priority {
        NotificationPriority::Emergency => egui::Color32::from_rgba_premultiplied(60, 20, 20, 230),
        NotificationPriority::Warning => egui::Color32::from_rgba_premultiplied(50, 40, 15, 230),
        NotificationPriority::Attention => egui::Color32::from_rgba_premultiplied(45, 40, 20, 230),
        NotificationPriority::Info => egui::Color32::from_rgba_premultiplied(25, 27, 35, 230),
        NotificationPriority::Positive => egui::Color32::from_rgba_premultiplied(20, 50, 25, 230),
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

fn toast_duration(priority: NotificationPriority) -> f32 {
    match priority {
        NotificationPriority::Emergency => TOAST_DURATION_CRITICAL,
        NotificationPriority::Warning => TOAST_DURATION_WARNING,
        _ => TOAST_DURATION_DEFAULT,
    }
}

/// Ticks toast timers and dismisses expired toasts from the notification log.
pub fn tick_toast_timers(
    mut timers: ResMut<ToastTimers>,
    mut log: ResMut<NotificationLog>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    for notif in &log.active {
        timers.ensure_timer(notif.id, toast_duration(notif.priority));
    }
    let active_ids: Vec<u64> = log.active.iter().map(|n| n.id).collect();
    timers.timers.retain(|(id, _)| active_ids.contains(id));
    let expired = timers.tick(dt);
    for id in expired {
        log.dismiss(id);
    }
}

/// Renders vertically stacked toast notifications in the top-right corner.
#[allow(clippy::too_many_arguments)]
pub fn notification_toast_ui(
    mut contexts: EguiContexts,
    mut log: ResMut<NotificationLog>,
    mut orbit: ResMut<OrbitCamera>,
    mut journal_visible: ResMut<NotificationJournalVisible>,
    mut timers: ResMut<ToastTimers>,
) {
    let active_count = log.active.len();
    if active_count == 0 && !journal_visible.0 {
        return;
    }

    let mut jump_target: Option<(f32, f32)> = None;
    let mut dismiss_id: Option<u64> = None;
    let ctx = contexts.ctx_mut();
    let screen_width = ctx.screen_rect().width();

    let mut sorted_indices: Vec<usize> = (0..log.active.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        let na = &log.active[a];
        let nb = &log.active[b];
        na.priority
            .cmp(&nb.priority)
            .then(nb.created_tick.cmp(&na.created_tick))
    });
    sorted_indices.truncate(MAX_VISIBLE_TOASTS);

    for (toast_idx, &notif_idx) in sorted_indices.iter().enumerate() {
        let notif = &log.active[notif_idx];
        let toast_id = notif.id;
        let color = priority_color(notif.priority);
        let bg = priority_bg(notif.priority);
        let icon = priority_icon(notif.priority);
        let label_text = format!("{} {}", icon, notif.text);
        let priority = notif.priority;
        let location = notif.location;
        let day = notif.day;
        let hour = notif.hour;

        let y_pos = TOAST_TOP_MARGIN + (toast_idx as f32) * (36.0 + TOAST_SPACING);
        let x_pos = screen_width - TOAST_WIDTH - TOAST_RIGHT_MARGIN;

        egui::Area::new(egui::Id::new(("toast", toast_id)))
            .fixed_pos(egui::pos2(x_pos, y_pos))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(bg)
                    .stroke(egui::Stroke::new(1.0, color))
                    .corner_radius(egui::CornerRadius::same(4))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.set_min_width(TOAST_WIDTH - 20.0);
                        ui.set_max_width(TOAST_WIDTH - 20.0);
                        ui.horizontal(|ui| {
                            let response = ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&label_text)
                                        .color(color)
                                        .size(theme::FONT_BODY),
                                )
                                .sense(egui::Sense::click()),
                            );
                            if response.clicked() {
                                if let Some(loc) = location {
                                    jump_target = Some(loc);
                                }
                                dismiss_id = Some(toast_id);
                            }
                            let h = hour as u32;
                            let m = ((hour.fract()) * 60.0) as u32;
                            response.on_hover_text(format!(
                                "{} — Day {} {:02}:{:02}\nClick to dismiss{}",
                                priority.label(),
                                day,
                                h,
                                m,
                                if location.is_some() {
                                    " and jump to location"
                                } else {
                                    ""
                                },
                            ));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("x")
                                                    .color(theme::TEXT_MUTED)
                                                    .size(theme::FONT_SMALL),
                                            )
                                            .frame(false),
                                        )
                                        .clicked()
                                    {
                                        dismiss_id = Some(toast_id);
                                    }
                                },
                            );
                        });
                    });
            });
    }

    // Journal toggle button
    if active_count > 0 || journal_visible.0 {
        egui::Area::new(egui::Id::new("toast_journal_toggle"))
            .fixed_pos(egui::pos2(8.0, TOAST_TOP_MARGIN))
            .order(egui::Order::Middle)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let journal_btn = if journal_visible.0 {
                        "Journal [v]"
                    } else {
                        "Journal [>]"
                    };
                    if ui.small_button(journal_btn).clicked() {
                        journal_visible.0 = !journal_visible.0;
                    }
                    if active_count > 0 {
                        ui.label(
                            egui::RichText::new(format!("({} active)", active_count))
                                .small()
                                .color(theme::TEXT_MUTED),
                        );
                    }
                });
            });
    }

    if let Some((wx, wz)) = jump_target {
        orbit.focus.x = wx;
        orbit.focus.z = wz;
        orbit.distance = orbit.distance.min(400.0);
    }
    if let Some(id) = dismiss_id {
        log.dismiss(id);
        timers.remove(id);
    }
}

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
                                    .color(theme::TEXT_MUTED),
                            );
                            let label_text = format!("{} {}", icon, entry.text);
                            let response = ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&label_text).color(color),
                                )
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

pub struct NotificationTickerPlugin;

impl Plugin for NotificationTickerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationJournalVisible>()
            .init_resource::<ToastTimers>()
            .add_systems(
                Update,
                (
                    tick_toast_timers,
                    notification_toast_ui,
                    notification_journal_ui,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_colors_use_theme() {
        assert_eq!(priority_color(NotificationPriority::Emergency), theme::ERROR);
        assert_eq!(priority_color(NotificationPriority::Warning), theme::WARNING);
        assert_eq!(priority_color(NotificationPriority::Info), theme::TEXT);
        assert_eq!(priority_color(NotificationPriority::Positive), theme::SUCCESS);
    }

    #[test]
    fn test_priority_colors_distinct() {
        let colors = [
            priority_color(NotificationPriority::Emergency),
            priority_color(NotificationPriority::Warning),
            priority_color(NotificationPriority::Info),
            priority_color(NotificationPriority::Positive),
        ];
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
    fn test_toast_timers_ensure_and_tick() {
        let mut timers = ToastTimers::default();
        assert!(timers.ensure_timer(1, 5.0));
        assert!(!timers.ensure_timer(1, 5.0));
        assert!(timers.ensure_timer(2, 3.0));
        let expired = timers.tick(4.0);
        assert!(expired.contains(&2));
        assert!(!expired.contains(&1));
        assert_eq!(timers.timers.len(), 1);
    }

    #[test]
    fn test_toast_timers_remove() {
        let mut timers = ToastTimers::default();
        timers.ensure_timer(1, 5.0);
        timers.ensure_timer(2, 5.0);
        timers.remove(1);
        assert_eq!(timers.timers.len(), 1);
        assert_eq!(timers.timers[0].0, 2);
    }

    #[test]
    fn test_toast_duration_critical_longer() {
        assert!(
            toast_duration(NotificationPriority::Emergency)
                > toast_duration(NotificationPriority::Info)
        );
        assert!(
            toast_duration(NotificationPriority::Warning)
                > toast_duration(NotificationPriority::Info)
        );
    }

    #[test]
    fn test_priority_bg_distinct_for_key_priorities() {
        let bgs = [
            priority_bg(NotificationPriority::Emergency),
            priority_bg(NotificationPriority::Info),
            priority_bg(NotificationPriority::Positive),
        ];
        for i in 0..bgs.len() {
            for j in (i + 1)..bgs.len() {
                assert_ne!(bgs[i], bgs[j], "Priority backgrounds must be distinct");
            }
        }
    }
}
