// ---------------------------------------------------------------------------
// SystemParam bundles for save/load systems
// ---------------------------------------------------------------------------

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use simulation::agriculture::AgricultureState;
use simulation::budget::ExtendedBudget;
use simulation::cold_snap::ColdSnapState;
use simulation::composting::CompostingState;
use simulation::cso::SewerSystemState;
use simulation::degree_days::DegreeDays;
use simulation::drought::DroughtState;
use simulation::flood_simulation::{FloodGrid, FloodState};
use simulation::fog::FogState;
use simulation::groundwater_depletion::GroundwaterDepletionState;
use simulation::hazardous_waste::HazardousWasteState;
use simulation::heat_wave::HeatWaveState;
use simulation::landfill_gas::LandfillGasState;
use simulation::landfill_warning::LandfillCapacityState;
use simulation::life_simulation::LifeSimTimer;
use simulation::loans::LoanBook;
use simulation::policies::Policies;
use simulation::recycling::{RecyclingEconomics, RecyclingState};
use simulation::reservoir::ReservoirState;
use simulation::snow::{SnowGrid, SnowPlowingState, SnowStats};
use simulation::storm_drainage::StormDrainageState;
use simulation::stormwater::StormwaterGrid;
use simulation::unlocks::UnlockState;
use simulation::urban_growth_boundary::UrbanGrowthBoundary;
use simulation::urban_heat_island::UhiGrid;
use simulation::virtual_population::VirtualPopulation;
use simulation::wastewater::WastewaterState;
use simulation::water_conservation::WaterConservationState;
use simulation::water_treatment::WaterTreatmentState;
use simulation::weather::{ClimateZone, ConstructionModifiers, Weather};
use simulation::wind_damage::WindDamageState;

/// Read-only access to the V2+ resources (policies, weather, unlocks, ext budget, loans, virtual pop, life sim timer, stormwater, degree days, climate zone, construction modifiers).
#[derive(SystemParam)]
pub(crate) struct V2ResourcesRead<'w> {
    pub policies: Res<'w, Policies>,
    pub weather: Res<'w, Weather>,
    pub unlock_state: Res<'w, UnlockState>,
    pub extended_budget: Res<'w, ExtendedBudget>,
    pub loan_book: Res<'w, LoanBook>,
    pub virtual_population: Res<'w, VirtualPopulation>,
    pub life_sim_timer: Res<'w, LifeSimTimer>,
    pub stormwater_grid: Res<'w, StormwaterGrid>,
    pub degree_days: Res<'w, DegreeDays>,
    pub climate_zone: Res<'w, ClimateZone>,
    pub construction_modifiers: Res<'w, ConstructionModifiers>,
    pub recycling_state: Res<'w, RecyclingState>,
    pub recycling_economics: Res<'w, RecyclingEconomics>,
    pub wind_damage_state: Res<'w, WindDamageState>,
    pub uhi_grid: Res<'w, UhiGrid>,
    pub drought_state: Res<'w, DroughtState>,
    pub heat_wave_state: Res<'w, HeatWaveState>,
    pub composting_state: Res<'w, CompostingState>,
    pub cold_snap_state: Res<'w, ColdSnapState>,
    pub water_treatment_state: Res<'w, WaterTreatmentState>,
    pub groundwater_depletion_state: Res<'w, GroundwaterDepletionState>,
    pub wastewater_state: Res<'w, WastewaterState>,
    pub hazardous_waste_state: Res<'w, HazardousWasteState>,
    pub storm_drainage_state: Res<'w, StormDrainageState>,
    pub landfill_capacity_state: Res<'w, LandfillCapacityState>,
    pub flood_state: Res<'w, FloodState>,
    pub reservoir_state: Res<'w, ReservoirState>,
    pub landfill_gas_state: Res<'w, LandfillGasState>,
    pub cso_state: Res<'w, SewerSystemState>,
    pub water_conservation_state: Res<'w, WaterConservationState>,
    pub fog_state: Res<'w, FogState>,
    pub urban_growth_boundary: Res<'w, UrbanGrowthBoundary>,
    pub snow_grid: Res<'w, SnowGrid>,
    pub snow_plowing_state: Res<'w, SnowPlowingState>,
    pub agriculture_state: Res<'w, AgricultureState>,
}

/// Mutable access to the V2+ resources.
#[derive(SystemParam)]
pub(crate) struct V2ResourcesWrite<'w> {
    pub policies: ResMut<'w, Policies>,
    pub weather: ResMut<'w, Weather>,
    pub unlock_state: ResMut<'w, UnlockState>,
    pub extended_budget: ResMut<'w, ExtendedBudget>,
    pub loan_book: ResMut<'w, LoanBook>,
    pub virtual_population: ResMut<'w, VirtualPopulation>,
    pub life_sim_timer: ResMut<'w, LifeSimTimer>,
    pub stormwater_grid: ResMut<'w, StormwaterGrid>,
    pub degree_days: ResMut<'w, DegreeDays>,
    pub climate_zone: ResMut<'w, ClimateZone>,
    pub construction_modifiers: ResMut<'w, ConstructionModifiers>,
    pub recycling_state: ResMut<'w, RecyclingState>,
    pub recycling_economics: ResMut<'w, RecyclingEconomics>,
    pub wind_damage_state: ResMut<'w, WindDamageState>,
    pub uhi_grid: ResMut<'w, UhiGrid>,
    pub drought_state: ResMut<'w, DroughtState>,
    pub heat_wave_state: ResMut<'w, HeatWaveState>,
    pub composting_state: ResMut<'w, CompostingState>,
    pub cold_snap_state: ResMut<'w, ColdSnapState>,
    pub water_treatment_state: ResMut<'w, WaterTreatmentState>,
    pub groundwater_depletion_state: ResMut<'w, GroundwaterDepletionState>,
    pub wastewater_state: ResMut<'w, WastewaterState>,
    pub hazardous_waste_state: ResMut<'w, HazardousWasteState>,
    pub storm_drainage_state: ResMut<'w, StormDrainageState>,
    pub landfill_capacity_state: ResMut<'w, LandfillCapacityState>,
    pub flood_state: ResMut<'w, FloodState>,
    pub flood_grid: ResMut<'w, FloodGrid>,
    pub reservoir_state: ResMut<'w, ReservoirState>,
    pub landfill_gas_state: ResMut<'w, LandfillGasState>,
    pub cso_state: ResMut<'w, SewerSystemState>,
    pub water_conservation_state: ResMut<'w, WaterConservationState>,
    pub fog_state: ResMut<'w, FogState>,
    pub urban_growth_boundary: ResMut<'w, UrbanGrowthBoundary>,
    pub snow_grid: ResMut<'w, SnowGrid>,
    pub snow_plowing_state: ResMut<'w, SnowPlowingState>,
    pub snow_stats: ResMut<'w, SnowStats>,
    pub agriculture_state: ResMut<'w, AgricultureState>,
}
