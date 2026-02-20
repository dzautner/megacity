use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::economy::CityBudget;

use super::BudgetPanelVisible;

/// Displays a detailed budget breakdown window with income and expense lines.
/// Shows per-zone tax income, per-category expenses, and net income.
pub fn budget_panel_ui(
    mut contexts: EguiContexts,
    budget: Res<CityBudget>,
    ext_budget: Res<simulation::budget::ExtendedBudget>,
    visible: Res<BudgetPanelVisible>,
) {
    if !visible.0 {
        return;
    }

    let income = &ext_budget.income_breakdown;
    let expenses = &ext_budget.expense_breakdown;

    let total_income = income.residential_tax
        + income.commercial_tax
        + income.industrial_tax
        + income.office_tax
        + income.trade_income;
    let total_expenses = expenses.road_maintenance
        + expenses.service_costs
        + expenses.policy_costs
        + expenses.loan_payments;
    let net = total_income - total_expenses;

    egui::Window::new("Budget Breakdown")
        .default_open(true)
        .default_width(320.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Press [B] to toggle");
            ui.separator();

            // Treasury
            ui.horizontal(|ui| {
                ui.strong("Treasury:");
                ui.label(format!("${:.0}", budget.treasury));
            });
            ui.separator();

            // --- Income section ---
            ui.heading("Income");
            budget_line(ui, "Residential Tax", income.residential_tax);
            budget_line(ui, "Commercial Tax", income.commercial_tax);
            budget_line(ui, "Industrial Tax", income.industrial_tax);
            budget_line(ui, "Office Tax", income.office_tax);
            budget_line(ui, "Trade / Tourism", income.trade_income);
            ui.separator();
            ui.horizontal(|ui| {
                ui.strong("Total Income:");
                ui.colored_label(
                    egui::Color32::from_rgb(80, 200, 80),
                    format!("${:.0}/mo", total_income),
                );
            });
            ui.add_space(4.0);

            // --- Expense section ---
            ui.heading("Expenses");
            budget_line(ui, "Road Maintenance", expenses.road_maintenance);
            budget_line(ui, "Service Costs", expenses.service_costs);
            budget_line(ui, "Policy Costs", expenses.policy_costs);
            budget_line(ui, "Loan Payments", expenses.loan_payments);
            ui.separator();
            ui.horizontal(|ui| {
                ui.strong("Total Expenses:");
                ui.colored_label(
                    egui::Color32::from_rgb(220, 80, 80),
                    format!("${:.0}/mo", total_expenses),
                );
            });
            ui.add_space(8.0);

            // --- Net income ---
            ui.separator();
            let net_color = if net >= 0.0 {
                egui::Color32::from_rgb(80, 220, 80)
            } else {
                egui::Color32::from_rgb(255, 60, 60)
            };
            let net_sign = if net >= 0.0 { "+" } else { "" };
            ui.horizontal(|ui| {
                ui.strong("Net Income:");
                ui.colored_label(net_color, format!("{}{:.0}/mo", net_sign, net));
            });

            // Debt summary (only when loans exist)
            if !ext_budget.loans.is_empty() {
                ui.add_space(4.0);
                ui.separator();
                ui.heading("Outstanding Debt");
                ui.label(format!(
                    "Total debt: ${:.0} ({} loan{})",
                    ext_budget.total_debt(),
                    ext_budget.loans.len(),
                    if ext_budget.loans.len() == 1 { "" } else { "s" }
                ));
            }
        });
}

/// Helper to render a single budget line item.
fn budget_line(ui: &mut egui::Ui, label: &str, amount: f64) {
    ui.horizontal(|ui| {
        ui.label(format!("  {label}:"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("${:.0}", amount));
        });
    });
}
