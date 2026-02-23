//! Population line chart and budget stacked-area chart.

use bevy_egui::egui;

use simulation::chart_data::ChartHistory;

use super::drawing::{
    draw_multi_line_chart, draw_sparkline, draw_stacked_area, legend_item, tail_slice,
};
use super::{HistoryData, TimeRange};

// -----------------------------------------------------------------------
// Population line chart with R/C/I sub-lines
// -----------------------------------------------------------------------

pub(crate) fn draw_population_chart(
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

pub(crate) fn draw_budget_chart(ui: &mut egui::Ui, chart: &ChartHistory, range: TimeRange) {
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
