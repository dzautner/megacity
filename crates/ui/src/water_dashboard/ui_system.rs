//! Main water dashboard UI system.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::groundwater::GroundwaterStats;
use simulation::reservoir::ReservoirState;
use simulation::wastewater::WastewaterState;
use simulation::water_demand::WaterSupply;
use simulation::water_sources::{WaterSource, WaterSourceType};
use simulation::water_treatment::WaterTreatmentState;

use super::panels;
use super::types::{SourceAggregation, WaterDashboardVisible, MGD_TO_GPD};

/// Aggregates water source contributions by type from all source entities.
fn aggregate_sources(sources: &Query<&WaterSource>) -> SourceAggregation {
    let mut agg = SourceAggregation {
        well_supply_mgd: 0.0,
        surface_supply_mgd: 0.0,
        reservoir_supply_mgd: 0.0,
        desal_supply_mgd: 0.0,
        well_count: 0,
        surface_count: 0,
        reservoir_source_count: 0,
        desal_count: 0,
        total_source_operating_cost: 0.0,
    };

    for source in sources {
        match source.source_type {
            WaterSourceType::Well => {
                agg.well_supply_mgd += source.capacity_mgd;
                agg.well_count += 1;
            }
            WaterSourceType::SurfaceIntake => {
                agg.surface_supply_mgd += source.capacity_mgd;
                agg.surface_count += 1;
            }
            WaterSourceType::Reservoir => {
                agg.reservoir_supply_mgd += source.capacity_mgd;
                agg.reservoir_source_count += 1;
            }
            WaterSourceType::Desalination => {
                agg.desal_supply_mgd += source.capacity_mgd;
                agg.desal_count += 1;
            }
        }
        agg.total_source_operating_cost += source.operating_cost;
    }

    agg
}

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

    let agg = aggregate_sources(&sources);
    let total_demand_mgd = water_supply.total_demand_gpd / MGD_TO_GPD;
    let total_supply_mgd = water_supply.total_supply_gpd / MGD_TO_GPD;

    egui::Window::new("Water Supply Dashboard")
        .default_open(true)
        .default_width(360.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Water dashboard");
            ui.separator();

            panels::render_supply_demand(ui, total_demand_mgd, total_supply_mgd);

            ui.add_space(4.0);
            ui.separator();

            panels::render_source_breakdown(ui, &agg);

            ui.add_space(4.0);
            ui.separator();

            panels::render_groundwater(ui, &groundwater_stats);

            ui.add_space(4.0);
            ui.separator();

            panels::render_reservoir(ui, &reservoir_state);

            ui.add_space(4.0);
            ui.separator();

            panels::render_service_coverage(ui, &water_supply);

            ui.add_space(4.0);
            ui.separator();

            panels::render_water_treatment(ui, &treatment_state);

            ui.add_space(4.0);
            ui.separator();

            panels::render_sewage(ui, &wastewater_state);

            ui.add_space(4.0);
            ui.separator();

            panels::render_water_budget(ui, &treatment_state, agg.total_source_operating_cost);
        });
}
