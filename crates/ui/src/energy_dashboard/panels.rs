//! Individual UI panel rendering functions for the energy dashboard.

use bevy_egui::egui;

use super::types::{EnergyHistory, GenerationMix, HISTORY_CAPACITY};

// =============================================================================
// Colors
// =============================================================================

const COLOR_GREEN: egui::Color32 = egui::Color32::from_rgb(80, 220, 80);
const COLOR_YELLOW: egui::Color32 = egui::Color32::from_rgb(220, 200, 50);
const COLOR_RED: egui::Color32 = egui::Color32::from_rgb(255, 60, 60);
const COLOR_DEMAND: egui::Color32 = egui::Color32::from_rgb(220, 180, 80);
const COLOR_SUPPLY: egui::Color32 = egui::Color32::from_rgb(80, 180, 220);
const COLOR_COAL: egui::Color32 = egui::Color32::from_rgb(139, 90, 43);
const COLOR_GAS: egui::Color32 = egui::Color32::from_rgb(100, 149, 237);
const COLOR_WIND: egui::Color32 = egui::Color32::from_rgb(144, 238, 144);
const COLOR_BATTERY: egui::Color32 = egui::Color32::from_rgb(186, 85, 211);

// =============================================================================
// Supply & Demand Overview
// =============================================================================

/// Renders the supply/demand/reserve overview.
pub fn render_supply_demand(
    ui: &mut egui::Ui,
    demand_mw: f32,
    supply_mw: f32,
    reserve_margin: f32,
) {
    ui.heading("Supply & Demand");
    ui.horizontal(|ui| {
        ui.label("Total Demand:");
        ui.colored_label(COLOR_DEMAND, format!("{:.1} MW", demand_mw));
    });
    ui.horizontal(|ui| {
        ui.label("Total Supply:");
        ui.colored_label(COLOR_SUPPLY, format!("{:.1} MW", supply_mw));
    });

    let margin_pct = reserve_margin * 100.0;
    let margin_color = reserve_margin_color(reserve_margin);
    ui.horizontal(|ui| {
        ui.label("Reserve Margin:");
        ui.colored_label(margin_color, format!("{:+.1}%", margin_pct));
    });
}

fn reserve_margin_color(margin: f32) -> egui::Color32 {
    if margin < 0.0 {
        COLOR_RED
    } else if margin < 0.10 {
        COLOR_YELLOW
    } else {
        COLOR_GREEN
    }
}

// =============================================================================
// Blackout Status Indicator
// =============================================================================

/// Renders the blackout status indicator (green/yellow/red).
pub fn render_blackout_status(
    ui: &mut egui::Ui,
    has_deficit: bool,
    load_shed_fraction: f32,
    blackout_cells: u32,
) {
    ui.heading("Grid Status");
    let (status_text, status_color) = if !has_deficit {
        ("NORMAL", COLOR_GREEN)
    } else if load_shed_fraction < 0.3 {
        ("BROWNOUT", COLOR_YELLOW)
    } else {
        ("BLACKOUT", COLOR_RED)
    };

    ui.horizontal(|ui| {
        // Status indicator circle
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), 6.0, status_color);
        ui.colored_label(status_color, status_text);
    });

    if has_deficit {
        let shed_pct = load_shed_fraction * 100.0;
        ui.horizontal(|ui| {
            ui.label("Load Shed:");
            ui.colored_label(COLOR_RED, format!("{:.1}%", shed_pct));
        });
        if blackout_cells > 0 {
            ui.horizontal(|ui| {
                ui.label("Affected Cells:");
                ui.colored_label(COLOR_RED, format!("{}", blackout_cells));
            });
        }
    }
}

// =============================================================================
// Electricity Price
// =============================================================================

/// Renders the current electricity price.
pub fn render_price(ui: &mut egui::Ui, price_per_kwh: f32, period_name: &str) {
    ui.heading("Electricity Price");
    ui.horizontal(|ui| {
        ui.label("Current Price:");
        let price_color = if price_per_kwh > 0.25 {
            COLOR_RED
        } else if price_per_kwh > 0.15 {
            COLOR_YELLOW
        } else {
            COLOR_GREEN
        };
        ui.colored_label(price_color, format!("${:.3}/kWh", price_per_kwh));
    });
    ui.horizontal(|ui| {
        ui.label("Period:");
        ui.label(period_name);
    });
}

// =============================================================================
// Generation Mix
// =============================================================================

/// Renders the generation mix as colored horizontal bars.
pub fn render_generation_mix(ui: &mut egui::Ui, mix: &GenerationMix) {
    ui.heading("Generation Mix");
    let total = mix.total();
    if total < 0.01 {
        ui.label("No generation");
        return;
    }

    render_mix_bar(ui, "Coal", mix.coal_mw, total, COLOR_COAL);
    render_mix_bar(ui, "Gas", mix.gas_mw, total, COLOR_GAS);
    render_mix_bar(ui, "Wind", mix.wind_mw, total, COLOR_WIND);
    render_mix_bar(ui, "Battery", mix.battery_mw, total, COLOR_BATTERY);
}

fn render_mix_bar(
    ui: &mut egui::Ui,
    label: &str,
    mw: f32,
    total: f32,
    color: egui::Color32,
) {
    if mw < 0.01 {
        return;
    }
    let frac = mw / total;
    let pct = frac * 100.0;

    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        ui.label(format!("{:.1} MW ({:.0}%)", mw, pct));
    });

    // Draw a colored progress bar.
    let desired_width = ui.available_width().min(280.0);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(desired_width, 14.0),
        egui::Sense::hover(),
    );

    let painter = ui.painter();
    // Background
    painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(40, 40, 40));
    // Filled portion
    let filled_rect = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(rect.width() * frac, rect.height()),
    );
    painter.rect_filled(filled_rect, 2.0, color);
}

// =============================================================================
// History Graph
// =============================================================================

/// Renders a simple 24-hour demand/supply line graph.
pub fn render_history_graph(ui: &mut egui::Ui, history: &EnergyHistory) {
    ui.heading("24-Hour History");
    let count = history.valid_count();
    if count < 2 {
        ui.label("Collecting data...");
        return;
    }

    let demand = history.ordered_demand();
    let supply = history.ordered_supply();

    // Find the max value for scaling.
    let max_val = demand
        .iter()
        .chain(supply.iter())
        .cloned()
        .fold(0.0_f32, f32::max)
        .max(1.0);

    let desired_width = ui.available_width().min(300.0);
    let graph_height = 80.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(desired_width, graph_height),
        egui::Sense::hover(),
    );

    let painter = ui.painter();
    // Background
    painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(30, 30, 30));

    // Draw lines
    draw_line(painter, &rect, &demand, max_val, COLOR_DEMAND);
    draw_line(painter, &rect, &supply, max_val, COLOR_SUPPLY);

    // Legend
    ui.horizontal(|ui| {
        ui.colored_label(COLOR_DEMAND, "Demand");
        ui.colored_label(COLOR_SUPPLY, "Supply");
        ui.label(format!("(max: {:.0} MW)", max_val));
    });
}

fn draw_line(
    painter: &egui::Painter,
    rect: &egui::Rect,
    data: &[f32],
    max_val: f32,
    color: egui::Color32,
) {
    if data.len() < 2 {
        return;
    }

    let points: Vec<egui::Pos2> = data
        .iter()
        .enumerate()
        .map(|(i, &val)| {
            let x = rect.min.x
                + (i as f32 / (HISTORY_CAPACITY - 1) as f32) * rect.width();
            let y = rect.max.y - (val / max_val) * rect.height();
            egui::pos2(x, y)
        })
        .collect();

    for window in points.windows(2) {
        painter.line_segment([window[0], window[1]], egui::Stroke::new(1.5, color));
    }
}
