use bevy::prelude::*;
use std::collections::HashSet;

mod save_codec;
mod save_migrate;
mod save_restore;
pub mod save_stages;
mod save_types;
pub mod saveable_ext;
pub mod serialization;

#[cfg(target_arch = "wasm32")]
mod wasm_idb;

pub use saveable_ext::SaveableAppExt;

use save_stages::{
    assemble_save_data, collect_disaster_stage, collect_economy_stage, collect_entity_stage,
    collect_environment_stage, collect_grid_stage, collect_policy_stage,
};

use serialization::{
    migrate_save, restore_agriculture, restore_climate_zone, restore_cold_snap, restore_composting,
    restore_construction_modifiers, restore_cso, restore_degree_days, restore_drought,
    restore_extended_budget, restore_flood_state, restore_fog_state, restore_groundwater_depletion,
    restore_hazardous_waste, restore_heat_wave, restore_landfill_capacity, restore_landfill_gas,
    restore_life_sim_timer, restore_lifecycle_timer, restore_loan_book, restore_policies,
    restore_recycling, restore_reservoir_state, restore_road_segment_store, restore_snow,
    restore_storm_drainage, restore_stormwater_grid, restore_uhi_grid, restore_unlock_state,
    restore_urban_growth_boundary, restore_virtual_population, restore_wastewater,
    restore_water_conservation, restore_water_source, restore_water_treatment, restore_weather,
    restore_wind_damage_state, u8_to_road_type, u8_to_service_type, u8_to_utility_type,
    u8_to_zone_type, CitizenSaveInput, SaveData, CURRENT_SAVE_VERSION,
};
use simulation::SaveLoadState;
use simulation::SaveableRegistry;

use simulation::agriculture::AgricultureState;
use simulation::budget::ExtendedBudget;
use simulation::buildings::{Building, MixedUseBuilding};
use simulation::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
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
use simulation::lod::LodTier;
use simulation::movement::ActivityTimer;
use simulation::policies::Policies;
use simulation::recycling::{RecyclingEconomics, RecyclingState};
use simulation::reservoir::ReservoirState;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::roads::RoadNode;
use simulation::services::ServiceBuilding;
use simulation::snow::{SnowGrid, SnowPlowingState, SnowStats};
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

use rendering::building_render::BuildingMesh3d;
use rendering::citizen_render::CitizenSprite;

// ---------------------------------------------------------------------------
// Buffer resources
// ---------------------------------------------------------------------------

/// Holds raw bytes loaded from disk (native) or IndexedDB (WASM) that the
/// exclusive load system will parse and restore.
#[derive(Resource, Default)]
struct PendingLoadBytes(Option<Vec<u8>>);

/// On WASM, holds bytes arriving from an async IndexedDB read.
/// The `poll_wasm_load` system checks this each frame and, when data arrives,
/// stores it in `PendingLoadBytes` and triggers state transition.
#[cfg(target_arch = "wasm32")]
#[derive(Resource, Default)]
struct WasmLoadBuffer(std::rc::Rc<std::cell::RefCell<Option<Result<Vec<u8>, String>>>>);

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveGameEvent>()
            .add_event::<LoadGameEvent>()
            .add_event::<NewGameEvent>()
            .init_resource::<SaveableRegistry>()
            .init_resource::<PendingLoadBytes>();

        // On WASM, register IndexedDB async load infrastructure.
        #[cfg(target_arch = "wasm32")]
        app.init_resource::<WasmLoadBuffer>();

        // Event detection: runs every frame, reads events and triggers state
        // transitions.  These are lightweight systems that only read events.
        app.add_systems(Update, (detect_save_event, detect_new_game_event));

        // Native: synchronous load event detection (reads file, stores bytes,
        // transitions to Loading state).
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, detect_load_event);

        // WASM: async two-phase load detection.
        // 1) `start_wasm_load` kicks off async IndexedDB read
        // 2) `poll_wasm_load` checks for completed read and transitions to Loading
        #[cfg(target_arch = "wasm32")]
        app.add_systems(
            Update,
            (start_wasm_load, poll_wasm_load.after(start_wasm_load)),
        );

        // Exclusive systems for each state: these run on state entry,
        // perform all work with exclusive world access, and transition back
        // to Idle.
        app.add_systems(OnEnter(SaveLoadState::Saving), exclusive_save);
        app.add_systems(OnEnter(SaveLoadState::Loading), exclusive_load);
        app.add_systems(OnEnter(SaveLoadState::NewGame), exclusive_new_game);
    }
}

#[derive(Event)]
pub struct SaveGameEvent;

#[derive(Event)]
pub struct LoadGameEvent;

#[derive(Event)]
pub struct NewGameEvent;

// ---------------------------------------------------------------------------
// Event detection systems (lightweight, run in Update)
// ---------------------------------------------------------------------------

/// Detects `SaveGameEvent` and transitions to `Saving` state.
fn detect_save_event(
    mut events: EventReader<SaveGameEvent>,
    mut next_state: ResMut<NextState<SaveLoadState>>,
) {
    if events.read().next().is_some() {
        // Drain remaining events (only process one per frame).
        events.read().for_each(drop);
        next_state.set(SaveLoadState::Saving);
    }
}

/// Detects `NewGameEvent` and transitions to `NewGame` state.
fn detect_new_game_event(
    mut events: EventReader<NewGameEvent>,
    mut next_state: ResMut<NextState<SaveLoadState>>,
) {
    if events.read().next().is_some() {
        events.read().for_each(drop);
        next_state.set(SaveLoadState::NewGame);
    }
}

/// Native: detects `LoadGameEvent`, reads save file, stores bytes, and
/// transitions to `Loading` state.
#[cfg(not(target_arch = "wasm32"))]
fn detect_load_event(
    mut events: EventReader<LoadGameEvent>,
    mut next_state: ResMut<NextState<SaveLoadState>>,
    mut pending: ResMut<PendingLoadBytes>,
) {
    if events.read().next().is_some() {
        events.read().for_each(drop);
        let path = save_file_path();
        match std::fs::read(&path) {
            Ok(bytes) => {
                pending.0 = Some(bytes);
                next_state.set(SaveLoadState::Loading);
            }
            Err(e) => {
                eprintln!("Failed to load: {}", e);
            }
        }
    }
}

/// WASM phase 1: consumes `LoadGameEvent` and kicks off an async IndexedDB read.
#[cfg(target_arch = "wasm32")]
fn start_wasm_load(mut events: EventReader<LoadGameEvent>, buffer: Res<WasmLoadBuffer>) {
    for _ in events.read() {
        let slot = buffer.0.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = wasm_idb::idb_load().await;
            *slot.borrow_mut() = Some(result);
        });
    }
}

/// WASM phase 2: polls the shared buffer; when bytes arrive, stores them in
/// `PendingLoadBytes` and transitions to `Loading` state.
#[cfg(target_arch = "wasm32")]
fn poll_wasm_load(
    buffer: Res<WasmLoadBuffer>,
    mut pending: ResMut<PendingLoadBytes>,
    mut next_state: ResMut<NextState<SaveLoadState>>,
) {
    let mut slot = buffer.0.borrow_mut();
    if let Some(result) = slot.take() {
        match result {
            Ok(bytes) => {
                pending.0 = Some(bytes);
                next_state.set(SaveLoadState::Loading);
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to load from IndexedDB: {}", e).into());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Exclusive save system
// ---------------------------------------------------------------------------

/// Exclusive system that performs the entire save operation with full world
/// access.  Runs on `OnEnter(SaveLoadState::Saving)`, then transitions back
/// to `Idle`.
fn exclusive_save(world: &mut World) {
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
    let bytes = save.encode();

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = save_file_path();
        if let Err(e) = std::fs::write(&path, &bytes) {
            eprintln!("Failed to save: {}", e);
        } else {
            println!("Saved {} bytes to {}", bytes.len(), path);
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        let len = bytes.len();
        wasm_bindgen_futures::spawn_local(async move {
            match wasm_idb::idb_save(bytes).await {
                Ok(()) => {
                    web_sys::console::log_1(&format!("Saved {} bytes to IndexedDB", len).into());
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to save to IndexedDB: {}", e).into(),
                    );
                }
            }
        });
    }

    // -- Stage 4: Transition back to Idle --
    world
        .resource_mut::<NextState<SaveLoadState>>()
        .set(SaveLoadState::Idle);
}

// ---------------------------------------------------------------------------
// Exclusive load system
// ---------------------------------------------------------------------------

/// Exclusive system that performs the entire load operation with full world
/// access.  Entity despawns are immediate (no deferred Commands).
/// Runs on `OnEnter(SaveLoadState::Loading)`, then transitions back to `Idle`.
fn exclusive_load(world: &mut World) {
    // Take pending bytes (either from native file read or WASM IndexedDB).
    let bytes = world.resource_mut::<PendingLoadBytes>().0.take();
    let Some(bytes) = bytes else {
        eprintln!("exclusive_load: no pending bytes — skipping");
        world
            .resource_mut::<NextState<SaveLoadState>>()
            .set(SaveLoadState::Idle);
        return;
    };

    // -- Stage 1: Parse and migrate --
    let mut save = match SaveData::decode(&bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to decode save: {}", e);
            world
                .resource_mut::<NextState<SaveLoadState>>()
                .set(SaveLoadState::Idle);
            return;
        }
    };

    let old_version = match migrate_save(&mut save) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Save migration failed: {}", e);
            world
                .resource_mut::<NextState<SaveLoadState>>()
                .set(SaveLoadState::Idle);
            return;
        }
    };
    if old_version != CURRENT_SAVE_VERSION {
        println!(
            "Migrated save from v{} to v{}",
            old_version, CURRENT_SAVE_VERSION
        );
    }

    // -- Stage 2: Despawn existing entities (immediate, not deferred) --
    despawn_all_game_entities(world);

    // -- Stage 3: Restore resources --
    restore_resources_from_save(world, &save);

    // -- Stage 4: Spawn entities --
    spawn_entities_from_save(world, &save);

    // -- Stage 5: Apply extension map via SaveableRegistry --
    let registry = world
        .remove_resource::<SaveableRegistry>()
        .expect("SaveableRegistry must exist");
    registry.load_all(world, &save.extensions);
    world.insert_resource(registry);

    #[cfg(not(target_arch = "wasm32"))]
    println!("Loaded save from {}", save_file_path());
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&"Loaded save from IndexedDB".into());

    // -- Stage 6: Transition back to Idle --
    world
        .resource_mut::<NextState<SaveLoadState>>()
        .set(SaveLoadState::Idle);
}

// ---------------------------------------------------------------------------
// Exclusive new-game system
// ---------------------------------------------------------------------------

/// Exclusive system that resets the world for a new game.  Entity despawns
/// are immediate (no deferred Commands).
/// Runs on `OnEnter(SaveLoadState::NewGame)`, then transitions back to `Idle`.
fn exclusive_new_game(world: &mut World) {
    // -- Stage 1: Despawn existing entities (immediate) --
    despawn_all_game_entities(world);

    // -- Stage 2: Reset all resources to defaults --
    reset_all_resources(world);

    // -- Stage 3: Reset extension-registered resources via SaveableRegistry --
    let registry = world
        .remove_resource::<SaveableRegistry>()
        .expect("SaveableRegistry must exist");
    registry.reset_all(world);
    world.insert_resource(registry);

    // -- Stage 4: Generate starter terrain --
    {
        let (width, height) = {
            let grid = world.resource::<WorldGrid>();
            (grid.width, grid.height)
        };
        let mut grid = world.resource_mut::<WorldGrid>();
        for y in 0..height {
            for x in 0..width {
                let cell = grid.get_mut(x, y);
                if x < 10 {
                    cell.cell_type = simulation::grid::CellType::Water;
                    cell.elevation = 0.3;
                } else {
                    cell.cell_type = simulation::grid::CellType::Grass;
                    cell.elevation = 0.5;
                }
            }
        }
    }

    println!("New game started — blank map with $50,000 treasury");

    // -- Stage 5: Transition back to Idle --
    world
        .resource_mut::<NextState<SaveLoadState>>()
        .set(SaveLoadState::Idle);
}

// ---------------------------------------------------------------------------
// Helper: immediate entity despawn
// ---------------------------------------------------------------------------

/// Collects all game entities (buildings, citizens, utilities, services,
/// water sources, meshes, sprites) and despawns them immediately using
/// direct world access.  This avoids the deferred-Commands race condition.
fn despawn_all_game_entities(world: &mut World) {
    let mut entities = HashSet::new();

    // Collect entities from each component query.
    let mut q = world.query_filtered::<Entity, With<Building>>();
    for e in q.iter(world) {
        entities.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<Citizen>>();
    for e in q.iter(world) {
        entities.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<UtilitySource>>();
    for e in q.iter(world) {
        entities.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<ServiceBuilding>>();
    for e in q.iter(world) {
        entities.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<WaterSource>>();
    for e in q.iter(world) {
        entities.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<BuildingMesh3d>>();
    for e in q.iter(world) {
        entities.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<CitizenSprite>>();
    for e in q.iter(world) {
        entities.insert(e);
    }

    // Despawn each entity immediately.
    for entity in entities {
        if world.get_entity(entity).is_ok() {
            world.despawn(entity);
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: restore resources from SaveData
// ---------------------------------------------------------------------------

/// Restores all core resources from a parsed SaveData.
///
/// Internally delegates to focused sub-functions:
///   - `restore_grid_and_roads`: grid cells, road network, road segments
///   - `restore_economy_state`: clock, budget, demand
///   - `restore_v2_resources`: all V2+ optional resources (policies, weather, etc.)
fn restore_resources_from_save(world: &mut World, save: &SaveData) {
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

// ---------------------------------------------------------------------------
// Helper: spawn entities from SaveData
// ---------------------------------------------------------------------------

/// Spawns all game entities from a parsed SaveData using direct world access.
fn spawn_entities_from_save(world: &mut World, save: &SaveData) {
    // Spawn buildings
    for sb in &save.buildings {
        let zone = u8_to_zone_type(sb.zone_type);
        let building = Building {
            zone_type: zone,
            level: sb.level,
            grid_x: sb.grid_x,
            grid_y: sb.grid_y,
            capacity: sb.capacity,
            occupants: sb.occupants,
        };
        let entity = if zone.is_mixed_use() {
            let (comm_cap, res_cap) = if sb.commercial_capacity > 0 || sb.residential_capacity > 0 {
                (sb.commercial_capacity, sb.residential_capacity)
            } else {
                MixedUseBuilding::capacities_for_level(sb.level)
            };
            world
                .spawn((
                    building,
                    MixedUseBuilding {
                        commercial_capacity: comm_cap,
                        commercial_occupants: sb.commercial_occupants,
                        residential_capacity: res_cap,
                        residential_occupants: sb.residential_occupants,
                    },
                ))
                .id()
        } else {
            world.spawn(building).id()
        };
        let mut grid = world.resource_mut::<WorldGrid>();
        if grid.in_bounds(sb.grid_x, sb.grid_y) {
            grid.get_mut(sb.grid_x, sb.grid_y).building_id = Some(entity);
        }
    }

    // Spawn utility sources
    for su in &save.utility_sources {
        let ut = u8_to_utility_type(su.utility_type);
        world.spawn(UtilitySource {
            utility_type: ut,
            grid_x: su.grid_x,
            grid_y: su.grid_y,
            range: su.range,
        });
    }

    // Spawn service buildings
    for ss in &save.service_buildings {
        if let Some(service_type) = u8_to_service_type(ss.service_type) {
            let radius = ServiceBuilding::coverage_radius(service_type);
            let entity = world
                .spawn(ServiceBuilding {
                    service_type,
                    grid_x: ss.grid_x,
                    grid_y: ss.grid_y,
                    radius,
                })
                .id();
            let mut grid = world.resource_mut::<WorldGrid>();
            if grid.in_bounds(ss.grid_x, ss.grid_y) {
                grid.get_mut(ss.grid_x, ss.grid_y).building_id = Some(entity);
            }
        }
    }

    // Spawn water sources
    if let Some(ref saved_water_sources) = save.water_sources {
        for sws in saved_water_sources {
            if let Some(ws) = restore_water_source(sws) {
                let entity = world.spawn(ws).id();
                let mut grid = world.resource_mut::<WorldGrid>();
                if grid.in_bounds(sws.grid_x, sws.grid_y) {
                    grid.get_mut(sws.grid_x, sws.grid_y).building_id = Some(entity);
                }
            }
        }
    }

    // Spawn citizens
    let mut citizen_entities: Vec<Entity> = Vec::with_capacity(save.citizens.len());

    // Pre-compute all citizen data in an inner scope so the grid borrow
    // ends before we call world.spawn().
    let citizen_spawn_data: Vec<_> = {
        let grid = world.resource::<WorldGrid>();
        save.citizens
            .iter()
            .map(|sc| {
                let state = match sc.state {
                    1 => CitizenState::CommutingToWork,
                    2 => CitizenState::Working,
                    3 => CitizenState::CommutingHome,
                    4 => CitizenState::CommutingToShop,
                    5 => CitizenState::Shopping,
                    6 => CitizenState::CommutingToLeisure,
                    7 => CitizenState::AtLeisure,
                    8 => CitizenState::CommutingToSchool,
                    9 => CitizenState::AtSchool,
                    _ => CitizenState::AtHome,
                };

                let home_building = if grid.in_bounds(sc.home_x, sc.home_y) {
                    grid.get(sc.home_x, sc.home_y)
                        .building_id
                        .unwrap_or(Entity::PLACEHOLDER)
                } else {
                    Entity::PLACEHOLDER
                };

                let work_building = if grid.in_bounds(sc.work_x, sc.work_y) {
                    grid.get(sc.work_x, sc.work_y)
                        .building_id
                        .unwrap_or(Entity::PLACEHOLDER)
                } else {
                    Entity::PLACEHOLDER
                };

                let (pos_x, pos_y) = if sc.pos_x != 0.0 || sc.pos_y != 0.0 {
                    (sc.pos_x, sc.pos_y)
                } else {
                    WorldGrid::grid_to_world(sc.home_x, sc.home_y)
                };

                let (path_cache, restored_state) = {
                    let waypoints: Vec<RoadNode> = sc
                        .path_waypoints
                        .iter()
                        .map(|&(x, y)| RoadNode(x, y))
                        .collect();

                    let all_valid = waypoints.iter().all(|n| grid.in_bounds(n.0, n.1));

                    if !waypoints.is_empty() && all_valid {
                        let mut pc = PathCache::new(waypoints);
                        pc.current_index = sc.path_current_index;
                        (pc, state)
                    } else if state.is_commuting() {
                        (PathCache::new(vec![]), CitizenState::AtHome)
                    } else {
                        (PathCache::new(vec![]), state)
                    }
                };

                let velocity = Velocity {
                    x: sc.velocity_x,
                    y: sc.velocity_y,
                };

                let gender = if sc.gender == 1 {
                    Gender::Female
                } else {
                    Gender::Male
                };

                let salary = if sc.salary != 0.0 {
                    sc.salary
                } else {
                    CitizenDetails::base_salary_for_education(sc.education)
                };

                let savings = if sc.savings != 0.0 {
                    sc.savings
                } else {
                    salary * 2.0
                };

                (
                    Citizen,
                    CitizenDetails {
                        age: sc.age,
                        gender,
                        happiness: sc.happiness,
                        health: sc.health,
                        education: sc.education,
                        salary,
                        savings,
                    },
                    CitizenStateComp(restored_state),
                    HomeLocation {
                        grid_x: sc.home_x,
                        grid_y: sc.home_y,
                        building: home_building,
                    },
                    WorkLocation {
                        grid_x: sc.work_x,
                        grid_y: sc.work_y,
                        building: work_building,
                    },
                    Position { x: pos_x, y: pos_y },
                    velocity,
                    path_cache,
                    Personality {
                        ambition: sc.ambition,
                        sociability: sc.sociability,
                        materialism: sc.materialism,
                        resilience: sc.resilience,
                    },
                    Needs {
                        hunger: sc.need_hunger,
                        energy: sc.need_energy,
                        social: sc.need_social,
                        fun: sc.need_fun,
                        comfort: sc.need_comfort,
                    },
                    Family::default(),
                    ActivityTimer(sc.activity_timer),
                    LodTier::default(),
                )
            })
            .collect()
    }; // grid borrow ends here

    for data in citizen_spawn_data {
        let entity = world.spawn(data).id();
        citizen_entities.push(entity);
    }

    // Second pass: restore family relationships using saved citizen indices.
    let num_citizens = citizen_entities.len();
    for (i, sc) in save.citizens.iter().enumerate() {
        let mut family = Family::default();
        if (sc.family_partner as usize) < num_citizens {
            family.partner = Some(citizen_entities[sc.family_partner as usize]);
        }
        for &child_idx in &sc.family_children {
            if (child_idx as usize) < num_citizens {
                family.children.push(citizen_entities[child_idx as usize]);
            }
        }
        if (sc.family_parent as usize) < num_citizens {
            family.parent = Some(citizen_entities[sc.family_parent as usize]);
        }
        if family.partner.is_some() || !family.children.is_empty() || family.parent.is_some() {
            if let Ok(mut entity_mut) = world.get_entity_mut(citizen_entities[i]) {
                entity_mut.insert(family);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: reset all resources for new game
// ---------------------------------------------------------------------------

fn reset_all_resources(world: &mut World) {
    {
        let (width, height) = {
            let grid = world.resource::<WorldGrid>();
            (grid.width, grid.height)
        };
        *world.resource_mut::<WorldGrid>() = WorldGrid::new(width, height);
    }
    *world.resource_mut::<RoadNetwork>() = RoadNetwork::default();
    *world.resource_mut::<RoadSegmentStore>() = RoadSegmentStore::default();

    // Reset clock
    {
        let mut clock = world.resource_mut::<GameClock>();
        clock.day = 1;
        clock.hour = 8.0;
        clock.speed = 1.0;
        clock.paused = false;
    }

    // Reset budget
    {
        let mut budget = world.resource_mut::<CityBudget>();
        budget.treasury = 50_000.0;
        budget.tax_rate = 0.10;
        budget.last_collection_day = 0;
    }

    // Reset demand
    *world.resource_mut::<ZoneDemand>() = ZoneDemand::default();

    // Reset V2 resources
    *world.resource_mut::<Policies>() = Policies::default();
    *world.resource_mut::<Weather>() = Weather::default();
    *world.resource_mut::<ClimateZone>() = ClimateZone::default();
    *world.resource_mut::<UnlockState>() = UnlockState::default();
    *world.resource_mut::<ExtendedBudget>() = ExtendedBudget::default();
    *world.resource_mut::<LoanBook>() = LoanBook::default();
    *world.resource_mut::<VirtualPopulation>() = VirtualPopulation::default();
    *world.resource_mut::<LifecycleTimer>() = LifecycleTimer::default();
    *world.resource_mut::<LifeSimTimer>() = LifeSimTimer::default();
    *world.resource_mut::<StormwaterGrid>() = StormwaterGrid::default();
    *world.resource_mut::<DegreeDays>() = DegreeDays::default();
    *world.resource_mut::<ConstructionModifiers>() = ConstructionModifiers::default();
    *world.resource_mut::<RecyclingState>() = RecyclingState::default();
    *world.resource_mut::<RecyclingEconomics>() = RecyclingEconomics::default();
    *world.resource_mut::<WindDamageState>() = WindDamageState::default();
    *world.resource_mut::<UhiGrid>() = UhiGrid::default();
    *world.resource_mut::<DroughtState>() = DroughtState::default();
    *world.resource_mut::<HeatWaveState>() = HeatWaveState::default();
    *world.resource_mut::<CompostingState>() = CompostingState::default();
    *world.resource_mut::<ColdSnapState>() = ColdSnapState::default();
    *world.resource_mut::<WaterTreatmentState>() = WaterTreatmentState::default();
    *world.resource_mut::<GroundwaterDepletionState>() = GroundwaterDepletionState::default();
    *world.resource_mut::<WastewaterState>() = WastewaterState::default();
    *world.resource_mut::<HazardousWasteState>() = HazardousWasteState::default();
    *world.resource_mut::<StormDrainageState>() = StormDrainageState::default();
    *world.resource_mut::<LandfillCapacityState>() = LandfillCapacityState::default();
    *world.resource_mut::<FloodState>() = FloodState::default();
    *world.resource_mut::<FloodGrid>() = FloodGrid::default();
    *world.resource_mut::<ReservoirState>() = ReservoirState::default();
    *world.resource_mut::<LandfillGasState>() = LandfillGasState::default();
    *world.resource_mut::<SewerSystemState>() = SewerSystemState::default();
    *world.resource_mut::<WaterConservationState>() = WaterConservationState::default();
    *world.resource_mut::<FogState>() = FogState::default();
    *world.resource_mut::<UrbanGrowthBoundary>() = UrbanGrowthBoundary::default();
    *world.resource_mut::<SnowGrid>() = SnowGrid::default();
    *world.resource_mut::<SnowPlowingState>() = SnowPlowingState::default();
    *world.resource_mut::<SnowStats>() = SnowStats::default();
    *world.resource_mut::<AgricultureState>() = AgricultureState::default();
}

#[cfg(not(target_arch = "wasm32"))]
fn save_file_path() -> String {
    "megacity_save.bin".to_string()
}
