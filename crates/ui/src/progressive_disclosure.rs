//! Progressive Disclosure in Info Panels (UX-061).
//!
//! Provides collapsible sections for the Building Inspector panel so that the
//! most important information (type, level, occupancy, happiness) is always
//! visible, while detailed sections (services, environment, economy,
//! residents/workers) can be expanded or collapsed. The collapsed/expanded
//! state is remembered per session via a Bevy [`Resource`].

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::input::SelectedBuilding;
use simulation::buildings::Building;
use simulation::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    Personality, WorkLocation,
};
use simulation::config::CELL_SIZE;
use simulation::economy::CityBudget;
use simulation::grid::{WorldGrid, ZoneType};
use simulation::land_value::LandValueGrid;
use simulation::pollution::PollutionGrid;
use simulation::services::ServiceBuilding;
use simulation::utilities::UtilitySource;
use simulation::wealth::WealthTier;

use simulation::config::GRID_WIDTH;

// =============================================================================
// Section identifiers
// =============================================================================

/// Identifies a collapsible section in the Building Inspector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InfoSection {
    /// Power and water status.
    Services,
    /// Pollution and land value.
    Environment,
    /// Salary, tax revenue, payroll.
    Economy,
    /// Resident or worker list and breakdowns.
    Residents,
}

impl InfoSection {
    /// Human-readable header for this section.
    pub fn header(&self) -> &'static str {
        match self {
            InfoSection::Services => "Services",
            InfoSection::Environment => "Environment",
            InfoSection::Economy => "Economy",
            InfoSection::Residents => "Residents / Workers",
        }
    }

    /// All section variants in display order.
    pub const ALL: [InfoSection; 4] = [
        InfoSection::Services,
        InfoSection::Environment,
        InfoSection::Economy,
        InfoSection::Residents,
    ];
}

// =============================================================================
// Per-session expanded/collapsed state
// =============================================================================

/// Tracks which collapsible sections are expanded.
/// Defaults to `Services` expanded; others collapsed.
#[derive(Resource, Debug, Clone)]
pub struct SectionStates {
    pub expanded: [bool; 4],
}

impl Default for SectionStates {
    fn default() -> Self {
        Self {
            // Services expanded by default; others collapsed
            expanded: [true, false, false, false],
        }
    }
}

impl SectionStates {
    /// Returns whether the given section is expanded.
    pub fn is_expanded(&self, section: InfoSection) -> bool {
        self.expanded[section as usize]
    }

    /// Toggles the expanded state for the given section.
    pub fn toggle(&mut self, section: InfoSection) {
        let idx = section as usize;
        self.expanded[idx] = !self.expanded[idx];
    }

    /// Sets the expanded state for the given section.
    pub fn set_expanded(&mut self, section: InfoSection, expanded: bool) {
        self.expanded[section as usize] = expanded;
    }
}

// =============================================================================
// Helper functions
// =============================================================================

fn zone_type_label(zone: ZoneType) -> &'static str {
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

fn happiness_color(happiness: f32) -> egui::Color32 {
    if happiness >= 70.0 {
        egui::Color32::from_rgb(50, 200, 50)
    } else if happiness >= 40.0 {
        egui::Color32::from_rgb(220, 180, 50)
    } else {
        egui::Color32::from_rgb(220, 50, 50)
    }
}

fn occupancy_color(pct: f32) -> egui::Color32 {
    if pct >= 90.0 {
        egui::Color32::from_rgb(220, 50, 50)
    } else if pct >= 70.0 {
        egui::Color32::from_rgb(220, 180, 50)
    } else {
        egui::Color32::from_rgb(50, 200, 50)
    }
}

fn pollution_color(level: u8) -> egui::Color32 {
    if level > 50 {
        egui::Color32::from_rgb(200, 50, 50)
    } else if level > 20 {
        egui::Color32::from_rgb(200, 150, 50)
    } else {
        egui::Color32::from_rgb(50, 200, 50)
    }
}

fn education_short(education: u8) -> &'static str {
    match education {
        0 => "-",
        1 => "Elem",
        2 => "HS",
        3 => "Uni",
        _ => "Adv",
    }
}

fn citizen_state_label(state: CitizenState) -> &'static str {
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

fn gen_citizen_name(entity: Entity, gender: Gender) -> String {
    let idx = entity.index() as usize;
    let first = match gender {
        Gender::Male => FIRST_NAMES_M[idx % FIRST_NAMES_M.len()],
        Gender::Female => FIRST_NAMES_F[idx % FIRST_NAMES_F.len()],
    };
    let last = LAST_NAMES[(idx / 31) % LAST_NAMES.len()];
    format!("{} {}", first, last)
}

/// Draws a collapsible section header. Returns `true` if the section body
/// should be rendered (i.e. the section is expanded).
fn section_header(ui: &mut egui::Ui, states: &mut SectionStates, section: InfoSection) -> bool {
    let expanded = states.is_expanded(section);
    let arrow = if expanded { "v" } else { ">" };
    let header_text = format!("{} {}", arrow, section.header());

    let header_color = egui::Color32::from_rgb(160, 180, 220);
    if ui
        .add(
            egui::Button::new(
                egui::RichText::new(header_text)
                    .color(header_color)
                    .strong(),
            )
            .frame(false),
        )
        .clicked()
    {
        states.toggle(section);
        return !expanded; // return the NEW state after toggle
    }
    expanded
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

// =============================================================================
// Progressive Disclosure UI system
// =============================================================================

/// Renders the Building Inspector with progressive disclosure.
///
/// The top section (type, level, occupancy, happiness) is always visible.
/// Detailed sections are collapsible. This system replaces the flat
/// `building_inspection_ui` when the plugin is active.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn progressive_building_inspection_ui(
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
    mut section_states: ResMut<SectionStates>,
) {
    let Some(entity) = selected.0 else {
        return;
    };

    // === Zone building inspection with progressive disclosure ===
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

        // Compute average happiness for this building's occupants
        let avg_happiness = if building.zone_type.is_residential() {
            let residents: Vec<f32> = citizens
                .iter()
                .filter(|(_, _, home, _, _, _, _, _)| home.building == entity)
                .map(|(_, details, _, _, _, _, _, _)| details.happiness)
                .collect();
            if residents.is_empty() {
                0.0
            } else {
                residents.iter().sum::<f32>() / residents.len() as f32
            }
        } else {
            let workers: Vec<f32> = citizens
                .iter()
                .filter(|(_, _, _, work, _, _, _, _)| {
                    work.map(|w| w.building == entity).unwrap_or(false)
                })
                .map(|(_, details, _, _, _, _, _, _)| details.happiness)
                .collect();
            if workers.is_empty() {
                0.0
            } else {
                workers.iter().sum::<f32>() / workers.len() as f32
            }
        };

        egui::Window::new("Building Inspector")
            .default_width(320.0)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
            .show(contexts.ctx_mut(), |ui| {
                // ============================================================
                // Top section: always visible (type, level, occupancy, happiness)
                // ============================================================
                ui.heading(zone_type_label(building.zone_type));
                ui.separator();

                egui::Grid::new("pd_building_top")
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
                        ui.colored_label(
                            occupancy_color(occupancy_pct),
                            format!(
                                "{} / {} ({:.0}%)",
                                building.occupants, building.capacity, occupancy_pct
                            ),
                        );
                        ui.end_row();

                        ui.label("Happiness:");
                        ui.colored_label(
                            happiness_color(avg_happiness),
                            format!("{:.0}%", avg_happiness),
                        );
                        ui.end_row();
                    });

                ui.separator();

                // ============================================================
                // Collapsible: Services (power/water)
                // ============================================================
                if section_header(ui, &mut section_states, InfoSection::Services) {
                    ui.indent("pd_services_body", |ui| {
                        ui.horizontal(|ui| {
                            let power_color = if cell.has_power {
                                egui::Color32::from_rgb(50, 200, 50)
                            } else {
                                egui::Color32::from_rgb(200, 50, 50)
                            };
                            let water_color = if cell.has_water {
                                egui::Color32::from_rgb(50, 130, 220)
                            } else {
                                egui::Color32::from_rgb(200, 50, 50)
                            };
                            ui.colored_label(
                                power_color,
                                if cell.has_power {
                                    "Power: ON"
                                } else {
                                    "Power: OFF"
                                },
                            );
                            ui.colored_label(
                                water_color,
                                if cell.has_water {
                                    "Water: ON"
                                } else {
                                    "Water: OFF"
                                },
                            );
                        });

                        // Nearby service coverage
                        let mut nearby_services: Vec<&str> = Vec::new();
                        for service in service_buildings.iter() {
                            let dx = (service.grid_x as f32 - building.grid_x as f32) * CELL_SIZE;
                            let dy = (service.grid_y as f32 - building.grid_y as f32) * CELL_SIZE;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist <= service.radius {
                                let name = service.service_type.name();
                                if !nearby_services.contains(&name) {
                                    nearby_services.push(name);
                                }
                            }
                        }

                        if !nearby_services.is_empty() {
                            ui.label("Nearby services:");
                            for name in &nearby_services {
                                ui.label(format!("  - {}", name));
                            }
                        } else {
                            ui.label("No nearby services");
                        }
                    });
                    ui.separator();
                }

                // ============================================================
                // Collapsible: Environment (pollution, land value, location)
                // ============================================================
                if section_header(ui, &mut section_states, InfoSection::Environment) {
                    ui.indent("pd_environment_body", |ui| {
                        egui::Grid::new("pd_env_grid")
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.label("Location:");
                                ui.label(format!("({}, {})", building.grid_x, building.grid_y));
                                ui.end_row();

                                ui.label("Land Value:");
                                ui.label(format!("{}/255", lv));
                                ui.end_row();

                                ui.label("Pollution:");
                                ui.colored_label(
                                    pollution_color(poll_level),
                                    format!("{}/255", poll_level),
                                );
                                ui.end_row();
                            });
                    });
                    ui.separator();
                }

                // ============================================================
                // Collapsible: Economy (salary, tax, payroll)
                // ============================================================
                if section_header(ui, &mut section_states, InfoSection::Economy) {
                    ui.indent("pd_economy_body", |ui| {
                        if building.zone_type.is_residential() {
                            let residents: Vec<&CitizenDetails> = citizens
                                .iter()
                                .filter(|(_, _, home, _, _, _, _, _)| home.building == entity)
                                .map(|(_, details, _, _, _, _, _, _)| details)
                                .collect();

                            if !residents.is_empty() {
                                let count = residents.len() as f32;
                                let avg_salary: f32 =
                                    residents.iter().map(|r| r.salary).sum::<f32>() / count;
                                let tax_revenue: f32 =
                                    residents.iter().map(|r| r.salary * budget.tax_rate).sum();

                                egui::Grid::new("pd_econ_res")
                                    .num_columns(2)
                                    .show(ui, |ui| {
                                        ui.label("Avg salary:");
                                        ui.label(format!("${:.0}/mo", avg_salary));
                                        ui.end_row();
                                        ui.label("Tax revenue:");
                                        ui.label(format!("${:.0}/mo", tax_revenue));
                                        ui.end_row();
                                    });

                                // Income distribution
                                ui.label("Income distribution:");
                                let mut wealth_counts = [0u32; 3];
                                for r in &residents {
                                    match WealthTier::from_education(r.education) {
                                        WealthTier::LowIncome => wealth_counts[0] += 1,
                                        WealthTier::MiddleIncome => wealth_counts[1] += 1,
                                        WealthTier::HighIncome => wealth_counts[2] += 1,
                                    }
                                }
                                egui::Grid::new("pd_wealth").num_columns(2).show(ui, |ui| {
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
                            } else {
                                ui.label("No residents yet");
                            }
                        } else {
                            // Commercial/Industrial/Office
                            let workers: Vec<&CitizenDetails> = citizens
                                .iter()
                                .filter(|(_, _, _, work, _, _, _, _)| {
                                    work.map(|w| w.building == entity).unwrap_or(false)
                                })
                                .map(|(_, details, _, _, _, _, _, _)| details)
                                .collect();

                            if !workers.is_empty() {
                                let count = workers.len() as f32;
                                let avg_salary: f32 =
                                    workers.iter().map(|w| w.salary).sum::<f32>() / count;

                                egui::Grid::new("pd_econ_work")
                                    .num_columns(2)
                                    .show(ui, |ui| {
                                        ui.label("Avg salary:");
                                        ui.label(format!("${:.0}/mo", avg_salary));
                                        ui.end_row();
                                        ui.label("Payroll:");
                                        ui.label(format!("${:.0}/mo", avg_salary * count));
                                        ui.end_row();
                                    });
                            } else {
                                ui.label("No workers yet");
                            }
                        }
                    });
                    ui.separator();
                }

                // ============================================================
                // Collapsible: Residents / Workers
                // ============================================================
                if section_header(ui, &mut section_states, InfoSection::Residents) {
                    ui.indent("pd_residents_body", |ui| {
                        if building.zone_type.is_residential() {
                            render_residents_section(ui, entity, &citizens);
                        } else {
                            render_workers_section(ui, entity, &citizens);
                        }
                    });
                }
            });
        return;
    }

    // === Service building inspection (simpler, no progressive disclosure needed) ===
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

                egui::Grid::new("pd_service_overview")
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
                    let power_color = if cell.has_power {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else {
                        egui::Color32::from_rgb(200, 50, 50)
                    };
                    let water_color = if cell.has_water {
                        egui::Color32::from_rgb(50, 130, 220)
                    } else {
                        egui::Color32::from_rgb(200, 50, 50)
                    };
                    ui.colored_label(
                        power_color,
                        if cell.has_power {
                            "Power: ON"
                        } else {
                            "Power: OFF"
                        },
                    );
                    ui.colored_label(
                        water_color,
                        if cell.has_water {
                            "Water: ON"
                        } else {
                            "Water: OFF"
                        },
                    );
                });
            });
        return;
    }

    // === Utility building inspection ===
    if let Ok(utility) = utility_sources.get(entity) {
        let cell = grid.get(utility.grid_x, utility.grid_y);

        egui::Window::new("Building Inspector")
            .default_width(280.0)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
            .show(contexts.ctx_mut(), |ui| {
                ui.heading(utility.utility_type.name());
                ui.separator();

                egui::Grid::new("pd_utility_overview")
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
                    let power_color = if cell.has_power {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else {
                        egui::Color32::from_rgb(200, 50, 50)
                    };
                    let water_color = if cell.has_water {
                        egui::Color32::from_rgb(50, 130, 220)
                    } else {
                        egui::Color32::from_rgb(200, 50, 50)
                    };
                    ui.colored_label(
                        power_color,
                        if cell.has_power {
                            "Power: ON"
                        } else {
                            "Power: OFF"
                        },
                    );
                    ui.colored_label(
                        water_color,
                        if cell.has_water {
                            "Water: ON"
                        } else {
                            "Water: OFF"
                        },
                    );
                });
            });
    }
}

// =============================================================================
// Residents section rendering
// =============================================================================

#[allow(clippy::type_complexity)]
fn render_residents_section(
    ui: &mut egui::Ui,
    building_entity: Entity,
    citizens: &Query<
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
) {
    let mut residents: Vec<(
        Entity,
        &CitizenDetails,
        &CitizenStateComp,
        Option<&Needs>,
        Option<&Personality>,
        Option<&Family>,
    )> = citizens
        .iter()
        .filter(|(_, _, home, _, _, _, _, _)| home.building == building_entity)
        .map(|(e, details, _, _, state, needs, pers, fam)| (e, details, state, needs, pers, fam))
        .collect();

    let count = residents.len();
    ui.label(format!("{} residents tracked", count));

    if residents.is_empty() {
        return;
    }

    let avg_happiness: f32 = residents.iter().map(|r| r.1.happiness).sum::<f32>() / count as f32;
    let avg_age: f32 = residents.iter().map(|r| r.1.age as f32).sum::<f32>() / count as f32;
    let males = residents
        .iter()
        .filter(|r| r.1.gender == Gender::Male)
        .count();
    let children = residents
        .iter()
        .filter(|r| r.1.life_stage().should_attend_school() || !r.1.life_stage().can_work())
        .count();

    egui::Grid::new("pd_res_summary")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Avg happiness:");
            ui.colored_label(
                happiness_color(avg_happiness),
                format!("{:.0}%", avg_happiness),
            );
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
        });

    // Average needs satisfaction
    let needs_count = residents.iter().filter(|r| r.3.is_some()).count();
    if needs_count > 0 {
        ui.separator();
        ui.label("Average Needs:");
        let (avg_h, avg_e, avg_s, avg_f, avg_c) = residents.iter().filter_map(|r| r.3).fold(
            (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32),
            |acc, n| {
                (
                    acc.0 + n.hunger,
                    acc.1 + n.energy,
                    acc.2 + n.social,
                    acc.3 + n.fun,
                    acc.4 + n.comfort,
                )
            },
        );
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
    egui::Grid::new("pd_edu_breakdown")
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
            egui::Grid::new("pd_residents_list")
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

                    for (i, (ent, details, state, needs, _personality, _family)) in
                        residents.iter().enumerate()
                    {
                        if i >= 50 {
                            break;
                        }
                        ui.label(gen_citizen_name(*ent, details.gender));
                        ui.label(format!("{}", details.age));
                        ui.label(education_short(details.education));
                        ui.colored_label(
                            happiness_color(details.happiness),
                            format!("{:.0}%", details.happiness),
                        );
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
                        ui.label(citizen_state_label(state.0));
                        ui.end_row();
                    }
                });

            if count > 50 {
                ui.label(format!("... and {} more", count - 50));
            }
        });
}

// =============================================================================
// Workers section rendering
// =============================================================================

#[allow(clippy::type_complexity)]
fn render_workers_section(
    ui: &mut egui::Ui,
    building_entity: Entity,
    citizens: &Query<
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
) {
    let mut workers: Vec<(Entity, &CitizenDetails, &CitizenStateComp)> = citizens
        .iter()
        .filter(|(_, _, _, work, _, _, _, _)| {
            work.map(|w| w.building == building_entity).unwrap_or(false)
        })
        .map(|(e, details, _, _, state, _, _, _)| (e, details, state))
        .collect();

    let count = workers.len();
    ui.label(format!("{} workers tracked", count));

    if workers.is_empty() {
        return;
    }

    let avg_happiness: f32 = workers.iter().map(|w| w.1.happiness).sum::<f32>() / count as f32;
    let avg_salary: f32 = workers.iter().map(|w| w.1.salary).sum::<f32>() / count as f32;

    egui::Grid::new("pd_worker_summary")
        .num_columns(2)
        .show(ui, |ui| {
            ui.label("Avg happiness:");
            ui.colored_label(
                happiness_color(avg_happiness),
                format!("{:.0}%", avg_happiness),
            );
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
    egui::Grid::new("pd_worker_edu")
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
            egui::Grid::new("pd_workers_list")
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
                        if i >= 50 {
                            break;
                        }
                        ui.label(gen_citizen_name(*ent, details.gender));
                        ui.label(format!("{}", details.age));
                        ui.label(education_short(details.education));
                        ui.colored_label(
                            happiness_color(details.happiness),
                            format!("{:.0}%", details.happiness),
                        );
                        ui.label(format!("${:.0}", details.salary));
                        ui.end_row();
                    }
                });

            if count > 50 {
                ui.label(format!("... and {} more", count - 50));
            }
        });
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that adds progressive disclosure to the Building Inspector.
pub struct ProgressiveDisclosurePlugin;

impl Plugin for ProgressiveDisclosurePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SectionStates>()
            .add_systems(Update, progressive_building_inspection_ui);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // InfoSection header labels
    // =========================================================================

    #[test]
    fn test_info_section_headers() {
        assert_eq!(InfoSection::Services.header(), "Services");
        assert_eq!(InfoSection::Environment.header(), "Environment");
        assert_eq!(InfoSection::Economy.header(), "Economy");
        assert_eq!(InfoSection::Residents.header(), "Residents / Workers");
    }

    #[test]
    fn test_info_section_all_count() {
        assert_eq!(InfoSection::ALL.len(), 4);
    }

    // =========================================================================
    // SectionStates defaults
    // =========================================================================

    #[test]
    fn test_section_states_default() {
        let states = SectionStates::default();
        // Services expanded by default
        assert!(states.is_expanded(InfoSection::Services));
        // Others collapsed by default
        assert!(!states.is_expanded(InfoSection::Environment));
        assert!(!states.is_expanded(InfoSection::Economy));
        assert!(!states.is_expanded(InfoSection::Residents));
    }

    // =========================================================================
    // SectionStates toggle
    // =========================================================================

    #[test]
    fn test_section_states_toggle() {
        let mut states = SectionStates::default();

        // Toggle environment: collapsed -> expanded
        states.toggle(InfoSection::Environment);
        assert!(states.is_expanded(InfoSection::Environment));

        // Toggle environment again: expanded -> collapsed
        states.toggle(InfoSection::Environment);
        assert!(!states.is_expanded(InfoSection::Environment));
    }

    #[test]
    fn test_section_states_toggle_services() {
        let mut states = SectionStates::default();
        assert!(states.is_expanded(InfoSection::Services));

        states.toggle(InfoSection::Services);
        assert!(!states.is_expanded(InfoSection::Services));

        states.toggle(InfoSection::Services);
        assert!(states.is_expanded(InfoSection::Services));
    }

    // =========================================================================
    // SectionStates set_expanded
    // =========================================================================

    #[test]
    fn test_section_states_set_expanded() {
        let mut states = SectionStates::default();

        states.set_expanded(InfoSection::Economy, true);
        assert!(states.is_expanded(InfoSection::Economy));

        states.set_expanded(InfoSection::Economy, false);
        assert!(!states.is_expanded(InfoSection::Economy));
    }

    #[test]
    fn test_section_states_independent() {
        let mut states = SectionStates::default();

        // Expanding one section should not affect others
        states.set_expanded(InfoSection::Residents, true);
        assert!(states.is_expanded(InfoSection::Services)); // still default
        assert!(!states.is_expanded(InfoSection::Environment)); // still default
        assert!(!states.is_expanded(InfoSection::Economy)); // still default
        assert!(states.is_expanded(InfoSection::Residents)); // changed
    }

    // =========================================================================
    // Zone type label
    // =========================================================================

    #[test]
    fn test_zone_type_label() {
        assert_eq!(zone_type_label(ZoneType::None), "Unzoned");
        assert_eq!(
            zone_type_label(ZoneType::ResidentialLow),
            "Low-Density Residential"
        );
        assert_eq!(
            zone_type_label(ZoneType::ResidentialMedium),
            "Medium-Density Residential"
        );
        assert_eq!(
            zone_type_label(ZoneType::ResidentialHigh),
            "High-Density Residential"
        );
        assert_eq!(
            zone_type_label(ZoneType::CommercialLow),
            "Low-Density Commercial"
        );
        assert_eq!(
            zone_type_label(ZoneType::CommercialHigh),
            "High-Density Commercial"
        );
        assert_eq!(zone_type_label(ZoneType::Industrial), "Industrial");
        assert_eq!(zone_type_label(ZoneType::Office), "Office");
        assert_eq!(zone_type_label(ZoneType::MixedUse), "Mixed-Use");
    }

    // =========================================================================
    // Color helpers
    // =========================================================================

    #[test]
    fn test_happiness_color_green_high() {
        let color = happiness_color(80.0);
        assert_eq!(color, egui::Color32::from_rgb(50, 200, 50));
    }

    #[test]
    fn test_happiness_color_yellow_mid() {
        let color = happiness_color(55.0);
        assert_eq!(color, egui::Color32::from_rgb(220, 180, 50));
    }

    #[test]
    fn test_happiness_color_red_low() {
        let color = happiness_color(20.0);
        assert_eq!(color, egui::Color32::from_rgb(220, 50, 50));
    }

    #[test]
    fn test_occupancy_color_green() {
        let color = occupancy_color(50.0);
        assert_eq!(color, egui::Color32::from_rgb(50, 200, 50));
    }

    #[test]
    fn test_occupancy_color_yellow() {
        let color = occupancy_color(75.0);
        assert_eq!(color, egui::Color32::from_rgb(220, 180, 50));
    }

    #[test]
    fn test_occupancy_color_red() {
        let color = occupancy_color(95.0);
        assert_eq!(color, egui::Color32::from_rgb(220, 50, 50));
    }

    #[test]
    fn test_pollution_color_green() {
        let color = pollution_color(10);
        assert_eq!(color, egui::Color32::from_rgb(50, 200, 50));
    }

    #[test]
    fn test_pollution_color_yellow() {
        let color = pollution_color(30);
        assert_eq!(color, egui::Color32::from_rgb(200, 150, 50));
    }

    #[test]
    fn test_pollution_color_red() {
        let color = pollution_color(60);
        assert_eq!(color, egui::Color32::from_rgb(200, 50, 50));
    }

    // =========================================================================
    // Education abbreviation
    // =========================================================================

    #[test]
    fn test_education_short() {
        assert_eq!(education_short(0), "-");
        assert_eq!(education_short(1), "Elem");
        assert_eq!(education_short(2), "HS");
        assert_eq!(education_short(3), "Uni");
        assert_eq!(education_short(4), "Adv");
        assert_eq!(education_short(255), "Adv");
    }

    // =========================================================================
    // Citizen state label
    // =========================================================================

    #[test]
    fn test_citizen_state_labels() {
        assert_eq!(citizen_state_label(CitizenState::AtHome), "Home");
        assert_eq!(
            citizen_state_label(CitizenState::CommutingToWork),
            "To Work"
        );
        assert_eq!(citizen_state_label(CitizenState::Working), "Working");
        assert_eq!(
            citizen_state_label(CitizenState::CommutingHome),
            "Going Home"
        );
        assert_eq!(
            citizen_state_label(CitizenState::CommutingToShop),
            "To Shop"
        );
        assert_eq!(citizen_state_label(CitizenState::Shopping), "Shopping");
        assert_eq!(
            citizen_state_label(CitizenState::CommutingToLeisure),
            "To Leisure"
        );
        assert_eq!(citizen_state_label(CitizenState::AtLeisure), "Leisure");
        assert_eq!(
            citizen_state_label(CitizenState::CommutingToSchool),
            "To School"
        );
        assert_eq!(citizen_state_label(CitizenState::AtSchool), "At School");
    }

    // =========================================================================
    // Citizen name generation (deterministic)
    // =========================================================================

    #[test]
    fn test_citizen_name_deterministic() {
        let entity = Entity::from_raw(42);
        let name1 = gen_citizen_name(entity, Gender::Male);
        let name2 = gen_citizen_name(entity, Gender::Male);
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_citizen_name_gender_difference() {
        let entity = Entity::from_raw(7);
        let male_name = gen_citizen_name(entity, Gender::Male);
        let female_name = gen_citizen_name(entity, Gender::Female);
        // Same last name (based on entity index) but different first name
        assert_ne!(male_name, female_name);
    }

    // =========================================================================
    // Boundary conditions for section indexing
    // =========================================================================

    #[test]
    fn test_all_sections_indexable() {
        let states = SectionStates::default();
        for section in InfoSection::ALL {
            // Should not panic
            let _ = states.is_expanded(section);
        }
    }

    #[test]
    fn test_toggle_all_sections() {
        let mut states = SectionStates::default();
        for section in InfoSection::ALL {
            states.toggle(section);
        }
        // After toggling all: Services was true->false, others were false->true
        assert!(!states.is_expanded(InfoSection::Services));
        assert!(states.is_expanded(InfoSection::Environment));
        assert!(states.is_expanded(InfoSection::Economy));
        assert!(states.is_expanded(InfoSection::Residents));
    }
}
