use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::overlay::{OverlayMode, OverlayState};

/// Describes a single overlay's legend: name, min/max labels, and color stops.
struct LegendInfo {
    name: &'static str,
    min_label: &'static str,
    max_label: &'static str,
    /// Color stops from bottom (min) to top (max), as `(r, g, b)` in 0..=255.
    stops: Vec<(u8, u8, u8)>,
}

/// Returns the legend metadata for the active overlay, or `None` for `OverlayMode::None`.
fn legend_for(mode: OverlayMode) -> Option<LegendInfo> {
    match mode {
        OverlayMode::None => None,
        OverlayMode::Power => Some(LegendInfo {
            name: "Power",
            min_label: "No Power",
            max_label: "Powered",
            stops: vec![
                (179, 26, 26),  // unpowered red  (0.7, 0.1, 0.1)
                (230, 230, 26), // powered yellow  (0.9, 0.9, 0.1)
            ],
        }),
        OverlayMode::Water => Some(LegendInfo {
            name: "Water",
            min_label: "No Water",
            max_label: "Supplied",
            stops: vec![
                (179, 26, 26),  // no water red   (0.7, 0.1, 0.1)
                (26, 128, 230), // supplied blue   (0.1, 0.5, 0.9)
            ],
        }),
        OverlayMode::Traffic => Some(LegendInfo {
            name: "Traffic",
            min_label: "Free Flow",
            max_label: "Congested",
            stops: vec![
                (51, 255, 51),  // low congestion  green (0.0, 1.0, 0.2 blended)
                (255, 255, 51), // medium           yellow
                (255, 51, 51),  // high congestion  red  (1.0, 0.0, 0.2)
            ],
        }),
        OverlayMode::Pollution => Some(LegendInfo {
            name: "Pollution",
            min_label: "Clean",
            max_label: "Polluted",
            stops: vec![
                (0, 230, 51),   // clean   green (0.0, 0.9, 0.2 approx at intensity=0 => (0, ~255, 51))
                (128, 166, 51), // mid
                (255, 128, 51), // polluted (1.0, 0.5, 0.2)
            ],
        }),
        OverlayMode::LandValue => Some(LegendInfo {
            name: "Land Value",
            min_label: "Low",
            max_label: "High",
            stops: vec![
                (51, 77, 26),   // low value  (0.2, 0.3, 0.1)
                (128, 140, 26), // mid
                (204, 204, 26), // high value (0.8, 0.8, 0.1)
            ],
        }),
        OverlayMode::Education => Some(LegendInfo {
            name: "Education",
            min_label: "None",
            max_label: "University",
            stops: vec![
                (77, 77, 128),  // level 0 (0.3, 0.3, 0.5)
                (77, 140, 179), // level 1
                (77, 179, 230), // level 2
                (77, 204, 255), // level 3 (0.3, 0.8, 1.0)
            ],
        }),
        OverlayMode::Garbage => Some(LegendInfo {
            name: "Garbage",
            min_label: "Clean",
            max_label: "Overflowing",
            stops: vec![
                (77, 102, 51), // clean  (0.3, 0.4, 0.2)
                (128, 77, 51), // mid
                (204, 51, 51), // overflowing (0.8, 0.2, 0.2)
            ],
        }),
        OverlayMode::Noise => Some(LegendInfo {
            name: "Noise",
            min_label: "Quiet",
            max_label: "Loud",
            stops: vec![
                (51, 26, 102),  // quiet: dark purple (0.2, 0.1, 0.4)
                (128, 51, 77),  // mid
                (230, 102, 51), // loud: bright orange (0.9, 0.4, 0.2)
            ],
        }),
        OverlayMode::WaterPollution => Some(LegendInfo {
            name: "Water Pollution",
            min_label: "Clean",
            max_label: "Polluted",
            stops: vec![
                (26, 77, 153), // clean blue  (0.1, 0.3, 0.6)
                (77, 64, 64),  // mid brownish
                (153, 89, 38), // polluted brown (0.6, 0.35, 0.15)
            ],
        }),
    }
}

/// Linearly interpolate between color stops. `t` is in 0.0..=1.0.
fn sample_ramp(stops: &[(u8, u8, u8)], t: f32) -> egui::Color32 {
    if stops.is_empty() {
        return egui::Color32::BLACK;
    }
    if stops.len() == 1 {
        let (r, g, b) = stops[0];
        return egui::Color32::from_rgb(r, g, b);
    }

    let t = t.clamp(0.0, 1.0);
    let segments = (stops.len() - 1) as f32;
    let scaled = t * segments;
    let idx = (scaled as usize).min(stops.len() - 2);
    let frac = scaled - idx as f32;

    let (r0, g0, b0) = stops[idx];
    let (r1, g1, b1) = stops[idx + 1];

    let r = r0 as f32 + (r1 as f32 - r0 as f32) * frac;
    let g = g0 as f32 + (g1 as f32 - g0 as f32) * frac;
    let b = b0 as f32 + (b1 as f32 - b0 as f32) * frac;

    egui::Color32::from_rgb(r as u8, g as u8, b as u8)
}

/// System that draws the overlay legend panel when an overlay is active.
pub fn overlay_legend_ui(mut contexts: EguiContexts, overlay: Res<OverlayState>) {
    let Some(info) = legend_for(overlay.mode) else {
        return;
    };

    let bar_width = 20.0;
    let bar_height = 150.0;
    let padding = 8.0;

    // Position in the bottom-left corner, above the bottom toolbar (36px high)
    let screen = contexts.ctx_mut().screen_rect();
    let anchor_x = 12.0;
    let anchor_y = screen.bottom() - 36.0 - 12.0; // above the 36px bottom toolbar

    egui::Area::new(egui::Id::new("overlay_legend"))
        .fixed_pos(egui::pos2(anchor_x, anchor_y))
        .pivot(egui::Align2::LEFT_BOTTOM)
        .interactable(false)
        .show(contexts.ctx_mut(), |ui| {
            egui::Frame::popup(ui.style())
                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 220))
                .inner_margin(padding)
                .show(ui, |ui| {
                    // Overlay name
                    ui.label(
                        egui::RichText::new(info.name)
                            .strong()
                            .color(egui::Color32::from_rgb(220, 225, 240)),
                    );

                    ui.add_space(4.0);

                    // Max label (top of gradient)
                    ui.label(
                        egui::RichText::new(info.max_label)
                            .small()
                            .color(egui::Color32::from_rgb(200, 200, 200)),
                    );

                    ui.add_space(2.0);

                    // Gradient bar painted via egui painter
                    let (rect, _response) = ui.allocate_exact_size(
                        egui::vec2(bar_width, bar_height),
                        egui::Sense::hover(),
                    );

                    let painter = ui.painter_at(rect);
                    // Draw vertical gradient as horizontal stripe rows (1px each)
                    let rows = bar_height as usize;
                    for row in 0..rows {
                        // row 0 = top of rect = max value (t=1), row N = bottom = min value (t=0)
                        let t = 1.0 - row as f32 / (rows - 1).max(1) as f32;
                        let color = sample_ramp(&info.stops, t);
                        let y_top = rect.top() + row as f32;
                        let y_bot = y_top + 1.0;
                        painter.rect_filled(
                            egui::Rect::from_min_max(
                                egui::pos2(rect.left(), y_top),
                                egui::pos2(rect.right(), y_bot),
                            ),
                            0.0,
                            color,
                        );
                    }

                    ui.add_space(2.0);

                    // Min label (bottom of gradient)
                    ui.label(
                        egui::RichText::new(info.min_label)
                            .small()
                            .color(egui::Color32::from_rgb(200, 200, 200)),
                    );
                });
        });
}
