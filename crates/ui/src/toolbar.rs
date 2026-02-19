use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::economy::CityBudget;
use simulation::stats::CityStats;
use simulation::time_of_day::GameClock;
use simulation::weather::Weather;
use simulation::zones::ZoneDemand;

use rendering::input::{ActiveTool, StatusMessage};
use rendering::overlay::{OverlayMode, OverlayState};
use save::{LoadGameEvent, NewGameEvent, SaveGameEvent};

use crate::milestones::Milestones;

// ---------------------------------------------------------------------------
// Resource: which category popup is open
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct OpenCategory(pub Option<usize>);

// ---------------------------------------------------------------------------
// Data-driven category / item definitions
// ---------------------------------------------------------------------------

struct ToolItem {
    tool: Option<ActiveTool>,
    icon: &'static str,
    name: &'static str,
    cost: Option<f64>,
    overlay: Option<OverlayMode>,
}

struct ToolCategory {
    #[allow(dead_code)]
    icon: &'static str,
    name: &'static str,
    items: Vec<ToolItem>,
}

fn build_categories() -> Vec<ToolCategory> {
    vec![
        ToolCategory {
            icon: "R",
            name: "Roads",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::Road),
                    icon: "=",
                    name: "Local Road",
                    cost: Some(10.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::RoadAvenue),
                    icon: "==",
                    name: "Avenue",
                    cost: Some(20.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::RoadBoulevard),
                    icon: "===",
                    name: "Boulevard",
                    cost: Some(30.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::RoadHighway),
                    icon: "HW",
                    name: "Highway",
                    cost: Some(40.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::RoadOneWay),
                    icon: "->",
                    name: "One-Way",
                    cost: Some(15.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::RoadPath),
                    icon: "..",
                    name: "Path",
                    cost: Some(5.0),
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "Z",
            name: "Zones",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::ZoneResidentialLow),
                    icon: "RL",
                    name: "Res Low",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::ZoneResidentialHigh),
                    icon: "RH",
                    name: "Res High",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::ZoneCommercialLow),
                    icon: "CL",
                    name: "Com Low",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::ZoneCommercialHigh),
                    icon: "CH",
                    name: "Com High",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::ZoneIndustrial),
                    icon: "I",
                    name: "Industrial",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::ZoneOffice),
                    icon: "O",
                    name: "Office",
                    cost: None,
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "U",
            name: "Utilities",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::PlacePowerPlant),
                    icon: "PP",
                    name: "Power Plant",
                    cost: Some(800.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceSolarFarm),
                    icon: "SF",
                    name: "Solar Farm",
                    cost: Some(1200.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceWindTurbine),
                    icon: "Wi",
                    name: "Wind Turbine",
                    cost: Some(600.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceNuclearPlant),
                    icon: "NP",
                    name: "Nuclear Plant",
                    cost: Some(5000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceGeothermal),
                    icon: "GT",
                    name: "Geothermal",
                    cost: Some(3000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceWaterTower),
                    icon: "WA",
                    name: "Water Tower",
                    cost: Some(600.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceSewagePlant),
                    icon: "SP",
                    name: "Sewage Plant",
                    cost: Some(500.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlacePumpingStation),
                    icon: "PS",
                    name: "Pumping Station",
                    cost: Some(400.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceWaterTreatment),
                    icon: "WP",
                    name: "Water Treatment",
                    cost: Some(1000.0),
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "E",
            name: "Emergency",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::PlaceFireHouse),
                    icon: "Fh",
                    name: "Fire House",
                    cost: Some(200.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceFireStation),
                    icon: "Fi",
                    name: "Fire Station",
                    cost: Some(500.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceFireHQ),
                    icon: "FQ",
                    name: "Fire HQ",
                    cost: Some(1500.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlacePoliceKiosk),
                    icon: "Pk",
                    name: "Police Kiosk",
                    cost: Some(200.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlacePoliceStation),
                    icon: "Po",
                    name: "Police Station",
                    cost: Some(500.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlacePoliceHQ),
                    icon: "PQ",
                    name: "Police HQ",
                    cost: Some(1500.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlacePrison),
                    icon: "Pr",
                    name: "Prison",
                    cost: Some(2000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceMedicalClinic),
                    icon: "Mc",
                    name: "Medical Clinic",
                    cost: Some(300.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceHospital),
                    icon: "Ho",
                    name: "Hospital",
                    cost: Some(1000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceMedicalCenter),
                    icon: "MC",
                    name: "Medical Center",
                    cost: Some(3000.0),
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "S",
            name: "Education",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::PlaceKindergarten),
                    icon: "Kg",
                    name: "Kindergarten",
                    cost: Some(400.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceElementarySchool),
                    icon: "El",
                    name: "Elementary",
                    cost: Some(750.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceHighSchool),
                    icon: "HS",
                    name: "High School",
                    cost: Some(1000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceUniversity),
                    icon: "Un",
                    name: "University",
                    cost: Some(2000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceLibrary),
                    icon: "Li",
                    name: "Library",
                    cost: Some(500.0),
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "P",
            name: "Parks",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::PlaceSmallPark),
                    icon: "SP",
                    name: "Small Park",
                    cost: Some(100.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceLargePark),
                    icon: "LP",
                    name: "Large Park",
                    cost: Some(300.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlacePlayground),
                    icon: "Pg",
                    name: "Playground",
                    cost: Some(200.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlacePlaza),
                    icon: "Pz",
                    name: "Plaza",
                    cost: Some(150.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceSportsField),
                    icon: "Sf",
                    name: "Sports Field",
                    cost: Some(400.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceStadium),
                    icon: "St",
                    name: "Stadium",
                    cost: Some(2000.0),
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "L",
            name: "Landmarks",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::PlaceCityHall),
                    icon: "CH",
                    name: "City Hall",
                    cost: Some(5000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceMuseum),
                    icon: "Mu",
                    name: "Museum",
                    cost: Some(3000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceCathedral),
                    icon: "Ca",
                    name: "Cathedral",
                    cost: Some(4000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceTVStation),
                    icon: "TV",
                    name: "TV Station",
                    cost: Some(3500.0),
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "G",
            name: "Sanitation",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::PlaceLandfill),
                    icon: "Lf",
                    name: "Landfill",
                    cost: Some(300.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceRecyclingCenter),
                    icon: "RC",
                    name: "Recycling Center",
                    cost: Some(800.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceIncinerator),
                    icon: "In",
                    name: "Incinerator",
                    cost: Some(1500.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceTransferStation),
                    icon: "TS",
                    name: "Transfer Station",
                    cost: Some(500.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceCemetery),
                    icon: "Ce",
                    name: "Cemetery",
                    cost: Some(400.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceCrematorium),
                    icon: "Cr",
                    name: "Crematorium",
                    cost: Some(600.0),
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "Tr",
            name: "Transport",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::PlaceBusDepot),
                    icon: "BD",
                    name: "Bus Depot",
                    cost: Some(1000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceTrainStation),
                    icon: "TS",
                    name: "Train Station",
                    cost: Some(2000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceSubwayStation),
                    icon: "SS",
                    name: "Subway",
                    cost: Some(3000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceTramDepot),
                    icon: "TD",
                    name: "Tram Depot",
                    cost: Some(1500.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceFerryPier),
                    icon: "FP",
                    name: "Ferry Pier",
                    cost: Some(800.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceSmallAirstrip),
                    icon: "SA",
                    name: "Small Airstrip",
                    cost: Some(5000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceRegionalAirport),
                    icon: "RA",
                    name: "Regional Airport",
                    cost: Some(10000.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceInternationalAirport),
                    icon: "IA",
                    name: "Int'l Airport",
                    cost: Some(15000.0),
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "TC",
            name: "Telecom",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::PlaceCellTower),
                    icon: "CT",
                    name: "Cell Tower",
                    cost: Some(300.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::PlaceDataCenter),
                    icon: "DC",
                    name: "Data Center",
                    cost: Some(2000.0),
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "V",
            name: "Views",
            items: vec![
                ToolItem {
                    tool: None,
                    icon: "Pw",
                    name: "Power",
                    cost: None,
                    overlay: Some(OverlayMode::Power),
                },
                ToolItem {
                    tool: None,
                    icon: "Wa",
                    name: "Water",
                    cost: None,
                    overlay: Some(OverlayMode::Water),
                },
                ToolItem {
                    tool: None,
                    icon: "Tr",
                    name: "Traffic",
                    cost: None,
                    overlay: Some(OverlayMode::Traffic),
                },
                ToolItem {
                    tool: None,
                    icon: "Po",
                    name: "Pollution",
                    cost: None,
                    overlay: Some(OverlayMode::Pollution),
                },
                ToolItem {
                    tool: None,
                    icon: "LV",
                    name: "Land Value",
                    cost: None,
                    overlay: Some(OverlayMode::LandValue),
                },
                ToolItem {
                    tool: None,
                    icon: "Ed",
                    name: "Education",
                    cost: None,
                    overlay: Some(OverlayMode::Education),
                },
                ToolItem {
                    tool: None,
                    icon: "Gb",
                    name: "Garbage",
                    cost: None,
                    overlay: Some(OverlayMode::Garbage),
                },
                ToolItem {
                    tool: None,
                    icon: "No",
                    name: "Noise",
                    cost: None,
                    overlay: Some(OverlayMode::Noise),
                },
                ToolItem {
                    tool: None,
                    icon: "WP",
                    name: "Water Pollution",
                    cost: None,
                    overlay: Some(OverlayMode::WaterPollution),
                },
            ],
        },
        ToolCategory {
            icon: "Ev",
            name: "Environment",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::TreePlant),
                    icon: "Tp",
                    name: "Plant Tree",
                    cost: Some(50.0),
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::TreeRemove),
                    icon: "Tr",
                    name: "Remove Tree",
                    cost: None,
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "Te",
            name: "Terrain",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::TerrainRaise),
                    icon: "/\\",
                    name: "Raise",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::TerrainLower),
                    icon: "\\/",
                    name: "Lower",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::TerrainLevel),
                    icon: "--",
                    name: "Flatten",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::TerrainWater),
                    icon: "~~",
                    name: "Water",
                    cost: None,
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "D",
            name: "Districts",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::DistrictPaint(0)),
                    icon: "D0",
                    name: "Downtown",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::DistrictPaint(1)),
                    icon: "D1",
                    name: "Suburbs",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::DistrictPaint(2)),
                    icon: "D2",
                    name: "Industrial",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::DistrictPaint(3)),
                    icon: "D3",
                    name: "Waterfront",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::DistrictPaint(4)),
                    icon: "D4",
                    name: "Historic",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::DistrictPaint(5)),
                    icon: "D5",
                    name: "University",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::DistrictPaint(6)),
                    icon: "D6",
                    name: "Arts",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::DistrictPaint(7)),
                    icon: "D7",
                    name: "Tech Park",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::DistrictErase),
                    icon: "DE",
                    name: "Erase District",
                    cost: None,
                    overlay: None,
                },
            ],
        },
        ToolCategory {
            icon: "T",
            name: "Tools",
            items: vec![
                ToolItem {
                    tool: Some(ActiveTool::Bulldoze),
                    icon: "Bd",
                    name: "Bulldoze",
                    cost: None,
                    overlay: None,
                },
                ToolItem {
                    tool: Some(ActiveTool::Inspect),
                    icon: "?",
                    name: "Inspect",
                    cost: None,
                    overlay: None,
                },
            ],
        },
    ]
}

// ---------------------------------------------------------------------------
// Population formatting
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

fn milestone_name(pop: u32) -> &'static str {
    const POPS: &[(u32, &str)] = &[
        (1_000_000, "World Capital"),
        (500_000, "Megalopolis"),
        (250_000, "Megacity"),
        (100_000, "Major Metropolis"),
        (50_000, "Metropolis"),
        (25_000, "Large City"),
        (10_000, "City"),
        (5_000, "Small City"),
        (1_000, "Town"),
        (500, "Hamlet"),
        (100, "Village"),
    ];
    for &(threshold, name) in POPS {
        if pop >= threshold {
            return name;
        }
    }
    "Settlement"
}

// ---------------------------------------------------------------------------
// RCI Demand Bars
// ---------------------------------------------------------------------------

/// Draw a single vertical demand bar. `value` is in 0.0..=1.0.
/// 0.5 is the neutral midpoint: above 0.5 draws upward (demand), below draws
/// downward (surplus, shown in red).
fn demand_bar(ui: &mut egui::Ui, label: &str, value: f32, color: egui::Color32) {
    let bar_width = 8.0;
    let bar_height = 24.0;
    let midpoint = 0.5;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(bar_width + 12.0, bar_height),
        egui::Sense::hover(),
    );

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Bar background
        let bar_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x, rect.min.y),
            egui::vec2(bar_width, bar_height),
        );
        painter.rect_filled(bar_rect, 2.0, egui::Color32::from_gray(50));

        // Midpoint line
        let mid_y = bar_rect.min.y + bar_height * 0.5;
        painter.line_segment(
            [
                egui::pos2(bar_rect.min.x, mid_y),
                egui::pos2(bar_rect.max.x, mid_y),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_gray(120)),
        );

        // Filled portion
        let clamped = value.clamp(0.0, 1.0);
        if clamped > midpoint {
            // Demand: draw upward from midpoint
            let fill_frac = (clamped - midpoint) / midpoint;
            let fill_height = fill_frac * (bar_height * 0.5);
            let fill_rect = egui::Rect::from_min_max(
                egui::pos2(bar_rect.min.x + 1.0, mid_y - fill_height),
                egui::pos2(bar_rect.max.x - 1.0, mid_y),
            );
            painter.rect_filled(fill_rect, 1.0, color);
        } else if clamped < midpoint {
            // Surplus: draw downward from midpoint in red
            let fill_frac = (midpoint - clamped) / midpoint;
            let fill_height = fill_frac * (bar_height * 0.5);
            let fill_rect = egui::Rect::from_min_max(
                egui::pos2(bar_rect.min.x + 1.0, mid_y),
                egui::pos2(bar_rect.max.x - 1.0, mid_y + fill_height),
            );
            painter.rect_filled(fill_rect, 1.0, egui::Color32::from_rgb(220, 60, 50));
        }

        // Label to the right of the bar
        painter.text(
            egui::pos2(bar_rect.max.x + 2.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(10.0),
            color,
        );
    }

    // Tooltip with exact value on hover
    let pct = value * 100.0;
    let status = if value > 0.5 {
        "demand"
    } else if value < 0.5 {
        "surplus"
    } else {
        "balanced"
    };
    response.on_hover_text(format!("{label}: {pct:.0}% ({status})"));
}

fn rci_demand_bars(ui: &mut egui::Ui, demand: &ZoneDemand) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        demand_bar(
            ui,
            "R",
            demand.residential,
            egui::Color32::from_rgb(80, 200, 80),
        );
        demand_bar(
            ui,
            "C",
            demand.commercial,
            egui::Color32::from_rgb(80, 140, 220),
        );
        demand_bar(
            ui,
            "I",
            demand.industrial,
            egui::Color32::from_rgb(220, 200, 60),
        );
    });
}

// ---------------------------------------------------------------------------
// Main toolbar system
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn toolbar_ui(
    mut contexts: EguiContexts,
    mut tool: ResMut<ActiveTool>,
    mut clock: ResMut<GameClock>,
    stats: Res<CityStats>,
    budget: Res<CityBudget>,
    demand: Res<ZoneDemand>,
    mut overlay: ResMut<OverlayState>,
    status: Res<StatusMessage>,
    mut save_events: EventWriter<SaveGameEvent>,
    mut load_events: EventWriter<LoadGameEvent>,
    mut new_game_events: EventWriter<NewGameEvent>,
    mut open_cat: ResMut<OpenCategory>,
    _milestones: Res<Milestones>,
    weather: Res<Weather>,
) {
    let categories = build_categories();

    // ---- Top info bar ----
    egui::TopBottomPanel::top("top_info_bar")
        .exact_height(36.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;

                // Milestone name
                let name = milestone_name(stats.population);
                ui.label(
                    egui::RichText::new(name)
                        .strong()
                        .color(egui::Color32::from_rgb(180, 200, 240)),
                );

                ui.separator();

                // Population
                ui.label(format!("Pop: {}", format_pop(stats.population)));

                ui.separator();

                // RCI Demand Bars
                rci_demand_bars(ui, &demand);

                ui.separator();

                // Money
                ui.label(format!("${:.0}", budget.treasury));

                ui.separator();

                // Day / time / season
                ui.label(format!("{} | {}", clock.formatted(), weather.season.name()));

                // Speed controls
                if ui.selectable_label(clock.paused, "||").clicked() {
                    clock.paused = !clock.paused;
                }
                if ui
                    .selectable_label(!clock.paused && clock.speed == 1.0, "1x")
                    .clicked()
                {
                    clock.speed = 1.0;
                    clock.paused = false;
                }
                if ui
                    .selectable_label(!clock.paused && clock.speed == 2.0, "2x")
                    .clicked()
                {
                    clock.speed = 2.0;
                    clock.paused = false;
                }
                if ui
                    .selectable_label(!clock.paused && clock.speed == 4.0, "4x")
                    .clicked()
                {
                    clock.speed = 4.0;
                    clock.paused = false;
                }

                ui.separator();

                // Happiness
                ui.label(format!("Happy: {:.0}%", stats.average_happiness));

                ui.separator();

                // Save / Load / New Game
                if ui.button("New").clicked() {
                    new_game_events.send(NewGameEvent);
                }
                if ui.button("Save").clicked() {
                    save_events.send(SaveGameEvent);
                }
                if ui.button("Load").clicked() {
                    load_events.send(LoadGameEvent);
                }

                // Current overlay
                if overlay.mode != OverlayMode::None {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("Overlay: {}", overlay.mode.label()))
                            .color(egui::Color32::from_rgb(140, 220, 255)),
                    );
                }

                // Active tool + cost
                if let Some(cost) = tool.cost() {
                    ui.separator();
                    ui.label(format!("{}: ${:.0}", tool.label(), cost));
                } else {
                    ui.separator();
                    ui.label(tool.label());
                }
            });
        });

    // ---- Floating toast for status messages ----
    if status.active() {
        let color = if status.is_error {
            egui::Color32::from_rgb(220, 60, 50)
        } else {
            egui::Color32::from_rgb(60, 200, 80)
        };
        egui::Area::new(egui::Id::new("status_toast"))
            .fixed_pos(egui::pos2(
                contexts.ctx_mut().screen_rect().center().x - 100.0,
                42.0,
            ))
            .show(contexts.ctx_mut(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 220))
                    .show(ui, |ui| {
                        ui.colored_label(color, &status.text);
                    });
            });
    }

    // ---- Bottom toolbar: category buttons with full names ----
    let bottom_resp = egui::TopBottomPanel::bottom("bottom_toolbar")
        .exact_height(36.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                for (idx, cat) in categories.iter().enumerate() {
                    let is_open = open_cat.0 == Some(idx);
                    let btn = ui.selectable_label(is_open, egui::RichText::new(cat.name).strong());
                    if btn.clicked() {
                        if is_open {
                            open_cat.0 = None;
                        } else {
                            open_cat.0 = Some(idx);
                        }
                    }
                }
            });
        });

    // ---- Category popup (shown above bottom bar when a category is open) ----
    if let Some(cat_idx) = open_cat.0 {
        if cat_idx < categories.len() {
            let cat = &categories[cat_idx];
            let bottom_rect = bottom_resp.response.rect;

            let mut should_close = false;

            egui::Area::new(egui::Id::new("category_popup"))
                .fixed_pos(egui::pos2(
                    bottom_rect.left() + 4.0,
                    bottom_rect.top() - 8.0,
                ))
                .pivot(egui::Align2::LEFT_BOTTOM)
                .show(contexts.ctx_mut(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_min_width(200.0);
                        ui.heading(cat.name);
                        ui.separator();

                        // Grid layout: 3 columns
                        egui::Grid::new("cat_items_grid")
                            .num_columns(3)
                            .spacing([8.0, 6.0])
                            .show(ui, |ui| {
                                for (i, item) in cat.items.iter().enumerate() {
                                    let label_text = if let Some(cost) = item.cost {
                                        format!("{} {} ${:.0}", item.icon, item.name, cost)
                                    } else {
                                        format!("{} {}", item.icon, item.name)
                                    };

                                    let is_active = match item.tool {
                                        Some(ref t) => *tool == *t,
                                        None => match item.overlay {
                                            Some(ov) => overlay.mode == ov,
                                            None => false,
                                        },
                                    };

                                    if ui.selectable_label(is_active, &label_text).clicked() {
                                        if let Some(ref t) = item.tool {
                                            *tool = *t;
                                            should_close = true;
                                        } else if let Some(ov) = item.overlay {
                                            overlay.mode = if overlay.mode == ov {
                                                OverlayMode::None
                                            } else {
                                                ov
                                            };
                                        }
                                    }

                                    if (i + 1) % 3 == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    });
                });

            if should_close {
                open_cat.0 = None;
            }
        }
    }
}
