//! Charts panel (UX-046): Enhanced graphs with population lines, budget area,
//! traffic bars, service radar, and happiness breakdown.

use std::collections::VecDeque;
use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::chart_data::ChartHistory;
use simulation::stats::CityStats;
use simulation::time_of_day::GameClock;

const MAX_HISTORY: usize = 200;

// -----------------------------------------------------------------------
// Legacy history (kept for backward compat with existing record_history)
// -----------------------------------------------------------------------

#[derive(Resource)]
pub struct HistoryData {
    pub population: VecDeque<f32>,
    pub happiness: VecDeque<f32>,
    pub treasury: VecDeque<f32>,
    pub last_record_day: u32,
}

impl Default for HistoryData {
    fn default() -> Self {
        Self {
            population: VecDeque::with_capacity(MAX_HISTORY),
            happiness: VecDeque::with_capacity(MAX_HISTORY),
            treasury: VecDeque::with_capacity(MAX_HISTORY),
            last_record_day: 0,
        }
    }
}

pub fn record_history(
    clock: Res<GameClock>,
    stats: Res<CityStats>,
    budget: Res<simulation::economy::CityBudget>,
    mut history: ResMut<HistoryData>,
) {
    // Record every 10 game days
    if clock.day <= history.last_record_day + 10 {
        return;
    }
    history.last_record_day = clock.day;

    history.population.push_back(stats.population as f32);
    history.happiness.push_back(stats.average_happiness);
    history.treasury.push_back(budget.treasury as f32);

    // Trim old data (O(1) front removal with VecDeque)
    if history.population.len() > MAX_HISTORY {
        history.population.pop_front();
        history.happiness.pop_front();
        history.treasury.pop_front();
    }
}

// -----------------------------------------------------------------------
// Time range selector
// -----------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    Month, // ~3 snapshots (30 days / 10 days per snapshot)
    Year,  // ~36 snapshots (360 days)
    AllTime,
}

impl TimeRange {
    fn label(self) -> &'static str {
        match self {
            TimeRange::Month => "1 Month",
            TimeRange::Year => "1 Year",
            TimeRange::AllTime => "All Time",
        }
    }

    fn max_points(self) -> usize {
        match self {
            TimeRange::Month => 3,
            TimeRange::Year => 36,
            TimeRange::AllTime => usize::MAX,
        }
    }
}

// -----------------------------------------------------------------------
// Chart tab
// -----------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChartTab {
    Population,
    Budget,
    Traffic,
    Services,
    Happiness,
}

impl ChartTab {
    fn label(self) -> &'static str {
        match self {
            ChartTab::Population => "Population",
            ChartTab::Budget => "Budget",
            ChartTab::Traffic => "Traffic",
            ChartTab::Services => "Services",
            ChartTab::Happiness => "Happiness",
        }
    }

    const ALL: [ChartTab; 5] = [
        ChartTab::Population,
        ChartTab::Budget,
        ChartTab::Traffic,
        ChartTab::Services,
        ChartTab::Happiness,
    ];
}

// -----------------------------------------------------------------------
// Main UI system
// -----------------------------------------------------------------------

/// Persistent state for the charts panel (tab + range selection).
#[derive(Resource)]
pub struct ChartsState {
    pub tab: ChartTab,
    pub range: TimeRange,
}

impl Default for ChartsState {
    fn default() -> Self {
        Self {
            tab: ChartTab::Population,
            range: TimeRange::AllTime,
        }
    }
}

pub fn graphs_ui(
    mut contexts: EguiContexts,
    history: Res<HistoryData>,
    chart_history: Res<ChartHistory>,
    visible: Res<crate::info_panel::ChartsVisible>,
    mut state: ResMut<ChartsState>,
) {
    if !visible.0 {
        return;
    }

    egui::Window::new("Charts")
        .default_size([420.0, 380.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Press [C] to toggle");

            // Tab bar
            ui.horizontal(|ui| {
                for tab in ChartTab::ALL {
                    if ui.selectable_label(state.tab == tab, tab.label()).clicked() {
                        state.tab = tab;
                    }
                }
            });

            // Time range selector
            ui.horizontal(|ui| {
                ui.label("Range:");
                for range in [TimeRange::Month, TimeRange::Year, TimeRange::AllTime] {
                    if ui
                        .selectable_label(state.range == range, range.label())
                        .clicked()
                    {
                        state.range = range;
                    }
                }
            });

            ui.separator();

            match state.tab {
                ChartTab::Population => {
                    draw_population_chart(ui, &chart_history, &history, state.range)
                }
                ChartTab::Budget => draw_budget_chart(ui, &chart_history, state.range),
                ChartTab::Traffic => draw_traffic_chart(ui, &chart_history),
                ChartTab::Services => draw_service_radar(ui, &chart_history),
                ChartTab::Happiness => draw_happiness_breakdown(ui, &chart_history),
            }
        });
}

// -----------------------------------------------------------------------
// Population line chart with R/C/I sub-lines
// -----------------------------------------------------------------------

fn draw_population_chart(
    ui: &mut egui::Ui,
    chart: &ChartHistory,
    legacy: &HistoryData,
    range: TimeRange,
) {
    if chart.population.is_empty() && legacy.population.is_empty() {
        ui.label("No data yet...");
        return;
    }

    let max_pts = range.max_points();

    // Use chart_history data if available, fall back to legacy
    if !chart.population.is_empty() {
        let data = tail_slice(&chart.population, max_pts);

        ui.heading("Population");
        let total: Vec<f32> = data.iter().map(|s| s.total as f32).collect();
        let res: Vec<f32> = data.iter().map(|s| s.residential_workers as f32).collect();
        let com: Vec<f32> = data.iter().map(|s| s.commercial_workers as f32).collect();
        let ind: Vec<f32> = data.iter().map(|s| s.industrial_workers as f32).collect();

        draw_multi_line_chart(
            ui,
            &[
                (&total, egui::Color32::WHITE, "Total"),
                (&res, egui::Color32::from_rgb(100, 200, 100), "Residential"),
                (&com, egui::Color32::from_rgb(100, 150, 255), "Commercial"),
                (&ind, egui::Color32::from_rgb(255, 180, 50), "Industrial"),
            ],
            380.0,
            120.0,
        );

        // Legend
        ui.horizontal(|ui| {
            if let Some(last) = data.last() {
                legend_item(ui, egui::Color32::WHITE, &format!("Total: {}", last.total));
                legend_item(
                    ui,
                    egui::Color32::from_rgb(100, 200, 100),
                    &format!("R: {}", last.residential_workers),
                );
                legend_item(
                    ui,
                    egui::Color32::from_rgb(100, 150, 255),
                    &format!("C: {}", last.commercial_workers),
                );
                legend_item(
                    ui,
                    egui::Color32::from_rgb(255, 180, 50),
                    &format!("I: {}", last.industrial_workers),
                );
            }
        });
    } else {
        // Legacy fallback
        ui.heading("Population (legacy)");
        let pop: Vec<f32> = legacy.population.iter().copied().collect();
        let data = tail_slice(&pop, max_pts);
        draw_sparkline(ui, data, egui::Color32::GREEN);
        if let Some(&last) = data.last() {
            ui.label(format!("  Latest: {:.0}", last));
        }
    }
}

// -----------------------------------------------------------------------
// Budget stacked area chart
// -----------------------------------------------------------------------

fn draw_budget_chart(ui: &mut egui::Ui, chart: &ChartHistory, range: TimeRange) {
    if chart.budget.is_empty() {
        ui.label("No budget data yet...");
        return;
    }

    let max_pts = range.max_points();
    let data = tail_slice(&chart.budget, max_pts);

    // Income stacked area
    ui.heading("Income");
    let income_layers: Vec<(&str, egui::Color32, Vec<f64>)> = vec![
        (
            "Residential",
            egui::Color32::from_rgb(100, 200, 100),
            data.iter().map(|s| s.residential_tax).collect(),
        ),
        (
            "Commercial",
            egui::Color32::from_rgb(100, 150, 255),
            data.iter().map(|s| s.commercial_tax).collect(),
        ),
        (
            "Industrial",
            egui::Color32::from_rgb(255, 180, 50),
            data.iter().map(|s| s.industrial_tax).collect(),
        ),
        (
            "Office",
            egui::Color32::from_rgb(180, 130, 255),
            data.iter().map(|s| s.office_tax).collect(),
        ),
        (
            "Trade",
            egui::Color32::from_rgb(255, 100, 100),
            data.iter().map(|s| s.trade_income).collect(),
        ),
    ];
    draw_stacked_area(ui, &income_layers, 380.0, 100.0);

    // Income legend
    if let Some(last) = data.last() {
        ui.horizontal_wrapped(|ui| {
            legend_item(
                ui,
                egui::Color32::from_rgb(100, 200, 100),
                &format!("R: ${:.0}", last.residential_tax),
            );
            legend_item(
                ui,
                egui::Color32::from_rgb(100, 150, 255),
                &format!("C: ${:.0}", last.commercial_tax),
            );
            legend_item(
                ui,
                egui::Color32::from_rgb(255, 180, 50),
                &format!("I: ${:.0}", last.industrial_tax),
            );
            legend_item(
                ui,
                egui::Color32::from_rgb(180, 130, 255),
                &format!("O: ${:.0}", last.office_tax),
            );
            legend_item(
                ui,
                egui::Color32::from_rgb(255, 100, 100),
                &format!("Trade: ${:.0}", last.trade_income),
            );
        });
    }

    ui.add_space(8.0);

    // Expense stacked area
    ui.heading("Expenses");
    let expense_layers: Vec<(&str, egui::Color32, Vec<f64>)> = vec![
        (
            "Roads",
            egui::Color32::from_rgb(200, 200, 200),
            data.iter().map(|s| s.road_maintenance).collect(),
        ),
        (
            "Services",
            egui::Color32::from_rgb(255, 150, 150),
            data.iter().map(|s| s.service_costs).collect(),
        ),
        (
            "Policies",
            egui::Color32::from_rgb(150, 200, 255),
            data.iter().map(|s| s.policy_costs).collect(),
        ),
        (
            "Loans",
            egui::Color32::from_rgb(255, 200, 100),
            data.iter().map(|s| s.loan_payments).collect(),
        ),
    ];
    draw_stacked_area(ui, &expense_layers, 380.0, 100.0);

    // Expense legend
    if let Some(last) = data.last() {
        ui.horizontal_wrapped(|ui| {
            legend_item(
                ui,
                egui::Color32::from_rgb(200, 200, 200),
                &format!("Roads: ${:.0}", last.road_maintenance),
            );
            legend_item(
                ui,
                egui::Color32::from_rgb(255, 150, 150),
                &format!("Svc: ${:.0}", last.service_costs),
            );
            legend_item(
                ui,
                egui::Color32::from_rgb(150, 200, 255),
                &format!("Pol: ${:.0}", last.policy_costs),
            );
            legend_item(
                ui,
                egui::Color32::from_rgb(255, 200, 100),
                &format!("Loan: ${:.0}", last.loan_payments),
            );
        });
    }
}

// -----------------------------------------------------------------------
// Traffic congestion by hour (24-bar chart)
// -----------------------------------------------------------------------

fn draw_traffic_chart(ui: &mut egui::Ui, chart: &ChartHistory) {
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

    for hour in 0..24 {
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
                format!("{}", hour),
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

fn draw_service_radar(ui: &mut egui::Ui, chart: &ChartHistory) {
    ui.heading("Service Coverage");

    let cov = &chart.service_coverage;
    let labels = [
        "Health",
        "Education",
        "Police",
        "Fire",
        "Parks",
        "Entertain",
        "Telecom",
        "Transport",
    ];
    let values = [
        cov.health,
        cov.education,
        cov.police,
        cov.fire,
        cov.parks,
        cov.entertainment,
        cov.telecom,
        cov.transport,
    ];
    let n = labels.len();

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
            .map(|i| {
                let angle = (i % n) as f32 * 2.0 * PI / n as f32 - PI / 2.0;
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

    // Draw axis lines
    for i in 0..n {
        let angle = i as f32 * 2.0 * PI / n as f32 - PI / 2.0;
        let end = egui::pos2(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        );
        painter.line_segment(
            [center, end],
            egui::Stroke::new(0.5, egui::Color32::from_gray(60)),
        );

        // Labels
        let label_r = radius + 14.0;
        let label_pos = egui::pos2(
            center.x + label_r * angle.cos(),
            center.y + label_r * angle.sin(),
        );
        painter.text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            labels[i],
            egui::FontId::proportional(9.0),
            egui::Color32::LIGHT_GRAY,
        );
    }

    // Draw data polygon
    let data_points: Vec<egui::Pos2> = (0..n)
        .map(|i| {
            let angle = i as f32 * 2.0 * PI / n as f32 - PI / 2.0;
            let r = radius * values[i].clamp(0.0, 1.0);
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
        for (i, label) in labels.iter().enumerate() {
            ui.label(format!("{}: {:.0}%", label, values[i] * 100.0));
        }
    });
}

// -----------------------------------------------------------------------
// Happiness breakdown stacked horizontal bar
// -----------------------------------------------------------------------

fn draw_happiness_breakdown(ui: &mut egui::Ui, chart: &ChartHistory) {
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

// -----------------------------------------------------------------------
// Drawing helpers
// -----------------------------------------------------------------------

fn tail_slice<T>(data: &[T], max: usize) -> &[T] {
    if data.len() <= max {
        data
    } else {
        &data[data.len() - max..]
    }
}

fn draw_sparkline(ui: &mut egui::Ui, data: &[f32], color: egui::Color32) {
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

fn draw_multi_line_chart(
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

fn draw_stacked_area(
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
        for i in 0..n {
            let x = rect.min.x + (i as f32 / (n - 1) as f32) * rect.width();
            let y = rect.max.y - (top[i] as f32 / max_val as f32) * rect.height();
            polygon.push(egui::pos2(x, y));
        }

        // Bottom edge (right to left)
        for i in (0..n).rev() {
            let x = rect.min.x + (i as f32 / (n - 1) as f32) * rect.width();
            let y = rect.max.y - (bottom[i] as f32 / max_val as f32) * rect.height();
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

fn congestion_color(level: f32) -> egui::Color32 {
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

fn legend_item(ui: &mut egui::Ui, color: egui::Color32, text: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 1.0, color);
    ui.label(text);
}
