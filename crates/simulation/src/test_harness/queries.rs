//! Query and simulation-tick methods for `TestCity`.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenState, CitizenStateComp};
use crate::economy::CityBudget;
use crate::grid::{Cell, CellType, WorldGrid, ZoneType};
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;
use crate::time_of_day::GameClock;
use crate::SlowTickTimer;

use super::TestCity;

impl TestCity {
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

    /// Get the number of road segments in the store.
    pub fn segment_count(&self) -> usize {
        let segments = self.app.world().resource::<RoadSegmentStore>();
        segments.segments.len()
    }

    /// Get the road type of a segment by its index in the segment store.
    pub fn segment_road_type(&self, segment_index: usize) -> Option<crate::grid::RoadType> {
        let segments = self.app.world().resource::<RoadSegmentStore>();
        segments.segments.get(segment_index).map(|s| s.road_type)
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
}
