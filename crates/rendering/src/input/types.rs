use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::grid::{RoadType, ZoneType};
use simulation::services::{self, ServiceType};
use simulation::utilities::UtilityType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource)]
pub enum ActiveTool {
    Road,
    RoadAvenue,
    RoadBoulevard,
    RoadHighway,
    RoadOneWay,
    RoadPath,
    Bulldoze,
    #[default]
    Inspect,
    ZoneResidentialLow,
    ZoneResidentialMedium,
    ZoneResidentialHigh,
    ZoneCommercialLow,
    ZoneCommercialHigh,
    ZoneIndustrial,
    ZoneOffice,
    ZoneMixedUse,
    PlacePowerPlant,
    PlaceSolarFarm,
    PlaceWindTurbine,
    PlaceWaterTower,
    PlaceSewagePlant,
    PlaceNuclearPlant,
    PlaceGeothermal,
    PlacePumpingStation,
    PlaceWaterTreatment,
    PlaceFireStation,
    PlaceFireHouse,
    PlaceFireHQ,
    PlacePoliceStation,
    PlacePoliceKiosk,
    PlacePoliceHQ,
    PlacePrison,
    PlaceHospital,
    PlaceMedicalClinic,
    PlaceMedicalCenter,
    PlaceElementarySchool,
    PlaceHighSchool,
    PlaceUniversity,
    PlaceLibrary,
    PlaceKindergarten,
    PlaceSmallPark,
    PlaceLargePark,
    PlacePlayground,
    PlacePlaza,
    PlaceSportsField,
    PlaceStadium,
    PlaceLandfill,
    PlaceRecyclingCenter,
    PlaceIncinerator,
    PlaceTransferStation,
    PlaceCemetery,
    PlaceCrematorium,
    PlaceCityHall,
    PlaceMuseum,
    PlaceCathedral,
    PlaceTVStation,
    PlaceBusDepot,
    PlaceTrainStation,
    PlaceSubwayStation,
    PlaceTramDepot,
    PlaceFerryPier,
    PlaceSmallAirstrip,
    PlaceRegionalAirport,
    PlaceInternationalAirport,
    PlaceCellTower,
    PlaceDataCenter,
    // Terrain tools
    TerrainRaise,
    TerrainLower,
    TerrainLevel,
    TerrainWater,
    // District tools
    DistrictPaint(usize),
    DistrictErase,
    // Environment tools
    TreePlant,
    TreeRemove,
    // Road upgrade tool
    RoadUpgrade,
    // Auto-grid road placement tool
    AutoGrid,
}

impl ActiveTool {
    /// Returns the cost of the active tool, or None for free tools
    pub fn cost(&self) -> Option<f64> {
        match self {
            ActiveTool::Road => Some(RoadType::Local.cost()),
            ActiveTool::RoadAvenue => Some(RoadType::Avenue.cost()),
            ActiveTool::RoadBoulevard => Some(RoadType::Boulevard.cost()),
            ActiveTool::RoadHighway => Some(RoadType::Highway.cost()),
            ActiveTool::RoadOneWay => Some(RoadType::OneWay.cost()),
            ActiveTool::RoadPath => Some(RoadType::Path.cost()),
            ActiveTool::Bulldoze
            | ActiveTool::Inspect
            | ActiveTool::TerrainRaise
            | ActiveTool::TerrainLower
            | ActiveTool::TerrainLevel
            | ActiveTool::TerrainWater
            | ActiveTool::DistrictPaint(_)
            | ActiveTool::DistrictErase
            | ActiveTool::TreeRemove
            | ActiveTool::RoadUpgrade
            | ActiveTool::AutoGrid => None,
            ActiveTool::TreePlant => Some(simulation::trees::TREE_PLANT_COST),
            ActiveTool::ZoneResidentialLow
            | ActiveTool::ZoneResidentialMedium
            | ActiveTool::ZoneResidentialHigh
            | ActiveTool::ZoneCommercialLow
            | ActiveTool::ZoneCommercialHigh
            | ActiveTool::ZoneIndustrial
            | ActiveTool::ZoneOffice
            | ActiveTool::ZoneMixedUse => None,
            // Utilities
            ActiveTool::PlacePowerPlant => Some(services::utility_cost(UtilityType::PowerPlant)),
            ActiveTool::PlaceSolarFarm => Some(services::utility_cost(UtilityType::SolarFarm)),
            ActiveTool::PlaceWindTurbine => Some(services::utility_cost(UtilityType::WindTurbine)),
            ActiveTool::PlaceWaterTower => Some(services::utility_cost(UtilityType::WaterTower)),
            ActiveTool::PlaceSewagePlant => Some(services::utility_cost(UtilityType::SewagePlant)),
            ActiveTool::PlaceNuclearPlant => {
                Some(services::utility_cost(UtilityType::NuclearPlant))
            }
            ActiveTool::PlaceGeothermal => Some(services::utility_cost(UtilityType::Geothermal)),
            ActiveTool::PlacePumpingStation => {
                Some(services::utility_cost(UtilityType::PumpingStation))
            }
            ActiveTool::PlaceWaterTreatment => {
                Some(services::utility_cost(UtilityType::WaterTreatment))
            }
            // Services
            _ => self.service_type().map(services::ServiceBuilding::cost),
        }
    }

    /// Map tool to ServiceType if applicable
    pub fn service_type(&self) -> Option<ServiceType> {
        match self {
            ActiveTool::PlaceFireStation => Some(ServiceType::FireStation),
            ActiveTool::PlaceFireHouse => Some(ServiceType::FireHouse),
            ActiveTool::PlaceFireHQ => Some(ServiceType::FireHQ),
            ActiveTool::PlacePoliceStation => Some(ServiceType::PoliceStation),
            ActiveTool::PlacePoliceKiosk => Some(ServiceType::PoliceKiosk),
            ActiveTool::PlacePoliceHQ => Some(ServiceType::PoliceHQ),
            ActiveTool::PlacePrison => Some(ServiceType::Prison),
            ActiveTool::PlaceHospital => Some(ServiceType::Hospital),
            ActiveTool::PlaceMedicalClinic => Some(ServiceType::MedicalClinic),
            ActiveTool::PlaceMedicalCenter => Some(ServiceType::MedicalCenter),
            ActiveTool::PlaceElementarySchool => Some(ServiceType::ElementarySchool),
            ActiveTool::PlaceHighSchool => Some(ServiceType::HighSchool),
            ActiveTool::PlaceUniversity => Some(ServiceType::University),
            ActiveTool::PlaceLibrary => Some(ServiceType::Library),
            ActiveTool::PlaceKindergarten => Some(ServiceType::Kindergarten),
            ActiveTool::PlaceSmallPark => Some(ServiceType::SmallPark),
            ActiveTool::PlaceLargePark => Some(ServiceType::LargePark),
            ActiveTool::PlacePlayground => Some(ServiceType::Playground),
            ActiveTool::PlacePlaza => Some(ServiceType::Plaza),
            ActiveTool::PlaceSportsField => Some(ServiceType::SportsField),
            ActiveTool::PlaceStadium => Some(ServiceType::Stadium),
            ActiveTool::PlaceLandfill => Some(ServiceType::Landfill),
            ActiveTool::PlaceRecyclingCenter => Some(ServiceType::RecyclingCenter),
            ActiveTool::PlaceIncinerator => Some(ServiceType::Incinerator),
            ActiveTool::PlaceTransferStation => Some(ServiceType::TransferStation),
            ActiveTool::PlaceCemetery => Some(ServiceType::Cemetery),
            ActiveTool::PlaceCrematorium => Some(ServiceType::Crematorium),
            ActiveTool::PlaceCityHall => Some(ServiceType::CityHall),
            ActiveTool::PlaceMuseum => Some(ServiceType::Museum),
            ActiveTool::PlaceCathedral => Some(ServiceType::Cathedral),
            ActiveTool::PlaceTVStation => Some(ServiceType::TVStation),
            ActiveTool::PlaceBusDepot => Some(ServiceType::BusDepot),
            ActiveTool::PlaceTrainStation => Some(ServiceType::TrainStation),
            ActiveTool::PlaceSubwayStation => Some(ServiceType::SubwayStation),
            ActiveTool::PlaceTramDepot => Some(ServiceType::TramDepot),
            ActiveTool::PlaceFerryPier => Some(ServiceType::FerryPier),
            ActiveTool::PlaceSmallAirstrip => Some(ServiceType::SmallAirstrip),
            ActiveTool::PlaceRegionalAirport => Some(ServiceType::RegionalAirport),
            ActiveTool::PlaceInternationalAirport => Some(ServiceType::InternationalAirport),
            ActiveTool::PlaceCellTower => Some(ServiceType::CellTower),
            ActiveTool::PlaceDataCenter => Some(ServiceType::DataCenter),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ActiveTool::Road => "Local Road",
            ActiveTool::RoadAvenue => "Avenue",
            ActiveTool::RoadBoulevard => "Boulevard",
            ActiveTool::RoadHighway => "Highway",
            ActiveTool::RoadOneWay => "One-Way",
            ActiveTool::RoadPath => "Path",
            ActiveTool::Bulldoze => "Bulldoze",
            ActiveTool::Inspect => "Inspect",
            ActiveTool::ZoneResidentialLow => "Low-Density Residential",
            ActiveTool::ZoneResidentialMedium => "Medium-Density Residential",
            ActiveTool::ZoneResidentialHigh => "High-Density Residential",
            ActiveTool::ZoneCommercialLow => "Low-Density Commercial",
            ActiveTool::ZoneCommercialHigh => "High-Density Commercial",
            ActiveTool::ZoneIndustrial => "Industrial",
            ActiveTool::ZoneOffice => "Office",
            ActiveTool::ZoneMixedUse => "Mixed-Use",
            ActiveTool::PlacePowerPlant => "Power Plant",
            ActiveTool::PlaceSolarFarm => "Solar Farm",
            ActiveTool::PlaceWindTurbine => "Wind Turbine",
            ActiveTool::PlaceWaterTower => "Water Tower",
            ActiveTool::PlaceSewagePlant => "Sewage Plant",
            ActiveTool::PlaceNuclearPlant => "Nuclear Plant",
            ActiveTool::PlaceGeothermal => "Geothermal",
            ActiveTool::PlacePumpingStation => "Pumping Station",
            ActiveTool::PlaceWaterTreatment => "Water Treatment",
            ActiveTool::PlaceFireStation => "Fire Station",
            ActiveTool::PlaceFireHouse => "Fire House",
            ActiveTool::PlaceFireHQ => "Fire HQ",
            ActiveTool::PlacePoliceStation => "Police Station",
            ActiveTool::PlacePoliceKiosk => "Police Kiosk",
            ActiveTool::PlacePoliceHQ => "Police HQ",
            ActiveTool::PlacePrison => "Prison",
            ActiveTool::PlaceHospital => "Hospital",
            ActiveTool::PlaceMedicalClinic => "Medical Clinic",
            ActiveTool::PlaceMedicalCenter => "Medical Center",
            ActiveTool::PlaceElementarySchool => "Elementary School",
            ActiveTool::PlaceHighSchool => "High School",
            ActiveTool::PlaceUniversity => "University",
            ActiveTool::PlaceLibrary => "Library",
            ActiveTool::PlaceKindergarten => "Kindergarten",
            ActiveTool::PlaceSmallPark => "Small Park",
            ActiveTool::PlaceLargePark => "Large Park",
            ActiveTool::PlacePlayground => "Playground",
            ActiveTool::PlacePlaza => "Plaza",
            ActiveTool::PlaceSportsField => "Sports Field",
            ActiveTool::PlaceStadium => "Stadium",
            ActiveTool::PlaceLandfill => "Landfill",
            ActiveTool::PlaceRecyclingCenter => "Recycling Center",
            ActiveTool::PlaceIncinerator => "Incinerator",
            ActiveTool::PlaceTransferStation => "Transfer Station",
            ActiveTool::PlaceCemetery => "Cemetery",
            ActiveTool::PlaceCrematorium => "Crematorium",
            ActiveTool::PlaceCityHall => "City Hall",
            ActiveTool::PlaceMuseum => "Museum",
            ActiveTool::PlaceCathedral => "Cathedral",
            ActiveTool::PlaceTVStation => "TV Station",
            ActiveTool::PlaceBusDepot => "Bus Depot",
            ActiveTool::PlaceTrainStation => "Train Station",
            ActiveTool::PlaceSubwayStation => "Subway Station",
            ActiveTool::PlaceTramDepot => "Tram Depot",
            ActiveTool::PlaceFerryPier => "Ferry Pier",
            ActiveTool::PlaceSmallAirstrip => "Small Airstrip",
            ActiveTool::PlaceRegionalAirport => "Regional Airport",
            ActiveTool::PlaceInternationalAirport => "Int'l Airport",
            ActiveTool::PlaceCellTower => "Cell Tower",
            ActiveTool::PlaceDataCenter => "Data Center",
            ActiveTool::TerrainRaise => "Raise Terrain",
            ActiveTool::TerrainLower => "Lower Terrain",
            ActiveTool::TerrainLevel => "Level Terrain",
            ActiveTool::TerrainWater => "Place Water",
            ActiveTool::DistrictPaint(_) => "Paint District",
            ActiveTool::DistrictErase => "Erase District",
            ActiveTool::TreePlant => "Plant Tree",
            ActiveTool::TreeRemove => "Remove Tree",
            ActiveTool::RoadUpgrade => "Upgrade Road",
            ActiveTool::AutoGrid => "Auto-Grid",
        }
    }

    /// Returns the `RoadType` for road tools, or `None` for non-road tools.
    pub fn road_type(&self) -> Option<RoadType> {
        match self {
            ActiveTool::Road => Some(RoadType::Local),
            ActiveTool::RoadAvenue => Some(RoadType::Avenue),
            ActiveTool::RoadBoulevard => Some(RoadType::Boulevard),
            ActiveTool::RoadHighway => Some(RoadType::Highway),
            ActiveTool::RoadOneWay => Some(RoadType::OneWay),
            ActiveTool::RoadPath => Some(RoadType::Path),
            _ => None,
        }
    }

    /// Returns the `ZoneType` for zone tools, or `None` for non-zone tools.
    pub fn zone_type(&self) -> Option<ZoneType> {
        match self {
            ActiveTool::ZoneResidentialLow => Some(ZoneType::ResidentialLow),
            ActiveTool::ZoneResidentialMedium => Some(ZoneType::ResidentialMedium),
            ActiveTool::ZoneResidentialHigh => Some(ZoneType::ResidentialHigh),
            ActiveTool::ZoneCommercialLow => Some(ZoneType::CommercialLow),
            ActiveTool::ZoneCommercialHigh => Some(ZoneType::CommercialHigh),
            ActiveTool::ZoneIndustrial => Some(ZoneType::Industrial),
            ActiveTool::ZoneOffice => Some(ZoneType::Office),
            ActiveTool::ZoneMixedUse => Some(ZoneType::MixedUse),
            _ => None,
        }
    }
}

/// Grid snap mode: when enabled, cursor snaps to cell centers for precise placement.
#[derive(Resource, Default)]
pub struct GridSnap {
    pub enabled: bool,
}

#[derive(Resource, Default)]
pub struct CursorGridPos {
    pub grid_x: i32,
    pub grid_y: i32,
    pub world_pos: Vec2,
    pub valid: bool,
}

/// Currently selected building entity for inspection
#[derive(Resource, Default)]
pub struct SelectedBuilding(pub Option<Entity>);

/// Status message shown briefly on screen
#[derive(Resource, Default)]
pub struct StatusMessage {
    pub text: String,
    pub timer: f32,
    pub is_error: bool,
}

impl StatusMessage {
    pub fn set(&mut self, text: impl Into<String>, is_error: bool) {
        self.text = text.into();
        self.timer = 3.0;
        self.is_error = is_error;
    }

    pub fn active(&self) -> bool {
        self.timer > 0.0
    }
}

/// State machine for freeform Bezier road drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawPhase {
    /// No road drawing in progress.
    Idle,
    /// Start point has been placed; waiting for end point click (straight)
    /// or control point click (curve mode).
    PlacedStart,
    /// Curve mode only: start and control point placed; waiting for end point.
    PlacedControl,
}

#[derive(Resource)]
pub struct RoadDrawState {
    pub phase: DrawPhase,
    pub start_pos: Vec2,
    /// In curve mode, the user-specified control point (placed on second click).
    pub control_pos: Vec2,
}

impl Default for RoadDrawState {
    fn default() -> Self {
        Self {
            phase: DrawPhase::Idle,
            start_pos: Vec2::ZERO,
            control_pos: Vec2::ZERO,
        }
    }
}

/// Snap radius for intersection snapping (1 cell distance in world units).
pub(crate) const INTERSECTION_SNAP_RADIUS: f32 = CELL_SIZE;

/// Tracks whether the cursor is currently snapped to an existing intersection node.
#[derive(Resource, Default)]
pub struct IntersectionSnap {
    /// When `Some`, the cursor should snap to this world position.
    pub snapped_pos: Option<Vec2>,
}
