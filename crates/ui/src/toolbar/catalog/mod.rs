use bevy::prelude::*;
use bevy_egui::egui;

use simulation::config::CELL_SIZE;
use simulation::services::ServiceBuilding;
use simulation::utilities::UtilityType;

use rendering::input::ActiveTool;
use rendering::overlay::OverlayMode;

mod infrastructure;
mod services;
pub(super) mod unlock_filter;

// ---------------------------------------------------------------------------
// Resource: which category popup is open
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct OpenCategory(pub Option<usize>);

/// Cached toolbar catalog built once at startup — avoids per-frame allocation.
#[derive(Resource)]
pub struct ToolCatalog {
    pub(super) categories: Vec<ToolCategory>,
}

impl Default for ToolCatalog {
    fn default() -> Self {
        let mut cats = infrastructure::infrastructure_categories();
        cats.extend(services::services_categories());
        Self { categories: cats }
    }
}

// ---------------------------------------------------------------------------
// Data-driven category / item definitions
// ---------------------------------------------------------------------------

pub(super) struct ToolItem {
    pub tool: Option<ActiveTool>,
    #[allow(dead_code)]
    pub icon: &'static str,
    pub name: &'static str,
    pub cost: Option<f64>,
    pub overlay: Option<OverlayMode>,
}

pub(super) struct ToolCategory {
    #[allow(dead_code)]
    pub icon: &'static str,
    pub name: &'static str,
    pub items: Vec<ToolItem>,
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
        ActiveTool::RoadUpgrade => "Upgrade existing road to next tier",
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
        ActiveTool::AutoGrid => "Auto-generate a grid of roads in a rectangular area",
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
        UtilityType::HydroDam => 50.0,
        UtilityType::OilPlant => 35.0,
        UtilityType::GasPlant => 40.0,
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
        UtilityType::HydroDam => 40,
        UtilityType::OilPlant => 30,
        UtilityType::GasPlant => 30,
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
        UtilityType::HydroDam => "200 MW",
        UtilityType::OilPlant => "100 MW",
        UtilityType::GasPlant => "500 MW",
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

/// Returns a description for the given overlay mode.
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

/// Builds rich tooltip UI content for a tool item, optionally showing an
/// unlock hint when the item is locked.
pub(super) fn show_tool_tooltip(
    ui: &mut egui::Ui,
    item: &ToolItem,
    lock_hint: Option<&str>,
) {
    ui.set_max_width(250.0);

    // Show lock message prominently if present
    if let Some(hint) = lock_hint {
        ui.label(
            egui::RichText::new(hint)
                .color(egui::Color32::from_rgb(255, 180, 60))
                .strong(),
        );
        ui.add_space(4.0);
    }

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
