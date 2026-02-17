use bevy_egui::{egui, EguiContexts};

pub fn apply_cute_theme(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    let mut style = (*ctx.style()).clone();

    // Dark warm background
    let panel = egui::Color32::from_rgb(35, 37, 48);
    let inactive = egui::Color32::from_rgb(50, 55, 65);
    let hover = egui::Color32::from_rgb(70, 80, 100);
    let active = egui::Color32::from_rgb(100, 160, 220);

    style.visuals.widgets.noninteractive.bg_fill = panel;
    style.visuals.widgets.inactive.bg_fill = inactive;
    style.visuals.widgets.hovered.bg_fill = hover;
    style.visuals.widgets.active.bg_fill = active;
    style.visuals.widgets.inactive.weak_bg_fill = inactive;
    style.visuals.widgets.hovered.weak_bg_fill = hover;
    style.visuals.widgets.active.weak_bg_fill = active;

    style.visuals.window_fill = panel;
    style.visuals.panel_fill = panel;
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(30, 32, 40);
    style.visuals.faint_bg_color = egui::Color32::from_rgb(40, 42, 52);

    // Selection highlight
    style.visuals.selection.bg_fill = active;
    style.visuals.selection.stroke = egui::Stroke::new(1.0, active);

    // Rounded corners (egui 0.31+ uses CornerRadius with u8 values)
    let window_rounding = egui::CornerRadius::same(8);
    let widget_rounding = egui::CornerRadius::same(6);

    style.visuals.window_corner_radius = window_rounding;
    style.visuals.widgets.noninteractive.corner_radius = widget_rounding;
    style.visuals.widgets.inactive.corner_radius = widget_rounding;
    style.visuals.widgets.hovered.corner_radius = widget_rounding;
    style.visuals.widgets.active.corner_radius = widget_rounding;

    ctx.set_style(style);
}
