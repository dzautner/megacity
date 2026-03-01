use std::collections::VecDeque;

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::grid::{CellType, WorldGrid};
use crate::roads::RoadNetwork;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum UtilityType {
    PowerPlant,
    SolarFarm,
    WindTurbine,
    WaterTower,
    SewagePlant,
    NuclearPlant,
    Geothermal,
    PumpingStation,
    WaterTreatment,
    HydroDam,
    OilPlant,
    GasPlant,
}

impl UtilityType {
    pub fn is_power(self) -> bool {
        matches!(
            self,
            UtilityType::PowerPlant
                | UtilityType::SolarFarm
                | UtilityType::WindTurbine
                | UtilityType::NuclearPlant
                | UtilityType::Geothermal
                | UtilityType::HydroDam
                | UtilityType::OilPlant
                | UtilityType::GasPlant
        )
    }
    pub fn is_water(self) -> bool {
        matches!(
            self,
            UtilityType::WaterTower
                | UtilityType::SewagePlant
                | UtilityType::PumpingStation
                | UtilityType::WaterTreatment
        )
    }

    pub fn name(self) -> &'static str {
        match self {
            UtilityType::PowerPlant => "Power Plant",
            UtilityType::SolarFarm => "Solar Farm",
            UtilityType::WindTurbine => "Wind Turbine",
            UtilityType::WaterTower => "Water Tower",
            UtilityType::SewagePlant => "Sewage Plant",
            UtilityType::NuclearPlant => "Nuclear Plant",
            UtilityType::Geothermal => "Geothermal Plant",
            UtilityType::PumpingStation => "Pumping Station",
            UtilityType::WaterTreatment => "Water Treatment",
            UtilityType::HydroDam => "Hydroelectric Dam",
            UtilityType::OilPlant => "Oil Power Plant",
            UtilityType::GasPlant => "Gas Power Plant",
        }
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct UtilitySource {
    pub utility_type: UtilityType,
    pub grid_x: usize,
    pub grid_y: usize,
    pub range: u32,
}

pub fn propagate_utilities(
    mut grid: ResMut<WorldGrid>,
    roads: Res<RoadNetwork>,
    weather: Res<crate::weather::Weather>,
    sources: Query<(Ref<UtilitySource>,)>,
    mut visited_buf: Local<Vec<bool>>,
) {
    // Skip if nothing changed: roads, weather, or utility sources
    let sources_changed = sources.iter().any(|(s,)| s.is_changed());
    if !roads.is_changed() && !weather.is_changed() && !sources_changed {
        return;
    }
    // Reset all utility coverage
    for cell in &mut grid.cells {
        cell.has_power = false;
        cell.has_water = false;
    }

    // Lazily initialize the reusable visited buffer
    let grid_len = grid.width * grid.height;
    if visited_buf.len() != grid_len {
        *visited_buf = vec![false; grid_len];
    }

    // Weather affects effective utility range
    let power_mult = weather.power_multiplier(); // >1 in winter = reduced range
    let water_mult = weather.water_multiplier();

    // BFS from each source through road network
    for (source,) in &sources {
        let range_mult = if source.utility_type.is_power() {
            1.0 / power_mult
        } else {
            1.0 / water_mult
        };
        let effective_range = (source.range as f32 * range_mult) as u32;
        bfs_propagate(&mut grid, &source, effective_range, &mut visited_buf);
    }
}

/// Radius (Manhattan distance) around each visited road cell within which
/// grass/zone cells receive utility coverage.
const SEEP_RADIUS: i32 = 2;

fn bfs_propagate(
    grid: &mut WorldGrid,
    source: &UtilitySource,
    effective_range: u32,
    visited: &mut [bool],
) {
    let width = grid.width;
    let height = grid.height;
    visited.fill(false);
    let mut queue = VecDeque::new();

    let sx = source.grid_x;
    let sy = source.grid_y;
    queue.push_back(((sx, sy), 0u32));
    visited[sy * width + sx] = true;

    // Mark the source cell
    mark_cell(grid, sx, sy, source.utility_type);

    while let Some(((x, y), dist)) = queue.pop_front() {
        if dist >= effective_range {
            continue;
        }

        // Mark grass cells within SEEP_RADIUS of this road cell
        mark_nearby_grass(grid, x, y, width, height, source.utility_type);

        // Continue BFS through adjacent road cells
        let (neighbors, ncount) = grid.neighbors4(x, y);
        for &(nx, ny) in &neighbors[..ncount] {
            let idx = ny * width + nx;
            if visited[idx] {
                continue;
            }
            if grid.get(nx, ny).cell_type == CellType::Road {
                visited[idx] = true;
                mark_cell(grid, nx, ny, source.utility_type);
                queue.push_back(((nx, ny), dist + 1));
            }
        }
    }
}

/// Marks all grass cells within `SEEP_RADIUS` Manhattan distance of (cx, cy).
fn mark_nearby_grass(
    grid: &mut WorldGrid,
    cx: usize,
    cy: usize,
    width: usize,
    height: usize,
    utility: UtilityType,
) {
    let cx_i = cx as i32;
    let cy_i = cy as i32;
    for dy in -SEEP_RADIUS..=SEEP_RADIUS {
        for dx in -SEEP_RADIUS..=SEEP_RADIUS {
            if dx.abs() + dy.abs() > SEEP_RADIUS {
                continue;
            }
            let nx = cx_i + dx;
            let ny = cy_i + dy;
            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                continue;
            }
            let ux = nx as usize;
            let uy = ny as usize;
            if grid.get(ux, uy).cell_type == CellType::Grass {
                mark_cell(grid, ux, uy, utility);
            }
        }
    }
}

fn mark_cell(grid: &mut WorldGrid, x: usize, y: usize, utility: UtilityType) {
    let cell = grid.get_mut(x, y);
    if utility.is_power() {
        cell.has_power = true;
    }
    if utility.is_water() {
        cell.has_water = true;
    }
}

/// Public wrapper for `bfs_propagate` used by integration tests.
#[doc(hidden)]
pub fn bfs_propagate_pub(
    grid: &mut WorldGrid,
    source: &UtilitySource,
    effective_range: u32,
    visited: &mut [bool],
) {
    bfs_propagate(grid, source, effective_range, visited);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    #[test]
    fn test_bfs_range_limits() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();

        // Lay road from 10 to 30
        for x in 10..=30 {
            roads.place_road(&mut grid, x, 10);
        }

        let source = UtilitySource {
            utility_type: UtilityType::PowerPlant,
            grid_x: 10,
            grid_y: 10,
            range: 5,
        };

        let mut visited = vec![false; GRID_WIDTH * GRID_HEIGHT];
        bfs_propagate(&mut grid, &source, source.range, &mut visited);

        // Cell at distance 4 should have power
        assert!(grid.get(14, 10).has_power);
        // Cell at distance 15 should not
        assert!(!grid.get(25, 10).has_power);
    }

    #[test]
    fn test_disconnected_roads_no_coverage() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut roads = RoadNetwork::default();

        // Two disconnected road segments
        for x in 10..=15 {
            roads.place_road(&mut grid, x, 10);
        }
        for x in 20..=25 {
            roads.place_road(&mut grid, x, 10);
        }

        let source = UtilitySource {
            utility_type: UtilityType::WaterTower,
            grid_x: 10,
            grid_y: 10,
            range: 50,
        };

        let mut visited = vec![false; GRID_WIDTH * GRID_HEIGHT];
        bfs_propagate(&mut grid, &source, source.range, &mut visited);

        // Connected segment should have water
        assert!(grid.get(15, 10).has_water);
        // Disconnected segment should not
        assert!(!grid.get(20, 10).has_water);
    }
}

pub struct UtilitiesPlugin;

impl Plugin for UtilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            propagate_utilities
                .after(crate::stats::update_stats)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
