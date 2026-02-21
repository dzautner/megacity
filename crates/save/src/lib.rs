use bevy::prelude::*;
use std::collections::HashSet;

mod save_codec;
mod save_helpers;
mod save_migrate;
mod save_restore;
mod save_types;
pub mod saveable_ext;
pub mod serialization;

#[cfg(target_arch = "wasm32")]
mod wasm_idb;

pub use saveable_ext::SaveableAppExt;

use save_helpers::V2ResourcesRead;
use serialization::{
    create_save_data, migrate_save, restore_agriculture, restore_climate_zone, restore_cold_snap,
    restore_composting, restore_construction_modifiers, restore_cso, restore_degree_days,
    restore_drought, restore_extended_budget, restore_flood_state, restore_fog_state,
    restore_groundwater_depletion, restore_hazardous_waste, restore_heat_wave,
    restore_landfill_capacity, restore_landfill_gas, restore_life_sim_timer,
    restore_lifecycle_timer, restore_loan_book, restore_policies, restore_recycling,
    restore_reservoir_state, restore_road_segment_store, restore_snow, restore_storm_drainage,
    restore_stormwater_grid, restore_uhi_grid, restore_unlock_state, restore_urban_growth_boundary,
    restore_virtual_population, restore_wastewater, restore_water_conservation,
    restore_water_source, restore_water_treatment, restore_weather, restore_wind_damage_state,
    u8_to_road_type, u8_to_service_type, u8_to_utility_type, u8_to_zone_type, CitizenSaveInput,
    SaveData, CURRENT_SAVE_VERSION,
};
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
// Extension map buffer resources
// ---------------------------------------------------------------------------

/// Holds a fully-built SaveData that still needs extension map population.
/// Written by `handle_save`, consumed by `flush_save_with_extensions`.
#[derive(Resource, Default)]
struct PendingSaveData(Option<SaveData>);

/// Holds extension map data loaded from a save file.
/// Written by `handle_load`, consumed by `apply_load_extensions`.
#[derive(Resource, Default)]
struct PendingLoadExtensions(Option<std::collections::BTreeMap<String, Vec<u8>>>);

/// Signals that a new game was started, so extension-registered resources need resetting.
/// Written by `handle_new_game`, consumed by `reset_saveable_extensions`.
#[derive(Resource, Default)]
struct PendingNewGameReset(bool);

/// On WASM, holds bytes arriving from an async IndexedDB read.
/// The `poll_wasm_load` system checks this each frame and, when data arrives,
/// fires an internal `WasmLoadReady` event so the normal restore path runs.
#[cfg(target_arch = "wasm32")]
#[derive(Resource, Default)]
struct WasmLoadBuffer(std::rc::Rc<std::cell::RefCell<Option<Result<Vec<u8>, String>>>>);

/// Internal event carrying bytes loaded asynchronously from IndexedDB.
#[cfg(target_arch = "wasm32")]
#[derive(Event)]
struct WasmLoadReady(Vec<u8>);
/// Collect all game entities into a deduplicated set for teardown.
///
/// This is the exclusive-system equivalent of the former `ExistingEntities`
/// SystemParam.  It queries all entity types that are spawned during save/load
/// and returns them in a `HashSet` so each entity is despawned at most once
/// (entities may match multiple queries, e.g. Citizen + CitizenSprite).
fn collect_game_entities(world: &mut World) -> HashSet<Entity> {
    let mut set = HashSet::new();
    let mut q = world.query_filtered::<Entity, With<Building>>();
    for e in q.iter(world) {
        set.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<Citizen>>();
    for e in q.iter(world) {
        set.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<UtilitySource>>();
    for e in q.iter(world) {
        set.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<ServiceBuilding>>();
    for e in q.iter(world) {
        set.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<WaterSource>>();
    for e in q.iter(world) {
        set.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<BuildingMesh3d>>();
    for e in q.iter(world) {
        set.insert(e);
    }
    let mut q = world.query_filtered::<Entity, With<CitizenSprite>>();
    for e in q.iter(world) {
        set.insert(e);
    }
    set
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveGameEvent>()
            .add_event::<LoadGameEvent>()
            .add_event::<NewGameEvent>()
            .init_resource::<SaveableRegistry>()
            .init_resource::<PendingSaveData>()
            .init_resource::<PendingLoadExtensions>()
            .init_resource::<PendingNewGameReset>();

        // On WASM, register IndexedDB async load infrastructure.
        #[cfg(target_arch = "wasm32")]
        app.add_event::<WasmLoadReady>()
            .init_resource::<WasmLoadBuffer>();

        app.add_systems(
            Update,
            (
                handle_save,
                handle_new_game,
                // Extension-map systems run AFTER the core handlers in the same frame.
                flush_save_with_extensions.after(handle_save),
                reset_saveable_extensions.after(handle_new_game),
            ),
        );

        // Native: synchronous load path.
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            Update,
            (handle_load, apply_load_extensions.after(handle_load)),
        );

        // WASM: async two-phase load path.
        // 1) `start_wasm_load` consumes LoadGameEvent and kicks off async IndexedDB read
        // 2) `poll_wasm_load` checks for completed read and fires WasmLoadReady
        // 3) `handle_wasm_load_ready` restores world state from the loaded bytes
        #[cfg(target_arch = "wasm32")]
        app.add_systems(
            Update,
            (
                start_wasm_load,
                poll_wasm_load.after(start_wasm_load),
                handle_wasm_load_ready.after(poll_wasm_load),
                apply_load_extensions.after(handle_wasm_load_ready),
            ),
        );
    }
}

#[derive(Event)]
pub struct SaveGameEvent;

#[derive(Event)]
pub struct LoadGameEvent;

#[derive(Event)]
pub struct NewGameEvent;

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn handle_save(
    mut events: EventReader<SaveGameEvent>,
    grid: Res<WorldGrid>,
    roads: Res<RoadNetwork>,
    segments: Res<RoadSegmentStore>,
    clock: Res<GameClock>,
    budget: Res<CityBudget>,
    demand: Res<ZoneDemand>,
    buildings: Query<(&Building, Option<&MixedUseBuilding>)>,
    citizens: Query<
        (
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
        ),
        With<Citizen>,
    >,
    utility_sources: Query<&UtilitySource>,
    service_buildings: Query<&ServiceBuilding>,
    water_sources: Query<&WaterSource>,
    v2: V2ResourcesRead,
    lifecycle_timer: Res<LifecycleTimer>,
    mut pending: ResMut<PendingSaveData>,
) {
    for _ in events.read() {
        let building_data: Vec<(Building, Option<MixedUseBuilding>)> = buildings
            .iter()
            .map(|(b, mu)| (b.clone(), mu.cloned()))
            .collect();

        let citizen_data: Vec<CitizenSaveInput> = citizens
            .iter()
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
            .collect();

        let utility_data: Vec<_> = utility_sources.iter().cloned().collect();
        let service_data: Vec<(ServiceBuilding,)> =
            service_buildings.iter().map(|sb| (sb.clone(),)).collect();
        let water_source_data: Vec<WaterSource> = water_sources.iter().cloned().collect();

        let segment_ref = if segments.segments.is_empty() {
            None
        } else {
            Some(&*segments)
        };

        let save = create_save_data(
            &grid,
            &roads,
            &clock,
            &budget,
            &demand,
            &building_data,
            &citizen_data,
            &utility_data,
            &service_data,
            segment_ref,
            Some(&v2.policies),
            Some(&v2.weather),
            Some(&v2.unlock_state),
            Some(&v2.extended_budget),
            Some(&v2.loan_book),
            Some(&lifecycle_timer),
            Some(&v2.virtual_population),
            Some(&v2.life_sim_timer),
            Some(&v2.stormwater_grid),
            if water_source_data.is_empty() {
                None
            } else {
                Some(&water_source_data)
            },
            Some(&v2.degree_days),
            Some(&v2.climate_zone),
            Some(&v2.construction_modifiers),
            Some((&v2.recycling_state, &v2.recycling_economics)),
            Some(&v2.wind_damage_state),
            Some(&v2.uhi_grid),
            Some(&v2.drought_state),
            Some(&v2.heat_wave_state),
            Some(&v2.composting_state),
            Some(&v2.cold_snap_state),
            Some(&v2.water_treatment_state),
            Some(&v2.groundwater_depletion_state),
            Some(&v2.wastewater_state),
            Some(&v2.hazardous_waste_state),
            Some(&v2.storm_drainage_state),
            Some(&v2.landfill_capacity_state),
            Some(&v2.flood_state),
            Some(&v2.reservoir_state),
            Some(&v2.landfill_gas_state),
            Some(&v2.cso_state),
            Some(&v2.water_conservation_state),
            Some(&v2.fog_state),
            Some(&v2.urban_growth_boundary),
            Some((&v2.snow_grid, &v2.snow_plowing_state)),
            Some(&v2.agriculture_state),
        );

        // Store in buffer; the exclusive flush system will add extensions and write to disk.
        pending.0 = Some(save);
    }
}

/// Handle "Load Game" (native) — exclusive system for immediate entity teardown.
#[cfg(not(target_arch = "wasm32"))]
fn handle_load(world: &mut World) {
    let has_events = !world.resource::<Events<LoadGameEvent>>().is_empty();
    if !has_events {
        return;
    }
    world.resource_mut::<Events<LoadGameEvent>>().clear();

    let bytes = {
        let path = save_file_path();
        match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to load: {}", e);
                return;
            }
        }
    };

    restore_from_bytes(world, &bytes);

    println!("Loaded save from {}", save_file_path());
}

/// Core restore logic — uses exclusive `&mut World` for immediate entity ops.
///
/// Called by both native `handle_load` and WASM `handle_wasm_load_ready`.
/// Entity despawns and spawns take effect immediately, eliminating the race
/// where deferred Commands could overlap with same-frame gameplay systems.
fn restore_from_bytes(world: &mut World, bytes: &[u8]) {
    let mut save = match SaveData::decode(bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to decode save: {}", e);
            return;
        }
    };

    // Migrate older save formats to current version
    let old_version = match migrate_save(&mut save) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Save migration failed: {}", e);
            return;
        }
    };
    if old_version != CURRENT_SAVE_VERSION {
        println!(
            "Migrated save from v{} to v{}",
            old_version, CURRENT_SAVE_VERSION
        );
    }

    // Clear existing entities -- immediate despawn via World access.
    {
        let entities = collect_game_entities(world);
        for entity in entities {
            world.despawn(entity);
        }
    }

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
        let grid_width = world.resource::<WorldGrid>().width;
        let saved_road_types: Vec<(usize, usize, u8)> = save
            .roads
            .road_positions
            .iter()
            .map(|(x, y)| {
                let idx = y * grid_width + x;
                let rt = if idx < save.grid.cells.len() {
                    save.grid.cells[idx].road_type
                } else {
                    0
                };
                (*x, *y, rt)
            })
            .collect();

        *world.resource_mut::<RoadNetwork>() = RoadNetwork::default();

        world.resource_scope(|world, mut grid: Mut<WorldGrid>| {
            world.resource_scope(|_world, mut roads: Mut<RoadNetwork>| {
                for (x, y, _) in &saved_road_types {
                    roads.place_road(&mut grid, *x, *y);
                }
                for (x, y, rt) in &saved_road_types {
                    if grid.in_bounds(*x, *y) {
                        grid.get_mut(*x, *y).road_type = u8_to_road_type(*rt);
                    }
                }
            });
        });
    }

    // Restore road segments (if present in save)
    if let Some(ref saved_segments) = save.road_segments {
        let restored = restore_road_segment_store(saved_segments);
        *world.resource_mut::<RoadSegmentStore>() = restored;
        world.resource_scope(|world, mut segments: Mut<RoadSegmentStore>| {
            world.resource_scope(|world, mut grid: Mut<WorldGrid>| {
                world.resource_scope(|_world, mut roads: Mut<RoadNetwork>| {
                    segments.rasterize_all(&mut grid, &mut roads);
                });
            });
        });
    } else {
        *world.resource_mut::<RoadSegmentStore>() = RoadSegmentStore::default();
    }

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

    // Restore buildings
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
            // Restore MixedUseBuilding component; use saved data if non-zero,
            // otherwise derive from static capacities for the level.
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
        {
            let mut grid = world.resource_mut::<WorldGrid>();
            if grid.in_bounds(sb.grid_x, sb.grid_y) {
                grid.get_mut(sb.grid_x, sb.grid_y).building_id = Some(entity);
            }
        }
    }

    // Restore utility sources
    for su in &save.utility_sources {
        let ut = u8_to_utility_type(su.utility_type);
        world.spawn(UtilitySource {
            utility_type: ut,
            grid_x: su.grid_x,
            grid_y: su.grid_y,
            range: su.range,
        });
    }

    // Restore service buildings
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
            {
                let mut grid = world.resource_mut::<WorldGrid>();
                if grid.in_bounds(ss.grid_x, ss.grid_y) {
                    grid.get_mut(ss.grid_x, ss.grid_y).building_id = Some(entity);
                }
            }
        }
    }

    // Restore water sources
    if let Some(ref saved_water_sources) = save.water_sources {
        for sws in saved_water_sources {
            if let Some(ws) = restore_water_source(sws) {
                let entity = world.spawn(ws).id();
                {
                    let mut grid = world.resource_mut::<WorldGrid>();
                    if grid.in_bounds(sws.grid_x, sws.grid_y) {
                        grid.get_mut(sws.grid_x, sws.grid_y).building_id = Some(entity);
                    }
                }
            }
        }
    }

    // Restore citizens
    let mut citizen_entities: Vec<Entity> = Vec::with_capacity(save.citizens.len());
    for sc in &save.citizens {
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

        // We need building entities for home/work locations.
        // Find them from the grid if possible, otherwise use a dummy.
        let home_building = {
            let grid = world.resource::<WorldGrid>();
            if grid.in_bounds(sc.home_x, sc.home_y) {
                grid.get(sc.home_x, sc.home_y)
                    .building_id
                    .unwrap_or(Entity::PLACEHOLDER)
            } else {
                Entity::PLACEHOLDER
            }
        };

        let work_building = {
            let grid = world.resource::<WorldGrid>();
            if grid.in_bounds(sc.work_x, sc.work_y) {
                grid.get(sc.work_x, sc.work_y)
                    .building_id
                    .unwrap_or(Entity::PLACEHOLDER)
            } else {
                Entity::PLACEHOLDER
            }
        };

        // Restore position: use saved position if available (non-zero),
        // otherwise fall back to home grid position (backward compat).
        let (pos_x, pos_y) = if sc.pos_x != 0.0 || sc.pos_y != 0.0 {
            (sc.pos_x, sc.pos_y)
        } else {
            WorldGrid::grid_to_world(sc.home_x, sc.home_y)
        };

        // Restore path cache: convert saved waypoints to RoadNodes and
        // validate that all waypoints reference valid grid positions.
        let (path_cache, restored_state) = {
            let grid = world.resource::<WorldGrid>();
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

        // Restore velocity from saved data (defaults to zero for old saves).
        let velocity = Velocity {
            x: sc.velocity_x,
            y: sc.velocity_y,
        };

        // Restore gender from saved value; fall back to age parity for old saves
        let gender = if sc.gender == 1 {
            Gender::Female
        } else {
            Gender::Male
        };

        // Restore salary: use saved value if non-zero, otherwise derive from education
        let salary = if sc.salary != 0.0 {
            sc.salary
        } else {
            CitizenDetails::base_salary_for_education(sc.education)
        };

        // Restore savings: use saved value if non-zero, otherwise derive from salary
        let savings = if sc.savings != 0.0 {
            sc.savings
        } else {
            salary * 2.0
        };

        let cit_entity = world
            .spawn((
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
            ))
            .id();
        citizen_entities.push(cit_entity);
    }

    // Second pass: restore family relationships using saved citizen indices.
    // Each SaveCitizen stores partner/children/parent as indices into the
    // citizen array. Convert those indices to the new Entity IDs.
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
        // Only update if there are actual relationships to restore
        if family.partner.is_some() || !family.children.is_empty() || family.parent.is_some() {
            if let Ok(mut ec) = world.get_entity_mut(citizen_entities[i]) {
                ec.insert(family);
            }
        }
    }

    // Restore V2 fields (policies, weather, unlocks, extended budget, loans)
    // If the save is from V1 (fields are None), use defaults.
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

    // Restore lifecycle timer (prevents mass aging/death burst on load)
    if let Some(ref saved_timer) = save.lifecycle_timer {
        *world.resource_mut::<LifecycleTimer>() = restore_lifecycle_timer(saved_timer);
    } else {
        // Old save without lifecycle timer: set last_aging_day to current day
        // to prevent immediate aging burst on load.
        let day = world.resource::<GameClock>().day;
        let mut timer = world.resource_mut::<LifecycleTimer>();
        timer.last_aging_day = day;
        timer.last_emigration_tick = 0;
    }

    // Restore virtual population (prevents population count mismatch on load)
    if let Some(ref saved_vp) = save.virtual_population {
        *world.resource_mut::<VirtualPopulation>() = restore_virtual_population(saved_vp);
    } else {
        *world.resource_mut::<VirtualPopulation>() = VirtualPopulation::default();
    }

    // Restore life sim timer (prevents all life events firing simultaneously on load)
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

    // Restore degree days (HDD/CDD tracking)
    if let Some(ref saved_dd) = save.degree_days {
        *world.resource_mut::<DegreeDays>() = restore_degree_days(saved_dd);
    } else {
        *world.resource_mut::<DegreeDays>() = DegreeDays::default();
    }

    // Restore construction modifiers (recomputed each tick from weather, but
    // persisting avoids a 1-tick stale value after load).
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
    // FloodGrid is transient, always reset to default
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

    // Store extension map for the exclusive system to apply via SaveableRegistry.
    // Always enqueue -- even an empty map -- so that registered resources whose
    // keys are absent get reset to defaults (prevents cross-save contamination).
    world.resource_mut::<PendingLoadExtensions>().0 = Some(save.extensions.clone());
}

// ---------------------------------------------------------------------------
// WASM: async IndexedDB load systems
// ---------------------------------------------------------------------------

/// Phase 1: consumes `LoadGameEvent` and kicks off an async IndexedDB read.
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

/// Phase 2: polls the shared buffer; when bytes arrive, fires `WasmLoadReady`.
#[cfg(target_arch = "wasm32")]
fn poll_wasm_load(buffer: Res<WasmLoadBuffer>, mut ready_events: EventWriter<WasmLoadReady>) {
    let mut slot = buffer.0.borrow_mut();
    if let Some(result) = slot.take() {
        match result {
            Ok(bytes) => {
                ready_events.send(WasmLoadReady(bytes));
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to load from IndexedDB: {}", e).into());
            }
        }
    }
}

/// Phase 3: restores world state from the bytes loaded by IndexedDB.
/// Exclusive system for immediate entity teardown.
#[cfg(target_arch = "wasm32")]
fn handle_wasm_load_ready(world: &mut World) {
    // Extract bytes from the first WasmLoadReady event (if any), then clear.
    let bytes = {
        let events = world.resource::<Events<WasmLoadReady>>();
        let mut cursor = events.get_cursor();
        cursor.read(events).next().map(|e| e.0.clone())
    };
    let Some(bytes) = bytes else {
        return;
    };
    world.resource_mut::<Events<WasmLoadReady>>().clear();

    restore_from_bytes(world, &bytes);

    web_sys::console::log_1(&"Loaded save from IndexedDB".into());
}

/// Handle "New Game" -- exclusive system for immediate entity teardown.
///
/// Uses `&mut World` so that entity despawns take effect instantly, preventing
/// same-frame races with gameplay/render systems.
fn handle_new_game(world: &mut World) {
    {
        let mut events = world.resource_mut::<Events<NewGameEvent>>();
        if events.is_empty() {
            return;
        }
        events.clear();
    }

    // Immediate entity teardown.
    {
        let entities = collect_game_entities(world);
        for entity in entities {
            world.despawn(entity);
        }
    }

    let (width, height) = {
        let grid = world.resource::<WorldGrid>();
        (grid.width, grid.height)
    };
    *world.resource_mut::<WorldGrid>() = WorldGrid::new(width, height);
    *world.resource_mut::<RoadNetwork>() = RoadNetwork::default();
    *world.resource_mut::<RoadSegmentStore>() = RoadSegmentStore::default();

    {
        let mut clock = world.resource_mut::<GameClock>();
        clock.day = 1;
        clock.hour = 8.0;
        clock.speed = 1.0;
        clock.paused = false;
    }

    {
        let mut budget = world.resource_mut::<CityBudget>();
        budget.treasury = 50_000.0;
        budget.tax_rate = 0.10;
        budget.last_collection_day = 0;
    }

    *world.resource_mut::<ZoneDemand>() = ZoneDemand::default();

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

    world.resource_mut::<PendingNewGameReset>().0 = true;

    {
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
}

// ---------------------------------------------------------------------------
// Exclusive systems for extension map save/load/reset
// ---------------------------------------------------------------------------

/// Exclusive system: takes the pending SaveData, populates extensions from the
/// SaveableRegistry, encodes, and writes the final save file to disk.
fn flush_save_with_extensions(world: &mut World) {
    // Take the pending save data (if any).
    let save_opt = world.resource_mut::<PendingSaveData>().0.take();
    let Some(mut save) = save_opt else {
        return;
    };

    // Temporarily remove the registry so we can read resources from the world
    // without conflicting borrows.
    let registry = world
        .remove_resource::<SaveableRegistry>()
        .expect("SaveableRegistry must exist");
    save.extensions = registry.save_all(world);
    world.insert_resource(registry);

    // Encode and write.
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
}

/// Exclusive system: applies pending extension map data to the world via
/// the SaveableRegistry after `handle_load` has restored all named fields.
fn apply_load_extensions(world: &mut World) {
    let ext_opt = world.resource_mut::<PendingLoadExtensions>().0.take();
    let Some(extensions) = ext_opt else {
        return;
    };

    // Temporarily remove the registry from the world so we can iterate its
    // entries while mutating the world (the entries themselves are never
    // modified, only the world's other resources).
    let registry = world
        .remove_resource::<SaveableRegistry>()
        .expect("SaveableRegistry must exist");
    registry.load_all(world, &extensions);
    world.insert_resource(registry);
}

/// Exclusive system: resets all extension-registered resources to defaults
/// after `handle_new_game` has reset the named resources.
fn reset_saveable_extensions(world: &mut World) {
    let should_reset = world.resource_mut::<PendingNewGameReset>().0;
    if !should_reset {
        return;
    }
    world.resource_mut::<PendingNewGameReset>().0 = false;

    let registry = world
        .remove_resource::<SaveableRegistry>()
        .expect("SaveableRegistry must exist");
    registry.reset_all(world);
    world.insert_resource(registry);
}

#[cfg(not(target_arch = "wasm32"))]
fn save_file_path() -> String {
    "megacity_save.bin".to_string()
}
