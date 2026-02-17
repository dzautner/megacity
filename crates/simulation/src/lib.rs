use bevy::prelude::*;

pub mod budget;
pub mod building_upgrade;
pub mod buildings;
pub mod citizen;
pub mod citizen_spawner;
pub mod config;
pub mod contraction_hierarchy;
pub mod crime;
pub mod districts;
pub mod economy;
pub mod education;
pub mod garbage;
pub mod grid;
pub mod happiness;
pub mod health;
pub mod imports_exports;
pub mod land_value;
pub mod lifecycle;
pub mod lod;
pub mod movement;
pub mod natural_resources;
pub mod pathfinding_sys;
pub mod policies;
pub mod pollution;
pub mod road_graph_csr;
pub mod road_segments;
pub mod roads;
pub mod services;
pub mod spatial_grid;
pub mod stats;
pub mod terrain;
pub mod time_of_day;
pub mod tourism;
pub mod traffic;
pub mod unlocks;
pub mod utilities;
pub mod virtual_population;
pub mod wealth;
pub mod life_simulation;
pub mod weather;
pub mod zones;

use budget::ExtendedBudget;
use building_upgrade::UpgradeTimer;
use buildings::{Building, BuildingSpawnTimer};
use citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use life_simulation::LifeSimTimer;
use movement::ActivityTimer;
use citizen_spawner::CitizenSpawnTimer;
use config::{GRID_HEIGHT, GRID_WIDTH};
use crime::CrimeGrid;
use districts::Districts;
use economy::CityBudget;
use education::EducationGrid;
use garbage::GarbageGrid;
use grid::{CellType, RoadType, WorldGrid, ZoneType};
use health::HealthGrid;
use imports_exports::TradeConnections;
use land_value::LandValueGrid;
use lifecycle::LifecycleTimer;
use lod::ViewportBounds;
use natural_resources::{ResourceBalance, ResourceGrid};
use policies::Policies;
use pollution::PollutionGrid;
use road_graph_csr::CsrGraph;
use road_segments::RoadSegmentStore;
use roads::RoadNetwork;
use services::{ServiceBuilding, ServiceType};
use spatial_grid::SpatialGrid;
use stats::CityStats;
use time_of_day::GameClock;
use tourism::Tourism;
use traffic::TrafficGrid;
use unlocks::UnlockState;
use utilities::{UtilitySource, UtilityType};
use virtual_population::VirtualPopulation;
use wealth::WealthStats;
use weather::Weather;
use zones::ZoneDemand;

/// Global tick counter incremented each FixedUpdate, used for throttling simulation systems.
#[derive(Resource, Default)]
pub struct TickCounter(pub u64);

/// Shared throttle timer for grid-wide simulation systems that don't need to run every tick.
/// These systems (pollution, land value, crime, health, garbage) only run every N ticks.
#[derive(Resource, Default)]
pub struct SlowTickTimer {
    pub counter: u32,
}

impl SlowTickTimer {
    pub const INTERVAL: u32 = 100; // run slow systems every 100 ticks (~10 seconds at 10Hz)

    pub fn tick(&mut self) {
        self.counter += 1;
    }

    pub fn should_run(&self) -> bool {
        self.counter.is_multiple_of(Self::INTERVAL)
    }
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoneDemand>()
            .init_resource::<BuildingSpawnTimer>()
            .init_resource::<CitizenSpawnTimer>()
            .init_resource::<GameClock>()
            .init_resource::<CityBudget>()
            .init_resource::<CityStats>()
            .init_resource::<TrafficGrid>()
            .init_resource::<Districts>()
            .init_resource::<SpatialGrid>()
            .init_resource::<ViewportBounds>()
            .init_resource::<LifecycleTimer>()
            .init_resource::<UpgradeTimer>()
            .init_resource::<TradeConnections>()
            .init_resource::<EducationGrid>()
            .init_resource::<PollutionGrid>()
            .init_resource::<LandValueGrid>()
            .init_resource::<GarbageGrid>()
            .init_resource::<VirtualPopulation>()
            .init_resource::<Policies>()
            .init_resource::<Weather>()
            .init_resource::<ResourceGrid>()
            .init_resource::<ResourceBalance>()
            .init_resource::<ExtendedBudget>()
            .init_resource::<WealthStats>()
            .init_resource::<CrimeGrid>()
            .init_resource::<HealthGrid>()
            .init_resource::<Tourism>()
            .init_resource::<UnlockState>()
            .init_resource::<happiness::ServiceCoverageGrid>()
            .init_resource::<TickCounter>()
            .init_resource::<SlowTickTimer>()
            .init_resource::<CsrGraph>()
            .init_resource::<RoadSegmentStore>()
            .init_resource::<LifeSimTimer>()
            .init_resource::<movement::DestinationCache>()
            .add_systems(Startup, init_world)
            .add_systems(
                FixedUpdate,
                (
                    tick_slow_timer,
                    time_of_day::tick_game_clock,
                    zones::update_zone_demand,
                    buildings::building_spawner,
                    citizen_spawner::spawn_citizens,
                    movement::refresh_destination_cache,
                    movement::citizen_state_machine,
                    // apply_deferred flushes PathRequest insertions from the state machine
                    bevy::ecs::schedule::apply_deferred,
                    movement::process_path_requests,
                    movement::move_citizens,
                    traffic::update_traffic_density,
                    happiness::update_service_coverage,
                    happiness::update_happiness,
                    economy::collect_taxes,
                    stats::update_stats,
                    utilities::propagate_utilities,
                    education::propagate_education,
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                (
                    pollution::update_pollution,
                    land_value::update_land_value,
                    garbage::update_garbage,
                    districts::aggregate_districts,
                    lifecycle::age_citizens,
                    lifecycle::emigration,
                    building_upgrade::upgrade_buildings,
                    building_upgrade::downgrade_buildings,
                    imports_exports::process_trade,
                )
                    .chain()
                    .after(education::propagate_education),
            )
            .add_systems(
                FixedUpdate,
                (
                    weather::update_weather,
                    crime::update_crime,
                    health::update_health_grid,
                    natural_resources::update_resource_production,
                    wealth::update_wealth_stats,
                    tourism::update_tourism,
                    unlocks::award_development_points,
                )
                    .after(imports_exports::process_trade),
            )
            .add_systems(
                FixedUpdate,
                (
                    life_simulation::update_needs,
                    life_simulation::education_advancement,
                    life_simulation::salary_payment,
                    life_simulation::job_seeking,
                    life_simulation::life_events,
                    life_simulation::retire_workers,
                )
                    .after(happiness::update_happiness),
            )
            .add_systems(
                FixedUpdate,
                (
                    life_simulation::evolve_personality,
                    life_simulation::update_health,
                )
                    .after(life_simulation::update_needs),
            )
            .add_systems(
                Update,
                (
                    time_of_day::sync_fixed_timestep,
                    rebuild_csr_on_road_change,
                    virtual_population::adjust_real_citizen_cap
                        .run_if(bevy::time::common_conditions::on_timer(
                            std::time::Duration::from_secs(1),
                        )),
                ),
            )
            .init_resource::<LodFrameCounter>()
            .add_systems(
                Update,
                (
                    lod::update_viewport_bounds,
                    lod::update_spatial_grid.run_if(lod_frame_ready),
                    lod::assign_lod_tiers.run_if(lod_frame_ready),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    lod::compress_abstract_citizens,
                    lod::decompress_active_citizens,
                )
                    .after(lod::assign_lod_tiers),
            )
            .add_systems(Update, tick_lod_frame_counter);
    }
}

fn tick_slow_timer(mut timer: ResMut<SlowTickTimer>, mut tick: ResMut<TickCounter>) {
    timer.tick();
    tick.0 = tick.0.wrapping_add(1);
}

/// Counter for throttling LOD/spatial grid updates to every 6th render frame (~10Hz at 60fps).
#[derive(Resource, Default)]
struct LodFrameCounter(u32);

fn tick_lod_frame_counter(mut counter: ResMut<LodFrameCounter>) {
    counter.0 = counter.0.wrapping_add(1);
}

fn lod_frame_ready(counter: Res<LodFrameCounter>) -> bool {
    counter.0.is_multiple_of(6)
}

/// Rebuild the CSR graph whenever the road network changes.
fn rebuild_csr_on_road_change(
    roads: Res<RoadNetwork>,
    mut csr: ResMut<CsrGraph>,
) {
    if roads.is_changed() {
        *csr = CsrGraph::from_road_network(&roads);
    }
}

pub fn init_world(mut commands: Commands) {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    terrain::generate_terrain(&mut grid, 42);

    // Generate natural resources based on terrain elevation
    let mut resource_grid = ResourceGrid::default();
    let elevations: Vec<f32> = grid.cells.iter().map(|c| c.elevation).collect();
    natural_resources::generate_resources(&mut resource_grid, &elevations, 42);
    commands.insert_resource(resource_grid);

    let mut roads = RoadNetwork::default();

    // --- Realistic City Layout with Road Hierarchy ---
    let center_x = GRID_WIDTH / 2; // 128
    let center_y = GRID_HEIGHT / 2; // 128
    let city_radius = 50usize;
    let cx_f = center_x as f32;
    let cy_f = center_y as f32;

    // Phase 1: Road network with proper hierarchy
    // ---------------------------------------------------------------
    // 1a. Main boulevards through center (N-S and E-W)
    let road_min = center_x.saturating_sub(city_radius);
    let road_max = (center_x + city_radius).min(GRID_WIDTH - 1);
    for i in road_min..=road_max {
        place_road_typed_if_valid(&mut roads, &mut grid, i, center_y, RoadType::Boulevard);
        place_road_typed_if_valid(&mut roads, &mut grid, center_x, i, RoadType::Boulevard);
    }

    // 1b. Diagonal avenues from center (45° angles) — breaks the grid!
    let diag_len = (city_radius as f32 * 0.85) as i32;
    for &(sx, sy) in &[(1i32, 1i32), (1, -1), (-1, 1), (-1, -1)] {
        place_road_line(
            &mut roads, &mut grid,
            center_x as i32, center_y as i32,
            center_x as i32 + sx * diag_len, center_y as i32 + sy * diag_len,
            RoadType::Avenue,
        );
    }

    // 1c. Inner ring road (Avenue) at radius ~14
    for angle_step in 0..300 {
        let angle = angle_step as f32 * std::f32::consts::TAU / 300.0;
        let rx = (cx_f + angle.cos() * 14.0) as usize;
        let ry = (cy_f + angle.sin() * 14.0) as usize;
        place_road_typed_if_valid(&mut roads, &mut grid, rx, ry, RoadType::Avenue);
    }

    // 1d. Outer ring road (Highway) at radius ~38
    for angle_step in 0..500 {
        let angle = angle_step as f32 * std::f32::consts::TAU / 500.0;
        let rx = (cx_f + angle.cos() * 38.0) as usize;
        let ry = (cy_f + angle.sin() * 38.0) as usize;
        place_road_typed_if_valid(&mut roads, &mut grid, rx, ry, RoadType::Highway);
    }

    // 1e. Grid roads with VARIED spacing for different block sizes
    // Use different spacings per quadrant and distance to create variety
    for y in (center_y - city_radius)..=(center_y + city_radius) {
        for x in (center_x - city_radius)..=(center_x + city_radius) {
            if !grid.in_bounds(x, y) { continue; }
            let dx = x as f32 - cx_f;
            let dy = y as f32 - cy_f;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > city_radius as f32 { continue; }

            // Vary spacing by distance AND quadrant for asymmetric blocks
            let quadrant = if dx >= 0.0 { if dy >= 0.0 { 0 } else { 1 } } else { if dy >= 0.0 { 2 } else { 3 } };
            let rel_x = x.wrapping_sub(center_x - city_radius);
            let rel_y = y.wrapping_sub(center_y - city_radius);

            let (sp_x, sp_y) = if dist < 14.0 {
                // Downtown core: small dense blocks (3x4 and 4x3 alternating)
                if quadrant % 2 == 0 { (3, 4) } else { (4, 3) }
            } else if dist < 25.0 {
                // Inner city: medium blocks with variety
                match quadrant {
                    0 => (5, 4),
                    1 => (4, 6),
                    2 => (6, 4),
                    _ => (5, 5),
                }
            } else if dist < 38.0 {
                // Middle ring: larger blocks
                match quadrant {
                    0 => (6, 8),
                    1 => (7, 5),
                    2 => (5, 7),
                    _ => (8, 6),
                }
            } else {
                // Suburbs: large blocks with sparse grid
                match quadrant {
                    0 => (8, 10),
                    1 => (10, 7),
                    2 => (7, 9),
                    _ => (9, 8),
                }
            };

            let is_x_road = rel_x % sp_x == 0;
            let is_y_road = rel_y % sp_y == 0;

            if is_x_road || is_y_road {
                // Assign road type based on spacing (major grid lines = avenues)
                let road_type = if dist < 14.0 {
                    RoadType::Local
                } else if is_x_road && is_y_road {
                    // Intersection of grid lines — make at least one an avenue
                    RoadType::Avenue
                } else if (is_x_road && rel_x % (sp_x * 3) == 0) || (is_y_road && rel_y % (sp_y * 3) == 0) {
                    // Every 3rd grid line is an avenue (collector road)
                    RoadType::Avenue
                } else {
                    RoadType::Local
                };
                place_road_typed_if_valid(&mut roads, &mut grid, x, y, road_type);
            }
        }
    }

    // 1f. Radial connector roads at ~8 angles (spokes connecting rings)
    for spoke in 0..8 {
        let angle = spoke as f32 * std::f32::consts::TAU / 8.0;
        // Skip the main N/S/E/W boulevards and diagonals (already placed)
        let angle_deg = angle.to_degrees() % 360.0;
        let skip = [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0];
        if skip.iter().any(|&s| (angle_deg - s).abs() < 5.0) { continue; }
        // Connect inner ring to outer ring with avenue-type road
        let r_inner = 14.0;
        let r_outer = 38.0;
        let x0 = (cx_f + angle.cos() * r_inner) as i32;
        let y0 = (cy_f + angle.sin() * r_inner) as i32;
        let x1 = (cx_f + angle.cos() * r_outer) as i32;
        let y1 = (cy_f + angle.sin() * r_outer) as i32;
        place_road_line(&mut roads, &mut grid, x0, y0, x1, y1, RoadType::Avenue);
    }

    // 1g. Waterfront paths along water edges
    for y in 1..GRID_HEIGHT - 1 {
        for x in 1..GRID_WIDTH - 1 {
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Grass { continue; }
            // Check if adjacent to water
            let (n4, n4c) = grid.neighbors4(x, y);
            let adj_water = n4[..n4c].iter().any(|&(nx, ny)| {
                grid.get(nx, ny).cell_type == CellType::Water
            });
            if !adj_water { continue; }
            // Check if within city radius
            let dx = x as f32 - cx_f;
            let dy = y as f32 - cy_f;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > city_radius as f32 + 5.0 { continue; }
            // Place waterfront path
            place_road_typed_if_valid(&mut roads, &mut grid, x, y, RoadType::Path);
        }
    }

    // Phase 2: Zone with improved urban planning
    // First pass: identify cells adjacent to major roads (avenues/boulevards)
    // for commercial corridor zoning
    for y in (center_y - city_radius)..=(center_y + city_radius) {
        for x in (center_x - city_radius)..=(center_x + city_radius) {
            if !grid.in_bounds(x, y) { continue; }
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Grass || cell.zone != ZoneType::None { continue; }

            let dx = x as f32 - cx_f;
            let dy = y as f32 - cy_f;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > city_radius as f32 { continue; }

            // Central plaza: larger unzoned area at center (public square)
            if dist < 5.0 { continue; }

            // Must be adjacent to a road to be zoned
            let (n4b, n4bc) = grid.neighbors4(x, y);
            let has_road = n4b[..n4bc].iter().any(|(nx, ny)| {
                grid.get(*nx, *ny).cell_type == CellType::Road
            });
            if !has_road { continue; }

            // Check if adjacent to a major road (avenue/boulevard/highway)
            let adj_major = n4b[..n4bc].iter().any(|&(nx, ny)| {
                let nc = grid.get(nx, ny);
                nc.cell_type == CellType::Road
                    && matches!(nc.road_type, RoadType::Avenue | RoadType::Boulevard | RoadType::Highway)
            });

            // Check if near water (waterfront premium zone)
            let near_water = {
                let mut found = false;
                for wy in y.saturating_sub(3)..=(y + 3).min(GRID_HEIGHT - 1) {
                    for wx in x.saturating_sub(3)..=(x + 3).min(GRID_WIDTH - 1) {
                        if grid.get(wx, wy).cell_type == CellType::Water {
                            found = true;
                        }
                    }
                }
                found
            };

            let hash = x.wrapping_mul(31).wrapping_add(y.wrapping_mul(37));

            let zone = if dist < 14.0 {
                // Downtown core: Commercial High + Office
                if hash % 3 == 0 { ZoneType::Office } else { ZoneType::CommercialHigh }
            } else if near_water && dist < 40.0 {
                // Waterfront: premium residential + commercial
                match hash % 5 {
                    0..=1 => ZoneType::ResidentialHigh,
                    2..=3 => ZoneType::CommercialLow,
                    _ => ZoneType::Office,
                }
            } else if adj_major && dist < 38.0 {
                // Commercial corridors along major roads
                match hash % 6 {
                    0..=2 => ZoneType::CommercialLow,
                    3 => ZoneType::CommercialHigh,
                    4 => ZoneType::Office,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if dist < 25.0 {
                // Inner city: mixed residential + commercial
                match hash % 8 {
                    0..=3 => ZoneType::ResidentialHigh,
                    4..=5 => ZoneType::CommercialLow,
                    6 => ZoneType::Office,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if dist < 38.0 {
                // Middle ring: residential dominant
                match hash % 10 {
                    0..=5 => ZoneType::ResidentialHigh,
                    6..=7 => ZoneType::CommercialLow,
                    8 => ZoneType::Industrial,
                    _ => ZoneType::ResidentialLow,
                }
            } else {
                // Suburbs: low density residential + industrial clusters
                match hash % 12 {
                    0..=6 => ZoneType::ResidentialLow,
                    7 => ZoneType::CommercialLow,
                    8..=9 => ZoneType::Industrial,
                    _ => ZoneType::ResidentialLow,
                }
            };

            grid.get_mut(x, y).zone = zone;
        }
    }

    // Phase 3: Spawn buildings (~70% of zoned cells)
    let mut building_entities: Vec<(Entity, ZoneType, usize, usize, u32)> = Vec::new();

    for y in (center_y - city_radius)..=(center_y + city_radius) {
        for x in (center_x - city_radius)..=(center_x + city_radius) {
            if !grid.in_bounds(x, y) { continue; }
            let zone = grid.get(x, y).zone;
            let cell_type = grid.get(x, y).cell_type;
            if zone == ZoneType::None || cell_type != CellType::Grass { continue; }
            if grid.get(x, y).building_id.is_some() { continue; }

            let dx = x as f32 - center_x as f32;
            let dy = y as f32 - center_y as f32;
            let dist = (dx * dx + dy * dy).sqrt();

            // Fill rate varies by zone type — commercial areas are denser
            let hash = x.wrapping_mul(7).wrapping_add(y.wrapping_mul(13));
            let fill_pct = match zone {
                ZoneType::CommercialHigh | ZoneType::Office => 92,
                ZoneType::CommercialLow => 88,
                ZoneType::ResidentialHigh => 85,
                ZoneType::Industrial => 80,
                ZoneType::ResidentialLow => 75,
                _ => 70,
            };
            if hash % 100 > fill_pct { continue; }

            let level: u8 = if dist < 15.0 {
                if hash % 4 == 0 { 2 } else { 3 }
            } else if dist < 30.0 {
                match hash % 3 { 0 => 1, 1 => 2, _ => 3 }
            } else if hash % 3 == 0 { 2 } else { 1 };

            let capacity = Building::capacity_for_level(zone, level);

            let entity = commands
                .spawn(Building {
                    zone_type: zone,
                    level,
                    grid_x: x,
                    grid_y: y,
                    capacity,
                    occupants: 0,
                })
                .id();

            grid.get_mut(x, y).building_id = Some(entity);
            building_entities.push((entity, zone, x, y, capacity));
        }
    }

    // Phase 4: Place utility sources
    let utility_positions = [
        (UtilityType::PowerPlant, center_x - 40, center_y - 40),
        (UtilityType::PowerPlant, center_x + 40, center_y - 40),
        (UtilityType::PowerPlant, center_x - 40, center_y + 40),
        (UtilityType::PowerPlant, center_x + 40, center_y + 40),
        (UtilityType::PowerPlant, center_x, center_y - 25),
        (UtilityType::PowerPlant, center_x, center_y + 25),
        (UtilityType::WaterTower, center_x - 20, center_y - 20),
        (UtilityType::WaterTower, center_x + 20, center_y - 20),
        (UtilityType::WaterTower, center_x - 20, center_y + 20),
        (UtilityType::WaterTower, center_x + 20, center_y + 20),
        (UtilityType::WaterTower, center_x, center_y - 10),
        (UtilityType::WaterTower, center_x, center_y + 10),
    ];

    for (utype, ux, uy) in &utility_positions {
        if let Some((px, py)) = find_free_grass_cell(&grid, *ux, *uy, 10) {
            let range = match utype {
                UtilityType::PowerPlant => 30,
                UtilityType::WaterTower => 25,
                _ => 20,
            };
            let entity = commands
                .spawn(UtilitySource {
                    utility_type: *utype,
                    grid_x: px,
                    grid_y: py,
                    range,
                })
                .id();
            grid.get_mut(px, py).building_id = Some(entity);
        }
    }

    // Phase 5: Place service buildings in logical locations
    let service_positions = [
        (ServiceType::FireStation, center_x - 15, center_y - 15),
        (ServiceType::FireStation, center_x + 15, center_y - 15),
        (ServiceType::FireStation, center_x - 15, center_y + 15),
        (ServiceType::FireStation, center_x + 15, center_y + 15),
        (ServiceType::FireStation, center_x, center_y + 35),
        (ServiceType::FireStation, center_x, center_y - 35),
        (ServiceType::PoliceStation, center_x - 10, center_y),
        (ServiceType::PoliceStation, center_x + 10, center_y),
        (ServiceType::PoliceStation, center_x, center_y - 10),
        (ServiceType::PoliceStation, center_x, center_y + 10),
        (ServiceType::PoliceStation, center_x - 30, center_y - 30),
        (ServiceType::PoliceStation, center_x + 30, center_y + 30),
        (ServiceType::Hospital, center_x + 8, center_y + 5),
        (ServiceType::Hospital, center_x - 25, center_y - 20),
        (ServiceType::Hospital, center_x + 25, center_y + 20),
        (ServiceType::ElementarySchool, center_x - 20, center_y - 15),
        (ServiceType::ElementarySchool, center_x + 20, center_y - 15),
        (ServiceType::ElementarySchool, center_x - 20, center_y + 15),
        (ServiceType::ElementarySchool, center_x + 20, center_y + 15),
        (ServiceType::HighSchool, center_x - 12, center_y + 22),
        (ServiceType::HighSchool, center_x + 12, center_y - 22),
        (ServiceType::University, center_x - 5, center_y - 12),
        (ServiceType::SmallPark, center_x + 3, center_y + 3),
        (ServiceType::SmallPark, center_x - 3, center_y - 3),
        (ServiceType::SmallPark, center_x + 18, center_y + 8),
        (ServiceType::SmallPark, center_x - 18, center_y - 8),
        (ServiceType::SmallPark, center_x + 8, center_y - 18),
        (ServiceType::SmallPark, center_x - 8, center_y + 18),
        (ServiceType::LargePark, center_x + 30, center_y),
        (ServiceType::LargePark, center_x, center_y + 30),
        (ServiceType::LargePark, center_x - 30, center_y),
        (ServiceType::LargePark, center_x, center_y - 30),
        (ServiceType::Plaza, center_x, center_y),
        (ServiceType::Plaza, center_x + 14, center_y + 14),
        (ServiceType::Plaza, center_x - 14, center_y - 14),
        (ServiceType::Playground, center_x + 22, center_y + 22),
        (ServiceType::Playground, center_x - 22, center_y - 22),
        (ServiceType::SportsField, center_x + 35, center_y + 10),
        (ServiceType::Museum, center_x - 5, center_y + 5),
        (ServiceType::Cathedral, center_x + 5, center_y - 5),
        (ServiceType::CityHall, center_x - 2, center_y + 2),
        (ServiceType::Library, center_x + 12, center_y - 10),
        (ServiceType::TrainStation, center_x + 2, center_y - 15),
        (ServiceType::BusDepot, center_x - 18, center_y + 2),
        (ServiceType::SubwayStation, center_x - 8, center_y - 8),
        (ServiceType::SubwayStation, center_x + 8, center_y + 8),
        (ServiceType::Kindergarten, center_x + 25, center_y - 25),
        (ServiceType::Kindergarten, center_x - 25, center_y + 25),
    ];

    for (stype, sx, sy) in &service_positions {
        if let Some((px, py)) = find_free_grass_cell(&grid, *sx, *sy, 10) {
            let entity = commands
                .spawn(ServiceBuilding {
                    service_type: *stype,
                    grid_x: px,
                    grid_y: py,
                    radius: ServiceBuilding::coverage_radius(*stype),
                })
                .id();
            grid.get_mut(px, py).building_id = Some(entity);
        }
    }

    // Phase 6: Pre-spawn citizens to fill residential buildings
    // Collect work buildings first
    let work_buildings: Vec<(Entity, usize, usize)> = building_entities
        .iter()
        .filter(|(_, zt, _, _, _)| zt.is_job_zone())
        .map(|(e, _, x, y, _)| (*e, *x, *y))
        .collect();

    let residential_buildings: Vec<(Entity, usize, usize, u32)> = building_entities
        .iter()
        .filter(|(_, zt, _, _, _)| zt.is_residential())
        .map(|(e, _, x, y, cap)| (*e, *x, *y, *cap))
        .collect();

    if !work_buildings.is_empty() {
        let mut work_idx = 0usize;
        // Track work building occupancy locally
        let mut work_occupancy: Vec<u32> = vec![0; work_buildings.len()];
        let work_caps: Vec<u32> = building_entities
            .iter()
            .filter(|(_, zt, _, _, _)| zt.is_job_zone())
            .map(|(_, _, _, _, cap)| *cap)
            .collect();

        let mut citizen_count = 0u32;
        let target_pop = 1_000u32; // Spawn a small seed; burst spawner fills the rest
        let mut age_counter = 0u8;

        for (home_entity, hx, hy, cap) in &residential_buildings {
            if citizen_count >= target_pop {
                break;
            }
            // Fill each residential building to ~90% capacity
            let fill = (*cap as f32 * 0.9).ceil() as u32;
            for _ in 0..fill {
                if citizen_count >= target_pop {
                    break;
                }

                // Round-robin through work buildings, skipping full ones
                let start_idx = work_idx;
                loop {
                    if work_occupancy[work_idx] < work_caps[work_idx] {
                        break;
                    }
                    work_idx = (work_idx + 1) % work_buildings.len();
                    if work_idx == start_idx {
                        // All full, allow over-capacity
                        break;
                    }
                }

                let (work_entity, wx, wy) = work_buildings[work_idx];
                work_occupancy[work_idx] += 1;
                work_idx = (work_idx + 1) % work_buildings.len();

                let (home_wx, home_wy) = WorldGrid::grid_to_world(*hx, *hy);
                age_counter = age_counter.wrapping_add(7);
                let age = 18 + (age_counter % 47); // 18-64

                let gender = if citizen_count % 2 == 0 {
                    Gender::Male
                } else {
                    Gender::Female
                };
                let edu = match age {
                    18..=22 => (age_counter % 3).min(1),
                    23..=30 => (age_counter % 4).min(2),
                    _ => (age_counter % 5).min(3),
                };
                let salary =
                    CitizenDetails::base_salary_for_education(edu) * (1.0 + age.saturating_sub(18) as f32 * 0.01);

                commands.spawn((
                    Citizen,
                    Position {
                        x: home_wx,
                        y: home_wy,
                    },
                    Velocity { x: 0.0, y: 0.0 },
                    HomeLocation {
                        grid_x: *hx,
                        grid_y: *hy,
                        building: *home_entity,
                    },
                    WorkLocation {
                        grid_x: wx,
                        grid_y: wy,
                        building: work_entity,
                    },
                    CitizenStateComp(CitizenState::AtHome),
                    PathCache::new(Vec::new()),
                    CitizenDetails {
                        age,
                        gender,
                        education: edu,
                        happiness: 60.0,
                        health: 90.0,
                        salary,
                        savings: salary * 2.0,
                    },
                    Personality {
                        ambition: ((age_counter.wrapping_mul(3)) % 100) as f32 / 100.0,
                        sociability: ((age_counter.wrapping_mul(7)) % 100) as f32 / 100.0,
                        materialism: ((age_counter.wrapping_mul(11)) % 100) as f32 / 100.0,
                        resilience: ((age_counter.wrapping_mul(13)) % 100) as f32 / 100.0,
                    },
                    Needs::default(),
                    Family::default(),
                    ActivityTimer::default(),
                ));

                citizen_count += 1;
            }

            // Update building occupancy (will be applied when commands flush)
            // We track it via the Building component's occupants field
            // Since commands are deferred, we set this directly on the building spawn
        }

        // Re-spawn buildings with correct occupancy counts
        // We need to update the already-spawned building entities' occupancy
        // Since commands are deferred, we'll use a helper approach:
        // Despawn and re-spawn buildings with correct occupancy
        // Actually, we can use commands.entity(e).insert() to overwrite the Building component

        // Build occupancy map from citizen assignments
        let mut home_occ: std::collections::HashMap<Entity, u32> =
            std::collections::HashMap::new();
        // We know how many citizens we assigned per building
        // Recount from residential_buildings
        let mut recount = 0u32;
        for (home_entity, _, _, cap) in &residential_buildings {
            if recount >= citizen_count {
                break;
            }
            let fill = ((*cap as f32 * 0.9).ceil() as u32).min(citizen_count - recount);
            *home_occ.entry(*home_entity).or_insert(0) += fill;
            recount += fill;
        }

        for (home_entity, _, _, _) in &residential_buildings {
            if let Some(&occ) = home_occ.get(home_entity) {
                if occ > 0 {
                    // Find this building's data
                    if let Some((_, zt, gx, gy, cap)) = building_entities
                        .iter()
                        .find(|(e, _, _, _, _)| *e == *home_entity)
                    {
                        let dx2 = *gx as f32 - center_x as f32;
                        let dy2 = *gy as f32 - center_y as f32;
                        let dist2 = (dx2 * dx2 + dy2 * dy2).sqrt();
                        let hash = gx.wrapping_mul(7).wrapping_add(gy.wrapping_mul(13));
                        let level: u8 = if dist2 < 15.0 {
                            if hash % 4 == 0 { 2 } else { 3 }
                        } else if dist2 < 30.0 {
                            match hash % 3 { 0 => 1, 1 => 2, _ => 3 }
                        } else if hash % 3 == 0 { 2 } else { 1 };
                        commands.entity(*home_entity).insert(Building {
                            zone_type: *zt,
                            level,
                            grid_x: *gx,
                            grid_y: *gy,
                            capacity: *cap,
                            occupants: occ,
                        });
                    }
                }
            }
        }

        // Update work building occupancy
        for (i, (work_entity, _, _)) in work_buildings.iter().enumerate() {
            let occ = work_occupancy[i];
            if occ > 0 {
                if let Some((_, zt, gx, gy, cap)) = building_entities
                    .iter()
                    .find(|(e, _, _, _, _)| *e == *work_entity)
                {
                    let dx2 = *gx as f32 - center_x as f32;
                    let dy2 = *gy as f32 - center_y as f32;
                    let dist2 = (dx2 * dx2 + dy2 * dy2).sqrt();
                    let hash = gx.wrapping_mul(7).wrapping_add(gy.wrapping_mul(13));
                    let level: u8 = if dist2 < 15.0 {
                        if hash % 4 == 0 { 2 } else { 3 }
                    } else if dist2 < 30.0 {
                        match hash % 3 { 0 => 1, 1 => 2, _ => 3 }
                    } else if hash % 3 == 0 { 2 } else { 1 };
                    commands.entity(*work_entity).insert(Building {
                        zone_type: *zt,
                        level,
                        grid_x: *gx,
                        grid_y: *gy,
                        capacity: *cap,
                        occupants: occ,
                    });
                }
            }
        }
    }

    // Give the city a generous starting budget
    let budget = CityBudget {
        treasury: 100_000.0,
        ..CityBudget::default()
    };
    commands.insert_resource(budget);

    commands.insert_resource(grid);
    commands.insert_resource(roads);
}

fn place_road_typed_if_valid(
    roads: &mut RoadNetwork,
    grid: &mut WorldGrid,
    x: usize,
    y: usize,
    road_type: RoadType,
) {
    if grid.in_bounds(x, y) && grid.get(x, y).cell_type != CellType::Water {
        roads.place_road_typed(grid, x, y, road_type);
    }
}

/// Place a line of road cells from (x0,y0) to (x1,y1) using Bresenham's algorithm.
/// Supports diagonal and arbitrary angle roads.
fn place_road_line(
    roads: &mut RoadNetwork,
    grid: &mut WorldGrid,
    x0: i32, y0: i32,
    x1: i32, y1: i32,
    road_type: RoadType,
) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        if x >= 0 && y >= 0 {
            place_road_typed_if_valid(roads, grid, x as usize, y as usize, road_type);
            // Widen diagonal roads by 1 cell perpendicular for connectivity
            if dx > 0 && dy > 0 {
                // Add adjacent cell for wider path (prevents single-pixel diagonals)
                if dx > dy {
                    if y + 1 < GRID_HEIGHT as i32 {
                        place_road_typed_if_valid(roads, grid, x as usize, (y + 1) as usize, road_type);
                    }
                } else {
                    if x + 1 < GRID_WIDTH as i32 {
                        place_road_typed_if_valid(roads, grid, (x + 1) as usize, y as usize, road_type);
                    }
                }
            }
        }
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

/// Find a free grass cell near (cx, cy) within search_radius, spiraling outward
fn find_free_grass_cell(
    grid: &WorldGrid,
    cx: usize,
    cy: usize,
    search_radius: usize,
) -> Option<(usize, usize)> {
    for r in 0..=search_radius {
        let min_x = cx.saturating_sub(r);
        let max_x = (cx + r).min(GRID_WIDTH - 1);
        let min_y = cy.saturating_sub(r);
        let max_y = (cy + r).min(GRID_HEIGHT - 1);
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if (x != cx || y != cy) && r > 0 {
                    // Only check cells at distance r on the perimeter
                    let dx = x.abs_diff(cx);
                    let dy = y.abs_diff(cy);
                    if dx != r && dy != r {
                        continue;
                    }
                }
                let cell = grid.get(x, y);
                if cell.cell_type == CellType::Grass && cell.building_id.is_none() {
                    return Some((x, y));
                }
            }
        }
    }
    None
}
