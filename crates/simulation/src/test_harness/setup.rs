//! Builder methods for road, zone, and infrastructure setup in integration tests.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::bulldoze_refund;
use crate::economy::CityBudget;
use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::road_graph_csr::CsrGraph;
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;
use crate::services::{ServiceBuilding, ServiceType};
use crate::time_of_day::GameClock;
use crate::utilities::{UtilitySource, UtilityType};
use crate::weather::Weather;

use super::TestCity;

impl TestCity {
    // -----------------------------------------------------------------------
    // Budget, roads, zones, and infrastructure
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

    /// Remove a road segment by its index in the segment store.
    /// Records endpoint node IDs in `removed_segment_endpoints` before
    /// stripping connectivity (for intersection mesh invalidation).
    pub fn remove_segment_by_index(&mut self, segment_index: usize) {
        let world = self.app.world_mut();
        world.resource_scope(|world, mut segments: Mut<RoadSegmentStore>| {
            let seg_id = segments.segments[segment_index].id;
            world.resource_scope(|world, mut grid: Mut<WorldGrid>| {
                world.resource_scope(|_world, mut roads: Mut<RoadNetwork>| {
                    segments.remove_segment(seg_id, &mut grid, &mut roads);
                });
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
            UtilityType::PowerPlant
            | UtilityType::NuclearPlant
            | UtilityType::OilPlant
            | UtilityType::GasPlant => 120,
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
}
