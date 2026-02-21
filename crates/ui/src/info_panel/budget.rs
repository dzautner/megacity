use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::economy::CityBudget;

use super::BudgetPanelVisible;

// ---------------------------------------------------------------------------
// Trend tracking
// ---------------------------------------------------------------------------

/// Stores the previous month's income/expense values so we can show trend arrows.
#[derive(Resource, Default)]
pub struct BudgetTrends {
    pub prev_income: PrevIncome,
    pub prev_expenses: PrevExpenses,
    pub prev_total_income: f64,
    pub prev_total_expenses: f64,
    /// The simulation day when we last snapshotted.
    pub last_snapshot_day: u32,
}

#[derive(Default, Clone)]
pub struct PrevIncome {
    pub residential_tax: f64,
    pub commercial_tax: f64,
    pub industrial_tax: f64,
    pub office_tax: f64,
    pub trade_income: f64,
}

#[derive(Default, Clone)]
pub struct PrevExpenses {
    pub road_maintenance: f64,
    pub service_costs: f64,
    pub policy_costs: f64,
    pub loan_payments: f64,
}

/// System that snapshots budget values every 30 days for trend comparison.
pub fn snapshot_budget_trends(
    clock: Res<simulation::time_of_day::GameClock>,
    ext_budget: Res<simulation::budget::ExtendedBudget>,
    mut trends: ResMut<BudgetTrends>,
) {
    // Snapshot every 30 days, matching tax collection cadence.
    if clock.day <= trends.last_snapshot_day + 30 {
        return;
    }
    trends.last_snapshot_day = clock.day;

    let inc = &ext_budget.income_breakdown;
    let exp = &ext_budget.expense_breakdown;

    trends.prev_income = PrevIncome {
        residential_tax: inc.residential_tax,
        commercial_tax: inc.commercial_tax,
        industrial_tax: inc.industrial_tax,
        office_tax: inc.office_tax,
        trade_income: inc.trade_income,
    };
    trends.prev_expenses = PrevExpenses {
        road_maintenance: exp.road_maintenance,
        service_costs: exp.service_costs,
        policy_costs: exp.policy_costs,
        loan_payments: exp.loan_payments,
    };
    trends.prev_total_income = inc.residential_tax
        + inc.commercial_tax
        + inc.industrial_tax
        + inc.office_tax
        + inc.trade_income;
    trends.prev_total_expenses =
        exp.road_maintenance + exp.service_costs + exp.policy_costs + exp.loan_payments;
}

// ---------------------------------------------------------------------------
// Colors
// ---------------------------------------------------------------------------

const COLOR_INCOME_GREEN: egui::Color32 = egui::Color32::from_rgb(80, 200, 80);
const COLOR_EXPENSE_RED: egui::Color32 = egui::Color32::from_rgb(220, 80, 80);
const COLOR_BAR_BG: egui::Color32 = egui::Color32::from_rgb(40, 40, 50);
const COLOR_NET_POSITIVE: egui::Color32 = egui::Color32::from_rgb(80, 220, 80);
const COLOR_NET_NEGATIVE: egui::Color32 = egui::Color32::from_rgb(255, 60, 60);

/// Per-category income colors (shades of green/teal).
const INCOME_COLORS: [egui::Color32; 5] = [
    egui::Color32::from_rgb(76, 175, 80),   // Residential - green
    egui::Color32::from_rgb(38, 166, 154),  // Commercial - teal
    egui::Color32::from_rgb(129, 199, 132), // Industrial - light green
    egui::Color32::from_rgb(0, 150, 136),   // Office - dark teal
    egui::Color32::from_rgb(174, 213, 129), // Tourism - lime
];

/// Per-category expense colors (shades of red/orange).
const EXPENSE_COLORS: [egui::Color32; 4] = [
    egui::Color32::from_rgb(239, 83, 80),  // Road maintenance - red
    egui::Color32::from_rgb(255, 152, 0),  // Service costs - orange
    egui::Color32::from_rgb(255, 112, 67), // Policy costs - deep orange
    egui::Color32::from_rgb(244, 67, 54),  // Loan payments - bright red
];

// ---------------------------------------------------------------------------
// Main budget panel UI
// ---------------------------------------------------------------------------

/// Displays a comprehensive budget breakdown window with income and expense
/// categories, percentages, colored bars, and trend indicators.
pub fn budget_panel_ui(
    mut contexts: EguiContexts,
    budget: Res<CityBudget>,
    ext_budget: Res<simulation::budget::ExtendedBudget>,
    mut visible: ResMut<BudgetPanelVisible>,
    trends: Res<BudgetTrends>,
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

    let mut open = true;
    egui::Window::new("Budget Breakdown")
        .open(&mut open)
        .default_open(true)
        .default_width(380.0)
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            // ---- Treasury header ----
            ui.horizontal(|ui| {
                ui.strong("Treasury:");
                let treasury_color = if budget.treasury >= 0.0 {
                    egui::Color32::from_rgb(200, 200, 200)
                } else {
                    COLOR_NET_NEGATIVE
                };
                ui.colored_label(treasury_color, format!("${:.0}", budget.treasury));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let net_color = if net >= 0.0 {
                        COLOR_NET_POSITIVE
                    } else {
                        COLOR_NET_NEGATIVE
                    };
                    let sign = if net >= 0.0 { "+" } else { "" };
                    ui.colored_label(net_color, format!("Net: {sign}${:.0}/mo", net));
                });
            });
            ui.separator();

            // ---- Stacked overview bar ----
            draw_stacked_bar(ui, total_income, total_expenses);
            ui.add_space(4.0);

            // ---- Income section ----
            ui.heading("Income");
            ui.add_space(2.0);

            let income_items: [(&str, f64, f64, egui::Color32); 5] = [
                (
                    "Residential Tax",
                    income.residential_tax,
                    trends.prev_income.residential_tax,
                    INCOME_COLORS[0],
                ),
                (
                    "Commercial Tax",
                    income.commercial_tax,
                    trends.prev_income.commercial_tax,
                    INCOME_COLORS[1],
                ),
                (
                    "Industrial Tax",
                    income.industrial_tax,
                    trends.prev_income.industrial_tax,
                    INCOME_COLORS[2],
                ),
                (
                    "Office Tax",
                    income.office_tax,
                    trends.prev_income.office_tax,
                    INCOME_COLORS[3],
                ),
                (
                    "Trade / Tourism",
                    income.trade_income,
                    trends.prev_income.trade_income,
                    INCOME_COLORS[4],
                ),
            ];

            for (label, amount, prev, color) in &income_items {
                budget_line_with_bar(ui, label, *amount, total_income, *prev, *color);
            }

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.strong("Total Income:");
                ui.colored_label(COLOR_INCOME_GREEN, format!("${:.0}/mo", total_income));
                trend_indicator(ui, total_income, trends.prev_total_income);
            });
            ui.add_space(6.0);

            // ---- Expenses section ----
            ui.separator();
            ui.heading("Expenses");
            ui.add_space(2.0);

            let expense_items: [(&str, f64, f64, egui::Color32); 4] = [
                (
                    "Road Maintenance",
                    expenses.road_maintenance,
                    trends.prev_expenses.road_maintenance,
                    EXPENSE_COLORS[0],
                ),
                (
                    "Service Costs",
                    expenses.service_costs,
                    trends.prev_expenses.service_costs,
                    EXPENSE_COLORS[1],
                ),
                (
                    "Policy Costs",
                    expenses.policy_costs,
                    trends.prev_expenses.policy_costs,
                    EXPENSE_COLORS[2],
                ),
                (
                    "Loan Payments",
                    expenses.loan_payments,
                    trends.prev_expenses.loan_payments,
                    EXPENSE_COLORS[3],
                ),
            ];

            for (label, amount, prev, color) in &expense_items {
                budget_line_with_bar(ui, label, *amount, total_expenses, *prev, *color);
            }

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.strong("Total Expenses:");
                ui.colored_label(COLOR_EXPENSE_RED, format!("${:.0}/mo", total_expenses));
                trend_indicator(ui, total_expenses, trends.prev_total_expenses);
            });
            ui.add_space(6.0);

            // ---- Net income ----
            ui.separator();
            let net_color = if net >= 0.0 {
                COLOR_NET_POSITIVE
            } else {
                COLOR_NET_NEGATIVE
            };
            let sign = if net >= 0.0 { "+" } else { "" };
            ui.horizontal(|ui| {
                ui.heading("Net Income:");
                ui.colored_label(net_color, format!("{sign}${:.0}/mo", net));
                let prev_net = trends.prev_total_income - trends.prev_total_expenses;
                trend_indicator(ui, net, prev_net);
            });

            // ---- Debt summary ----
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
                for (i, loan) in ext_budget.loans.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("  Loan #{}", i + 1));
                        ui.label(format!(
                            "${:.0} remaining ({} mo @ {:.1}%)",
                            loan.remaining,
                            loan.months_remaining,
                            loan.interest_rate * 100.0
                        ));
                    });
                }
            }
        });

    if !open {
        visible.0 = false;
    }
}

// ---------------------------------------------------------------------------
// Drawing helpers
// ---------------------------------------------------------------------------

/// Draws a stacked horizontal bar showing income (green) vs expenses (red)
/// proportionally.
fn draw_stacked_bar(ui: &mut egui::Ui, total_income: f64, total_expenses: f64) {
    let bar_height = 16.0;
    let available_width = ui.available_width().min(360.0);
    let (rect, _response) = ui.allocate_exact_size(
        egui::vec2(available_width, bar_height),
        egui::Sense::hover(),
    );
    let painter = ui.painter_at(rect);

    // Background
    painter.rect_filled(rect, 3.0, COLOR_BAR_BG);

    let grand_total = total_income + total_expenses;
    if grand_total > 0.0 {
        let income_frac = (total_income / grand_total) as f32;

        // Income portion (left, green)
        let income_width = available_width * income_frac;
        if income_width > 0.5 {
            let income_rect =
                egui::Rect::from_min_size(rect.min, egui::vec2(income_width, bar_height));
            painter.rect_filled(income_rect, 3.0, COLOR_INCOME_GREEN);
        }

        // Expense portion (right, red)
        let expense_width = available_width * (1.0 - income_frac);
        if expense_width > 0.5 {
            let expense_rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x + income_width, rect.min.y),
                rect.max,
            );
            painter.rect_filled(expense_rect, 3.0, COLOR_EXPENSE_RED);
        }

        // Labels on the bar
        if income_frac > 0.15 {
            let income_pct = income_frac * 100.0;
            let income_rect =
                egui::Rect::from_min_size(rect.min, egui::vec2(income_width, bar_height));
            painter.text(
                income_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{income_pct:.0}%"),
                egui::FontId::proportional(11.0),
                egui::Color32::WHITE,
            );
        }
        let expense_frac = 1.0 - income_frac;
        if expense_frac > 0.15 {
            let expense_pct = expense_frac * 100.0;
            let expense_rect = egui::Rect::from_min_max(
                egui::pos2(rect.min.x + income_width, rect.min.y),
                rect.max,
            );
            painter.text(
                expense_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{expense_pct:.0}%"),
                egui::FontId::proportional(11.0),
                egui::Color32::WHITE,
            );
        }
    }

    // Legend below the bar
    ui.horizontal(|ui| {
        ui.colored_label(COLOR_INCOME_GREEN, "Income");
        ui.label("|");
        ui.colored_label(COLOR_EXPENSE_RED, "Expenses");
    });
}

/// Renders a single budget line item with label, amount, percentage, colored bar, and trend.
fn budget_line_with_bar(
    ui: &mut egui::Ui,
    label: &str,
    amount: f64,
    total: f64,
    prev_amount: f64,
    bar_color: egui::Color32,
) {
    let pct = if total > 0.0 {
        (amount / total * 100.0) as f32
    } else {
        0.0
    };

    ui.horizontal(|ui| {
        // Fixed-width label
        ui.allocate_ui_with_layout(
            egui::vec2(130.0, 18.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(format!("  {label}"));
            },
        );

        // Colored progress bar
        let bar_width = 80.0;
        let bar_height = 12.0;
        let (bar_rect, _) =
            ui.allocate_exact_size(egui::vec2(bar_width, bar_height), egui::Sense::hover());
        let painter = ui.painter_at(bar_rect);
        painter.rect_filled(bar_rect, 2.0, COLOR_BAR_BG);
        if pct > 0.0 {
            let fill_width = bar_width * (pct / 100.0).min(1.0);
            let fill_rect =
                egui::Rect::from_min_size(bar_rect.min, egui::vec2(fill_width, bar_height));
            painter.rect_filled(fill_rect, 2.0, bar_color);
        }

        // Amount and percentage
        ui.label(format!("${:.0}", amount));
        ui.colored_label(
            egui::Color32::from_rgb(160, 160, 180),
            format!("({pct:.0}%)"),
        );

        // Trend indicator
        trend_indicator(ui, amount, prev_amount);
    });
}

/// Draws a small trend arrow: green up-arrow if value increased, red down-arrow
/// if decreased, grey dash if unchanged.
fn trend_indicator(ui: &mut egui::Ui, current: f64, previous: f64) {
    let diff = current - previous;
    let threshold = 0.5; // Ignore tiny fluctuations
    if diff > threshold {
        ui.colored_label(egui::Color32::from_rgb(100, 220, 100), "\u{25B2}"); // up triangle
    } else if diff < -threshold {
        ui.colored_label(egui::Color32::from_rgb(220, 100, 100), "\u{25BC}"); // down triangle
    } else {
        ui.colored_label(egui::Color32::from_rgb(120, 120, 120), "\u{2014}"); // em dash
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin that registers the budget trend tracking system.
/// The budget panel UI itself is registered in `UiPlugin`.
pub struct BudgetBreakdownPlugin;

impl Plugin for BudgetBreakdownPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BudgetTrends>().add_systems(
            FixedUpdate,
            snapshot_budget_trends
                .after(simulation::economy::collect_taxes)
                .in_set(simulation::SimulationSet::PostSim),
        );
    }
}
