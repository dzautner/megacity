use std::collections::VecDeque;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::weather::Weather;
use crate::SlowTickTimer;

/// Heating plant types with different cost/efficiency/capacity profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeatingPlantType {
    /// Small local boiler: cheap, small radius, moderate efficiency.
    SmallBoiler,
    /// District heating plant: expensive, large radius, good efficiency.
    DistrictHeating,
    /// Geothermal heating: very expensive, very large radius, excellent efficiency.
    Geothermal,
}

impl HeatingPlantType {
    pub fn name(self) -> &'static str {
        match self {
            HeatingPlantType::SmallBoiler => "Small Boiler",
            HeatingPlantType::DistrictHeating => "District Heating Plant",
            HeatingPlantType::Geothermal => "Geothermal Heating",
        }
    }

    /// BFS propagation range in grid cells.
    pub fn range(self) -> u32 {
        match self {
            HeatingPlantType::SmallBoiler => 15,
            HeatingPlantType::DistrictHeating => 40,
            HeatingPlantType::Geothermal => 60,
        }
    }

    /// Maximum heat output (0-255 at the source).
    pub fn capacity(self) -> u8 {
        match self {
            HeatingPlantType::SmallBoiler => 180,
            HeatingPlantType::DistrictHeating => 240,
            HeatingPlantType::Geothermal => 255,
        }
    }

    /// Efficiency factor (0.0-1.0): higher means less fuel cost per unit of heat.
    pub fn efficiency(self) -> f32 {
        match self {
            HeatingPlantType::SmallBoiler => 0.65,
            HeatingPlantType::DistrictHeating => 0.80,
            HeatingPlantType::Geothermal => 0.95,
        }
    }

    /// Monthly operating cost per unit of heat delivered.
    pub fn cost_per_unit(self) -> f64 {
        match self {
            HeatingPlantType::SmallBoiler => 0.08,
            HeatingPlantType::DistrictHeating => 0.05,
            HeatingPlantType::Geothermal => 0.03,
        }
    }
}

/// ECS component marking an entity as a heating plant.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct HeatingPlant {
    pub plant_type: HeatingPlantType,
    pub grid_x: usize,
    pub grid_y: usize,
    pub capacity: u8,
    pub efficiency: f32,
}

/// Per-cell heating level grid (0 = no heat, 255 = maximum heat).
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct HeatingGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for HeatingGrid {
    fn default() -> Self {
        let n = GRID_WIDTH * GRID_HEIGHT;
        Self {
            levels: vec![0; n],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl HeatingGrid {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }

    /// Returns true if the cell has meaningful heating coverage (threshold > 0).
    pub fn is_heated(&self, x: usize, y: usize) -> bool {
        self.get(x, y) > 0
    }
}

/// Aggregate statistics about the heating network, updated each slow tick.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeatingStats {
    /// Number of cells with heating level > 0.
    pub total_heated_cells: u32,
    /// Fraction of zoned cells that have heating coverage (0.0-1.0).
    pub coverage_pct: f32,
    /// Estimated monthly cost of running all heating plants.
    pub monthly_cost: f64,
    /// Weighted average efficiency across all active plants.
    pub efficiency: f32,
}

/// Heating demand factor based on current temperature.
/// Returns a multiplier: 0.0 when warm (no heating needed), up to 1.0+ when very cold.
pub fn heating_demand(weather: &Weather) -> f32 {
    // Below 10C, heating demand starts; at -10C it's at maximum.
    // Above 10C, no heating demand.
    let comfort_threshold = 10.0;
    if weather.temperature >= comfort_threshold {
        0.0
    } else {
        // Linear ramp from 0 at 10C to 1.0 at -10C
        ((comfort_threshold - weather.temperature) / 20.0).clamp(0.0, 1.5)
    }
}

/// System: propagate heating from HeatingPlant entities via BFS, update HeatingGrid and HeatingStats.
/// Runs on slow tick (every 100 ticks).
pub fn update_heating(
    timer: Res<SlowTickTimer>,
    world_grid: Res<WorldGrid>,
    weather: Res<Weather>,
    plants: Query<&HeatingPlant>,
    mut heating_grid: ResMut<HeatingGrid>,
    mut heating_stats: ResMut<HeatingStats>,
    mut visited_buf: Local<Vec<bool>>,
) {
    if !timer.should_run() {
        return;
    }

    let demand = heating_demand(&weather);

    // Clear grid
    heating_grid.levels.fill(0);

    // If no demand, skip propagation but still update stats
    if demand <= 0.0 {
        heating_stats.total_heated_cells = 0;
        heating_stats.coverage_pct = 0.0;
        heating_stats.monthly_cost = 0.0;
        heating_stats.efficiency = 0.0;
        return;
    }

    let grid_len = world_grid.width * world_grid.height;
    if visited_buf.len() != grid_len {
        *visited_buf = vec![false; grid_len];
    }

    let mut total_efficiency = 0.0f32;
    let mut plant_count = 0u32;
    let mut total_cost = 0.0f64;

    for plant in &plants {
        bfs_propagate_heat(
            &world_grid,
            &mut heating_grid,
            plant,
            demand,
            &mut visited_buf,
        );
        total_efficiency += plant.efficiency;
        plant_count += 1;
    }

    // Compute stats
    let mut heated_cells = 0u32;
    let mut zoned_cells = 0u32;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = world_grid.get(x, y);
            if cell.zone != ZoneType::None {
                zoned_cells += 1;
                let heat_level = heating_grid.get(x, y);
                if heat_level > 0 {
                    heated_cells += 1;
                }
            }
        }
    }

    // Monthly cost scales with total heat output, demand, and plant cost profiles
    for plant in &plants {
        let plant_heat: u64 = count_plant_heat(&heating_grid, &world_grid, plant);
        total_cost += plant_heat as f64 * plant.plant_type.cost_per_unit() * demand as f64;
    }

    heating_stats.total_heated_cells = heated_cells;
    heating_stats.coverage_pct = if zoned_cells > 0 {
        heated_cells as f32 / zoned_cells as f32
    } else {
        0.0
    };
    heating_stats.monthly_cost = total_cost;
    heating_stats.efficiency = if plant_count > 0 {
        total_efficiency / plant_count as f32
    } else {
        0.0
    };
}

/// BFS heat propagation from a single heating plant.
/// Heat decays linearly with distance from the plant.
fn bfs_propagate_heat(
    world_grid: &WorldGrid,
    heating_grid: &mut HeatingGrid,
    plant: &HeatingPlant,
    demand: f32,
    visited: &mut [bool],
) {
    let width = world_grid.width;
    visited.fill(false);

    let mut queue: VecDeque<((usize, usize), u32)> = VecDeque::new();
    let sx = plant.grid_x;
    let sy = plant.grid_y;
    let range = plant.plant_type.range();
    let max_heat = (plant.capacity as f32 * demand).min(255.0) as u8;

    queue.push_back(((sx, sy), 0));
    visited[sy * width + sx] = true;

    // Set heat at source
    let current = heating_grid.get(sx, sy);
    heating_grid.set(sx, sy, current.max(max_heat));

    while let Some(((x, y), dist)) = queue.pop_front() {
        if dist >= range {
            continue;
        }

        let (neighbors, ncount) = world_grid.neighbors4(x, y);
        for &(nx, ny) in &neighbors[..ncount] {
            let idx = ny * width + nx;
            if visited[idx] {
                continue;
            }

            let cell_type = world_grid.get(nx, ny).cell_type;
            if cell_type == CellType::Water {
                continue;
            }

            visited[idx] = true;
            let new_dist = dist + 1;

            // Heat decays linearly with distance
            let decay = 1.0 - (new_dist as f32 / range as f32);
            let heat_at_cell = (max_heat as f32 * decay).max(0.0) as u8;

            // Take the max of existing heat and new heat (multiple plants can overlap)
            let current = heating_grid.get(nx, ny);
            if heat_at_cell > current {
                heating_grid.set(nx, ny, heat_at_cell);
            }

            // Continue BFS through roads and grass
            if cell_type == CellType::Road || cell_type == CellType::Grass {
                queue.push_back(((nx, ny), new_dist));
            }
        }
    }
}

/// Count total heat output attributed to a plant (approximate: sum of heat in its range).
fn count_plant_heat(
    heating_grid: &HeatingGrid,
    world_grid: &WorldGrid,
    plant: &HeatingPlant,
) -> u64 {
    let range = plant.plant_type.range() as i32;
    let mut total = 0u64;

    let min_x = (plant.grid_x as i32 - range).max(0) as usize;
    let max_x = (plant.grid_x as i32 + range).min(GRID_WIDTH as i32 - 1) as usize;
    let min_y = (plant.grid_y as i32 - range).max(0) as usize;
    let max_y = (plant.grid_y as i32 + range).min(GRID_HEIGHT as i32 - 1) as usize;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if world_grid.get(x, y).zone != ZoneType::None {
                total += heating_grid.get(x, y) as u64;
            }
        }
    }

    total
}

/// Happiness penalty for unheated buildings in cold weather.
pub const HEATING_COLD_PENALTY: f32 = 10.0;
/// Happiness bonus for heated buildings in cold weather.
pub const HEATING_WARM_BONUS: f32 = 3.0;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roads::RoadNetwork;

    #[test]
    fn test_heating_plant_types() {
        assert!(HeatingPlantType::SmallBoiler.range() < HeatingPlantType::DistrictHeating.range());
        assert!(HeatingPlantType::DistrictHeating.range() < HeatingPlantType::Geothermal.range());

        assert!(HeatingPlantType::SmallBoiler.efficiency() < HeatingPlantType::Geothermal.efficiency());
        assert!(HeatingPlantType::SmallBoiler.cost_per_unit() > HeatingPlantType::Geothermal.cost_per_unit());
    }

    #[test]
    fn test_heating_demand() {
        let mut weather = Weather::default();

        // Warm weather: no demand
        weather.temperature = 25.0;
        assert_eq!(heating_demand(&weather), 0.0);

        // At threshold: no demand
        weather.temperature = 10.0;
        assert_eq!(heating_demand(&weather), 0.0);

        // Below threshold: positive demand
        weather.temperature = 0.0;
        let d = heating_demand(&weather);
        assert!(d > 0.0, "demand should be positive at 0C, got {}", d);
        assert!(d <= 1.0, "demand at 0C should be <= 1.0, got {}", d);

        // Very cold: high demand
        weather.temperature = -10.0;
        let d = heating_demand(&weather);
        assert!(d >= 1.0, "demand at -10C should be >= 1.0, got {}", d);
    }

    #[test]
    fn test_heating_grid_default() {
        let grid = HeatingGrid::default();
        assert_eq!(grid.levels.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!(!grid.is_heated(0, 0));
    }

    #[test]
    fn test_bfs_heat_propagation() {
        let mut world_grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut heating_grid = HeatingGrid::default();
        let mut roads = RoadNetwork::default();

        // Lay a road
        for x in 100..=120 {
            roads.place_road(&mut world_grid, x, 100);
        }

        let plant = HeatingPlant {
            plant_type: HeatingPlantType::SmallBoiler,
            grid_x: 100,
            grid_y: 100,
            capacity: HeatingPlantType::SmallBoiler.capacity(),
            efficiency: HeatingPlantType::SmallBoiler.efficiency(),
        };

        let mut visited = vec![false; GRID_WIDTH * GRID_HEIGHT];
        bfs_propagate_heat(&world_grid, &mut heating_grid, &plant, 1.0, &mut visited);

        // Source should be heated
        assert!(heating_grid.is_heated(100, 100));

        // Nearby cell should be heated
        assert!(heating_grid.is_heated(105, 100));

        // Heat should decay with distance
        let heat_near = heating_grid.get(101, 100);
        let heat_far = heating_grid.get(110, 100);
        assert!(heat_near > heat_far, "heat should decay: near={} far={}", heat_near, heat_far);
    }

    #[test]
    fn test_heating_stats_default() {
        let stats = HeatingStats::default();
        assert_eq!(stats.total_heated_cells, 0);
        assert_eq!(stats.coverage_pct, 0.0);
        assert_eq!(stats.monthly_cost, 0.0);
    }
}
