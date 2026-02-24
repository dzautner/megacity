//! Shared types, resources, and helper functions for the info panel.

use bevy::prelude::*;
use bevy_egui::egui;

use simulation::achievements::{AchievementNotification, AchievementTracker};
use simulation::advisors::AdvisorPanel;
use simulation::airport::AirportStats;
use simulation::death_care::DeathCareStats;
use simulation::districts::DistrictMap;
use simulation::education_jobs::EmploymentStats;
use simulation::forest_fire::ForestFireStats;
use simulation::groundwater::GroundwaterStats;
use simulation::heating::HeatingStats;
use simulation::homelessness::HomelessnessStats;
use simulation::immigration::{CityAttractiveness, ImmigrationStats};
use simulation::market::MarketPrices;
use simulation::natural_resources::ResourceBalance;
use simulation::outside_connections::OutsideConnections;
use simulation::postal::PostalStats;
use simulation::production::CityGoods;
use simulation::specialization::{CitySpecializations, SpecializationBonuses};
use simulation::weather::Weather;
use simulation::welfare::WelfareStats;
use simulation::wind::WindState;

// ---------------------------------------------------------------------------
// Shared types & resources
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct MinimapCache {
    pub texture_handle: Option<egui::TextureHandle>,
    pub dirty_timer: f32,
}

/// Resource controlling whether the event journal window is visible.
/// Toggle with 'J' key.
#[derive(Resource, Default)]
pub struct JournalVisible(pub bool);

/// Resource controlling whether the charts/trends window is visible.
/// Toggle with 'C' key.
#[derive(Resource, Default)]
pub struct ChartsVisible(pub bool);

/// Resource controlling whether the advisor window is visible.
/// Toggle with 'A' key.
#[derive(Resource, Default)]
pub struct AdvisorVisible(pub bool);

/// Resource controlling whether the policies window is visible.
/// Toggle with 'P' key.
#[derive(Resource, Default)]
pub struct PoliciesVisible(pub bool);

/// Resource controlling whether the budget breakdown window is visible.
/// Toggle with 'B' key.
#[derive(Resource, Default)]
pub struct BudgetPanelVisible(pub bool);

/// Bundled secondary resources for info_panel_ui to stay within the 16-param limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct InfoPanelExtras<'w> {
    pub resource_balance: Res<'w, ResourceBalance>,
    pub employment_stats: Res<'w, EmploymentStats>,
    pub homeless_stats: Res<'w, HomelessnessStats>,
    pub district_map: Res<'w, DistrictMap>,
    pub city_goods: Res<'w, CityGoods>,
    pub wind: Res<'w, WindState>,
    pub attractiveness: Res<'w, CityAttractiveness>,
    pub imm_stats: Res<'w, ImmigrationStats>,
    pub specializations: Res<'w, CitySpecializations>,
    pub spec_bonuses: Res<'w, SpecializationBonuses>,
    pub road_condition: Res<'w, simulation::road_maintenance::RoadConditionGrid>,
    pub road_maint_budget: ResMut<'w, simulation::road_maintenance::RoadMaintenanceBudget>,
    pub road_maint_stats: Res<'w, simulation::road_maintenance::RoadMaintenanceStats>,
    pub outside_connections: Res<'w, OutsideConnections>,
    pub death_care_stats: Res<'w, DeathCareStats>,
    pub market_prices: Res<'w, MarketPrices>,
    pub forest_fire_stats: Res<'w, ForestFireStats>,
    pub advisor_panel: Res<'w, AdvisorPanel>,
    pub accident_tracker: Res<'w, simulation::traffic_accidents::AccidentTracker>,
    pub achievement_tracker: Res<'w, AchievementTracker>,
    pub achievement_notifications: ResMut<'w, AchievementNotification>,
    pub welfare_stats: Res<'w, WelfareStats>,
    pub airport_stats: Res<'w, AirportStats>,
    pub postal_stats: Res<'w, PostalStats>,
    pub heating_stats: Res<'w, HeatingStats>,
    pub weather: Res<'w, Weather>,
    pub groundwater_stats: Res<'w, GroundwaterStats>,
    pub budget_visible: ResMut<'w, BudgetPanelVisible>,
}

// ---------------------------------------------------------------------------
// Shared helper functions
// ---------------------------------------------------------------------------

pub fn format_pop(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

pub fn demand_bar(ui: &mut egui::Ui, label: &str, value: f32, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(label);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(120.0, 16.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
        let fill_rect = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(rect.width() * value.clamp(0.0, 1.0), rect.height()),
        );
        painter.rect_filled(fill_rect, 2.0, color);
        ui.label(format!("{:.0}%", value * 100.0));
    });
}

pub fn coverage_bar(ui: &mut egui::Ui, label: &str, value: f32, color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(format!("{:>6}", label));
        let (rect, _) = ui.allocate_exact_size(egui::vec2(90.0, 12.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));
        let fill_rect = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(rect.width() * value.clamp(0.0, 1.0), rect.height()),
        );
        painter.rect_filled(fill_rect, 2.0, color);
        ui.label(format!("{:.0}%", value * 100.0));
    });
}
