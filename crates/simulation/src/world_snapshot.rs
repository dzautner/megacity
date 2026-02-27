//! WorldSnapshot — spatial state serialization for LLM-readable city summaries.
//!
//! Provides [`build_world_snapshot`] to capture the current state of the city
//! (buildings, services, utilities, roads, zones, water) into a serializable
//! struct. Formatting helpers live in [`crate::world_snapshot_format`].

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::services::{ServiceBuilding, ServiceType};
use crate::utilities::{UtilitySource, UtilityType};

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldSnapshot {
    pub buildings: Vec<BuildingEntry>,
    pub services: Vec<ServiceEntry>,
    pub utilities: Vec<UtilityEntry>,
    pub road_cells: Vec<RoadCellEntry>,
    pub zone_regions: Vec<ZoneRegion>,
    pub water_regions: Vec<WaterRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingEntry {
    pub pos: (u32, u32),
    pub zone_type: ZoneType,
    pub level: u8,
    pub capacity: u32,
    pub occupancy: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub pos: (u32, u32),
    pub service_type: ServiceType,
    pub radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityEntry {
    pub pos: (u32, u32),
    pub utility_type: UtilityType,
    pub range: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadCellEntry {
    pub pos: (u32, u32),
    pub road_type: RoadType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneRegion {
    pub min: (u32, u32),
    pub max: (u32, u32),
    pub zone_type: ZoneType,
    pub building_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterRegion {
    pub min: (u32, u32),
    pub max: (u32, u32),
}

// ---------------------------------------------------------------------------
// Snapshot builder
// ---------------------------------------------------------------------------

/// Build a [`WorldSnapshot`] from the current ECS world state.
///
/// Queries `Building`, `ServiceBuilding`, and `UtilitySource` entities, then
/// scans the `WorldGrid` for road cells, zone regions, and water regions.
pub fn build_world_snapshot(world: &mut World) -> WorldSnapshot {
    let buildings = collect_buildings(world);
    let services = collect_services(world);
    let utilities = collect_utilities(world);

    let grid = world.resource::<WorldGrid>();
    let road_cells = collect_road_cells(grid);
    let zone_regions = collect_zone_regions(grid);
    let water_regions = collect_water_regions(grid);

    WorldSnapshot {
        buildings,
        services,
        utilities,
        road_cells,
        zone_regions,
        water_regions,
    }
}

fn collect_buildings(world: &mut World) -> Vec<BuildingEntry> {
    let mut entries = Vec::new();
    let mut query = world.query::<&Building>();
    for building in query.iter(world) {
        entries.push(BuildingEntry {
            pos: (building.grid_x as u32, building.grid_y as u32),
            zone_type: building.zone_type,
            level: building.level,
            capacity: building.capacity,
            occupancy: building.occupants,
        });
    }
    entries
}

fn collect_services(world: &mut World) -> Vec<ServiceEntry> {
    let mut entries = Vec::new();
    let mut query = world.query::<&ServiceBuilding>();
    for service in query.iter(world) {
        entries.push(ServiceEntry {
            pos: (service.grid_x as u32, service.grid_y as u32),
            service_type: service.service_type,
            radius: service.radius,
        });
    }
    entries
}

fn collect_utilities(world: &mut World) -> Vec<UtilityEntry> {
    let mut entries = Vec::new();
    let mut query = world.query::<&UtilitySource>();
    for utility in query.iter(world) {
        entries.push(UtilityEntry {
            pos: (utility.grid_x as u32, utility.grid_y as u32),
            utility_type: utility.utility_type,
            range: utility.range,
        });
    }
    entries
}

fn collect_road_cells(grid: &WorldGrid) -> Vec<RoadCellEntry> {
    let mut entries = Vec::new();
    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = grid.get(x, y);
            if cell.cell_type == CellType::Road {
                entries.push(RoadCellEntry {
                    pos: (x as u32, y as u32),
                    road_type: cell.road_type,
                });
            }
        }
    }
    entries
}

// ---------------------------------------------------------------------------
// Zone clustering (row-scan rectangular merge)
// ---------------------------------------------------------------------------

/// Cluster zone cells into rectangular regions by scanning row-by-row and
/// extending runs downward where possible.
fn collect_zone_regions(grid: &WorldGrid) -> Vec<ZoneRegion> {
    let w = grid.width;
    let h = grid.height;
    let mut visited = vec![false; w * h];
    let mut regions = Vec::new();

    for y in 0..h {
        let mut x = 0;
        while x < w {
            let idx = y * w + x;
            let cell = grid.get(x, y);
            if visited[idx] || cell.zone == ZoneType::None {
                x += 1;
                continue;
            }

            let zone = cell.zone;

            // Find the end of this horizontal run of the same zone type
            let mut x_end = x;
            while x_end + 1 < w
                && !visited[y * w + x_end + 1]
                && grid.get(x_end + 1, y).zone == zone
            {
                x_end += 1;
            }

            // Extend downward
            let mut y_end = y;
            'outer: loop {
                if y_end + 1 >= h {
                    break;
                }
                let next_y = y_end + 1;
                for cx in x..=x_end {
                    let ci = next_y * w + cx;
                    if visited[ci] || grid.get(cx, next_y).zone != zone {
                        break 'outer;
                    }
                }
                y_end = next_y;
            }

            // Count buildings in this region and mark visited
            let mut building_count = 0u32;
            for ry in y..=y_end {
                for rx in x..=x_end {
                    visited[ry * w + rx] = true;
                    if grid.get(rx, ry).building_id.is_some() {
                        building_count += 1;
                    }
                }
            }

            regions.push(ZoneRegion {
                min: (x as u32, y as u32),
                max: (x_end as u32, y_end as u32),
                zone_type: zone,
                building_count,
            });

            x = x_end + 1;
        }
    }

    regions
}

// ---------------------------------------------------------------------------
// Water region detection
// ---------------------------------------------------------------------------

/// Detect contiguous water regions by scanning row-by-row and merging adjacent
/// water runs into bounding boxes.
fn collect_water_regions(grid: &WorldGrid) -> Vec<WaterRegion> {
    let w = grid.width;
    let h = grid.height;
    let mut visited = vec![false; w * h];
    let mut regions = Vec::new();

    for y in 0..h {
        let mut x = 0;
        while x < w {
            let idx = y * w + x;
            let cell = grid.get(x, y);
            if visited[idx] || cell.cell_type != CellType::Water {
                x += 1;
                continue;
            }

            // Find horizontal run of water
            let mut x_end = x;
            while x_end + 1 < w
                && !visited[y * w + x_end + 1]
                && grid.get(x_end + 1, y).cell_type == CellType::Water
            {
                x_end += 1;
            }

            // Extend downward
            let mut y_end = y;
            'outer: loop {
                if y_end + 1 >= h {
                    break;
                }
                let next_y = y_end + 1;
                for cx in x..=x_end {
                    let ci = next_y * w + cx;
                    if visited[ci] || grid.get(cx, next_y).cell_type != CellType::Water {
                        break 'outer;
                    }
                }
                y_end = next_y;
            }

            // Mark visited
            for ry in y..=y_end {
                for rx in x..=x_end {
                    visited[ry * w + rx] = true;
                }
            }

            regions.push(WaterRegion {
                min: (x as u32, y as u32),
                max: (x_end as u32, y_end as u32),
            });

            x = x_end + 1;
        }
    }

    regions
}

// ---------------------------------------------------------------------------
// Plugin (empty — snapshot is built on demand)
// ---------------------------------------------------------------------------

pub struct WorldSnapshotPlugin;

impl Plugin for WorldSnapshotPlugin {
    fn build(&self, _app: &mut App) {}
}
