// ---------------------------------------------------------------------------
// SystemParam bundles for save/load systems
// ---------------------------------------------------------------------------

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use simulation::budget::ExtendedBudget;
use simulation::degree_days::DegreeDays;
use simulation::life_simulation::LifeSimTimer;
use simulation::loans::LoanBook;
use simulation::policies::Policies;
use simulation::recycling::{RecyclingEconomics, RecyclingState};
use simulation::stormwater::StormwaterGrid;
use simulation::unlocks::UnlockState;
use simulation::urban_heat_island::UhiGrid;
use simulation::virtual_population::VirtualPopulation;
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
}
