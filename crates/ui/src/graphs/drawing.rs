//! Shared drawing helpers for charts: sparklines, multi-line charts,
//! stacked areas, congestion colours, and legend items.

use bevy_egui::egui;

/// Return the last `max` elements of a slice.
pub(crate) fn tail_slice<T>(data: &[T], max: usize) -> &[T] {
    if data.len() <= max {
        data
    } else {
        &data[data.len() - max..]
    }
}

pub(crate) fn draw_sparkline(ui: &mut egui::Ui, data: &[f32], color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(380.0, 40.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

    if data.len() < 2 {
        return;
    }

    let min_val = data.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_val = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let range = (max_val - min_val).max(1.0);

    let points: Vec<egui::Pos2> = data
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = rect.min.x + (i as f32 / (data.len() - 1) as f32) * rect.width();
            let y = rect.max.y - ((v - min_val) / range) * rect.height();
            egui::pos2(x, y)
        })
        .collect();

    for window in points.windows(2) {
        painter.line_segment([window[0], window[1]], egui::Stroke::new(1.5, color));
    }
}

pub(crate) fn draw_multi_line_chart(
    ui: &mut egui::Ui,
    series: &[(&[f32], egui::Color32, &str)],
    width: f32,
    height: f32,
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

    // Find global min/max across all series
    let mut global_min = f32::INFINITY;
    let mut global_max = f32::NEG_INFINITY;
    let mut max_len = 0usize;
    for (data, _, _) in series {
        for &v in *data {
            global_min = global_min.min(v);
            global_max = global_max.max(v);
        }
        max_len = max_len.max(data.len());
    }
    let range = (global_max - global_min).max(1.0);

    if max_len < 2 {
        return;
    }

    // Draw grid lines
    for i in 0..=4 {
        let y = rect.min.y + (i as f32 / 4.0) * rect.height();
        painter.line_segment(
            [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
            egui::Stroke::new(0.3, egui::Color32::from_gray(50)),
        );
    }

    // Draw each series
    for (data, color, _) in series {
        if data.len() < 2 {
            continue;
        }
        let points: Vec<egui::Pos2> = data
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let x = rect.min.x + (i as f32 / (data.len() - 1) as f32) * rect.width();
                let y = rect.max.y - ((v - global_min) / range) * rect.height();
                egui::pos2(x, y)
            })
            .collect();

        for window in points.windows(2) {
            painter.line_segment([window[0], window[1]], egui::Stroke::new(1.5, *color));
        }
    }
}

pub(crate) fn draw_stacked_area(
    ui: &mut egui::Ui,
    layers: &[(&str, egui::Color32, Vec<f64>)],
    width: f32,
    height: f32,
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

    if layers.is_empty() {
        return;
    }

    let n = layers[0].2.len();
    if n < 2 {
        return;
    }

    // Compute cumulative stacks
    let mut cumulative: Vec<Vec<f64>> = vec![vec![0.0; n]; layers.len() + 1];
    for (li, (_, _, data)) in layers.iter().enumerate() {
        for (i, &v) in data.iter().enumerate() {
            cumulative[li + 1][i] = cumulative[li][i] + v;
        }
    }

    let max_val = cumulative
        .last()
        .unwrap()
        .iter()
        .cloned()
        .fold(1.0_f64, f64::max);

    // Draw layers from top to bottom (so last layer is on top)
    for li in (0..layers.len()).rev() {
        let (_, color, _) = &layers[li];
        let bottom = &cumulative[li];
        let top = &cumulative[li + 1];

        let mut polygon = Vec::with_capacity(n * 2 + 2);

        // Top edge (left to right)
        for (i, top_val) in top.iter().enumerate() {
            let x = rect.min.x + (i as f32 / (n - 1) as f32) * rect.width();
            let y = rect.max.y - (*top_val as f32 / max_val as f32) * rect.height();
            polygon.push(egui::pos2(x, y));
        }

        // Bottom edge (right to left)
        for (i, bot_val) in bottom.iter().enumerate().rev() {
            let x = rect.min.x + (i as f32 / (n - 1) as f32) * rect.width();
            let y = rect.max.y - (*bot_val as f32 / max_val as f32) * rect.height();
            polygon.push(egui::pos2(x, y));
        }

        let fill = egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 120);
        painter.add(egui::Shape::convex_polygon(
            polygon,
            fill,
            egui::Stroke::new(1.0, *color),
        ));
    }
}

pub(crate) fn congestion_color(level: f32) -> egui::Color32 {
    let t = level.clamp(0.0, 1.0);
    if t < 0.5 {
        // Green to Yellow
        let ratio = t * 2.0;
        egui::Color32::from_rgb((ratio * 255.0) as u8, 200, ((1.0 - ratio) * 100.0) as u8)
    } else {
        // Yellow to Red
        let ratio = (t - 0.5) * 2.0;
        egui::Color32::from_rgb(255, ((1.0 - ratio) * 200.0) as u8, 0)
    }
}

pub(crate) fn legend_item(ui: &mut egui::Ui, color: egui::Color32, text: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 1.0, color);
    ui.label(text);
}
