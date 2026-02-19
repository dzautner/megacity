use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

pub mod serialization;

use serialization::{
    create_save_data, restore_extended_budget, restore_lifecycle_timer, restore_loan_book,
    restore_policies, restore_road_segment_store, restore_unlock_state,
    restore_virtual_population, restore_weather, u8_to_road_type, u8_to_service_type,
    u8_to_utility_type, u8_to_zone_type, CitizenSaveInput, SaveData,
};
use simulation::budget::ExtendedBudget;
use simulation::buildings::Building;
use simulation::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use simulation::economy::CityBudget;
use simulation::grid::WorldGrid;
use simulation::lifecycle::LifecycleTimer;
use simulation::loans::LoanBook;
use simulation::movement::ActivityTimer;
use simulation::policies::Policies;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::roads::RoadNode;
use simulation::services::ServiceBuilding;
use simulation::time_of_day::GameClock;
use simulation::unlocks::UnlockState;
use simulation::utilities::UtilitySource;
use simulation::virtual_population::VirtualPopulation;
use simulation::weather::Weather;
use simulation::zones::ZoneDemand;

use rendering::building_render::BuildingMesh3d;
use rendering::citizen_render::CitizenSprite;

// ---------------------------------------------------------------------------
// SystemParam bundles to keep system parameter counts under Bevy's 16 limit
// ---------------------------------------------------------------------------

/// Read-only access to the V2 resources (policies, weather, unlocks, ext budget, loans, virtual pop).
#[derive(SystemParam)]
struct V2ResourcesRead<'w> {
    policies: Res<'w, Policies>,
    weather: Res<'w, Weather>,
    unlock_state: Res<'w, UnlockState>,
    extended_budget: Res<'w, ExtendedBudget>,
    loan_book: Res<'w, LoanBook>,
    virtual_population: Res<'w, VirtualPopulation>,
}

/// Mutable access to the V2 resources.
#[derive(SystemParam)]
struct V2ResourcesWrite<'w> {
    policies: ResMut<'w, Policies>,
    weather: ResMut<'w, Weather>,
    unlock_state: ResMut<'w, UnlockState>,
    extended_budget: ResMut<'w, ExtendedBudget>,
    loan_book: ResMut<'w, LoanBook>,
    virtual_population: ResMut<'w, VirtualPopulation>,
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveGameEvent>()
            .add_event::<LoadGameEvent>()
            .add_event::<NewGameEvent>()
            .add_systems(Update, (handle_save, handle_load, handle_new_game));
    }
}

#[derive(Event)]
pub struct SaveGameEvent;

#[derive(Event)]
pub struct LoadGameEvent;

#[derive(Event)]
pub struct NewGameEvent;

#[allow(clippy::too_many_arguments)]
fn handle_save(
    mut events: EventReader<SaveGameEvent>,
    grid: Res<WorldGrid>,
    roads: Res<RoadNetwork>,
    segments: Res<RoadSegmentStore>,
    clock: Res<GameClock>,
    budget: Res<CityBudget>,
    demand: Res<ZoneDemand>,
    buildings: Query<&Building>,
    citizens: Query<
        (
            &CitizenDetails,
            &CitizenStateComp,
            &HomeLocation,
            &WorkLocation,
            &PathCache,
            &Velocity,
            &Position,
        ),
        With<Citizen>,
    >,
    utility_sources: Query<&UtilitySource>,
    service_buildings: Query<&ServiceBuilding>,
    v2: V2ResourcesRead,
    lifecycle_timer: Res<LifecycleTimer>,
) {
    for _ in events.read() {
        let building_data: Vec<(Building,)> = buildings.iter().map(|b| (b.clone(),)).collect();

        let citizen_data: Vec<CitizenSaveInput> = citizens
            .iter()
            .map(|(d, state, home, work, path, vel, pos)| {
                CitizenSaveInput {
                    details: d.clone(),
                    state: state.0,
                    home_x: home.grid_x,
                    home_y: home.grid_y,
                    work_x: work.grid_x,
                    work_y: work.grid_y,
                    path: path.clone(),
                    velocity: vel.clone(),
                    position: pos.clone(),
                }
            })
            .collect();

        let utility_data: Vec<_> = utility_sources.iter().cloned().collect();
        let service_data: Vec<(ServiceBuilding,)> =
            service_buildings.iter().map(|sb| (sb.clone(),)).collect();

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
        );

        let bytes = save.encode();

        // Save to file
        let path = save_file_path();
        if let Err(e) = std::fs::write(&path, &bytes) {
            eprintln!("Failed to save: {}", e);
        } else {
            println!("Saved {} bytes to {}", bytes.len(), path);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_load(
    mut events: EventReader<LoadGameEvent>,
    mut commands: Commands,
    mut grid: ResMut<WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
    mut segments: ResMut<RoadSegmentStore>,
    mut clock: ResMut<GameClock>,
    mut budget: ResMut<CityBudget>,
    mut demand: ResMut<ZoneDemand>,
    existing_buildings: Query<Entity, With<Building>>,
    existing_citizens: Query<Entity, With<Citizen>>,
    existing_utilities: Query<Entity, With<UtilitySource>>,
    existing_services: Query<Entity, With<ServiceBuilding>>,
    existing_meshes: Query<Entity, With<BuildingMesh3d>>,
    existing_sprites: Query<Entity, With<CitizenSprite>>,
    mut v2: V2ResourcesWrite,
    mut lifecycle_timer: ResMut<LifecycleTimer>,
) {
    for _ in events.read() {
        let path = save_file_path();
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to load: {}", e);
                continue;
            }
        };

        let save = match SaveData::decode(&bytes) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to decode save: {}", e);
                continue;
            }
        };

        // Clear existing entities (including 3D mesh representations)
        for entity in &existing_meshes {
            commands.entity(entity).despawn();
        }
        for entity in &existing_sprites {
            commands.entity(entity).despawn();
        }
        for entity in &existing_buildings {
            commands.entity(entity).despawn();
        }
        for entity in &existing_citizens {
            commands.entity(entity).despawn();
        }
        for entity in &existing_utilities {
            commands.entity(entity).despawn();
        }
        for entity in &existing_services {
            commands.entity(entity).despawn();
        }

        // Restore grid
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

        // Restore roads - use saved road types, not default Local
        *roads = RoadNetwork::default();
        // Save the road types before place_road overwrites them
        let saved_road_types: Vec<(usize, usize, u8)> = save.roads.road_positions.iter()
            .map(|(x, y)| {
                let idx = y * grid.width + x;
                let rt = if idx < save.grid.cells.len() { save.grid.cells[idx].road_type } else { 0 };
                (*x, *y, rt)
            })
            .collect();
        for (x, y, _) in &saved_road_types {
            roads.place_road(&mut grid, *x, *y);
        }
        // Restore the saved road types (place_road overwrites with Local)
        for (x, y, rt) in &saved_road_types {
            if grid.in_bounds(*x, *y) {
                grid.get_mut(*x, *y).road_type = u8_to_road_type(*rt);
            }
        }

        // Restore road segments (if present in save)
        if let Some(ref saved_segments) = save.road_segments {
            let mut restored = restore_road_segment_store(saved_segments);
            restored.rasterize_all(&mut grid, &mut roads);
            *segments = restored;
        } else {
            *segments = RoadSegmentStore::default();
        }

        // Restore clock
        clock.day = save.clock.day;
        clock.hour = save.clock.hour;
        clock.speed = save.clock.speed;
        clock.paused = false;

        // Restore budget
        budget.treasury = save.budget.treasury;
        budget.tax_rate = save.budget.tax_rate;
        budget.last_collection_day = save.budget.last_collection_day;

        // Restore demand
        demand.residential = save.demand.residential;
        demand.commercial = save.demand.commercial;
        demand.industrial = save.demand.industrial;
        demand.office = save.demand.office;

        // Restore buildings
        for sb in &save.buildings {
            let zone = u8_to_zone_type(sb.zone_type);
            let entity = commands
                .spawn(Building {
                    zone_type: zone,
                    level: sb.level,
                    grid_x: sb.grid_x,
                    grid_y: sb.grid_y,
                    capacity: sb.capacity,
                    occupants: sb.occupants,
                })
                .id();
            if grid.in_bounds(sb.grid_x, sb.grid_y) {
                grid.get_mut(sb.grid_x, sb.grid_y).building_id = Some(entity);
            }
        }

        // Restore utility sources
        for su in &save.utility_sources {
            let ut = u8_to_utility_type(su.utility_type);
            commands.spawn(UtilitySource {
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
                let entity = commands
                    .spawn(ServiceBuilding {
                        service_type,
                        grid_x: ss.grid_x,
                        grid_y: ss.grid_y,
                        radius,
                    })
                    .id();
                if grid.in_bounds(ss.grid_x, ss.grid_y) {
                    grid.get_mut(ss.grid_x, ss.grid_y).building_id = Some(entity);
                }
            }
        }

        // Restore citizens
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
            let home_building = if grid.in_bounds(sc.home_x, sc.home_y) {
                grid.get(sc.home_x, sc.home_y).building_id.unwrap_or(Entity::PLACEHOLDER)
            } else {
                Entity::PLACEHOLDER
            };

            let work_building = if grid.in_bounds(sc.work_x, sc.work_y) {
                grid.get(sc.work_x, sc.work_y).building_id.unwrap_or(Entity::PLACEHOLDER)
            } else {
                Entity::PLACEHOLDER
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
                let waypoints: Vec<RoadNode> = sc.path_waypoints
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

            let salary = CitizenDetails::base_salary_for_education(sc.education);
            commands.spawn((
                Citizen,
                CitizenDetails {
                    age: sc.age,
                    gender: if sc.age % 2 == 0 { Gender::Male } else { Gender::Female },
                    happiness: sc.happiness,
                    health: 80.0,
                    education: sc.education,
                    salary,
                    savings: salary * 2.0,
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
                    ambition: 0.5,
                    sociability: 0.5,
                    materialism: 0.5,
                    resilience: 0.5,
                },
                Needs::default(),
                Family::default(),
                ActivityTimer::default(),
            ));
        }

        // Restore V2 fields (policies, weather, unlocks, extended budget, loans)
        // If the save is from V1 (fields are None), use defaults.
        if let Some(ref saved_policies) = save.policies {
            *v2.policies = restore_policies(saved_policies);
        } else {
            *v2.policies = Policies::default();
        }

        if let Some(ref saved_weather) = save.weather {
            *v2.weather = restore_weather(saved_weather);
        } else {
            *v2.weather = Weather::default();
        }

        if let Some(ref saved_unlocks) = save.unlock_state {
            *v2.unlock_state = restore_unlock_state(saved_unlocks);
        } else {
            *v2.unlock_state = UnlockState::default();
        }

        if let Some(ref saved_ext_budget) = save.extended_budget {
            *v2.extended_budget = restore_extended_budget(saved_ext_budget);
        } else {
            *v2.extended_budget = ExtendedBudget::default();
        }

        if let Some(ref saved_loans) = save.loan_book {
            *v2.loan_book = restore_loan_book(saved_loans);
        } else {
            *v2.loan_book = LoanBook::default();
        }

        // Restore lifecycle timer (prevents mass aging/death burst on load)
        if let Some(ref saved_timer) = save.lifecycle_timer {
            *lifecycle_timer = restore_lifecycle_timer(saved_timer);
        } else {
            // Old save without lifecycle timer: set last_aging_day to current day
            // to prevent immediate aging burst on load.
            lifecycle_timer.last_aging_day = clock.day;
            lifecycle_timer.last_emigration_tick = 0;
        }

        // Restore virtual population (prevents population count mismatch on load)
        if let Some(ref saved_vp) = save.virtual_population {
            *v2.virtual_population = restore_virtual_population(saved_vp);
        } else {
            *v2.virtual_population = VirtualPopulation::default();
        }

        println!("Loaded save from {}", path);
    }
}

/// Handle "New Game" -- despawn all entities, reset all resources, regenerate world.
#[allow(clippy::too_many_arguments)]
fn handle_new_game(
    mut events: EventReader<NewGameEvent>,
    mut commands: Commands,
    existing_buildings: Query<Entity, With<Building>>,
    existing_citizens: Query<Entity, With<Citizen>>,
    existing_utilities: Query<Entity, With<UtilitySource>>,
    existing_services: Query<Entity, With<ServiceBuilding>>,
    existing_meshes: Query<Entity, With<BuildingMesh3d>>,
    existing_sprites: Query<Entity, With<CitizenSprite>>,
    mut grid: ResMut<WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
    mut segments: ResMut<RoadSegmentStore>,
    mut clock: ResMut<GameClock>,
    mut budget: ResMut<CityBudget>,
    mut demand: ResMut<ZoneDemand>,
    mut v2: V2ResourcesWrite,
    mut lifecycle_timer: ResMut<LifecycleTimer>,
) {
    for _ in events.read() {
        // Despawn all game entities
        for entity in &existing_meshes {
            commands.entity(entity).despawn();
        }
        for entity in &existing_sprites {
            commands.entity(entity).despawn();
        }
        for entity in &existing_buildings {
            commands.entity(entity).despawn();
        }
        for entity in &existing_citizens {
            commands.entity(entity).despawn();
        }
        for entity in &existing_utilities {
            commands.entity(entity).despawn();
        }
        for entity in &existing_services {
            commands.entity(entity).despawn();
        }

        // Reset world grid to fresh empty terrain
        let width = grid.width;
        let height = grid.height;
        *grid = WorldGrid::new(width, height);
        *roads = RoadNetwork::default();
        *segments = RoadSegmentStore::default();

        // Reset clock
        clock.day = 1;
        clock.hour = 8.0;
        clock.speed = 1.0;
        clock.paused = false;

        // Reset budget to starting money
        budget.treasury = 50_000.0;
        budget.tax_rate = 0.10;
        budget.last_collection_day = 0;

        // Reset demand
        *demand = ZoneDemand::default();

        // Reset V2 resources
        *v2.policies = Policies::default();
        *v2.weather = Weather::default();
        *v2.unlock_state = UnlockState::default();
        *v2.extended_budget = ExtendedBudget::default();
        *v2.loan_book = LoanBook::default();
        *v2.virtual_population = VirtualPopulation::default();
        *lifecycle_timer = LifecycleTimer::default();

        // Generate a flat terrain with water on west edge (simple starter map)
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

        println!("New game started — blank map with $50,000 treasury");
    }
}

fn save_file_path() -> String {
    "megacity_save.bin".to_string()
}
