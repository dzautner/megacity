use bevy::prelude::*;

pub mod abandonment;
pub mod achievements;
pub mod advisors;
pub mod agriculture;
pub mod airport;
pub mod budget;
pub mod building_upgrade;
pub mod buildings;
pub mod citizen;
pub mod citizen_spawner;
pub mod cold_snap;
pub mod composting;
pub mod config;
pub mod crime;
pub mod cso;
pub mod death_care;
pub mod degree_days;
pub mod disasters;
pub mod districts;
pub mod drought;
pub mod economy;
pub mod education;
pub mod education_jobs;
pub mod events;
pub mod fire;
pub mod flood_simulation;
pub mod fog;
pub mod forest_fire;
pub mod garbage;
pub mod grid;
pub mod groundwater;
pub mod groundwater_depletion;
pub mod happiness;
pub mod hazardous_waste;
pub mod health;
pub mod heat_wave;
pub mod heating;
pub mod homelessness;
pub mod immigration;
pub mod imports_exports;
pub mod land_value;
pub mod landfill_gas;
pub mod landfill_warning;
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
pub mod recycling;
pub mod reservoir;
pub mod road_graph_csr;
pub mod road_maintenance;
pub mod road_segments;
pub mod roads;
pub mod services;
pub mod snow;
pub mod spatial_grid;
pub mod specialization;
pub mod stats;
pub mod storm_drainage;
pub mod stormwater;
pub mod terrain;
pub mod time_of_day;
pub mod tourism;
pub mod traffic;
pub mod traffic_accidents;
pub mod trees;
pub mod unlocks;
pub mod urban_growth_boundary;
pub mod urban_heat_island;
pub mod utilities;
pub mod virtual_population;
pub mod waste_composition;
pub mod waste_effects;
pub mod wastewater;
pub mod water_conservation;
pub mod water_demand;
pub mod water_pollution;
pub mod water_sources;
pub mod water_treatment;
pub mod wealth;
pub mod weather;
pub mod welfare;
pub mod wind;
pub mod wind_damage;
pub mod world_init;
pub mod zones;

use road_graph_csr::CsrGraph;
use road_segments::RoadSegmentStore;
use roads::RoadNetwork;
use spatial_grid::SpatialGrid;

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
        // Core resources and systems that don't belong to any feature
        app.init_resource::<TickCounter>()
            .init_resource::<SlowTickTimer>()
            .init_resource::<CsrGraph>()
            .init_resource::<RoadSegmentStore>()
            .init_resource::<SpatialGrid>()
            .init_resource::<policies::Policies>()
            .init_resource::<budget::ExtendedBudget>()
            .init_resource::<LodFrameCounter>()
            .add_systems(Startup, world_init::init_world)
            .add_systems(FixedUpdate, tick_slow_timer)
            .add_systems(Update, rebuild_csr_on_road_change)
            .add_systems(Update, tick_lod_frame_counter);

        // Core simulation chain
        app.add_plugins((
            time_of_day::TimeOfDayPlugin,
            zones::ZonesPlugin,
            buildings::BuildingsPlugin,
            education_jobs::EducationJobsPlugin,
            citizen_spawner::CitizenSpawnerPlugin,
            movement::MovementPlugin,
            traffic::TrafficPlugin,
        ));

        // Happiness and services
        app.add_plugins((
            postal::PostalPlugin,
            happiness::HappinessPlugin,
            economy::EconomyPlugin,
            stats::StatsPlugin,
            utilities::UtilitiesPlugin,
            education::EducationPlugin,
        ));

        // Pollution, land value, garbage, districts
        app.add_plugins((
            pollution::PollutionPlugin,
            land_value::LandValuePlugin,
            garbage::GarbagePlugin,
            districts::DistrictsPlugin,
            lifecycle::LifecyclePlugin,
            building_upgrade::BuildingUpgradePlugin,
            imports_exports::ImportsExportsPlugin,
        ));

        // Waste and recycling
        app.add_plugins((
            waste_effects::WasteEffectsPlugin,
            recycling::RecyclingPlugin,
            road_maintenance::RoadMaintenancePlugin,
            traffic_accidents::TrafficAccidentsPlugin,
            loans::LoansPlugin,
        ));

        // Weather and environment
        app.add_plugins((
            weather::WeatherPlugin,
            fog::FogPlugin,
            degree_days::DegreeDaysPlugin,
            heating::HeatingPlugin,
            wind::WindPlugin,
            wind_damage::WindDamagePlugin,
            urban_heat_island::UrbanHeatIslandPlugin,
            drought::DroughtPlugin,
            noise::NoisePlugin,
            crime::CrimePlugin,
            health::HealthPlugin,
            death_care::DeathCarePlugin,
        ));

        // Water systems
        app.add_plugins((
            water_pollution::WaterPollutionPlugin,
            groundwater::GroundwaterPlugin,
            stormwater::StormwaterPlugin,
            water_demand::WaterDemandPlugin,
            heat_wave::HeatWavePlugin,
            composting::CompostingPlugin,
            cold_snap::ColdSnapPlugin,
            cso::CsoPlugin,
            water_treatment::WaterTreatmentPlugin,
            water_conservation::WaterConservationPlugin,
            groundwater_depletion::GroundwaterDepletionPlugin,
            wastewater::WastewaterPlugin,
        ));

        // Waste management
        app.add_plugins((
            hazardous_waste::HazardousWastePlugin,
            landfill_gas::LandfillGasPlugin,
            landfill_warning::LandfillWarningPlugin,
        ));

        // Infrastructure and resources
        app.add_plugins((
            storm_drainage::StormDrainagePlugin,
            water_sources::WaterSourcesPlugin,
            natural_resources::NaturalResourcesPlugin,
            wealth::WealthPlugin,
            tourism::TourismPlugin,
            unlocks::UnlocksPlugin,
            reservoir::ReservoirPlugin,
            flood_simulation::FloodSimulationPlugin,
            trees::TreesPlugin,
            airport::AirportPlugin,
            outside_connections::OutsideConnectionsPlugin,
            snow::SnowPlugin,
        ));

        // Production and economy
        app.add_plugins((
            agriculture::AgriculturePlugin,
            production::ProductionPlugin,
            market::MarketPlugin,
            events::EventsPlugin,
            specialization::SpecializationPlugin,
            advisors::AdvisorsPlugin,
            achievements::AchievementsPlugin,
        ));

        // Building lifecycle and disasters
        app.add_plugins((
            abandonment::AbandonmentPlugin,
            fire::FirePlugin,
            forest_fire::ForestFirePlugin,
            disasters::DisastersPlugin,
        ));

        // Citizens and population
        app.add_plugins((
            life_simulation::LifeSimulationPlugin,
            homelessness::HomelessnessPlugin,
            welfare::WelfarePlugin,
            immigration::ImmigrationPlugin,
            lod::LodPlugin,
            virtual_population::VirtualPopulationPlugin,
            urban_growth_boundary::UrbanGrowthBoundaryPlugin,
        ));
    }
}

pub fn tick_slow_timer(mut timer: ResMut<SlowTickTimer>, mut tick: ResMut<TickCounter>) {
    timer.tick();
    tick.0 = tick.0.wrapping_add(1);
}

/// Counter for throttling LOD/spatial grid updates to every 6th render frame (~10Hz at 60fps).
#[derive(Resource, Default)]
struct LodFrameCounter(u32);

fn tick_lod_frame_counter(mut counter: ResMut<LodFrameCounter>) {
    counter.0 = counter.0.wrapping_add(1);
}

pub fn lod_frame_ready(counter: Res<LodFrameCounter>) -> bool {
    counter.0.is_multiple_of(6)
}

/// Rebuild the CSR graph whenever the road network changes.
fn rebuild_csr_on_road_change(roads: Res<RoadNetwork>, mut csr: ResMut<CsrGraph>) {
    if roads.is_changed() {
        *csr = CsrGraph::from_road_network(&roads);
    }
}
