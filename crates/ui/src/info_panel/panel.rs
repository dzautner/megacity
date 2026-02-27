//! Main info panel system that composes all sub-sections.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::coverage_metrics::CoverageMetrics;
use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::loans::LoanBook;
use simulation::new_game_config::NewGameConfig;
use simulation::stats::CityStats;
use simulation::zones::ZoneDemand;

use rendering::overlay::OverlayState;

use super::city_overview;
use super::economy_section;
use super::finance_section;
use super::services_section;
use super::types::{InfoPanelExtras, MinimapCache};

/// Main info panel system — renders the right-side panel with all city info.
#[allow(clippy::too_many_arguments)]
pub fn info_panel_ui(
    mut contexts: EguiContexts,
    stats: Res<CityStats>,
    mut budget: ResMut<CityBudget>,
    demand: Res<ZoneDemand>,
    grid: Res<WorldGrid>,
    overlay: Res<OverlayState>,
    mut minimap_cache: Local<MinimapCache>,
    time: Res<Time>,
    coverage: Res<CoverageMetrics>,
    mut ext_budget: ResMut<simulation::budget::ExtendedBudget>,
    mut loan_book: ResMut<LoanBook>,
    mut extras: InfoPanelExtras,
    new_game_config: Res<NewGameConfig>,
) {
    egui::SidePanel::right("info_panel")
        .default_width(200.0)
        .show(contexts.ctx_mut(), |ui| {
            // City Stats, Buildings, RCIO Demand, Employment
            city_overview::draw_city_stats(ui, &stats, &demand, &extras, &new_game_config);

            // City Attractiveness
            city_overview::draw_attractiveness(ui, &extras.attractiveness, &extras.imm_stats);

            // Budget overview + per-zone tax sliders
            finance_section::draw_budget(ui, &mut budget, &mut ext_budget, &mut extras);

            // Road Maintenance + Traffic Safety
            finance_section::draw_road_maintenance(ui, &mut extras);

            // Finance (loans, credit rating, trade balance)
            finance_section::draw_finance(ui, &mut budget, &mut loan_book, &extras);

            // Service budget sliders
            finance_section::draw_service_budgets(ui, &mut ext_budget);

            // Service coverage bars
            services_section::draw_service_coverage(ui, &coverage, &extras);

            // Groundwater
            services_section::draw_groundwater(ui, &extras);

            // Districts
            services_section::draw_districts(ui, &extras.district_map);

            // Outside Connections
            services_section::draw_outside_connections(ui, &extras);

            // Aviation
            services_section::draw_aviation(ui, &extras);

            // Economy: Production Chains
            economy_section::draw_production_chains(ui, &extras);

            // Market Prices
            economy_section::draw_market_prices(ui, &extras);

            // City Specializations
            economy_section::draw_specializations(ui, &extras);

            // City Advisors
            economy_section::draw_advisors(ui, &extras);

            // Achievements
            economy_section::draw_achievements(ui, &mut extras);

            // Mini-map
            economy_section::draw_minimap(ui, &grid, &overlay, &mut minimap_cache, &time);
        });
}
