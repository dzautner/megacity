//! Traffic congestion bar chart, service coverage radar, and happiness
//! breakdown stacked-bar chart.

use std::f32::consts::PI;

use bevy_egui::egui;

use simulation::chart_data::ChartHistory;

use super::drawing::congestion_color;

// -----------------------------------------------------------------------
// Traffic congestion by hour (24-bar chart)
// -----------------------------------------------------------------------

pub(crate) fn draw_traffic_chart(ui: &mut egui::Ui, chart: &ChartHistory) {
    ui.heading("Congestion by Hour");

    let (rect, _) = ui.allocate_exact_size(egui::vec2(380.0, 140.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

    let bar_width = rect.width() / 24.0;
    let max_congestion = chart
        .traffic_hourly
        .congestion
        .iter()
        .cloned()
        .fold(0.01_f32, f32::max); // avoid div by zero

    for hour in 0..24_usize {
        let val = chart.traffic_hourly.congestion[hour];
        let normalized = val / max_congestion;
        let bar_height = normalized * (rect.height() - 16.0); // leave room for labels

        let x = rect.min.x + hour as f32 * bar_width;
        let bar_rect = egui::Rect::from_min_max(
            egui::pos2(x + 1.0, rect.max.y - 14.0 - bar_height),
            egui::pos2(x + bar_width - 1.0, rect.max.y - 14.0),
        );

        // Color: green -> yellow -> red based on congestion level
        let color = congestion_color(val);
        painter.rect_filled(bar_rect, 1.0, color);

        // Hour labels (every 3 hours)
        if hour % 3 == 0 {
            painter.text(
                egui::pos2(x + bar_width / 2.0, rect.max.y - 6.0),
                egui::Align2::CENTER_CENTER,
                format!("{hour}"),
                egui::FontId::proportional(9.0),
                egui::Color32::GRAY,
            );
        }
    }

    // Peak hour info
    let peak_hour = chart
        .traffic_hourly
        .congestion
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(h, _)| h)
        .unwrap_or(0);
    let peak_val = chart.traffic_hourly.congestion[peak_hour];
    ui.label(format!(
        "Peak: {:02}:00 ({:.0}% congestion)",
        peak_hour,
        peak_val * 100.0
    ));
}

// -----------------------------------------------------------------------
// Service coverage radar/spider chart
// -----------------------------------------------------------------------

pub(crate) fn draw_service_radar(ui: &mut egui::Ui, chart: &ChartHistory) {
    ui.heading("Service Coverage");

    let cov = &chart.service_coverage;
    let categories: [(&str, f32); 8] = [
        ("Health", cov.health),
        ("Education", cov.education),
        ("Police", cov.police),
        ("Fire", cov.fire),
        ("Parks", cov.parks),
        ("Entertain", cov.entertainment),
        ("Telecom", cov.telecom),
        ("Transport", cov.transport),
    ];
    let n = categories.len();

    let size = 200.0_f32;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(size + 120.0, size + 40.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

    let center = egui::pos2(
        rect.min.x + size / 2.0 + 20.0,
        rect.min.y + size / 2.0 + 10.0,
    );
    let radius = size / 2.0 - 20.0;

    // Draw concentric rings (25%, 50%, 75%, 100%)
    for ring in 1..=4 {
        let r = radius * ring as f32 / 4.0;
        let ring_points: Vec<egui::Pos2> = (0..=n)
            .map(|idx| {
                let angle = (idx % n) as f32 * 2.0 * PI / n as f32 - PI / 2.0;
                egui::pos2(center.x + r * angle.cos(), center.y + r * angle.sin())
            })
            .collect();
        for pair in ring_points.windows(2) {
            painter.line_segment(
                [pair[0], pair[1]],
                egui::Stroke::new(0.5, egui::Color32::from_gray(60)),
            );
        }
    }

    // Draw axis lines and labels
    for (idx, (label, _)) in categories.iter().enumerate() {
        let angle = idx as f32 * 2.0 * PI / n as f32 - PI / 2.0;
        let end = egui::pos2(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        );
        painter.line_segment(
            [center, end],
            egui::Stroke::new(0.5, egui::Color32::from_gray(60)),
        );

        let label_r = radius + 14.0;
        let label_pos = egui::pos2(
            center.x + label_r * angle.cos(),
            center.y + label_r * angle.sin(),
        );
        painter.text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            *label,
            egui::FontId::proportional(9.0),
            egui::Color32::LIGHT_GRAY,
        );
    }

    // Draw data polygon
    let data_points: Vec<egui::Pos2> = categories
        .iter()
        .enumerate()
        .map(|(idx, (_, val))| {
            let angle = idx as f32 * 2.0 * PI / n as f32 - PI / 2.0;
            let r = radius * val.clamp(0.0, 1.0);
            egui::pos2(center.x + r * angle.cos(), center.y + r * angle.sin())
        })
        .collect();

    // Fill polygon
    let fill_color = egui::Color32::from_rgba_premultiplied(80, 180, 255, 40);
    painter.add(egui::Shape::convex_polygon(
        data_points.clone(),
        fill_color,
        egui::Stroke::NONE,
    ));

    // Outline
    let mut outline_points = data_points.clone();
    outline_points.push(data_points[0]); // close the loop
    for pair in outline_points.windows(2) {
        painter.line_segment(
            [pair[0], pair[1]],
            egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 180, 255)),
        );
    }

    // Data points
    for pt in &data_points {
        painter.circle_filled(*pt, 3.0, egui::Color32::from_rgb(80, 180, 255));
    }

    // Legend with percentages
    ui.horizontal_wrapped(|ui| {
        for (label, val) in &categories {
            ui.label(format!("{}: {:.0}%", label, val * 100.0));
        }
    });
}

// -----------------------------------------------------------------------
// Happiness breakdown stacked horizontal bar
// -----------------------------------------------------------------------

pub(crate) fn draw_happiness_breakdown(ui: &mut egui::Ui, chart: &ChartHistory) {
    ui.heading("Happiness Breakdown");

    let hap = &chart.happiness;
    let factors = [
        ("Base", hap.base, egui::Color32::from_rgb(100, 100, 100)),
        (
            "Employment",
            hap.employment,
            egui::Color32::from_rgb(100, 200, 100),
        ),
        (
            "Services",
            hap.services,
            egui::Color32::from_rgb(100, 150, 255),
        ),
        (
            "Environment",
            hap.environment,
            if hap.environment >= 0.0 {
                egui::Color32::from_rgb(100, 220, 180)
            } else {
                egui::Color32::from_rgb(255, 100, 100)
            },
        ),
        (
            "Economy",
            hap.economy,
            if hap.economy >= 0.0 {
                egui::Color32::from_rgb(255, 215, 0)
            } else {
                egui::Color32::from_rgb(255, 120, 50)
            },
        ),
    ];

    // Compute total positive and negative contributions
    let total_positive: f32 = factors.iter().map(|(_, v, _)| v.max(0.0)).sum();
    let total_negative: f32 = factors.iter().map(|(_, v, _)| v.min(0.0).abs()).sum();
    let max_extent = total_positive.max(total_negative).max(1.0);

    let bar_width = 380.0_f32;
    let bar_height = 28.0_f32;

    // Positive bar
    ui.label("Positive factors:");
    {
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(bar_width, bar_height), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

        let mut x_offset = 0.0_f32;
        for (_, val, color) in &factors {
            if *val <= 0.0 {
                continue;
            }
            let w = (*val / max_extent) * bar_width;
            let segment = egui::Rect::from_min_size(
                egui::pos2(rect.min.x + x_offset, rect.min.y),
                egui::vec2(w, bar_height),
            );
            painter.rect_filled(segment, 0.0, *color);
            x_offset += w;
        }
    }

    // Negative bar (if any)
    if total_negative > 0.0 {
        ui.label("Negative factors:");
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(bar_width, bar_height), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

        let mut x_offset = 0.0_f32;
        for (_, val, color) in &factors {
            if *val >= 0.0 {
                continue;
            }
            let w = (val.abs() / max_extent) * bar_width;
            let segment = egui::Rect::from_min_size(
                egui::pos2(rect.min.x + x_offset, rect.min.y),
                egui::vec2(w, bar_height),
            );
            painter.rect_filled(segment, 0.0, *color);
            x_offset += w;
        }
    }

    // Legend
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        for (name, val, color) in &factors {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 1.0, *color);
            ui.label(format!("{}: {:.1}", name, val));
        }
    });

    // Net happiness
    let net: f32 = factors.iter().map(|(_, v, _)| v).sum();
    ui.label(format!("Net happiness contribution: {:.1}", net));
}
