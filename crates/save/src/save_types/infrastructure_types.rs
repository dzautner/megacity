// ---------------------------------------------------------------------------
// Infrastructure & environment save structs: recycling, wind, UHI, drought,
// heat waves, composting, cold snaps, water treatment, groundwater,
// wastewater, hazardous waste, storm drainage, landfill, flood, reservoir,
// landfill gas, CSO, water conservation, fog, agriculture, snow, UGB
// ---------------------------------------------------------------------------

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

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
