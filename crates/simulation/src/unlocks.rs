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
                // Starter unlocks (always available)
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
    // Starter (free)
    BasicRoads,
    ResidentialZoning,
    CommercialZoning,
    IndustrialZoning,
    BasicPower,    // PowerPlant
    BasicWater,    // WaterTower

    // Tier 1 (1 DP each, unlock at 500 pop)
    FireService,
    PoliceService,
    ElementaryEducation,
    SmallParks,
    BasicSanitation,  // Landfill

    // Tier 2 (2 DP each, unlock at 2000 pop)
    BasicHeating,     // HeatingBoiler
    HealthCare,       // Hospital
    HighSchoolEducation,
    HighDensityResidential,
    HighDensityCommercial,
    SolarPower,
    SewagePlant,
    AdvancedParks,    // Large parks, playground, sports
    DeathCare,        // Cemetery, Crematorium

    DistrictHeatingNetwork, // DistrictHeatingPlant, GeothermalPlant

    // Tier 3 (3 DP each, unlock at 10000 pop)
    OfficeZoning,
    UniversityEducation,
    WindPower,
    AdvancedSanitation, // Recycling, Incinerator
    PublicTransport,    // BusDepot, TrainStation
    Entertainment,      // Stadium, Plaza
    AdvancedEmergency,  // FireHQ, PoliceHQ, Prison, MedicalCenter
    Telecom,            // CellTower, DataCenter
    AdvancedTransport,  // Subway, Tram, Ferry
    SmallAirstrips,     // SmallAirstrip (5K pop)
    PostalService,      // PostOffice, MailSortingCenter
    WaterInfrastructure, // WaterTreatmentPlant, WellPump

    // Tier 4 (5 DP each, unlock at 50000 pop)
    Landmarks,        // CityHall, Museum, Cathedral, TVStation
    PolicySystem,     // Enables policies
    NuclearPower,     // NuclearPlant
    RegionalAirports,   // RegionalAirport (20K pop)

    // Tier 5 (7 DP, unlock at 100000 pop)
    InternationalAirports, // InternationalAirport (100K pop)
}

impl UnlockNode {
    pub fn cost(self) -> u32 {
        match self {
            UnlockNode::BasicRoads | UnlockNode::ResidentialZoning |
            UnlockNode::CommercialZoning | UnlockNode::IndustrialZoning |
            UnlockNode::BasicPower | UnlockNode::BasicWater => 0,

            UnlockNode::FireService | UnlockNode::PoliceService |
            UnlockNode::ElementaryEducation | UnlockNode::SmallParks |
            UnlockNode::BasicSanitation => 1,

            UnlockNode::BasicHeating |
            UnlockNode::HealthCare | UnlockNode::HighSchoolEducation |
            UnlockNode::HighDensityResidential | UnlockNode::HighDensityCommercial |
            UnlockNode::SolarPower | UnlockNode::SewagePlant |
            UnlockNode::AdvancedParks | UnlockNode::DeathCare => 2,

            UnlockNode::DistrictHeatingNetwork |
            UnlockNode::OfficeZoning | UnlockNode::UniversityEducation |
            UnlockNode::WindPower | UnlockNode::AdvancedSanitation |
            UnlockNode::PublicTransport | UnlockNode::Entertainment |
            UnlockNode::AdvancedEmergency | UnlockNode::Telecom |
            UnlockNode::AdvancedTransport |
            UnlockNode::SmallAirstrips |
            UnlockNode::PostalService |
            UnlockNode::WaterInfrastructure => 3,

            UnlockNode::Landmarks | UnlockNode::PolicySystem |
            UnlockNode::NuclearPower |
            UnlockNode::RegionalAirports => 5,

            UnlockNode::InternationalAirports => 7,
        }
    }

    pub fn required_population(self) -> u32 {
        match self {
            UnlockNode::BasicRoads | UnlockNode::ResidentialZoning |
            UnlockNode::CommercialZoning | UnlockNode::IndustrialZoning |
            UnlockNode::BasicPower | UnlockNode::BasicWater => 0,

            UnlockNode::FireService | UnlockNode::PoliceService |
            UnlockNode::ElementaryEducation | UnlockNode::SmallParks |
            UnlockNode::BasicSanitation => 500,

            UnlockNode::BasicHeating |
            UnlockNode::HealthCare | UnlockNode::HighSchoolEducation |
            UnlockNode::HighDensityResidential | UnlockNode::HighDensityCommercial |
            UnlockNode::SolarPower | UnlockNode::SewagePlant |
            UnlockNode::AdvancedParks | UnlockNode::DeathCare => 2_000,

            UnlockNode::DistrictHeatingNetwork |
            UnlockNode::OfficeZoning | UnlockNode::UniversityEducation |
            UnlockNode::WindPower | UnlockNode::AdvancedSanitation |
            UnlockNode::PublicTransport | UnlockNode::Entertainment |
            UnlockNode::AdvancedEmergency | UnlockNode::Telecom |
            UnlockNode::AdvancedTransport |
            UnlockNode::SmallAirstrips |
            UnlockNode::PostalService |
            UnlockNode::WaterInfrastructure => 5_000,

            UnlockNode::RegionalAirports => 20_000,

            UnlockNode::Landmarks | UnlockNode::PolicySystem |
            UnlockNode::NuclearPower => 50_000,

            UnlockNode::InternationalAirports => 100_000,
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
            UnlockNode::BasicRoads, UnlockNode::ResidentialZoning,
            UnlockNode::CommercialZoning, UnlockNode::IndustrialZoning,
            UnlockNode::BasicPower, UnlockNode::BasicWater,
            UnlockNode::FireService, UnlockNode::PoliceService,
            UnlockNode::ElementaryEducation, UnlockNode::SmallParks,
            UnlockNode::BasicSanitation,
            UnlockNode::HealthCare, UnlockNode::HighSchoolEducation,
            UnlockNode::HighDensityResidential, UnlockNode::HighDensityCommercial,
            UnlockNode::SolarPower, UnlockNode::SewagePlant,
            UnlockNode::AdvancedParks, UnlockNode::DeathCare,
            UnlockNode::BasicHeating, UnlockNode::DistrictHeatingNetwork,
            UnlockNode::OfficeZoning, UnlockNode::UniversityEducation,
            UnlockNode::WindPower, UnlockNode::AdvancedSanitation,
            UnlockNode::PublicTransport, UnlockNode::Entertainment,
            UnlockNode::AdvancedEmergency, UnlockNode::Telecom,
            UnlockNode::AdvancedTransport,
            UnlockNode::SmallAirstrips,
            UnlockNode::PostalService,
            UnlockNode::WaterInfrastructure,
            UnlockNode::Landmarks, UnlockNode::PolicySystem,
            UnlockNode::NuclearPower,
            UnlockNode::RegionalAirports,
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
            ServiceType::FireStation | ServiceType::FireHouse => self.is_unlocked(UnlockNode::FireService),
            ServiceType::FireHQ => self.is_unlocked(UnlockNode::AdvancedEmergency),
            ServiceType::PoliceStation | ServiceType::PoliceKiosk => self.is_unlocked(UnlockNode::PoliceService),
            ServiceType::PoliceHQ | ServiceType::Prison => self.is_unlocked(UnlockNode::AdvancedEmergency),
            ServiceType::Hospital => self.is_unlocked(UnlockNode::HealthCare),
            ServiceType::MedicalClinic => self.is_unlocked(UnlockNode::HealthCare),
            ServiceType::MedicalCenter => self.is_unlocked(UnlockNode::AdvancedEmergency),
            ServiceType::ElementarySchool | ServiceType::Library => self.is_unlocked(UnlockNode::ElementaryEducation),
            ServiceType::Kindergarten => self.is_unlocked(UnlockNode::ElementaryEducation),
            ServiceType::HighSchool => self.is_unlocked(UnlockNode::HighSchoolEducation),
            ServiceType::University => self.is_unlocked(UnlockNode::UniversityEducation),
            ServiceType::SmallPark | ServiceType::Playground => self.is_unlocked(UnlockNode::SmallParks),
            ServiceType::LargePark | ServiceType::SportsField => self.is_unlocked(UnlockNode::AdvancedParks),
            ServiceType::Plaza | ServiceType::Stadium => self.is_unlocked(UnlockNode::Entertainment),
            ServiceType::Landfill => self.is_unlocked(UnlockNode::BasicSanitation),
            ServiceType::RecyclingCenter | ServiceType::Incinerator => self.is_unlocked(UnlockNode::AdvancedSanitation),
            ServiceType::TransferStation => self.is_unlocked(UnlockNode::BasicSanitation),
            ServiceType::Cemetery | ServiceType::Crematorium => self.is_unlocked(UnlockNode::DeathCare),
            ServiceType::CityHall | ServiceType::Museum |
            ServiceType::Cathedral | ServiceType::TVStation => self.is_unlocked(UnlockNode::Landmarks),
            ServiceType::BusDepot | ServiceType::TrainStation => self.is_unlocked(UnlockNode::PublicTransport),
            ServiceType::SubwayStation | ServiceType::TramDepot |
            ServiceType::FerryPier => self.is_unlocked(UnlockNode::AdvancedTransport),
            ServiceType::SmallAirstrip => self.is_unlocked(UnlockNode::SmallAirstrips),
            ServiceType::RegionalAirport => self.is_unlocked(UnlockNode::RegionalAirports),
            ServiceType::InternationalAirport => self.is_unlocked(UnlockNode::InternationalAirports),
            ServiceType::CellTower | ServiceType::DataCenter => self.is_unlocked(UnlockNode::Telecom),
            ServiceType::HomelessShelter => self.is_unlocked(UnlockNode::HealthCare),
            ServiceType::PostOffice | ServiceType::MailSortingCenter => self.is_unlocked(UnlockNode::PostalService),
            ServiceType::WaterTreatmentPlant | ServiceType::WellPump => self.is_unlocked(UnlockNode::WaterInfrastructure),
            ServiceType::WelfareOffice => self.is_unlocked(UnlockNode::HealthCare),
            ServiceType::HeatingBoiler => self.is_unlocked(UnlockNode::BasicHeating),
            ServiceType::DistrictHeatingPlant |
            ServiceType::GeothermalPlant => self.is_unlocked(UnlockNode::DistrictHeatingNetwork),
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
            UtilityType::NuclearPlant => self.is_unlocked(UnlockNode::NuclearPower),
            UtilityType::Geothermal => self.is_unlocked(UnlockNode::WindPower),
            UtilityType::PumpingStation => self.is_unlocked(UnlockNode::BasicWater),
            UtilityType::WaterTreatment => self.is_unlocked(UnlockNode::SewagePlant),
        }
    }
}

/// Award development points at population milestones
pub fn award_development_points(
    stats: Res<crate::stats::CityStats>,
    mut unlocks: ResMut<UnlockState>,
) {
    const MILESTONES: &[(u32, u32)] = &[
        (500, 2),     // +2 DP at 500 pop
        (1_000, 2),
        (2_000, 3),
        (5_000, 3),
        (10_000, 4),
        (25_000, 4),
        (50_000, 5),
        (100_000, 5),
        (250_000, 6),
        (500_000, 8),
        (1_000_000, 10),
    ];

    for &(pop_threshold, points) in MILESTONES {
        if stats.population >= pop_threshold && unlocks.last_milestone_pop < pop_threshold {
            unlocks.development_points += points;
            unlocks.last_milestone_pop = pop_threshold;
        }
    }
}
