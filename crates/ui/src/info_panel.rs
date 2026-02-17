use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::buildings::Building;
use simulation::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    Personality, WorkLocation,
};
use simulation::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use simulation::economy::CityBudget;
use simulation::grid::{CellType, WorldGrid, ZoneType};
use simulation::land_value::LandValueGrid;
use simulation::policies::{Policies, Policy};
use simulation::pollution::PollutionGrid;
use simulation::services::ServiceBuilding;
use simulation::stats::CityStats;
use simulation::utilities::UtilitySource;
use simulation::wealth::WealthTier;
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
) {
    // Update minimap every 2 seconds (or first frame)
    let needs_update = minimap_cache.texture_handle.is_none() || minimap_cache.dirty_timer <= 0.0;

    egui::SidePanel::right("info_panel")
        .default_width(200.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("City Stats");
            ui.separator();

            ui.label(format!("Population: {}", format_pop(stats.population)));
            ui.label(format!("Happiness: {:.0}%", stats.average_happiness));
            ui.label(format!("Roads: {} cells", stats.road_cells));

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

            ui.separator();
            ui.heading("Service Coverage");
            // Coverage bars (placeholder values derived from grid)
            let (power_cov, water_cov) = compute_utility_coverage(&grid);
            coverage_bar(ui, "Power", power_cov, egui::Color32::from_rgb(220, 200, 50));
            coverage_bar(ui, "Water", water_cov, egui::Color32::from_rgb(50, 130, 220));
            // Education / fire / police / health computed from ServiceBuilding entities
            coverage_bar(ui, "Education", compute_service_coverage(&services, &grid, "edu"), egui::Color32::from_rgb(100, 180, 220));
            coverage_bar(ui, "Fire", compute_service_coverage(&services, &grid, "fire"), egui::Color32::from_rgb(220, 80, 50));
            coverage_bar(ui, "Police", compute_service_coverage(&services, &grid, "police"), egui::Color32::from_rgb(50, 80, 200));
            coverage_bar(ui, "Health", compute_service_coverage(&services, &grid, "health"), egui::Color32::from_rgb(220, 50, 120));
            coverage_bar(ui, "Telecom", compute_service_coverage(&services, &grid, "telecom"), egui::Color32::from_rgb(150, 200, 80));

            ui.separator();
            ui.heading("Mini-map");

            // Overlay info
            let overlay_text = match overlay.mode {
                OverlayMode::None => "P=Pwr O=Wtr T=Trf N=Pol L=LV E=Edu G=Gar",
                OverlayMode::Power => "Power overlay active [P]",
                OverlayMode::Water => "Water overlay active [O]",
                OverlayMode::Traffic => "Traffic overlay active [T]",
                OverlayMode::Pollution => "Pollution overlay active [N]",
                OverlayMode::LandValue => "Land Value overlay active [L]",
                OverlayMode::Education => "Education overlay active [E]",
                OverlayMode::Garbage => "Garbage overlay active [G]",
            };
            ui.small(overlay_text);

            // Render minimap
            if needs_update {
                let pixels = build_minimap_pixels(&grid, &overlay);
                let color_image = egui::ColorImage {
                    size: [MINIMAP_SIZE, MINIMAP_SIZE],
                    pixels,
                };
                let texture = ui.ctx().load_texture(
                    "minimap",
                    color_image,
                    egui::TextureOptions::NEAREST,
                );
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
        ZoneType::ResidentialHigh => "High-Density Residential",
        ZoneType::CommercialLow => "Low-Density Commercial",
        ZoneType::CommercialHigh => "High-Density Commercial",
        ZoneType::Industrial => "Industrial",
        ZoneType::Office => "Office",
    }
}

#[allow(clippy::too_many_arguments)]
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
                egui::Grid::new("building_overview").num_columns(2).show(ui, |ui| {
                    ui.label("Level:");
                    ui.label(format!("{} / {}", building.level, building.zone_type.max_level()));
                    ui.end_row();

                    ui.label("Occupancy:");
                    let occ_color = if occupancy_pct >= 90.0 {
                        egui::Color32::from_rgb(220, 50, 50)
                    } else if occupancy_pct >= 70.0 {
                        egui::Color32::from_rgb(220, 180, 50)
                    } else {
                        egui::Color32::from_rgb(50, 200, 50)
                    };
                    ui.colored_label(occ_color, format!("{} / {} ({:.0}%)", building.occupants, building.capacity, occupancy_pct));
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

                    let mut residents: Vec<(Entity, &CitizenDetails, &CitizenStateComp, Option<&Needs>, Option<&Personality>, Option<&Family>)> = citizens
                        .iter()
                        .filter(|(_, _, home, _, _, _, _, _)| home.building == entity)
                        .map(|(e, details, _, _, state, needs, pers, fam)| (e, details, state, needs, pers, fam))
                        .collect();

                    let count = residents.len();
                    ui.label(format!("{} residents tracked (entity-backed)", count));

                    if !residents.is_empty() {
                        let avg_happiness: f32 = residents.iter().map(|r| r.1.happiness).sum::<f32>() / count as f32;
                        let avg_age: f32 = residents.iter().map(|r| r.1.age as f32).sum::<f32>() / count as f32;
                        let avg_salary: f32 = residents.iter().map(|r| r.1.salary).sum::<f32>() / count as f32;
                        let males = residents.iter().filter(|r| r.1.gender == Gender::Male).count();
                        let children = residents.iter().filter(|r| r.1.life_stage().should_attend_school() || !r.1.life_stage().can_work()).count();

                        egui::Grid::new("res_summary").num_columns(2).show(ui, |ui| {
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
                            let tax: f32 = residents.iter()
                                .map(|r| r.1.salary * budget.tax_rate)
                                .sum();
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
                                    (acc.0 + n.hunger, acc.1 + n.energy, acc.2 + n.social, acc.3 + n.fun, acc.4 + n.comfort)
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
                        egui::Grid::new("edu_breakdown").num_columns(2).show(ui, |ui| {
                            for (i, name) in ["None", "Elementary", "High School", "University"].iter().enumerate() {
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
                        egui::Grid::new("wealth_breakdown").num_columns(2).show(ui, |ui| {
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
                        residents.sort_by(|a, b| b.1.happiness.partial_cmp(&a.1.happiness).unwrap_or(std::cmp::Ordering::Equal));

                        egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
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

                                    for (i, (ent, details, state, needs, _personality, _family)) in residents.iter().enumerate() {
                                        if i >= 50 { break; }
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
                                            ui.colored_label(color, format!("{:.0}%", sat * 100.0));
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
                        let avg_happiness: f32 = workers.iter().map(|w| w.1.happiness).sum::<f32>() / count as f32;
                        let avg_salary: f32 = workers.iter().map(|w| w.1.salary).sum::<f32>() / count as f32;

                        egui::Grid::new("worker_summary").num_columns(2).show(ui, |ui| {
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
                            for (i, name) in ["None", "Elementary", "High School", "University"].iter().enumerate() {
                                ui.label(*name);
                                ui.label(format!("{}", edu_counts[i]));
                                ui.end_row();
                            }
                        });

                        // Individual worker list
                        ui.separator();
                        ui.label("Individual Workers:");
                        workers.sort_by(|a, b| b.1.happiness.partial_cmp(&a.1.happiness).unwrap_or(std::cmp::Ordering::Equal));

                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
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

                                    for (i, (ent, details, _state)) in workers.iter().enumerate() {
                                        if i >= 50 { break; }
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
        let monthly_cost = simulation::services::ServiceBuilding::monthly_maintenance(service.service_type);

        egui::Window::new("Building Inspector")
            .default_width(280.0)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
            .show(contexts.ctx_mut(), |ui| {
                ui.heading(service.service_type.name());
                ui.separator();

                egui::Grid::new("service_overview").num_columns(2).show(ui, |ui| {
                    ui.label("Location:");
                    ui.label(format!("({}, {})", service.grid_x, service.grid_y));
                    ui.end_row();
                    ui.label("Coverage:");
                    ui.label(format!("{:.0} px ({:.0} cells)", service.radius, service.radius / CELL_SIZE));
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

                egui::Grid::new("utility_overview").num_columns(2).show(ui, |ui| {
                    ui.label("Type:");
                    ui.label(if utility.utility_type.is_power() { "Power Generation" } else { "Water Supply" });
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
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(80.0, 10.0),
            egui::Sense::hover(),
        );
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
        let fill_rect = egui::Rect::from_min_size(
            rect.min,
            egui::vec2(rect.width() * pct, rect.height()),
        );
        painter.rect_filled(fill_rect, 2.0, color);
        ui.label(format!("{:.0}%", value));
    });
}

const FIRST_NAMES_M: &[&str] = &[
    "James", "John", "Robert", "Michael", "David", "William", "Richard", "Joseph",
    "Thomas", "Daniel", "Matthew", "Anthony", "Mark", "Steven", "Paul", "Andrew",
    "Joshua", "Kenneth", "Kevin", "Brian", "George", "Timothy", "Ronald", "Edward",
    "Jason", "Jeffrey", "Ryan", "Jacob", "Gary", "Nicholas", "Eric", "Jonathan",
];
const FIRST_NAMES_F: &[&str] = &[
    "Mary", "Patricia", "Jennifer", "Linda", "Barbara", "Elizabeth", "Susan", "Jessica",
    "Sarah", "Karen", "Lisa", "Nancy", "Betty", "Margaret", "Sandra", "Ashley",
    "Emily", "Donna", "Michelle", "Carol", "Amanda", "Dorothy", "Melissa", "Deborah",
    "Stephanie", "Rebecca", "Sharon", "Laura", "Cynthia", "Kathleen", "Amy", "Angela",
];
const LAST_NAMES: &[&str] = &[
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis",
    "Rodriguez", "Martinez", "Hernandez", "Lopez", "Wilson", "Anderson", "Thomas",
    "Taylor", "Moore", "Jackson", "Martin", "Lee", "Thompson", "White", "Harris",
    "Clark", "Lewis", "Robinson", "Walker", "Young", "Allen", "King", "Wright", "Hill",
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
    ui.colored_label(power_color, if has_power { "Power: ON" } else { "Power: OFF" });
    ui.colored_label(water_color, if has_water { "Water: ON" } else { "Water: OFF" });
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

fn compute_service_coverage(services: &Query<&ServiceBuilding>, grid: &WorldGrid, category: &str) -> f32 {
    let mut covered_cells = 0u32;
    let total_zoned = grid.cells.iter().filter(|c| c.zone != ZoneType::None).count() as f32;
    if total_zoned == 0.0 { return 0.0; }

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
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(120.0, 16.0),
            egui::Sense::hover(),
        );
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
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(90.0, 12.0),
            egui::Sense::hover(),
        );
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
) {
    egui::Window::new("Policies")
        .default_open(false)
        .default_width(300.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Monthly cost: ${:.0}", policies.total_monthly_cost()));
            ui.separator();

            for &policy in Policy::all() {
                let mut active = policies.is_active(policy);
                let cost_str = if policy.monthly_cost() > 0.0 {
                    format!(" (${:.0}/mo)", policy.monthly_cost())
                } else {
                    String::new()
                };
                if ui.checkbox(&mut active, format!("{}{}", policy.name(), cost_str)).changed() {
                    policies.toggle(policy);
                }
                ui.label(format!("  {}", policy.description()));
                ui.add_space(4.0);
            }
        });
}
