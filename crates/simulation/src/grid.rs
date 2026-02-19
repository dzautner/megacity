use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::CELL_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CellType {
    Grass,
    Water,
    Road,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ZoneType {
    #[default]
    None,
    ResidentialLow,
    ResidentialHigh,
    CommercialLow,
    CommercialHigh,
    Industrial,
    Office,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RoadType {
    #[default]
    Local, // 2-lane, speed 30, $10, enables zoning
    Avenue,    // 4-lane, speed 50, $20, enables zoning
    Boulevard, // 6-lane, speed 60, $30, enables zoning
    Highway,   // 4-lane divided, speed 100, $40, NO zoning, high noise
    OneWay,    // 2-lane one-way, speed 40, $15
    Path,      // pedestrian, speed 5, $5, no vehicles
}

impl RoadType {
    pub fn speed(self) -> f32 {
        match self {
            RoadType::Local => 30.0,
            RoadType::Avenue => 50.0,
            RoadType::Boulevard => 60.0,
            RoadType::Highway => 100.0,
            RoadType::OneWay => 40.0,
            RoadType::Path => 5.0,
        }
    }

    pub fn cost(self) -> f64 {
        match self {
            RoadType::Local => 10.0,
            RoadType::Avenue => 20.0,
            RoadType::Boulevard => 30.0,
            RoadType::Highway => 40.0,
            RoadType::OneWay => 15.0,
            RoadType::Path => 5.0,
        }
    }

    pub fn lane_count(self) -> u8 {
        match self {
            RoadType::Local => 2,
            RoadType::Avenue => 4,
            RoadType::Boulevard => 6,
            RoadType::Highway => 4,
            RoadType::OneWay => 2,
            RoadType::Path => 0,
        }
    }

    pub fn allows_zoning(self) -> bool {
        matches!(
            self,
            RoadType::Local | RoadType::Avenue | RoadType::Boulevard
        )
    }

    pub fn allows_vehicles(self) -> bool {
        !matches!(self, RoadType::Path)
    }

    pub fn width_cells(self) -> usize {
        match self {
            RoadType::Local | RoadType::OneWay | RoadType::Path => 1,
            RoadType::Avenue => 1,
            RoadType::Boulevard | RoadType::Highway => 1,
        }
    }

    pub fn noise_radius(self) -> u8 {
        match self {
            RoadType::Local => 2,
            RoadType::Avenue => 3,
            RoadType::Boulevard => 4,
            RoadType::Highway => 8,
            RoadType::OneWay => 2,
            RoadType::Path => 0,
        }
    }
}

impl ZoneType {
    pub fn is_residential(self) -> bool {
        matches!(self, ZoneType::ResidentialLow | ZoneType::ResidentialHigh)
    }
    pub fn is_commercial(self) -> bool {
        matches!(self, ZoneType::CommercialLow | ZoneType::CommercialHigh)
    }
    pub fn is_job_zone(self) -> bool {
        self.is_commercial() || matches!(self, ZoneType::Industrial | ZoneType::Office)
    }
    pub fn max_level(self) -> u8 {
        match self {
            ZoneType::ResidentialLow | ZoneType::CommercialLow => 3,
            ZoneType::ResidentialHigh
            | ZoneType::CommercialHigh
            | ZoneType::Industrial
            | ZoneType::Office => 5,
            ZoneType::None => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cell {
    pub elevation: f32,
    pub cell_type: CellType,
    pub zone: ZoneType,
    pub road_type: RoadType,
    pub building_id: Option<Entity>,
    pub has_power: bool,
    pub has_water: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            elevation: 0.0,
            cell_type: CellType::Grass,
            zone: ZoneType::None,
            road_type: RoadType::Local,
            building_id: None,
            has_power: false,
            has_water: false,
        }
    }
}

#[derive(Resource, Serialize, Deserialize)]
pub struct WorldGrid {
    pub cells: Vec<Cell>,
    pub width: usize,
    pub height: usize,
}

impl WorldGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![Cell::default(); width * height],
            width,
            height,
        }
    }

    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline]
    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> &Cell {
        &self.cells[self.index(x, y)]
    }

    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        let idx = self.index(x, y);
        &mut self.cells[idx]
    }

    pub fn world_to_grid(world_x: f32, world_y: f32) -> (i32, i32) {
        let gx = (world_x / CELL_SIZE).floor() as i32;
        let gy = (world_y / CELL_SIZE).floor() as i32;
        (gx, gy)
    }

    pub fn grid_to_world(gx: usize, gy: usize) -> (f32, f32) {
        let wx = gx as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        let wy = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        (wx, wy)
    }

    /// Returns up to 4 cardinal neighbors and the count of valid entries.
    /// Use `&result[..count]` to iterate over valid neighbors.
    pub fn neighbors4(&self, x: usize, y: usize) -> ([(usize, usize); 4], usize) {
        let mut result = [(0, 0); 4];
        let mut count = 0;
        if x > 0 {
            result[count] = (x - 1, y);
            count += 1;
        }
        if x + 1 < self.width {
            result[count] = (x + 1, y);
            count += 1;
        }
        if y > 0 {
            result[count] = (x, y - 1);
            count += 1;
        }
        if y + 1 < self.height {
            result[count] = (x, y + 1);
            count += 1;
        }
        (result, count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    #[test]
    fn test_grid_coord_roundtrip() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        for gx in [0, 50, 128, 255] {
            for gy in [0, 50, 128, 255] {
                let (wx, wy) = WorldGrid::grid_to_world(gx, gy);
                let (rx, ry) = WorldGrid::world_to_grid(wx, wy);
                assert_eq!((rx as usize, ry as usize), (gx, gy));
                assert!(grid.in_bounds(gx, gy));
            }
        }
    }

    #[test]
    fn test_out_of_bounds() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert!(!grid.in_bounds(GRID_WIDTH, 0));
        assert!(!grid.in_bounds(0, GRID_HEIGHT));
    }

    #[test]
    fn test_neighbors() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert_eq!(grid.neighbors4(0, 0).1, 2);
        assert_eq!(grid.neighbors4(128, 128).1, 4);
        assert_eq!(grid.neighbors4(255, 255).1, 2);
    }
}
