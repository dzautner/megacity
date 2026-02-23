use crate::save_types::*;

use super::disaster_stage::DisasterStageOutput;
use super::economy_stage::EconomyStageOutput;
use super::entity_stage::EntityStageOutput;
use super::environment_stage::EnvironmentStageOutput;
use super::grid_stage::GridStageOutput;
use super::policy_stage::PolicyStageOutput;

/// Assemble a complete `SaveData` from the outputs of all collection stages.
///
/// Extensions are left empty -- they are populated separately by the save
/// system via `SaveableRegistry`.
pub fn assemble_save_data(
    grid_stage: GridStageOutput,
    economy_stage: EconomyStageOutput,
    entity_stage: EntityStageOutput,
    environment_stage: EnvironmentStageOutput,
    disaster_stage: DisasterStageOutput,
    policy_stage: PolicyStageOutput,
) -> SaveData {
    SaveData {
        version: CURRENT_SAVE_VERSION,
        grid: grid_stage.grid,
        roads: grid_stage.roads,
        road_segments: grid_stage.road_segments,
        clock: economy_stage.clock,
        budget: economy_stage.budget,
        demand: economy_stage.demand,
        extended_budget: economy_stage.extended_budget,
        loan_book: economy_stage.loan_book,
        buildings: entity_stage.buildings,
        citizens: entity_stage.citizens,
        utility_sources: entity_stage.utility_sources,
        service_buildings: entity_stage.service_buildings,
        water_sources: entity_stage.water_sources,
        weather: environment_stage.weather,
        uhi_grid: environment_stage.uhi_grid,
        stormwater_grid: environment_stage.stormwater_grid,
        degree_days: environment_stage.degree_days,
        construction_modifiers: environment_stage.construction_modifiers,
        snow_state: environment_stage.snow_state,
        agriculture_state: environment_stage.agriculture_state,
        fog_state: environment_stage.fog_state,
        urban_growth_boundary: environment_stage.urban_growth_boundary,
        drought_state: disaster_stage.drought_state,
        heat_wave_state: disaster_stage.heat_wave_state,
        cold_snap_state: disaster_stage.cold_snap_state,
        flood_state: disaster_stage.flood_state,
        wind_damage_state: disaster_stage.wind_damage_state,
        reservoir_state: disaster_stage.reservoir_state,
        landfill_gas_state: disaster_stage.landfill_gas_state,
        cso_state: disaster_stage.cso_state,
        hazardous_waste_state: disaster_stage.hazardous_waste_state,
        wastewater_state: disaster_stage.wastewater_state,
        storm_drainage_state: disaster_stage.storm_drainage_state,
        landfill_capacity_state: disaster_stage.landfill_capacity_state,
        groundwater_depletion_state: disaster_stage.groundwater_depletion_state,
        water_treatment_state: disaster_stage.water_treatment_state,
        water_conservation_state: disaster_stage.water_conservation_state,
        policies: policy_stage.policies,
        unlock_state: policy_stage.unlock_state,
        recycling_state: policy_stage.recycling_state,
        composting_state: policy_stage.composting_state,
        lifecycle_timer: policy_stage.lifecycle_timer,
        life_sim_timer: policy_stage.life_sim_timer,
        virtual_population: policy_stage.virtual_population,
        extensions: std::collections::BTreeMap::new(),
    }
}
