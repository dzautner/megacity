//! Placement-side unlock guard: rejects locked tools before they can modify
//! the world. This is a safety net in case the UI-side graying-out is
//! bypassed (e.g. via keyboard shortcuts or future hotkey bindings).

use simulation::unlocks::{UnlockNode, UnlockState};
use simulation::utilities::UtilityType;

use super::types::ActiveTool;

/// Returns `true` if the active tool is gated by an unlock that has not yet
/// been purchased. Always-available tools (roads, basic zones, terrain,
/// districts, bulldoze, inspect, etc.) return `false`.
pub(crate) fn is_tool_locked(tool: &ActiveTool, unlocks: &UnlockState) -> bool {
    // Utility placement tools
    if let Some(ut) = tool_to_utility_type(tool) {
        return !unlocks.is_utility_unlocked(ut);
    }
    // Service placement tools
    if let Some(st) = tool.service_type() {
        return !unlocks.is_service_unlocked(st);
    }
    // Zone tools with unlock requirements
    match tool {
        ActiveTool::ZoneResidentialHigh => {
            !unlocks.is_unlocked(UnlockNode::HighDensityResidential)
        }
        ActiveTool::ZoneCommercialHigh => {
            !unlocks.is_unlocked(UnlockNode::HighDensityCommercial)
        }
        ActiveTool::ZoneOffice => !unlocks.is_unlocked(UnlockNode::OfficeZoning),
        _ => false,
    }
}

/// Maps an ActiveTool to its UtilityType for unlock checking.
fn tool_to_utility_type(tool: &ActiveTool) -> Option<UtilityType> {
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
