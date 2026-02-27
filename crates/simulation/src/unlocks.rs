use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::services::ServiceType;
use crate::utilities::UtilityType;

/// Milestone-based development points and unlock tree
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct UnlockState {
    pub development_points: u32,
    pub spent_points: u32,
    pub unlocked_nodes: Vec<UnlockNode>,
    pub last_milestone_pop: u32,
}

impl Default for UnlockState {
    fn default() -> Self {
        Self {
            development_points: 3, // Start with 3 DP for basic buildings
            spent_points: 0,
            unlocked_nodes: vec![
                // Starter unlocks (always available — Hamlet tier)
                UnlockNode::BasicRoads,
                UnlockNode::ResidentialZoning,
                UnlockNode::CommercialZoning,
                UnlockNode::IndustrialZoning,
                UnlockNode::BasicPower,
                UnlockNode::BasicWater,
            ],
            last_milestone_pop: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnlockNode {
    // Tier 0 — Hamlet (0 pop, starter)
    BasicRoads,
    ResidentialZoning,
    CommercialZoning,
    IndustrialZoning,
    BasicPower,
    BasicWater,

    // Tier 1 — Small Settlement (240 pop)
    HealthCare,
    DeathCare,
    BasicSanitation,

    // Tier 2 — Village (1,200 pop)
    FireService,
    PoliceService,
    ElementaryEducation,

    // Tier 3 — Large Village (2,600 pop)
    HighSchoolEducation,
    SmallParks,
    PolicySystem,

    // Tier 4 — Town (5,000 pop)
    PublicTransport,
    Landmarks,

    // Tier 5 — Large Town (7,500 pop)
    HighDensityResidential,
    HighDensityCommercial,
    AdvancedTransport,
    OfficeZoning,

    // Tier 6 — Small City (12,000 pop)
    UniversityEducation,
    AdvancedSanitation,
    PostalService,

    // Tier 7 — City (20,000 pop)
    SmallAirstrips,
    AdvancedParks,
    WaterInfrastructure,

    // Tier 8 — Large City (36,000 pop)
    Telecom,
    Entertainment,
    BasicHeating,

    // Tier 9 — Metropolis (50,000 pop)
    RegionalAirports,
    SolarPower,
    WindPower,
    SewagePlant,

    // Tier 10 — Large Metropolis (65,000 pop)
    AdvancedEmergency,
    DistrictHeatingNetwork,
    NuclearPower,

    // Tier 11 — Megalopolis (80,000 pop)
    InternationalAirports,
}

impl UnlockNode {
    pub fn cost(self) -> u32 {
        match self {
            // Tier 0 — free starters
            UnlockNode::BasicRoads
            | UnlockNode::ResidentialZoning
            | UnlockNode::CommercialZoning
            | UnlockNode::IndustrialZoning
            | UnlockNode::BasicPower
            | UnlockNode::BasicWater => 0,

            // Tier 1 (240 pop)
            UnlockNode::HealthCare
            | UnlockNode::DeathCare
            | UnlockNode::BasicSanitation => 1,

            // Tier 2 (1,200 pop)
            UnlockNode::FireService
            | UnlockNode::PoliceService
            | UnlockNode::ElementaryEducation => 1,

            // Tier 3 (2,600 pop)
            UnlockNode::HighSchoolEducation
            | UnlockNode::SmallParks
            | UnlockNode::PolicySystem => 2,

            // Tier 4 (5,000 pop)
            UnlockNode::PublicTransport
            | UnlockNode::Landmarks => 2,

            // Tier 5 (7,500 pop)
            UnlockNode::HighDensityResidential
            | UnlockNode::HighDensityCommercial
            | UnlockNode::AdvancedTransport
            | UnlockNode::OfficeZoning => 3,

            // Tier 6 (12,000 pop)
            UnlockNode::UniversityEducation
            | UnlockNode::AdvancedSanitation
            | UnlockNode::PostalService => 3,

            // Tier 7 (20,000 pop)
            UnlockNode::SmallAirstrips
            | UnlockNode::AdvancedParks
            | UnlockNode::WaterInfrastructure => 3,

            // Tier 8 (36,000 pop)
            UnlockNode::Telecom
            | UnlockNode::Entertainment
            | UnlockNode::BasicHeating => 4,

            // Tier 9 (50,000 pop)
            UnlockNode::RegionalAirports
            | UnlockNode::SolarPower
            | UnlockNode::WindPower
            | UnlockNode::SewagePlant => 4,

            // Tier 10 (65,000 pop)
            UnlockNode::AdvancedEmergency
            | UnlockNode::DistrictHeatingNetwork
            | UnlockNode::NuclearPower => 5,

            // Tier 11 (80,000 pop)
            UnlockNode::InternationalAirports => 7,
        }
    }

    /// Population threshold aligned with the 12-tier milestone system.
    pub fn required_population(self) -> u32 {
        match self {
            // Tier 0 — Hamlet
            UnlockNode::BasicRoads
            | UnlockNode::ResidentialZoning
            | UnlockNode::CommercialZoning
            | UnlockNode::IndustrialZoning
            | UnlockNode::BasicPower
            | UnlockNode::BasicWater => 0,

            // Tier 1 — Small Settlement
            UnlockNode::HealthCare
            | UnlockNode::DeathCare
            | UnlockNode::BasicSanitation => 240,

            // Tier 2 — Village
            UnlockNode::FireService
            | UnlockNode::PoliceService
            | UnlockNode::ElementaryEducation => 1_200,

            // Tier 3 — Large Village
            UnlockNode::HighSchoolEducation
            | UnlockNode::SmallParks
            | UnlockNode::PolicySystem => 2_600,

            // Tier 4 — Town
            UnlockNode::PublicTransport
            | UnlockNode::Landmarks => 5_000,

            // Tier 5 — Large Town
            UnlockNode::HighDensityResidential
            | UnlockNode::HighDensityCommercial
            | UnlockNode::AdvancedTransport
            | UnlockNode::OfficeZoning => 7_500,

            // Tier 6 — Small City
            UnlockNode::UniversityEducation
            | UnlockNode::AdvancedSanitation
            | UnlockNode::PostalService => 12_000,

            // Tier 7 — City
            UnlockNode::SmallAirstrips
            | UnlockNode::AdvancedParks
            | UnlockNode::WaterInfrastructure => 20_000,

            // Tier 8 — Large City
            UnlockNode::Telecom
            | UnlockNode::Entertainment
            | UnlockNode::BasicHeating => 36_000,

            // Tier 9 — Metropolis
            UnlockNode::RegionalAirports
            | UnlockNode::SolarPower
            | UnlockNode::WindPower
            | UnlockNode::SewagePlant => 50_000,

            // Tier 10 — Large Metropolis
            UnlockNode::AdvancedEmergency
            | UnlockNode::DistrictHeatingNetwork
            | UnlockNode::NuclearPower => 65_000,

            // Tier 11 — Megalopolis
            UnlockNode::InternationalAirports => 80_000,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            UnlockNode::BasicRoads => "Basic Roads",
            UnlockNode::ResidentialZoning => "Residential Zoning",
            UnlockNode::CommercialZoning => "Commercial Zoning",
            UnlockNode::IndustrialZoning => "Industrial Zoning",
            UnlockNode::BasicPower => "Power Plant",
            UnlockNode::BasicWater => "Water Tower",
            UnlockNode::FireService => "Fire Service",
            UnlockNode::PoliceService => "Police Service",
            UnlockNode::ElementaryEducation => "Elementary Education",
            UnlockNode::SmallParks => "Small Parks",
            UnlockNode::BasicSanitation => "Basic Sanitation",
            UnlockNode::HealthCare => "Healthcare",
            UnlockNode::HighSchoolEducation => "High School",
            UnlockNode::HighDensityResidential => "High-Density Residential",
            UnlockNode::HighDensityCommercial => "High-Density Commercial",
            UnlockNode::SolarPower => "Solar Power",
            UnlockNode::SewagePlant => "Sewage Plant",
            UnlockNode::AdvancedParks => "Advanced Parks",
            UnlockNode::DeathCare => "Death Care",
            UnlockNode::BasicHeating => "Basic Heating",
            UnlockNode::DistrictHeatingNetwork => "District Heating",
            UnlockNode::OfficeZoning => "Office Zoning",
            UnlockNode::UniversityEducation => "University",
            UnlockNode::WindPower => "Wind Power",
            UnlockNode::AdvancedSanitation => "Advanced Sanitation",
            UnlockNode::PublicTransport => "Public Transport",
            UnlockNode::Entertainment => "Entertainment",
            UnlockNode::AdvancedEmergency => "Advanced Emergency",
            UnlockNode::Telecom => "Telecommunications",
            UnlockNode::AdvancedTransport => "Advanced Transport",
            UnlockNode::SmallAirstrips => "Small Airstrips",
            UnlockNode::PostalService => "Postal Service",
            UnlockNode::WaterInfrastructure => "Water Infrastructure",
            UnlockNode::RegionalAirports => "Regional Airports",
            UnlockNode::InternationalAirports => "International Airports",
            UnlockNode::Landmarks => "Landmarks",
            UnlockNode::PolicySystem => "City Policies",
            UnlockNode::NuclearPower => "Nuclear Power",
        }
    }

    pub fn all() -> &'static [UnlockNode] {
        &[
            UnlockNode::BasicRoads,
            UnlockNode::ResidentialZoning,
            UnlockNode::CommercialZoning,
            UnlockNode::IndustrialZoning,
            UnlockNode::BasicPower,
            UnlockNode::BasicWater,
            UnlockNode::HealthCare,
            UnlockNode::DeathCare,
            UnlockNode::BasicSanitation,
            UnlockNode::FireService,
            UnlockNode::PoliceService,
            UnlockNode::ElementaryEducation,
            UnlockNode::HighSchoolEducation,
            UnlockNode::SmallParks,
            UnlockNode::PolicySystem,
            UnlockNode::PublicTransport,
            UnlockNode::Landmarks,
            UnlockNode::HighDensityResidential,
            UnlockNode::HighDensityCommercial,
            UnlockNode::AdvancedTransport,
            UnlockNode::OfficeZoning,
            UnlockNode::UniversityEducation,
            UnlockNode::AdvancedSanitation,
            UnlockNode::PostalService,
            UnlockNode::SmallAirstrips,
            UnlockNode::AdvancedParks,
            UnlockNode::WaterInfrastructure,
            UnlockNode::Telecom,
            UnlockNode::Entertainment,
            UnlockNode::BasicHeating,
            UnlockNode::RegionalAirports,
            UnlockNode::SolarPower,
            UnlockNode::WindPower,
            UnlockNode::SewagePlant,
            UnlockNode::AdvancedEmergency,
            UnlockNode::DistrictHeatingNetwork,
            UnlockNode::NuclearPower,
            UnlockNode::InternationalAirports,
        ]
    }
}

impl UnlockState {
    pub fn is_unlocked(&self, node: UnlockNode) -> bool {
        self.unlocked_nodes.contains(&node)
    }

    pub fn can_purchase(&self, node: UnlockNode, population: u32) -> bool {
        !self.is_unlocked(node)
            && population >= node.required_population()
            && self.available_points() >= node.cost()
    }

    pub fn available_points(&self) -> u32 {
        self.development_points.saturating_sub(self.spent_points)
    }

    pub fn purchase(&mut self, node: UnlockNode) -> bool {
        let cost = node.cost();
        if self.available_points() < cost || self.is_unlocked(node) {
            return false;
        }
        self.spent_points += cost;
        self.unlocked_nodes.push(node);
        true
    }

    /// Check if a specific service type is unlocked
    pub fn is_service_unlocked(&self, service_type: ServiceType) -> bool {
        match service_type {
            ServiceType::FireStation | ServiceType::FireHouse => {
                self.is_unlocked(UnlockNode::FireService)
            }
            ServiceType::FireHQ => self.is_unlocked(UnlockNode::AdvancedEmergency),
            ServiceType::PoliceStation | ServiceType::PoliceKiosk => {
                self.is_unlocked(UnlockNode::PoliceService)
            }
            ServiceType::PoliceHQ | ServiceType::Prison => {
                self.is_unlocked(UnlockNode::AdvancedEmergency)
            }
            ServiceType::Hospital => self.is_unlocked(UnlockNode::HealthCare),
            ServiceType::MedicalClinic => self.is_unlocked(UnlockNode::HealthCare),
            ServiceType::MedicalCenter => self.is_unlocked(UnlockNode::AdvancedEmergency),
            ServiceType::ElementarySchool | ServiceType::Library => {
                self.is_unlocked(UnlockNode::ElementaryEducation)
            }
            ServiceType::Kindergarten => {
                self.is_unlocked(UnlockNode::ElementaryEducation)
            }
            ServiceType::HighSchool => {
                self.is_unlocked(UnlockNode::HighSchoolEducation)
            }
            ServiceType::University => {
                self.is_unlocked(UnlockNode::UniversityEducation)
            }
            ServiceType::SmallPark | ServiceType::Playground => {
                self.is_unlocked(UnlockNode::SmallParks)
            }
            ServiceType::LargePark | ServiceType::SportsField => {
                self.is_unlocked(UnlockNode::AdvancedParks)
            }
            ServiceType::Plaza | ServiceType::Stadium => {
                self.is_unlocked(UnlockNode::Entertainment)
            }
            ServiceType::Landfill => self.is_unlocked(UnlockNode::BasicSanitation),
            ServiceType::RecyclingCenter | ServiceType::Incinerator => {
                self.is_unlocked(UnlockNode::AdvancedSanitation)
            }
            ServiceType::TransferStation => {
                self.is_unlocked(UnlockNode::BasicSanitation)
            }
            ServiceType::Cemetery | ServiceType::Crematorium => {
                self.is_unlocked(UnlockNode::DeathCare)
            }
            ServiceType::CityHall
            | ServiceType::Museum
            | ServiceType::Cathedral
            | ServiceType::TVStation => self.is_unlocked(UnlockNode::Landmarks),
            ServiceType::BusDepot | ServiceType::TrainStation => {
                self.is_unlocked(UnlockNode::PublicTransport)
            }
            ServiceType::SubwayStation
            | ServiceType::TramDepot
            | ServiceType::FerryPier => {
                self.is_unlocked(UnlockNode::AdvancedTransport)
            }
            ServiceType::SmallAirstrip => {
                self.is_unlocked(UnlockNode::SmallAirstrips)
            }
            ServiceType::RegionalAirport => {
                self.is_unlocked(UnlockNode::RegionalAirports)
            }
            ServiceType::InternationalAirport => {
                self.is_unlocked(UnlockNode::InternationalAirports)
            }
            ServiceType::CellTower | ServiceType::DataCenter => {
                self.is_unlocked(UnlockNode::Telecom)
            }
            ServiceType::HomelessShelter => {
                self.is_unlocked(UnlockNode::HealthCare)
            }
            ServiceType::PostOffice | ServiceType::MailSortingCenter => {
                self.is_unlocked(UnlockNode::PostalService)
            }
            ServiceType::WaterTreatmentPlant | ServiceType::WellPump => {
                self.is_unlocked(UnlockNode::WaterInfrastructure)
            }
            ServiceType::WelfareOffice => {
                self.is_unlocked(UnlockNode::HealthCare)
            }
            ServiceType::HeatingBoiler => {
                self.is_unlocked(UnlockNode::BasicHeating)
            }
            ServiceType::DistrictHeatingPlant
            | ServiceType::GeothermalPlant => {
                self.is_unlocked(UnlockNode::DistrictHeatingNetwork)
            }
            ServiceType::Daycare | ServiceType::Eldercare => {
                self.is_unlocked(UnlockNode::HealthCare)
            }
            ServiceType::CommunityCenter
            | ServiceType::SubstanceAbuseTreatmentCenter
            | ServiceType::SeniorCenter
            | ServiceType::YouthCenter => self.is_unlocked(UnlockNode::HealthCare),
        }
    }

    /// Check if a utility type is unlocked
    pub fn is_utility_unlocked(&self, utility_type: UtilityType) -> bool {
        match utility_type {
            UtilityType::PowerPlant => self.is_unlocked(UnlockNode::BasicPower),
            UtilityType::SolarFarm => self.is_unlocked(UnlockNode::SolarPower),
            UtilityType::WindTurbine => self.is_unlocked(UnlockNode::WindPower),
            UtilityType::WaterTower => self.is_unlocked(UnlockNode::BasicWater),
            UtilityType::SewagePlant => self.is_unlocked(UnlockNode::SewagePlant),
            UtilityType::NuclearPlant => {
                self.is_unlocked(UnlockNode::NuclearPower)
            }
            UtilityType::Geothermal => self.is_unlocked(UnlockNode::WindPower),
            UtilityType::PumpingStation => {
                self.is_unlocked(UnlockNode::BasicWater)
            }
            UtilityType::HydroDam => self.is_unlocked(UnlockNode::BasicPower),
            UtilityType::WaterTreatment => {
                self.is_unlocked(UnlockNode::SewagePlant)
            }
            UtilityType::OilPlant => self.is_unlocked(UnlockNode::BasicPower),
            UtilityType::GasPlant => self.is_unlocked(UnlockNode::BasicPower),
        }
    }
}

pub struct UnlocksPlugin;

impl Plugin for UnlocksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UnlockState>();
        // NOTE: Milestone-based DP awards and auto-unlocks are now handled
        // by MilestonesPlugin in milestones.rs. The old `award_development_points`
        // system has been removed.
    }
}
