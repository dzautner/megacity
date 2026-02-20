use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::achievements::{Achievement, AchievementNotification, AchievementTracker};
use simulation::advisors::AdvisorPanel;
use simulation::airport::AirportStats;
use simulation::buildings::Building;
use simulation::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    Personality, WorkLocation,
};
use simulation::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use simulation::death_care::DeathCareStats;
use simulation::districts::DistrictMap;
use simulation::economy::CityBudget;
use simulation::education_jobs::{EmploymentStats, JobType};
use simulation::events::{ActiveCityEffects, EventJournal};
use simulation::forest_fire::ForestFireStats;
use simulation::grid::{CellType, WorldGrid, ZoneType};
use simulation::groundwater::GroundwaterStats;
use simulation::heating::HeatingStats;
use simulation::homelessness::HomelessnessStats;
use simulation::immigration::{CityAttractiveness, ImmigrationStats};
use simulation::land_value::LandValueGrid;
use simulation::loans::{LoanBook, LoanTier};
use simulation::market::MarketPrices;
use simulation::natural_resources::ResourceBalance;
use simulation::outside_connections::OutsideConnections;
use simulation::policies::{Policies, Policy};
use simulation::pollution::PollutionGrid;
use simulation::postal::PostalStats;
use simulation::production::{CityGoods, GoodsType};
use simulation::services::ServiceBuilding;
use simulation::specialization::{
    CitySpecialization, CitySpecializations, SpecializationBonuses, SpecializationScore,
};
use simulation::stats::CityStats;
use simulation::utilities::UtilitySource;
use simulation::wealth::WealthTier;
use simulation::weather::Weather;
use simulation::welfare::WelfareStats;
use simulation::wind::WindState;
use simulation::zones::ZoneDemand;

use rendering::input::SelectedBuilding;
use rendering::overlay::{OverlayMode, OverlayState};

const MINIMAP_SIZE: usize = 128;
const SAMPLE_STEP: usize = 2; // Sample every Nth cell

#[derive(Resource, Default)]
pub struct MinimapCache {
    pub texture_handle: Option<egui::TextureHandle>,
    pub dirty_timer: f32,
}

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

fn format_pop(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

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
    services: Query<&ServiceBuilding>,
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
            // Coverage bars (placeholder values derived from grid)
            let (power_cov, water_cov) = compute_utility_coverage(&grid);
            coverage_bar(
                ui,
                "Power",
                power_cov,
                egui::Color32::from_rgb(220, 200, 50),
            );
            coverage_bar(
                ui,
                "Water",
                water_cov,
                egui::Color32::from_rgb(50, 130, 220),
            );
            // Education / fire / police / health computed from ServiceBuilding entities
            coverage_bar(
                ui,
                "Education",
                compute_service_coverage(&services, &grid, "edu"),
                egui::Color32::from_rgb(100, 180, 220),
            );
            coverage_bar(
                ui,
                "Fire",
                compute_service_coverage(&services, &grid, "fire"),
                egui::Color32::from_rgb(220, 80, 50),
            );
            coverage_bar(
                ui,
                "Police",
                compute_service_coverage(&services, &grid, "police"),
                egui::Color32::from_rgb(50, 80, 200),
            );
            coverage_bar(
                ui,
                "Health",
                compute_service_coverage(&services, &grid, "health"),
                egui::Color32::from_rgb(220, 50, 120),
            );
            coverage_bar(
                ui,
                "Telecom",
                compute_service_coverage(&services, &grid, "telecom"),
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
                OverlayMode::None => "P=Pwr O=Wtr T=Trf N=Pol L=LV E=Edu G=Gar M=Noi U=WP W=GW",
                OverlayMode::Power => "Power overlay active [P]",
                OverlayMode::Water => "Water overlay active [O]",
                OverlayMode::Traffic => "Traffic overlay active [T]",
                OverlayMode::Pollution => "Pollution overlay active [N]",
                OverlayMode::LandValue => "Land Value overlay active [L]",
                OverlayMode::Education => "Education overlay active [E]",
                OverlayMode::Garbage => "Garbage overlay active [G]",
                OverlayMode::Noise => "Noise overlay active [M]",
                OverlayMode::WaterPollution => "Water Pollution overlay active [U]",
                OverlayMode::GroundwaterLevel => {
                    "GW Level overlay active [W] (press W for Quality)"
                }
                OverlayMode::GroundwaterQuality => {
                    "GW Quality overlay active [W] (press W to close)"
                }
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

// ---------------------------------------------------------------------------
// Building Inspection Panel
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn education_name(level: u8) -> &'static str {
    match level {
        0 => "None",
        1 => "Elementary",
        2 => "High School",
        3 => "University",
        _ => "Advanced",
    }
}

fn zone_type_name(zone: ZoneType) -> &'static str {
    match zone {
        ZoneType::None => "Unzoned",
        ZoneType::ResidentialLow => "Low-Density Residential",
        ZoneType::ResidentialMedium => "Medium-Density Residential",
        ZoneType::ResidentialHigh => "High-Density Residential",
        ZoneType::CommercialLow => "Low-Density Commercial",
        ZoneType::CommercialHigh => "High-Density Commercial",
        ZoneType::Industrial => "Industrial",
        ZoneType::Office => "Office",
        ZoneType::MixedUse => "Mixed-Use",
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn building_inspection_ui(
    mut contexts: EguiContexts,
    selected: Res<SelectedBuilding>,
    buildings: Query<&Building>,
    service_buildings: Query<&ServiceBuilding>,
    utility_sources: Query<&UtilitySource>,
    citizens: Query<
        (
            Entity,
            &CitizenDetails,
            &HomeLocation,
            Option<&WorkLocation>,
            &CitizenStateComp,
            Option<&Needs>,
            Option<&Personality>,
            Option<&Family>,
        ),
        With<Citizen>,
    >,
    grid: Res<WorldGrid>,
    pollution: Res<PollutionGrid>,
    land_value: Res<LandValueGrid>,
    budget: Res<CityBudget>,
) {
    let Some(entity) = selected.0 else {
        return;
    };

    // Zone building inspection
    if let Ok(building) = buildings.get(entity) {
        let cell = grid.get(building.grid_x, building.grid_y);
        let idx = building.grid_y * GRID_WIDTH + building.grid_x;
        let poll_level = pollution.levels.get(idx).copied().unwrap_or(0);
        let lv = land_value.values.get(idx).copied().unwrap_or(0);
        let occupancy_pct = if building.capacity > 0 {
            (building.occupants as f32 / building.capacity as f32 * 100.0).min(100.0)
        } else {
            0.0
        };

        egui::Window::new("Building Inspector")
            .default_width(320.0)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
            .show(contexts.ctx_mut(), |ui| {
                ui.heading(zone_type_name(building.zone_type));
                ui.separator();

                // Building overview
                egui::Grid::new("building_overview")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Level:");
                        ui.label(format!(
                            "{} / {}",
                            building.level,
                            building.zone_type.max_level()
                        ));
                        ui.end_row();

                        ui.label("Occupancy:");
                        let occ_color = if occupancy_pct >= 90.0 {
                            egui::Color32::from_rgb(220, 50, 50)
                        } else if occupancy_pct >= 70.0 {
                            egui::Color32::from_rgb(220, 180, 50)
                        } else {
                            egui::Color32::from_rgb(50, 200, 50)
                        };
                        ui.colored_label(
                            occ_color,
                            format!(
                                "{} / {} ({:.0}%)",
                                building.occupants, building.capacity, occupancy_pct
                            ),
                        );
                        ui.end_row();

                        ui.label("Location:");
                        ui.label(format!("({}, {})", building.grid_x, building.grid_y));
                        ui.end_row();

                        ui.label("Land Value:");
                        ui.label(format!("{}/255", lv));
                        ui.end_row();

                        ui.label("Pollution:");
                        let poll_color = if poll_level > 50 {
                            egui::Color32::from_rgb(200, 50, 50)
                        } else if poll_level > 20 {
                            egui::Color32::from_rgb(200, 150, 50)
                        } else {
                            egui::Color32::from_rgb(50, 200, 50)
                        };
                        ui.colored_label(poll_color, format!("{}/255", poll_level));
                        ui.end_row();
                    });

                // Power/Water status
                ui.separator();
                ui.horizontal(|ui| {
                    power_water_labels(ui, cell.has_power, cell.has_water);
                });

                if building.zone_type.is_residential() {
                    // Residential: show resident info
                    ui.separator();
                    ui.heading("Residents");

                    let mut residents: Vec<(
                        Entity,
                        &CitizenDetails,
                        &CitizenStateComp,
                        Option<&Needs>,
                        Option<&Personality>,
                        Option<&Family>,
                    )> = citizens
                        .iter()
                        .filter(|(_, _, home, _, _, _, _, _)| home.building == entity)
                        .map(|(e, details, _, _, state, needs, pers, fam)| {
                            (e, details, state, needs, pers, fam)
                        })
                        .collect();

                    let count = residents.len();
                    ui.label(format!("{} residents tracked (entity-backed)", count));

                    if !residents.is_empty() {
                        let avg_happiness: f32 =
                            residents.iter().map(|r| r.1.happiness).sum::<f32>() / count as f32;
                        let avg_age: f32 =
                            residents.iter().map(|r| r.1.age as f32).sum::<f32>() / count as f32;
                        let avg_salary: f32 =
                            residents.iter().map(|r| r.1.salary).sum::<f32>() / count as f32;
                        let males = residents
                            .iter()
                            .filter(|r| r.1.gender == Gender::Male)
                            .count();
                        let children = residents
                            .iter()
                            .filter(|r| {
                                r.1.life_stage().should_attend_school()
                                    || !r.1.life_stage().can_work()
                            })
                            .count();

                        egui::Grid::new("res_summary")
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.label("Avg happiness:");
                                happiness_label(ui, avg_happiness);
                                ui.end_row();
                                ui.label("Avg age:");
                                ui.label(format!("{:.0}", avg_age));
                                ui.end_row();
                                ui.label("Gender:");
                                ui.label(format!("{} M / {} F", males, count - males));
                                ui.end_row();
                                ui.label("Children:");
                                ui.label(format!("{}", children));
                                ui.end_row();
                                ui.label("Avg salary:");
                                ui.label(format!("${:.0}/mo", avg_salary));
                                ui.end_row();
                                ui.label("Tax revenue:");
                                let tax: f32 =
                                    residents.iter().map(|r| r.1.salary * budget.tax_rate).sum();
                                ui.label(format!("${:.0}/mo", tax));
                                ui.end_row();
                            });

                        // Average needs satisfaction
                        let needs_count = residents.iter().filter(|r| r.3.is_some()).count();
                        if needs_count > 0 {
                            ui.separator();
                            ui.label("Average Needs:");
                            let (avg_h, avg_e, avg_s, avg_f, avg_c) = residents
                                .iter()
                                .filter_map(|r| r.3)
                                .fold((0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32), |acc, n| {
                                    (
                                        acc.0 + n.hunger,
                                        acc.1 + n.energy,
                                        acc.2 + n.social,
                                        acc.3 + n.fun,
                                        acc.4 + n.comfort,
                                    )
                                });
                            let nc = needs_count as f32;
                            needs_bar(ui, "Hunger", avg_h / nc);
                            needs_bar(ui, "Energy", avg_e / nc);
                            needs_bar(ui, "Social", avg_s / nc);
                            needs_bar(ui, "Fun", avg_f / nc);
                            needs_bar(ui, "Comfort", avg_c / nc);
                        }

                        // Education breakdown
                        ui.separator();
                        ui.label("Education breakdown:");
                        let mut edu_counts = [0u32; 4];
                        for r in &residents {
                            let idx = (r.1.education as usize).min(3);
                            edu_counts[idx] += 1;
                        }
                        egui::Grid::new("edu_breakdown")
                            .num_columns(2)
                            .show(ui, |ui| {
                                for (i, name) in ["None", "Elementary", "High School", "University"]
                                    .iter()
                                    .enumerate()
                                {
                                    ui.label(*name);
                                    ui.label(format!("{}", edu_counts[i]));
                                    ui.end_row();
                                }
                            });

                        // Wealth breakdown
                        ui.separator();
                        ui.label("Income distribution:");
                        let mut wealth_counts = [0u32; 3];
                        for r in &residents {
                            match WealthTier::from_education(r.1.education) {
                                WealthTier::LowIncome => wealth_counts[0] += 1,
                                WealthTier::MiddleIncome => wealth_counts[1] += 1,
                                WealthTier::HighIncome => wealth_counts[2] += 1,
                            }
                        }
                        egui::Grid::new("wealth_breakdown")
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.label("Low income");
                                ui.label(format!("{}", wealth_counts[0]));
                                ui.end_row();
                                ui.label("Middle income");
                                ui.label(format!("{}", wealth_counts[1]));
                                ui.end_row();
                                ui.label("High income");
                                ui.label(format!("{}", wealth_counts[2]));
                                ui.end_row();
                            });

                        // Individual resident list (scrollable, up to 50)
                        ui.separator();
                        ui.label("Individual Residents:");
                        residents.sort_by(|a, b| {
                            b.1.happiness
                                .partial_cmp(&a.1.happiness)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });

                        egui::ScrollArea::vertical()
                            .max_height(280.0)
                            .show(ui, |ui| {
                                egui::Grid::new("residents_list")
                                    .num_columns(7)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.strong("Name");
                                        ui.strong("Age");
                                        ui.strong("Edu");
                                        ui.strong("Happy");
                                        ui.strong("Salary");
                                        ui.strong("Needs");
                                        ui.strong("Status");
                                        ui.end_row();

                                        for (
                                            i,
                                            (ent, details, state, needs, _personality, _family),
                                        ) in residents.iter().enumerate()
                                        {
                                            if i >= 50 {
                                                break;
                                            }
                                            ui.label(citizen_name(*ent, details.gender));
                                            ui.label(format!("{}", details.age));
                                            ui.label(education_abbrev(details.education));
                                            happiness_label(ui, details.happiness);
                                            ui.label(format!("${:.0}", details.salary));
                                            if let Some(n) = needs {
                                                let sat = n.overall_satisfaction();
                                                let color = if sat > 0.7 {
                                                    egui::Color32::from_rgb(50, 200, 50)
                                                } else if sat > 0.4 {
                                                    egui::Color32::from_rgb(220, 180, 50)
                                                } else {
                                                    egui::Color32::from_rgb(220, 50, 50)
                                                };
                                                ui.colored_label(
                                                    color,
                                                    format!("{:.0}%", sat * 100.0),
                                                );
                                            } else {
                                                ui.label("-");
                                            }
                                            ui.label(state_name(state.0));
                                            ui.end_row();
                                        }
                                    });

                                if count > 50 {
                                    ui.label(format!("... and {} more", count - 50));
                                }
                            });
                    }
                } else {
                    // Commercial/Industrial/Office: show worker info
                    ui.separator();
                    ui.heading("Workers");

                    let mut workers: Vec<(Entity, &CitizenDetails, &CitizenStateComp)> = citizens
                        .iter()
                        .filter(|(_, _, _, work, _, _, _, _)| {
                            work.map(|w| w.building == entity).unwrap_or(false)
                        })
                        .map(|(e, details, _, _, state, _, _, _)| (e, details, state))
                        .collect();

                    let count = workers.len();
                    ui.label(format!("{} workers tracked", count));

                    if !workers.is_empty() {
                        let avg_happiness: f32 =
                            workers.iter().map(|w| w.1.happiness).sum::<f32>() / count as f32;
                        let avg_salary: f32 =
                            workers.iter().map(|w| w.1.salary).sum::<f32>() / count as f32;

                        egui::Grid::new("worker_summary")
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.label("Avg happiness:");
                                happiness_label(ui, avg_happiness);
                                ui.end_row();
                                ui.label("Avg salary:");
                                ui.label(format!("${:.0}/mo", avg_salary));
                                ui.end_row();
                                ui.label("Payroll:");
                                ui.label(format!("${:.0}/mo", avg_salary * count as f32));
                                ui.end_row();
                            });

                        // Education breakdown
                        ui.separator();
                        let mut edu_counts = [0u32; 4];
                        for w in &workers {
                            let idx = (w.1.education as usize).min(3);
                            edu_counts[idx] += 1;
                        }
                        ui.label("Workforce education:");
                        egui::Grid::new("worker_edu").num_columns(2).show(ui, |ui| {
                            for (i, name) in ["None", "Elementary", "High School", "University"]
                                .iter()
                                .enumerate()
                            {
                                ui.label(*name);
                                ui.label(format!("{}", edu_counts[i]));
                                ui.end_row();
                            }
                        });

                        // Individual worker list
                        ui.separator();
                        ui.label("Individual Workers:");
                        workers.sort_by(|a, b| {
                            b.1.happiness
                                .partial_cmp(&a.1.happiness)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });

                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                egui::Grid::new("workers_list")
                                    .num_columns(5)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.strong("Name");
                                        ui.strong("Age");
                                        ui.strong("Edu");
                                        ui.strong("Happy");
                                        ui.strong("Salary");
                                        ui.end_row();

                                        for (i, (ent, details, _state)) in
                                            workers.iter().enumerate()
                                        {
                                            if i >= 50 {
                                                break;
                                            }
                                            ui.label(citizen_name(*ent, details.gender));
                                            ui.label(format!("{}", details.age));
                                            ui.label(education_abbrev(details.education));
                                            happiness_label(ui, details.happiness);
                                            ui.label(format!("${:.0}", details.salary));
                                            ui.end_row();
                                        }
                                    });

                                if count > 50 {
                                    ui.label(format!("... and {} more", count - 50));
                                }
                            });
                    }
                }
            });
        return;
    }

    // Service building inspection
    if let Ok(service) = service_buildings.get(entity) {
        let cell = grid.get(service.grid_x, service.grid_y);
        let idx = service.grid_y * GRID_WIDTH + service.grid_x;
        let lv = land_value.values.get(idx).copied().unwrap_or(0);
        let monthly_cost =
            simulation::services::ServiceBuilding::monthly_maintenance(service.service_type);

        egui::Window::new("Building Inspector")
            .default_width(280.0)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
            .show(contexts.ctx_mut(), |ui| {
                ui.heading(service.service_type.name());
                ui.separator();

                egui::Grid::new("service_overview")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Location:");
                        ui.label(format!("({}, {})", service.grid_x, service.grid_y));
                        ui.end_row();
                        ui.label("Coverage:");
                        ui.label(format!(
                            "{:.0} px ({:.0} cells)",
                            service.radius,
                            service.radius / CELL_SIZE
                        ));
                        ui.end_row();
                        ui.label("Monthly cost:");
                        ui.label(format!("${:.0}", monthly_cost));
                        ui.end_row();
                        ui.label("Land value:");
                        ui.label(format!("{}/255", lv));
                        ui.end_row();
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    power_water_labels(ui, cell.has_power, cell.has_water);
                });
            });
        return;
    }

    // Utility building inspection
    if let Ok(utility) = utility_sources.get(entity) {
        let cell = grid.get(utility.grid_x, utility.grid_y);

        egui::Window::new("Building Inspector")
            .default_width(280.0)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
            .show(contexts.ctx_mut(), |ui| {
                ui.heading(utility.utility_type.name());
                ui.separator();

                egui::Grid::new("utility_overview")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Type:");
                        ui.label(if utility.utility_type.is_power() {
                            "Power Generation"
                        } else {
                            "Water Supply"
                        });
                        ui.end_row();
                        ui.label("Location:");
                        ui.label(format!("({}, {})", utility.grid_x, utility.grid_y));
                        ui.end_row();
                        ui.label("Range:");
                        ui.label(format!("{} cells", utility.range));
                        ui.end_row();
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    power_water_labels(ui, cell.has_power, cell.has_water);
                });
            });
    }
}

fn education_abbrev(education: u8) -> &'static str {
    match education {
        0 => "-",
        1 => "Elem",
        2 => "HS",
        3 => "Uni",
        _ => "Adv",
    }
}

fn state_name(state: CitizenState) -> &'static str {
    match state {
        CitizenState::AtHome => "Home",
        CitizenState::CommutingToWork => "To Work",
        CitizenState::Working => "Working",
        CitizenState::CommutingHome => "Going Home",
        CitizenState::CommutingToShop => "To Shop",
        CitizenState::Shopping => "Shopping",
        CitizenState::CommutingToLeisure => "To Leisure",
        CitizenState::AtLeisure => "Leisure",
        CitizenState::CommutingToSchool => "To School",
        CitizenState::AtSchool => "At School",
    }
}

fn needs_bar(ui: &mut egui::Ui, label: &str, value: f32) {
    ui.horizontal(|ui| {
        ui.label(format!("{:>7}", label));
        let (rect, _) = ui.allocate_exact_size(egui::vec2(80.0, 10.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
        let pct = (value / 100.0).clamp(0.0, 1.0);
        let color = if pct > 0.6 {
            egui::Color32::from_rgb(50, 200, 50)
        } else if pct > 0.3 {
            egui::Color32::from_rgb(220, 180, 50)
        } else {
            egui::Color32::from_rgb(220, 50, 50)
        };
        let fill_rect =
            egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * pct, rect.height()));
        painter.rect_filled(fill_rect, 2.0, color);
        ui.label(format!("{:.0}%", value));
    });
}

const FIRST_NAMES_M: &[&str] = &[
    "James", "John", "Robert", "Michael", "David", "William", "Richard", "Joseph", "Thomas",
    "Daniel", "Matthew", "Anthony", "Mark", "Steven", "Paul", "Andrew", "Joshua", "Kenneth",
    "Kevin", "Brian", "George", "Timothy", "Ronald", "Edward", "Jason", "Jeffrey", "Ryan", "Jacob",
    "Gary", "Nicholas", "Eric", "Jonathan",
];
const FIRST_NAMES_F: &[&str] = &[
    "Mary",
    "Patricia",
    "Jennifer",
    "Linda",
    "Barbara",
    "Elizabeth",
    "Susan",
    "Jessica",
    "Sarah",
    "Karen",
    "Lisa",
    "Nancy",
    "Betty",
    "Margaret",
    "Sandra",
    "Ashley",
    "Emily",
    "Donna",
    "Michelle",
    "Carol",
    "Amanda",
    "Dorothy",
    "Melissa",
    "Deborah",
    "Stephanie",
    "Rebecca",
    "Sharon",
    "Laura",
    "Cynthia",
    "Kathleen",
    "Amy",
    "Angela",
];
const LAST_NAMES: &[&str] = &[
    "Smith",
    "Johnson",
    "Williams",
    "Brown",
    "Jones",
    "Garcia",
    "Miller",
    "Davis",
    "Rodriguez",
    "Martinez",
    "Hernandez",
    "Lopez",
    "Wilson",
    "Anderson",
    "Thomas",
    "Taylor",
    "Moore",
    "Jackson",
    "Martin",
    "Lee",
    "Thompson",
    "White",
    "Harris",
    "Clark",
    "Lewis",
    "Robinson",
    "Walker",
    "Young",
    "Allen",
    "King",
    "Wright",
    "Hill",
];

fn citizen_name(entity: Entity, gender: Gender) -> String {
    let idx = entity.index() as usize;
    let first = match gender {
        Gender::Male => FIRST_NAMES_M[idx % FIRST_NAMES_M.len()],
        Gender::Female => FIRST_NAMES_F[idx % FIRST_NAMES_F.len()],
    };
    let last = LAST_NAMES[(idx / 31) % LAST_NAMES.len()];
    format!("{} {}", first, last)
}

fn happiness_label(ui: &mut egui::Ui, happiness: f32) {
    let color = if happiness >= 70.0 {
        egui::Color32::from_rgb(50, 200, 50)
    } else if happiness >= 40.0 {
        egui::Color32::from_rgb(220, 180, 50)
    } else {
        egui::Color32::from_rgb(220, 50, 50)
    };
    ui.colored_label(color, format!("{:.0}%", happiness));
}

fn power_water_labels(ui: &mut egui::Ui, has_power: bool, has_water: bool) {
    let power_color = if has_power {
        egui::Color32::from_rgb(50, 200, 50)
    } else {
        egui::Color32::from_rgb(200, 50, 50)
    };
    let water_color = if has_water {
        egui::Color32::from_rgb(50, 130, 220)
    } else {
        egui::Color32::from_rgb(200, 50, 50)
    };
    ui.colored_label(
        power_color,
        if has_power { "Power: ON" } else { "Power: OFF" },
    );
    ui.colored_label(
        water_color,
        if has_water { "Water: ON" } else { "Water: OFF" },
    );
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

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

fn build_minimap_pixels(grid: &WorldGrid, overlay: &OverlayState) -> Vec<egui::Color32> {
    let mut pixels = vec![egui::Color32::BLACK; MINIMAP_SIZE * MINIMAP_SIZE];

    for my in 0..MINIMAP_SIZE {
        for mx in 0..MINIMAP_SIZE {
            let gx = (mx * SAMPLE_STEP).min(GRID_WIDTH - 1);
            let gy_raw = (MINIMAP_SIZE - 1 - my) * SAMPLE_STEP; // Flip Y for screen coords
            let gy = gy_raw.min(GRID_HEIGHT - 1);
            let cell = grid.get(gx, gy);

            let color = match overlay.mode {
                OverlayMode::Power if cell.cell_type != CellType::Water => {
                    if cell.has_power {
                        egui::Color32::from_rgb(200, 200, 50)
                    } else {
                        egui::Color32::from_rgb(150, 30, 30)
                    }
                }
                OverlayMode::Water if cell.cell_type != CellType::Water => {
                    if cell.has_water {
                        egui::Color32::from_rgb(50, 120, 200)
                    } else {
                        egui::Color32::from_rgb(150, 30, 30)
                    }
                }
                _ => {
                    // Normal colors
                    if cell.building_id.is_some() {
                        if cell.zone.is_residential() {
                            egui::Color32::from_rgb(80, 180, 80)
                        } else if cell.zone.is_commercial() {
                            egui::Color32::from_rgb(60, 100, 200)
                        } else if cell.zone == ZoneType::Industrial {
                            egui::Color32::from_rgb(200, 170, 40)
                        } else if cell.zone == ZoneType::Office {
                            egui::Color32::from_rgb(150, 120, 210)
                        } else if cell.zone.is_mixed_use() {
                            egui::Color32::from_rgb(160, 140, 80)
                        } else {
                            egui::Color32::from_rgb(140, 140, 140)
                        }
                    } else if cell.zone != ZoneType::None {
                        if cell.zone.is_residential() {
                            egui::Color32::from_rgb(60, 120, 60)
                        } else if cell.zone.is_commercial() {
                            egui::Color32::from_rgb(40, 60, 140)
                        } else if cell.zone == ZoneType::Industrial {
                            egui::Color32::from_rgb(140, 120, 30)
                        } else if cell.zone == ZoneType::Office {
                            egui::Color32::from_rgb(100, 80, 160)
                        } else if cell.zone.is_mixed_use() {
                            egui::Color32::from_rgb(120, 100, 50)
                        } else {
                            egui::Color32::from_rgb(80, 80, 80)
                        }
                    } else {
                        match cell.cell_type {
                            CellType::Water => egui::Color32::from_rgb(20, 60, 160),
                            CellType::Road => egui::Color32::from_rgb(80, 80, 80),
                            CellType::Grass => {
                                let g = (80.0 + cell.elevation * 100.0) as u8;
                                egui::Color32::from_rgb(30, g, 25)
                            }
                        }
                    }
                }
            };

            pixels[my * MINIMAP_SIZE + mx] = color;
        }
    }

    pixels
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

pub fn policies_ui(
    mut contexts: EguiContexts,
    mut policies: ResMut<Policies>,
    visible: Res<PoliciesVisible>,
) {
    if !visible.0 {
        return;
    }

    egui::Window::new("Policies")
        .default_open(true)
        .default_width(300.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Press [P] to toggle");
            ui.label(format!(
                "Monthly cost: ${:.0}",
                policies.total_monthly_cost()
            ));
            ui.separator();

            for &policy in Policy::all() {
                let mut active = policies.is_active(policy);
                let cost_str = if policy.monthly_cost() > 0.0 {
                    format!(" (${:.0}/mo)", policy.monthly_cost())
                } else {
                    String::new()
                };
                if ui
                    .checkbox(&mut active, format!("{}{}", policy.name(), cost_str))
                    .changed()
                {
                    policies.toggle(policy);
                }
                ui.label(format!("  {}", policy.description()));
                ui.add_space(4.0);
            }
        });
}

// ---------------------------------------------------------------------------
// Event Journal UI
// ---------------------------------------------------------------------------

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

/// Toggles UI panel visibility when keybinds are pressed.
/// J = Event Journal, C = Charts, A = Advisors, P = Policies, B = Budget.
/// Keys are ignored when egui has keyboard focus (e.g. text input).
pub fn panel_keybinds(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut journal: ResMut<JournalVisible>,
    mut charts: ResMut<ChartsVisible>,
    mut advisor: ResMut<AdvisorVisible>,
    mut policies: ResMut<PoliciesVisible>,
    mut budget_panel: ResMut<BudgetPanelVisible>,
    mut contexts: EguiContexts,
) {
    // Don't toggle panels when a text field or other egui widget wants keyboard input
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyJ) {
        journal.0 = !journal.0;
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        charts.0 = !charts.0;
    }
    if keyboard.just_pressed(KeyCode::KeyA) {
        advisor.0 = !advisor.0;
    }
    if keyboard.just_pressed(KeyCode::KeyP) {
        policies.0 = !policies.0;
    }
    if keyboard.just_pressed(KeyCode::KeyB) {
        budget_panel.0 = !budget_panel.0;
    }
}

/// Keyboard shortcuts for quick save (Ctrl+S), quick load (Ctrl+L), and new game (Ctrl+N).
/// Skipped when egui wants keyboard input (e.g. a text field is focused).
pub fn quick_save_load_keybinds(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut save_events: EventWriter<save::SaveGameEvent>,
    mut load_events: EventWriter<save::LoadGameEvent>,
    mut new_game_events: EventWriter<save::NewGameEvent>,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if !ctrl {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyS) {
        save_events.send(save::SaveGameEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        load_events.send(save::LoadGameEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyN) {
        new_game_events.send(save::NewGameEvent);
    }
}

/// Displays the Event Journal as a collapsible egui window.
/// Shows the last 10 events with day/hour and description.
/// Also shows active effects status.
pub fn event_journal_ui(
    mut contexts: EguiContexts,
    journal: Res<EventJournal>,
    effects: Res<ActiveCityEffects>,
    visible: Res<JournalVisible>,
) {
    if !visible.0 {
        return;
    }

    egui::Window::new("Event Journal")
        .default_open(true)
        .default_width(350.0)
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(8.0, -8.0))
        .show(contexts.ctx_mut(), |ui| {
            // Active effects section
            let has_effects = effects.festival_ticks > 0
                || effects.economic_boom_ticks > 0
                || effects.epidemic_ticks > 0;

            if has_effects {
                ui.heading("Active Effects");
                if effects.festival_ticks > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 215, 0),
                        format!("Festival ({} ticks remaining)", effects.festival_ticks),
                    );
                }
                if effects.economic_boom_ticks > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(50, 200, 50),
                        format!(
                            "Economic Boom ({} ticks remaining)",
                            effects.economic_boom_ticks
                        ),
                    );
                }
                if effects.epidemic_ticks > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 50, 50),
                        format!("Epidemic ({} ticks remaining)", effects.epidemic_ticks),
                    );
                }
                ui.separator();
            }

            // Recent events
            ui.heading("Recent Events");
            ui.small("Press [J] to toggle");
            ui.separator();

            if journal.events.is_empty() {
                ui.label("No events recorded yet.");
            } else {
                // Show last 10 events, most recent first
                let start = journal.events.len().saturating_sub(10);
                let recent = &journal.events[start..];

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for event in recent.iter().rev() {
                            let h = event.hour as u32;
                            let m = ((event.hour - h as f32) * 60.0) as u32;
                            let time_str = format!("Day {} {:02}:{:02}", event.day, h, m);

                            let event_color = event_type_color(&event.event_type);

                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::from_rgb(150, 150, 150), &time_str);
                                ui.colored_label(event_color, &event.description);
                            });
                            ui.add_space(2.0);
                        }
                    });
            }
        });
}

/// Displays the City Advisors as a standalone egui window.
/// Toggle visibility with the 'A' key.
pub fn advisor_window_ui(
    mut contexts: EguiContexts,
    advisor_panel: Res<AdvisorPanel>,
    visible: Res<AdvisorVisible>,
) {
    if !visible.0 {
        return;
    }

    egui::Window::new("City Advisors")
        .default_open(true)
        .default_width(350.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Press [A] to toggle");
            ui.separator();

            let messages = &advisor_panel.messages;
            if messages.is_empty() {
                ui.label("No advisor messages at this time.");
            } else {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for msg in messages {
                            let priority_color = match msg.priority {
                                5 => egui::Color32::from_rgb(220, 50, 50),   // red
                                4 => egui::Color32::from_rgb(230, 150, 30),  // orange
                                3 => egui::Color32::from_rgb(220, 200, 50),  // yellow
                                2 => egui::Color32::from_rgb(50, 130, 220),  // blue
                                _ => egui::Color32::from_rgb(150, 150, 150), // grey
                            };

                            ui.horizontal(|ui| {
                                let (dot_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(10.0, 10.0),
                                    egui::Sense::hover(),
                                );
                                let painter = ui.painter_at(dot_rect);
                                painter.circle_filled(dot_rect.center(), 4.0, priority_color);

                                ui.colored_label(priority_color, msg.advisor_type.name());
                            });

                            ui.label(&msg.message);
                            ui.small(&msg.suggestion);
                            ui.add_space(4.0);
                        }
                    });
            }
        });
}

/// Returns a color associated with a given event type for visual distinction.
fn event_type_color(event_type: &simulation::events::CityEventType) -> egui::Color32 {
    use simulation::events::CityEventType;
    match event_type {
        CityEventType::MilestoneReached(_) => egui::Color32::from_rgb(50, 200, 255),
        CityEventType::BuildingFire(_, _) => egui::Color32::from_rgb(255, 100, 30),
        CityEventType::DisasterStrike(_) => egui::Color32::from_rgb(255, 50, 50),
        CityEventType::NewPolicy(_) => egui::Color32::from_rgb(100, 200, 100),
        CityEventType::BudgetCrisis => egui::Color32::from_rgb(255, 50, 50),
        CityEventType::PopulationBoom => egui::Color32::from_rgb(50, 200, 255),
        CityEventType::Epidemic => egui::Color32::from_rgb(200, 50, 200),
        CityEventType::Festival => egui::Color32::from_rgb(255, 215, 0),
        CityEventType::EconomicBoom => egui::Color32::from_rgb(50, 220, 50),
        CityEventType::ResourceDepleted(_) => egui::Color32::from_rgb(200, 150, 50),
    }
}

// ---------------------------------------------------------------------------
// Budget Breakdown Panel
// ---------------------------------------------------------------------------

/// Displays a detailed budget breakdown window with income and expense lines.
/// Shows per-zone tax income, per-category expenses, and net income.
pub fn budget_panel_ui(
    mut contexts: EguiContexts,
    budget: Res<CityBudget>,
    ext_budget: Res<simulation::budget::ExtendedBudget>,
    visible: Res<BudgetPanelVisible>,
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

    egui::Window::new("Budget Breakdown")
        .default_open(true)
        .default_width(320.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Press [B] to toggle");
            ui.separator();

            // Treasury
            ui.horizontal(|ui| {
                ui.strong("Treasury:");
                ui.label(format!("${:.0}", budget.treasury));
            });
            ui.separator();

            // --- Income section ---
            ui.heading("Income");
            budget_line(ui, "Residential Tax", income.residential_tax);
            budget_line(ui, "Commercial Tax", income.commercial_tax);
            budget_line(ui, "Industrial Tax", income.industrial_tax);
            budget_line(ui, "Office Tax", income.office_tax);
            budget_line(ui, "Trade / Tourism", income.trade_income);
            ui.separator();
            ui.horizontal(|ui| {
                ui.strong("Total Income:");
                ui.colored_label(
                    egui::Color32::from_rgb(80, 200, 80),
                    format!("${:.0}/mo", total_income),
                );
            });
            ui.add_space(4.0);

            // --- Expense section ---
            ui.heading("Expenses");
            budget_line(ui, "Road Maintenance", expenses.road_maintenance);
            budget_line(ui, "Service Costs", expenses.service_costs);
            budget_line(ui, "Policy Costs", expenses.policy_costs);
            budget_line(ui, "Loan Payments", expenses.loan_payments);
            ui.separator();
            ui.horizontal(|ui| {
                ui.strong("Total Expenses:");
                ui.colored_label(
                    egui::Color32::from_rgb(220, 80, 80),
                    format!("${:.0}/mo", total_expenses),
                );
            });
            ui.add_space(8.0);

            // --- Net income ---
            ui.separator();
            let net_color = if net >= 0.0 {
                egui::Color32::from_rgb(80, 220, 80)
            } else {
                egui::Color32::from_rgb(255, 60, 60)
            };
            let net_sign = if net >= 0.0 { "+" } else { "" };
            ui.horizontal(|ui| {
                ui.strong("Net Income:");
                ui.colored_label(net_color, format!("{}{:.0}/mo", net_sign, net));
            });

            // Debt summary (only when loans exist)
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
            }
        });
}

/// Helper to render a single budget line item.
fn budget_line(ui: &mut egui::Ui, label: &str, amount: f64) {
    ui.horizontal(|ui| {
        ui.label(format!("  {label}:"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(format!("${:.0}", amount));
        });
    });
}

/// Groundwater tooltip: shows per-cell groundwater level, quality, extraction
/// rate, and recharge rate when a groundwater overlay is active and the cursor
/// is over a valid cell.
#[allow(clippy::too_many_arguments)]
pub fn groundwater_tooltip_ui(
    mut contexts: EguiContexts,
    overlay: Res<OverlayState>,
    cursor: Res<rendering::input::CursorGridPos>,
    groundwater: Res<simulation::groundwater::GroundwaterGrid>,
    water_quality: Res<simulation::groundwater::WaterQualityGrid>,
    depletion: Res<simulation::groundwater_depletion::GroundwaterDepletionState>,
    services: Query<&ServiceBuilding>,
) {
    // Only show when a groundwater overlay is active
    if overlay.mode != OverlayMode::GroundwaterLevel
        && overlay.mode != OverlayMode::GroundwaterQuality
    {
        return;
    }

    if !cursor.valid {
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;
    if gx >= GRID_WIDTH || gy >= GRID_HEIGHT {
        return;
    }

    let level = groundwater.get(gx, gy);
    let quality = water_quality.get(gx, gy);
    let level_pct = level as f32 / 255.0 * 100.0;
    let quality_pct = quality as f32 / 255.0 * 100.0;

    // Check if there is a well pump at or near this cell
    let mut nearby_well = false;
    for service in &services {
        if service.service_type == simulation::services::ServiceType::WellPump {
            let dx = (service.grid_x as i32 - gx as i32).abs();
            let dy = (service.grid_y as i32 - gy as i32).abs();
            if dx <= 1 && dy <= 1 {
                nearby_well = true;
                break;
            }
        }
    }

    egui::Window::new("Groundwater Info")
        .fixed_pos(egui::pos2(
            contexts.ctx_mut().screen_rect().max.x - 240.0,
            contexts.ctx_mut().screen_rect().max.y - 180.0,
        ))
        .auto_sized()
        .title_bar(true)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Cell ({}, {})", gx, gy));
            ui.separator();

            egui::Grid::new("gw_tooltip_grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Level:");
                    let level_color = if level < 76 {
                        egui::Color32::from_rgb(220, 80, 50)
                    } else if level < 128 {
                        egui::Color32::from_rgb(220, 180, 50)
                    } else {
                        egui::Color32::from_rgb(50, 180, 220)
                    };
                    ui.colored_label(level_color, format!("{}/255 ({:.0}%)", level, level_pct));
                    ui.end_row();

                    ui.label("Quality:");
                    let quality_color = if quality < 50 {
                        egui::Color32::from_rgb(200, 50, 50)
                    } else if quality < 128 {
                        egui::Color32::from_rgb(200, 150, 50)
                    } else {
                        egui::Color32::from_rgb(50, 200, 80)
                    };
                    ui.colored_label(
                        quality_color,
                        format!("{}/255 ({:.0}%)", quality, quality_pct),
                    );
                    ui.end_row();

                    ui.label("Extraction:");
                    ui.label(format!("{:.1} units/tick", depletion.extraction_rate));
                    ui.end_row();

                    ui.label("Recharge:");
                    ui.label(format!("{:.1} units/tick", depletion.recharge_rate));
                    ui.end_row();

                    if nearby_well {
                        ui.label("Well:");
                        ui.colored_label(
                            egui::Color32::from_rgb(100, 200, 255),
                            "Well pump nearby",
                        );
                        ui.end_row();
                    }

                    if level < 76 {
                        ui.label("Warning:");
                        ui.colored_label(egui::Color32::from_rgb(255, 100, 50), "Depletion risk!");
                        ui.end_row();
                    }
                });
        });
}
