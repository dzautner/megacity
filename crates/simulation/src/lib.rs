use bevy::prelude::*;
use std::collections::BTreeMap;

pub mod abandonment;
pub mod achievements;
pub mod advisors;
pub mod agriculture;
pub mod airport;
pub mod budget;
pub mod building_upgrade;
pub mod buildings;
pub mod chart_data;
pub mod citizen;
pub mod citizen_spawner;
pub mod climate_change;
pub mod cold_snap;
pub mod colorblind;
pub mod composting;
pub mod config;
pub mod crime;
pub mod cso;
pub mod cumulative_zoning;
pub mod day_night_controls;
pub mod death_care;
pub mod degree_days;
pub mod disasters;
pub mod district_policies;
pub mod districts;
pub mod drought;
pub mod economy;
pub mod education;
pub mod education_jobs;
pub mod events;
pub mod far_transfer;
pub mod fire;
pub mod flood_protection;
pub mod flood_simulation;
pub mod fog;
pub mod forest_fire;
pub mod form_transect;
pub mod garbage;
pub mod grid;
pub mod groundwater;
pub mod groundwater_depletion;
pub mod happiness;
pub mod hazardous_waste;
pub mod health;
pub mod heat_mitigation;
pub mod heat_wave;
pub mod heating;
pub mod historic_preservation;
pub mod homelessness;
pub mod immigration;
pub mod imports_exports;
pub mod inclusionary_zoning;
pub mod land_value;
pub mod landfill;
pub mod landfill_gas;
pub mod landfill_warning;
pub mod life_simulation;
pub mod lifecycle;
pub mod loans;
pub mod localization;
pub mod lod;
pub mod market;
pub mod movement;
pub mod multi_select;
pub mod natural_resources;
pub mod neighborhood_quality;
pub mod network_viz;
pub mod nimby;
pub mod noise;
pub mod oneway;
pub mod outside_connections;
pub mod parking;
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
pub mod seasonal_rendering;
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
pub mod traffic_los;
pub mod trees;
pub mod tutorial;
pub mod uhi_mitigation;
pub mod unlocks;
pub mod urban_growth_boundary;
pub mod urban_heat_island;
pub mod utilities;
pub mod virtual_population;
pub mod walkability;
pub mod waste_composition;
pub mod waste_effects;
pub mod waste_policies;
pub mod wastewater;
pub mod water_conservation;
pub mod water_demand;
pub mod water_pollution;
pub mod water_pressure;
pub mod water_sources;
pub mod water_treatment;
pub mod wealth;
pub mod weather;
pub mod welfare;
pub mod wind;
pub mod wind_damage;
pub mod world_init;
pub mod zones;

#[cfg(test)]
mod integration_tests;
#[cfg(any(test, feature = "bench"))]
pub mod test_harness;

use road_graph_csr::CsrGraph;
use road_segments::RoadSegmentStore;
use spatial_grid::SpatialGrid;

// ---------------------------------------------------------------------------
// Saveable trait + registry for the extension map save pattern
// ---------------------------------------------------------------------------

/// Trait for resources that can be saved/loaded via the extension map.
///
/// Each implementing resource provides its own serialization logic, so adding a new
/// saveable feature requires ZERO changes to any save system file -- the feature
/// plugin just calls `app.register_saveable::<T>()` in its `build()`.
pub trait Saveable: Resource + Default + Send + Sync + 'static {
    /// Unique key for this resource in the save file's extension map.
    /// Must be stable across versions (used for deserialization lookup).
    const SAVE_KEY: &'static str;

    /// Serialize this resource to bytes.
    /// Return `None` to skip saving (e.g. when the resource is at its default state).
    fn save_to_bytes(&self) -> Option<Vec<u8>>;

    /// Deserialize from bytes, returning the restored resource.
    fn load_from_bytes(bytes: &[u8]) -> Self;
}

/// Decode bytes via `bitcode::decode`, logging a warning and returning `Default` on failure.
/// Use this in `Saveable::load_from_bytes` implementations to surface decode errors.
pub fn decode_or_warn<T: bitcode::DecodeOwned + Default>(key: &str, bytes: &[u8]) -> T {
    match bitcode::decode(bytes) {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "Saveable {}: failed to decode {} bytes, falling back to default: {}",
                key,
                bytes.len(),
                e
            );
            T::default()
        }
    }
}

/// Type alias for the save function stored in a `SaveableEntry`.
pub type SaveFn = Box<dyn Fn(&World) -> Option<Vec<u8>> + Send + Sync>;
/// Type alias for the load function stored in a `SaveableEntry`.
pub type LoadFn = Box<dyn Fn(&mut World, &[u8]) + Send + Sync>;
/// Type alias for the reset function stored in a `SaveableEntry`.
pub type ResetFn = Box<dyn Fn(&mut World) + Send + Sync>;

/// Type-erased save/load/reset operations for a single registered resource.
pub struct SaveableEntry {
    pub key: String,
    pub save_fn: SaveFn,
    pub load_fn: LoadFn,
    pub reset_fn: ResetFn,
}

/// Registry of all saveable resources, populated during plugin setup.
///
/// The save system iterates this registry to persist/restore extension map entries
/// without needing to know about individual feature types.
#[derive(Resource, Default)]
pub struct SaveableRegistry {
    pub entries: Vec<SaveableEntry>,
}

impl SaveableRegistry {
    /// Register a resource type that implements `Saveable`.
    ///
    /// Panics in debug builds if a resource with the same `SAVE_KEY` is already
    /// registered, preventing silent data loss from duplicate registrations.
    pub fn register<T: Saveable>(&mut self) {
        let key = T::SAVE_KEY.to_string();
        if self.entries.iter().any(|e| e.key == key) {
            warn!(
                "SaveableRegistry: duplicate key '{}' — ignoring second registration",
                key
            );
            debug_assert!(false, "SaveableRegistry: duplicate key '{}'", key);
            return;
        }
        self.entries.push(SaveableEntry {
            key,
            save_fn: Box::new(|world: &World| {
                world.get_resource::<T>().and_then(|r| r.save_to_bytes())
            }),
            load_fn: Box::new(|world: &mut World, bytes: &[u8]| {
                let value = T::load_from_bytes(bytes);
                world.insert_resource(value);
            }),
            reset_fn: Box::new(|world: &mut World| {
                world.insert_resource(T::default());
            }),
        });
    }

    /// Save all registered resources into an extension map.
    pub fn save_all(&self, world: &World) -> BTreeMap<String, Vec<u8>> {
        let mut extensions = BTreeMap::new();
        for entry in &self.entries {
            if let Some(bytes) = (entry.save_fn)(world) {
                extensions.insert(entry.key.clone(), bytes);
            }
        }
        extensions
    }

    /// Load registered resources from an extension map.
    /// Resources whose key is absent are left unchanged (they keep their init_resource default).
    pub fn load_all(&self, world: &mut World, extensions: &BTreeMap<String, Vec<u8>>) {
        for entry in &self.entries {
            if let Some(bytes) = extensions.get(&entry.key) {
                (entry.load_fn)(world, bytes);
            }
        }
    }

    /// Reset all registered resources to their defaults (used by new-game).
    pub fn reset_all(&self, world: &mut World) {
        for entry in &self.entries {
            (entry.reset_fn)(world);
        }
    }
}

// ---------------------------------------------------------------------------
// Core resources
// ---------------------------------------------------------------------------

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
            chart_data::ChartDataPlugin,
            utilities::UtilitiesPlugin,
            network_viz::NetworkVizPlugin,
            education::EducationPlugin,
        ));

        // Pollution, land value, garbage, districts
        app.add_plugins((
            pollution::PollutionPlugin,
            land_value::LandValuePlugin,
            garbage::GarbagePlugin,
            districts::DistrictsPlugin,
            district_policies::DistrictPoliciesPlugin,
            neighborhood_quality::NeighborhoodQualityPlugin,
            lifecycle::LifecyclePlugin,
            building_upgrade::BuildingUpgradePlugin,
            imports_exports::ImportsExportsPlugin,
            historic_preservation::HistoricPreservationPlugin,
            inclusionary_zoning::InclusionaryZoningPlugin,
            far_transfer::FarTransferPlugin,
        ));

        // Waste and recycling
        app.add_plugins((
            waste_effects::WasteEffectsPlugin,
            recycling::RecyclingPlugin,
            road_maintenance::RoadMaintenancePlugin,
            oneway::OneWayPlugin,
            traffic_accidents::TrafficAccidentsPlugin,
            traffic_los::TrafficLosPlugin,
            loans::LoansPlugin,
        ));

        // Day/night visual controls
        app.add_plugins(day_night_controls::DayNightControlsPlugin);

        // Weather and environment
        app.add_plugins((
            weather::WeatherPlugin,
            fog::FogPlugin,
            degree_days::DegreeDaysPlugin,
            heating::HeatingPlugin,
            wind::WindPlugin,
            wind_damage::WindDamagePlugin,
            urban_heat_island::UrbanHeatIslandPlugin,
            uhi_mitigation::UhiMitigationPlugin,
            drought::DroughtPlugin,
            noise::NoisePlugin,
            crime::CrimePlugin,
            health::HealthPlugin,
            death_care::DeathCarePlugin,
            climate_change::ClimateChangePlugin,
            seasonal_rendering::SeasonalRenderingPlugin,
        ));

        // Water systems
        app.add_plugins((
            water_pollution::WaterPollutionPlugin,
            groundwater::GroundwaterPlugin,
            stormwater::StormwaterPlugin,
            water_demand::WaterDemandPlugin,
            heat_wave::HeatWavePlugin,
            heat_mitigation::HeatMitigationPlugin,
            composting::CompostingPlugin,
            cold_snap::ColdSnapPlugin,
            cso::CsoPlugin,
            water_treatment::WaterTreatmentPlugin,
            water_conservation::WaterConservationPlugin,
            water_pressure::WaterPressurePlugin,
            groundwater_depletion::GroundwaterDepletionPlugin,
            wastewater::WastewaterPlugin,
        ));

        // Waste management
        app.add_plugins((
            hazardous_waste::HazardousWastePlugin,
            landfill::LandfillPlugin,
            landfill_gas::LandfillGasPlugin,
            landfill_warning::LandfillWarningPlugin,
            waste_policies::WastePoliciesPlugin,
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
            flood_protection::FloodProtectionPlugin,
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
            nimby::NimbyPlugin,
            walkability::WalkabilityPlugin,
            form_transect::FormTransectPlugin,
            cumulative_zoning::CumulativeZoningPlugin,
            parking::ParkingPlugin,
            tutorial::TutorialPlugin,
            multi_select::MultiSelectPlugin,
        ));

        // Localization infrastructure
        app.add_plugins(localization::LocalizationPlugin);

        // Accessibility
        app.add_plugins(colorblind::ColorblindPlugin);
    }
}

pub fn tick_slow_timer(mut timer: ResMut<SlowTickTimer>, mut tick: ResMut<TickCounter>) {
    timer.tick();
    tick.0 = tick.0.wrapping_add(1);
}

/// Counter for throttling LOD/spatial grid updates to every 6th render frame (~10Hz at 60fps).
#[derive(Resource, Default)]
pub struct LodFrameCounter(u32);

fn tick_lod_frame_counter(mut counter: ResMut<LodFrameCounter>) {
    counter.0 = counter.0.wrapping_add(1);
}

pub fn lod_frame_ready(counter: Res<LodFrameCounter>) -> bool {
    counter.0.is_multiple_of(6)
}

#[cfg(test)]
mod saveable_tests {
    use super::*;
    use bevy::prelude::*;

    /// A trivial resource implementing `Saveable` for testing.
    #[derive(Resource, Default, Debug, PartialEq)]
    struct TestCounter {
        value: u32,
    }

    impl Saveable for TestCounter {
        const SAVE_KEY: &'static str = "test_counter";

        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            if self.value == 0 {
                None // skip saving default state
            } else {
                Some(self.value.to_le_bytes().to_vec())
            }
        }

        fn load_from_bytes(bytes: &[u8]) -> Self {
            let value = u32::from_le_bytes(bytes.try_into().unwrap_or([0; 4]));
            TestCounter { value }
        }
    }

    #[test]
    fn test_registry_register_and_save() {
        let mut world = World::new();
        world.insert_resource(TestCounter { value: 42 });

        let mut registry = SaveableRegistry::default();
        registry.register::<TestCounter>();

        let extensions = registry.save_all(&world);
        assert_eq!(extensions.len(), 1);
        assert!(extensions.contains_key("test_counter"));
        assert_eq!(extensions["test_counter"], 42u32.to_le_bytes().to_vec());
    }

    #[test]
    fn test_registry_save_skips_default() {
        let mut world = World::new();
        world.insert_resource(TestCounter { value: 0 });

        let mut registry = SaveableRegistry::default();
        registry.register::<TestCounter>();

        let extensions = registry.save_all(&world);
        assert!(extensions.is_empty(), "default state should be skipped");
    }

    #[test]
    fn test_registry_load_all() {
        let mut world = World::new();
        world.insert_resource(TestCounter::default());

        let mut registry = SaveableRegistry::default();
        registry.register::<TestCounter>();

        let mut extensions = BTreeMap::new();
        extensions.insert("test_counter".to_string(), 99u32.to_le_bytes().to_vec());

        registry.load_all(&mut world, &extensions);

        let counter = world.resource::<TestCounter>();
        assert_eq!(counter.value, 99);
    }

    #[test]
    fn test_registry_reset_all() {
        let mut world = World::new();
        world.insert_resource(TestCounter { value: 999 });

        let mut registry = SaveableRegistry::default();
        registry.register::<TestCounter>();

        registry.reset_all(&mut world);

        let counter = world.resource::<TestCounter>();
        assert_eq!(counter.value, 0);
    }

    #[test]
    fn test_registry_load_ignores_unknown_keys() {
        let mut world = World::new();
        world.insert_resource(TestCounter { value: 5 });

        let mut registry = SaveableRegistry::default();
        registry.register::<TestCounter>();

        let mut extensions = BTreeMap::new();
        extensions.insert("unknown_feature".to_string(), vec![0xFF, 0xFF]);

        registry.load_all(&mut world, &extensions);

        // TestCounter should be unchanged since its key wasn't in extensions
        let counter = world.resource::<TestCounter>();
        assert_eq!(counter.value, 5);
    }

    #[test]
    #[should_panic(expected = "duplicate key")]
    fn test_registry_duplicate_key_panics_in_debug() {
        let mut registry = SaveableRegistry::default();
        registry.register::<TestCounter>();

        // Second registration with the same SAVE_KEY should panic in debug builds
        registry.register::<TestCounter>();
    }
}
