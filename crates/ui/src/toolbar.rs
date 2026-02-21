use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::budget::ExtendedBudget;
use simulation::config::CELL_SIZE;
use simulation::economy::CityBudget;
use simulation::services::ServiceBuilding;
use simulation::stats::CityStats;
use simulation::time_of_day::GameClock;
use simulation::utilities::UtilityType;
use simulation::weather::Weather;
use simulation::zones::ZoneDemand;

use rendering::input::{ActiveTool, GridSnap, StatusMessage};
use rendering::overlay::{OverlayMode, OverlayState};
use save::{LoadGameEvent, NewGameEvent, SaveGameEvent};

// ---------------------------------------------------------------------------
// Simulation speed keybinds (Space / 1 / 2 / 3)
// ---------------------------------------------------------------------------

/// Handles keyboard shortcuts for simulation speed control:
/// - Space: toggle pause / unpause
/// - 1: normal speed (1x)
/// - 2: fast speed (2x)
/// - 3: fastest speed (4x)
pub fn speed_keybinds(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut clock: ResMut<GameClock>,
    mut contexts: EguiContexts,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }

    if bindings.toggle_pause.just_pressed(&keyboard) {
        clock.paused = !clock.paused;
    }
    if bindings.speed_normal.just_pressed(&keyboard) {
        clock.speed = 1.0;
        clock.paused = false;
    }
    if bindings.speed_fast.just_pressed(&keyboard) {
        clock.speed = 2.0;
        clock.paused = false;
    }
    if bindings.speed_fastest.just_pressed(&keyboard) {
        clock.speed = 4.0;
        clock.paused = false;
    }
}

// ---------------------------------------------------------------------------
// Resource: which category popup is open
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct OpenCategory(pub Option<usize>);

/// Cached toolbar catalog built once at startup — avoids per-frame allocation.
#[derive(Resource)]
pub struct ToolCatalog {
    categories: Vec<ToolCategory>,
}

impl Default for ToolCatalog {
    fn default() -> Self {
        Self {
            categories: build_categories(),
        }
    }
}

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

// ---------------------------------------------------------------------------
// Tool tooltip builder
// ---------------------------------------------------------------------------

/// Returns a brief description for each tool.
fn tool_description(item: &ToolItem) -> Option<&'static str> {
    let tool = item.tool.as_ref()?;
    Some(match tool {
        // Roads
        ActiveTool::Road => "Two-lane local road for neighborhood traffic",
        ActiveTool::RoadAvenue => "Four-lane avenue for medium traffic",
        ActiveTool::RoadBoulevard => "Wide boulevard with high capacity",
        ActiveTool::RoadHighway => "High-speed limited-access highway",
        ActiveTool::RoadOneWay => "One-way road for directional traffic flow",
        ActiveTool::RoadPath => "Pedestrian and bicycle path",
        // Zones
        ActiveTool::ZoneResidentialLow => "Low-density houses and small apartments",
        ActiveTool::ZoneResidentialMedium => "Townhouses, duplexes, and small apartments",
        ActiveTool::ZoneResidentialHigh => "Apartment blocks and residential towers",
        ActiveTool::ZoneCommercialLow => "Shops and small retail stores",
        ActiveTool::ZoneCommercialHigh => "Malls and department stores",
        ActiveTool::ZoneIndustrial => "Factories and warehouses",
        ActiveTool::ZoneOffice => "Office buildings and business parks",
        ActiveTool::ZoneMixedUse => "Combined commercial ground floor and residential above",
        // Utilities
        ActiveTool::PlacePowerPlant => "Coal-fired power plant providing electricity",
        ActiveTool::PlaceSolarFarm => "Renewable solar energy generation",
        ActiveTool::PlaceWindTurbine => "Wind-powered electricity generation",
        ActiveTool::PlaceNuclearPlant => "High-output nuclear power generation",
        ActiveTool::PlaceGeothermal => "Geothermal energy from underground heat",
        ActiveTool::PlaceWaterTower => "Stores and distributes water to nearby areas",
        ActiveTool::PlaceSewagePlant => "Processes wastewater from the city",
        ActiveTool::PlacePumpingStation => "Boosts water pressure in the network",
        ActiveTool::PlaceWaterTreatment => "Purifies water for city consumption",
        // Emergency
        ActiveTool::PlaceFireHouse => "Small fire response station",
        ActiveTool::PlaceFireStation => "Standard fire protection and response",
        ActiveTool::PlaceFireHQ => "Regional fire department headquarters",
        ActiveTool::PlacePoliceKiosk => "Small police outpost for local patrol",
        ActiveTool::PlacePoliceStation => "Standard law enforcement station",
        ActiveTool::PlacePoliceHQ => "Regional police headquarters",
        ActiveTool::PlacePrison => "Detains criminals, city-wide effect",
        ActiveTool::PlaceMedicalClinic => "Basic healthcare for minor ailments",
        ActiveTool::PlaceHospital => "Full-service medical facility",
        ActiveTool::PlaceMedicalCenter => "Advanced medical campus with specialty care",
        // Education
        ActiveTool::PlaceKindergarten => "Early childhood education center",
        ActiveTool::PlaceElementarySchool => "Primary education for children",
        ActiveTool::PlaceHighSchool => "Secondary education for teenagers",
        ActiveTool::PlaceUniversity => "Higher education and research institution",
        ActiveTool::PlaceLibrary => "Public library for education and culture",
        // Parks
        ActiveTool::PlaceSmallPark => "Small green space for nearby residents",
        ActiveTool::PlaceLargePark => "Large recreational park area",
        ActiveTool::PlacePlayground => "Play area for children",
        ActiveTool::PlacePlaza => "Public gathering space and marketplace",
        ActiveTool::PlaceSportsField => "Outdoor sports and recreation facility",
        ActiveTool::PlaceStadium => "Large venue for sports events",
        // Landmarks
        ActiveTool::PlaceCityHall => "Seat of city government, boosts happiness",
        ActiveTool::PlaceMuseum => "Cultural institution attracting tourists",
        ActiveTool::PlaceCathedral => "Historic religious landmark",
        ActiveTool::PlaceTVStation => "Broadcasting station for city media",
        // Sanitation
        ActiveTool::PlaceLandfill => "Basic waste disposal site",
        ActiveTool::PlaceRecyclingCenter => "Sorts and recycles waste materials",
        ActiveTool::PlaceIncinerator => "Burns waste, generates some energy",
        ActiveTool::PlaceTransferStation => "Collects waste for transport to landfill",
        ActiveTool::PlaceCemetery => "Burial ground for deceased citizens",
        ActiveTool::PlaceCrematorium => "Cremation facility for deceased citizens",
        // Transport
        ActiveTool::PlaceBusDepot => "Public bus service hub",
        ActiveTool::PlaceTrainStation => "Rail transit station for commuters",
        ActiveTool::PlaceSubwayStation => "Underground rapid transit station",
        ActiveTool::PlaceTramDepot => "Streetcar/tram service depot",
        ActiveTool::PlaceFerryPier => "Water transit terminal",
        ActiveTool::PlaceSmallAirstrip => "Basic aviation facility",
        ActiveTool::PlaceRegionalAirport => "Domestic flight hub",
        ActiveTool::PlaceInternationalAirport => "Major international aviation hub",
        // Telecom
        ActiveTool::PlaceCellTower => "Mobile network coverage tower",
        ActiveTool::PlaceDataCenter => "Internet and data processing facility",
        // Environment
        ActiveTool::TreePlant => "Plant a tree to improve air quality",
        ActiveTool::TreeRemove => "Remove an existing tree",
        // Terrain
        ActiveTool::TerrainRaise => "Raise terrain elevation",
        ActiveTool::TerrainLower => "Lower terrain elevation",
        ActiveTool::TerrainLevel => "Flatten terrain to uniform height",
        ActiveTool::TerrainWater => "Create water body on terrain",
        // Tools
        ActiveTool::Bulldoze => "Demolish buildings and roads",
        ActiveTool::Inspect => "View detailed cell information",
        // Districts
        ActiveTool::DistrictPaint(_) => "Paint cells to assign to this district",
        ActiveTool::DistrictErase => "Remove district assignment from cells",
    })
}

/// Returns the monthly maintenance cost for a utility type.
fn utility_maintenance(ut: UtilityType) -> f64 {
    match ut {
        UtilityType::PowerPlant => 30.0,
        UtilityType::SolarFarm => 15.0,
        UtilityType::WindTurbine => 10.0,
        UtilityType::NuclearPlant => 80.0,
        UtilityType::Geothermal => 40.0,
        UtilityType::WaterTower => 10.0,
        UtilityType::SewagePlant => 15.0,
        UtilityType::PumpingStation => 8.0,
        UtilityType::WaterTreatment => 25.0,
    }
}

/// Returns the coverage range (in grid cells) for a utility type.
fn utility_range(ut: UtilityType) -> u32 {
    match ut {
        UtilityType::PowerPlant => 30,
        UtilityType::SolarFarm => 25,
        UtilityType::WindTurbine => 20,
        UtilityType::WaterTower => 25,
        UtilityType::SewagePlant => 20,
        UtilityType::NuclearPlant => 50,
        UtilityType::Geothermal => 35,
        UtilityType::PumpingStation => 15,
        UtilityType::WaterTreatment => 35,
    }
}

/// Returns the power/water output description for a utility type.
fn utility_capacity(ut: UtilityType) -> &'static str {
    match ut {
        UtilityType::PowerPlant => "500 MW",
        UtilityType::SolarFarm => "200 MW",
        UtilityType::WindTurbine => "100 MW",
        UtilityType::NuclearPlant => "2000 MW",
        UtilityType::Geothermal => "800 MW",
        UtilityType::WaterTower => "1000 m\u{00b3}/day",
        UtilityType::SewagePlant => "800 m\u{00b3}/day",
        UtilityType::PumpingStation => "500 m\u{00b3}/day",
        UtilityType::WaterTreatment => "1500 m\u{00b3}/day",
    }
}

/// Maps an ActiveTool to its UtilityType, if applicable.
fn tool_utility_type(tool: &ActiveTool) -> Option<UtilityType> {
    match tool {
        ActiveTool::PlacePowerPlant => Some(UtilityType::PowerPlant),
        ActiveTool::PlaceSolarFarm => Some(UtilityType::SolarFarm),
        ActiveTool::PlaceWindTurbine => Some(UtilityType::WindTurbine),
        ActiveTool::PlaceNuclearPlant => Some(UtilityType::NuclearPlant),
        ActiveTool::PlaceGeothermal => Some(UtilityType::Geothermal),
        ActiveTool::PlaceWaterTower => Some(UtilityType::WaterTower),
        ActiveTool::PlaceSewagePlant => Some(UtilityType::SewagePlant),
        ActiveTool::PlacePumpingStation => Some(UtilityType::PumpingStation),
        ActiveTool::PlaceWaterTreatment => Some(UtilityType::WaterTreatment),
        _ => None,
    }
}

/// Builds rich tooltip UI content for a tool item.
fn overlay_description(mode: OverlayMode) -> &'static str {
    match mode {
        OverlayMode::Power => "Shows electrical grid coverage",
        OverlayMode::Water => "Shows water supply coverage",
        OverlayMode::Traffic => "Shows traffic congestion levels",
        OverlayMode::Pollution => "Shows air pollution levels",
        OverlayMode::LandValue => "Shows property land values",
        OverlayMode::Education => "Shows education coverage levels",
        OverlayMode::Garbage => "Shows waste collection coverage",
        OverlayMode::Noise => "Shows noise pollution levels",
        OverlayMode::WaterPollution => "Shows water contamination levels",
        OverlayMode::GroundwaterLevel => "Shows underground water table depth",
        OverlayMode::GroundwaterQuality => "Shows groundwater purity levels",
        OverlayMode::Wind => "Shows wind speed and direction",
        OverlayMode::None => "",
    }
}

fn show_tool_tooltip(ui: &mut egui::Ui, item: &ToolItem) {
    ui.set_max_width(250.0);

    // Description from tool or overlay
    if let Some(desc) = tool_description(item) {
        ui.label(egui::RichText::new(desc).weak());
        ui.add_space(4.0);
    } else if let Some(ov) = item.overlay {
        let desc = overlay_description(ov);
        if !desc.is_empty() {
            ui.label(egui::RichText::new(desc).weak());
            ui.add_space(4.0);
        }
    }

    // Placement cost
    if let Some(cost) = item.cost {
        ui.label(format!("Cost: ${:.0}", cost));
    }

    // Service buildings: maintenance, coverage, capacity
    if let Some(ref tool) = item.tool {
        if let Some(st) = tool.service_type() {
            let maint = ServiceBuilding::monthly_maintenance(st);
            ui.label(format!("Maintenance: ${:.0}/mo", maint));

            let radius = ServiceBuilding::coverage_radius(st);
            if radius > 0.0 {
                let cells = (radius / CELL_SIZE).round() as u32;
                ui.label(format!("Coverage: {} cells", cells));
            }

            let (fw, fh) = ServiceBuilding::footprint(st);
            if fw > 1 || fh > 1 {
                ui.label(format!("Footprint: {}x{}", fw, fh));
            }
        }

        // Utility buildings: maintenance, range, capacity
        if let Some(ut) = tool_utility_type(tool) {
            let maint = utility_maintenance(ut);
            ui.label(format!("Maintenance: ${:.0}/mo", maint));

            let range = utility_range(ut);
            ui.label(format!("Coverage: {} cells", range));

            let cap = utility_capacity(ut);
            ui.label(format!("Capacity: {}", cap));
        }
    }
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
                    tool: Some(ActiveTool::ZoneResidentialMedium),
                    icon: "RM",
                    name: "Res Medium",
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
                ToolItem {
                    tool: Some(ActiveTool::ZoneMixedUse),
                    icon: "MU",
                    name: "Mixed-Use",
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
                ToolItem {
                    tool: None,
                    icon: "GL",
                    name: "GW Level",
                    cost: None,
                    overlay: Some(OverlayMode::GroundwaterLevel),
                },
                ToolItem {
                    tool: None,
                    icon: "GQ",
                    name: "GW Quality",
                    cost: None,
                    overlay: Some(OverlayMode::GroundwaterQuality),
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
// Speed button with color-coded dot indicator
// ---------------------------------------------------------------------------

/// Scale a `Color32` by a factor (0.0 = black, 1.0 = unchanged).
fn dim_color(c: egui::Color32, factor: f32) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(
        (c.r() as f32 * factor) as u8,
        (c.g() as f32 * factor) as u8,
        (c.b() as f32 * factor) as u8,
        c.a(),
    )
}

/// Renders a speed control button with a colored dot indicator.
/// When `active` the dot is filled and the label uses the accent color;
/// when inactive the dot is a dim outline.
fn speed_button(
    ui: &mut egui::Ui,
    label: &str,
    active: bool,
    color: egui::Color32,
) -> egui::Response {
    let dot_radius = 4.0;
    let desired_size = egui::vec2(
        ui.spacing().interact_size.x + dot_radius * 2.0 + 4.0,
        ui.spacing().interact_size.y,
    );
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        // Highlight background when active
        if active {
            let bg = egui::Color32::from_rgba_premultiplied(
                (color.r() as f32 * 0.18) as u8,
                (color.g() as f32 * 0.18) as u8,
                (color.b() as f32 * 0.18) as u8,
                45,
            );
            painter.rect_filled(rect.shrink(1.0), 4.0, bg);
            painter.rect_stroke(
                rect.shrink(1.0),
                4.0,
                egui::Stroke::new(1.0, dim_color(color, 0.5)),
                egui::StrokeKind::Inside,
            );
        } else if response.hovered() {
            painter.rect_filled(rect.shrink(1.0), 4.0, egui::Color32::from_white_alpha(10));
        }

        // Draw the colored dot
        let dot_center = egui::pos2(rect.left() + dot_radius + 4.0, rect.center().y);
        if active {
            painter.circle_filled(dot_center, dot_radius, color);
        } else {
            painter.circle_stroke(
                dot_center,
                dot_radius,
                egui::Stroke::new(1.0, dim_color(color, 0.4)),
            );
        }

        // Draw the label text
        let text_color = if active {
            color
        } else {
            egui::Color32::from_gray(180)
        };
        let text_pos = egui::pos2(dot_center.x + dot_radius + 4.0, rect.center().y - 6.0);
        painter.text(
            text_pos,
            egui::Align2::LEFT_TOP,
            label,
            egui::FontId::proportional(13.0),
            text_color,
        );
    }

    response
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
    weather: Res<Weather>,
    grid_snap: Res<GridSnap>,
    extended_budget: Res<ExtendedBudget>,
    catalog: Res<ToolCatalog>,
) {
    let categories = &catalog.categories;

    // Set tooltip delay to 300ms for tool tooltips
    contexts
        .ctx_mut()
        .style_mut(|style| style.interaction.tooltip_delay = 0.3);

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

                // Net income indicator
                {
                    let net = budget.monthly_income - budget.monthly_expenses;
                    let (sign, color) = if net >= 0.0 {
                        ("+", egui::Color32::from_rgb(80, 200, 80))
                    } else {
                        ("", egui::Color32::from_rgb(220, 60, 60))
                    };
                    let label_text =
                        egui::RichText::new(format!("{}${:.0}/mo", sign, net)).color(color);
                    let resp = ui.label(label_text);
                    let ib = &extended_budget.income_breakdown;
                    let eb = &extended_budget.expense_breakdown;
                    let total_income = budget.monthly_income;
                    let total_expenses = budget.monthly_expenses;
                    resp.on_hover_ui(|ui| {
                        ui.heading("Monthly Budget");
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("Income: ${:.0}", total_income))
                                .color(egui::Color32::from_rgb(80, 200, 80)),
                        );
                        ui.indent("income_details", |ui| {
                            ui.label(format!("Residential Tax: ${:.0}", ib.residential_tax));
                            ui.label(format!("Commercial Tax: ${:.0}", ib.commercial_tax));
                            ui.label(format!("Industrial Tax: ${:.0}", ib.industrial_tax));
                            ui.label(format!("Office Tax: ${:.0}", ib.office_tax));
                            ui.label(format!("Tourism: ${:.0}", ib.trade_income));
                        });
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("Expenses: ${:.0}", total_expenses))
                                .color(egui::Color32::from_rgb(220, 60, 60)),
                        );
                        ui.indent("expense_details", |ui| {
                            ui.label(format!("Road Maintenance: ${:.0}", eb.road_maintenance));
                            ui.label(format!("Service Costs: ${:.0}", eb.service_costs));
                            ui.label(format!("Policy Costs: ${:.0}", eb.policy_costs));
                            ui.label(format!("Loan Payments: ${:.0}", eb.loan_payments));
                        });
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("Net: {}${:.0}/mo", sign, net))
                                .strong()
                                .color(color),
                        );
                    });
                }

                ui.separator();

                // Day / time / season
                ui.label(format!("{} | {}", clock.formatted(), weather.season.name()));

                // Speed controls with color-coded indicators
                // Colors: red = paused, green = 1x, yellow = 2x, orange = 4x
                let pause_color = egui::Color32::from_rgb(220, 60, 60);
                let speed1_color = egui::Color32::from_rgb(60, 200, 60);
                let speed2_color = egui::Color32::from_rgb(230, 220, 50);
                let speed4_color = egui::Color32::from_rgb(240, 160, 40);

                let pause_active = clock.paused;
                let speed1_active = !clock.paused && clock.speed == 1.0;
                let speed2_active = !clock.paused && clock.speed == 2.0;
                let speed4_active = !clock.paused && clock.speed == 4.0;

                if speed_button(ui, "||", pause_active, pause_color).clicked() {
                    clock.paused = !clock.paused;
                }
                if speed_button(ui, "1x", speed1_active, speed1_color).clicked() {
                    clock.speed = 1.0;
                    clock.paused = false;
                }
                if speed_button(ui, "2x", speed2_active, speed2_color).clicked() {
                    clock.speed = 2.0;
                    clock.paused = false;
                }
                if speed_button(ui, "4x", speed4_active, speed4_color).clicked() {
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

                // Grid snap indicator
                if grid_snap.enabled {
                    ui.separator();
                    ui.label(
                        egui::RichText::new("[GRID SNAP]")
                            .strong()
                            .color(egui::Color32::from_rgb(100, 255, 100)),
                    );
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

    // ---- Category popup: compact horizontal strip just above the bottom bar ----
    if let Some(cat_idx) = open_cat.0 {
        if cat_idx < categories.len() {
            let cat = &categories[cat_idx];
            let bottom_rect = bottom_resp.response.rect;
            let screen_width = contexts.ctx_mut().screen_rect().width();

            let mut should_close = false;

            egui::Area::new(egui::Id::new("category_popup"))
                .fixed_pos(egui::pos2(0.0, bottom_rect.top() - 2.0))
                .pivot(egui::Align2::LEFT_BOTTOM)
                .show(contexts.ctx_mut(), |ui| {
                    egui::Frame::popup(ui.style())
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.set_min_width(screen_width - 16.0);
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;

                                // Category label
                                ui.label(egui::RichText::new(cat.name).strong().size(12.0));
                                ui.separator();

                                // All items in a single horizontal row
                                for item in cat.items.iter() {
                                    let label_text = if let Some(cost) = item.cost {
                                        format!("{} ${:.0}", item.name, cost)
                                    } else {
                                        item.name.to_string()
                                    };

                                    let is_active = match item.tool {
                                        Some(ref t) => *tool == *t,
                                        None => match item.overlay {
                                            Some(ov) => overlay.mode == ov,
                                            None => false,
                                        },
                                    };

                                    let response = ui
                                        .selectable_label(
                                            is_active,
                                            egui::RichText::new(&label_text).size(11.0),
                                        )
                                        .on_hover_ui(|ui| {
                                            show_tool_tooltip(ui, item);
                                        });

                                    if response.clicked() {
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
