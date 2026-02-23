use bevy_egui::egui;

use simulation::zones::ZoneDemand;

// ---------------------------------------------------------------------------
// Population formatting
// ---------------------------------------------------------------------------

pub(crate) fn format_pop(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

pub(crate) fn milestone_name(pop: u32) -> &'static str {
    const POPS: &[(u32, &str)] = &[
        (1_000_000, "World Capital"),
        (500_000, "Megalopolis"),
        (250_000, "Megacity"),
        (100_000, "Major Metropolis"),
        (50_000, "Metropolis"),
        (25_000, "Large City"),
        (10_000, "City"),
        (5_000, "Small City"),
        (1_000, "Town"),
        (500, "Hamlet"),
        (100, "Village"),
    ];
    for &(threshold, name) in POPS {
        if pop >= threshold {
            return name;
        }
    }
    "Settlement"
}

// ---------------------------------------------------------------------------
// RCI Demand Bars
// ---------------------------------------------------------------------------

/// Draw a single vertical demand bar. `value` is in 0.0..=1.0.
/// 0.5 is the neutral midpoint: above 0.5 draws upward (demand), below draws
/// downward (surplus, shown in red).
fn demand_bar(ui: &mut egui::Ui, label: &str, value: f32, color: egui::Color32) {
    let bar_width = 8.0;
    let bar_height = 24.0;
    let midpoint = 0.5;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(bar_width + 12.0, bar_height),
        egui::Sense::hover(),
    );

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Bar background
        let bar_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x, rect.min.y),
            egui::vec2(bar_width, bar_height),
        );
        painter.rect_filled(bar_rect, 2.0, egui::Color32::from_gray(50));

        // Midpoint line
        let mid_y = bar_rect.min.y + bar_height * 0.5;
        painter.line_segment(
            [
                egui::pos2(bar_rect.min.x, mid_y),
                egui::pos2(bar_rect.max.x, mid_y),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_gray(120)),
        );

        // Filled portion
        let clamped = value.clamp(0.0, 1.0);
        if clamped > midpoint {
            // Demand: draw upward from midpoint
            let fill_frac = (clamped - midpoint) / midpoint;
            let fill_height = fill_frac * (bar_height * 0.5);
            let fill_rect = egui::Rect::from_min_max(
                egui::pos2(bar_rect.min.x + 1.0, mid_y - fill_height),
                egui::pos2(bar_rect.max.x - 1.0, mid_y),
            );
            painter.rect_filled(fill_rect, 1.0, color);
        } else if clamped < midpoint {
            // Surplus: draw downward from midpoint in red
            let fill_frac = (midpoint - clamped) / midpoint;
            let fill_height = fill_frac * (bar_height * 0.5);
            let fill_rect = egui::Rect::from_min_max(
                egui::pos2(bar_rect.min.x + 1.0, mid_y),
                egui::pos2(bar_rect.max.x - 1.0, mid_y + fill_height),
            );
            painter.rect_filled(fill_rect, 1.0, egui::Color32::from_rgb(220, 60, 50));
        }

        // Label to the right of the bar
        painter.text(
            egui::pos2(bar_rect.max.x + 2.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(10.0),
            color,
        );
    }

    // Tooltip with exact value on hover
    let pct = value * 100.0;
    let status = if value > 0.5 {
        "demand"
    } else if value < 0.5 {
        "surplus"
    } else {
        "balanced"
    };
    response.on_hover_text(format!("{label}: {pct:.0}% ({status})"));
}

pub(crate) fn rci_demand_bars(ui: &mut egui::Ui, demand: &ZoneDemand) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        demand_bar(
            ui,
            "R",
            demand.residential,
            egui::Color32::from_rgb(80, 200, 80),
        );
        demand_bar(
            ui,
            "C",
            demand.commercial,
            egui::Color32::from_rgb(80, 140, 220),
        );
        demand_bar(
            ui,
            "I",
            demand.industrial,
            egui::Color32::from_rgb(220, 200, 60),
        );
    });
}

// ---------------------------------------------------------------------------
// Speed button with color-coded dot indicator
// ---------------------------------------------------------------------------

/// Scale a `Color32` by a factor (0.0 = black, 1.0 = unchanged).
fn dim_color(c: egui::Color32, factor: f32) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(
        (c.r() as f32 * factor) as u8,
        (c.g() as f32 * factor) as u8,
        (c.b() as f32 * factor) as u8,
        c.a(),
    )
}

/// Renders a speed control button with a colored dot indicator.
/// When `active` the dot is filled and the label uses the accent color;
/// when inactive the dot is a dim outline.
pub(crate) fn speed_button(
    ui: &mut egui::Ui,
    label: &str,
    active: bool,
    color: egui::Color32,
) -> egui::Response {
    let dot_radius = 4.0;
    let desired_size = egui::vec2(
        ui.spacing().interact_size.x + dot_radius * 2.0 + 4.0,
        ui.spacing().interact_size.y,
    );
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Highlight background when active
        if active {
            let bg = egui::Color32::from_rgba_premultiplied(
                (color.r() as f32 * 0.18) as u8,
                (color.g() as f32 * 0.18) as u8,
                (color.b() as f32 * 0.18) as u8,
                45,
            );
            painter.rect_filled(rect.shrink(1.0), 4.0, bg);
            painter.rect_stroke(
                rect.shrink(1.0),
                4.0,
                egui::Stroke::new(1.0, dim_color(color, 0.5)),
                egui::StrokeKind::Inside,
            );
        } else if response.hovered() {
            painter.rect_filled(rect.shrink(1.0), 4.0, egui::Color32::from_white_alpha(10));
        }

        // Draw the colored dot
        let dot_center = egui::pos2(rect.left() + dot_radius + 4.0, rect.center().y);
        if active {
            painter.circle_filled(dot_center, dot_radius, color);
        } else {
            painter.circle_stroke(
                dot_center,
                dot_radius,
                egui::Stroke::new(1.0, dim_color(color, 0.4)),
            );
        }

        // Draw the label text
        let text_color = if active {
            color
        } else {
            egui::Color32::from_gray(180)
        };
        let text_pos = egui::pos2(dot_center.x + dot_radius + 4.0, rect.center().y - 6.0);
        painter.text(
            text_pos,
            egui::Align2::LEFT_TOP,
            label,
            egui::FontId::proportional(13.0),
            text_color,
        );
    }

    response
}
