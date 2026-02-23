//! Shared types, resources, and helper functions for the info panel.

use bevy::prelude::*;
use bevy_egui::egui;

use simulation::achievements::{AchievementNotification, AchievementTracker};
use simulation::advisors::AdvisorPanel;
use simulation::airport::AirportStats;
use simulation::config::CELL_SIZE;
use simulation::death_care::DeathCareStats;
use simulation::districts::DistrictMap;
use simulation::education_jobs::EmploymentStats;
use simulation::forest_fire::ForestFireStats;
use simulation::grid::{CellType, WorldGrid, ZoneType};
use simulation::groundwater::GroundwaterStats;
use simulation::heating::HeatingStats;
use simulation::homelessness::HomelessnessStats;
use simulation::immigration::{CityAttractiveness, ImmigrationStats};
use simulation::market::MarketPrices;
use simulation::natural_resources::ResourceBalance;
use simulation::outside_connections::OutsideConnections;
use simulation::postal::PostalStats;
use simulation::production::CityGoods;
use simulation::services::ServiceBuilding;
use simulation::specialization::{CitySpecializations, SpecializationBonuses};
use simulation::weather::Weather;
use simulation::welfare::WelfareStats;
use simulation::wind::WindState;

use super::BudgetPanelVisible;

// ---------------------------------------------------------------------------
// Shared types & resources
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct MinimapCache {
    pub texture_handle: Option<egui::TextureHandle>,
    pub dirty_timer: f32,
}

/// Cached coverage metrics, updated once per second instead of every frame.
#[derive(Resource)]
pub struct CoverageCache {
    pub power: f32,
    pub water: f32,
    pub education: f32,
    pub fire: f32,
    pub police: f32,
    pub health: f32,
    pub telecom: f32,
    /// Seconds remaining until next refresh.
    timer: f32,
}

impl Default for CoverageCache {
    fn default() -> Self {
        Self {
            power: 0.0,
            water: 0.0,
            education: 0.0,
            fire: 0.0,
            police: 0.0,
            health: 0.0,
            telecom: 0.0,
            timer: 0.0, // refresh immediately on first frame
        }
    }
}

const COVERAGE_REFRESH_INTERVAL: f32 = 1.0;

pub fn update_coverage_cache(
    mut cache: ResMut<CoverageCache>,
    time: Res<Time>,
    grid: Res<WorldGrid>,
    services: Query<&ServiceBuilding>,
) {
    cache.timer -= time.delta_secs();
    if cache.timer > 0.0 {
        return;
    }
    cache.timer = COVERAGE_REFRESH_INTERVAL;

    let (power, water) = compute_utility_coverage(&grid);
    cache.power = power;
    cache.water = water;
    cache.education = compute_service_coverage(&services, &grid, "edu");
    cache.fire = compute_service_coverage(&services, &grid, "fire");
    cache.police = compute_service_coverage(&services, &grid, "police");
    cache.health = compute_service_coverage(&services, &grid, "health");
    cache.telecom = compute_service_coverage(&services, &grid, "telecom");
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

fn compute_utility_coverage(grid: &WorldGrid) -> (f32, f32) {
    let mut total = 0u32;
    let mut powered = 0u32;
    let mut watered = 0u32;
    for cell in &grid.cells {
        if cell.cell_type == CellType::Grass && cell.zone != ZoneType::None {
            total += 1;
            if cell.has_power {
                powered += 1;
            }
            if cell.has_water {
                watered += 1;
            }
        }
    }
    if total == 0 {
        return (1.0, 1.0);
    }
    (powered as f32 / total as f32, watered as f32 / total as f32)
}

fn compute_service_coverage(
    services: &Query<&ServiceBuilding>,
    grid: &WorldGrid,
    category: &str,
) -> f32 {
    let mut covered_cells = 0u32;
    let total_zoned = grid
        .cells
        .iter()
        .filter(|c| c.zone != ZoneType::None)
        .count() as f32;
    if total_zoned == 0.0 {
        return 0.0;
    }

    for service in services.iter() {
        let matches = match category {
            "edu" => ServiceBuilding::is_education(service.service_type),
            "fire" => ServiceBuilding::is_fire(service.service_type),
            "police" => ServiceBuilding::is_police(service.service_type),
            "health" => ServiceBuilding::is_health(service.service_type),
            "telecom" => ServiceBuilding::is_telecom(service.service_type),
            _ => false,
        };
        if matches {
            let radius_cells = service.radius / CELL_SIZE;
            covered_cells += (std::f32::consts::PI * radius_cells * radius_cells) as u32;
        }
    }

    (covered_cells as f32 / total_zoned).min(1.0)
}
