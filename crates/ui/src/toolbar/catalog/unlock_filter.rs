//! Maps toolbar items to unlock requirements and checks whether they are
//! available based on the current `UnlockState`.

use rendering::input::ActiveTool;
use simulation::unlocks::{UnlockNode, UnlockState};

/// Returns `true` if the given tool is available to the player.
///
/// Tools that are always available (roads, basic zones, terrain, districts,
/// environment, overlays, bulldoze, inspect) return `true` unconditionally.
/// Service and utility placement tools delegate to `UnlockState`.
pub fn is_tool_unlocked(tool: &ActiveTool, unlocks: &UnlockState) -> bool {
    // Check utility tools
    if let Some(ut) = tool_utility_type(tool) {
        return unlocks.is_utility_unlocked(ut);
    }
    // Check service tools
    if let Some(st) = tool.service_type() {
        return unlocks.is_service_unlocked(st);
    }
    // Check zone tools that require unlocks
    match tool {
        ActiveTool::ZoneResidentialHigh => {
            unlocks.is_unlocked(UnlockNode::HighDensityResidential)
        }
        ActiveTool::ZoneCommercialHigh => {
            unlocks.is_unlocked(UnlockNode::HighDensityCommercial)
        }
        ActiveTool::ZoneOffice => unlocks.is_unlocked(UnlockNode::OfficeZoning),
        // All other tools (roads, basic zones, terrain, districts, views,
        // environment, bulldoze, inspect, etc.) are always available.
        _ => true,
    }
}

/// Returns a human-readable hint describing when a locked tool becomes
/// available, e.g. `"Unlocks at Village (pop 1,200)"`.
///
/// Returns `None` for tools that are always available or already unlocked.
pub fn unlock_hint(tool: &ActiveTool, unlocks: &UnlockState) -> Option<String> {
    if is_tool_unlocked(tool, unlocks) {
        return None;
    }
    let node = required_unlock_node(tool)?;
    let pop = node.required_population();
    let tier = tier_name_for_population(pop);
    Some(format!(
        "Unlocks at {} (pop {:>5})",
        tier,
        format_pop_with_commas(pop)
    ))
}

/// Maps an `ActiveTool` to the `UnlockNode` that gates it.
fn required_unlock_node(tool: &ActiveTool) -> Option<UnlockNode> {
    // Utility tools
    match tool {
        ActiveTool::PlacePowerPlant => return Some(UnlockNode::BasicPower),
        ActiveTool::PlaceSolarFarm => return Some(UnlockNode::SolarPower),
        ActiveTool::PlaceWindTurbine => return Some(UnlockNode::WindPower),
        ActiveTool::PlaceNuclearPlant => return Some(UnlockNode::NuclearPower),
        ActiveTool::PlaceGeothermal => return Some(UnlockNode::WindPower),
        ActiveTool::PlaceWaterTower => return Some(UnlockNode::BasicWater),
        ActiveTool::PlaceSewagePlant => return Some(UnlockNode::SewagePlant),
        ActiveTool::PlacePumpingStation => return Some(UnlockNode::BasicWater),
        ActiveTool::PlaceWaterTreatment => return Some(UnlockNode::SewagePlant),
        _ => {}
    }

    // Zone tools
    match tool {
        ActiveTool::ZoneResidentialHigh => {
            return Some(UnlockNode::HighDensityResidential)
        }
        ActiveTool::ZoneCommercialHigh => {
            return Some(UnlockNode::HighDensityCommercial)
        }
        ActiveTool::ZoneOffice => return Some(UnlockNode::OfficeZoning),
        _ => {}
    }

    // Service tools — delegate through service_type mapping
    let st = tool.service_type()?;
    service_to_unlock_node(st)
}

/// Maps a `ServiceType` to its gating `UnlockNode`.
fn service_to_unlock_node(
    st: simulation::services::ServiceType,
) -> Option<UnlockNode> {
    use simulation::services::ServiceType;
    match st {
        ServiceType::FireStation | ServiceType::FireHouse => {
            Some(UnlockNode::FireService)
        }
        ServiceType::FireHQ => Some(UnlockNode::AdvancedEmergency),
        ServiceType::PoliceStation | ServiceType::PoliceKiosk => {
            Some(UnlockNode::PoliceService)
        }
        ServiceType::PoliceHQ | ServiceType::Prison => {
            Some(UnlockNode::AdvancedEmergency)
        }
        ServiceType::Hospital | ServiceType::MedicalClinic => {
            Some(UnlockNode::HealthCare)
        }
        ServiceType::MedicalCenter => Some(UnlockNode::AdvancedEmergency),
        ServiceType::ElementarySchool
        | ServiceType::Library
        | ServiceType::Kindergarten => Some(UnlockNode::ElementaryEducation),
        ServiceType::HighSchool => Some(UnlockNode::HighSchoolEducation),
        ServiceType::University => Some(UnlockNode::UniversityEducation),
        ServiceType::SmallPark | ServiceType::Playground => {
            Some(UnlockNode::SmallParks)
        }
        ServiceType::LargePark | ServiceType::SportsField => {
            Some(UnlockNode::AdvancedParks)
        }
        ServiceType::Plaza | ServiceType::Stadium => {
            Some(UnlockNode::Entertainment)
        }
        ServiceType::Landfill | ServiceType::TransferStation => {
            Some(UnlockNode::BasicSanitation)
        }
        ServiceType::RecyclingCenter | ServiceType::Incinerator => {
            Some(UnlockNode::AdvancedSanitation)
        }
        ServiceType::Cemetery | ServiceType::Crematorium => {
            Some(UnlockNode::DeathCare)
        }
        ServiceType::CityHall
        | ServiceType::Museum
        | ServiceType::Cathedral
        | ServiceType::TVStation => Some(UnlockNode::Landmarks),
        ServiceType::BusDepot | ServiceType::TrainStation => {
            Some(UnlockNode::PublicTransport)
        }
        ServiceType::SubwayStation
        | ServiceType::TramDepot
        | ServiceType::FerryPier => Some(UnlockNode::AdvancedTransport),
        ServiceType::SmallAirstrip => Some(UnlockNode::SmallAirstrips),
        ServiceType::RegionalAirport => Some(UnlockNode::RegionalAirports),
        ServiceType::InternationalAirport => {
            Some(UnlockNode::InternationalAirports)
        }
        ServiceType::CellTower | ServiceType::DataCenter => {
            Some(UnlockNode::Telecom)
        }
        ServiceType::HomelessShelter
        | ServiceType::WelfareOffice
        | ServiceType::Daycare
        | ServiceType::Eldercare
        | ServiceType::CommunityCenter
        | ServiceType::SubstanceAbuseTreatmentCenter
        | ServiceType::SeniorCenter
        | ServiceType::YouthCenter => Some(UnlockNode::HealthCare),
        ServiceType::PostOffice | ServiceType::MailSortingCenter => {
            Some(UnlockNode::PostalService)
        }
        ServiceType::WaterTreatmentPlant | ServiceType::WellPump => {
            Some(UnlockNode::WaterInfrastructure)
        }
        ServiceType::HeatingBoiler => Some(UnlockNode::BasicHeating),
        ServiceType::DistrictHeatingPlant | ServiceType::GeothermalPlant => {
            Some(UnlockNode::DistrictHeatingNetwork)
        }
    }
}

/// Maps a tool to its `UtilityType`, mirroring the catalog's
/// `tool_utility_type` but kept local to avoid cross-module coupling.
fn tool_utility_type(
    tool: &ActiveTool,
) -> Option<simulation::utilities::UtilityType> {
    use simulation::utilities::UtilityType;
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

/// Returns the milestone tier name for a given population threshold.
fn tier_name_for_population(pop: u32) -> &'static str {
    match pop {
        0 => "Hamlet",
        240 => "Small Settlement",
        1_200 => "Village",
        2_600 => "Large Village",
        5_000 => "Town",
        7_500 => "Large Town",
        12_000 => "Township",
        20_000 => "City",
        36_000 => "Grand City",
        50_000 => "Metropolis",
        65_000 => "Conurbation",
        80_000 => "Megalopolis",
        _ => "Unknown",
    }
}

/// Format a population number with comma separators.
fn format_pop_with_commas(pop: u32) -> String {
    let s = pop.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
