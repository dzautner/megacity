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
pub mod composting;
pub mod config;
pub mod crime;
pub mod death_care;
pub mod degree_days;
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
pub mod stormwater;
pub mod terrain;
pub mod time_of_day;
pub mod tourism;
pub mod traffic;
pub mod traffic_accidents;
pub mod trees;
pub mod unlocks;
pub mod utilities;
pub mod virtual_population;
pub mod waste_composition;
pub mod waste_effects;
pub mod water_demand;
pub mod water_pollution;
pub mod water_sources;
pub mod wealth;
pub mod weather;
pub mod welfare;
pub mod wind;
pub mod world_init;
pub mod zones;

use achievements::{AchievementNotification, AchievementTracker};
use advisors::AdvisorPanel;
use airport::AirportStats;
use budget::ExtendedBudget;
use building_upgrade::UpgradeTimer;
use buildings::{BuildingSpawnTimer, EligibleCells};
use citizen_spawner::CitizenSpawnTimer;
use composting::CompostingState;
use crime::CrimeGrid;
use death_care::{DeathCareGrid, DeathCareStats};
use degree_days::DegreeDays;
use disasters::ActiveDisaster;
use districts::{DistrictMap, Districts};
use economy::CityBudget;
use education::EducationGrid;
use education_jobs::EmploymentStats;
use events::{ActiveCityEffects, EventJournal, MilestoneTracker};
use fire::FireGrid;
use forest_fire::{ForestFireGrid, ForestFireStats};
use garbage::{GarbageGrid, WasteCollectionGrid, WasteSystem};
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
use natural_resources::{ResourceBalance, ResourceGrid};
use noise::NoisePollutionGrid;
use outside_connections::OutsideConnections;
use policies::Policies;
use pollution::PollutionGrid;
use road_graph_csr::CsrGraph;
use road_maintenance::{RoadConditionGrid, RoadMaintenanceBudget, RoadMaintenanceStats};
use road_segments::RoadSegmentStore;
use roads::RoadNetwork;
use spatial_grid::SpatialGrid;
use specialization::{CitySpecializations, SpecializationBonuses};
use stats::CityStats;
use stormwater::StormwaterGrid;
use time_of_day::GameClock;
use tourism::Tourism;
use traffic::TrafficGrid;
use traffic_accidents::AccidentTracker;
use trees::TreeGrid;
use unlocks::UnlockState;
use virtual_population::VirtualPopulation;
use waste_effects::{WasteAccumulation, WasteCrisisEvent};
use water_demand::WaterSupply;
use water_pollution::WaterPollutionGrid;
use wealth::WealthStats;
use weather::{ClimateZone, ConstructionModifiers, Weather, WeatherChangeEvent};
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
            .init_resource::<ClimateZone>()
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
            .init_resource::<StormwaterGrid>()
            .init_resource::<DegreeDays>()
            .init_resource::<ConstructionModifiers>()
            .init_resource::<WasteAccumulation>()
            .init_resource::<CompostingState>()
            .add_event::<BankruptcyEvent>()
            .add_event::<WeatherChangeEvent>()
            .add_event::<WasteCrisisEvent>()
            .add_systems(Startup, world_init::init_world)
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
                    waste_effects::update_waste_accumulation,
                    waste_effects::waste_health_penalty,
                    waste_effects::check_waste_crisis,
                )
                    .chain()
                    .after(garbage::update_waste_collection),
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
                    degree_days::update_degree_days,
                    weather::update_construction_modifiers,
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
                    stormwater::update_stormwater,
                    water_demand::calculate_building_water_demand,
                    water_demand::aggregate_water_supply,
                )
                    .after(imports_exports::process_trade),
            )
            .add_systems(
                FixedUpdate,
                (
                    water_sources::update_water_sources,
                    water_sources::aggregate_water_source_supply,
                    water_sources::replenish_reservoirs,
                    water_demand::water_service_happiness_penalty,
                    natural_resources::update_resource_production,
                    wealth::update_wealth_stats,
                    tourism::update_tourism,
                    unlocks::award_development_points,
                )
                    .after(imports_exports::process_trade),
            )
            .add_systems(
                FixedUpdate,
                trees::tree_effects.after(imports_exports::process_trade),
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
            .add_systems(Update, tick_lod_frame_counter)
            .add_systems(
                Update,
                composting::update_composting.run_if(bevy::time::common_conditions::on_timer(
                    std::time::Duration::from_secs(2),
                )),
            );
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
