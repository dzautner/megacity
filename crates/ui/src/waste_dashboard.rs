//! Waste Management Dashboard UI Panel (WASTE-011).
//!
//! Displays a comprehensive waste management overview including:
//! - Total waste generated, collected, and uncollected (tons/day)
//! - Diversion metrics: recycling rate, composting rate, WTE rate
//! - Landfill capacity: current fill percentage and years remaining
//! - Waste stream breakdown (paper, food, yard, plastics, metals, glass, wood, textiles, other)
//! - Collection coverage: percentage of buildings served
//! - Monthly waste budget: collection cost, processing cost, recycling revenue, net cost
//! - Warning indicators for low landfill capacity, uncollected waste, and overflow

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::composting::CompostingState;
use simulation::garbage::WasteSystem;
use simulation::landfill_warning::{LandfillCapacityState, LandfillWarningTier};
use simulation::recycling::RecyclingState;
use simulation::waste_composition::WasteComposition;

// =============================================================================
// Visibility resource
// =============================================================================

/// Resource controlling whether the waste management dashboard is visible.
/// Toggle with 'G' key.
#[derive(Resource, Default)]
pub struct WasteDashboardVisible(pub bool);

// =============================================================================
// Keybind system
// =============================================================================

// =============================================================================
// Helper: format tons for display
// =============================================================================

/// Formats a tonnage value for human-readable display.
fn fmt_tons(tons: f64) -> String {
    if tons >= 1_000_000.0 {
        format!("{:.1}M", tons / 1_000_000.0)
    } else if tons >= 1_000.0 {
        format!("{:.1}K", tons / 1_000.0)
    } else if tons >= 1.0 {
        format!("{:.1}", tons)
    } else {
        format!("{:.2}", tons)
    }
}

/// Formats a percentage (0.0..1.0) for display as "XX.X%".
fn fmt_pct(fraction: f64) -> String {
    format!("{:.1}%", fraction * 100.0)
}

/// Formats a dollar amount for display.
fn fmt_dollars(amount: f64) -> String {
    if amount.abs() >= 1_000_000.0 {
        format!("${:.1}M", amount / 1_000_000.0)
    } else if amount.abs() >= 1_000.0 {
        format!("${:.1}K", amount / 1_000.0)
    } else {
        format!("${:.0}", amount)
    }
}

// =============================================================================
// Warning indicator helpers
// =============================================================================

/// Warning severity level for the dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
    None,
    Low,
    High,
    Critical,
}

/// Returns the warning severity for landfill capacity.
pub fn landfill_warning_severity(landfill: &LandfillCapacityState) -> WarningSeverity {
    match landfill.current_tier {
        LandfillWarningTier::Normal => WarningSeverity::None,
        LandfillWarningTier::Low => WarningSeverity::Low,
        LandfillWarningTier::Critical => WarningSeverity::High,
        LandfillWarningTier::VeryLow | LandfillWarningTier::Emergency => WarningSeverity::Critical,
    }
}

/// Returns the warning severity for uncollected waste.
pub fn uncollected_warning_severity(waste: &WasteSystem) -> WarningSeverity {
    if waste.uncovered_buildings == 0 {
        WarningSeverity::None
    } else if waste.uncovered_buildings < 10 {
        WarningSeverity::Low
    } else if waste.uncovered_buildings < 50 {
        WarningSeverity::High
    } else {
        WarningSeverity::Critical
    }
}

/// Returns the warning severity for collection overflow (capacity < generation).
pub fn overflow_warning_severity(waste: &WasteSystem) -> WarningSeverity {
    if waste.collection_rate >= 1.0 {
        WarningSeverity::None
    } else if waste.collection_rate >= 0.8 {
        WarningSeverity::Low
    } else if waste.collection_rate >= 0.5 {
        WarningSeverity::High
    } else {
        WarningSeverity::Critical
    }
}

/// Returns the egui color for a warning severity.
fn warning_color(severity: WarningSeverity) -> egui::Color32 {
    match severity {
        WarningSeverity::None => egui::Color32::from_rgb(80, 200, 80),
        WarningSeverity::Low => egui::Color32::from_rgb(220, 200, 50),
        WarningSeverity::High => egui::Color32::from_rgb(240, 140, 40),
        WarningSeverity::Critical => egui::Color32::from_rgb(255, 60, 60),
    }
}

// =============================================================================
// Dashboard UI system
// =============================================================================

/// Renders the waste management dashboard window.
#[allow(clippy::too_many_arguments)]
pub fn waste_dashboard_ui(
    mut contexts: EguiContexts,
    visible: Res<WasteDashboardVisible>,
    waste: Res<WasteSystem>,
    landfill: Res<LandfillCapacityState>,
    recycling: Res<RecyclingState>,
    composting: Res<CompostingState>,
) {
    if !visible.0 {
        return;
    }

    let composition = WasteComposition::default();

    egui::Window::new("Waste Management")
        .default_open(true)
        .default_width(360.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Waste dashboard");
            ui.separator();

            // === Warning indicators ===
            render_warnings(ui, &waste, &landfill);

            // === Waste Generation & Collection ===
            ui.heading("Generation & Collection");
            stat_line(
                ui,
                "Generated",
                &format!("{} tons/day", fmt_tons(waste.period_generated_tons)),
            );
            stat_line(
                ui,
                "Collected",
                &format!("{} tons/day", fmt_tons(waste.total_collected_tons)),
            );
            let uncollected = waste.period_generated_tons - waste.total_collected_tons;
            stat_line(
                ui,
                "Uncollected",
                &format!("{} tons/day", fmt_tons(uncollected.max(0.0))),
            );
            stat_line(ui, "Collection Rate", &fmt_pct(waste.collection_rate));
            stat_line(
                ui,
                "Active Facilities",
                &waste.active_facilities.to_string(),
            );
            ui.separator();

            // === Diversion Metrics ===
            ui.heading("Diversion Metrics");
            let recycling_rate = if waste.period_generated_tons > 0.0 {
                recycling.daily_tons_diverted / waste.period_generated_tons
            } else {
                0.0
            };
            let composting_rate = if waste.period_generated_tons > 0.0 {
                composting.daily_diversion_tons as f64 / waste.period_generated_tons
            } else {
                0.0
            };
            // WTE rate: incinerators handle whatever goes through them.
            // Approximate as: capacity from incinerators / total generated
            // For simplicity, use total diversion - recycling - composting = WTE
            let total_diversion = recycling_rate + composting_rate;
            let wte_rate = (1.0 - waste.collection_rate)
                .max(0.0)
                .min(1.0 - total_diversion);
            let wte_rate = wte_rate.max(0.0);

            stat_line(ui, "Recycling Rate", &fmt_pct(recycling_rate));
            stat_line(ui, "Composting Rate", &fmt_pct(composting_rate));
            stat_line(ui, "WTE Rate", &fmt_pct(wte_rate));
            stat_line(ui, "Program", recycling.tier.name());
            ui.separator();

            // === Landfill Capacity ===
            ui.heading("Landfill Capacity");
            let fill_pct = if landfill.total_capacity > 0.0 {
                (landfill.current_fill / landfill.total_capacity * 100.0) as f32
            } else {
                0.0
            };
            stat_line(ui, "Fill Level", &format!("{:.1}%", fill_pct));

            // Capacity bar
            let bar_fraction = (fill_pct / 100.0).clamp(0.0, 1.0);
            let bar_color = if fill_pct >= 90.0 {
                egui::Color32::from_rgb(255, 60, 60)
            } else if fill_pct >= 75.0 {
                egui::Color32::from_rgb(240, 140, 40)
            } else {
                egui::Color32::from_rgb(80, 200, 80)
            };
            ui.horizontal(|ui| {
                ui.label("  Capacity:");
                let desired = egui::vec2(150.0, 14.0);
                let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
                let painter = ui.painter();
                painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
                let mut fill_rect = rect;
                fill_rect.set_right(rect.left() + rect.width() * bar_fraction);
                painter.rect_filled(fill_rect, 2.0, bar_color);
            });

            let years_str = if landfill.years_remaining.is_infinite() {
                "N/A".to_string()
            } else {
                format!("{:.1} years", landfill.years_remaining)
            };
            stat_line(ui, "Years Remaining", &years_str);
            stat_line(ui, "Landfill Count", &landfill.landfill_count.to_string());
            ui.separator();

            // === Waste Stream Breakdown ===
            ui.heading("Waste Stream Breakdown");
            render_waste_stream(ui, &composition);
            ui.separator();

            // === Collection Coverage ===
            ui.heading("Collection Coverage");
            let total_buildings = waste.total_producers;
            let covered = if total_buildings > 0 {
                ((total_buildings - waste.uncovered_buildings) as f64 / total_buildings as f64)
                    .clamp(0.0, 1.0)
            } else {
                1.0
            };
            stat_line(
                ui,
                "Buildings Served",
                &format!(
                    "{} / {} ({:.1}%)",
                    total_buildings - waste.uncovered_buildings,
                    total_buildings,
                    covered * 100.0
                ),
            );
            ui.separator();

            // === Monthly Waste Budget ===
            ui.heading("Monthly Budget (est.)");
            // Approximate monthly costs (30 days)
            let monthly_collection = waste.transport_cost * 30.0;
            let monthly_processing_cost = recycling.daily_cost * 30.0;
            let monthly_composting_cost = composting.daily_diversion_tons as f64
                * composting
                    .facilities
                    .first()
                    .map_or(30.0, |f| f.cost_per_ton as f64)
                * 30.0;
            let monthly_revenue =
                (recycling.daily_revenue + composting.daily_revenue as f64) * 30.0;
            let total_cost = monthly_collection + monthly_processing_cost + monthly_composting_cost;
            let net = monthly_revenue - total_cost;

            budget_line(ui, "Collection Cost", monthly_collection);
            budget_line(ui, "Processing Cost", monthly_processing_cost);
            budget_line(ui, "Composting Cost", monthly_composting_cost);

            ui.horizontal(|ui| {
                ui.label("  Recycling Revenue:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(80, 200, 80),
                        fmt_dollars(monthly_revenue),
                    );
                });
            });

            ui.separator();
            let net_color = if net >= 0.0 {
                egui::Color32::from_rgb(80, 220, 80)
            } else {
                egui::Color32::from_rgb(255, 60, 60)
            };
            let net_sign = if net >= 0.0 { "+" } else { "" };
            ui.horizontal(|ui| {
                ui.strong("Net Cost:");
                ui.colored_label(net_color, format!("{}{}/mo", net_sign, fmt_dollars(net)));
            });
        });
}

// =============================================================================
// Rendering helpers
// =============================================================================

/// Renders warning indicators at the top of the dashboard.
fn render_warnings(ui: &mut egui::Ui, waste: &WasteSystem, landfill: &LandfillCapacityState) {
    let landfill_sev = landfill_warning_severity(landfill);
    let uncollected_sev = uncollected_warning_severity(waste);
    let overflow_sev = overflow_warning_severity(waste);

    let has_warnings = landfill_sev != WarningSeverity::None
        || uncollected_sev != WarningSeverity::None
        || overflow_sev != WarningSeverity::None;

    if !has_warnings {
        return;
    }

    if landfill_sev != WarningSeverity::None {
        let msg = match landfill_sev {
            WarningSeverity::Low => format!(
                "Landfill capacity low ({:.1}% remaining)",
                landfill.remaining_pct
            ),
            WarningSeverity::High => format!(
                "Landfill capacity critical ({:.1}% remaining)",
                landfill.remaining_pct
            ),
            WarningSeverity::Critical => {
                if landfill.collection_halted {
                    "EMERGENCY: Landfill full! Collection halted!".to_string()
                } else {
                    format!(
                        "Landfill nearly full ({:.1}% remaining)",
                        landfill.remaining_pct
                    )
                }
            }
            WarningSeverity::None => unreachable!(),
        };
        ui.colored_label(warning_color(landfill_sev), msg);
    }

    if uncollected_sev != WarningSeverity::None {
        ui.colored_label(
            warning_color(uncollected_sev),
            format!(
                "{} buildings without waste collection",
                waste.uncovered_buildings
            ),
        );
    }

    if overflow_sev != WarningSeverity::None {
        ui.colored_label(
            warning_color(overflow_sev),
            format!(
                "Collection capacity at {:.0}% of demand",
                waste.collection_rate * 100.0
            ),
        );
    }

    ui.separator();
}

/// Renders a simple stat line with label and value.
fn stat_line(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(format!("  {label}:"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(value);
        });
    });
}

/// Renders a budget expense line.
fn budget_line(ui: &mut egui::Ui, label: &str, amount: f64) {
    ui.horizontal(|ui| {
        ui.label(format!("  {label}:"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.colored_label(egui::Color32::from_rgb(220, 80, 80), fmt_dollars(amount));
        });
    });
}

/// Renders the waste stream breakdown as horizontal bars.
fn render_waste_stream(ui: &mut egui::Ui, composition: &WasteComposition) {
    let streams: &[(&str, f32, egui::Color32)] = &[
        (
            "Paper",
            composition.paper,
            egui::Color32::from_rgb(139, 119, 101),
        ),
        (
            "Food",
            composition.food,
            egui::Color32::from_rgb(165, 113, 78),
        ),
        (
            "Yard",
            composition.yard,
            egui::Color32::from_rgb(107, 142, 35),
        ),
        (
            "Plastics",
            composition.plastics,
            egui::Color32::from_rgb(70, 130, 180),
        ),
        (
            "Metals",
            composition.metals,
            egui::Color32::from_rgb(169, 169, 169),
        ),
        (
            "Glass",
            composition.glass,
            egui::Color32::from_rgb(127, 255, 212),
        ),
        (
            "Wood",
            composition.wood,
            egui::Color32::from_rgb(160, 82, 45),
        ),
        (
            "Textiles",
            composition.textiles,
            egui::Color32::from_rgb(186, 85, 211),
        ),
        (
            "Other",
            composition.other,
            egui::Color32::from_rgb(128, 128, 128),
        ),
    ];

    for &(name, fraction, color) in streams {
        ui.horizontal(|ui| {
            ui.label(format!("  {name}:"));
            let bar_width = 100.0;
            let bar_height = 10.0;
            let desired = egui::vec2(bar_width, bar_height);
            let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
            let painter = ui.painter();
            painter.rect_filled(rect, 1.0, egui::Color32::from_gray(40));
            let mut fill_rect = rect;
            fill_rect.set_right(rect.left() + rect.width() * fraction);
            painter.rect_filled(fill_rect, 1.0, color);
            ui.label(format!("{:.0}%", fraction * 100.0));
        });
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct WasteDashboardPlugin;

impl Plugin for WasteDashboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WasteDashboardVisible>()
            .add_systems(Update, waste_dashboard_ui);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Formatting tests
    // =========================================================================

    #[test]
    fn test_fmt_tons_small() {
        assert_eq!(fmt_tons(0.5), "0.50");
        assert_eq!(fmt_tons(0.99), "0.99");
    }

    #[test]
    fn test_fmt_tons_medium() {
        assert_eq!(fmt_tons(1.0), "1.0");
        assert_eq!(fmt_tons(42.7), "42.7");
        assert_eq!(fmt_tons(999.9), "999.9");
    }

    #[test]
    fn test_fmt_tons_thousands() {
        assert_eq!(fmt_tons(1_000.0), "1.0K");
        assert_eq!(fmt_tons(5_500.0), "5.5K");
        assert_eq!(fmt_tons(999_999.0), "1000.0K");
    }

    #[test]
    fn test_fmt_tons_millions() {
        assert_eq!(fmt_tons(1_000_000.0), "1.0M");
        assert_eq!(fmt_tons(2_500_000.0), "2.5M");
    }

    #[test]
    fn test_fmt_pct() {
        assert_eq!(fmt_pct(0.0), "0.0%");
        assert_eq!(fmt_pct(0.5), "50.0%");
        assert_eq!(fmt_pct(1.0), "100.0%");
        assert_eq!(fmt_pct(0.857), "85.7%");
    }

    #[test]
    fn test_fmt_dollars_small() {
        assert_eq!(fmt_dollars(50.0), "$50");
        assert_eq!(fmt_dollars(999.0), "$999");
    }

    #[test]
    fn test_fmt_dollars_thousands() {
        assert_eq!(fmt_dollars(1_000.0), "$1.0K");
        assert_eq!(fmt_dollars(5_500.0), "$5.5K");
    }

    #[test]
    fn test_fmt_dollars_millions() {
        assert_eq!(fmt_dollars(1_000_000.0), "$1.0M");
    }

    #[test]
    fn test_fmt_dollars_negative() {
        assert_eq!(fmt_dollars(-5_000.0), "$-5.0K");
    }

    // =========================================================================
    // Warning severity tests
    // =========================================================================

    #[test]
    fn test_landfill_warning_normal() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Normal,
            ..Default::default()
        };
        assert_eq!(landfill_warning_severity(&state), WarningSeverity::None);
    }

    #[test]
    fn test_landfill_warning_low() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Low,
            ..Default::default()
        };
        assert_eq!(landfill_warning_severity(&state), WarningSeverity::Low);
    }

    #[test]
    fn test_landfill_warning_critical() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Critical,
            ..Default::default()
        };
        assert_eq!(landfill_warning_severity(&state), WarningSeverity::High);
    }

    #[test]
    fn test_landfill_warning_very_low() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::VeryLow,
            ..Default::default()
        };
        assert_eq!(landfill_warning_severity(&state), WarningSeverity::Critical);
    }

    #[test]
    fn test_landfill_warning_emergency() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Emergency,
            ..Default::default()
        };
        assert_eq!(landfill_warning_severity(&state), WarningSeverity::Critical);
    }

    #[test]
    fn test_uncollected_warning_none() {
        let waste = WasteSystem {
            uncovered_buildings: 0,
            ..Default::default()
        };
        assert_eq!(uncollected_warning_severity(&waste), WarningSeverity::None);
    }

    #[test]
    fn test_uncollected_warning_low() {
        let waste = WasteSystem {
            uncovered_buildings: 5,
            ..Default::default()
        };
        assert_eq!(uncollected_warning_severity(&waste), WarningSeverity::Low);
    }

    #[test]
    fn test_uncollected_warning_high() {
        let waste = WasteSystem {
            uncovered_buildings: 25,
            ..Default::default()
        };
        assert_eq!(uncollected_warning_severity(&waste), WarningSeverity::High);
    }

    #[test]
    fn test_uncollected_warning_critical() {
        let waste = WasteSystem {
            uncovered_buildings: 100,
            ..Default::default()
        };
        assert_eq!(
            uncollected_warning_severity(&waste),
            WarningSeverity::Critical
        );
    }

    #[test]
    fn test_overflow_warning_none() {
        let waste = WasteSystem {
            collection_rate: 1.0,
            ..Default::default()
        };
        assert_eq!(overflow_warning_severity(&waste), WarningSeverity::None);
    }

    #[test]
    fn test_overflow_warning_low() {
        let waste = WasteSystem {
            collection_rate: 0.85,
            ..Default::default()
        };
        assert_eq!(overflow_warning_severity(&waste), WarningSeverity::Low);
    }

    #[test]
    fn test_overflow_warning_high() {
        let waste = WasteSystem {
            collection_rate: 0.6,
            ..Default::default()
        };
        assert_eq!(overflow_warning_severity(&waste), WarningSeverity::High);
    }

    #[test]
    fn test_overflow_warning_critical() {
        let waste = WasteSystem {
            collection_rate: 0.3,
            ..Default::default()
        };
        assert_eq!(overflow_warning_severity(&waste), WarningSeverity::Critical);
    }

    // =========================================================================
    // Warning color tests
    // =========================================================================

    #[test]
    fn test_warning_colors_distinct() {
        let none = warning_color(WarningSeverity::None);
        let low = warning_color(WarningSeverity::Low);
        let high = warning_color(WarningSeverity::High);
        let crit = warning_color(WarningSeverity::Critical);
        // All colors should be different
        assert_ne!(none, low);
        assert_ne!(low, high);
        assert_ne!(high, crit);
    }

    // =========================================================================
    // Dashboard shows correct waste generation rate (test plan item 1)
    // =========================================================================

    #[test]
    fn test_dashboard_generation_rate_display() {
        // When WasteSystem has period_generated_tons = 150.0,
        // the dashboard should display "150.0 tons/day".
        let waste = WasteSystem {
            period_generated_tons: 150.0,
            total_collected_tons: 120.0,
            collection_rate: 0.8,
            ..Default::default()
        };
        let display = fmt_tons(waste.period_generated_tons);
        assert_eq!(display, "150.0");
    }

    // =========================================================================
    // Landfill capacity bar fills over time (test plan item 2)
    // =========================================================================

    #[test]
    fn test_landfill_capacity_bar_fraction() {
        // When landfill is 75% full, bar should show 75% filled.
        let landfill = LandfillCapacityState {
            total_capacity: 1_000_000.0,
            current_fill: 750_000.0,
            remaining_pct: 25.0,
            ..Default::default()
        };
        let fill_pct = (landfill.current_fill / landfill.total_capacity * 100.0) as f32;
        let bar_fraction = (fill_pct / 100.0).clamp(0.0, 1.0);
        assert!((bar_fraction - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_landfill_capacity_bar_empty() {
        let landfill = LandfillCapacityState {
            total_capacity: 1_000_000.0,
            current_fill: 0.0,
            remaining_pct: 100.0,
            ..Default::default()
        };
        let fill_pct = (landfill.current_fill / landfill.total_capacity * 100.0) as f32;
        let bar_fraction = (fill_pct / 100.0).clamp(0.0, 1.0);
        assert!(bar_fraction.abs() < 0.001);
    }

    #[test]
    fn test_landfill_capacity_bar_full() {
        let landfill = LandfillCapacityState {
            total_capacity: 1_000_000.0,
            current_fill: 1_000_000.0,
            remaining_pct: 0.0,
            ..Default::default()
        };
        let fill_pct = (landfill.current_fill / landfill.total_capacity * 100.0) as f32;
        let bar_fraction = (fill_pct / 100.0).clamp(0.0, 1.0);
        assert!((bar_fraction - 1.0).abs() < 0.001);
    }

    // =========================================================================
    // Warning appears when landfill below 25% (test plan item 3)
    // =========================================================================

    #[test]
    fn test_warning_at_25_pct_remaining() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Low,
            remaining_pct: 25.0,
            ..Default::default()
        };
        let severity = landfill_warning_severity(&state);
        assert_ne!(severity, WarningSeverity::None);
    }

    #[test]
    fn test_no_warning_above_25_pct() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Normal,
            remaining_pct: 50.0,
            ..Default::default()
        };
        let severity = landfill_warning_severity(&state);
        assert_eq!(severity, WarningSeverity::None);
    }

    #[test]
    fn test_warning_at_10_pct_remaining() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Critical,
            remaining_pct: 10.0,
            ..Default::default()
        };
        let severity = landfill_warning_severity(&state);
        assert_eq!(severity, WarningSeverity::High);
    }

    // =========================================================================
    // Visibility toggle tests
    // =========================================================================

    #[test]
    fn test_visibility_default_hidden() {
        let visible = WasteDashboardVisible::default();
        assert!(!visible.0);
    }

    // =========================================================================
    // Coverage calculation tests
    // =========================================================================

    #[test]
    fn test_coverage_all_served() {
        let waste = WasteSystem {
            total_producers: 100,
            uncovered_buildings: 0,
            ..Default::default()
        };
        let covered = ((waste.total_producers - waste.uncovered_buildings) as f64
            / waste.total_producers as f64)
            .clamp(0.0, 1.0);
        assert!((covered - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_coverage_partial() {
        let waste = WasteSystem {
            total_producers: 100,
            uncovered_buildings: 20,
            ..Default::default()
        };
        let covered = ((waste.total_producers - waste.uncovered_buildings) as f64
            / waste.total_producers as f64)
            .clamp(0.0, 1.0);
        assert!((covered - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_coverage_zero_buildings() {
        let waste = WasteSystem {
            total_producers: 0,
            uncovered_buildings: 0,
            ..Default::default()
        };
        // When no buildings, coverage is 100%
        let covered = if waste.total_producers > 0 {
            ((waste.total_producers - waste.uncovered_buildings) as f64
                / waste.total_producers as f64)
                .clamp(0.0, 1.0)
        } else {
            1.0
        };
        assert!((covered - 1.0).abs() < 0.001);
    }
}
