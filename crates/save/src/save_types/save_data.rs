// ---------------------------------------------------------------------------
// SaveData: the top-level save file struct
// ---------------------------------------------------------------------------

use std::collections::BTreeMap;

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::core_types::*;
use super::infrastructure_types::*;
use super::policy_types::*;

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
    pub stormwater_grid: Option<SaveStormwaterGrid>,
    #[serde(default)]
    pub water_sources: Option<Vec<SaveWaterSource>>,
    #[serde(default)]
    pub degree_days: Option<SaveDegreeDays>,
    #[serde(default)]
    pub construction_modifiers: Option<SaveConstructionModifiers>,
    #[serde(default)]
    pub recycling_state: Option<SaveRecyclingState>,
    #[serde(default)]
    pub wind_damage_state: Option<SaveWindDamageState>,
    #[serde(default)]
    pub uhi_grid: Option<SaveUhiGrid>,
    #[serde(default)]
    pub drought_state: Option<SaveDroughtState>,
    #[serde(default)]
    pub heat_wave_state: Option<SaveHeatWaveState>,
    #[serde(default)]
    pub composting_state: Option<SaveCompostingState>,
    #[serde(default)]
    pub cold_snap_state: Option<SaveColdSnapState>,
    #[serde(default)]
    pub water_treatment_state: Option<SaveWaterTreatmentState>,
    #[serde(default)]
    pub groundwater_depletion_state: Option<SaveGroundwaterDepletionState>,
    #[serde(default)]
    pub wastewater_state: Option<SaveWastewaterState>,
    #[serde(default)]
    pub hazardous_waste_state: Option<SaveHazardousWasteState>,
    #[serde(default)]
    pub storm_drainage_state: Option<SaveStormDrainageState>,
    #[serde(default)]
    pub landfill_capacity_state: Option<SaveLandfillCapacityState>,
    #[serde(default)]
    pub flood_state: Option<SaveFloodState>,
    #[serde(default)]
    pub reservoir_state: Option<SaveReservoirState>,
    #[serde(default)]
    pub landfill_gas_state: Option<SaveLandfillGasState>,
    #[serde(default)]
    pub cso_state: Option<SaveCsoState>,
    #[serde(default)]
    pub water_conservation_state: Option<SaveWaterConservationState>,
    #[serde(default)]
    pub fog_state: Option<SaveFogState>,
    #[serde(default)]
    pub urban_growth_boundary: Option<SaveUrbanGrowthBoundary>,
    #[serde(default)]
    pub snow_state: Option<SaveSnowState>,
    #[serde(default)]
    pub agriculture_state: Option<SaveAgricultureState>,
    // --- Extension map for dynamic feature persistence (no save-file changes needed) ---
    /// Generic extension map: each key is a `Saveable::SAVE_KEY`, value is bitcode-encoded bytes.
    /// New features use this instead of adding named fields above.
    #[serde(default)]
    pub extensions: BTreeMap<String, Vec<u8>>,
}

impl SaveData {
    pub fn encode(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, bitcode::Error> {
        bitcode::decode(bytes)
    }
}
