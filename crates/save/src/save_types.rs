// ---------------------------------------------------------------------------
// Save structs and version constants
// ---------------------------------------------------------------------------

use std::collections::BTreeMap;

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use simulation::citizen::{CitizenDetails, CitizenState, PathCache, Position, Velocity};

// ---------------------------------------------------------------------------
// Version constants
// ---------------------------------------------------------------------------

/// Current save file version.
/// v1 = original fields (grid, roads, clock, budget, demand, buildings, citizens, utilities, services, road_segments)
/// v2 = policies, weather, unlock_state, extended_budget, loans
/// v3 = lifecycle_timer, path_cache, velocity per citizen
/// v4 = life_sim_timer (LifeSimTimer serialization)
/// v5 = stormwater_grid (StormwaterGrid serialization)
/// v6 = water_sources (WaterSource component serialization), market-driven zone demand with vacancy rates
/// v7 = degree_days (HDD/CDD tracking for HVAC energy demand)
/// v8 = climate_zone in SaveWeather (ClimateZone resource)
/// v9 = construction_modifiers (ConstructionModifiers serialization)
/// v10 = recycling_state (RecyclingState + RecyclingEconomics serialization)
/// v11 = wind_damage_state (WindDamageState serialization)
/// v12 = uhi_grid (UhiGrid serialization for urban heat island)
/// v13 = drought_state (DroughtState serialization for drought index)
/// v14 = heat_wave_state (HeatWaveState serialization for heat wave effects)
/// v15 = composting_state (CompostingState serialization for composting facilities)
/// v16 = cold_snap_state (ColdSnapState serialization for cold snap effects)
/// v17 = water_treatment_state (WaterTreatmentState serialization for water treatment plants)
/// v18 = groundwater_depletion_state (GroundwaterDepletionState serialization)
/// v19 = wastewater_state (WastewaterState serialization)
/// v20 = hazardous_waste_state (HazardousWasteState serialization)
/// v21 = storm_drainage_state (StormDrainageState serialization for storm drainage infrastructure)
/// v22 = landfill_capacity_state (LandfillCapacityState serialization for landfill warnings)
/// v23 = flood_state (FloodState serialization for urban flooding simulation)
/// v24 = reservoir_state (ReservoirState serialization for reservoir water level tracking)
/// v25 = landfill_gas_state (LandfillGasState serialization for landfill gas collection and energy)
/// v26 = cso_state (SewerSystemState serialization for CSO events)
/// v27 = water_conservation_state (WaterConservationState serialization for water conservation)
/// v28 = fog_state (FogState serialization for fog and visibility)
/// v29 = urban_growth_boundary (UrbanGrowthBoundary serialization for UGB polygon)
/// v30 = snow_state (SnowGrid + SnowPlowingState serialization for snow accumulation and plowing)
/// v31 = agriculture_state (AgricultureState serialization for growing season and crop yield)
pub const CURRENT_SAVE_VERSION: u32 = 31; // v31: Agriculture growing season

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
    /// Vacancy rates per zone type (added in v5).
    #[serde(default)]
    pub vacancy_residential: f32,
    #[serde(default)]
    pub vacancy_commercial: f32,
    #[serde(default)]
    pub vacancy_industrial: f32,
    #[serde(default)]
    pub vacancy_office: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveBuilding {
    pub zone_type: u8,
    pub level: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub capacity: u32,
    pub occupants: u32,
    // MixedUse fields (backward-compatible via serde defaults)
    #[serde(default)]
    pub commercial_capacity: u32,
    #[serde(default)]
    pub commercial_occupants: u32,
    #[serde(default)]
    pub residential_capacity: u32,
    #[serde(default)]
    pub residential_occupants: u32,
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
    /// Climate zone (0=Temperate default for backward compat).
    #[serde(default)]
    pub climate_zone: u8,
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
            climate_zone: 0, // Temperate
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
pub struct SaveStormwaterGrid {
    pub runoff: Vec<f32>,
    pub total_runoff: f32,
    pub total_infiltration: f32,
    pub width: usize,
    pub height: usize,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveDegreeDays {
    pub daily_hdd: f32,
    pub daily_cdd: f32,
    pub monthly_hdd: [f32; 12],
    pub monthly_cdd: [f32; 12],
    pub annual_hdd: f32,
    pub annual_cdd: f32,
    pub last_update_day: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveConstructionModifiers {
    pub speed_factor: f32,
    pub cost_factor: f32,
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

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveRecyclingState {
    /// Recycling tier discriminant (0=None, 1=VoluntaryDropoff, ..., 6=ZeroWaste).
    pub tier: u8,
    pub daily_tons_diverted: f64,
    pub daily_tons_contaminated: f64,
    pub daily_revenue: f64,
    pub daily_cost: f64,
    pub total_revenue: f64,
    pub total_cost: f64,
    pub participating_households: u32,
    // Economics
    pub price_paper: f64,
    pub price_plastic: f64,
    pub price_glass: f64,
    pub price_metal: f64,
    pub price_organic: f64,
    pub market_cycle_position: f64,
    pub economics_last_update_day: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveWindDamageState {
    pub current_tier: u8,
    pub accumulated_building_damage: f32,
    pub trees_knocked_down: u32,
    pub power_outage_active: bool,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveUhiGrid {
    pub cells: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveDroughtState {
    pub rainfall_history: Vec<f32>,
    pub current_index: f32,
    pub current_tier: u8,
    pub expected_daily_rainfall: f32,
    pub water_demand_modifier: f32,
    pub agriculture_modifier: f32,
    pub fire_risk_multiplier: f32,
    pub happiness_modifier: f32,
    pub last_record_day: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveHeatWaveState {
    pub consecutive_hot_days: u32,
    pub severity: u8,
    pub excess_mortality_per_100k: f32,
    pub energy_demand_multiplier: f32,
    pub water_demand_multiplier: f32,
    pub road_damage_active: bool,
    pub fire_risk_multiplier: f32,
    pub blackout_risk: f32,
    pub heat_threshold_c: f32,
    pub consecutive_extreme_days: u32,
    pub last_check_day: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveCompostFacility {
    pub method: u8,
    pub capacity_tons_per_day: f32,
    pub cost_per_ton: f32,
    pub tons_processed_today: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveCompostingState {
    pub facilities: Vec<SaveCompostFacility>,
    pub participation_rate: f32,
    pub organic_fraction: f32,
    pub total_diverted_tons: f32,
    pub daily_diversion_tons: f32,
    pub compost_revenue_per_ton: f32,
    pub daily_revenue: f32,
    pub biogas_mwh_per_ton: f32,
    pub daily_biogas_mwh: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveColdSnapState {
    pub consecutive_cold_days: u32,
    pub pipe_burst_count: u32,
    pub is_active: bool,
    pub current_tier: u8,
    pub heating_demand_modifier: f32,
    pub traffic_capacity_modifier: f32,
    pub schools_closed: bool,
    pub construction_halted: bool,
    pub homeless_mortality_rate: f32,
    pub water_service_modifier: f32,
    pub last_check_day: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SavePlantState {
    pub level: u8,
    pub capacity_mgd: f32,
    pub current_flow_mgd: f32,
    pub effluent_quality: f32,
    pub period_cost: f64,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveWaterTreatmentState {
    pub plants: Vec<SavePlantState>,
    pub total_capacity_mgd: f32,
    pub total_flow_mgd: f32,
    pub avg_effluent_quality: f32,
    pub total_period_cost: f64,
    pub city_demand_mgd: f32,
    pub treatment_coverage: f32,
    pub avg_input_quality: f32,
    pub disease_risk: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveGroundwaterDepletionState {
    pub extraction_rate: f32,
    pub recharge_rate: f32,
    pub sustainability_ratio: f32,
    pub critical_depletion: bool,
    pub subsidence_cells: u32,
    pub well_yield_modifier: f32,
    pub ticks_below_threshold: Vec<u16>,
    pub previous_levels: Vec<u8>,
    pub recharge_basin_count: u32,
    pub avg_groundwater_level: f32,
    pub cells_at_risk: u32,
    pub over_extracted_cells: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveWastewaterState {
    pub total_sewage_generated: f32,
    pub total_treatment_capacity: f32,
    pub overflow_amount: f32,
    pub coverage_ratio: f32,
    pub pollution_events: u32,
    pub health_penalty_active: bool,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveHazardousWasteState {
    pub total_generation: f32,
    pub treatment_capacity: f32,
    pub overflow: f32,
    pub illegal_dump_events: u32,
    pub contamination_level: f32,
    pub federal_fines: f64,
    pub facility_count: u32,
    pub daily_operating_cost: f64,
    pub chemical_treated: f32,
    pub thermal_treated: f32,
    pub biological_treated: f32,
    pub stabilization_treated: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveStormDrainageState {
    pub total_drain_capacity: f32,
    pub total_retention_capacity: f32,
    pub current_retention_stored: f32,
    pub drain_count: u32,
    pub retention_pond_count: u32,
    pub rain_garden_count: u32,
    pub overflow_cells: u32,
    pub drainage_coverage: f32,
}
#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveLandfillCapacityState {
    pub total_capacity: f64,
    pub current_fill: f64,
    pub daily_input_rate: f64,
    pub days_remaining: f32,
    pub years_remaining: f32,
    pub remaining_pct: f32,
    pub current_tier: u8,
    pub collection_halted: bool,
    pub landfill_count: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default, Clone, Debug)]
pub struct SaveFloodState {
    pub is_flooding: bool,
    pub total_flooded_cells: u32,
    pub total_damage: f64,
    pub max_depth: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default, Clone, Debug)]
pub struct SaveReservoirState {
    pub total_storage_capacity_mg: f32,
    pub current_level_mg: f32,
    pub inflow_rate_mgd: f32,
    pub outflow_rate_mgd: f32,
    pub evaporation_rate_mgd: f32,
    pub net_change_mgd: f32,
    pub storage_days: f32,
    pub reservoir_count: u32,
    pub warning_tier: u8,
    pub min_reserve_pct: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default, Clone, Debug)]
pub struct SaveLandfillGasState {
    pub total_gas_generation_cf_per_year: f64,
    pub methane_fraction: f32,
    pub co2_fraction: f32,
    pub collection_active: bool,
    pub collection_efficiency: f32,
    pub electricity_generated_mw: f32,
    pub uncaptured_methane_cf: f32,
    pub infrastructure_cost: f64,
    pub maintenance_cost_per_year: f64,
    pub fire_explosion_risk: f32,
    pub landfills_with_collection: u32,
    pub total_landfills: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default, Clone, Debug)]
pub struct SaveCsoState {
    pub sewer_type: u8,
    pub combined_capacity: f32,
    pub current_flow: f32,
    pub cso_active: bool,
    pub cso_discharge_gallons: f32,
    pub cso_events_total: u32,
    pub cso_events_this_year: u32,
    pub cells_with_separated_sewer: u32,
    pub total_sewer_cells: u32,
    pub separation_coverage: f32,
    pub annual_cso_volume: f32,
    pub pollution_contribution: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default, Clone, Debug)]
pub struct SaveWaterConservationState {
    pub low_flow_fixtures: bool,
    pub xeriscaping: bool,
    pub tiered_pricing: bool,
    pub greywater_recycling: bool,
    pub rainwater_harvesting: bool,
    pub demand_reduction_pct: f32,
    pub sewage_reduction_pct: f32,
    pub total_retrofit_cost: f64,
    pub annual_savings_gallons: f64,
    pub buildings_retrofitted: u32,
}
#[derive(Serialize, Deserialize, Encode, Decode, Default, Clone, Debug)]
pub struct SaveFogState {
    pub active: bool,
    pub density: u8,
    pub visibility_m: f32,
    pub hours_active: u32,
    pub max_duration_hours: u32,
    pub water_fraction: f32,
    pub traffic_speed_modifier: f32,
    pub flights_suspended: bool,
    pub last_update_hour: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default, Clone, Debug)]
pub struct SaveAgricultureState {
    pub growing_season_active: bool,
    pub crop_yield_modifier: f32,
    pub rainfall_adequacy: f32,
    pub temperature_suitability: f32,
    pub soil_quality: f32,
    pub fertilizer_bonus: f32,
    pub frost_risk: f32,
    pub frost_events_this_year: u32,
    pub frost_damage_total: f32,
    pub has_irrigation: bool,
    pub farm_count: u32,
    pub annual_rainfall_estimate: f32,
    pub last_frost_check_day: u32,
    pub last_rainfall_day: u32,
}
#[derive(Serialize, Deserialize, Encode, Decode, Default, Clone, Debug)]
pub struct SaveSnowState {
    pub depths: Vec<f32>,
    pub width: usize,
    pub height: usize,
    pub plowing_enabled: bool,
    pub season_cost: f64,
    pub cells_plowed_season: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default, Clone, Debug)]
pub struct SaveUrbanGrowthBoundary {
    pub enabled: bool,
    pub vertices_x: Vec<f32>,
    pub vertices_y: Vec<f32>,
}

impl SaveData {
    pub fn encode(&self) -> Vec<u8> {
        bitcode::encode(self)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, bitcode::Error> {
        bitcode::decode(bytes)
    }
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
