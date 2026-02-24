// ---------------------------------------------------------------------------
// Serialization: re-exports from sub-modules + create_save_data
// ---------------------------------------------------------------------------
//
// ## Staged save pipeline
//
// The save pipeline is organized as a series of focused collection stages,
// each responsible for one domain of game state. This replaces the former
// monolithic `create_save_data` function that had 40+ parameters.
//
// The stages are:
//   - **Grid**: world grid cells, road network, road segments
//   - **Economy**: clock, budget, demand, extended budget, loans
//   - **Entity**: buildings, citizens, utilities, services, water sources
//   - **Environment**: weather, climate, UHI, stormwater, snow, agriculture, fog
//   - **Disaster**: drought, heat wave, cold snap, flood, wind damage, etc.
//   - **Policy**: policies, unlocks, recycling, composting, lifecycle, life sim
//
// Each stage has a typed output struct defined in `save_stages.rs`. The final
// `SaveData` is assembled from these stage outputs by `assemble_save_data`.

// Re-export everything from sub-modules so existing `use crate::serialization::*` works.
pub use crate::save_codec::*;
pub use crate::save_migrate::*;
pub use crate::save_restore::*;
pub use crate::save_stages::*;
pub use crate::save_types::*;

#[cfg(test)]
mod tests_basic_roundtrip;
#[cfg(test)]
mod tests_citizen;
#[cfg(test)]
mod tests_compat_v2;
#[cfg(test)]
mod tests_extensions_vacancy;
#[cfg(test)]
mod tests_family;
#[cfg(test)]
mod tests_life_sim;
#[cfg(test)]
mod tests_migration;
#[cfg(test)]
mod tests_migration_chain;
#[cfg(test)]
mod tests_save_error;
#[cfg(test)]
mod tests_savings;
#[cfg(test)]
mod tests_stormwater_climate;
#[cfg(test)]
mod tests_water;

use simulation::agriculture::AgricultureState;
use simulation::buildings::{Building, MixedUseBuilding};
use simulation::cso::SewerSystemState;
use simulation::drought::DroughtState;
use simulation::economy::CityBudget;
use simulation::flood_simulation::FloodState;
use simulation::fog::FogState;
use simulation::grid::WorldGrid;
use simulation::groundwater_depletion::GroundwaterDepletionState;
use simulation::hazardous_waste::HazardousWasteState;
use simulation::heat_wave::HeatWaveState;
use simulation::landfill_gas::LandfillGasState;
use simulation::landfill_warning::LandfillCapacityState;
use simulation::life_simulation::LifeSimTimer;
use simulation::lifecycle::LifecycleTimer;
use simulation::loans::LoanBook;
use simulation::policies::Policies;
use simulation::reservoir::ReservoirState;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::services::ServiceBuilding;
use simulation::storm_drainage::StormDrainageState;
use simulation::stormwater::StormwaterGrid;
use simulation::time_of_day::GameClock;
use simulation::unlocks::UnlockState;
use simulation::urban_growth_boundary::UrbanGrowthBoundary;
use simulation::urban_heat_island::UhiGrid;
use simulation::utilities::UtilitySource;
use simulation::virtual_population::VirtualPopulation;
use simulation::wastewater::WastewaterState;
use simulation::water_conservation::WaterConservationState;
use simulation::water_sources::WaterSource;
use simulation::water_treatment::WaterTreatmentState;
use simulation::weather::{ClimateZone, ConstructionModifiers, Weather};
use simulation::wind_damage::WindDamageState;
use simulation::zones::ZoneDemand;

use simulation::budget::ExtendedBudget;
use simulation::cold_snap::ColdSnapState;
use simulation::degree_days::DegreeDays;
use simulation::recycling::{RecyclingEconomics, RecyclingState};
use simulation::snow::{SnowGrid, SnowPlowingState};

/// Create a complete `SaveData` from game state.
///
/// This function preserves the original public API but internally delegates to
/// the staged collection pipeline defined in `save_stages.rs`. Each stage
/// collects a focused subset of the data, and `assemble_save_data` combines
/// them into the final `SaveData`.
#[allow(clippy::too_many_arguments)]
pub fn create_save_data(
    grid: &WorldGrid,
    roads: &RoadNetwork,
    clock: &GameClock,
    budget: &CityBudget,
    demand: &ZoneDemand,
    buildings: &[(Building, Option<MixedUseBuilding>)],
    citizens: &[CitizenSaveInput],
    utility_sources: &[UtilitySource],
    service_buildings: &[(ServiceBuilding,)],
    segment_store: Option<&RoadSegmentStore>,
    policies: Option<&Policies>,
    weather: Option<&Weather>,
    unlock_state: Option<&UnlockState>,
    extended_budget: Option<&ExtendedBudget>,
    loan_book: Option<&LoanBook>,
    lifecycle_timer: Option<&LifecycleTimer>,
    virtual_population: Option<&VirtualPopulation>,
    life_sim_timer: Option<&LifeSimTimer>,
    stormwater_grid: Option<&StormwaterGrid>,
    water_sources: Option<&[WaterSource]>,
    degree_days: Option<&DegreeDays>,
    climate_zone: Option<&ClimateZone>,
    construction_modifiers: Option<&ConstructionModifiers>,
    recycling_state: Option<(&RecyclingState, &RecyclingEconomics)>,
    wind_damage_state: Option<&WindDamageState>,
    uhi_grid: Option<&UhiGrid>,
    drought_state: Option<&DroughtState>,
    heat_wave_state: Option<&HeatWaveState>,
    composting_state: Option<&simulation::composting::CompostingState>,
    cold_snap_state: Option<&ColdSnapState>,
    water_treatment_state: Option<&WaterTreatmentState>,
    groundwater_depletion_state: Option<&GroundwaterDepletionState>,
    wastewater_state: Option<&WastewaterState>,
    hazardous_waste_state: Option<&HazardousWasteState>,
    storm_drainage_state: Option<&StormDrainageState>,
    landfill_capacity_state: Option<&LandfillCapacityState>,
    flood_state: Option<&FloodState>,
    reservoir_state: Option<&ReservoirState>,
    landfill_gas_state: Option<&LandfillGasState>,
    cso_state: Option<&SewerSystemState>,
    water_conservation_state: Option<&WaterConservationState>,
    fog_state: Option<&FogState>,
    urban_growth_boundary: Option<&UrbanGrowthBoundary>,
    snow_state: Option<(&SnowGrid, &SnowPlowingState)>,
    agriculture_state: Option<&AgricultureState>,
) -> SaveData {
    let grid_stage = collect_grid_stage(grid, roads, segment_store);
    let economy_stage = collect_economy_stage(clock, budget, demand, extended_budget, loan_book);
    let entity_stage = collect_entity_stage(
        buildings,
        citizens,
        utility_sources,
        service_buildings,
        water_sources,
    );
    let environment_stage = collect_environment_stage(
        weather,
        climate_zone,
        uhi_grid,
        stormwater_grid,
        degree_days,
        construction_modifiers,
        snow_state,
        agriculture_state,
        fog_state,
        urban_growth_boundary,
    );
    let disaster_stage = collect_disaster_stage(
        drought_state,
        heat_wave_state,
        cold_snap_state,
        flood_state,
        wind_damage_state,
        reservoir_state,
        landfill_gas_state,
        cso_state,
        hazardous_waste_state,
        wastewater_state,
        storm_drainage_state,
        landfill_capacity_state,
        groundwater_depletion_state,
        water_treatment_state,
        water_conservation_state,
    );
    let policy_stage = collect_policy_stage(
        policies,
        unlock_state,
        recycling_state,
        composting_state,
        lifecycle_timer,
        life_sim_timer,
        virtual_population,
    );

    assemble_save_data(
        grid_stage,
        economy_stage,
        entity_stage,
        environment_stage,
        disaster_stage,
        policy_stage,
    )
}
