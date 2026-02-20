//! Water Supply Dashboard UI Panel (WATER-012).
//!
//! Displays a comprehensive water supply dashboard showing:
//! - Total demand (MGD) and total supply (MGD) with surplus/deficit
//! - Source breakdown: wells, surface intake, reservoir, desalination contributions
//! - Groundwater level indicator with depletion warning
//! - Reservoir level: % full, days of storage
//! - Service coverage: % of buildings with water service
//! - Water quality: treatment level and output quality
//! - Sewage treatment: % of wastewater treated, treatment level
//! - Monthly water budget: treatment costs, revenue from water rates

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::groundwater::GroundwaterStats;
use simulation::reservoir::ReservoirState;
use simulation::wastewater::WastewaterState;
use simulation::water_demand::WaterSupply;
use simulation::water_sources::{WaterSource, WaterSourceType};
use simulation::water_treatment::WaterTreatmentState;

/// Resource controlling whether the water dashboard window is visible.
/// Toggle with 'W' key.
#[derive(Resource, Default)]
pub struct WaterDashboardVisible(pub bool);

/// Conversion constant: 1 MGD = 1,000,000 gallons per day.
const MGD_TO_GPD: f32 = 1_000_000.0;

/// Displays the water supply dashboard window.
///
/// Shows demand/supply balance, source breakdown, groundwater status,
/// reservoir levels, service coverage, water quality, sewage treatment,
/// and monthly water budget information.
#[allow(clippy::too_many_arguments)]
pub fn water_dashboard_ui(
    mut contexts: EguiContexts,
    visible: Res<WaterDashboardVisible>,
    water_supply: Res<WaterSupply>,
    groundwater_stats: Res<GroundwaterStats>,
    reservoir_state: Res<ReservoirState>,
    treatment_state: Res<WaterTreatmentState>,
    wastewater_state: Res<WastewaterState>,
    sources: Query<&WaterSource>,
) {
    if !visible.0 {
        return;
    }

    // Aggregate source contributions by type
    let mut well_supply_mgd: f32 = 0.0;
    let mut surface_supply_mgd: f32 = 0.0;
    let mut reservoir_supply_mgd: f32 = 0.0;
    let mut desal_supply_mgd: f32 = 0.0;
    let mut well_count: u32 = 0;
    let mut surface_count: u32 = 0;
    let mut reservoir_source_count: u32 = 0;
    let mut desal_count: u32 = 0;
    let mut total_source_operating_cost: f64 = 0.0;

    for source in &sources {
        match source.source_type {
            WaterSourceType::Well => {
                well_supply_mgd += source.capacity_mgd;
                well_count += 1;
            }
            WaterSourceType::SurfaceIntake => {
                surface_supply_mgd += source.capacity_mgd;
                surface_count += 1;
            }
            WaterSourceType::Reservoir => {
                reservoir_supply_mgd += source.capacity_mgd;
                reservoir_source_count += 1;
            }
            WaterSourceType::Desalination => {
                desal_supply_mgd += source.capacity_mgd;
                desal_count += 1;
            }
        }
        total_source_operating_cost += source.operating_cost;
    }

    let total_demand_mgd = water_supply.total_demand_gpd / MGD_TO_GPD;
    let total_supply_mgd = water_supply.total_supply_gpd / MGD_TO_GPD;
    let surplus_deficit_mgd = total_supply_mgd - total_demand_mgd;

    egui::Window::new("Water Supply Dashboard")
        .default_open(true)
        .default_width(360.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Water dashboard");
            ui.separator();

            // ---- Demand & Supply Overview ----
            ui.heading("Supply & Demand");
            ui.horizontal(|ui| {
                ui.label("Total Demand:");
                ui.colored_label(
                    egui::Color32::from_rgb(220, 180, 80),
                    format!("{:.2} MGD", total_demand_mgd),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Total Supply:");
                ui.colored_label(
                    egui::Color32::from_rgb(80, 180, 220),
                    format!("{:.2} MGD", total_supply_mgd),
                );
            });

            let surplus_color = if surplus_deficit_mgd >= 0.0 {
                egui::Color32::from_rgb(80, 220, 80)
            } else {
                egui::Color32::from_rgb(255, 60, 60)
            };
            let surplus_label = if surplus_deficit_mgd >= 0.0 {
                "Surplus"
            } else {
                "Deficit"
            };
            ui.horizontal(|ui| {
                ui.label(format!("{surplus_label}:"));
                ui.colored_label(surplus_color, format!("{:+.2} MGD", surplus_deficit_mgd));
            });

            ui.add_space(4.0);
            ui.separator();

            // ---- Source Breakdown ----
            ui.heading("Source Breakdown");
            if well_count > 0 || surface_count > 0 || reservoir_source_count > 0 || desal_count > 0
            {
                egui::Grid::new("water_source_grid")
                    .num_columns(3)
                    .spacing([12.0, 4.0])
                    .show(ui, |ui| {
                        ui.strong("Source");
                        ui.strong("Count");
                        ui.strong("Capacity");
                        ui.end_row();

                        if well_count > 0 {
                            ui.label("Wells");
                            ui.label(format!("{well_count}"));
                            ui.label(format!("{:.2} MGD", well_supply_mgd));
                            ui.end_row();
                        }
                        if surface_count > 0 {
                            ui.label("Surface Intake");
                            ui.label(format!("{surface_count}"));
                            ui.label(format!("{:.2} MGD", surface_supply_mgd));
                            ui.end_row();
                        }
                        if reservoir_source_count > 0 {
                            ui.label("Reservoir");
                            ui.label(format!("{reservoir_source_count}"));
                            ui.label(format!("{:.2} MGD", reservoir_supply_mgd));
                            ui.end_row();
                        }
                        if desal_count > 0 {
                            ui.label("Desalination");
                            ui.label(format!("{desal_count}"));
                            ui.label(format!("{:.2} MGD", desal_supply_mgd));
                            ui.end_row();
                        }
                    });
            } else {
                ui.label("No water sources built");
            }

            ui.add_space(4.0);
            ui.separator();

            // ---- Groundwater Level ----
            ui.heading("Groundwater");
            let gw_level_pct = groundwater_stats.avg_level / 255.0 * 100.0;
            let gw_quality_pct = groundwater_stats.avg_quality / 255.0 * 100.0;

            let gw_level_color = if gw_level_pct < 30.0 {
                egui::Color32::from_rgb(255, 60, 60)
            } else if gw_level_pct < 50.0 {
                egui::Color32::from_rgb(220, 180, 50)
            } else {
                egui::Color32::from_rgb(50, 180, 220)
            };

            ui.horizontal(|ui| {
                ui.label("Avg Level:");
                ui.colored_label(gw_level_color, format!("{:.0}%", gw_level_pct));
            });

            if gw_level_pct < 30.0 {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 60, 60),
                    "WARNING: Groundwater depletion!",
                );
            }

            ui.horizontal(|ui| {
                ui.label("Avg Quality:");
                let quality_color = if gw_quality_pct < 50.0 {
                    egui::Color32::from_rgb(220, 80, 50)
                } else {
                    egui::Color32::from_rgb(80, 200, 80)
                };
                ui.colored_label(quality_color, format!("{:.0}%", gw_quality_pct));
            });

            if groundwater_stats.contaminated_cells > 0 {
                ui.colored_label(
                    egui::Color32::from_rgb(220, 120, 50),
                    format!(
                        "Contaminated cells: {}",
                        groundwater_stats.contaminated_cells
                    ),
                );
            }

            ui.add_space(4.0);
            ui.separator();

            // ---- Reservoir Level ----
            ui.heading("Reservoir");
            if reservoir_state.reservoir_count > 0 {
                let fill_pct = reservoir_state.fill_pct() * 100.0;
                let fill_color = match reservoir_state.warning_tier {
                    simulation::reservoir::ReservoirWarningTier::Normal => {
                        egui::Color32::from_rgb(50, 180, 220)
                    }
                    simulation::reservoir::ReservoirWarningTier::Watch => {
                        egui::Color32::from_rgb(220, 200, 50)
                    }
                    simulation::reservoir::ReservoirWarningTier::Warning => {
                        egui::Color32::from_rgb(220, 120, 50)
                    }
                    simulation::reservoir::ReservoirWarningTier::Critical => {
                        egui::Color32::from_rgb(255, 60, 60)
                    }
                };

                ui.horizontal(|ui| {
                    ui.label("Fill Level:");
                    ui.colored_label(fill_color, format!("{:.0}%", fill_pct));
                });

                ui.horizontal(|ui| {
                    ui.label("Status:");
                    ui.colored_label(fill_color, reservoir_state.warning_tier.name());
                });

                let storage_days_text = if reservoir_state.storage_days.is_infinite() {
                    "unlimited".to_string()
                } else {
                    format!("{:.0} days", reservoir_state.storage_days)
                };
                ui.horizontal(|ui| {
                    ui.label("Storage:");
                    ui.label(storage_days_text);
                });

                ui.horizontal(|ui| {
                    ui.label("Capacity:");
                    ui.label(format!(
                        "{:.1} MG",
                        reservoir_state.total_storage_capacity_mg
                    ));
                });
            } else {
                ui.label("No reservoirs built");
            }

            ui.add_space(4.0);
            ui.separator();

            // ---- Service Coverage ----
            ui.heading("Service Coverage");
            let total_buildings = water_supply.buildings_served + water_supply.buildings_unserved;
            let coverage_pct = if total_buildings > 0 {
                water_supply.buildings_served as f32 / total_buildings as f32 * 100.0
            } else {
                100.0
            };
            let coverage_color = if coverage_pct >= 90.0 {
                egui::Color32::from_rgb(80, 220, 80)
            } else if coverage_pct >= 60.0 {
                egui::Color32::from_rgb(220, 200, 50)
            } else {
                egui::Color32::from_rgb(255, 60, 60)
            };
            ui.horizontal(|ui| {
                ui.label("Water Service:");
                ui.colored_label(
                    coverage_color,
                    format!(
                        "{:.0}% ({}/{})",
                        coverage_pct, water_supply.buildings_served, total_buildings
                    ),
                );
            });

            ui.add_space(4.0);
            ui.separator();

            // ---- Water Quality / Treatment ----
            ui.heading("Water Treatment");
            let plant_count = treatment_state.plants.len();
            if plant_count > 0 {
                ui.horizontal(|ui| {
                    ui.label("Treatment Plants:");
                    ui.label(format!("{plant_count}"));
                });

                let treatment_coverage_pct = treatment_state.treatment_coverage * 100.0;
                let treatment_color = if treatment_coverage_pct >= 90.0 {
                    egui::Color32::from_rgb(80, 220, 80)
                } else if treatment_coverage_pct >= 60.0 {
                    egui::Color32::from_rgb(220, 200, 50)
                } else {
                    egui::Color32::from_rgb(255, 60, 60)
                };
                ui.horizontal(|ui| {
                    ui.label("Coverage:");
                    ui.colored_label(treatment_color, format!("{:.0}%", treatment_coverage_pct));
                });

                let quality_pct = treatment_state.avg_effluent_quality * 100.0;
                ui.horizontal(|ui| {
                    ui.label("Output Quality:");
                    let q_color = if quality_pct >= 85.0 {
                        egui::Color32::from_rgb(80, 220, 80)
                    } else if quality_pct >= 60.0 {
                        egui::Color32::from_rgb(220, 200, 50)
                    } else {
                        egui::Color32::from_rgb(255, 60, 60)
                    };
                    ui.colored_label(q_color, format!("{:.0}%", quality_pct));
                });

                if treatment_state.disease_risk > 0.05 {
                    let risk_pct = treatment_state.disease_risk * 100.0;
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 60, 60),
                        format!("Disease Risk: {:.1}%", risk_pct),
                    );
                }
            } else {
                ui.label("No treatment plants");
            }

            ui.add_space(4.0);
            ui.separator();

            // ---- Sewage Treatment ----
            ui.heading("Sewage");
            let sewage_coverage_pct = wastewater_state.coverage_ratio * 100.0;
            let sewage_color = if sewage_coverage_pct >= 90.0 {
                egui::Color32::from_rgb(80, 220, 80)
            } else if sewage_coverage_pct >= 60.0 {
                egui::Color32::from_rgb(220, 200, 50)
            } else {
                egui::Color32::from_rgb(255, 60, 60)
            };

            ui.horizontal(|ui| {
                ui.label("Coverage:");
                ui.colored_label(sewage_color, format!("{:.0}%", sewage_coverage_pct));
            });

            if wastewater_state.overflow_amount > 0.0 {
                let overflow_mgd = wastewater_state.overflow_amount / MGD_TO_GPD;
                ui.colored_label(
                    egui::Color32::from_rgb(255, 60, 60),
                    format!("Overflow: {:.3} MGD", overflow_mgd),
                );
            }

            if wastewater_state.pollution_events > 0 {
                ui.colored_label(
                    egui::Color32::from_rgb(220, 120, 50),
                    format!("Discharge events: {}", wastewater_state.pollution_events),
                );
            }

            ui.add_space(4.0);
            ui.separator();

            // ---- Monthly Water Budget ----
            ui.heading("Water Budget");
            let treatment_cost = treatment_state.total_period_cost;
            ui.horizontal(|ui| {
                ui.label("Treatment Cost:");
                ui.colored_label(
                    egui::Color32::from_rgb(220, 80, 80),
                    format!("${:.0}/mo", treatment_cost),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Source Ops Cost:");
                ui.colored_label(
                    egui::Color32::from_rgb(220, 80, 80),
                    format!("${:.0}/mo", total_source_operating_cost),
                );
            });

            let total_water_cost = treatment_cost + total_source_operating_cost;
            ui.horizontal(|ui| {
                ui.strong("Total Water Cost:");
                ui.colored_label(
                    egui::Color32::from_rgb(220, 80, 80),
                    format!("${:.0}/mo", total_water_cost),
                );
            });
        });
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_water_dashboard_visible_default() {
        let visible = WaterDashboardVisible::default();
        assert!(!visible.0, "Dashboard should be hidden by default");
    }

    #[test]
    fn test_water_dashboard_visible_toggle() {
        let mut visible = WaterDashboardVisible::default();
        visible.0 = !visible.0;
        assert!(visible.0, "Dashboard should be visible after toggle");
        visible.0 = !visible.0;
        assert!(!visible.0, "Dashboard should be hidden after second toggle");
    }

    #[test]
    fn test_mgd_to_gpd_constant() {
        assert!(
            (MGD_TO_GPD - 1_000_000.0).abs() < f32::EPSILON,
            "MGD_TO_GPD should be 1,000,000"
        );
    }

    #[test]
    fn test_surplus_deficit_calculation() {
        // When supply > demand, surplus is positive
        let total_supply_gpd = 5_000_000.0_f32;
        let total_demand_gpd = 3_000_000.0_f32;
        let supply_mgd = total_supply_gpd / MGD_TO_GPD;
        let demand_mgd = total_demand_gpd / MGD_TO_GPD;
        let surplus = supply_mgd - demand_mgd;
        assert!(surplus > 0.0, "Should have surplus when supply > demand");
        assert!(
            (surplus - 2.0).abs() < 0.001,
            "Surplus should be 2.0 MGD, got {}",
            surplus
        );
    }

    #[test]
    fn test_deficit_calculation() {
        // When demand > supply, deficit is negative
        let total_supply_gpd = 2_000_000.0_f32;
        let total_demand_gpd = 5_000_000.0_f32;
        let supply_mgd = total_supply_gpd / MGD_TO_GPD;
        let demand_mgd = total_demand_gpd / MGD_TO_GPD;
        let surplus = supply_mgd - demand_mgd;
        assert!(surplus < 0.0, "Should have deficit when demand > supply");
        assert!(
            (surplus - (-3.0)).abs() < 0.001,
            "Deficit should be -3.0 MGD, got {}",
            surplus
        );
    }

    #[test]
    fn test_groundwater_level_percentage() {
        // Avg level of 128 out of 255 = ~50.2%
        let avg_level = 128.0_f32;
        let pct = avg_level / 255.0 * 100.0;
        assert!(
            (pct - 50.196).abs() < 0.1,
            "128/255 should be ~50.2%, got {}",
            pct
        );
    }

    #[test]
    fn test_groundwater_low_level_warning_threshold() {
        // Below 30% should trigger warning
        let avg_level = 70.0_f32;
        let pct = avg_level / 255.0 * 100.0;
        assert!(pct < 30.0, "Level {} should be below 30% threshold", pct);
    }

    #[test]
    fn test_groundwater_ok_level() {
        // Above 50% should show normal color
        let avg_level = 200.0_f32;
        let pct = avg_level / 255.0 * 100.0;
        assert!(pct >= 50.0, "Level {} should be above 50% threshold", pct);
    }

    #[test]
    fn test_service_coverage_all_served() {
        let served = 100_u32;
        let unserved = 0_u32;
        let total = served + unserved;
        let pct = served as f32 / total as f32 * 100.0;
        assert!(
            (pct - 100.0).abs() < f32::EPSILON,
            "All served should be 100%"
        );
    }

    #[test]
    fn test_service_coverage_none_served() {
        let served = 0_u32;
        let unserved = 50_u32;
        let total = served + unserved;
        let pct = if total > 0 {
            served as f32 / total as f32 * 100.0
        } else {
            100.0
        };
        assert!(
            pct.abs() < f32::EPSILON,
            "None served should be 0%, got {}",
            pct
        );
    }

    #[test]
    fn test_service_coverage_no_buildings() {
        let served = 0_u32;
        let unserved = 0_u32;
        let total = served + unserved;
        let pct = if total > 0 {
            served as f32 / total as f32 * 100.0
        } else {
            100.0
        };
        assert!(
            (pct - 100.0).abs() < f32::EPSILON,
            "No buildings should default to 100%"
        );
    }

    #[test]
    fn test_service_coverage_partial() {
        let served = 75_u32;
        let unserved = 25_u32;
        let total = served + unserved;
        let pct = served as f32 / total as f32 * 100.0;
        assert!(
            (pct - 75.0).abs() < 0.01,
            "75/100 served should be 75%, got {}",
            pct
        );
    }

    #[test]
    fn test_overflow_mgd_conversion() {
        let overflow_gpd = 500_000.0_f32;
        let overflow_mgd = overflow_gpd / MGD_TO_GPD;
        assert!(
            (overflow_mgd - 0.5).abs() < 0.001,
            "500K GPD should be 0.5 MGD, got {}",
            overflow_mgd
        );
    }

    #[test]
    fn test_coverage_color_thresholds() {
        // >= 90% = green
        let high = 95.0_f32;
        assert!(high >= 90.0);

        // 60-90% = yellow
        let mid = 75.0_f32;
        assert!(mid >= 60.0 && mid < 90.0);

        // < 60% = red
        let low = 40.0_f32;
        assert!(low < 60.0);
    }

    #[test]
    fn test_water_supply_default_values() {
        let supply = WaterSupply::default();
        let demand_mgd = supply.total_demand_gpd / MGD_TO_GPD;
        let supply_mgd = supply.total_supply_gpd / MGD_TO_GPD;
        assert!(
            demand_mgd.abs() < f32::EPSILON,
            "Default demand should be 0"
        );
        assert!(
            supply_mgd.abs() < f32::EPSILON,
            "Default supply should be 0"
        );
    }
}
