// ---------------------------------------------------------------------------
// Encoding helpers
// ---------------------------------------------------------------------------

use simulation::grid::{RoadType, ZoneType};
use simulation::policies::Policy;
use simulation::recycling::RecyclingTier;
use simulation::services::ServiceType;
use simulation::unlocks::UnlockNode;
use simulation::utilities::UtilityType;
use simulation::water_sources::WaterSourceType;
use simulation::weather::{ClimateZone, Season, WeatherCondition, WeatherEvent};

pub fn zone_type_to_u8(z: ZoneType) -> u8 {
    match z {
        ZoneType::None => 0,
        ZoneType::ResidentialLow => 1,
        ZoneType::ResidentialHigh => 2,
        ZoneType::CommercialLow => 3,
        ZoneType::CommercialHigh => 4,
        ZoneType::Industrial => 5,
        ZoneType::Office => 6,
        ZoneType::ResidentialMedium => 7,
        ZoneType::MixedUse => 8,
    }
}

pub fn u8_to_zone_type(v: u8) -> ZoneType {
    match v {
        1 => ZoneType::ResidentialLow,
        2 => ZoneType::ResidentialHigh,
        3 => ZoneType::CommercialLow,
        4 => ZoneType::CommercialHigh,
        5 => ZoneType::Industrial,
        6 => ZoneType::Office,
        7 => ZoneType::ResidentialMedium,
        8 => ZoneType::MixedUse,
        _ => ZoneType::None,
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

pub fn climate_zone_to_u8(z: ClimateZone) -> u8 {
    match z {
        ClimateZone::Temperate => 0,
        ClimateZone::Tropical => 1,
        ClimateZone::Arid => 2,
        ClimateZone::Mediterranean => 3,
        ClimateZone::Continental => 4,
        ClimateZone::Subarctic => 5,
        ClimateZone::Oceanic => 6,
    }
}

pub fn u8_to_climate_zone(v: u8) -> ClimateZone {
    match v {
        0 => ClimateZone::Temperate,
        1 => ClimateZone::Tropical,
        2 => ClimateZone::Arid,
        3 => ClimateZone::Mediterranean,
        4 => ClimateZone::Continental,
        5 => ClimateZone::Subarctic,
        6 => ClimateZone::Oceanic,
        _ => ClimateZone::Temperate, // fallback
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

pub fn recycling_tier_to_u8(t: RecyclingTier) -> u8 {
    match t {
        RecyclingTier::None => 0,
        RecyclingTier::VoluntaryDropoff => 1,
        RecyclingTier::CurbsideBasic => 2,
        RecyclingTier::CurbsideSort => 3,
        RecyclingTier::SingleStream => 4,
        RecyclingTier::PayAsYouThrow => 5,
        RecyclingTier::ZeroWaste => 6,
    }
}

pub fn u8_to_recycling_tier(v: u8) -> RecyclingTier {
    match v {
        0 => RecyclingTier::None,
        1 => RecyclingTier::VoluntaryDropoff,
        2 => RecyclingTier::CurbsideBasic,
        3 => RecyclingTier::CurbsideSort,
        4 => RecyclingTier::SingleStream,
        5 => RecyclingTier::PayAsYouThrow,
        6 => RecyclingTier::ZeroWaste,
        _ => RecyclingTier::None, // fallback
    }
}

pub fn wind_damage_tier_to_u8(t: simulation::wind_damage::WindDamageTier) -> u8 {
    use simulation::wind_damage::WindDamageTier;
    match t {
        WindDamageTier::Calm => 0,
        WindDamageTier::Breezy => 1,
        WindDamageTier::Strong => 2,
        WindDamageTier::Gale => 3,
        WindDamageTier::Storm => 4,
        WindDamageTier::Severe => 5,
        WindDamageTier::HurricaneForce => 6,
        WindDamageTier::Extreme => 7,
    }
}

pub fn u8_to_wind_damage_tier(v: u8) -> simulation::wind_damage::WindDamageTier {
    use simulation::wind_damage::WindDamageTier;
    match v {
        0 => WindDamageTier::Calm,
        1 => WindDamageTier::Breezy,
        2 => WindDamageTier::Strong,
        3 => WindDamageTier::Gale,
        4 => WindDamageTier::Storm,
        5 => WindDamageTier::Severe,
        6 => WindDamageTier::HurricaneForce,
        7 => WindDamageTier::Extreme,
        _ => WindDamageTier::Calm, // fallback
    }
}

pub fn drought_tier_to_u8(t: simulation::drought::DroughtTier) -> u8 {
    use simulation::drought::DroughtTier;
    match t {
        DroughtTier::Normal => 0,
        DroughtTier::Moderate => 1,
        DroughtTier::Severe => 2,
        DroughtTier::Extreme => 3,
    }
}

pub fn u8_to_drought_tier(v: u8) -> simulation::drought::DroughtTier {
    use simulation::drought::DroughtTier;
    match v {
        0 => DroughtTier::Normal,
        1 => DroughtTier::Moderate,
        2 => DroughtTier::Severe,
        3 => DroughtTier::Extreme,
        _ => DroughtTier::Normal, // fallback
    }
}

pub fn heat_wave_severity_to_u8(s: simulation::heat_wave::HeatWaveSeverity) -> u8 {
    use simulation::heat_wave::HeatWaveSeverity;
    match s {
        HeatWaveSeverity::None => 0,
        HeatWaveSeverity::Moderate => 1,
        HeatWaveSeverity::Severe => 2,
        HeatWaveSeverity::Extreme => 3,
    }
}

pub fn u8_to_heat_wave_severity(v: u8) -> simulation::heat_wave::HeatWaveSeverity {
    use simulation::heat_wave::HeatWaveSeverity;
    match v {
        0 => HeatWaveSeverity::None,
        1 => HeatWaveSeverity::Moderate,
        2 => HeatWaveSeverity::Severe,
        3 => HeatWaveSeverity::Extreme,
        _ => HeatWaveSeverity::None, // fallback
    }
}

pub fn compost_method_to_u8(m: simulation::composting::CompostMethod) -> u8 {
    use simulation::composting::CompostMethod;
    match m {
        CompostMethod::Windrow => 0,
        CompostMethod::AeratedStaticPile => 1,
        CompostMethod::InVessel => 2,
        CompostMethod::AnaerobicDigestion => 3,
    }
}

pub fn u8_to_compost_method(v: u8) -> simulation::composting::CompostMethod {
    use simulation::composting::CompostMethod;
    match v {
        0 => CompostMethod::Windrow,
        1 => CompostMethod::AeratedStaticPile,
        2 => CompostMethod::InVessel,
        3 => CompostMethod::AnaerobicDigestion,
        _ => CompostMethod::Windrow, // fallback
    }
}

pub fn cold_snap_tier_to_u8(t: simulation::cold_snap::ColdSnapTier) -> u8 {
    use simulation::cold_snap::ColdSnapTier;
    match t {
        ColdSnapTier::Normal => 0,
        ColdSnapTier::Watch => 1,
        ColdSnapTier::Warning => 2,
        ColdSnapTier::Emergency => 3,
    }
}

pub fn u8_to_cold_snap_tier(v: u8) -> simulation::cold_snap::ColdSnapTier {
    use simulation::cold_snap::ColdSnapTier;
    match v {
        0 => ColdSnapTier::Normal,
        1 => ColdSnapTier::Watch,
        2 => ColdSnapTier::Warning,
        3 => ColdSnapTier::Emergency,
        _ => ColdSnapTier::Normal, // fallback
    }
}
