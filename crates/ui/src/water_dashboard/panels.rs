//! Individual UI panel rendering functions for the water dashboard.

use bevy_egui::egui;

use simulation::groundwater::GroundwaterStats;
use simulation::reservoir::ReservoirState;
use simulation::wastewater::WastewaterState;
use simulation::water_demand::WaterSupply;
use simulation::water_treatment::WaterTreatmentState;

use super::types::{SourceAggregation, MGD_TO_GPD};

/// Renders the supply and demand overview panel.
pub fn render_supply_demand(ui: &mut egui::Ui, total_demand_mgd: f32, total_supply_mgd: f32) {
    let surplus_deficit_mgd = total_supply_mgd - total_demand_mgd;

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
}

/// Renders the source breakdown panel.
pub fn render_source_breakdown(ui: &mut egui::Ui, agg: &SourceAggregation) {
    ui.heading("Source Breakdown");
    if agg.has_sources() {
        egui::Grid::new("water_source_grid")
            .num_columns(3)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.strong("Source");
                ui.strong("Count");
                ui.strong("Capacity");
                ui.end_row();

                if agg.well_count > 0 {
                    ui.label("Wells");
                    ui.label(format!("{}", agg.well_count));
                    ui.label(format!("{:.2} MGD", agg.well_supply_mgd));
                    ui.end_row();
                }
                if agg.surface_count > 0 {
                    ui.label("Surface Intake");
                    ui.label(format!("{}", agg.surface_count));
                    ui.label(format!("{:.2} MGD", agg.surface_supply_mgd));
                    ui.end_row();
                }
                if agg.reservoir_source_count > 0 {
                    ui.label("Reservoir");
                    ui.label(format!("{}", agg.reservoir_source_count));
                    ui.label(format!("{:.2} MGD", agg.reservoir_supply_mgd));
                    ui.end_row();
                }
                if agg.desal_count > 0 {
                    ui.label("Desalination");
                    ui.label(format!("{}", agg.desal_count));
                    ui.label(format!("{:.2} MGD", agg.desal_supply_mgd));
                    ui.end_row();
                }
            });
    } else {
        ui.label("No water sources built");
    }
}

/// Renders the groundwater status panel.
pub fn render_groundwater(ui: &mut egui::Ui, groundwater_stats: &GroundwaterStats) {
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
}

/// Renders the reservoir status panel.
pub fn render_reservoir(ui: &mut egui::Ui, reservoir_state: &ReservoirState) {
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
}

/// Renders the service coverage panel.
pub fn render_service_coverage(ui: &mut egui::Ui, water_supply: &WaterSupply) {
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
}

/// Renders the water treatment panel.
pub fn render_water_treatment(ui: &mut egui::Ui, treatment_state: &WaterTreatmentState) {
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
}

/// Renders the sewage treatment panel.
pub fn render_sewage(ui: &mut egui::Ui, wastewater_state: &WastewaterState) {
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
}

/// Renders the monthly water budget panel.
pub fn render_water_budget(
    ui: &mut egui::Ui,
    treatment_state: &WaterTreatmentState,
    total_source_operating_cost: f64,
) {
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
}
