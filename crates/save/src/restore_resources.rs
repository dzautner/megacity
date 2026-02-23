use bevy::prelude::*;

use crate::serialization::{
    restore_agriculture, restore_climate_zone, restore_cold_snap, restore_composting,
    restore_construction_modifiers, restore_cso, restore_degree_days, restore_drought,
    restore_extended_budget, restore_flood_state, restore_fog_state, restore_groundwater_depletion,
    restore_hazardous_waste, restore_heat_wave, restore_landfill_capacity, restore_landfill_gas,
    restore_life_sim_timer, restore_lifecycle_timer, restore_loan_book, restore_policies,
    restore_recycling, restore_reservoir_state, restore_road_segment_store, restore_snow,
    restore_storm_drainage, restore_stormwater_grid, restore_uhi_grid, restore_unlock_state,
    restore_urban_growth_boundary, restore_virtual_population, restore_wastewater,
    restore_water_conservation, restore_water_source, restore_water_treatment, restore_weather,
    restore_wind_damage_state, u8_to_road_type, u8_to_zone_type, SaveData,
};

use simulation::agriculture::AgricultureState;
use simulation::budget::ExtendedBudget;
use simulation::cold_snap::ColdSnapState;
use simulation::composting::CompostingState;
use simulation::cso::SewerSystemState;
use simulation::degree_days::DegreeDays;
use simulation::drought::DroughtState;
use simulation::economy::CityBudget;
use simulation::flood_simulation::{FloodGrid, FloodState};
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
use simulation::recycling::{RecyclingEconomics, RecyclingState};
use simulation::reservoir::ReservoirState;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::snow::{SnowGrid, SnowPlowingState, SnowStats};
use simulation::storm_drainage::StormDrainageState;
use simulation::stormwater::StormwaterGrid;
use simulation::time_of_day::GameClock;
use simulation::unlocks::UnlockState;
use simulation::urban_growth_boundary::UrbanGrowthBoundary;
use simulation::urban_heat_island::UhiGrid;
use simulation::virtual_population::VirtualPopulation;
use simulation::wastewater::WastewaterState;
use simulation::water_conservation::WaterConservationState;
use simulation::water_treatment::WaterTreatmentState;
use simulation::weather::{ClimateZone, ConstructionModifiers, Weather};
use simulation::wind_damage::WindDamageState;
use simulation::zones::ZoneDemand;

/// Restores all core resources from a parsed SaveData.
///
/// Internally delegates to focused sub-functions:
///   - `restore_grid_and_roads`: grid cells, road network, road segments
///   - `restore_economy_state`: clock, budget, demand
///   - `restore_v2_resources`: all V2+ optional resources (policies, weather, etc.)
pub(crate) fn restore_resources_from_save(world: &mut World, save: &SaveData) {
    restore_grid_and_roads(world, save);
    restore_economy_state(world, save);
    restore_v2_resources(world, save);
}

/// Restore grid cells, road network, and road segments from save data.
fn restore_grid_and_roads(world: &mut World, save: &SaveData) {
    // Restore grid
    {
        let mut grid = world.resource_mut::<WorldGrid>();
        *grid = WorldGrid::new(save.grid.width, save.grid.height);
        for (i, sc) in save.grid.cells.iter().enumerate() {
            grid.cells[i].elevation = sc.elevation;
            grid.cells[i].cell_type = match sc.cell_type {
                1 => simulation::grid::CellType::Water,
                2 => simulation::grid::CellType::Road,
                _ => simulation::grid::CellType::Grass,
            };
            grid.cells[i].zone = u8_to_zone_type(sc.zone);
            grid.cells[i].road_type = u8_to_road_type(sc.road_type);
            grid.cells[i].has_power = sc.has_power;
            grid.cells[i].has_water = sc.has_water;
        }
    }

    // Restore roads - use saved road types, not default Local
    {
        let saved_road_types: Vec<(usize, usize, u8)> = save
            .roads
            .road_positions
            .iter()
            .map(|(x, y)| {
                let idx = y * save.grid.width + x;
                let rt = if idx < save.grid.cells.len() {
                    save.grid.cells[idx].road_type
                } else {
                    0
                };
                (*x, *y, rt)
            })
            .collect();

        *world.resource_mut::<RoadNetwork>() = RoadNetwork::default();

        // Use resource_scope to access both grid and roads mutably
        // since place_road needs both.
        world.resource_scope(|world, mut grid: Mut<WorldGrid>| {
            let mut roads = world.resource_mut::<RoadNetwork>();
            for (x, y, _) in &saved_road_types {
                roads.place_road(&mut grid, *x, *y);
            }
            // Restore the saved road types (place_road overwrites with Local)
            for (x, y, rt) in &saved_road_types {
                if grid.in_bounds(*x, *y) {
                    grid.get_mut(*x, *y).road_type = u8_to_road_type(*rt);
                }
            }
        });
    }

    // Restore road segments (if present in save)
    {
        if let Some(ref saved_segments) = save.road_segments {
            let mut restored = restore_road_segment_store(saved_segments);
            world.resource_scope(|world, mut grid: Mut<WorldGrid>| {
                let mut roads = world.resource_mut::<RoadNetwork>();
                restored.rasterize_all(&mut grid, &mut roads);
            });
            *world.resource_mut::<RoadSegmentStore>() = restored;
        } else {
            *world.resource_mut::<RoadSegmentStore>() = RoadSegmentStore::default();
        }
    }
}

/// Restore clock, budget, and demand from save data.
fn restore_economy_state(world: &mut World, save: &SaveData) {
    // Restore clock
    {
        let mut clock = world.resource_mut::<GameClock>();
        clock.day = save.clock.day;
        clock.hour = save.clock.hour;
        clock.speed = save.clock.speed;
        clock.paused = false;
    }

    // Restore budget
    {
        let mut budget = world.resource_mut::<CityBudget>();
        budget.treasury = save.budget.treasury;
        budget.tax_rate = save.budget.tax_rate;
        budget.last_collection_day = save.budget.last_collection_day;
    }

    // Restore demand
    {
        let mut demand = world.resource_mut::<ZoneDemand>();
        demand.residential = save.demand.residential;
        demand.commercial = save.demand.commercial;
        demand.industrial = save.demand.industrial;
        demand.office = save.demand.office;
        demand.vacancy_residential = save.demand.vacancy_residential;
        demand.vacancy_commercial = save.demand.vacancy_commercial;
        demand.vacancy_industrial = save.demand.vacancy_industrial;
        demand.vacancy_office = save.demand.vacancy_office;
    }
}

/// Restore all V2+ resources (policies, weather, unlocks, extended budget,
/// loans, lifecycle, virtual population, life sim, environment, disasters).
fn restore_v2_resources(world: &mut World, save: &SaveData) {
    // Restore V2 fields
    if let Some(ref saved_policies) = save.policies {
        *world.resource_mut::<Policies>() = restore_policies(saved_policies);
    } else {
        *world.resource_mut::<Policies>() = Policies::default();
    }

    if let Some(ref saved_weather) = save.weather {
        *world.resource_mut::<Weather>() = restore_weather(saved_weather);
        *world.resource_mut::<ClimateZone>() = restore_climate_zone(saved_weather);
    } else {
        *world.resource_mut::<Weather>() = Weather::default();
        *world.resource_mut::<ClimateZone>() = ClimateZone::default();
    }

    if let Some(ref saved_unlocks) = save.unlock_state {
        *world.resource_mut::<UnlockState>() = restore_unlock_state(saved_unlocks);
    } else {
        *world.resource_mut::<UnlockState>() = UnlockState::default();
    }

    if let Some(ref saved_ext_budget) = save.extended_budget {
        *world.resource_mut::<ExtendedBudget>() = restore_extended_budget(saved_ext_budget);
    } else {
        *world.resource_mut::<ExtendedBudget>() = ExtendedBudget::default();
    }

    if let Some(ref saved_loans) = save.loan_book {
        *world.resource_mut::<LoanBook>() = restore_loan_book(saved_loans);
    } else {
        *world.resource_mut::<LoanBook>() = LoanBook::default();
    }

    // Restore lifecycle timer
    if let Some(ref saved_timer) = save.lifecycle_timer {
        *world.resource_mut::<LifecycleTimer>() = restore_lifecycle_timer(saved_timer);
    } else {
        let day = world.resource::<GameClock>().day;
        let mut timer = world.resource_mut::<LifecycleTimer>();
        timer.last_aging_day = day;
        timer.last_emigration_tick = 0;
    }

    // Restore virtual population
    if let Some(ref saved_vp) = save.virtual_population {
        *world.resource_mut::<VirtualPopulation>() = restore_virtual_population(saved_vp);
    } else {
        *world.resource_mut::<VirtualPopulation>() = VirtualPopulation::default();
    }

    // Restore life sim timer
    if let Some(ref saved_lst) = save.life_sim_timer {
        *world.resource_mut::<LifeSimTimer>() = restore_life_sim_timer(saved_lst);
    } else {
        *world.resource_mut::<LifeSimTimer>() = LifeSimTimer::default();
    }

    // Restore stormwater grid
    if let Some(ref saved_sw) = save.stormwater_grid {
        *world.resource_mut::<StormwaterGrid>() = restore_stormwater_grid(saved_sw);
    } else {
        *world.resource_mut::<StormwaterGrid>() = StormwaterGrid::default();
    }

    // Restore degree days
    if let Some(ref saved_dd) = save.degree_days {
        *world.resource_mut::<DegreeDays>() = restore_degree_days(saved_dd);
    } else {
        *world.resource_mut::<DegreeDays>() = DegreeDays::default();
    }

    // Restore construction modifiers
    if let Some(ref saved_cm) = save.construction_modifiers {
        *world.resource_mut::<ConstructionModifiers>() = restore_construction_modifiers(saved_cm);
    } else {
        *world.resource_mut::<ConstructionModifiers>() = ConstructionModifiers::default();
    }

    // Restore recycling state and economics
    if let Some(ref saved_recycling) = save.recycling_state {
        let (rs, re) = restore_recycling(saved_recycling);
        *world.resource_mut::<RecyclingState>() = rs;
        *world.resource_mut::<RecyclingEconomics>() = re;
    } else {
        *world.resource_mut::<RecyclingState>() = RecyclingState::default();
        *world.resource_mut::<RecyclingEconomics>() = RecyclingEconomics::default();
    }

    // Restore wind damage state
    if let Some(ref saved_wds) = save.wind_damage_state {
        *world.resource_mut::<WindDamageState>() = restore_wind_damage_state(saved_wds);
    } else {
        *world.resource_mut::<WindDamageState>() = WindDamageState::default();
    }

    // Restore UHI grid
    if let Some(ref saved_uhi) = save.uhi_grid {
        *world.resource_mut::<UhiGrid>() = restore_uhi_grid(saved_uhi);
    } else {
        *world.resource_mut::<UhiGrid>() = UhiGrid::default();
    }

    // Restore drought state
    if let Some(ref saved_drought) = save.drought_state {
        *world.resource_mut::<DroughtState>() = restore_drought(saved_drought);
    } else {
        *world.resource_mut::<DroughtState>() = DroughtState::default();
    }

    // Restore heat wave state
    if let Some(ref saved_hw) = save.heat_wave_state {
        *world.resource_mut::<HeatWaveState>() = restore_heat_wave(saved_hw);
    } else {
        *world.resource_mut::<HeatWaveState>() = HeatWaveState::default();
    }

    // Restore composting state
    if let Some(ref saved_cs) = save.composting_state {
        *world.resource_mut::<CompostingState>() = restore_composting(saved_cs);
    } else {
        *world.resource_mut::<CompostingState>() = CompostingState::default();
    }

    // Restore cold snap state
    if let Some(ref saved_cs) = save.cold_snap_state {
        *world.resource_mut::<ColdSnapState>() = restore_cold_snap(saved_cs);
    } else {
        *world.resource_mut::<ColdSnapState>() = ColdSnapState::default();
    }

    // Restore water treatment state
    if let Some(ref wts) = save.water_treatment_state {
        *world.resource_mut::<WaterTreatmentState>() = restore_water_treatment(wts);
    } else {
        *world.resource_mut::<WaterTreatmentState>() = WaterTreatmentState::default();
    }

    // Restore groundwater depletion state
    if let Some(ref gds) = save.groundwater_depletion_state {
        *world.resource_mut::<GroundwaterDepletionState>() = restore_groundwater_depletion(gds);
    } else {
        *world.resource_mut::<GroundwaterDepletionState>() = GroundwaterDepletionState::default();
    }

    // Restore wastewater state
    if let Some(ref ws) = save.wastewater_state {
        *world.resource_mut::<WastewaterState>() = restore_wastewater(ws);
    } else {
        *world.resource_mut::<WastewaterState>() = WastewaterState::default();
    }

    // Restore hazardous waste state
    if let Some(ref hws) = save.hazardous_waste_state {
        *world.resource_mut::<HazardousWasteState>() = restore_hazardous_waste(hws);
    } else {
        *world.resource_mut::<HazardousWasteState>() = HazardousWasteState::default();
    }

    // Restore storm drainage state
    if let Some(ref sds) = save.storm_drainage_state {
        *world.resource_mut::<StormDrainageState>() = restore_storm_drainage(sds);
    } else {
        *world.resource_mut::<StormDrainageState>() = StormDrainageState::default();
    }

    // Restore landfill capacity state
    if let Some(ref lcs) = save.landfill_capacity_state {
        *world.resource_mut::<LandfillCapacityState>() = restore_landfill_capacity(lcs);
    } else {
        *world.resource_mut::<LandfillCapacityState>() = LandfillCapacityState::default();
    }

    // Restore flood state
    if let Some(ref fs) = save.flood_state {
        *world.resource_mut::<FloodState>() = restore_flood_state(fs);
    }
    *world.resource_mut::<FloodGrid>() = FloodGrid::default();

    // Restore reservoir state
    if let Some(ref rs) = save.reservoir_state {
        *world.resource_mut::<ReservoirState>() = restore_reservoir_state(rs);
    }

    // Restore landfill gas state
    if let Some(ref lgs) = save.landfill_gas_state {
        *world.resource_mut::<LandfillGasState>() = restore_landfill_gas(lgs);
    }

    // Restore CSO state
    if let Some(ref s) = save.cso_state {
        *world.resource_mut::<SewerSystemState>() = restore_cso(s);
    }

    // Restore water conservation state
    if let Some(ref s) = save.water_conservation_state {
        *world.resource_mut::<WaterConservationState>() = restore_water_conservation(s);
    }

    // Restore fog state
    if let Some(ref s) = save.fog_state {
        *world.resource_mut::<FogState>() = restore_fog_state(s);
    }

    // Restore agriculture state
    if let Some(ref s) = save.agriculture_state {
        *world.resource_mut::<AgricultureState>() = restore_agriculture(s);
    }

    // Restore urban growth boundary
    if let Some(ref s) = save.urban_growth_boundary {
        *world.resource_mut::<UrbanGrowthBoundary>() = restore_urban_growth_boundary(s);
    } else {
        *world.resource_mut::<UrbanGrowthBoundary>() = UrbanGrowthBoundary::default();
    }

    // Restore snow state
    if let Some(ref s) = save.snow_state {
        let (sg, sp) = restore_snow(s);
        *world.resource_mut::<SnowGrid>() = sg;
        *world.resource_mut::<SnowPlowingState>() = sp;
    } else {
        *world.resource_mut::<SnowGrid>() = SnowGrid::default();
        *world.resource_mut::<SnowPlowingState>() = SnowPlowingState::default();
    }
    *world.resource_mut::<SnowStats>() = SnowStats::default();
}
