// ---------------------------------------------------------------------------
// Infrastructure codecs: Policy, UnlockNode, RecyclingTier, WaterSourceType,
// TreatmentLevel, CompostMethod, LandfillWarningTier, ReservoirWarningTier,
// SewerType
// ---------------------------------------------------------------------------

use simulation::policies::Policy;
use simulation::recycling::RecyclingTier;
use simulation::reservoir::ReservoirWarningTier;
use simulation::unlocks::UnlockNode;
use simulation::water_sources::WaterSourceType;
use simulation::water_treatment::TreatmentLevel;

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
        Policy::EminentDomain => 15,
        Policy::CumulativeZoning => 16,
        Policy::EncourageBiking => 17,
        // New tradeoff policies (issue #613)
        Policy::CombustionEngineBan => 18,
        Policy::SmallBusinessEnthusiast => 19,
        Policy::HeavyTrafficBan => 20,
        Policy::SmokeDetectorDistribution => 21,
        Policy::OldTownHistoric => 22,
        Policy::IndustrialSpacePlanning => 23,
        Policy::RentControl => 24,
        Policy::MinimumWage => 25,
        Policy::TaxIncentiveZone => 26,
        Policy::PetBan => 27,
        Policy::ParksAndRec => 28,
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
        15 => Some(Policy::EminentDomain),
        16 => Some(Policy::CumulativeZoning),
        17 => Some(Policy::EncourageBiking),
        // New tradeoff policies (issue #613)
        18 => Some(Policy::CombustionEngineBan),
        19 => Some(Policy::SmallBusinessEnthusiast),
        20 => Some(Policy::HeavyTrafficBan),
        21 => Some(Policy::SmokeDetectorDistribution),
        22 => Some(Policy::OldTownHistoric),
        23 => Some(Policy::IndustrialSpacePlanning),
        24 => Some(Policy::RentControl),
        25 => Some(Policy::MinimumWage),
        26 => Some(Policy::TaxIncentiveZone),
        27 => Some(Policy::PetBan),
        28 => Some(Policy::ParksAndRec),
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

pub fn treatment_level_to_u8(t: TreatmentLevel) -> u8 {
    match t {
        TreatmentLevel::None => 0,
        TreatmentLevel::Primary => 1,
        TreatmentLevel::Secondary => 2,
        TreatmentLevel::Tertiary => 3,
        TreatmentLevel::Advanced => 4,
    }
}

pub fn u8_to_treatment_level(v: u8) -> TreatmentLevel {
    match v {
        0 => TreatmentLevel::None,
        1 => TreatmentLevel::Primary,
        2 => TreatmentLevel::Secondary,
        3 => TreatmentLevel::Tertiary,
        4 => TreatmentLevel::Advanced,
        _ => TreatmentLevel::None, // fallback
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

pub fn landfill_warning_tier_to_u8(t: simulation::landfill_warning::LandfillWarningTier) -> u8 {
    use simulation::landfill_warning::LandfillWarningTier;
    match t {
        LandfillWarningTier::Normal => 0,
        LandfillWarningTier::Low => 1,
        LandfillWarningTier::Critical => 2,
        LandfillWarningTier::VeryLow => 3,
        LandfillWarningTier::Emergency => 4,
    }
}

pub fn u8_to_landfill_warning_tier(v: u8) -> simulation::landfill_warning::LandfillWarningTier {
    use simulation::landfill_warning::LandfillWarningTier;
    match v {
        0 => LandfillWarningTier::Normal,
        1 => LandfillWarningTier::Low,
        2 => LandfillWarningTier::Critical,
        3 => LandfillWarningTier::VeryLow,
        4 => LandfillWarningTier::Emergency,
        _ => LandfillWarningTier::Normal, // fallback
    }
}

pub fn reservoir_warning_tier_to_u8(tier: ReservoirWarningTier) -> u8 {
    match tier {
        ReservoirWarningTier::Normal => 0,
        ReservoirWarningTier::Watch => 1,
        ReservoirWarningTier::Warning => 2,
        ReservoirWarningTier::Critical => 3,
    }
}

pub fn u8_to_reservoir_warning_tier(val: u8) -> ReservoirWarningTier {
    match val {
        0 => ReservoirWarningTier::Normal,
        1 => ReservoirWarningTier::Watch,
        2 => ReservoirWarningTier::Warning,
        3 => ReservoirWarningTier::Critical,
        _ => ReservoirWarningTier::Normal,
    }
}

pub fn sewer_type_to_u8(st: &simulation::cso::SewerType) -> u8 {
    match st {
        simulation::cso::SewerType::Combined => 0,
        simulation::cso::SewerType::Separated => 1,
    }
}

pub fn u8_to_sewer_type(v: u8) -> simulation::cso::SewerType {
    match v {
        1 => simulation::cso::SewerType::Separated,
        _ => simulation::cso::SewerType::Combined,
    }
}
