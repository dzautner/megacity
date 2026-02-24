use bevy::prelude::*;
use simulation::SaveLoadState;
use simulation::SaveableRegistry;

use crate::save_stages::{
    assemble_save_data, collect_disaster_stage, collect_economy_stage, collect_entity_stage,
    collect_environment_stage, collect_grid_stage, collect_policy_stage,
};
use crate::serialization::CitizenSaveInput;

use simulation::agriculture::AgricultureState;
use simulation::budget::ExtendedBudget;
use simulation::buildings::{Building, MixedUseBuilding};
use simulation::citizen::{
    CitizenDetails, CitizenStateComp, Family, HomeLocation, Needs, PathCache, Personality,
    Position, Velocity, WorkLocation,
};
use simulation::cold_snap::ColdSnapState;
use simulation::composting::CompostingState;
use simulation::cso::SewerSystemState;
use simulation::degree_days::DegreeDays;
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
use simulation::movement::ActivityTimer;
use simulation::policies::Policies;
use simulation::recycling::{RecyclingEconomics, RecyclingState};
use simulation::reservoir::ReservoirState;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::services::ServiceBuilding;
use simulation::snow::{SnowGrid, SnowPlowingState};
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

/// Exclusive system that performs the entire save operation with full world
/// access.  Runs on `OnEnter(SaveLoadState::Saving)`, then transitions back
/// to `Idle`.
pub(crate) fn exclusive_save(world: &mut World) {
    // -- Stage 1: Collect entity data via queries (needs &mut World) --
    let building_data: Vec<(Building, Option<MixedUseBuilding>)> = {
        let mut q = world.query::<(&Building, Option<&MixedUseBuilding>)>();
        q.iter(world)
            .map(|(b, mu)| (b.clone(), mu.cloned()))
            .collect()
    };

    let citizen_data: Vec<CitizenSaveInput> = {
        let mut q = world.query::<(
            Entity,
            &CitizenDetails,
            &CitizenStateComp,
            &HomeLocation,
            &WorkLocation,
            &PathCache,
            &Velocity,
            &Position,
            &Personality,
            &Needs,
            &ActivityTimer,
            &Family,
        )>();
        q.iter(world)
            .map(
                |(entity, d, state, home, work, path, vel, pos, pers, needs, timer, family)| {
                    CitizenSaveInput {
                        entity,
                        details: d.clone(),
                        state: state.0,
                        home_x: home.grid_x,
                        home_y: home.grid_y,
                        work_x: work.grid_x,
                        work_y: work.grid_y,
                        path: path.clone(),
                        velocity: vel.clone(),
                        position: pos.clone(),
                        personality: pers.clone(),
                        needs: needs.clone(),
                        activity_timer: timer.0,
                        family: family.clone(),
                    }
                },
            )
            .collect()
    };

    let utility_data: Vec<UtilitySource> = {
        let mut q = world.query::<&UtilitySource>();
        q.iter(world).cloned().collect()
    };

    let service_data: Vec<(ServiceBuilding,)> = {
        let mut q = world.query::<&ServiceBuilding>();
        q.iter(world).map(|sb| (sb.clone(),)).collect()
    };

    let water_source_data: Vec<WaterSource> = {
        let mut q = world.query::<&WaterSource>();
        q.iter(world).cloned().collect()
    };

    // -- Stage 2: Build SaveData via staged collection pipeline --
    //
    // Each stage reads only the resources it needs, keeping the code focused
    // and avoiding a single function with 40+ parameters.
    let save = {
        let grid = world.resource::<WorldGrid>();
        let roads = world.resource::<RoadNetwork>();
        let segments = world.resource::<RoadSegmentStore>();

        let segment_ref = if segments.segments.is_empty() {
            None
        } else {
            Some(segments)
        };

        let grid_stage = collect_grid_stage(grid, roads, segment_ref);

        let clock = world.resource::<GameClock>();
        let budget = world.resource::<CityBudget>();
        let demand = world.resource::<ZoneDemand>();
        let extended_budget = world.resource::<ExtendedBudget>();
        let loan_book = world.resource::<LoanBook>();

        let economy_stage = collect_economy_stage(
            clock,
            budget,
            demand,
            Some(extended_budget),
            Some(loan_book),
        );

        let entity_stage = collect_entity_stage(
            &building_data,
            &citizen_data,
            &utility_data,
            &service_data,
            if water_source_data.is_empty() {
                None
            } else {
                Some(&water_source_data)
            },
        );

        let weather = world.resource::<Weather>();
        let climate_zone = world.resource::<ClimateZone>();
        let uhi_grid = world.resource::<UhiGrid>();
        let stormwater_grid = world.resource::<StormwaterGrid>();
        let degree_days = world.resource::<DegreeDays>();
        let construction_modifiers = world.resource::<ConstructionModifiers>();
        let snow_grid = world.resource::<SnowGrid>();
        let snow_plowing_state = world.resource::<SnowPlowingState>();
        let agriculture_state = world.resource::<AgricultureState>();
        let fog_state = world.resource::<FogState>();
        let urban_growth_boundary = world.resource::<UrbanGrowthBoundary>();

        let environment_stage = collect_environment_stage(
            Some(weather),
            Some(climate_zone),
            Some(uhi_grid),
            Some(stormwater_grid),
            Some(degree_days),
            Some(construction_modifiers),
            Some((snow_grid, snow_plowing_state)),
            Some(agriculture_state),
            Some(fog_state),
            Some(urban_growth_boundary),
        );

        let drought_state = world.resource::<DroughtState>();
        let heat_wave_state = world.resource::<HeatWaveState>();
        let cold_snap_state = world.resource::<ColdSnapState>();
        let flood_state = world.resource::<FloodState>();
        let wind_damage_state = world.resource::<WindDamageState>();
        let reservoir_state = world.resource::<ReservoirState>();
        let landfill_gas_state = world.resource::<LandfillGasState>();
        let cso_state = world.resource::<SewerSystemState>();
        let hazardous_waste_state = world.resource::<HazardousWasteState>();
        let wastewater_state = world.resource::<WastewaterState>();
        let storm_drainage_state = world.resource::<StormDrainageState>();
        let landfill_capacity_state = world.resource::<LandfillCapacityState>();
        let groundwater_depletion_state = world.resource::<GroundwaterDepletionState>();
        let water_treatment_state = world.resource::<WaterTreatmentState>();
        let water_conservation_state = world.resource::<WaterConservationState>();

        let disaster_stage = collect_disaster_stage(
            Some(drought_state),
            Some(heat_wave_state),
            Some(cold_snap_state),
            Some(flood_state),
            Some(wind_damage_state),
            Some(reservoir_state),
            Some(landfill_gas_state),
            Some(cso_state),
            Some(hazardous_waste_state),
            Some(wastewater_state),
            Some(storm_drainage_state),
            Some(landfill_capacity_state),
            Some(groundwater_depletion_state),
            Some(water_treatment_state),
            Some(water_conservation_state),
        );

        let policies = world.resource::<Policies>();
        let unlock_state = world.resource::<UnlockState>();
        let recycling_state = world.resource::<RecyclingState>();
        let recycling_economics = world.resource::<RecyclingEconomics>();
        let composting_state = world.resource::<CompostingState>();
        let lifecycle_timer = world.resource::<LifecycleTimer>();
        let life_sim_timer = world.resource::<LifeSimTimer>();
        let virtual_population = world.resource::<VirtualPopulation>();

        let policy_stage = collect_policy_stage(
            Some(policies),
            Some(unlock_state),
            Some((recycling_state, recycling_economics)),
            Some(composting_state),
            Some(lifecycle_timer),
            Some(life_sim_timer),
            Some(virtual_population),
        );

        assemble_save_data(
            grid_stage,
            economy_stage,
            entity_stage,
            environment_stage,
            disaster_stage,
            policy_stage,
        )
    };
    // -- Stage 2: Populate extension map from SaveableRegistry --
    let mut save = save;
    let registry = world
        .remove_resource::<SaveableRegistry>()
        .expect("SaveableRegistry must exist");
    save.extensions = registry.save_all(world);
    world.insert_resource(registry);

    // -- Stage 3: Encode and write to disk/IndexedDB --
    let encoded = save.encode();
    let bytes = crate::file_header::wrap_with_header(&encoded);

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = crate::save_plugin::save_file_path();
        if let Err(e) = std::fs::write(&path, &bytes) {
            eprintln!("Failed to save: {}", e);
        } else {
            println!("Saved {} bytes to {}", bytes.len(), path);
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let len = bytes.len();
        let error_slot = world
            .resource::<crate::save_plugin::WasmSaveErrorBuffer>()
            .0
            .clone();
        wasm_bindgen_futures::spawn_local(async move {
            match crate::wasm_idb::idb_save(bytes).await {
                Ok(()) => {
                    web_sys::console::log_1(&format!("Saved {} bytes to IndexedDB", len).into());
                }
                Err(e) => {
                    let msg = e.to_string();
                    web_sys::console::error_1(&msg.clone().into());
                    *error_slot.lock().unwrap() = Some(msg);
                }
            }
        });
    }

    // -- Stage 4: Transition back to Idle --
    world
        .resource_mut::<NextState<SaveLoadState>>()
        .set(SaveLoadState::Idle);
}
