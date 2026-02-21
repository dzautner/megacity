//! # TestCity — headless integration test harness for Megacity
//!
//! Provides a fluent builder that wraps `bevy::app::App` + `SimulationPlugin`
//! for running integration tests without a window or renderer.

use bevy::app::App;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::buildings::Building;
use crate::bulldoze_refund;
use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::grid::{Cell, CellType, RoadType, WorldGrid, ZoneType};
use crate::groundwater;
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::natural_resources::ResourceGrid;
use crate::road_graph_csr::CsrGraph;
use crate::road_segments::RoadSegmentStore;
use crate::roads::{RoadNetwork, RoadNode};
use crate::services::{ServiceBuilding, ServiceType};
use crate::time_of_day::GameClock;
use crate::tutorial::TutorialState;
use crate::utilities::{UtilitySource, UtilityType};
use crate::weather::Weather;
use crate::world_init::SkipWorldInit;
use crate::SimulationPlugin;
use crate::SlowTickTimer;

/// A headless Bevy App wrapping `SimulationPlugin` for integration testing.
///
/// Use builder methods to set up city state, then call `tick()` to advance the
/// simulation and query/assert on the resulting ECS state.
pub struct TestCity {
    app: App,
}

impl TestCity {
    // -----------------------------------------------------------------------
    // Constructors
    // -----------------------------------------------------------------------

    /// Create a new **empty** city: a 256x256 grass grid with all resources at
    /// their defaults. The Tel Aviv map is NOT loaded.
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);

        // Insert the marker BEFORE SimulationPlugin so init_world skips.
        app.insert_resource(SkipWorldInit);
        // Skip the tutorial so it doesn't pause the GameClock on first update.
        app.insert_resource(TutorialState {
            completed: true,
            active: false,
            ..Default::default()
        });
        app.add_plugins(SimulationPlugin);

        // Insert blank world resources BEFORE the first update, so that
        // systems which depend on Res<WorldGrid> etc. don't panic.
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let (gw_grid, wq_grid) = groundwater::init_groundwater(&grid);
        app.insert_resource(grid);
        app.insert_resource(RoadNetwork::default());
        app.insert_resource(CityBudget::default());
        app.insert_resource(ResourceGrid::default());
        app.insert_resource(gw_grid);
        app.insert_resource(wq_grid);

        // Run one update so Startup systems execute (init_world will no-op).
        app.update();

        // After the first update, the tutorial starts active by default
        // (TutorialState::default() has active=true) which causes
        // `check_tutorial_progress` to pause the GameClock.
        // Force both back to a clean state for testing.
        if let Some(mut tutorial) = app.world_mut().get_resource_mut::<TutorialState>() {
            tutorial.completed = true;
            tutorial.active = false;
            tutorial.paused_by_tutorial = false;
        }
        if let Some(mut clock) = app.world_mut().get_resource_mut::<GameClock>() {
            clock.paused = false;
        }

        Self { app }
    }

    /// Create a city with the full Tel Aviv init_world map.
    /// This spawns ~10K citizens, all roads, buildings, services, and utilities.
    pub fn with_tel_aviv() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        // Skip the tutorial so it doesn't pause the GameClock on first update.
        app.insert_resource(TutorialState {
            completed: true,
            active: false,
            ..Default::default()
        });
        app.add_plugins(SimulationPlugin);
        // Run one update so Startup systems execute (init_world runs fully).
        app.update();

        // Ensure tutorial doesn't interfere with test simulation.
        if let Some(mut tutorial) = app.world_mut().get_resource_mut::<TutorialState>() {
            tutorial.completed = true;
            tutorial.active = false;
            tutorial.paused_by_tutorial = false;
        }
        if let Some(mut clock) = app.world_mut().get_resource_mut::<GameClock>() {
            clock.paused = false;
        }

        Self { app }
    }

    // -----------------------------------------------------------------------
    // World Setup (builder pattern — consumes and returns Self)
    // -----------------------------------------------------------------------

    /// Set the city treasury to the given amount.
    pub fn with_budget(mut self, treasury: f64) -> Self {
        if let Some(mut budget) = self.app.world_mut().get_resource_mut::<CityBudget>() {
            budget.treasury = treasury;
        }
        self
    }

    /// Place a straight road from (x0,y0) to (x1,y1) using the `RoadSegmentStore`.
    pub fn with_road(
        mut self,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        road_type: RoadType,
    ) -> Self {
        let world = self.app.world_mut();
        let (from, to) = {
            let (wx0, wy0) = WorldGrid::grid_to_world(x0, y0);
            let (wx1, wy1) = WorldGrid::grid_to_world(x1, y1);
            (
                bevy::math::Vec2::new(wx0, wy0),
                bevy::math::Vec2::new(wx1, wy1),
            )
        };

        world.resource_scope(|world, mut segments: Mut<RoadSegmentStore>| {
            world.resource_scope(|world, mut grid: Mut<WorldGrid>| {
                world.resource_scope(|_world, mut roads: Mut<RoadNetwork>| {
                    segments.add_straight_segment(from, to, road_type, 16.0, &mut grid, &mut roads);
                });
            });
        });

        self
    }

    /// Place a curved road from (x0,y0) through control point (cx,cy) to (x1,y1)
    /// using the `RoadSegmentStore::add_curved_segment`.
    pub fn with_curved_road(
        mut self,
        x0: usize,
        y0: usize,
        cx: usize,
        cy: usize,
        x1: usize,
        y1: usize,
        road_type: RoadType,
    ) -> Self {
        let world = self.app.world_mut();
        let (from, control, to) = {
            let (wx0, wy0) = WorldGrid::grid_to_world(x0, y0);
            let (wcx, wcy) = WorldGrid::grid_to_world(cx, cy);
            let (wx1, wy1) = WorldGrid::grid_to_world(x1, y1);
            (
                bevy::math::Vec2::new(wx0, wy0),
                bevy::math::Vec2::new(wcx, wcy),
                bevy::math::Vec2::new(wx1, wy1),
            )
        };

        world.resource_scope(|world, mut segments: Mut<RoadSegmentStore>| {
            world.resource_scope(|world, mut grid: Mut<WorldGrid>| {
                world.resource_scope(|_world, mut roads: Mut<RoadNetwork>| {
                    segments.add_curved_segment(
                        from, control, to, road_type, 16.0, &mut grid, &mut roads,
                    );
                });
            });
        });

        self
    }
    /// Remove a single road cell at (x, y). Used to test path invalidation
    /// after bulldozing.
    pub fn remove_road_at(&mut self, x: usize, y: usize) {
        let world = self.app.world_mut();
        world.resource_scope(|world, mut grid: Mut<WorldGrid>| {
            world.resource_scope(|_world, mut roads: Mut<RoadNetwork>| {
                roads.remove_road(&mut grid, x, y);
            });
        });
    }

    /// Bulldoze a road cell at (x, y) and credit the refund to the treasury.
    pub fn bulldoze_road_at(&mut self, x: usize, y: usize) {
        let world = self.app.world_mut();
        world.resource_scope(|world, mut grid: Mut<WorldGrid>| {
            let road_type = grid.get(x, y).road_type;
            world.resource_scope(|world, mut roads: Mut<RoadNetwork>| {
                if roads.remove_road(&mut grid, x, y) {
                    world.resource_scope(|_world, mut budget: Mut<CityBudget>| {
                        budget.treasury += bulldoze_refund::refund_for_road(road_type);
                    });
                }
            });
        });
    }

    /// Upgrade a road segment by its index in the segment store.
    /// Returns `Ok(new_road_type)` on success or `Err(reason)` on failure.
    pub fn upgrade_segment_by_index(
        &mut self,
        segment_index: usize,
    ) -> Result<RoadType, &'static str> {
        let world = self.app.world_mut();
        let seg_id = {
            let segments = world.resource::<RoadSegmentStore>();
            if segment_index >= segments.segments.len() {
                return Err("Segment index out of bounds");
            }
            segments.segments[segment_index].id
        };
        world.resource_scope(|world, mut segments: Mut<RoadSegmentStore>| {
            world.resource_scope(|world, mut grid: Mut<WorldGrid>| {
                world.resource_scope(|world, mut roads: Mut<RoadNetwork>| {
                    world.resource_scope(|_world, mut budget: Mut<CityBudget>| {
                        crate::road_upgrade::upgrade_segment(
                            seg_id,
                            &mut segments,
                            &mut grid,
                            &mut roads,
                            &mut budget,
                        )
                    })
                })
            })
        })
    }

    /// Get the road type of a segment by its index in the segment store.
    pub fn segment_road_type(&self, segment_index: usize) -> Option<RoadType> {
        let segments = self.app.world().resource::<RoadSegmentStore>();
        segments.segments.get(segment_index).map(|s| s.road_type)
    }

    /// Get the number of road segments in the store.
    pub fn segment_count(&self) -> usize {
        let segments = self.app.world().resource::<RoadSegmentStore>();
        segments.segments.len()
    }

    /// Bulldoze a service building at (x, y) and credit the refund to the
    /// treasury. The entity is despawned and grid cells are cleared.
    pub fn bulldoze_service_at(&mut self, x: usize, y: usize) {
        let world = self.app.world_mut();
        let entity = {
            let grid = world.resource::<WorldGrid>();
            grid.get(x, y).building_id
        };
        let Some(entity) = entity else {
            return;
        };
        // Look up the service type for refund calculation
        if let Some(service) = world.get::<ServiceBuilding>(entity) {
            let service_type = service.service_type;
            let sx = service.grid_x;
            let sy = service.grid_y;
            let (fw, fh) = ServiceBuilding::footprint(service_type);
            let refund = bulldoze_refund::refund_for_service(service_type);
            {
                let mut grid = world.resource_mut::<WorldGrid>();
                for fy in sy..sy + fh {
                    for fx in sx..sx + fw {
                        if grid.in_bounds(fx, fy) {
                            grid.get_mut(fx, fy).building_id = None;
                            grid.get_mut(fx, fy).zone = ZoneType::None;
                        }
                    }
                }
            }
            world.resource_mut::<CityBudget>().treasury += refund;
        }
        world.despawn(entity);
    }

    /// Set a single cell's zone type.
    pub fn with_zone(mut self, x: usize, y: usize, zone: ZoneType) -> Self {
        if let Some(mut grid) = self.app.world_mut().get_resource_mut::<WorldGrid>() {
            if grid.in_bounds(x, y) {
                grid.get_mut(x, y).zone = zone;
            }
        }
        self
    }

    /// Set zone type for a rectangular area (inclusive).
    pub fn with_zone_rect(
        mut self,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        zone: ZoneType,
    ) -> Self {
        if let Some(mut grid) = self.app.world_mut().get_resource_mut::<WorldGrid>() {
            for y in y0..=y1 {
                for x in x0..=x1 {
                    if grid.in_bounds(x, y) {
                        grid.get_mut(x, y).zone = zone;
                    }
                }
            }
        }
        self
    }

    /// Spawn a building at the given cell.
    pub fn with_building(mut self, x: usize, y: usize, zone: ZoneType, level: u8) -> Self {
        let capacity = Building::capacity_for_level(zone, level);
        let entity = self
            .app
            .world_mut()
            .spawn(Building {
                zone_type: zone,
                level,
                grid_x: x,
                grid_y: y,
                capacity,
                occupants: 0,
            })
            .id();
        if let Some(mut grid) = self.app.world_mut().get_resource_mut::<WorldGrid>() {
            if grid.in_bounds(x, y) {
                grid.get_mut(x, y).building_id = Some(entity);
                grid.get_mut(x, y).zone = zone;
            }
        }
        self
    }

    /// Spawn a citizen with a home and work location.
    /// The home and work buildings must already exist (use `with_building` first).
    pub fn with_citizen(mut self, home: (usize, usize), work: (usize, usize)) -> Self {
        let world = self.app.world_mut();
        let home_entity = {
            let grid = world.resource::<WorldGrid>();
            grid.get(home.0, home.1)
                .building_id
                .unwrap_or(Entity::PLACEHOLDER)
        };
        let work_entity = {
            let grid = world.resource::<WorldGrid>();
            grid.get(work.0, work.1)
                .building_id
                .unwrap_or(Entity::PLACEHOLDER)
        };

        let (hx, hy) = WorldGrid::grid_to_world(home.0, home.1);

        world.spawn((
            Citizen,
            Position { x: hx, y: hy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: home.0,
                grid_y: home.1,
                building: home_entity,
            },
            WorkLocation {
                grid_x: work.0,
                grid_y: work.1,
                building: work_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30,
                gender: Gender::Male,
                education: 2,
                happiness: 60.0,
                health: 90.0,
                salary: 3500.0,
                savings: 7000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
            ChosenTransportMode::default(),
        ));
        self
    }

    /// Spawn an unemployed citizen (no `WorkLocation`) at the given home.
    /// The home building must already exist (use `with_building` first).
    pub fn with_unemployed_citizen(mut self, home: (usize, usize)) -> Self {
        let world = self.app.world_mut();
        let home_entity = {
            let grid = world.resource::<WorldGrid>();
            grid.get(home.0, home.1)
                .building_id
                .unwrap_or(Entity::PLACEHOLDER)
        };

        let (hx, hy) = WorldGrid::grid_to_world(home.0, home.1);

        world.spawn((
            Citizen,
            Position { x: hx, y: hy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: home.0,
                grid_y: home.1,
                building: home_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30,
                gender: Gender::Male,
                education: 0,
                happiness: 60.0,
                health: 90.0,
                salary: 0.0,
                savings: 1000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
            ChosenTransportMode::default(),
        ));
        self
    }

    /// Spawn a service building at the given cell.
    pub fn with_service(mut self, x: usize, y: usize, service_type: ServiceType) -> Self {
        let entity = self
            .app
            .world_mut()
            .spawn(ServiceBuilding {
                service_type,
                grid_x: x,
                grid_y: y,
                radius: ServiceBuilding::coverage_radius(service_type),
            })
            .id();
        if let Some(mut grid) = self.app.world_mut().get_resource_mut::<WorldGrid>() {
            if grid.in_bounds(x, y) {
                grid.get_mut(x, y).building_id = Some(entity);
            }
        }
        self
    }

    /// Spawn a utility source at the given cell.
    pub fn with_utility(mut self, x: usize, y: usize, utility_type: UtilityType) -> Self {
        let range = match utility_type {
            UtilityType::PowerPlant | UtilityType::NuclearPlant => 120,
            UtilityType::WaterTower | UtilityType::PumpingStation => 90,
            _ => 50,
        };
        let entity = self
            .app
            .world_mut()
            .spawn(UtilitySource {
                utility_type,
                grid_x: x,
                grid_y: y,
                range,
            })
            .id();
        if let Some(mut grid) = self.app.world_mut().get_resource_mut::<WorldGrid>() {
            if grid.in_bounds(x, y) {
                grid.get_mut(x, y).building_id = Some(entity);
            }
        }
        self
    }

    /// Set weather conditions.
    pub fn with_weather(mut self, temperature: f32) -> Self {
        if let Some(mut weather) = self.app.world_mut().get_resource_mut::<Weather>() {
            weather.temperature = temperature;
        }
        self
    }

    /// Set the game clock hour (0.0-24.0).
    pub fn with_time(mut self, hour: f32) -> Self {
        if let Some(mut clock) = self.app.world_mut().get_resource_mut::<GameClock>() {
            clock.hour = hour;
        }
        self
    }

    /// Rebuild the CSR graph from the current RoadNetwork.
    ///
    /// This is necessary for pathfinding tests using `TestCity::new()` because
    /// the `Update` schedule (which normally triggers CSR rebuild) is never run
    /// when using `run_schedule(FixedUpdate)` directly.
    pub fn rebuild_csr(mut self) -> Self {
        let world = self.app.world_mut();
        world.resource_scope(|world, roads: Mut<RoadNetwork>| {
            let mut csr = world.resource_mut::<CsrGraph>();
            *csr = CsrGraph::from_road_network(&roads);
        });
        self
    }

    // -----------------------------------------------------------------------
    // Simulation
    // -----------------------------------------------------------------------

    /// Run N fixed-update ticks by directly executing the `FixedUpdate`
    /// schedule. This bypasses Bevy's time system entirely, which avoids
    /// issues with `MinimalPlugins` + `ScheduleRunnerPlugin` not advancing
    /// virtual time between updates.
    ///
    /// A `yield_now()` is inserted between ticks so that background threads
    /// (e.g. `AsyncComputeTaskPool`) get a chance to make progress even when
    /// the test drives the schedule in a tight loop on a low-core CI runner.
    pub fn tick(&mut self, n: u32) {
        for _ in 0..n {
            self.app.world_mut().run_schedule(FixedUpdate);
            std::thread::yield_now();
        }
    }

    /// Run until the SlowTickTimer fires at least once (~100 ticks).
    pub fn tick_slow_cycle(&mut self) {
        self.tick(SlowTickTimer::INTERVAL);
    }

    /// Run a specific number of slow cycles.
    pub fn tick_slow_cycles(&mut self, n: u32) {
        self.tick(SlowTickTimer::INTERVAL * n);
    }

    // -----------------------------------------------------------------------
    // Queries (note: Bevy's World::query() requires &mut World)
    // -----------------------------------------------------------------------

    /// Access the ECS world mutably (needed for queries in Bevy).
    pub fn world_mut(&mut self) -> &mut World {
        self.app.world_mut()
    }

    /// Get a reference to the world grid.
    pub fn grid(&self) -> &WorldGrid {
        self.app.world().resource::<WorldGrid>()
    }

    /// Get a reference to the city budget.
    pub fn budget(&self) -> &CityBudget {
        self.app.world().resource::<CityBudget>()
    }

    /// Get the game clock.
    pub fn clock(&self) -> &GameClock {
        self.app.world().resource::<GameClock>()
    }

    /// Count all citizen entities.
    pub fn citizen_count(&mut self) -> usize {
        let world = self.app.world_mut();
        world
            .query_filtered::<Entity, With<Citizen>>()
            .iter(world)
            .count()
    }

    /// Count citizens in a specific state.
    pub fn citizens_in_state(&mut self, state: CitizenState) -> usize {
        let world = self.app.world_mut();
        world
            .query::<&CitizenStateComp>()
            .iter(world)
            .filter(|s| s.0 == state)
            .count()
    }

    /// Count all building entities.
    pub fn building_count(&mut self) -> usize {
        let world = self.app.world_mut();
        world
            .query_filtered::<Entity, With<Building>>()
            .iter(world)
            .count()
    }

    /// Count buildings in a specific zone type.
    pub fn buildings_in_zone(&mut self, zone: ZoneType) -> usize {
        let world = self.app.world_mut();
        world
            .query::<&Building>()
            .iter(world)
            .filter(|b| b.zone_type == zone)
            .count()
    }

    /// Get a reference to a specific cell.
    pub fn cell(&self, x: usize, y: usize) -> &Cell {
        self.grid().get(x, y)
    }

    /// Get a reference to any resource.
    pub fn resource<T: Resource>(&self) -> &T {
        self.app.world().resource::<T>()
    }

    /// Get the road network.
    pub fn road_network(&self) -> &RoadNetwork {
        self.app.world().resource::<RoadNetwork>()
    }

    /// Get the road segment store.
    pub fn road_segments(&self) -> &RoadSegmentStore {
        self.app.world().resource::<RoadSegmentStore>()
    }

    /// Get the slow tick timer.
    pub fn slow_tick_timer(&self) -> &SlowTickTimer {
        self.app.world().resource::<SlowTickTimer>()
    }

    /// Count road cells in the grid.
    pub fn road_cell_count(&self) -> usize {
        let grid = self.grid();
        grid.cells
            .iter()
            .filter(|c| c.cell_type == CellType::Road)
            .count()
    }

    /// Count cells with a specific zone type.
    pub fn zoned_cell_count(&self, zone: ZoneType) -> usize {
        let grid = self.grid();
        grid.cells.iter().filter(|c| c.zone == zone).count()
    }

    /// Count cells that have a building_id set.
    pub fn cells_with_buildings(&self) -> usize {
        let grid = self.grid();
        grid.cells
            .iter()
            .filter(|c| c.building_id.is_some())
            .count()
    }

    /// Get total occupants across all buildings.
    pub fn total_occupants(&mut self) -> u32 {
        let world = self.app.world_mut();
        world
            .query::<&Building>()
            .iter(world)
            .map(|b| b.occupants)
            .sum()
    }

    // -----------------------------------------------------------------------
    // Assertions
    // -----------------------------------------------------------------------

    /// Assert citizen count is between min and max (inclusive).
    pub fn assert_citizen_count_between(&mut self, min: usize, max: usize) {
        let count = self.citizen_count();
        assert!(
            count >= min && count <= max,
            "Expected citizen count in [{min}, {max}], got {count}"
        );
    }

    /// Assert treasury is above a given amount.
    pub fn assert_budget_above(&self, amount: f64) {
        let treasury = self.budget().treasury;
        assert!(
            treasury > amount,
            "Expected treasury > {amount}, got {treasury}"
        );
    }

    /// Assert treasury is below a given amount.
    pub fn assert_budget_below(&self, amount: f64) {
        let treasury = self.budget().treasury;
        assert!(
            treasury < amount,
            "Expected treasury < {amount}, got {treasury}"
        );
    }

    /// Assert that a cell contains a road.
    pub fn assert_has_road(&self, x: usize, y: usize) {
        let cell = self.cell(x, y);
        assert_eq!(
            cell.cell_type,
            CellType::Road,
            "Expected road at ({x}, {y}), found {:?}",
            cell.cell_type
        );
    }

    /// Assert that a cell has a building.
    pub fn assert_has_building(&self, x: usize, y: usize) {
        let cell = self.cell(x, y);
        assert!(
            cell.building_id.is_some(),
            "Expected building at ({x}, {y}), found none"
        );
    }

    /// Assert that a cell has a specific zone type.
    pub fn assert_zone(&self, x: usize, y: usize, expected: ZoneType) {
        let cell = self.cell(x, y);
        assert_eq!(
            cell.zone, expected,
            "Expected zone {:?} at ({x}, {y}), found {:?}",
            expected, cell.zone
        );
    }

    /// Assert the road network contains a node at (x, y).
    pub fn assert_road_node_exists(&self, x: usize, y: usize) {
        let network = self.road_network();
        let node = RoadNode(x, y);
        assert!(
            network.edges.contains_key(&node),
            "Expected road node at ({x}, {y}) in RoadNetwork"
        );
    }

    /// Assert the slow tick timer has reached at least the given count.
    pub fn assert_ticks_at_least(&self, min: u32) {
        let counter = self.slow_tick_timer().counter;
        assert!(
            counter >= min,
            "Expected at least {min} ticks, got {counter}"
        );
    }

    /// Assert the game clock hour is approximately the expected value.
    pub fn assert_hour_approx(&self, expected: f32, tolerance: f32) {
        let hour = self.clock().hour;
        assert!(
            (hour - expected).abs() < tolerance,
            "Expected hour ~{expected} (±{tolerance}), got {hour}"
        );
    }

    /// Assert a resource has been initialized (exists in the world).
    pub fn assert_resource_exists<T: Resource>(&self) {
        assert!(
            self.app.world().get_resource::<T>().is_some(),
            "Expected resource {} to exist",
            std::any::type_name::<T>()
        );
    }
}
