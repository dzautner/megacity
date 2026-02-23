//! Info panel sections: Budget, Loans/Finance, Road Maintenance, Traffic Safety,
//! and Service Budgets.

use bevy_egui::egui;

use simulation::economy::CityBudget;
use simulation::loans::{LoanBook, LoanTier};

use super::types::InfoPanelExtras;

/// Render the budget overview with tax slider and budget-details button.
pub fn draw_budget(ui: &mut egui::Ui, budget: &mut CityBudget, extras: &mut InfoPanelExtras) {
    ui.separator();
    ui.heading("Budget");
    ui.label(format!("Treasury: ${:.0}", budget.treasury));
    ui.label(format!("Income: ${:.0}/month", budget.monthly_income));
    ui.label(format!("Expenses: ${:.0}/month", budget.monthly_expenses));

    ui.horizontal(|ui| {
        ui.label("Tax rate:");
        let mut tax_pct = budget.tax_rate * 100.0;
        if ui
            .add(egui::Slider::new(&mut tax_pct, 0.0..=25.0).suffix("%"))
            .changed()
        {
            budget.tax_rate = tax_pct / 100.0;
        }
    });

    if ui.button("Budget Details...").clicked() {
        extras.budget_visible.0 = !extras.budget_visible.0;
    }
}

/// Render the Road Maintenance collapsing section (includes Traffic Safety).
pub fn draw_road_maintenance(ui: &mut egui::Ui, extras: &mut InfoPanelExtras) {
    let road_maint_stats = &extras.road_maint_stats;

    ui.separator();
    ui.collapsing("Road Maintenance", |ui| {
        let avg = road_maint_stats.avg_condition;
        let avg_pct = avg / 255.0;
        let cond_color = if avg > 150.0 {
            egui::Color32::from_rgb(50, 200, 50)
        } else if avg > 80.0 {
            egui::Color32::from_rgb(220, 180, 50)
        } else {
            egui::Color32::from_rgb(220, 50, 50)
        };
        ui.horizontal(|ui| {
            ui.label("Avg condition:");
            let (rect, _) = ui.allocate_exact_size(egui::vec2(80.0, 12.0), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
            let fill_rect = egui::Rect::from_min_size(
                rect.min,
                egui::vec2(rect.width() * avg_pct.clamp(0.0, 1.0), rect.height()),
            );
            painter.rect_filled(fill_rect, 2.0, cond_color);
            ui.colored_label(cond_color, format!("{:.0}", avg));
        });

        let poor_color = if road_maint_stats.poor_roads_count > 100 {
            egui::Color32::from_rgb(220, 180, 50)
        } else {
            egui::Color32::from_rgb(180, 180, 180)
        };
        let crit_color = if road_maint_stats.critical_roads_count > 0 {
            egui::Color32::from_rgb(220, 50, 50)
        } else {
            egui::Color32::from_rgb(180, 180, 180)
        };
        ui.horizontal(|ui| {
            ui.colored_label(
                poor_color,
                format!("Poor: {}", road_maint_stats.poor_roads_count),
            );
            ui.label("|");
            ui.colored_label(
                crit_color,
                format!("Critical: {}", road_maint_stats.critical_roads_count),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Maintenance:");
            let mut maint_pct = extras.road_maint_budget.budget_level * 100.0;
            if ui
                .add(egui::Slider::new(&mut maint_pct, 0.0..=200.0).suffix("%"))
                .changed()
            {
                extras.road_maint_budget.budget_level = maint_pct / 100.0;
            }
        });

        ui.label(format!(
            "Cost: ${:.0}/mo",
            extras.road_maint_budget.monthly_cost
        ));

        // ---- Traffic Safety ----
        ui.separator();
        ui.label("Traffic Safety");
        let tracker = &extras.accident_tracker;
        let active = tracker.active_accidents.len();
        let accident_color = if active > 5 {
            egui::Color32::from_rgb(220, 50, 50)
        } else if active > 0 {
            egui::Color32::from_rgb(220, 200, 50)
        } else {
            egui::Color32::from_rgb(50, 200, 50)
        };
        ui.colored_label(accident_color, format!("Active accidents: {}", active));
        ui.label(format!("Total accidents: {}", tracker.total_accidents));
        ui.label(format!("This month: {}", tracker.accidents_this_month));
        if tracker.response_count > 0 {
            ui.label(format!(
                "Avg response: {:.1} ticks",
                tracker.avg_response_time
            ));
        }
    });
}

/// Render the Finance collapsing section (loans, credit rating, trade balance).
pub fn draw_finance(
    ui: &mut egui::Ui,
    budget: &mut CityBudget,
    loan_book: &mut LoanBook,
    extras: &InfoPanelExtras,
) {
    let resource_balance = &extras.resource_balance;

    ui.separator();
    ui.collapsing("Finance", |ui| {
        // Credit rating display
        let cr = loan_book.credit_rating;
        let cr_color = if cr >= 1.5 {
            egui::Color32::from_rgb(50, 200, 50)
        } else if cr >= 0.8 {
            egui::Color32::from_rgb(220, 200, 50)
        } else {
            egui::Color32::from_rgb(220, 50, 50)
        };
        let cr_label = if cr >= 1.5 {
            "Excellent"
        } else if cr >= 1.2 {
            "Good"
        } else if cr >= 0.8 {
            "Fair"
        } else if cr >= 0.5 {
            "Poor"
        } else {
            "Critical"
        };
        ui.horizontal(|ui| {
            ui.label("Credit Rating:");
            ui.colored_label(cr_color, format!("{:.2} ({})", cr, cr_label));
        });

        // Trade balance
        let trade_bal = resource_balance.trade_balance();
        let trade_color = if trade_bal >= 0.0 {
            egui::Color32::from_rgb(50, 200, 50)
        } else {
            egui::Color32::from_rgb(220, 50, 50)
        };
        ui.horizontal(|ui| {
            ui.label("Trade Balance:");
            ui.colored_label(trade_color, format!("${:.0}/mo", trade_bal));
        });

        // Total debt and debt-to-income
        let total_debt = loan_book.total_debt();
        ui.label(format!("Total Debt: ${:.0}", total_debt));
        let dti = loan_book.debt_to_income(budget.monthly_income);
        let dti_str = if dti.is_finite() {
            format!("{:.1}x", dti)
        } else {
            "N/A".to_string()
        };
        let dti_color = if dti < 2.0 {
            egui::Color32::from_rgb(50, 200, 50)
        } else if dti < 5.0 {
            egui::Color32::from_rgb(220, 200, 50)
        } else {
            egui::Color32::from_rgb(220, 50, 50)
        };
        ui.horizontal(|ui| {
            ui.label("Debt/Income:");
            ui.colored_label(dti_color, dti_str);
        });

        ui.add_space(4.0);

        // Active loans list
        if loan_book.active_loans.is_empty() {
            ui.label("No active loans.");
        } else {
            ui.label("Active Loans:");
            egui::Grid::new("active_loans_grid")
                .num_columns(3)
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Name");
                    ui.strong("Balance");
                    ui.strong("Payment");
                    ui.end_row();
                    for loan in &loan_book.active_loans {
                        ui.label(&loan.name);
                        ui.label(format!("${:.0}", loan.remaining_balance));
                        ui.label(format!("${:.0}/mo", loan.monthly_payment));
                        ui.end_row();
                    }
                });
        }

        ui.add_space(4.0);

        // Take Loan buttons
        let at_max = loan_book.active_loans.len() >= loan_book.max_loans;
        ui.label("Take a Loan:");
        for tier in LoanTier::ALL {
            let label = format!(
                "{}: ${:.0} @ {:.0}% / {}mo",
                tier.name(),
                tier.amount(),
                tier.interest_rate() * 100.0,
                tier.term_months(),
            );
            let button = egui::Button::new(&label);
            let response = ui.add_enabled(!at_max, button);
            if response.clicked() {
                loan_book.take_loan(tier, &mut budget.treasury);
            }
            if at_max {
                response.on_hover_text("Maximum loans reached");
            }
        }
    });
}

/// Render the service budget sliders.
pub fn draw_service_budgets(
    ui: &mut egui::Ui,
    ext_budget: &mut simulation::budget::ExtendedBudget,
) {
    ui.separator();
    ui.heading("Service Budgets");
    {
        let sb = &mut ext_budget.service_budgets;
        let mut fire_pct = sb.fire * 100.0;
        let mut police_pct = sb.police * 100.0;
        let mut health_pct = sb.healthcare * 100.0;
        let mut edu_pct = sb.education * 100.0;
        let mut sanit_pct = sb.sanitation * 100.0;
        let mut trans_pct = sb.transport * 100.0;

        ui.horizontal(|ui| {
            ui.label("Fire:");
            if ui
                .add(egui::Slider::new(&mut fire_pct, 0.0..=150.0).suffix("%"))
                .changed()
            {
                sb.fire = fire_pct / 100.0;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Police:");
            if ui
                .add(egui::Slider::new(&mut police_pct, 0.0..=150.0).suffix("%"))
                .changed()
            {
                sb.police = police_pct / 100.0;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Health:");
            if ui
                .add(egui::Slider::new(&mut health_pct, 0.0..=150.0).suffix("%"))
                .changed()
            {
                sb.healthcare = health_pct / 100.0;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Education:");
            if ui
                .add(egui::Slider::new(&mut edu_pct, 0.0..=150.0).suffix("%"))
                .changed()
            {
                sb.education = edu_pct / 100.0;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Sanitation:");
            if ui
                .add(egui::Slider::new(&mut sanit_pct, 0.0..=150.0).suffix("%"))
                .changed()
            {
                sb.sanitation = sanit_pct / 100.0;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Transport:");
            if ui
                .add(egui::Slider::new(&mut trans_pct, 0.0..=150.0).suffix("%"))
                .changed()
            {
                sb.transport = trans_pct / 100.0;
            }
        });
    }
}
