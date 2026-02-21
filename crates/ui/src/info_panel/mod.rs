mod advisor;
mod budget;
mod building_inspection;
mod event_journal;
mod groundwater_tooltip;
mod keybinds;
mod minimap;
mod policies;

// Re-export all public items so the rest of the crate sees the same API.
pub use advisor::advisor_window_ui;
pub use budget::budget_panel_ui;
pub use building_inspection::building_inspection_ui;
pub use event_journal::event_journal_ui;
pub use groundwater_tooltip::groundwater_tooltip_ui;
pub use keybinds::{panel_keybinds, quick_save_load_keybinds};
pub use policies::policies_ui;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::achievements::{Achievement, AchievementNotification, AchievementTracker};
use simulation::advisors::AdvisorPanel;
use simulation::airport::AirportStats;
use simulation::config::CELL_SIZE;
use simulation::death_care::DeathCareStats;
use simulation::districts::DistrictMap;
use simulation::economy::CityBudget;
use simulation::education_jobs::{EmploymentStats, JobType};
use simulation::forest_fire::ForestFireStats;
use simulation::grid::{CellType, WorldGrid, ZoneType};
use simulation::groundwater::GroundwaterStats;
use simulation::heating::HeatingStats;
use simulation::homelessness::HomelessnessStats;
use simulation::immigration::{CityAttractiveness, ImmigrationStats};
use simulation::loans::{LoanBook, LoanTier};
use simulation::market::MarketPrices;
use simulation::natural_resources::ResourceBalance;
use simulation::outside_connections::OutsideConnections;
use simulation::postal::PostalStats;
use simulation::production::{CityGoods, GoodsType};
use simulation::services::ServiceBuilding;
use simulation::specialization::{
    CitySpecialization, CitySpecializations, SpecializationBonuses, SpecializationScore,
};
use simulation::stats::CityStats;
use simulation::weather::Weather;
use simulation::welfare::WelfareStats;
use simulation::wind::WindState;
use simulation::zones::ZoneDemand;

use rendering::overlay::{OverlayMode, OverlayState};

use minimap::{build_minimap_pixels, MINIMAP_SIZE};

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
    cache.timer -= time.delta_seconds();
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
}

// ---------------------------------------------------------------------------
// Shared helper functions
// ---------------------------------------------------------------------------

fn format_pop(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

fn demand_bar(ui: &mut egui::Ui, label: &str, value: f32, color: egui::Color32) {
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

fn coverage_bar(ui: &mut egui::Ui, label: &str, value: f32, color: egui::Color32) {
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

// ---------------------------------------------------------------------------
// Main info panel system
// ---------------------------------------------------------------------------

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
    coverage: Res<CoverageCache>,
    mut ext_budget: ResMut<simulation::budget::ExtendedBudget>,
    mut loan_book: ResMut<LoanBook>,
    mut extras: InfoPanelExtras,
) {
    let resource_balance = &extras.resource_balance;
    let employment_stats = &extras.employment_stats;
    let homeless_stats = &extras.homeless_stats;
    let district_map = &extras.district_map;
    let city_goods = &extras.city_goods;
    let wind = &extras.wind;
    let attractiveness = &extras.attractiveness;
    let imm_stats = &extras.imm_stats;
    let specializations = &extras.specializations;
    let _spec_bonuses = &extras.spec_bonuses;
    let road_maint_stats = &extras.road_maint_stats;
    // Update minimap every 2 seconds (or first frame)
    let needs_update = minimap_cache.texture_handle.is_none() || minimap_cache.dirty_timer <= 0.0;

    egui::SidePanel::right("info_panel")
        .default_width(200.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("City Stats");
            ui.separator();

            ui.label(format!("Population: {}", format_pop(stats.population)));
            ui.label(format!("Happiness: {:.0}%", stats.average_happiness));
            if homeless_stats.total_homeless > 0 {
                let homeless_color = egui::Color32::from_rgb(220, 120, 50);
                ui.colored_label(
                    homeless_color,
                    format!(
                        "Homeless: {} ({} sheltered)",
                        homeless_stats.total_homeless, homeless_stats.sheltered
                    ),
                );
            }
            // Welfare stats (show if any welfare infrastructure exists)
            {
                let ws = &extras.welfare_stats;
                if ws.shelter_count > 0 || ws.welfare_office_count > 0 {
                    let welfare_color = egui::Color32::from_rgb(90, 160, 140);
                    if ws.shelter_count > 0 {
                        ui.colored_label(
                            welfare_color,
                            format!(
                                "Shelters: {} ({}/{} beds)",
                                ws.shelter_count, ws.shelter_occupancy, ws.shelter_capacity
                            ),
                        );
                    }
                    if ws.welfare_office_count > 0 {
                        ui.colored_label(
                            welfare_color,
                            format!(
                                "Welfare: {} offices, {} recipients",
                                ws.welfare_office_count, ws.total_welfare_recipients
                            ),
                        );
                    }
                }
            }
            if extras.death_care_stats.unprocessed > 0 {
                ui.colored_label(
                    egui::Color32::from_rgb(220, 50, 50),
                    format!("Unprocessed: {}", extras.death_care_stats.unprocessed),
                );
            }
            if extras.forest_fire_stats.active_fires > 0 {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 60, 20),
                    format!(
                        "FOREST FIRE: {} cells burning!",
                        extras.forest_fire_stats.active_fires
                    ),
                );
            }
            ui.label(format!("Roads: {} cells", stats.road_cells));
            {
                let active_accidents = extras.accident_tracker.active_accidents.len();
                if active_accidents > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 200, 50),
                        format!("Accidents: {} active", active_accidents),
                    );
                }
            }
            ui.label(format!(
                "Wind: {} {}",
                wind.compass_direction(),
                wind.speed_label(),
            ));

            ui.separator();
            ui.heading("Buildings");
            ui.label(format!("Residential: {}", stats.residential_buildings));
            ui.label(format!("Commercial: {}", stats.commercial_buildings));
            ui.label(format!("Industrial: {}", stats.industrial_buildings));
            ui.label(format!("Office: {}", stats.office_buildings));

            ui.separator();
            ui.heading("RCIO Demand");

            // Demand bars
            let r_color = egui::Color32::from_rgb(50, 180, 50);
            let c_color = egui::Color32::from_rgb(50, 80, 200);
            let i_color = egui::Color32::from_rgb(200, 180, 30);
            let o_color = egui::Color32::from_rgb(160, 130, 220); // lavender

            demand_bar(ui, "R", demand.residential, r_color);
            demand_bar(ui, "C", demand.commercial, c_color);
            demand_bar(ui, "I", demand.industrial, i_color);
            demand_bar(ui, "O", demand.office, o_color);

            ui.separator();
            ui.heading("Employment");
            let unemp_color = if employment_stats.unemployment_rate > 0.10 {
                egui::Color32::from_rgb(220, 50, 50)
            } else if employment_stats.unemployment_rate > 0.05 {
                egui::Color32::from_rgb(220, 180, 50)
            } else {
                egui::Color32::from_rgb(50, 200, 50)
            };
            ui.colored_label(
                unemp_color,
                format!(
                    "Unemployment: {:.1}%",
                    employment_stats.unemployment_rate * 100.0
                ),
            );
            ui.label(format!(
                "Employed: {} | Unemployed: {}",
                format_pop(employment_stats.total_employed),
                format_pop(employment_stats.total_unemployed),
            ));
            for &jt in JobType::all() {
                let (filled, total) = employment_stats
                    .jobs_by_type
                    .get(&jt)
                    .copied()
                    .unwrap_or((0, 0));
                if total > 0 {
                    let fill_pct = filled as f32 / total as f32;
                    let bar_color = match jt {
                        JobType::Unskilled => egui::Color32::from_rgb(180, 140, 80),
                        JobType::Service => egui::Color32::from_rgb(80, 160, 180),
                        JobType::Skilled => egui::Color32::from_rgb(80, 180, 80),
                        JobType::Professional => egui::Color32::from_rgb(100, 100, 220),
                        JobType::Executive => egui::Color32::from_rgb(180, 80, 180),
                    };
                    coverage_bar(ui, jt.name(), fill_pct, bar_color);
                }
            }

            ui.separator();
            ui.collapsing("City Attractiveness", |ui| {
                // Overall score with color bar
                let score = attractiveness.overall_score;
                let score_color = if score >= 70.0 {
                    egui::Color32::from_rgb(50, 200, 50)
                } else if score >= 40.0 {
                    egui::Color32::from_rgb(220, 180, 50)
                } else {
                    egui::Color32::from_rgb(220, 50, 50)
                };

                ui.horizontal(|ui| {
                    ui.label("Score:");
                    ui.colored_label(score_color, format!("{:.0}/100", score));
                });

                // Score bar
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(160.0, 12.0), egui::Sense::hover());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
                let fill_rect = egui::Rect::from_min_size(
                    rect.min,
                    egui::vec2(
                        rect.width() * (score / 100.0).clamp(0.0, 1.0),
                        rect.height(),
                    ),
                );
                painter.rect_filled(fill_rect, 2.0, score_color);

                ui.add_space(4.0);

                // Breakdown factors
                let factor_color = |v: f32| -> egui::Color32 {
                    if v >= 0.7 {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else if v >= 0.4 {
                        egui::Color32::from_rgb(220, 180, 50)
                    } else {
                        egui::Color32::from_rgb(220, 50, 50)
                    }
                };

                egui::Grid::new("attractiveness_breakdown")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Employment:");
                        ui.colored_label(
                            factor_color(attractiveness.employment_factor),
                            format!("{:.0}%", attractiveness.employment_factor * 100.0),
                        );
                        ui.end_row();

                        ui.label("Happiness:");
                        ui.colored_label(
                            factor_color(attractiveness.happiness_factor),
                            format!("{:.0}%", attractiveness.happiness_factor * 100.0),
                        );
                        ui.end_row();

                        ui.label("Services:");
                        ui.colored_label(
                            factor_color(attractiveness.services_factor),
                            format!("{:.0}%", attractiveness.services_factor * 100.0),
                        );
                        ui.end_row();

                        ui.label("Housing:");
                        ui.colored_label(
                            factor_color(attractiveness.housing_factor),
                            format!("{:.0}%", attractiveness.housing_factor * 100.0),
                        );
                        ui.end_row();

                        ui.label("Tax:");
                        ui.colored_label(
                            factor_color(attractiveness.tax_factor),
                            format!("{:.0}%", attractiveness.tax_factor * 100.0),
                        );
                        ui.end_row();
                    });

                ui.add_space(4.0);

                // Migration stats
                let net = imm_stats.net_migration;
                let net_color = if net > 0 {
                    egui::Color32::from_rgb(50, 200, 50)
                } else if net < 0 {
                    egui::Color32::from_rgb(220, 50, 50)
                } else {
                    egui::Color32::from_rgb(180, 180, 180)
                };
                let sign = if net > 0 { "+" } else { "" };
                ui.horizontal(|ui| {
                    ui.label("Net migration:");
                    ui.colored_label(net_color, format!("{}{}", sign, net));
                });
                ui.label(format!(
                    "In: {} | Out: {}",
                    imm_stats.immigrants_this_month, imm_stats.emigrants_this_month
                ));
            });

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

            // ---- Road Maintenance ----
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
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(80.0, 12.0), egui::Sense::hover());
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

            ui.separator();
            ui.heading("Service Coverage");
            // Use cached coverage metrics (updated once per second)
            coverage_bar(
                ui,
                "Power",
                coverage.power,
                egui::Color32::from_rgb(220, 200, 50),
            );
            coverage_bar(
                ui,
                "Water",
                coverage.water,
                egui::Color32::from_rgb(50, 130, 220),
            );
            coverage_bar(
                ui,
                "Education",
                coverage.education,
                egui::Color32::from_rgb(100, 180, 220),
            );
            coverage_bar(
                ui,
                "Fire",
                coverage.fire,
                egui::Color32::from_rgb(220, 80, 50),
            );
            coverage_bar(
                ui,
                "Police",
                coverage.police,
                egui::Color32::from_rgb(50, 80, 200),
            );
            coverage_bar(
                ui,
                "Health",
                coverage.health,
                egui::Color32::from_rgb(220, 50, 120),
            );
            coverage_bar(
                ui,
                "Telecom",
                coverage.telecom,
                egui::Color32::from_rgb(150, 200, 80),
            );
            coverage_bar(
                ui,
                "Postal",
                extras.postal_stats.coverage_percentage / 100.0,
                egui::Color32::from_rgb(180, 130, 80),
            );

            // Heating coverage (only show when there's cold weather demand)
            {
                let heat_demand = simulation::heating::heating_demand(&extras.weather);
                if heat_demand > 0.0 {
                    let heat_color = egui::Color32::from_rgb(220, 120, 30);
                    coverage_bar(ui, "Heating", extras.heating_stats.coverage_pct, heat_color);
                    ui.small(format!(
                        "Demand: {:.0}% | Cost: ${:.0}/mo | Eff: {:.0}%",
                        heat_demand * 100.0,
                        extras.heating_stats.monthly_cost,
                        extras.heating_stats.efficiency * 100.0,
                    ));
                }
            }

            // ---- Groundwater ----
            ui.separator();
            ui.collapsing("Groundwater", |ui| {
                let gw = &extras.groundwater_stats;
                let level_pct = gw.avg_level / 255.0;
                let quality_pct = gw.avg_quality / 255.0;
                coverage_bar(
                    ui,
                    "Water Table",
                    level_pct,
                    egui::Color32::from_rgb(50, 120, 200),
                );
                let quality_color = if quality_pct >= 0.7 {
                    egui::Color32::from_rgb(50, 180, 80)
                } else if quality_pct >= 0.4 {
                    egui::Color32::from_rgb(200, 180, 50)
                } else {
                    egui::Color32::from_rgb(200, 60, 50)
                };
                coverage_bar(ui, "Water Quality", quality_pct, quality_color);
                if gw.contaminated_cells > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(200, 80, 50),
                        format!("{} contaminated cells", gw.contaminated_cells),
                    );
                }
                if gw.treatment_capacity > 0 {
                    ui.small(format!(
                        "{} treatment plant(s) active",
                        gw.treatment_capacity
                    ));
                }
            });

            // ---- Districts ----
            ui.separator();
            ui.heading("Districts");
            let has_any_district = district_map.districts.iter().any(|d| !d.cells.is_empty());
            if has_any_district {
                egui::Grid::new("district_stats_grid")
                    .num_columns(3)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("District");
                        ui.strong("Pop");
                        ui.strong("Happy");
                        ui.end_row();

                        for d in &district_map.districts {
                            if d.cells.is_empty() {
                                continue;
                            }
                            let color = egui::Color32::from_rgba_unmultiplied(
                                (d.color[0] * 255.0) as u8,
                                (d.color[1] * 255.0) as u8,
                                (d.color[2] * 255.0) as u8,
                                255,
                            );
                            ui.colored_label(color, &d.name);
                            ui.label(format_pop(d.stats.population));
                            if d.stats.population > 0 {
                                let h = d.stats.avg_happiness;
                                let h_color = if h >= 70.0 {
                                    egui::Color32::from_rgb(50, 200, 50)
                                } else if h >= 40.0 {
                                    egui::Color32::from_rgb(220, 180, 50)
                                } else {
                                    egui::Color32::from_rgb(220, 50, 50)
                                };
                                ui.colored_label(h_color, format!("{:.0}%", h));
                            } else {
                                ui.label("-");
                            }
                            ui.end_row();
                        }
                    });
            } else {
                ui.small("No districts painted yet.");
                ui.small("Use the Districts toolbar to paint.");
            }

            ui.separator();
            ui.collapsing("Outside Connections", |ui| {
                let outside = &extras.outside_connections;
                let conn_stats = outside.stats();

                for stat in &conn_stats {
                    ui.horizontal(|ui| {
                        // Status indicator: green circle if connected, grey if not
                        let (status_color, status_text) = if stat.active {
                            (egui::Color32::from_rgb(50, 200, 50), "Connected")
                        } else {
                            (egui::Color32::from_rgb(120, 120, 120), "Not available")
                        };

                        // Draw status dot
                        let (dot_rect, _) =
                            ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                        let painter = ui.painter_at(dot_rect);
                        painter.circle_filled(dot_rect.center(), 4.0, status_color);

                        ui.label(stat.connection_type.name());
                        ui.colored_label(status_color, status_text);
                    });

                    if stat.active {
                        // Utilization bar
                        ui.horizontal(|ui| {
                            ui.add_space(14.0); // indent
                            ui.label("Utilization:");
                            let (bar_rect, _) = ui
                                .allocate_exact_size(egui::vec2(80.0, 10.0), egui::Sense::hover());
                            let painter = ui.painter_at(bar_rect);
                            painter.rect_filled(bar_rect, 2.0, egui::Color32::from_gray(40));
                            let util_color = if stat.avg_utilization > 0.8 {
                                egui::Color32::from_rgb(220, 50, 50)
                            } else if stat.avg_utilization > 0.5 {
                                egui::Color32::from_rgb(220, 180, 50)
                            } else {
                                egui::Color32::from_rgb(50, 200, 50)
                            };
                            let fill_rect = egui::Rect::from_min_size(
                                bar_rect.min,
                                egui::vec2(
                                    bar_rect.width() * stat.avg_utilization.clamp(0.0, 1.0),
                                    bar_rect.height(),
                                ),
                            );
                            painter.rect_filled(fill_rect, 2.0, util_color);
                            ui.label(format!("{:.0}%", stat.avg_utilization * 100.0));
                        });

                        // Effect description
                        ui.horizontal(|ui| {
                            ui.add_space(14.0);
                            ui.small(stat.effect_description);
                        });

                        if stat.count > 1 {
                            ui.horizontal(|ui| {
                                ui.add_space(14.0);
                                ui.small(format!("{} connection points", stat.count));
                            });
                        }
                    }
                    ui.add_space(2.0);
                }
            });

            // ---- Aviation ----
            {
                let airport = &extras.airport_stats;
                if airport.total_airports > 0 {
                    ui.separator();
                    ui.collapsing("Aviation", |ui| {
                        egui::Grid::new("aviation_stats_grid")
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.label("Total airports:");
                                ui.label(format!("{}", airport.total_airports));
                                ui.end_row();

                                if airport.airports_by_tier[0] > 0 {
                                    ui.label("  Airstrips:");
                                    ui.label(format!("{}", airport.airports_by_tier[0]));
                                    ui.end_row();
                                }
                                if airport.airports_by_tier[1] > 0 {
                                    ui.label("  Regional:");
                                    ui.label(format!("{}", airport.airports_by_tier[1]));
                                    ui.end_row();
                                }
                                if airport.airports_by_tier[2] > 0 {
                                    ui.label("  International:");
                                    ui.label(format!("{}", airport.airports_by_tier[2]));
                                    ui.end_row();
                                }

                                ui.label("Passenger flights/mo:");
                                ui.label(format_pop(airport.passenger_flights_per_month));
                                ui.end_row();

                                ui.label("Cargo flights/mo:");
                                ui.label(format_pop(airport.cargo_flights_per_month));
                                ui.end_row();

                                ui.label("Tourism multiplier:");
                                let mult_color = if airport.tourism_multiplier > 1.5 {
                                    egui::Color32::from_rgb(50, 200, 50)
                                } else if airport.tourism_multiplier > 1.0 {
                                    egui::Color32::from_rgb(220, 200, 50)
                                } else {
                                    egui::Color32::from_rgb(180, 180, 180)
                                };
                                ui.colored_label(
                                    mult_color,
                                    format!("{:.2}x", airport.tourism_multiplier),
                                );
                                ui.end_row();

                                ui.label("Revenue:");
                                let rev_color = if airport.revenue > airport.total_monthly_cost {
                                    egui::Color32::from_rgb(50, 200, 50)
                                } else {
                                    egui::Color32::from_rgb(220, 50, 50)
                                };
                                ui.colored_label(rev_color, format!("${:.0}/mo", airport.revenue));
                                ui.end_row();

                                ui.label("Operating cost:");
                                ui.label(format!("${:.0}/mo", airport.total_monthly_cost));
                                ui.end_row();
                            });
                    });
                }
            }

            ui.separator();
            ui.collapsing("Economy: Production Chains", |ui| {
                for &g in GoodsType::all() {
                    let prod = city_goods.production_rate.get(&g).copied().unwrap_or(0.0);
                    let cons = city_goods.consumption_rate.get(&g).copied().unwrap_or(0.0);
                    let stock = city_goods.available.get(&g).copied().unwrap_or(0.0);
                    let net = prod - cons;

                    ui.horizontal(|ui| {
                        ui.label(format!("{:>14}", g.name()));

                        // Color the net value
                        let net_color = if net > 0.1 {
                            egui::Color32::from_rgb(50, 200, 50)
                        } else if net < -0.1 {
                            egui::Color32::from_rgb(220, 50, 50)
                        } else {
                            egui::Color32::from_rgb(180, 180, 180)
                        };

                        let sign = if net >= 0.0 { "+" } else { "" };
                        ui.colored_label(net_color, format!("{}{:.1}", sign, net));
                        ui.label(format!("({:.0})", stock));
                    });
                }

                // Trade balance
                ui.separator();
                let tb = city_goods.trade_balance;
                let tb_color = if tb > 0.0 {
                    egui::Color32::from_rgb(50, 200, 50)
                } else if tb < -1.0 {
                    egui::Color32::from_rgb(220, 50, 50)
                } else {
                    egui::Color32::from_rgb(180, 180, 180)
                };
                ui.horizontal(|ui| {
                    ui.label("Trade balance:");
                    ui.colored_label(tb_color, format!("${:.1}/tick", tb));
                });
            });

            ui.separator();
            ui.collapsing("Market Prices", |ui| {
                let market = &extras.market_prices;

                // Active market events
                if !market.active_events.is_empty() {
                    ui.label("Active Events:");
                    for active in &market.active_events {
                        let event_color = egui::Color32::from_rgb(255, 200, 50);
                        ui.colored_label(
                            event_color,
                            format!(
                                "{} ({} ticks left)",
                                active.event.name(),
                                active.remaining_ticks
                            ),
                        );
                    }
                    ui.add_space(4.0);
                }

                // Goods prices
                ui.label("Goods:");
                egui::Grid::new("market_goods_grid")
                    .num_columns(3)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Good");
                        ui.strong("Price");
                        ui.strong("Trend");
                        ui.end_row();

                        for &g in GoodsType::all() {
                            if let Some(entry) = market.goods_prices.get(&g) {
                                ui.label(g.name());
                                let mult = entry.multiplier();
                                let price_color = if mult > 1.15 {
                                    egui::Color32::from_rgb(220, 50, 50)
                                } else if mult < 0.85 {
                                    egui::Color32::from_rgb(50, 200, 50)
                                } else {
                                    egui::Color32::from_rgb(180, 180, 180)
                                };
                                ui.colored_label(
                                    price_color,
                                    format!("${:.1}", entry.current_price),
                                );

                                let trend = entry.trend();
                                let (trend_str, trend_color) = if trend > 0.1 {
                                    (
                                        format!("+{:.1} ^", trend),
                                        egui::Color32::from_rgb(220, 50, 50),
                                    )
                                } else if trend < -0.1 {
                                    (
                                        format!("{:.1} v", trend),
                                        egui::Color32::from_rgb(50, 200, 50),
                                    )
                                } else {
                                    ("~0.0 -".to_string(), egui::Color32::from_rgb(140, 140, 140))
                                };
                                ui.colored_label(trend_color, trend_str);
                                ui.end_row();
                            }
                        }
                    });

                ui.add_space(4.0);

                // Resource prices
                ui.label("Resources:");
                egui::Grid::new("market_resource_grid")
                    .num_columns(3)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Resource");
                        ui.strong("Price");
                        ui.strong("Trend");
                        ui.end_row();

                        for (&rt, entry) in &market.resource_prices {
                            ui.label(rt.name());
                            let mult = entry.multiplier();
                            let price_color = if mult > 1.15 {
                                egui::Color32::from_rgb(220, 50, 50)
                            } else if mult < 0.85 {
                                egui::Color32::from_rgb(50, 200, 50)
                            } else {
                                egui::Color32::from_rgb(180, 180, 180)
                            };
                            ui.colored_label(price_color, format!("${:.1}", entry.current_price));

                            let trend = entry.trend();
                            let (trend_str, trend_color) = if trend > 0.05 {
                                (
                                    format!("+{:.1} ^", trend),
                                    egui::Color32::from_rgb(220, 50, 50),
                                )
                            } else if trend < -0.05 {
                                (
                                    format!("{:.1} v", trend),
                                    egui::Color32::from_rgb(50, 200, 50),
                                )
                            } else {
                                ("~0.0 -".to_string(), egui::Color32::from_rgb(140, 140, 140))
                            };
                            ui.colored_label(trend_color, trend_str);
                            ui.end_row();
                        }
                    });
            });

            ui.separator();
            ui.collapsing("City Specializations", |ui| {
                for &spec in CitySpecialization::ALL {
                    let s = specializations.get(spec);
                    let level_name = SpecializationScore::level_name(s.level);
                    let level_color = match s.level {
                        0 => egui::Color32::from_rgb(140, 140, 140), // grey
                        1 => egui::Color32::from_rgb(220, 200, 50),  // yellow
                        2 => egui::Color32::from_rgb(50, 200, 50),   // green
                        3 => egui::Color32::from_rgb(255, 200, 50),  // gold
                        _ => egui::Color32::from_rgb(140, 140, 140),
                    };

                    ui.horizontal(|ui| {
                        ui.label(format!("{:>10}", spec.name()));
                        ui.colored_label(level_color, format!("[{}]", level_name));
                    });

                    // Score bar (0-100)
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(160.0, 10.0), egui::Sense::hover());
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
                    let fill_pct = (s.score / 100.0).clamp(0.0, 1.0);
                    let fill_rect = egui::Rect::from_min_size(
                        rect.min,
                        egui::vec2(rect.width() * fill_pct, rect.height()),
                    );
                    painter.rect_filled(fill_rect, 2.0, level_color);

                    ui.add_space(2.0);
                }
            });

            // ---- City Advisors ----
            ui.separator();
            ui.collapsing("City Advisors", |ui| {
                let messages = &extras.advisor_panel.messages;
                if messages.is_empty() {
                    ui.small("No advisor messages at this time.");
                } else {
                    for msg in messages {
                        let priority_color = match msg.priority {
                            5 => egui::Color32::from_rgb(220, 50, 50),   // red
                            4 => egui::Color32::from_rgb(230, 150, 30),  // orange
                            3 => egui::Color32::from_rgb(220, 200, 50),  // yellow
                            2 => egui::Color32::from_rgb(50, 130, 220),  // blue
                            _ => egui::Color32::from_rgb(150, 150, 150), // grey
                        };

                        ui.horizontal(|ui| {
                            // Colored dot for advisor type
                            let (dot_rect, _) = ui
                                .allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                            let painter = ui.painter_at(dot_rect);
                            painter.circle_filled(dot_rect.center(), 4.0, priority_color);

                            ui.colored_label(priority_color, msg.advisor_type.name());
                        });

                        ui.label(&msg.message);
                        ui.small(&msg.suggestion);
                        ui.add_space(4.0);
                    }
                }
            });

            // ---- Achievements ----
            ui.separator();
            {
                let tracker = &extras.achievement_tracker;
                let unlocked = tracker.unlocked_count();
                let total = Achievement::total_count();

                ui.collapsing(format!("Achievements ({}/{})", unlocked, total), |ui| {
                    for &achievement in Achievement::ALL {
                        let is_unlocked = tracker.is_unlocked(achievement);
                        ui.horizontal(|ui| {
                            if is_unlocked {
                                ui.colored_label(egui::Color32::from_rgb(50, 200, 50), "[v]");
                                ui.label(achievement.name());
                            } else {
                                ui.colored_label(egui::Color32::from_rgb(100, 100, 100), "[ ]");
                                ui.colored_label(
                                    egui::Color32::from_rgb(100, 100, 100),
                                    achievement.name(),
                                );
                            }
                        });
                        if is_unlocked {
                            ui.small(format!(
                                "  {} ({})",
                                achievement.description(),
                                achievement.reward().description(),
                            ));
                        } else {
                            ui.small(format!("  {}", achievement.description()));
                        }
                        ui.add_space(1.0);
                    }
                });
            }

            // ---- Achievement Notifications Popup ----
            {
                let recent = extras.achievement_notifications.take();
                if !recent.is_empty() {
                    for achievement in &recent {
                        ui.separator();
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 215, 0),
                            format!("Achievement Unlocked: {}", achievement.name()),
                        );
                        ui.small(format!("  Reward: {}", achievement.reward().description(),));
                    }
                }
            }

            ui.separator();
            ui.heading("Mini-map");

            // Overlay info
            let overlay_text = match overlay.mode {
                OverlayMode::None => "Tab to cycle overlays",
                OverlayMode::Power => "Power overlay [Tab]",
                OverlayMode::Water => "Water overlay [Tab]",
                OverlayMode::Traffic => "Traffic overlay [Tab]",
                OverlayMode::Pollution => "Pollution overlay [Tab]",
                OverlayMode::LandValue => "Land Value overlay [Tab]",
                OverlayMode::Education => "Education overlay [Tab]",
                OverlayMode::Garbage => "Garbage overlay [Tab]",
                OverlayMode::Noise => "Noise overlay [Tab]",
                OverlayMode::WaterPollution => "Water Pollution overlay [Tab]",
                OverlayMode::GroundwaterLevel => "GW Level overlay [Tab]",
                OverlayMode::GroundwaterQuality => "GW Quality overlay [Tab]",
                OverlayMode::Wind => "Wind overlay [Tab]",
            };
            ui.small(overlay_text);

            // Render minimap
            if needs_update {
                let pixels = build_minimap_pixels(&grid, &overlay);
                let color_image = egui::ColorImage {
                    size: [MINIMAP_SIZE, MINIMAP_SIZE],
                    pixels,
                };
                let texture =
                    ui.ctx()
                        .load_texture("minimap", color_image, egui::TextureOptions::NEAREST);
                minimap_cache.texture_handle = Some(texture);
                minimap_cache.dirty_timer = 2.0;
            }

            if let Some(ref tex) = minimap_cache.texture_handle {
                let size = egui::vec2(MINIMAP_SIZE as f32, MINIMAP_SIZE as f32);
                ui.image(egui::load::SizedTexture::new(tex.id(), size));
            }

            // Decrement timer using real delta time
            minimap_cache.dirty_timer -= time.delta_secs();
        });
}
