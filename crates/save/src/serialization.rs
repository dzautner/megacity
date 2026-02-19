use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use simulation::budget::{ExtendedBudget, ServiceBudgets, ZoneTaxRates};
use simulation::buildings::Building;
use simulation::citizen::{CitizenDetails, CitizenState, PathCache, Position, Velocity};
use simulation::economy::CityBudget;
use simulation::grid::{RoadType, WorldGrid};
use simulation::life_simulation::LifeSimTimer;
use simulation::lifecycle::LifecycleTimer;
use simulation::loans::{self, LoanBook};
use simulation::policies::{Policies, Policy};
use simulation::road_segments::{
    RoadSegment, RoadSegmentStore, SegmentId, SegmentNode, SegmentNodeId,
};
use simulation::roads::RoadNetwork;
use simulation::services::{ServiceBuilding, ServiceType};
use simulation::time_of_day::GameClock;
use simulation::unlocks::{UnlockNode, UnlockState};
use simulation::utilities::{UtilitySource, UtilityType};
use simulation::virtual_population::{DistrictStats, VirtualPopulation};
use simulation::weather::{Season, Weather, WeatherCondition, WeatherEvent};
// Note: WeatherEvent is a type alias for WeatherCondition (kept for backward compat)
use simulation::water_sources::{WaterSource, WaterSourceType};
use simulation::zones::ZoneDemand;

// ---------------------------------------------------------------------------
// Encoding helpers
// ---------------------------------------------------------------------------

pub fn zone_type_to_u8(z: simulation::grid::ZoneType) -> u8 {
    match z {
        simulation::grid::ZoneType::None => 0,
        simulation::grid::ZoneType::ResidentialLow => 1,
        simulation::grid::ZoneType::ResidentialHigh => 2,
        simulation::grid::ZoneType::CommercialLow => 3,
        simulation::grid::ZoneType::CommercialHigh => 4,
        simulation::grid::ZoneType::Industrial => 5,
        simulation::grid::ZoneType::Office => 6,
        simulation::grid::ZoneType::ResidentialMedium => 7,
    }
}

pub fn u8_to_zone_type(v: u8) -> simulation::grid::ZoneType {
    match v {
        1 => simulation::grid::ZoneType::ResidentialLow,
        2 => simulation::grid::ZoneType::ResidentialHigh,
        3 => simulation::grid::ZoneType::CommercialLow,
        4 => simulation::grid::ZoneType::CommercialHigh,
        5 => simulation::grid::ZoneType::Industrial,
        6 => simulation::grid::ZoneType::Office,
        7 => simulation::grid::ZoneType::ResidentialMedium,
        _ => simulation::grid::ZoneType::None,
    }
}

pub fn utility_type_to_u8(u: UtilityType) -> u8 {
    match u {
        UtilityType::PowerPlant => 0,
        UtilityType::SolarFarm => 1,
        UtilityType::WindTurbine => 2,
        UtilityType::WaterTower => 3,
        UtilityType::SewagePlant => 4,
        UtilityType::NuclearPlant => 5,
        UtilityType::Geothermal => 6,
        UtilityType::PumpingStation => 7,
        UtilityType::WaterTreatment => 8,
    }
}

pub fn u8_to_utility_type(v: u8) -> UtilityType {
    match v {
        0 => UtilityType::PowerPlant,
        1 => UtilityType::SolarFarm,
        2 => UtilityType::WindTurbine,
        3 => UtilityType::WaterTower,
        4 => UtilityType::SewagePlant,
        5 => UtilityType::NuclearPlant,
        6 => UtilityType::Geothermal,
        7 => UtilityType::PumpingStation,
        8 => UtilityType::WaterTreatment,
        _ => UtilityType::PowerPlant, // fallback
    }
}

pub fn service_type_to_u8(s: ServiceType) -> u8 {
    match s {
        ServiceType::FireStation => 0,
        ServiceType::PoliceStation => 1,
        ServiceType::Hospital => 2,
        ServiceType::ElementarySchool => 3,
        ServiceType::HighSchool => 4,
        ServiceType::University => 5,
        ServiceType::Library => 6,
        ServiceType::SmallPark => 7,
        ServiceType::LargePark => 8,
        ServiceType::Playground => 9,
        ServiceType::Plaza => 10,
        ServiceType::SportsField => 11,
        ServiceType::Stadium => 12,
        ServiceType::Landfill => 13,
        ServiceType::RecyclingCenter => 14,
        ServiceType::Incinerator => 15,
        ServiceType::Cemetery => 16,
        ServiceType::Crematorium => 17,
        ServiceType::CityHall => 18,
        ServiceType::Museum => 19,
        ServiceType::Cathedral => 20,
        ServiceType::TVStation => 21,
        ServiceType::BusDepot => 22,
        ServiceType::TrainStation => 23,
        ServiceType::FireHouse => 24,
        ServiceType::FireHQ => 25,
        ServiceType::PoliceKiosk => 26,
        ServiceType::PoliceHQ => 27,
        ServiceType::Prison => 28,
        ServiceType::MedicalClinic => 29,
        ServiceType::MedicalCenter => 30,
        ServiceType::Kindergarten => 31,
        ServiceType::SubwayStation => 32,
        ServiceType::TramDepot => 33,
        ServiceType::FerryPier => 34,
        ServiceType::SmallAirstrip => 35,
        ServiceType::InternationalAirport => 36,
        ServiceType::TransferStation => 37,
        ServiceType::CellTower => 38,
        ServiceType::DataCenter => 39,
        ServiceType::HomelessShelter => 40,
        ServiceType::PostOffice => 41,
        ServiceType::MailSortingCenter => 42,
        ServiceType::RegionalAirport => 43,
        ServiceType::WelfareOffice => 44,
        ServiceType::HeatingBoiler => 45,
        ServiceType::DistrictHeatingPlant => 46,
        ServiceType::GeothermalPlant => 47,
        ServiceType::WaterTreatmentPlant => 48,
        ServiceType::WellPump => 49,
    }
}

pub fn u8_to_service_type(v: u8) -> Option<ServiceType> {
    match v {
        0 => Some(ServiceType::FireStation),
        1 => Some(ServiceType::PoliceStation),
        2 => Some(ServiceType::Hospital),
        3 => Some(ServiceType::ElementarySchool),
        4 => Some(ServiceType::HighSchool),
        5 => Some(ServiceType::University),
        6 => Some(ServiceType::Library),
        7 => Some(ServiceType::SmallPark),
        8 => Some(ServiceType::LargePark),
        9 => Some(ServiceType::Playground),
        10 => Some(ServiceType::Plaza),
        11 => Some(ServiceType::SportsField),
        12 => Some(ServiceType::Stadium),
        13 => Some(ServiceType::Landfill),
        14 => Some(ServiceType::RecyclingCenter),
        15 => Some(ServiceType::Incinerator),
        16 => Some(ServiceType::Cemetery),
        17 => Some(ServiceType::Crematorium),
        18 => Some(ServiceType::CityHall),
        19 => Some(ServiceType::Museum),
        20 => Some(ServiceType::Cathedral),
        21 => Some(ServiceType::TVStation),
        22 => Some(ServiceType::BusDepot),
        23 => Some(ServiceType::TrainStation),
        24 => Some(ServiceType::FireHouse),
        25 => Some(ServiceType::FireHQ),
        26 => Some(ServiceType::PoliceKiosk),
        27 => Some(ServiceType::PoliceHQ),
        28 => Some(ServiceType::Prison),
        29 => Some(ServiceType::MedicalClinic),
        30 => Some(ServiceType::MedicalCenter),
        31 => Some(ServiceType::Kindergarten),
        32 => Some(ServiceType::SubwayStation),
        33 => Some(ServiceType::TramDepot),
        34 => Some(ServiceType::FerryPier),
        35 => Some(ServiceType::SmallAirstrip),
        36 => Some(ServiceType::InternationalAirport),
        37 => Some(ServiceType::TransferStation),
        38 => Some(ServiceType::CellTower),
        39 => Some(ServiceType::DataCenter),
        40 => Some(ServiceType::HomelessShelter),
        41 => Some(ServiceType::PostOffice),
        42 => Some(ServiceType::MailSortingCenter),
        43 => Some(ServiceType::RegionalAirport),
        44 => Some(ServiceType::WelfareOffice),
        45 => Some(ServiceType::HeatingBoiler),
        46 => Some(ServiceType::DistrictHeatingPlant),
        47 => Some(ServiceType::GeothermalPlant),
        48 => Some(ServiceType::WaterTreatmentPlant),
        49 => Some(ServiceType::WellPump),
        _ => None,
    }
}

pub fn road_type_to_u8(r: RoadType) -> u8 {
    match r {
        RoadType::Local => 0,
        RoadType::Avenue => 1,
        RoadType::Boulevard => 2,
        RoadType::Highway => 3,
        RoadType::OneWay => 4,
        RoadType::Path => 5,
    }
}

pub fn u8_to_road_type(v: u8) -> RoadType {
    match v {
        0 => RoadType::Local,
        1 => RoadType::Avenue,
        2 => RoadType::Boulevard,
        3 => RoadType::Highway,
        4 => RoadType::OneWay,
        5 => RoadType::Path,
        _ => RoadType::Local,
    }
}

pub fn policy_to_u8(p: Policy) -> u8 {
    match p {
        Policy::FreePublicTransport => 0,
        Policy::HeavyIndustryTaxBreak => 1,
        Policy::TourismPromotion => 2,
        Policy::SmallBusinessGrant => 3,
        Policy::RecyclingProgram => 4,
        Policy::IndustrialAirFilters => 5,
        Policy::WaterConservation => 6,
        Policy::GreenSpaceInitiative => 7,
        Policy::EducationPush => 8,
        Policy::HealthcareForAll => 9,
        Policy::SmokeDetectorMandate => 10,
        Policy::NeighborhoodWatch => 11,
        Policy::HighRiseBan => 12,
        Policy::NightShiftBan => 13,
        Policy::IndustrialZoningRestriction => 14,
    }
}

pub fn u8_to_policy(v: u8) -> Option<Policy> {
    match v {
        0 => Some(Policy::FreePublicTransport),
        1 => Some(Policy::HeavyIndustryTaxBreak),
        2 => Some(Policy::TourismPromotion),
        3 => Some(Policy::SmallBusinessGrant),
        4 => Some(Policy::RecyclingProgram),
        5 => Some(Policy::IndustrialAirFilters),
        6 => Some(Policy::WaterConservation),
        7 => Some(Policy::GreenSpaceInitiative),
        8 => Some(Policy::EducationPush),
        9 => Some(Policy::HealthcareForAll),
        10 => Some(Policy::SmokeDetectorMandate),
        11 => Some(Policy::NeighborhoodWatch),
        12 => Some(Policy::HighRiseBan),
        13 => Some(Policy::NightShiftBan),
        14 => Some(Policy::IndustrialZoningRestriction),
        _ => None,
    }
}

pub fn weather_event_to_u8(w: WeatherEvent) -> u8 {
    match w {
        WeatherCondition::Sunny => 0,
        WeatherCondition::Rain => 1,
        WeatherCondition::PartlyCloudy => 2,
        WeatherCondition::Overcast => 3,
        WeatherCondition::Storm => 4,
        WeatherCondition::HeavyRain => 5,
        WeatherCondition::Snow => 6,
    }
}

pub fn u8_to_weather_event(v: u8) -> WeatherEvent {
    match v {
        0 => WeatherCondition::Sunny,
        1 => WeatherCondition::Rain,
        2 => WeatherCondition::PartlyCloudy, // was HeatWave, now PartlyCloudy
        3 => WeatherCondition::Overcast,     // was ColdSnap, now Overcast
        4 => WeatherCondition::Storm,
        5 => WeatherCondition::HeavyRain,
        6 => WeatherCondition::Snow,
        _ => WeatherCondition::Sunny,
    }
}

pub fn season_to_u8(s: Season) -> u8 {
    match s {
        Season::Spring => 0,
        Season::Summer => 1,
        Season::Autumn => 2,
        Season::Winter => 3,
    }
}

pub fn u8_to_season(v: u8) -> Season {
    match v {
        0 => Season::Spring,
        1 => Season::Summer,
        2 => Season::Autumn,
        3 => Season::Winter,
        _ => Season::Spring,
    }
}

pub fn water_source_type_to_u8(w: WaterSourceType) -> u8 {
    match w {
        WaterSourceType::Well => 0,
        WaterSourceType::SurfaceIntake => 1,
        WaterSourceType::Reservoir => 2,
        WaterSourceType::Desalination => 3,
    }
}

pub fn u8_to_water_source_type(v: u8) -> Option<WaterSourceType> {
    match v {
        0 => Some(WaterSourceType::Well),
        1 => Some(WaterSourceType::SurfaceIntake),
        2 => Some(WaterSourceType::Reservoir),
        3 => Some(WaterSourceType::Desalination),
        _ => None,
    }
}

pub fn unlock_node_to_u8(n: UnlockNode) -> u8 {
    match n {
        UnlockNode::BasicRoads => 0,
        UnlockNode::ResidentialZoning => 1,
        UnlockNode::CommercialZoning => 2,
        UnlockNode::IndustrialZoning => 3,
        UnlockNode::BasicPower => 4,
        UnlockNode::BasicWater => 5,
        UnlockNode::FireService => 6,
        UnlockNode::PoliceService => 7,
        UnlockNode::ElementaryEducation => 8,
        UnlockNode::SmallParks => 9,
        UnlockNode::BasicSanitation => 10,
        UnlockNode::HealthCare => 11,
        UnlockNode::HighSchoolEducation => 12,
        UnlockNode::HighDensityResidential => 13,
        UnlockNode::HighDensityCommercial => 14,
        UnlockNode::SolarPower => 15,
        UnlockNode::SewagePlant => 16,
        UnlockNode::AdvancedParks => 17,
        UnlockNode::DeathCare => 18,
        UnlockNode::OfficeZoning => 19,
        UnlockNode::UniversityEducation => 20,
        UnlockNode::WindPower => 21,
        UnlockNode::AdvancedSanitation => 22,
        UnlockNode::PublicTransport => 23,
        UnlockNode::Entertainment => 24,
        UnlockNode::AdvancedEmergency => 25,
        UnlockNode::Telecom => 26,
        UnlockNode::AdvancedTransport => 27,
        UnlockNode::Landmarks => 28,
        UnlockNode::PolicySystem => 29,
        UnlockNode::NuclearPower => 30,
        UnlockNode::BasicHeating => 31,
        UnlockNode::DistrictHeatingNetwork => 32,
        UnlockNode::SmallAirstrips => 33,
        UnlockNode::PostalService => 34,
        UnlockNode::WaterInfrastructure => 35,
        UnlockNode::RegionalAirports => 36,
        UnlockNode::InternationalAirports => 37,
    }
}

pub fn u8_to_unlock_node(v: u8) -> Option<UnlockNode> {
    match v {
        0 => Some(UnlockNode::BasicRoads),
        1 => Some(UnlockNode::ResidentialZoning),
        2 => Some(UnlockNode::CommercialZoning),
        3 => Some(UnlockNode::IndustrialZoning),
        4 => Some(UnlockNode::BasicPower),
        5 => Some(UnlockNode::BasicWater),
        6 => Some(UnlockNode::FireService),
        7 => Some(UnlockNode::PoliceService),
        8 => Some(UnlockNode::ElementaryEducation),
        9 => Some(UnlockNode::SmallParks),
        10 => Some(UnlockNode::BasicSanitation),
        11 => Some(UnlockNode::HealthCare),
        12 => Some(UnlockNode::HighSchoolEducation),
        13 => Some(UnlockNode::HighDensityResidential),
        14 => Some(UnlockNode::HighDensityCommercial),
        15 => Some(UnlockNode::SolarPower),
        16 => Some(UnlockNode::SewagePlant),
        17 => Some(UnlockNode::AdvancedParks),
        18 => Some(UnlockNode::DeathCare),
        19 => Some(UnlockNode::OfficeZoning),
        20 => Some(UnlockNode::UniversityEducation),
        21 => Some(UnlockNode::WindPower),
        22 => Some(UnlockNode::AdvancedSanitation),
        23 => Some(UnlockNode::PublicTransport),
        24 => Some(UnlockNode::Entertainment),
        25 => Some(UnlockNode::AdvancedEmergency),
        26 => Some(UnlockNode::Telecom),
        27 => Some(UnlockNode::AdvancedTransport),
        28 => Some(UnlockNode::Landmarks),
        29 => Some(UnlockNode::PolicySystem),
        30 => Some(UnlockNode::NuclearPower),
        31 => Some(UnlockNode::BasicHeating),
        32 => Some(UnlockNode::DistrictHeatingNetwork),
        33 => Some(UnlockNode::SmallAirstrips),
        34 => Some(UnlockNode::PostalService),
        35 => Some(UnlockNode::WaterInfrastructure),
        36 => Some(UnlockNode::RegionalAirports),
        37 => Some(UnlockNode::InternationalAirports),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Version constants
// ---------------------------------------------------------------------------

/// Current save file version.
/// v1 = original fields (grid, roads, clock, budget, demand, buildings, citizens, utilities, services, road_segments)
/// v2 = policies, weather, unlock_state, extended_budget, loans
/// v3 = lifecycle_timer, path_cache, velocity per citizen
/// v4 = life_sim_timer (LifeSimTimer serialization)
/// v5 = water_sources (WaterSource component serialization)
pub const CURRENT_SAVE_VERSION: u32 = 5;

// ---------------------------------------------------------------------------
// Save structs
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveSegmentNode {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub connected_segments: Vec<u32>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveRoadSegment {
    pub id: u32,
    pub start_node: u32,
    pub end_node: u32,
    pub p0_x: f32,
    pub p0_y: f32,
    pub p1_x: f32,
    pub p1_y: f32,
    pub p2_x: f32,
    pub p2_y: f32,
    pub p3_x: f32,
    pub p3_y: f32,
    pub road_type: u8,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveRoadSegmentStore {
    pub nodes: Vec<SaveSegmentNode>,
    pub segments: Vec<SaveRoadSegment>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveData {
    /// Save file format version. Defaults to 0 for legacy saves that predate versioning.
    #[serde(default)]
    pub version: u32,
    pub grid: SaveGrid,
    pub roads: SaveRoadNetwork,
    pub clock: SaveClock,
    pub budget: SaveBudget,
    pub demand: SaveDemand,
    pub buildings: Vec<SaveBuilding>,
    pub citizens: Vec<SaveCitizen>,
    pub utility_sources: Vec<SaveUtilitySource>,
    pub service_buildings: Vec<SaveServiceBuilding>,
    #[serde(default)]
    pub road_segments: Option<SaveRoadSegmentStore>,
    // --- V2 fields (backward-compatible via serde defaults) ---
    #[serde(default)]
    pub policies: Option<SavePolicies>,
    #[serde(default)]
    pub weather: Option<SaveWeather>,
    #[serde(default)]
    pub unlock_state: Option<SaveUnlockState>,
    #[serde(default)]
    pub extended_budget: Option<SaveExtendedBudget>,
    #[serde(default)]
    pub loan_book: Option<SaveLoanBook>,
    #[serde(default)]
    pub lifecycle_timer: Option<SaveLifecycleTimer>,
    #[serde(default)]
    pub virtual_population: Option<SaveVirtualPopulation>,
    #[serde(default)]
    pub life_sim_timer: Option<SaveLifeSimTimer>,
    #[serde(default)]
    pub water_sources: Option<Vec<SaveWaterSource>>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveGrid {
    pub cells: Vec<SaveCell>,
    pub width: usize,
    pub height: usize,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveCell {
    pub elevation: f32,
    pub cell_type: u8,
    pub zone: u8,
    pub road_type: u8,
    pub has_power: bool,
    pub has_water: bool,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveRoadNetwork {
    pub road_positions: Vec<(usize, usize)>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveClock {
    pub day: u32,
    pub hour: f32,
    pub speed: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveBudget {
    pub treasury: f64,
    pub tax_rate: f32,
    pub last_collection_day: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveDemand {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
    pub office: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveBuilding {
    pub zone_type: u8,
    pub level: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub capacity: u32,
    pub occupants: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveCitizen {
    pub age: u8,
    pub happiness: f32,
    pub education: u8,
    pub state: u8,
    pub home_x: usize,
    pub home_y: usize,
    pub work_x: usize,
    pub work_y: usize,
    // V3 fields: PathCache, Velocity, Position (backward-compatible via serde defaults)
    #[serde(default)]
    pub path_waypoints: Vec<(usize, usize)>,
    #[serde(default)]
    pub path_current_index: usize,
    #[serde(default)]
    pub velocity_x: f32,
    #[serde(default)]
    pub velocity_y: f32,
    #[serde(default)]
    pub pos_x: f32,
    #[serde(default)]
    pub pos_y: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveUtilitySource {
    pub utility_type: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub range: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveServiceBuilding {
    pub service_type: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub radius_cells: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveWaterSource {
    pub source_type: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub capacity_mgd: f32,
    pub quality: f32,
    pub operating_cost: f64,
    pub stored_gallons: f32,
    pub storage_capacity: f32,
}

// ---------------------------------------------------------------------------
// V2 save structs: Policies, Weather, UnlockState, ExtendedBudget, LoanBook
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SavePolicies {
    /// Active policy discriminants
    pub active: Vec<u8>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveWeather {
    pub season: u8,
    pub temperature: f32,
    pub current_event: u8,
    pub event_days_remaining: u32,
    pub last_update_day: u32,
    pub disasters_enabled: bool,
    #[serde(default = "default_save_humidity")]
    pub humidity: f32,
    #[serde(default)]
    pub cloud_cover: f32,
    #[serde(default)]
    pub precipitation_intensity: f32,
    #[serde(default)]
    pub last_update_hour: u32,
}

fn default_save_humidity() -> f32 {
    0.5
}

impl Default for SaveWeather {
    fn default() -> Self {
        Self {
            season: 0, // Spring
            temperature: 15.0,
            current_event: 0, // Sunny
            event_days_remaining: 0,
            last_update_day: 0,
            disasters_enabled: true,
            humidity: 0.5,
            cloud_cover: 0.0,
            precipitation_intensity: 0.0,
            last_update_hour: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveUnlockState {
    pub development_points: u32,
    pub spent_points: u32,
    pub unlocked_nodes: Vec<u8>,
    pub last_milestone_pop: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveExtendedBudget {
    // Zone tax rates
    pub residential_tax: f32,
    pub commercial_tax: f32,
    pub industrial_tax: f32,
    pub office_tax: f32,
    // Service budgets
    pub fire_budget: f32,
    pub police_budget: f32,
    pub healthcare_budget: f32,
    pub education_budget: f32,
    pub sanitation_budget: f32,
    pub transport_budget: f32,
}

impl Default for SaveExtendedBudget {
    fn default() -> Self {
        Self {
            residential_tax: 0.10,
            commercial_tax: 0.10,
            industrial_tax: 0.10,
            office_tax: 0.10,
            fire_budget: 1.0,
            police_budget: 1.0,
            healthcare_budget: 1.0,
            education_budget: 1.0,
            sanitation_budget: 1.0,
            transport_budget: 1.0,
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveLifecycleTimer {
    pub last_aging_day: u32,
    pub last_emigration_tick: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveLifeSimTimer {
    pub needs_tick: u32,
    pub life_event_tick: u32,
    pub salary_tick: u32,
    pub education_tick: u32,
    pub job_seek_tick: u32,
    pub personality_tick: u32,
    pub health_tick: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveLoanBook {
    pub loans: Vec<SaveLoan>,
    pub max_loans: u32,
    pub credit_rating: f64,
    pub last_payment_day: u32,
    pub consecutive_solvent_days: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveLoan {
    pub name: String,
    pub amount: f64,
    pub interest_rate: f64,
    pub monthly_payment: f64,
    pub remaining_balance: f64,
    pub term_months: u32,
    pub months_paid: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveDistrictStats {
    pub population: u32,
    pub employed: u32,
    pub avg_happiness: f32,
    pub avg_age: f32,
    pub age_brackets: [u32; 5],
    pub commuters_out: u32,
    pub tax_contribution: f32,
    pub service_demand: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveVirtualPopulation {
    pub total_virtual: u32,
    pub virtual_employed: u32,
    pub district_stats: Vec<SaveDistrictStats>,
    pub max_real_citizens: u32,
}

impl SaveData {
    pub fn encode(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, bitcode::Error> {
        bitcode::decode(bytes)
    }
}

/// Migrate a `SaveData` from any older version up to `CURRENT_SAVE_VERSION`.
///
/// Each migration step handles one version bump. All new fields use `#[serde(default)]`
/// and `Option<T>`, so deserialization itself fills in safe defaults -- migration mostly
/// just bumps the version number so the save will be written at the current version on
/// the next save.
///
/// Returns the original version so callers can log the migration.
pub fn migrate_save(save: &mut SaveData) -> u32 {
    let original_version = save.version;

    // v0 -> v1: Legacy unversioned saves. All required fields (grid, roads,
    // clock, budget, demand, buildings, citizens, etc.) are already present
    // in the original format.  Option fields default to None.
    if save.version == 0 {
        save.version = 1;
    }

    // v1 -> v2: Added policies, weather, unlock_state, extended_budget, loan_book.
    // These are all `Option<T>` with `#[serde(default)]`, so they deserialize as None
    // from a v1 save -- no data fixup needed.
    if save.version == 1 {
        save.version = 2;
    }

    // v2 -> v3: Added lifecycle_timer and per-citizen path_cache / velocity / position.
    // All use `#[serde(default)]` so they already have safe zero/empty defaults.
    if save.version == 2 {
        save.version = 3;
    }

    // v3 -> v4: Added life_sim_timer (LifeSimTimer serialization).
    // Uses `#[serde(default)]` so it deserializes as None from a v3 save.
    if save.version == 3 {
        save.version = 4;
    }

    // v4 -> v5: Added water_sources (WaterSource component serialization).
    // Uses `#[serde(default)]` so it deserializes as None from a v4 save.
    if save.version == 4 {
        save.version = 5;
    }

    // Ensure version is at the current value (safety net for future additions).
    debug_assert_eq!(save.version, CURRENT_SAVE_VERSION);

    original_version
}

/// Input data for serializing a single citizen, collected from ECS queries.
pub struct CitizenSaveInput {
    pub details: CitizenDetails,
    pub state: CitizenState,
    pub home_x: usize,
    pub home_y: usize,
    pub work_x: usize,
    pub work_y: usize,
    pub path: PathCache,
    pub velocity: Velocity,
    pub position: Position,
}

#[allow(clippy::too_many_arguments)]
pub fn create_save_data(
    grid: &WorldGrid,
    roads: &RoadNetwork,
    clock: &GameClock,
    budget: &CityBudget,
    demand: &ZoneDemand,
    buildings: &[(Building,)],
    citizens: &[CitizenSaveInput],
    utility_sources: &[UtilitySource],
    service_buildings: &[(ServiceBuilding,)],
    segment_store: Option<&RoadSegmentStore>,
    policies: Option<&Policies>,
    weather: Option<&Weather>,
    unlock_state: Option<&UnlockState>,
    extended_budget: Option<&ExtendedBudget>,
    loan_book: Option<&LoanBook>,
    lifecycle_timer: Option<&LifecycleTimer>,
    virtual_population: Option<&VirtualPopulation>,
    life_sim_timer: Option<&LifeSimTimer>,
    water_sources: Option<&[WaterSource]>,
) -> SaveData {
    let save_cells: Vec<SaveCell> = grid
        .cells
        .iter()
        .map(|c| SaveCell {
            elevation: c.elevation,
            cell_type: match c.cell_type {
                simulation::grid::CellType::Grass => 0,
                simulation::grid::CellType::Water => 1,
                simulation::grid::CellType::Road => 2,
            },
            zone: zone_type_to_u8(c.zone),
            road_type: road_type_to_u8(c.road_type),
            has_power: c.has_power,
            has_water: c.has_water,
        })
        .collect();

    SaveData {
        version: CURRENT_SAVE_VERSION,
        grid: SaveGrid {
            cells: save_cells,
            width: grid.width,
            height: grid.height,
        },
        roads: SaveRoadNetwork {
            road_positions: roads.edges.keys().map(|n| (n.0, n.1)).collect(),
        },
        clock: SaveClock {
            day: clock.day,
            hour: clock.hour,
            speed: clock.speed,
        },
        budget: SaveBudget {
            treasury: budget.treasury,
            tax_rate: budget.tax_rate,
            last_collection_day: budget.last_collection_day,
        },
        demand: SaveDemand {
            residential: demand.residential,
            commercial: demand.commercial,
            industrial: demand.industrial,
            office: demand.office,
        },
        buildings: buildings
            .iter()
            .map(|(b,)| SaveBuilding {
                zone_type: zone_type_to_u8(b.zone_type),
                level: b.level,
                grid_x: b.grid_x,
                grid_y: b.grid_y,
                capacity: b.capacity,
                occupants: b.occupants,
            })
            .collect(),
        citizens: citizens
            .iter()
            .map(|c| SaveCitizen {
                age: c.details.age,
                happiness: c.details.happiness,
                education: c.details.education,
                state: match c.state {
                    CitizenState::AtHome => 0,
                    CitizenState::CommutingToWork => 1,
                    CitizenState::Working => 2,
                    CitizenState::CommutingHome => 3,
                    CitizenState::CommutingToShop => 4,
                    CitizenState::Shopping => 5,
                    CitizenState::CommutingToLeisure => 6,
                    CitizenState::AtLeisure => 7,
                    CitizenState::CommutingToSchool => 8,
                    CitizenState::AtSchool => 9,
                },
                home_x: c.home_x,
                home_y: c.home_y,
                work_x: c.work_x,
                work_y: c.work_y,
                path_waypoints: c.path.waypoints.iter().map(|n| (n.0, n.1)).collect(),
                path_current_index: c.path.current_index,
                velocity_x: c.velocity.x,
                velocity_y: c.velocity.y,
                pos_x: c.position.x,
                pos_y: c.position.y,
            })
            .collect(),
        utility_sources: utility_sources
            .iter()
            .map(|u| SaveUtilitySource {
                utility_type: utility_type_to_u8(u.utility_type),
                grid_x: u.grid_x,
                grid_y: u.grid_y,
                range: u.range,
            })
            .collect(),
        service_buildings: service_buildings
            .iter()
            .map(|(sb,)| SaveServiceBuilding {
                service_type: service_type_to_u8(sb.service_type),
                grid_x: sb.grid_x,
                grid_y: sb.grid_y,
                radius_cells: (sb.radius / simulation::config::CELL_SIZE) as u32,
            })
            .collect(),
        road_segments: segment_store.map(|store| SaveRoadSegmentStore {
            nodes: store
                .nodes
                .iter()
                .map(|n| SaveSegmentNode {
                    id: n.id.0,
                    x: n.position.x,
                    y: n.position.y,
                    connected_segments: n.connected_segments.iter().map(|s| s.0).collect(),
                })
                .collect(),
            segments: store
                .segments
                .iter()
                .map(|s| SaveRoadSegment {
                    id: s.id.0,
                    start_node: s.start_node.0,
                    end_node: s.end_node.0,
                    p0_x: s.p0.x,
                    p0_y: s.p0.y,
                    p1_x: s.p1.x,
                    p1_y: s.p1.y,
                    p2_x: s.p2.x,
                    p2_y: s.p2.y,
                    p3_x: s.p3.x,
                    p3_y: s.p3.y,
                    road_type: road_type_to_u8(s.road_type),
                })
                .collect(),
        }),
        policies: policies.map(|p| SavePolicies {
            active: p.active.iter().map(|&pol| policy_to_u8(pol)).collect(),
        }),
        weather: weather.map(|w| SaveWeather {
            season: season_to_u8(w.season),
            temperature: w.temperature,
            current_event: weather_event_to_u8(w.current_event),
            event_days_remaining: w.event_days_remaining,
            last_update_day: w.last_update_day,
            disasters_enabled: w.disasters_enabled,
            humidity: w.humidity,
            cloud_cover: w.cloud_cover,
            precipitation_intensity: w.precipitation_intensity,
            last_update_hour: w.last_update_hour,
        }),
        unlock_state: unlock_state.map(|u| SaveUnlockState {
            development_points: u.development_points,
            spent_points: u.spent_points,
            unlocked_nodes: u
                .unlocked_nodes
                .iter()
                .map(|&n| unlock_node_to_u8(n))
                .collect(),
            last_milestone_pop: u.last_milestone_pop,
        }),
        extended_budget: extended_budget.map(|eb| SaveExtendedBudget {
            residential_tax: eb.zone_taxes.residential,
            commercial_tax: eb.zone_taxes.commercial,
            industrial_tax: eb.zone_taxes.industrial,
            office_tax: eb.zone_taxes.office,
            fire_budget: eb.service_budgets.fire,
            police_budget: eb.service_budgets.police,
            healthcare_budget: eb.service_budgets.healthcare,
            education_budget: eb.service_budgets.education,
            sanitation_budget: eb.service_budgets.sanitation,
            transport_budget: eb.service_budgets.transport,
        }),
        loan_book: loan_book.map(|lb| SaveLoanBook {
            loans: lb
                .active_loans
                .iter()
                .map(|l| SaveLoan {
                    name: l.name.clone(),
                    amount: l.amount,
                    interest_rate: l.interest_rate,
                    monthly_payment: l.monthly_payment,
                    remaining_balance: l.remaining_balance,
                    term_months: l.term_months,
                    months_paid: l.months_paid,
                })
                .collect(),
            max_loans: lb.max_loans as u32,
            credit_rating: lb.credit_rating,
            last_payment_day: lb.last_payment_day,
            consecutive_solvent_days: lb.consecutive_solvent_days,
        }),
        lifecycle_timer: lifecycle_timer.map(|lt| SaveLifecycleTimer {
            last_aging_day: lt.last_aging_day,
            last_emigration_tick: lt.last_emigration_tick,
        }),
        virtual_population: virtual_population.map(|vp| SaveVirtualPopulation {
            total_virtual: vp.total_virtual,
            virtual_employed: vp.virtual_employed,
            district_stats: vp
                .district_stats
                .iter()
                .map(|ds| SaveDistrictStats {
                    population: ds.population,
                    employed: ds.employed,
                    avg_happiness: ds.avg_happiness,
                    avg_age: ds.avg_age,
                    age_brackets: ds.age_brackets,
                    commuters_out: ds.commuters_out,
                    tax_contribution: ds.tax_contribution,
                    service_demand: ds.service_demand,
                })
                .collect(),
            max_real_citizens: vp.max_real_citizens,
        }),
        life_sim_timer: life_sim_timer.map(|lst| SaveLifeSimTimer {
            needs_tick: lst.needs_tick,
            life_event_tick: lst.life_event_tick,
            salary_tick: lst.salary_tick,
            education_tick: lst.education_tick,
            job_seek_tick: lst.job_seek_tick,
            personality_tick: lst.personality_tick,
            health_tick: lst.health_tick,
        }),
        water_sources: water_sources.map(|ws| {
            ws.iter()
                .map(|s| SaveWaterSource {
                    source_type: water_source_type_to_u8(s.source_type),
                    grid_x: s.grid_x,
                    grid_y: s.grid_y,
                    capacity_mgd: s.capacity_mgd,
                    quality: s.quality,
                    operating_cost: s.operating_cost,
                    stored_gallons: s.stored_gallons,
                    storage_capacity: s.storage_capacity,
                })
                .collect()
        }),
    }
}

/// Reconstruct a `RoadSegmentStore` from saved data.
/// After calling this, call `store.rasterize_all(grid, roads)` to rebuild grid cells.
pub fn restore_road_segment_store(save: &SaveRoadSegmentStore) -> RoadSegmentStore {
    use bevy::math::Vec2;

    let nodes: Vec<SegmentNode> = save
        .nodes
        .iter()
        .map(|n| SegmentNode {
            id: SegmentNodeId(n.id),
            position: Vec2::new(n.x, n.y),
            connected_segments: n.connected_segments.iter().map(|&s| SegmentId(s)).collect(),
        })
        .collect();

    let segments: Vec<RoadSegment> = save
        .segments
        .iter()
        .map(|s| RoadSegment {
            id: SegmentId(s.id),
            start_node: SegmentNodeId(s.start_node),
            end_node: SegmentNodeId(s.end_node),
            p0: Vec2::new(s.p0_x, s.p0_y),
            p1: Vec2::new(s.p1_x, s.p1_y),
            p2: Vec2::new(s.p2_x, s.p2_y),
            p3: Vec2::new(s.p3_x, s.p3_y),
            road_type: u8_to_road_type(s.road_type),
            arc_length: 0.0,
            rasterized_cells: Vec::new(),
        })
        .collect();

    RoadSegmentStore::from_parts(nodes, segments)
}

/// Restore a `Policies` resource from saved data.
pub fn restore_policies(save: &SavePolicies) -> Policies {
    let active = save
        .active
        .iter()
        .filter_map(|&v| u8_to_policy(v))
        .collect();
    Policies { active }
}

/// Restore a `Weather` resource from saved data.
pub fn restore_weather(save: &SaveWeather) -> Weather {
    Weather {
        season: u8_to_season(save.season),
        temperature: save.temperature,
        current_event: u8_to_weather_event(save.current_event),
        event_days_remaining: save.event_days_remaining,
        last_update_day: save.last_update_day,
        disasters_enabled: save.disasters_enabled,
        humidity: save.humidity,
        cloud_cover: save.cloud_cover,
        precipitation_intensity: save.precipitation_intensity,
        last_update_hour: save.last_update_hour,
    }
}

/// Restore an `UnlockState` resource from saved data.
pub fn restore_unlock_state(save: &SaveUnlockState) -> UnlockState {
    let unlocked_nodes = save
        .unlocked_nodes
        .iter()
        .filter_map(|&v| u8_to_unlock_node(v))
        .collect();
    UnlockState {
        development_points: save.development_points,
        spent_points: save.spent_points,
        unlocked_nodes,
        last_milestone_pop: save.last_milestone_pop,
    }
}

/// Restore an `ExtendedBudget` resource from saved data.
pub fn restore_extended_budget(save: &SaveExtendedBudget) -> ExtendedBudget {
    ExtendedBudget {
        zone_taxes: ZoneTaxRates {
            residential: save.residential_tax,
            commercial: save.commercial_tax,
            industrial: save.industrial_tax,
            office: save.office_tax,
        },
        service_budgets: ServiceBudgets {
            fire: save.fire_budget,
            police: save.police_budget,
            healthcare: save.healthcare_budget,
            education: save.education_budget,
            sanitation: save.sanitation_budget,
            transport: save.transport_budget,
        },
        // Loans are stored separately in the LoanBook (budget.rs loans are legacy);
        // leave the ExtendedBudget.loans empty.
        loans: Vec::new(),
        income_breakdown: Default::default(),
        expense_breakdown: Default::default(),
    }
}

/// Restore a `LoanBook` resource from saved data.
pub fn restore_loan_book(save: &SaveLoanBook) -> LoanBook {
    let active_loans = save
        .loans
        .iter()
        .map(|sl| loans::Loan {
            name: sl.name.clone(),
            amount: sl.amount,
            interest_rate: sl.interest_rate,
            monthly_payment: sl.monthly_payment,
            remaining_balance: sl.remaining_balance,
            term_months: sl.term_months,
            months_paid: sl.months_paid,
        })
        .collect();
    LoanBook {
        active_loans,
        max_loans: save.max_loans as usize,
        credit_rating: save.credit_rating,
        last_payment_day: save.last_payment_day,
        consecutive_solvent_days: save.consecutive_solvent_days,
    }
}

/// Restore a `LifecycleTimer` resource from saved data.
pub fn restore_lifecycle_timer(save: &SaveLifecycleTimer) -> LifecycleTimer {
    LifecycleTimer {
        last_aging_day: save.last_aging_day,
        last_emigration_tick: save.last_emigration_tick,
    }
}

/// Restore a `LifeSimTimer` resource from saved data.
pub fn restore_life_sim_timer(save: &SaveLifeSimTimer) -> LifeSimTimer {
    LifeSimTimer {
        needs_tick: save.needs_tick,
        life_event_tick: save.life_event_tick,
        salary_tick: save.salary_tick,
        education_tick: save.education_tick,
        job_seek_tick: save.job_seek_tick,
        personality_tick: save.personality_tick,
        health_tick: save.health_tick,
    }
}

/// Restore a `WaterSource` component from saved data.
pub fn restore_water_source(save: &SaveWaterSource) -> Option<WaterSource> {
    let source_type = u8_to_water_source_type(save.source_type)?;
    Some(WaterSource {
        source_type,
        capacity_mgd: save.capacity_mgd,
        quality: save.quality,
        operating_cost: save.operating_cost,
        grid_x: save.grid_x,
        grid_y: save.grid_y,
        stored_gallons: save.stored_gallons,
        storage_capacity: save.storage_capacity,
    })
}

/// Restore a `VirtualPopulation` resource from saved data.
pub fn restore_virtual_population(save: &SaveVirtualPopulation) -> VirtualPopulation {
    let district_stats = save
        .district_stats
        .iter()
        .map(|ds| DistrictStats {
            population: ds.population,
            employed: ds.employed,
            avg_happiness: ds.avg_happiness,
            avg_age: ds.avg_age,
            age_brackets: ds.age_brackets,
            commuters_out: ds.commuters_out,
            tax_contribution: ds.tax_contribution,
            service_demand: ds.service_demand,
        })
        .collect();
    VirtualPopulation::from_saved(
        save.total_virtual,
        save.virtual_employed,
        district_stats,
        save.max_real_citizens,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_serialization() {
        let mut grid = WorldGrid::new(16, 16);
        simulation::terrain::generate_terrain(&mut grid, 42);

        // Set some zones to test the new types
        grid.get_mut(5, 5).zone = simulation::grid::ZoneType::ResidentialLow;
        grid.get_mut(6, 6).zone = simulation::grid::ZoneType::ResidentialHigh;
        grid.get_mut(7, 7).zone = simulation::grid::ZoneType::CommercialLow;
        grid.get_mut(8, 8).zone = simulation::grid::ZoneType::CommercialHigh;
        grid.get_mut(9, 9).zone = simulation::grid::ZoneType::Office;

        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");

        assert_eq!(restored.grid.width, 16);
        assert_eq!(restored.grid.height, 16);
        assert_eq!(restored.grid.cells.len(), 256);
        assert_eq!(restored.clock.day, clock.day);
        assert!((restored.budget.treasury - budget.treasury).abs() < 0.01);

        // Verify zone roundtrip
        let idx55 = 5 * 16 + 5;
        assert_eq!(restored.grid.cells[idx55].zone, 1); // ResidentialLow
        let idx66 = 6 * 16 + 6;
        assert_eq!(restored.grid.cells[idx66].zone, 2); // ResidentialHigh
        let idx77 = 7 * 16 + 7;
        assert_eq!(restored.grid.cells[idx77].zone, 3); // CommercialLow
        let idx88 = 8 * 16 + 8;
        assert_eq!(restored.grid.cells[idx88].zone, 4); // CommercialHigh
        let idx99 = 9 * 16 + 9;
        assert_eq!(restored.grid.cells[idx99].zone, 6); // Office

        // V2 fields should be None when not provided
        assert!(restored.policies.is_none());
        assert!(restored.weather.is_none());
        assert!(restored.unlock_state.is_none());
        assert!(restored.extended_budget.is_none());
        assert!(restored.loan_book.is_none());
        assert!(restored.virtual_population.is_none());
    }

    #[test]
    fn test_zone_type_roundtrip() {
        use simulation::grid::ZoneType;
        let types = [
            ZoneType::None,
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
        ];
        for zt in &types {
            let encoded = zone_type_to_u8(*zt);
            let decoded = u8_to_zone_type(encoded);
            assert_eq!(*zt, decoded);
        }
    }

    #[test]
    fn test_utility_type_roundtrip() {
        let types = [
            UtilityType::PowerPlant,
            UtilityType::SolarFarm,
            UtilityType::WindTurbine,
            UtilityType::WaterTower,
            UtilityType::SewagePlant,
        ];
        for ut in &types {
            let encoded = utility_type_to_u8(*ut);
            let decoded = u8_to_utility_type(encoded);
            assert_eq!(*ut, decoded);
        }
    }

    #[test]
    fn test_service_type_roundtrip() {
        for i in 0..=49u8 {
            let st = u8_to_service_type(i).expect("valid service type");
            let encoded = service_type_to_u8(st);
            assert_eq!(i, encoded);
        }
        assert!(u8_to_service_type(50).is_none());
    }

    #[test]
    fn test_policy_roundtrip() {
        for &p in Policy::all() {
            let encoded = policy_to_u8(p);
            let decoded = u8_to_policy(encoded).expect("valid policy");
            assert_eq!(p, decoded);
        }
        assert!(u8_to_policy(255).is_none());
    }

    #[test]
    fn test_weather_roundtrip() {
        let weather = Weather {
            season: Season::Winter,
            temperature: -5.0,
            current_event: WeatherCondition::Snow,
            event_days_remaining: 3,
            last_update_day: 42,
            disasters_enabled: false,
            humidity: 0.8,
            cloud_cover: 0.7,
            precipitation_intensity: 0.5,
            last_update_hour: 14,
        };

        let save = SaveWeather {
            season: season_to_u8(weather.season),
            temperature: weather.temperature,
            current_event: weather_event_to_u8(weather.current_event),
            event_days_remaining: weather.event_days_remaining,
            last_update_day: weather.last_update_day,
            disasters_enabled: weather.disasters_enabled,
            humidity: weather.humidity,
            cloud_cover: weather.cloud_cover,
            precipitation_intensity: weather.precipitation_intensity,
            last_update_hour: weather.last_update_hour,
        };

        let restored = restore_weather(&save);
        assert_eq!(restored.season, Season::Winter);
        assert!((restored.temperature - (-5.0)).abs() < 0.001);
        assert_eq!(restored.current_event, WeatherCondition::Snow);
        assert_eq!(restored.event_days_remaining, 3);
        assert_eq!(restored.last_update_day, 42);
        assert!(!restored.disasters_enabled);
        assert!((restored.humidity - 0.8).abs() < 0.001);
        assert!((restored.cloud_cover - 0.7).abs() < 0.001);
        assert!((restored.precipitation_intensity - 0.5).abs() < 0.001);
        assert_eq!(restored.last_update_hour, 14);
    }

    #[test]
    fn test_unlock_state_roundtrip() {
        let mut state = UnlockState::default();
        state.development_points = 10;
        state.spent_points = 3;
        state.last_milestone_pop = 2000;
        // Default already has BasicRoads, etc. Add another
        state.unlocked_nodes.push(UnlockNode::FireService);

        let save = SaveUnlockState {
            development_points: state.development_points,
            spent_points: state.spent_points,
            unlocked_nodes: state
                .unlocked_nodes
                .iter()
                .map(|&n| unlock_node_to_u8(n))
                .collect(),
            last_milestone_pop: state.last_milestone_pop,
        };

        let restored = restore_unlock_state(&save);
        assert_eq!(restored.development_points, 10);
        assert_eq!(restored.spent_points, 3);
        assert_eq!(restored.last_milestone_pop, 2000);
        assert!(restored.is_unlocked(UnlockNode::BasicRoads));
        assert!(restored.is_unlocked(UnlockNode::FireService));
        assert!(!restored.is_unlocked(UnlockNode::NuclearPower));
    }

    #[test]
    fn test_unlock_node_roundtrip() {
        for &n in UnlockNode::all() {
            let encoded = unlock_node_to_u8(n);
            let decoded = u8_to_unlock_node(encoded).expect("valid unlock node");
            assert_eq!(n, decoded);
        }
        assert!(u8_to_unlock_node(255).is_none());
    }

    #[test]
    fn test_policies_serialize_roundtrip() {
        let policies = Policies {
            active: vec![
                Policy::FreePublicTransport,
                Policy::RecyclingProgram,
                Policy::HighRiseBan,
            ],
        };

        let save = SavePolicies {
            active: policies.active.iter().map(|&p| policy_to_u8(p)).collect(),
        };

        let restored = restore_policies(&save);
        assert_eq!(restored.active.len(), 3);
        assert!(restored.is_active(Policy::FreePublicTransport));
        assert!(restored.is_active(Policy::RecyclingProgram));
        assert!(restored.is_active(Policy::HighRiseBan));
        assert!(!restored.is_active(Policy::EducationPush));
    }

    #[test]
    fn test_extended_budget_roundtrip() {
        let save = SaveExtendedBudget {
            residential_tax: 0.12,
            commercial_tax: 0.08,
            industrial_tax: 0.15,
            office_tax: 0.11,
            fire_budget: 1.2,
            police_budget: 0.8,
            healthcare_budget: 1.0,
            education_budget: 1.5,
            sanitation_budget: 0.5,
            transport_budget: 1.1,
        };

        let restored = restore_extended_budget(&save);
        assert!((restored.zone_taxes.residential - 0.12).abs() < 0.001);
        assert!((restored.zone_taxes.commercial - 0.08).abs() < 0.001);
        assert!((restored.zone_taxes.industrial - 0.15).abs() < 0.001);
        assert!((restored.zone_taxes.office - 0.11).abs() < 0.001);
        assert!((restored.service_budgets.fire - 1.2).abs() < 0.001);
        assert!((restored.service_budgets.police - 0.8).abs() < 0.001);
        assert!((restored.service_budgets.education - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_loan_book_roundtrip() {
        let save = SaveLoanBook {
            loans: vec![SaveLoan {
                name: "Small Loan".into(),
                amount: 10_000.0,
                interest_rate: 0.05,
                monthly_payment: 856.07,
                remaining_balance: 8_500.0,
                term_months: 12,
                months_paid: 2,
            }],
            max_loans: 3,
            credit_rating: 1.5,
            last_payment_day: 60,
            consecutive_solvent_days: 45,
        };

        let restored = restore_loan_book(&save);
        assert_eq!(restored.active_loans.len(), 1);
        assert_eq!(restored.active_loans[0].name, "Small Loan");
        assert!((restored.active_loans[0].amount - 10_000.0).abs() < 0.01);
        assert!((restored.active_loans[0].remaining_balance - 8_500.0).abs() < 0.01);
        assert_eq!(restored.active_loans[0].months_paid, 2);
        assert_eq!(restored.max_loans, 3);
        assert!((restored.credit_rating - 1.5).abs() < 0.001);
        assert_eq!(restored.last_payment_day, 60);
        assert_eq!(restored.consecutive_solvent_days, 45);
    }

    #[test]
    fn test_v2_full_roundtrip() {
        // Test that all V2 fields survive a full encode/decode cycle
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let policies = Policies {
            active: vec![Policy::EducationPush, Policy::WaterConservation],
        };
        let weather = Weather {
            season: Season::Summer,
            temperature: 32.0,
            current_event: WeatherCondition::Sunny,
            event_days_remaining: 4,
            last_update_day: 100,
            disasters_enabled: true,
            humidity: 0.3,
            cloud_cover: 0.05,
            precipitation_intensity: 0.0,
            last_update_hour: 12,
        };
        let mut unlock = UnlockState::default();
        unlock.development_points = 15;
        unlock.spent_points = 5;
        unlock.unlocked_nodes.push(UnlockNode::HealthCare);
        unlock.last_milestone_pop = 5000;

        let ext_budget = ExtendedBudget {
            zone_taxes: ZoneTaxRates {
                residential: 0.12,
                commercial: 0.09,
                industrial: 0.14,
                office: 0.11,
            },
            service_budgets: ServiceBudgets {
                fire: 1.3,
                police: 0.9,
                healthcare: 1.0,
                education: 1.2,
                sanitation: 0.7,
                transport: 1.1,
            },
            loans: Vec::new(),
            income_breakdown: Default::default(),
            expense_breakdown: Default::default(),
        };

        let mut loan_book = LoanBook::default();
        let mut treasury = 0.0;
        loan_book.take_loan(loans::LoanTier::Small, &mut treasury);

        let lifecycle_timer = LifecycleTimer {
            last_aging_day: 200,
            last_emigration_tick: 15,
        };

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            Some(&policies),
            Some(&weather),
            Some(&unlock),
            Some(&ext_budget),
            Some(&loan_book),
            Some(&lifecycle_timer),
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode v2 should succeed");

        // Policies
        let rp = restored.policies.as_ref().expect("policies present");
        assert_eq!(rp.active.len(), 2);

        // Weather
        let rw = restored.weather.as_ref().expect("weather present");
        assert_eq!(rw.season, season_to_u8(Season::Summer));
        assert!((rw.temperature - 32.0).abs() < 0.001);
        assert_eq!(
            rw.current_event,
            weather_event_to_u8(WeatherCondition::Sunny)
        );

        // Unlock state
        let ru = restored
            .unlock_state
            .as_ref()
            .expect("unlock_state present");
        assert_eq!(ru.development_points, 15);
        assert_eq!(ru.spent_points, 5);
        assert_eq!(ru.last_milestone_pop, 5000);

        // Extended budget
        let reb = restored
            .extended_budget
            .as_ref()
            .expect("extended_budget present");
        assert!((reb.fire_budget - 1.3).abs() < 0.001);
        assert!((reb.residential_tax - 0.12).abs() < 0.001);

        // Loan book
        let rlb = restored.loan_book.as_ref().expect("loan_book present");
        assert_eq!(rlb.loans.len(), 1);
        assert_eq!(rlb.loans[0].name, "Small Loan");
    }

    #[test]
    fn test_backward_compat_v1_defaults() {
        // Simulate a V1 save that has no V2 fields: create a SaveData with
        // all V2 fields set to None, encode it, decode it, and verify defaults work.
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode v1 should succeed");

        // V2 fields should be None
        assert!(restored.policies.is_none());
        assert!(restored.weather.is_none());
        assert!(restored.unlock_state.is_none());
        assert!(restored.extended_budget.is_none());
        assert!(restored.loan_book.is_none());
        assert!(restored.lifecycle_timer.is_none());
        assert!(restored.virtual_population.is_none());
        assert!(restored.life_sim_timer.is_none());
    }

    #[test]
    fn test_lifecycle_timer_roundtrip() {
        let timer = LifecycleTimer {
            last_aging_day: 730,
            last_emigration_tick: 25,
        };

        let save = SaveLifecycleTimer {
            last_aging_day: timer.last_aging_day,
            last_emigration_tick: timer.last_emigration_tick,
        };

        let restored = restore_lifecycle_timer(&save);
        assert_eq!(restored.last_aging_day, 730);
        assert_eq!(restored.last_emigration_tick, 25);
    }

    // -----------------------------------------------------------------------
    // Save versioning / migration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_save_data_sets_current_version() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_migrate_from_v0_to_current() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        save.version = 0;

        let old = migrate_save(&mut save);
        assert_eq!(old, 0);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_migrate_current_version_is_noop() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert_eq!(save.version, CURRENT_SAVE_VERSION);
        let old = migrate_save(&mut save);
        assert_eq!(old, CURRENT_SAVE_VERSION);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_version_roundtrips_through_encode_decode() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");
        assert_eq!(restored.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_migrate_from_v1() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        save.version = 1;

        let old = migrate_save(&mut save);
        assert_eq!(old, 1);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_migrate_from_v2() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        save.version = 2;

        let old = migrate_save(&mut save);
        assert_eq!(old, 2);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_virtual_population_roundtrip() {
        let mut vp = VirtualPopulation::default();
        vp.add_virtual_citizen(0, 25, true, 75.0, 1000.0, 0.1);
        vp.add_virtual_citizen(0, 40, false, 50.0, 0.0, 0.0);
        vp.add_virtual_citizen(1, 60, true, 80.0, 1500.0, 0.12);

        let save = SaveVirtualPopulation {
            total_virtual: vp.total_virtual,
            virtual_employed: vp.virtual_employed,
            district_stats: vp
                .district_stats
                .iter()
                .map(|ds| SaveDistrictStats {
                    population: ds.population,
                    employed: ds.employed,
                    avg_happiness: ds.avg_happiness,
                    avg_age: ds.avg_age,
                    age_brackets: ds.age_brackets,
                    commuters_out: ds.commuters_out,
                    tax_contribution: ds.tax_contribution,
                    service_demand: ds.service_demand,
                })
                .collect(),
            max_real_citizens: vp.max_real_citizens,
        };

        let restored = restore_virtual_population(&save);
        assert_eq!(restored.total_virtual, 3);
        assert_eq!(restored.virtual_employed, 2);
        assert_eq!(restored.district_stats.len(), 2);
        assert_eq!(restored.district_stats[0].population, 2);
        assert_eq!(restored.district_stats[0].employed, 1);
        assert_eq!(restored.district_stats[1].population, 1);
        assert_eq!(restored.district_stats[1].employed, 1);
        assert_eq!(restored.max_real_citizens, vp.max_real_citizens);
    }

    #[test]
    fn test_pathcache_velocity_citizen_roundtrip() {
        use simulation::roads::RoadNode;
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();
        let citizens = vec![
            CitizenSaveInput {
                details: CitizenDetails {
                    age: 30,
                    gender: simulation::citizen::Gender::Male,
                    education: 2,
                    happiness: 75.0,
                    health: 90.0,
                    salary: 3500.0,
                    savings: 7000.0,
                },
                state: CitizenState::CommutingToWork,
                home_x: 1,
                home_y: 1,
                work_x: 3,
                work_y: 3,
                path: PathCache {
                    waypoints: vec![
                        RoadNode(1, 1),
                        RoadNode(2, 1),
                        RoadNode(2, 2),
                        RoadNode(3, 3),
                    ],
                    current_index: 1,
                },
                velocity: Velocity { x: 4.5, y: -2.3 },
                position: Position { x: 100.0, y: 200.0 },
            },
            CitizenSaveInput {
                details: CitizenDetails {
                    age: 45,
                    gender: simulation::citizen::Gender::Female,
                    education: 1,
                    happiness: 60.0,
                    health: 80.0,
                    salary: 2200.0,
                    savings: 4400.0,
                },
                state: CitizenState::AtHome,
                home_x: 2,
                home_y: 2,
                work_x: 3,
                work_y: 2,
                path: PathCache {
                    waypoints: vec![],
                    current_index: 0,
                },
                velocity: Velocity { x: 0.0, y: 0.0 },
                position: Position { x: 50.0, y: 75.0 },
            },
        ];
        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &citizens,
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");
        assert_eq!(restored.citizens.len(), 2);
        // First citizen: active path with waypoints
        let c0 = &restored.citizens[0];
        assert_eq!(c0.path_waypoints, vec![(1, 1), (2, 1), (2, 2), (3, 3)]);
        assert_eq!(c0.path_current_index, 1);
        assert!((c0.velocity_x - 4.5).abs() < 0.001);
        assert!((c0.velocity_y - (-2.3)).abs() < 0.001);
        assert!((c0.pos_x - 100.0).abs() < 0.001);
        assert!((c0.pos_y - 200.0).abs() < 0.001);
        assert_eq!(c0.state, 1); // CommutingToWork
                                 // Second citizen: idle, empty path
        let c1 = &restored.citizens[1];
        assert!(c1.path_waypoints.is_empty());
        assert_eq!(c1.path_current_index, 0);
        assert!((c1.velocity_x).abs() < 0.001);
        assert!((c1.velocity_y).abs() < 0.001);
        assert!((c1.pos_x - 50.0).abs() < 0.001);
        assert!((c1.pos_y - 75.0).abs() < 0.001);
        assert_eq!(c1.state, 0); // AtHome
    }

    #[test]
    fn test_pathcache_velocity_v2_backward_compat() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();
        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        // Simulate an old save citizen with default V3 fields
        save.citizens.push(SaveCitizen {
            age: 25,
            happiness: 70.0,
            education: 1,
            state: 1, // CommutingToWork
            home_x: 1,
            home_y: 1,
            work_x: 3,
            work_y: 3,
            path_waypoints: vec![],
            path_current_index: 0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            pos_x: 0.0,
            pos_y: 0.0,
        });
        save.version = 2;
        let old = migrate_save(&mut save);
        assert_eq!(old, 2);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
        let c = &save.citizens[0];
        assert!(c.path_waypoints.is_empty());
        assert_eq!(c.path_current_index, 0);
        assert!((c.velocity_x).abs() < 0.001);
        assert!((c.velocity_y).abs() < 0.001);
    }

    #[test]
    fn test_life_sim_timer_roundtrip() {
        let timer = LifeSimTimer {
            needs_tick: 7,
            life_event_tick: 123,
            salary_tick: 9999,
            education_tick: 500,
            job_seek_tick: 42,
            personality_tick: 1234,
            health_tick: 777,
        };

        let save = SaveLifeSimTimer {
            needs_tick: timer.needs_tick,
            life_event_tick: timer.life_event_tick,
            salary_tick: timer.salary_tick,
            education_tick: timer.education_tick,
            job_seek_tick: timer.job_seek_tick,
            personality_tick: timer.personality_tick,
            health_tick: timer.health_tick,
        };

        let restored = restore_life_sim_timer(&save);
        assert_eq!(restored.needs_tick, 7);
        assert_eq!(restored.life_event_tick, 123);
        assert_eq!(restored.salary_tick, 9999);
        assert_eq!(restored.education_tick, 500);
        assert_eq!(restored.job_seek_tick, 42);
        assert_eq!(restored.personality_tick, 1234);
        assert_eq!(restored.health_tick, 777);
    }

    #[test]
    fn test_life_sim_timer_full_roundtrip() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let life_sim_timer = LifeSimTimer {
            needs_tick: 5,
            life_event_tick: 300,
            salary_tick: 20000,
            education_tick: 700,
            job_seek_tick: 100,
            personality_tick: 1500,
            health_tick: 900,
        };

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(&life_sim_timer),
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");

        let rlst = restored
            .life_sim_timer
            .as_ref()
            .expect("life_sim_timer present");
        assert_eq!(rlst.needs_tick, 5);
        assert_eq!(rlst.life_event_tick, 300);
        assert_eq!(rlst.salary_tick, 20000);
        assert_eq!(rlst.education_tick, 700);
        assert_eq!(rlst.job_seek_tick, 100);
        assert_eq!(rlst.personality_tick, 1500);
        assert_eq!(rlst.health_tick, 900);
    }

    #[test]
    fn test_life_sim_timer_backward_compat() {
        // Saves without life_sim_timer should have it as None
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");
        assert!(restored.life_sim_timer.is_none());
    }

    #[test]
    fn test_migrate_from_v3() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        save.version = 3;

        let old = migrate_save(&mut save);
        assert_eq!(old, 3);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_water_source_type_roundtrip() {
        let types = [
            WaterSourceType::Well,
            WaterSourceType::SurfaceIntake,
            WaterSourceType::Reservoir,
            WaterSourceType::Desalination,
        ];
        for wt in &types {
            let encoded = water_source_type_to_u8(*wt);
            let decoded = u8_to_water_source_type(encoded).expect("valid water source type");
            assert_eq!(*wt, decoded);
        }
        assert!(u8_to_water_source_type(255).is_none());
    }

    #[test]
    fn test_water_source_save_roundtrip() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let water_sources = vec![
            WaterSource {
                source_type: WaterSourceType::Well,
                capacity_mgd: 0.5,
                quality: 0.7,
                operating_cost: 15.0,
                grid_x: 2,
                grid_y: 2,
                stored_gallons: 0.0,
                storage_capacity: 0.0,
            },
            WaterSource {
                source_type: WaterSourceType::Reservoir,
                capacity_mgd: 20.0,
                quality: 0.8,
                operating_cost: 200.0,
                grid_x: 1,
                grid_y: 1,
                stored_gallons: 1_800_000_000.0,
                storage_capacity: 1_800_000_000.0,
            },
        ];

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(&water_sources),
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");

        let rws = restored
            .water_sources
            .as_ref()
            .expect("water_sources present");
        assert_eq!(rws.len(), 2);

        let w0 = &rws[0];
        assert_eq!(
            u8_to_water_source_type(w0.source_type),
            Some(WaterSourceType::Well)
        );
        assert!((w0.capacity_mgd - 0.5).abs() < 0.001);
        assert!((w0.quality - 0.7).abs() < 0.001);
        assert_eq!(w0.grid_x, 2);
        assert_eq!(w0.grid_y, 2);

        let w1 = &rws[1];
        assert_eq!(
            u8_to_water_source_type(w1.source_type),
            Some(WaterSourceType::Reservoir)
        );
        assert!((w1.capacity_mgd - 20.0).abs() < 0.001);
        assert!(w1.stored_gallons > 0.0);
    }

    #[test]
    fn test_water_source_restore() {
        let save = SaveWaterSource {
            source_type: water_source_type_to_u8(WaterSourceType::Desalination),
            grid_x: 5,
            grid_y: 5,
            capacity_mgd: 10.0,
            quality: 0.95,
            operating_cost: 500.0,
            stored_gallons: 0.0,
            storage_capacity: 0.0,
        };

        let ws = restore_water_source(&save).expect("valid water source");
        assert_eq!(ws.source_type, WaterSourceType::Desalination);
        assert!((ws.capacity_mgd - 10.0).abs() < 0.001);
        assert!((ws.quality - 0.95).abs() < 0.001);
        assert_eq!(ws.grid_x, 5);
        assert_eq!(ws.grid_y, 5);
    }

    #[test]
    fn test_water_source_backward_compat() {
        // Saves without water_sources should have it as None
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        let bytes = save.encode();
        let restored = SaveData::decode(&bytes).expect("decode should succeed");
        assert!(restored.water_sources.is_none());
    }

    #[test]
    fn test_migrate_from_v4() {
        let mut grid = WorldGrid::new(4, 4);
        simulation::terrain::generate_terrain(&mut grid, 42);
        let roads = RoadNetwork::default();
        let clock = GameClock::default();
        let budget = CityBudget::default();
        let demand = ZoneDemand::default();

        let mut save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &[],
            &[],
            &[],
            &[],
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        save.version = 4;

        let old = migrate_save(&mut save);
        assert_eq!(old, 4);
        assert_eq!(save.version, CURRENT_SAVE_VERSION);
    }
}
