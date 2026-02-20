//! Tabbed Building Info Panel (UX-005 + UX-061).
//!
//! Organizes the Building Inspector into tabs: Overview, Services, Economy,
//! Residents/Workers, and Environment. The active tab is tracked per session
//! via a Bevy [`Resource`]. Service and utility buildings retain their
//! simpler flat layouts since they have less information.

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
use simulation::noise::NoisePollutionGrid;
use simulation::pollution::PollutionGrid;
use simulation::services::ServiceBuilding;
use simulation::trees::TreeGrid;
use simulation::utilities::UtilitySource;
use simulation::wealth::WealthTier;

use simulation::config::GRID_WIDTH;

use crate::citizen_info::{FollowCitizen, SelectedCitizen};

// =============================================================================
// Tab identifiers
// =============================================================================

/// Identifies a tab in the Building Inspector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BuildingTab {
    /// Type, level, occupancy, happiness at a glance.
    #[default]
    Overview,
    /// Power, water, and nearby service coverage/quality.
    Services,
    /// Land value, rent/salary, property value, taxes.
    Economy,
    /// List of residents or workers (clickable to follow).
    Residents,
    /// Pollution, noise, green space.
    Environment,
}

impl BuildingTab {
    /// Human-readable label for this tab.
    pub fn label(&self) -> &'static str {
        match self {
            BuildingTab::Overview => "Overview",
            BuildingTab::Services => "Services",
            BuildingTab::Economy => "Economy",
            BuildingTab::Residents => "Residents",
            BuildingTab::Environment => "Environment",
        }
    }

    /// All tab variants in display order.
    pub const ALL: [BuildingTab; 5] = [
        BuildingTab::Overview,
        BuildingTab::Services,
        BuildingTab::Economy,
        BuildingTab::Residents,
        BuildingTab::Environment,
    ];
}

// =============================================================================
// Active tab resource
// =============================================================================

/// Tracks which tab is currently active in the Building Inspector.
#[derive(Resource, Debug, Clone, Default)]
pub struct SelectedBuildingTab(pub BuildingTab);

// Keep backward compatibility: SectionStates is now an alias.
// (Nothing outside this file uses it, but this avoids breakage if something
// referenced the type via the re-export.)
/// Legacy alias kept for backward compatibility.
pub type SectionStates = SelectedBuildingTab;

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

fn noise_color(level: u8) -> egui::Color32 {
    if level > 60 {
        egui::Color32::from_rgb(200, 50, 50)
    } else if level > 30 {
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

/// Renders a horizontal tab bar, returning the updated active tab.
fn tab_bar(ui: &mut egui::Ui, active: &mut BuildingTab) {
    ui.horizontal(|ui| {
        for tab in BuildingTab::ALL {
            let is_selected = *active == tab;
            let text = egui::RichText::new(tab.label());
            let text = if is_selected {
                text.strong().color(egui::Color32::from_rgb(220, 220, 255))
            } else {
                text.color(egui::Color32::from_rgb(160, 160, 180))
            };
            if ui.add(egui::Button::new(text).frame(is_selected)).clicked() {
                *active = tab;
            }
        }
    });
    ui.separator();
}

/// Count nearby trees (green space) within a radius around a grid position.
fn count_nearby_trees(tree_grid: &TreeGrid, gx: usize, gy: usize, radius: usize) -> usize {
    let mut count = 0;
    let min_x = gx.saturating_sub(radius);
    let max_x = (gx + radius).min(tree_grid.width.saturating_sub(1));
    let min_y = gy.saturating_sub(radius);
    let max_y = (gy + radius).min(tree_grid.height.saturating_sub(1));
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if tree_grid.has_tree(x, y) {
                count += 1;
            }
        }
    }
    count
}

fn green_space_label(count: usize) -> (&'static str, egui::Color32) {
    if count >= 20 {
        ("Excellent", egui::Color32::from_rgb(50, 200, 50))
    } else if count >= 10 {
        ("Good", egui::Color32::from_rgb(80, 200, 80))
    } else if count >= 4 {
        ("Moderate", egui::Color32::from_rgb(220, 180, 50))
    } else if count >= 1 {
        ("Low", egui::Color32::from_rgb(220, 120, 50))
    } else {
        ("None", egui::Color32::from_rgb(180, 80, 80))
    }
}

// =============================================================================
// Tabbed Building Inspector UI system
// =============================================================================

/// Renders the Building Inspector with a tabbed layout.
///
/// The tab bar at the top allows switching between Overview, Services, Economy,
/// Residents/Workers, and Environment. This system replaces the flat
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
    noise: Res<NoisePollutionGrid>,
    land_value: Res<LandValueGrid>,
    budget: Res<CityBudget>,
    tree_grid: Res<TreeGrid>,
    mut tab_state: ResMut<SelectedBuildingTab>,
    mut selected_citizen: ResMut<SelectedCitizen>,
    mut follow_citizen: ResMut<FollowCitizen>,
) {
    let Some(entity) = selected.0 else {
        return;
    };

    // === Zone building inspection with tabs ===
    if let Ok(building) = buildings.get(entity) {
        let cell = grid.get(building.grid_x, building.grid_y);
        let idx = building.grid_y * GRID_WIDTH + building.grid_x;
        let poll_level = pollution.levels.get(idx).copied().unwrap_or(0);
        let noise_level = noise.levels.get(idx).copied().unwrap_or(0);
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
            .default_width(340.0)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 40.0))
            .show(contexts.ctx_mut(), |ui| {
                // Heading: always visible regardless of tab
                ui.heading(zone_type_label(building.zone_type));
                ui.separator();

                // Tab bar
                tab_bar(ui, &mut tab_state.0);

                match tab_state.0 {
                    // ========================================================
                    // Overview tab: type, level, occupancy, happiness
                    // ========================================================
                    BuildingTab::Overview => {
                        render_overview_tab(
                            ui,
                            building,
                            occupancy_pct,
                            avg_happiness,
                            cell.has_power,
                            cell.has_water,
                        );
                    }

                    // ========================================================
                    // Services tab: power, water, nearby service coverage
                    // ========================================================
                    BuildingTab::Services => {
                        render_services_tab(
                            ui,
                            building,
                            cell.has_power,
                            cell.has_water,
                            &service_buildings,
                        );
                    }

                    // ========================================================
                    // Economy tab: land value, salary, tax, payroll
                    // ========================================================
                    BuildingTab::Economy => {
                        render_economy_tab(ui, entity, building, lv, &citizens, &budget);
                    }

                    // ========================================================
                    // Residents/Workers tab: citizen list
                    // ========================================================
                    BuildingTab::Residents => {
                        if building.zone_type.is_residential() {
                            render_residents_tab(
                                ui,
                                entity,
                                &citizens,
                                &mut selected_citizen,
                                &mut follow_citizen,
                            );
                        } else {
                            render_workers_tab(
                                ui,
                                entity,
                                &citizens,
                                &mut selected_citizen,
                                &mut follow_citizen,
                            );
                        }
                    }

                    // ========================================================
                    // Environment tab: pollution, noise, green space
                    // ========================================================
                    BuildingTab::Environment => {
                        render_environment_tab(
                            ui,
                            building,
                            poll_level,
                            noise_level,
                            lv,
                            &tree_grid,
                        );
                    }
                }
            });
        return;
    }

    // === Service building inspection (simpler, no tabs needed) ===
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
// Tab content: Overview
// =============================================================================

fn render_overview_tab(
    ui: &mut egui::Ui,
    building: &Building,
    occupancy_pct: f32,
    avg_happiness: f32,
    has_power: bool,
    has_water: bool,
) {
    egui::Grid::new("tab_overview_grid")
        .num_columns(2)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            ui.label("Type:");
            ui.label(zone_type_label(building.zone_type));
            ui.end_row();

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

            ui.label("Location:");
            ui.label(format!("({}, {})", building.grid_x, building.grid_y));
            ui.end_row();
        });

    // Quick power/water status
    ui.separator();
    ui.horizontal(|ui| {
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
    });
}

// =============================================================================
// Tab content: Services
// =============================================================================

fn render_services_tab(
    ui: &mut egui::Ui,
    building: &Building,
    has_power: bool,
    has_water: bool,
    service_buildings: &Query<&ServiceBuilding>,
) {
    // Utility connections
    ui.label("Utility Connections:");
    ui.horizontal(|ui| {
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
    });

    ui.separator();
    ui.label("Nearby Service Coverage:");

    // Gather services that cover this building
    let mut covered: Vec<(&str, f32, f32)> = Vec::new(); // (name, distance, radius)
    for service in service_buildings.iter() {
        let dx = (service.grid_x as f32 - building.grid_x as f32) * CELL_SIZE;
        let dy = (service.grid_y as f32 - building.grid_y as f32) * CELL_SIZE;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist <= service.radius {
            let name = service.service_type.name();
            // Avoid duplicates of the same service type
            if !covered.iter().any(|(n, _, _)| *n == name) {
                covered.push((name, dist, service.radius));
            }
        }
    }

    if covered.is_empty() {
        ui.colored_label(
            egui::Color32::from_rgb(160, 160, 160),
            "No services in range",
        );
    } else {
        covered.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        egui::Grid::new("tab_services_grid")
            .num_columns(3)
            .spacing([12.0, 2.0])
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Service");
                ui.strong("Distance");
                ui.strong("Quality");
                ui.end_row();

                for (name, dist, radius) in &covered {
                    let cells_away = (dist / CELL_SIZE).round() as u32;
                    // Quality based on how close to the center of coverage
                    let quality_pct = ((1.0 - dist / radius) * 100.0).clamp(0.0, 100.0);
                    let quality_color = if quality_pct >= 60.0 {
                        egui::Color32::from_rgb(50, 200, 50)
                    } else if quality_pct >= 30.0 {
                        egui::Color32::from_rgb(220, 180, 50)
                    } else {
                        egui::Color32::from_rgb(220, 120, 50)
                    };

                    ui.colored_label(egui::Color32::from_rgb(80, 200, 140), *name);
                    ui.colored_label(
                        egui::Color32::from_rgb(160, 160, 160),
                        format!("{} cells", cells_away),
                    );
                    ui.colored_label(quality_color, format!("{:.0}%", quality_pct));
                    ui.end_row();
                }
            });
    }
}

// =============================================================================
// Tab content: Economy
// =============================================================================

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn render_economy_tab(
    ui: &mut egui::Ui,
    building_entity: Entity,
    building: &Building,
    land_val: u8,
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
    budget: &CityBudget,
) {
    egui::Grid::new("tab_econ_overview")
        .num_columns(2)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            ui.label("Land Value:");
            let lv_color = if land_val >= 180 {
                egui::Color32::from_rgb(50, 200, 50)
            } else if land_val >= 80 {
                egui::Color32::from_rgb(220, 180, 50)
            } else {
                egui::Color32::from_rgb(180, 100, 60)
            };
            ui.colored_label(lv_color, format!("{}/255", land_val));
            ui.end_row();

            // Estimated property value (land value * capacity factor)
            let property_value = land_val as f32 * building.capacity as f32 * 10.0;
            ui.label("Property Value:");
            ui.label(format!("${:.0}", property_value));
            ui.end_row();

            // Tax rate
            ui.label("Tax Rate:");
            ui.label(format!("{:.1}%", budget.tax_rate * 100.0));
            ui.end_row();
        });

    ui.separator();

    if building.zone_type.is_residential() {
        let residents: Vec<&CitizenDetails> = citizens
            .iter()
            .filter(|(_, _, home, _, _, _, _, _)| home.building == building_entity)
            .map(|(_, details, _, _, _, _, _, _)| details)
            .collect();

        if !residents.is_empty() {
            let count = residents.len() as f32;
            let avg_salary: f32 = residents.iter().map(|r| r.salary).sum::<f32>() / count;
            let tax_revenue: f32 = residents.iter().map(|r| r.salary * budget.tax_rate).sum();
            let avg_rent = avg_salary * 0.3; // estimate 30% of income

            egui::Grid::new("tab_econ_res")
                .num_columns(2)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Avg Salary:");
                    ui.label(format!("${:.0}/mo", avg_salary));
                    ui.end_row();
                    ui.label("Est. Avg Rent:");
                    ui.label(format!("${:.0}/mo", avg_rent));
                    ui.end_row();
                    ui.label("Tax Revenue:");
                    ui.label(format!("${:.0}/mo", tax_revenue));
                    ui.end_row();
                });

            // Income distribution
            ui.separator();
            ui.label("Income Distribution:");
            let mut wealth_counts = [0u32; 3];
            for r in &residents {
                match WealthTier::from_education(r.education) {
                    WealthTier::LowIncome => wealth_counts[0] += 1,
                    WealthTier::MiddleIncome => wealth_counts[1] += 1,
                    WealthTier::HighIncome => wealth_counts[2] += 1,
                }
            }
            egui::Grid::new("tab_wealth")
                .num_columns(2)
                .spacing([16.0, 2.0])
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
        } else {
            ui.label("No residents yet");
        }
    } else {
        // Commercial/Industrial/Office
        let workers: Vec<&CitizenDetails> = citizens
            .iter()
            .filter(|(_, _, _, work, _, _, _, _)| {
                work.map(|w| w.building == building_entity).unwrap_or(false)
            })
            .map(|(_, details, _, _, _, _, _, _)| details)
            .collect();

        if !workers.is_empty() {
            let count = workers.len() as f32;
            let avg_salary: f32 = workers.iter().map(|w| w.salary).sum::<f32>() / count;

            egui::Grid::new("tab_econ_work")
                .num_columns(2)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Avg Salary:");
                    ui.label(format!("${:.0}/mo", avg_salary));
                    ui.end_row();
                    ui.label("Payroll:");
                    ui.label(format!("${:.0}/mo", avg_salary * count));
                    ui.end_row();
                });

            // Workforce education
            ui.separator();
            ui.label("Workforce Education:");
            let mut edu_counts = [0u32; 4];
            for w in &workers {
                let idx = (w.education as usize).min(3);
                edu_counts[idx] += 1;
            }
            egui::Grid::new("tab_worker_edu")
                .num_columns(2)
                .spacing([16.0, 2.0])
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
        } else {
            ui.label("No workers yet");
        }
    }
}

// =============================================================================
// Tab content: Residents (clickable)
// =============================================================================

#[allow(clippy::type_complexity)]
fn render_residents_tab(
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
    selected_citizen: &mut SelectedCitizen,
    follow_citizen: &mut FollowCitizen,
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

    egui::Grid::new("tab_res_summary")
        .num_columns(2)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            ui.label("Avg Happiness:");
            ui.colored_label(
                happiness_color(avg_happiness),
                format!("{:.0}%", avg_happiness),
            );
            ui.end_row();
            ui.label("Avg Age:");
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
    ui.label("Education Breakdown:");
    let mut edu_counts = [0u32; 4];
    for r in &residents {
        let idx = (r.1.education as usize).min(3);
        edu_counts[idx] += 1;
    }
    egui::Grid::new("tab_edu_breakdown")
        .num_columns(2)
        .spacing([16.0, 2.0])
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

    // Individual resident list (scrollable, clickable)
    ui.separator();
    ui.label("Individual Residents (click to follow):");
    residents.sort_by(|a, b| {
        b.1.happiness
            .partial_cmp(&a.1.happiness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    egui::ScrollArea::vertical()
        .max_height(280.0)
        .show(ui, |ui| {
            egui::Grid::new("tab_residents_list")
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
                        let name = gen_citizen_name(*ent, details.gender);
                        // Clickable name to select/follow citizen
                        if ui.small_button(&name).clicked() {
                            selected_citizen.0 = Some(*ent);
                            follow_citizen.0 = Some(*ent);
                        }
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
// Tab content: Workers (clickable)
// =============================================================================

#[allow(clippy::type_complexity)]
fn render_workers_tab(
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
    selected_citizen: &mut SelectedCitizen,
    follow_citizen: &mut FollowCitizen,
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

    egui::Grid::new("tab_worker_summary")
        .num_columns(2)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            ui.label("Avg Happiness:");
            ui.colored_label(
                happiness_color(avg_happiness),
                format!("{:.0}%", avg_happiness),
            );
            ui.end_row();
            ui.label("Avg Salary:");
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
    ui.label("Workforce Education:");
    egui::Grid::new("tab_wrk_edu")
        .num_columns(2)
        .spacing([16.0, 2.0])
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

    // Individual worker list (clickable)
    ui.separator();
    ui.label("Individual Workers (click to follow):");
    workers.sort_by(|a, b| {
        b.1.happiness
            .partial_cmp(&a.1.happiness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            egui::Grid::new("tab_workers_list")
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
                        let name = gen_citizen_name(*ent, details.gender);
                        // Clickable name to select/follow citizen
                        if ui.small_button(&name).clicked() {
                            selected_citizen.0 = Some(*ent);
                            follow_citizen.0 = Some(*ent);
                        }
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
// Tab content: Environment
// =============================================================================

fn render_environment_tab(
    ui: &mut egui::Ui,
    building: &Building,
    poll_level: u8,
    noise_level: u8,
    land_val: u8,
    tree_grid: &TreeGrid,
) {
    let nearby_trees = count_nearby_trees(tree_grid, building.grid_x, building.grid_y, 5);
    let (gs_label, gs_color) = green_space_label(nearby_trees);

    egui::Grid::new("tab_env_grid")
        .num_columns(2)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            ui.label("Pollution:");
            ui.colored_label(pollution_color(poll_level), format!("{}/255", poll_level));
            ui.end_row();

            ui.label("Noise:");
            ui.colored_label(noise_color(noise_level), format!("{}/100", noise_level));
            ui.end_row();

            ui.label("Green Space:");
            ui.colored_label(
                gs_color,
                format!("{} ({} trees nearby)", gs_label, nearby_trees),
            );
            ui.end_row();

            ui.label("Land Value:");
            let lv_color = if land_val >= 180 {
                egui::Color32::from_rgb(50, 200, 50)
            } else if land_val >= 80 {
                egui::Color32::from_rgb(220, 180, 50)
            } else {
                egui::Color32::from_rgb(180, 100, 60)
            };
            ui.colored_label(lv_color, format!("{}/255", land_val));
            ui.end_row();

            ui.label("Location:");
            ui.label(format!("({}, {})", building.grid_x, building.grid_y));
            ui.end_row();
        });

    // Environmental quality summary
    ui.separator();
    ui.label("Environmental Quality:");

    let env_score = {
        // Invert pollution and noise (lower is better), combine with green space
        let poll_score = (255.0 - poll_level as f32) / 255.0 * 40.0;
        let noise_score = (100.0 - noise_level as f32) / 100.0 * 30.0;
        let green_score = (nearby_trees as f32).min(20.0) / 20.0 * 30.0;
        (poll_score + noise_score + green_score).clamp(0.0, 100.0)
    };

    let env_color = if env_score >= 70.0 {
        egui::Color32::from_rgb(50, 200, 50)
    } else if env_score >= 40.0 {
        egui::Color32::from_rgb(220, 180, 50)
    } else {
        egui::Color32::from_rgb(220, 50, 50)
    };

    let env_label = if env_score >= 80.0 {
        "Excellent"
    } else if env_score >= 60.0 {
        "Good"
    } else if env_score >= 40.0 {
        "Fair"
    } else if env_score >= 20.0 {
        "Poor"
    } else {
        "Very Poor"
    };

    ui.colored_label(env_color, format!("{} ({:.0}/100)", env_label, env_score));
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that adds the tabbed Building Inspector panel.
pub struct ProgressiveDisclosurePlugin;

impl Plugin for ProgressiveDisclosurePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedBuildingTab>()
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
    // BuildingTab labels
    // =========================================================================

    #[test]
    fn test_building_tab_labels() {
        assert_eq!(BuildingTab::Overview.label(), "Overview");
        assert_eq!(BuildingTab::Services.label(), "Services");
        assert_eq!(BuildingTab::Economy.label(), "Economy");
        assert_eq!(BuildingTab::Residents.label(), "Residents");
        assert_eq!(BuildingTab::Environment.label(), "Environment");
    }

    #[test]
    fn test_building_tab_all_count() {
        assert_eq!(BuildingTab::ALL.len(), 5);
    }

    #[test]
    fn test_building_tab_default_is_overview() {
        let tab = BuildingTab::default();
        assert_eq!(tab, BuildingTab::Overview);
    }

    // =========================================================================
    // SelectedBuildingTab defaults
    // =========================================================================

    #[test]
    fn test_selected_building_tab_default() {
        let state = SelectedBuildingTab::default();
        assert_eq!(state.0, BuildingTab::Overview);
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

    #[test]
    fn test_noise_color_green() {
        let color = noise_color(10);
        assert_eq!(color, egui::Color32::from_rgb(50, 200, 50));
    }

    #[test]
    fn test_noise_color_yellow() {
        let color = noise_color(45);
        assert_eq!(color, egui::Color32::from_rgb(200, 150, 50));
    }

    #[test]
    fn test_noise_color_red() {
        let color = noise_color(70);
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
    // Green space helpers
    // =========================================================================

    #[test]
    fn test_green_space_label_none() {
        let (label, _) = green_space_label(0);
        assert_eq!(label, "None");
    }

    #[test]
    fn test_green_space_label_low() {
        let (label, _) = green_space_label(2);
        assert_eq!(label, "Low");
    }

    #[test]
    fn test_green_space_label_moderate() {
        let (label, _) = green_space_label(5);
        assert_eq!(label, "Moderate");
    }

    #[test]
    fn test_green_space_label_good() {
        let (label, _) = green_space_label(15);
        assert_eq!(label, "Good");
    }

    #[test]
    fn test_green_space_label_excellent() {
        let (label, _) = green_space_label(25);
        assert_eq!(label, "Excellent");
    }

    #[test]
    fn test_count_nearby_trees_empty_grid() {
        let grid = TreeGrid::default();
        assert_eq!(count_nearby_trees(&grid, 128, 128, 5), 0);
    }

    // =========================================================================
    // Tab cycling (no mutation needed, just equality checks)
    // =========================================================================

    #[test]
    fn test_building_tab_equality() {
        assert_eq!(BuildingTab::Overview, BuildingTab::Overview);
        assert_ne!(BuildingTab::Overview, BuildingTab::Services);
        assert_ne!(BuildingTab::Economy, BuildingTab::Environment);
    }

    #[test]
    fn test_all_tabs_in_order() {
        assert_eq!(BuildingTab::ALL[0], BuildingTab::Overview);
        assert_eq!(BuildingTab::ALL[1], BuildingTab::Services);
        assert_eq!(BuildingTab::ALL[2], BuildingTab::Economy);
        assert_eq!(BuildingTab::ALL[3], BuildingTab::Residents);
        assert_eq!(BuildingTab::ALL[4], BuildingTab::Environment);
    }
}
