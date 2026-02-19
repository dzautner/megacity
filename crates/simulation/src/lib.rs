use bevy::prelude::*;

pub mod abandonment;
pub mod achievements;
pub mod advisors;
pub mod airport;
pub mod budget;
pub mod building_upgrade;
pub mod buildings;
pub mod citizen;
pub mod citizen_spawner;
pub mod config;
pub mod crime;
pub mod death_care;
pub mod disasters;
pub mod districts;
pub mod economy;
pub mod education;
pub mod education_jobs;
pub mod events;
pub mod fire;
pub mod forest_fire;
pub mod garbage;
pub mod grid;
pub mod groundwater;
pub mod happiness;
pub mod health;
pub mod heating;
pub mod homelessness;
pub mod immigration;
pub mod imports_exports;
pub mod land_value;
pub mod life_simulation;
pub mod lifecycle;
pub mod loans;
pub mod lod;
pub mod market;
pub mod movement;
pub mod natural_resources;
pub mod noise;
pub mod outside_connections;
pub mod pathfinding_sys;
pub mod policies;
pub mod pollution;
pub mod postal;
pub mod production;
pub mod road_graph_csr;
pub mod road_maintenance;
pub mod road_segments;
pub mod roads;
pub mod services;
pub mod spatial_grid;
pub mod specialization;
pub mod stats;
pub mod terrain;
pub mod time_of_day;
pub mod tourism;
pub mod traffic;
pub mod traffic_accidents;
pub mod trees;
pub mod unlocks;
pub mod utilities;
pub mod virtual_population;
pub mod water_demand;
pub mod water_pollution;
pub mod wealth;
pub mod weather;
pub mod welfare;
pub mod wind;
pub mod zones;

use achievements::{AchievementNotification, AchievementTracker};
use advisors::AdvisorPanel;
use airport::AirportStats;
use budget::ExtendedBudget;
use building_upgrade::UpgradeTimer;
use buildings::{Building, BuildingSpawnTimer, EligibleCells};
use citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use citizen_spawner::CitizenSpawnTimer;
use config::{GRID_HEIGHT, GRID_WIDTH};
use crime::CrimeGrid;
use death_care::{DeathCareGrid, DeathCareStats};
use disasters::ActiveDisaster;
use districts::{DistrictMap, Districts};
use economy::CityBudget;
use education::EducationGrid;
use education_jobs::EmploymentStats;
use events::{ActiveCityEffects, EventJournal, MilestoneTracker};
use fire::FireGrid;
use forest_fire::{ForestFireGrid, ForestFireStats};
use garbage::{GarbageGrid, WasteCollectionGrid, WasteSystem};
use grid::{CellType, RoadType, WorldGrid, ZoneType};
use groundwater::{GroundwaterGrid, GroundwaterStats, WaterQualityGrid};
use health::HealthGrid;
use heating::{HeatingGrid, HeatingStats};
use imports_exports::TradeConnections;
use land_value::LandValueGrid;
use life_simulation::LifeSimTimer;
use lifecycle::LifecycleTimer;
use loans::{BankruptcyEvent, LoanBook};
use lod::ViewportBounds;
use market::MarketPrices;
use movement::ActivityTimer;
use natural_resources::{ResourceBalance, ResourceGrid};
use noise::NoisePollutionGrid;
use outside_connections::OutsideConnections;
use policies::Policies;
use pollution::PollutionGrid;
use road_graph_csr::CsrGraph;
use road_maintenance::{RoadConditionGrid, RoadMaintenanceBudget, RoadMaintenanceStats};
use road_segments::RoadSegmentStore;
use roads::RoadNetwork;
use services::{ServiceBuilding, ServiceType};
use spatial_grid::SpatialGrid;
use specialization::{CitySpecializations, SpecializationBonuses};
use stats::CityStats;
use time_of_day::GameClock;
use tourism::Tourism;
use traffic::TrafficGrid;
use traffic_accidents::AccidentTracker;
use trees::TreeGrid;
use unlocks::UnlockState;
use utilities::{UtilitySource, UtilityType};
use virtual_population::VirtualPopulation;
use water_demand::WaterSupply;
use water_pollution::WaterPollutionGrid;
use wealth::WealthStats;
use weather::Weather;
use wind::WindState;
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
            .init_resource::<EligibleCells>()
            .init_resource::<CitizenSpawnTimer>()
            .init_resource::<GameClock>()
            .init_resource::<CityBudget>()
            .init_resource::<CityStats>()
            .init_resource::<TrafficGrid>()
            .init_resource::<Districts>()
            .init_resource::<DistrictMap>()
            .init_resource::<SpatialGrid>()
            .init_resource::<ViewportBounds>()
            .init_resource::<LifecycleTimer>()
            .init_resource::<UpgradeTimer>()
            .init_resource::<TradeConnections>()
            .init_resource::<EducationGrid>()
            .init_resource::<PollutionGrid>()
            .init_resource::<LandValueGrid>()
            .init_resource::<GarbageGrid>()
            .init_resource::<WasteSystem>()
            .init_resource::<WasteCollectionGrid>()
            .init_resource::<VirtualPopulation>()
            .init_resource::<Policies>()
            .init_resource::<Weather>()
            .init_resource::<ResourceGrid>()
            .init_resource::<ResourceBalance>()
            .init_resource::<ExtendedBudget>()
            .init_resource::<WealthStats>()
            .init_resource::<CrimeGrid>()
            .init_resource::<FireGrid>()
            .init_resource::<ForestFireGrid>()
            .init_resource::<ForestFireStats>()
            .init_resource::<NoisePollutionGrid>()
            .init_resource::<HealthGrid>()
            .init_resource::<WaterPollutionGrid>()
            .init_resource::<Tourism>()
            .init_resource::<UnlockState>()
            .init_resource::<happiness::ServiceCoverageGrid>()
            .init_resource::<TickCounter>()
            .init_resource::<SlowTickTimer>()
            .init_resource::<CsrGraph>()
            .init_resource::<RoadSegmentStore>()
            .init_resource::<LifeSimTimer>()
            .init_resource::<movement::DestinationCache>()
            .init_resource::<EventJournal>()
            .init_resource::<ActiveCityEffects>()
            .init_resource::<MilestoneTracker>()
            .init_resource::<ActiveDisaster>()
            .init_resource::<LoanBook>()
            .init_resource::<homelessness::HomelessnessStats>()
            .init_resource::<welfare::WelfareStats>()
            .init_resource::<immigration::CityAttractiveness>()
            .init_resource::<immigration::ImmigrationStats>()
            .init_resource::<EmploymentStats>()
            .init_resource::<TreeGrid>()
            .init_resource::<WindState>()
            .init_resource::<DeathCareGrid>()
            .init_resource::<DeathCareStats>()
            .init_resource::<production::CityGoods>()
            .init_resource::<MarketPrices>()
            .init_resource::<RoadConditionGrid>()
            .init_resource::<RoadMaintenanceBudget>()
            .init_resource::<RoadMaintenanceStats>()
            .init_resource::<AccidentTracker>()
            .init_resource::<CitySpecializations>()
            .init_resource::<SpecializationBonuses>()
            .init_resource::<OutsideConnections>()
            .init_resource::<AirportStats>()
            .init_resource::<AdvisorPanel>()
            .init_resource::<AchievementTracker>()
            .init_resource::<AchievementNotification>()
            .init_resource::<HeatingGrid>()
            .init_resource::<HeatingStats>()
            .init_resource::<GroundwaterGrid>()
            .init_resource::<WaterQualityGrid>()
            .init_resource::<GroundwaterStats>()
            .init_resource::<postal::PostalCoverage>()
            .init_resource::<postal::PostalStats>()
            .init_resource::<WaterSupply>()
            .add_event::<BankruptcyEvent>()
            .add_systems(Startup, init_world)
            .add_systems(
                FixedUpdate,
                (
                    tick_slow_timer,
                    time_of_day::tick_game_clock,
                    zones::update_zone_demand,
                    buildings::rebuild_eligible_cells,
                    buildings::building_spawner,
                    buildings::progress_construction,
                    education_jobs::assign_workplace_details,
                    citizen_spawner::spawn_citizens,
                    movement::refresh_destination_cache,
                    movement::citizen_state_machine,
                    // apply_deferred flushes PathRequest insertions from the state machine
                    bevy::ecs::schedule::apply_deferred,
                    movement::process_path_requests,
                    movement::move_citizens,
                    traffic::update_traffic_density,
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                (
                    happiness::update_service_coverage,
                    postal::update_postal_coverage,
                    happiness::update_happiness,
                )
                    .chain()
                    .after(traffic::update_traffic_density),
            )
            .add_systems(
                FixedUpdate,
                (
                    economy::collect_taxes,
                    stats::update_stats,
                    utilities::propagate_utilities,
                    education::propagate_education,
                )
                    .chain()
                    .after(happiness::update_happiness),
            )
            .add_systems(
                FixedUpdate,
                (
                    pollution::update_pollution,
                    land_value::update_land_value,
                    garbage::attach_waste_producers,
                    bevy::ecs::schedule::apply_deferred,
                    garbage::sync_recycling_policy,
                    garbage::update_garbage,
                    garbage::update_waste_generation,
                    garbage::update_waste_collection,
                    districts::aggregate_districts,
                    districts::district_stats,
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
                    road_maintenance::degrade_roads,
                    road_maintenance::repair_roads,
                    road_maintenance::update_road_maintenance_stats,
                )
                    .chain()
                    .after(traffic::update_traffic_density),
            )
            .add_systems(
                FixedUpdate,
                (
                    traffic_accidents::spawn_accidents,
                    traffic_accidents::process_accidents,
                )
                    .chain()
                    .after(traffic::update_traffic_density),
            )
            .add_systems(
                FixedUpdate,
                (loans::process_loan_payments, loans::update_credit_rating)
                    .chain()
                    .after(economy::collect_taxes),
            )
            .add_systems(
                FixedUpdate,
                (
                    weather::update_weather,
                    heating::update_heating,
                    wind::update_wind,
                    noise::update_noise_pollution,
                    crime::update_crime,
                    health::update_health_grid,
                    death_care::death_care_processing,
                    water_pollution::update_water_pollution,
                    water_pollution::water_pollution_health_penalty,
                    groundwater::update_groundwater,
                    groundwater::groundwater_health_penalty,
                    water_demand::calculate_building_water_demand,
                    water_demand::aggregate_water_supply,
                    water_demand::water_service_happiness_penalty,
                    natural_resources::update_resource_production,
                    wealth::update_wealth_stats,
                    tourism::update_tourism,
                    unlocks::award_development_points,
                    trees::tree_effects,
                )
                    .after(imports_exports::process_trade),
            )
            .add_systems(
                FixedUpdate,
                airport::update_airports.after(tourism::update_tourism),
            )
            .add_systems(
                FixedUpdate,
                outside_connections::update_outside_connections.after(airport::update_airports),
            )
            .add_systems(
                FixedUpdate,
                (
                    production::assign_industry_type,
                    production::update_production_chains,
                    market::update_market_prices,
                )
                    .chain()
                    .after(natural_resources::update_resource_production),
            )
            .add_systems(
                FixedUpdate,
                (events::random_city_events, events::apply_active_effects)
                    .chain()
                    .after(stats::update_stats),
            )
            .add_systems(
                FixedUpdate,
                specialization::compute_specializations.after(stats::update_stats),
            )
            .add_systems(
                FixedUpdate,
                advisors::update_advisors.after(stats::update_stats),
            )
            .add_systems(
                FixedUpdate,
                achievements::check_achievements
                    .after(stats::update_stats)
                    .after(specialization::compute_specializations),
            )
            .add_systems(
                FixedUpdate,
                (
                    abandonment::check_building_abandonment,
                    bevy::ecs::schedule::apply_deferred,
                    abandonment::process_abandoned_buildings,
                )
                    .chain()
                    .after(utilities::propagate_utilities),
            )
            .add_systems(
                FixedUpdate,
                abandonment::abandoned_land_value_penalty.after(land_value::update_land_value),
            )
            .add_systems(
                FixedUpdate,
                (
                    fire::start_random_fires,
                    fire::spread_fire,
                    fire::extinguish_fires,
                    fire::fire_damage,
                )
                    .chain()
                    .after(happiness::update_service_coverage),
            )
            .add_systems(
                FixedUpdate,
                forest_fire::update_forest_fire.after(fire::fire_damage),
            )
            .add_systems(
                FixedUpdate,
                (
                    disasters::trigger_random_disaster,
                    disasters::process_active_disaster,
                    bevy::ecs::schedule::apply_deferred,
                    disasters::apply_earthquake_damage,
                )
                    .chain()
                    .after(fire::fire_damage),
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
                education_jobs::job_matching.after(life_simulation::job_seeking),
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
                FixedUpdate,
                (
                    homelessness::check_homelessness,
                    bevy::ecs::schedule::apply_deferred,
                    homelessness::seek_shelter,
                    homelessness::recover_from_homelessness,
                )
                    .chain()
                    .after(happiness::update_happiness),
            )
            .add_systems(
                FixedUpdate,
                welfare::update_welfare.after(homelessness::recover_from_homelessness),
            )
            .add_systems(
                FixedUpdate,
                (
                    immigration::compute_attractiveness,
                    immigration::immigration_wave,
                )
                    .chain()
                    .after(stats::update_stats),
            )
            .add_systems(
                Update,
                (
                    time_of_day::sync_fixed_timestep,
                    rebuild_csr_on_road_change,
                    virtual_population::adjust_real_citizen_cap.run_if(
                        bevy::time::common_conditions::on_timer(std::time::Duration::from_secs(1)),
                    ),
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
fn rebuild_csr_on_road_change(roads: Res<RoadNetwork>, mut csr: ResMut<CsrGraph>) {
    if roads.is_changed() {
        *csr = CsrGraph::from_road_network(&roads);
    }
}

pub fn init_world(mut commands: Commands, mut segments: ResMut<RoadSegmentStore>) {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    // --- Tel Aviv terrain: Mediterranean coast on west, Yarkon River in north ---
    generate_tel_aviv_terrain(&mut grid);

    // Natural resources
    let mut resource_grid = ResourceGrid::default();
    let elevations: Vec<f32> = grid.cells.iter().map(|c| c.elevation).collect();
    natural_resources::generate_resources(&mut resource_grid, &elevations, 42);
    commands.insert_resource(resource_grid);

    let mut roads = RoadNetwork::default();

    // --- Road network using Bezier segments ---
    build_tel_aviv_roads(&mut segments, &mut grid, &mut roads);

    // --- Zoning ---
    apply_zones(&mut grid);

    // --- Buildings ---
    let building_entities = spawn_tel_aviv_buildings(&mut commands, &mut grid);

    // --- Utilities ---
    spawn_tel_aviv_utilities(&mut commands, &mut grid);

    // --- Services ---
    spawn_tel_aviv_services(&mut commands, &mut grid);

    // --- Citizens ---
    spawn_tel_aviv_citizens(&mut commands, &grid, &building_entities);

    // --- Groundwater ---
    let (gw_grid, wq_grid) = groundwater::init_groundwater(&grid);
    commands.insert_resource(gw_grid);
    commands.insert_resource(wq_grid);

    let budget = CityBudget {
        treasury: 100_000.0,
        ..CityBudget::default()
    };
    commands.insert_resource(budget);
    commands.insert_resource(grid);
    commands.insert_resource(roads);
}

// =============================================================================
// Tel Aviv terrain
// =============================================================================

fn generate_tel_aviv_terrain(grid: &mut WorldGrid) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            let xf = x as f32;
            let yf = y as f32;

            let coast = coastline_x(yf);

            // Yarkon River (east-west around y≈185, meandering)
            let yarkon_cy = 185.0 + 1.5 * (xf * 0.04).sin();
            let yarkon_hw = 2.0;
            let is_yarkon = (yf - yarkon_cy).abs() < yarkon_hw && xf > coast - 3.0 && xf < 195.0;

            let noise =
                ((x.wrapping_mul(7919).wrapping_add(y.wrapping_mul(6271))) % 100) as f32 / 100.0;

            let cell = grid.get_mut(x, y);

            if xf < coast || is_yarkon {
                cell.cell_type = CellType::Water;
                cell.elevation = 0.15 + noise * 0.1;
            } else {
                cell.cell_type = CellType::Grass;
                let dist_from_coast = (xf - coast).max(0.0);
                cell.elevation = 0.35 + (dist_from_coast * 0.002).min(0.3) + noise * 0.05;
            }
        }
    }
}

/// Coastline x-position as a function of y (north-south).
/// Models Tel Aviv's coast: Jaffa headland in the south, gentle curves northward.
fn coastline_x(y: f32) -> f32 {
    let base = 55.0;

    // Jaffa headland pushes west around y=40-60
    let jaffa = if y > 25.0 && y < 75.0 {
        let t = (y - 50.0) / 25.0;
        -7.0 * (1.0 - t * t).max(0.0)
    } else {
        0.0
    };

    // Gentle coastal waves
    let wave = 2.5 * (y * 0.03).sin() + 1.5 * (y * 0.08).cos();

    base + jaffa + wave
}

// =============================================================================
// Road helpers
// =============================================================================

/// Add a straight Bezier road between two grid positions.
#[allow(clippy::too_many_arguments)]
fn road_straight(
    seg: &mut RoadSegmentStore,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
    gx0: usize,
    gy0: usize,
    gx1: usize,
    gy1: usize,
    rt: RoadType,
) {
    let (wx0, wy0) = WorldGrid::grid_to_world(gx0, gy0);
    let (wx1, wy1) = WorldGrid::grid_to_world(gx1, gy1);
    seg.add_straight_segment(
        Vec2::new(wx0, wy0),
        Vec2::new(wx1, wy1),
        rt,
        16.0,
        grid,
        roads,
    );
}

/// Add a curved Bezier road with explicit control points (world coords).
#[allow(clippy::too_many_arguments)]
fn road_curve(
    seg: &mut RoadSegmentStore,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
    from: Vec2,
    c1: Vec2,
    c2: Vec2,
    to: Vec2,
    rt: RoadType,
) {
    let start = seg.find_or_create_node(from, 16.0);
    let end = seg.find_or_create_node(to, 16.0);
    seg.add_segment(start, end, from, c1, c2, to, rt, grid, roads);
}

/// Convert grid coords to world Vec2 (center of cell).
fn gw(gx: usize, gy: usize) -> Vec2 {
    let (wx, wy) = WorldGrid::grid_to_world(gx, gy);
    Vec2::new(wx, wy)
}

// =============================================================================
// Tel Aviv road network
// =============================================================================

fn build_tel_aviv_roads(seg: &mut RoadSegmentStore, grid: &mut WorldGrid, roads: &mut RoadNetwork) {
    // --- 1. Jaffa old city: winding local roads near the coast (SW) ---
    // Yefet Street: main road through Jaffa, slightly curving
    let jaffa_n = gw(62, 65);
    let jaffa_s = gw(55, 35);
    let jaffa_mid = gw(58, 50);
    road_curve(
        seg,
        grid,
        roads,
        jaffa_s,
        jaffa_s + Vec2::new(30.0, 80.0),
        jaffa_mid + Vec2::new(-10.0, 80.0),
        jaffa_n,
        RoadType::Local,
    );
    // Jaffa side streets
    road_straight(seg, grid, roads, 55, 42, 62, 42, RoadType::Local);
    road_straight(seg, grid, roads, 53, 50, 62, 50, RoadType::Local);
    road_straight(seg, grid, roads, 56, 58, 65, 58, RoadType::Local);

    // --- 2. Coastal Boulevard (Herbert Samuel → HaYarkon) ---
    // Runs north along the coast from Jaffa to the Yarkon river
    road_straight(seg, grid, roads, 63, 65, 63, 90, RoadType::Boulevard);
    road_straight(seg, grid, roads, 63, 90, 62, 120, RoadType::Boulevard);
    road_straight(seg, grid, roads, 62, 120, 62, 150, RoadType::Boulevard);
    road_straight(seg, grid, roads, 62, 150, 63, 180, RoadType::Boulevard);

    // --- 3. Allenby Street: coast to city center (NW to SE diagonal) ---
    let allenby_coast = gw(65, 82);
    let allenby_mid = gw(95, 88);
    let allenby_end = gw(140, 92);
    road_curve(
        seg,
        grid,
        roads,
        allenby_coast,
        allenby_coast + Vec2::new(200.0, 20.0),
        allenby_mid + Vec2::new(200.0, 30.0),
        allenby_end,
        RoadType::Avenue,
    );

    // --- 4. Rothschild Boulevard: the iconic tree-lined boulevard ---
    let roth_start = gw(78, 72);
    let roth_mid = gw(95, 88);
    let roth_end = gw(118, 108);
    road_curve(
        seg,
        grid,
        roads,
        roth_start,
        roth_start + Vec2::new(150.0, 100.0),
        roth_mid + Vec2::new(100.0, 100.0),
        roth_end,
        RoadType::Boulevard,
    );

    // --- 5. Dizengoff Street (N-S avenue through the White City) ---
    road_straight(seg, grid, roads, 102, 75, 102, 105, RoadType::Avenue);
    road_straight(seg, grid, roads, 102, 105, 102, 135, RoadType::Avenue);
    road_straight(seg, grid, roads, 102, 135, 102, 170, RoadType::Avenue);

    // --- 6. Ibn Gabirol Street (N-S avenue, east of Dizengoff) ---
    road_straight(seg, grid, roads, 125, 75, 125, 105, RoadType::Avenue);
    road_straight(seg, grid, roads, 125, 105, 125, 135, RoadType::Avenue);
    road_straight(seg, grid, roads, 125, 135, 125, 170, RoadType::Avenue);

    // --- 7. King George Street (E-W) ---
    road_straight(seg, grid, roads, 80, 120, 110, 120, RoadType::Avenue);
    road_straight(seg, grid, roads, 110, 120, 145, 120, RoadType::Avenue);

    // --- 8. Ben Gurion Boulevard (E-W, from coast to center) ---
    road_straight(seg, grid, roads, 63, 105, 95, 105, RoadType::Boulevard);
    road_straight(seg, grid, roads, 95, 105, 125, 105, RoadType::Boulevard);

    // --- 9. Arlozorov Street (E-W, major crosstown) ---
    road_straight(seg, grid, roads, 63, 155, 100, 155, RoadType::Avenue);
    road_straight(seg, grid, roads, 100, 155, 140, 155, RoadType::Avenue);
    road_straight(seg, grid, roads, 140, 155, 185, 155, RoadType::Avenue);

    // --- 10. Ayalon Highway (N-S expressway on the east) ---
    road_straight(seg, grid, roads, 185, 25, 185, 60, RoadType::Highway);
    road_straight(seg, grid, roads, 185, 60, 185, 100, RoadType::Highway);
    road_straight(seg, grid, roads, 185, 100, 185, 140, RoadType::Highway);
    road_straight(seg, grid, roads, 185, 140, 185, 180, RoadType::Highway);
    road_straight(seg, grid, roads, 185, 180, 185, 220, RoadType::Highway);

    // --- 11. Namir Road / Begin Road (N-S, center to north) ---
    road_straight(seg, grid, roads, 140, 108, 140, 140, RoadType::Boulevard);
    road_straight(seg, grid, roads, 140, 140, 140, 170, RoadType::Boulevard);

    // --- 12. Eilat Street (E-W through south) ---
    road_straight(seg, grid, roads, 62, 65, 90, 65, RoadType::Avenue);
    road_straight(seg, grid, roads, 90, 65, 130, 65, RoadType::Avenue);
    road_straight(seg, grid, roads, 130, 65, 185, 65, RoadType::Avenue);

    // --- 13. Highway on-ramps connecting Ayalon to city grid ---
    road_straight(seg, grid, roads, 145, 92, 185, 92, RoadType::Avenue);
    road_straight(seg, grid, roads, 145, 120, 185, 120, RoadType::Avenue);
    road_straight(seg, grid, roads, 145, 170, 185, 170, RoadType::Avenue);

    // --- 14. White City local grid streets (E-W, between the major avenues) ---
    // Between Eilat (y=65) and Arlozorov (y=155), every ~8 cells
    for &gy in &[75, 82, 92, 100, 112, 128, 140, 148] {
        road_straight(seg, grid, roads, 68, gy, 100, gy, RoadType::Local);
        road_straight(seg, grid, roads, 100, gy, 125, gy, RoadType::Local);
        road_straight(seg, grid, roads, 125, gy, 145, gy, RoadType::Local);
    }

    // --- 15. White City local grid streets (N-S, between the major avenues) ---
    for &gx in &[75, 82, 90, 110, 118, 132, 138] {
        road_straight(seg, grid, roads, gx, 68, gx, 95, RoadType::Local);
        road_straight(seg, grid, roads, gx, 95, gx, 120, RoadType::Local);
        road_straight(seg, grid, roads, gx, 120, gx, 150, RoadType::Local);
    }

    // --- 16. Ramat Aviv (north of Yarkon River, wider spacing) ---
    road_straight(seg, grid, roads, 75, 192, 75, 240, RoadType::Local);
    road_straight(seg, grid, roads, 100, 192, 100, 240, RoadType::Avenue);
    road_straight(seg, grid, roads, 125, 192, 125, 240, RoadType::Avenue);
    road_straight(seg, grid, roads, 150, 192, 150, 240, RoadType::Local);
    road_straight(seg, grid, roads, 75, 200, 150, 200, RoadType::Local);
    road_straight(seg, grid, roads, 75, 215, 150, 215, RoadType::Avenue);
    road_straight(seg, grid, roads, 75, 230, 150, 230, RoadType::Local);

    // Bridges over Yarkon River
    road_straight(seg, grid, roads, 100, 178, 100, 192, RoadType::Avenue);
    road_straight(seg, grid, roads, 125, 178, 125, 192, RoadType::Avenue);
    road_straight(seg, grid, roads, 140, 178, 140, 192, RoadType::Boulevard);

    // --- 17. Eastern areas (between city grid and Ayalon) ---
    for &gy in &[75, 92, 112, 135, 148] {
        road_straight(seg, grid, roads, 145, gy, 180, gy, RoadType::Local);
    }
    for &gx in &[155, 168] {
        road_straight(seg, grid, roads, gx, 68, gx, 100, RoadType::Local);
        road_straight(seg, grid, roads, gx, 100, gx, 150, RoadType::Local);
    }

    // --- 18. Waterfront promenade (path along the beach) ---
    for &(gy0, gy1) in &[(35, 65), (65, 90), (90, 120), (120, 150), (150, 180)] {
        let coast_x0 = (coastline_x(gy0 as f32) + 2.0) as usize;
        let coast_x1 = (coastline_x(gy1 as f32) + 2.0) as usize;
        road_straight(
            seg,
            grid,
            roads,
            coast_x0,
            gy0,
            coast_x1,
            gy1,
            RoadType::Path,
        );
    }
}

// =============================================================================
// Zoning (Tel Aviv neighborhoods)
// =============================================================================

#[allow(dead_code)]
fn zone_tel_aviv(grid: &WorldGrid, commands: &mut Commands) {
    // We need mutable grid but also read it for adjacency checks.
    // Clone zone assignments, then apply.
    let mut zone_map: Vec<(usize, usize, ZoneType)> = Vec::new();

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Grass || cell.zone != ZoneType::None {
                continue;
            }

            // Must be adjacent to a road
            let (n4, n4c) = grid.neighbors4(x, y);
            let has_road = n4[..n4c]
                .iter()
                .any(|&(nx, ny)| grid.get(nx, ny).cell_type == CellType::Road);
            if !has_road {
                continue;
            }

            let xf = x as f32;
            let yf = y as f32;
            let hash = x.wrapping_mul(31).wrapping_add(y.wrapping_mul(37));

            // Check if near coast
            let near_coast = xf < coastline_x(yf) + 12.0;

            let zone = if yf < 70.0 && xf < 80.0 {
                // Jaffa & Neve Tzedek: mixed old neighborhood
                match hash % 6 {
                    0..=2 => ZoneType::ResidentialLow,
                    3..=4 => ZoneType::CommercialLow,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if near_coast && yf < 160.0 {
                // Coastal strip: hotels, commercial, high-end residential
                match hash % 5 {
                    0..=1 => ZoneType::CommercialHigh,
                    2..=3 => ZoneType::ResidentialHigh,
                    _ => ZoneType::Office,
                }
            } else if xf > 70.0 && xf < 145.0 && yf > 70.0 && yf < 120.0 {
                // Central Tel Aviv / White City: dense residential + commercial
                match hash % 8 {
                    0..=3 => ZoneType::ResidentialHigh,
                    4..=5 => ZoneType::CommercialLow,
                    6 => ZoneType::Office,
                    _ => ZoneType::CommercialHigh,
                }
            } else if xf > 100.0 && xf < 150.0 && yf > 100.0 && yf < 115.0 {
                // Azrieli / Hashalom area: office towers
                match hash % 4 {
                    0..=1 => ZoneType::Office,
                    2 => ZoneType::CommercialHigh,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if xf > 145.0 && xf < 185.0 {
                // East of center, along Ayalon: industrial + commercial
                match hash % 8 {
                    0..=2 => ZoneType::Industrial,
                    3..=5 => ZoneType::CommercialLow,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if yf > 192.0 {
                // Ramat Aviv: residential suburbs + university area
                match hash % 6 {
                    0..=3 => ZoneType::ResidentialLow,
                    4 => ZoneType::ResidentialHigh,
                    _ => ZoneType::CommercialLow,
                }
            } else if xf > 70.0 && xf < 150.0 && yf > 120.0 && yf < 170.0 {
                // North-central: residential with some commercial
                match hash % 8 {
                    0..=4 => ZoneType::ResidentialHigh,
                    5..=6 => ZoneType::CommercialLow,
                    _ => ZoneType::Office,
                }
            } else {
                // Fallback: residential
                if hash % 3 == 0 {
                    ZoneType::ResidentialLow
                } else {
                    ZoneType::ResidentialHigh
                }
            };

            zone_map.push((x, y, zone));
        }
    }

    // Apply (need to drop immutable borrow first — we use commands for deferred grid mutation)
    // Actually we can't mutate grid here since we took it as &WorldGrid.
    // We'll apply zones after this function returns. Store them and apply in init_world.
    // For now, let's use a different approach: store zone_map as a resource and apply later.
    // Actually, simpler: just pass &mut WorldGrid. Let me fix the signature.
    let _ = commands;
    let _ = zone_map;
}

#[allow(dead_code)]
fn apply_zones(grid: &mut WorldGrid) {
    // Precompute which cells are near roads (within manhattan distance 5)
    let zone_depth: isize = 5;
    let mut near_road = vec![false; GRID_WIDTH * GRID_HEIGHT];
    for ry in 0..GRID_HEIGHT {
        for rx in 0..GRID_WIDTH {
            if grid.get(rx, ry).cell_type != CellType::Road {
                continue;
            }
            for dy in -zone_depth..=zone_depth {
                for dx in -zone_depth..=zone_depth {
                    if dx.abs() + dy.abs() > zone_depth {
                        continue;
                    }
                    let nx = rx as isize + dx;
                    let ny = ry as isize + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                    {
                        near_road[ny as usize * GRID_WIDTH + nx as usize] = true;
                    }
                }
            }
        }
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell_type = grid.get(x, y).cell_type;
            let current_zone = grid.get(x, y).zone;
            if cell_type != CellType::Grass || current_zone != ZoneType::None {
                continue;
            }

            // Must be within zone_depth cells of a road
            if !near_road[y * GRID_WIDTH + x] {
                continue;
            }

            let xf = x as f32;
            let yf = y as f32;
            let hash = x.wrapping_mul(31).wrapping_add(y.wrapping_mul(37));
            let near_coast = xf < coastline_x(yf) + 12.0;

            let zone = if yf < 70.0 && xf < 80.0 {
                match hash % 6 {
                    0..=2 => ZoneType::ResidentialLow,
                    3..=4 => ZoneType::CommercialLow,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if near_coast && yf < 160.0 {
                match hash % 5 {
                    0..=1 => ZoneType::CommercialHigh,
                    2..=3 => ZoneType::ResidentialHigh,
                    _ => ZoneType::Office,
                }
            } else if xf > 70.0 && xf < 145.0 && yf > 70.0 && yf < 120.0 {
                match hash % 8 {
                    0..=3 => ZoneType::ResidentialHigh,
                    4..=5 => ZoneType::CommercialLow,
                    6 => ZoneType::Office,
                    _ => ZoneType::CommercialHigh,
                }
            } else if xf > 100.0 && xf < 150.0 && yf > 100.0 && yf < 115.0 {
                match hash % 4 {
                    0..=1 => ZoneType::Office,
                    2 => ZoneType::CommercialHigh,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if xf > 145.0 && xf < 185.0 {
                match hash % 8 {
                    0..=2 => ZoneType::Industrial,
                    3..=5 => ZoneType::CommercialLow,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if yf > 192.0 {
                match hash % 6 {
                    0..=3 => ZoneType::ResidentialLow,
                    4 => ZoneType::ResidentialHigh,
                    _ => ZoneType::CommercialLow,
                }
            } else if xf > 70.0 && xf < 150.0 && yf > 120.0 && yf < 170.0 {
                match hash % 8 {
                    0..=4 => ZoneType::ResidentialHigh,
                    5..=6 => ZoneType::CommercialLow,
                    _ => ZoneType::Office,
                }
            } else if hash % 3 == 0 {
                ZoneType::ResidentialLow
            } else {
                ZoneType::ResidentialHigh
            };

            grid.get_mut(x, y).zone = zone;
        }
    }
}

// =============================================================================
// Buildings
// =============================================================================

fn spawn_tel_aviv_buildings(
    commands: &mut Commands,
    grid: &mut WorldGrid,
) -> Vec<(Entity, ZoneType, usize, usize, u32)> {
    let mut building_entities: Vec<(Entity, ZoneType, usize, usize, u32)> = Vec::new();

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let zone = grid.get(x, y).zone;
            let cell_type = grid.get(x, y).cell_type;
            if zone == ZoneType::None || cell_type != CellType::Grass {
                continue;
            }
            if grid.get(x, y).building_id.is_some() {
                continue;
            }

            // Building setback: skip cells directly adjacent to road cells
            let (n4, n4c) = grid.neighbors4(x, y);
            let adjacent_to_road = n4[..n4c]
                .iter()
                .any(|&(nx, ny)| grid.get(nx, ny).cell_type == CellType::Road);
            if adjacent_to_road {
                continue;
            }

            let hash = x.wrapping_mul(7).wrapping_add(y.wrapping_mul(13));
            let fill_pct = match zone {
                ZoneType::CommercialHigh | ZoneType::Office => 90,
                ZoneType::CommercialLow => 85,
                ZoneType::ResidentialHigh => 82,
                ZoneType::ResidentialMedium => 80,
                ZoneType::Industrial => 78,
                ZoneType::ResidentialLow => 70,
                _ => 65,
            };
            if hash % 100 > fill_pct {
                continue;
            }

            let xf = x as f32;
            let yf = y as f32;
            // Building level based on neighborhood
            let level: u8 = if xf > 100.0 && xf < 150.0 && yf > 90.0 && yf < 115.0 {
                // Azrieli area: tall
                if hash % 3 == 0 {
                    2
                } else {
                    3
                }
            } else if xf > 70.0 && xf < 140.0 && yf > 70.0 && yf < 160.0 {
                // White City: medium-tall
                match hash % 4 {
                    0 => 1,
                    1..=2 => 2,
                    _ => 3,
                }
            } else if yf < 70.0 && xf < 80.0 {
                // Jaffa: low
                if hash % 4 == 0 {
                    2
                } else {
                    1
                }
            } else if yf > 192.0 {
                // Ramat Aviv: medium
                match hash % 3 {
                    0 => 1,
                    1 => 2,
                    _ => 1,
                }
            } else {
                match hash % 3 {
                    0 => 1,
                    1 => 2,
                    _ => 1,
                }
            };

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

    building_entities
}

// =============================================================================
// Utilities
// =============================================================================

fn spawn_tel_aviv_utilities(commands: &mut Commands, grid: &mut WorldGrid) {
    let positions = [
        (UtilityType::PowerPlant, 200usize, 50usize),
        (UtilityType::PowerPlant, 200, 150),
        (UtilityType::PowerPlant, 200, 220),
        (UtilityType::PowerPlant, 120, 30),
        (UtilityType::WaterTower, 90, 90),
        (UtilityType::WaterTower, 130, 130),
        (UtilityType::WaterTower, 80, 160),
        (UtilityType::WaterTower, 110, 210),
        (UtilityType::WaterTower, 160, 80),
        (UtilityType::WaterTower, 160, 160),
    ];

    for &(utype, ux, uy) in &positions {
        if let Some((px, py)) = find_free_grass_cell(grid, ux, uy, 10) {
            let range = match utype {
                UtilityType::PowerPlant => 120,
                UtilityType::WaterTower => 90,
                _ => 50,
            };
            let entity = commands
                .spawn(UtilitySource {
                    utility_type: utype,
                    grid_x: px,
                    grid_y: py,
                    range,
                })
                .id();
            grid.get_mut(px, py).building_id = Some(entity);
        }
    }
}

// =============================================================================
// Services
// =============================================================================

fn spawn_tel_aviv_services(commands: &mut Commands, grid: &mut WorldGrid) {
    let positions = [
        // Fire stations
        (ServiceType::FireStation, 85usize, 55usize),
        (ServiceType::FireStation, 130, 100),
        (ServiceType::FireStation, 80, 145),
        (ServiceType::FireStation, 120, 210),
        // Police
        (ServiceType::PoliceStation, 65, 48), // Jaffa
        (ServiceType::PoliceStation, 110, 90),
        (ServiceType::PoliceStation, 90, 135),
        (ServiceType::PoliceStation, 130, 160),
        // Hospitals
        (ServiceType::Hospital, 95, 80), // Ichilov area
        (ServiceType::Hospital, 150, 130),
        // Schools
        (ServiceType::ElementarySchool, 78, 80),
        (ServiceType::ElementarySchool, 115, 130),
        (ServiceType::ElementarySchool, 88, 210),
        (ServiceType::HighSchool, 105, 95),
        (ServiceType::HighSchool, 90, 150),
        (ServiceType::University, 110, 215), // Tel Aviv University area
        // Parks
        (ServiceType::LargePark, 80, 180), // Yarkon Park
        (ServiceType::LargePark, 105, 180),
        (ServiceType::SmallPark, 95, 88), // Rothschild gardens
        (ServiceType::SmallPark, 110, 105),
        (ServiceType::SmallPark, 130, 140),
        (ServiceType::SmallPark, 70, 50), // Jaffa garden
        (ServiceType::Plaza, 100, 135),   // Dizengoff Square area
        (ServiceType::Plaza, 118, 108),   // Habima area
        // Culture & civic
        (ServiceType::Museum, 112, 100), // Art museum area
        (ServiceType::CityHall, 115, 95),
        (ServiceType::Library, 105, 110),
        // Transport
        (ServiceType::TrainStation, 145, 95), // HaShalom station
        (ServiceType::TrainStation, 145, 155), // Arlozorov station
        (ServiceType::BusDepot, 100, 105),
        (ServiceType::SubwayStation, 110, 85),
        (ServiceType::SubwayStation, 115, 120),
    ];

    for &(stype, sx, sy) in &positions {
        if let Some((px, py)) = find_free_grass_cell(grid, sx, sy, 10) {
            let entity = commands
                .spawn(ServiceBuilding {
                    service_type: stype,
                    grid_x: px,
                    grid_y: py,
                    radius: ServiceBuilding::coverage_radius(stype),
                })
                .id();
            grid.get_mut(px, py).building_id = Some(entity);
        }
    }
}

// =============================================================================
// Citizens
// =============================================================================

fn spawn_tel_aviv_citizens(
    commands: &mut Commands,
    _grid: &WorldGrid,
    building_entities: &[(Entity, ZoneType, usize, usize, u32)],
) {
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

    if work_buildings.is_empty() {
        return;
    }

    let work_caps: Vec<u32> = building_entities
        .iter()
        .filter(|(_, zt, _, _, _)| zt.is_job_zone())
        .map(|(_, _, _, _, cap)| *cap)
        .collect();

    let mut work_idx = 0usize;
    let mut work_occupancy: Vec<u32> = vec![0; work_buildings.len()];
    let mut citizen_count = 0u32;
    let target_pop = 10_000u32;
    let mut age_counter = 0u8;

    for (home_entity, hx, hy, cap) in &residential_buildings {
        if citizen_count >= target_pop {
            break;
        }
        let fill = (*cap as f32 * 0.9).ceil() as u32;
        for _ in 0..fill {
            if citizen_count >= target_pop {
                break;
            }

            let start_idx = work_idx;
            loop {
                if work_occupancy[work_idx] < work_caps[work_idx] {
                    break;
                }
                work_idx = (work_idx + 1) % work_buildings.len();
                if work_idx == start_idx {
                    break;
                }
            }

            let (work_entity, wx, wy) = work_buildings[work_idx];
            work_occupancy[work_idx] += 1;
            work_idx = (work_idx + 1) % work_buildings.len();

            let (home_wx, home_wy) = WorldGrid::grid_to_world(*hx, *hy);
            age_counter = age_counter.wrapping_add(7);
            let age = 18 + (age_counter % 47);

            let gender = if citizen_count.is_multiple_of(2) {
                Gender::Male
            } else {
                Gender::Female
            };
            let edu = match age {
                18..=22 => (age_counter % 3).min(1),
                23..=30 => (age_counter % 4).min(2),
                _ => (age_counter % 5).min(3),
            };
            let salary = CitizenDetails::base_salary_for_education(edu)
                * (1.0 + age.saturating_sub(18) as f32 * 0.01);

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
    }
}

// =============================================================================
// Helpers
// =============================================================================

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
                if r > 0 {
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
