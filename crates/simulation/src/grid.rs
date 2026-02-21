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
    ResidentialMedium,
    ResidentialHigh,
    CommercialLow,
    CommercialHigh,
    Industrial,
    Office,
    MixedUse,
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
    /// Returns the next upgrade tier for this road type, or `None` if already at max tier.
    /// Upgrade path: Path -> Local -> Avenue -> Boulevard, OneWay -> Avenue.
    /// Highway and Boulevard have no further upgrade.
    pub fn upgrade_tier(self) -> Option<RoadType> {
        match self {
            RoadType::Path => Some(RoadType::Local),
            RoadType::Local => Some(RoadType::Avenue),
            RoadType::Avenue => Some(RoadType::Boulevard),
            RoadType::OneWay => Some(RoadType::Avenue),
            RoadType::Boulevard | RoadType::Highway => None,
        }
    }

    /// Returns the cost to upgrade this road type to its next tier.
    /// The upgrade cost is the difference between the next tier cost and the current cost.
    /// Returns `None` if no upgrade is available.
    pub fn upgrade_cost(self) -> Option<f64> {
        self.upgrade_tier().map(|next| next.cost() - self.cost())
    }

    /// Returns the monthly maintenance cost per cell for this road type.
    /// Higher-capacity roads cost more to maintain.
    pub fn maintenance_cost(self) -> f64 {
        match self {
            RoadType::Path => 0.1,
            RoadType::Local => 0.3,
            RoadType::OneWay => 0.4,
            RoadType::Avenue => 0.5,
            RoadType::Boulevard => 1.5,
            RoadType::Highway => 2.0,
        }
    }

    /// Returns the vehicle capacity per time unit for this road type.
    /// Based on lane count and speed characteristics.
    /// Used by BPR travel time function to model congestion.
    pub fn capacity(self) -> u32 {
        match self {
            RoadType::Local => 20,
            RoadType::Avenue => 40,
            RoadType::Boulevard => 60,
            RoadType::Highway => 80,
            RoadType::OneWay => 25,
            RoadType::Path => 5,
        }
    }
}

impl ZoneType {
    pub fn is_residential(self) -> bool {
        matches!(
            self,
            ZoneType::ResidentialLow | ZoneType::ResidentialMedium | ZoneType::ResidentialHigh
        )
    }
    pub fn is_commercial(self) -> bool {
        matches!(self, ZoneType::CommercialLow | ZoneType::CommercialHigh)
    }
    pub fn is_mixed_use(self) -> bool {
        matches!(self, ZoneType::MixedUse)
    }
    pub fn is_job_zone(self) -> bool {
        self.is_commercial()
            || matches!(
                self,
                ZoneType::Industrial | ZoneType::Office | ZoneType::MixedUse
            )
    }
    pub fn max_level(self) -> u8 {
        match self {
            ZoneType::ResidentialLow | ZoneType::CommercialLow => 3,
            ZoneType::ResidentialMedium => 4,
            ZoneType::ResidentialHigh
            | ZoneType::CommercialHigh
            | ZoneType::Industrial
            | ZoneType::Office
            | ZoneType::MixedUse => 5,
            ZoneType::None => 0,
        }
    }

    /// Returns the default Floor Area Ratio (FAR) limit for this zone type.
    /// FAR = total floor area / lot area. Higher values allow denser buildings.
    pub fn default_far(self) -> f32 {
        match self {
            ZoneType::ResidentialLow => 0.5,
            ZoneType::ResidentialMedium => 1.5,
            ZoneType::ResidentialHigh => 3.0,
            ZoneType::CommercialLow => 1.5,
            ZoneType::CommercialHigh => 3.0,
            ZoneType::Industrial => 0.8,
            ZoneType::Office => 1.5,
            ZoneType::MixedUse => 3.0,
            ZoneType::None => 0.0,
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

    #[test]
    fn test_residential_medium_is_residential() {
        assert!(ZoneType::ResidentialMedium.is_residential());
        assert!(!ZoneType::ResidentialMedium.is_commercial());
        assert!(!ZoneType::ResidentialMedium.is_job_zone());
    }

    #[test]
    fn test_residential_medium_max_level() {
        assert_eq!(ZoneType::ResidentialMedium.max_level(), 4);
    }

    #[test]
    fn test_mixed_use_zone_type() {
        let mu = ZoneType::MixedUse;
        assert!(!mu.is_residential());
        assert!(!mu.is_commercial());
        assert!(mu.is_mixed_use());
        assert!(mu.is_job_zone());
        assert_eq!(mu.max_level(), 5);
    }

    #[test]
    fn test_default_far_values() {
        assert_eq!(ZoneType::ResidentialLow.default_far(), 0.5);
        assert_eq!(ZoneType::ResidentialMedium.default_far(), 1.5);
        assert_eq!(ZoneType::ResidentialHigh.default_far(), 3.0);
        assert_eq!(ZoneType::CommercialLow.default_far(), 1.5);
        assert_eq!(ZoneType::CommercialHigh.default_far(), 3.0);
        assert_eq!(ZoneType::Industrial.default_far(), 0.8);
        assert_eq!(ZoneType::Office.default_far(), 1.5);
        assert_eq!(ZoneType::MixedUse.default_far(), 3.0);
        assert_eq!(ZoneType::None.default_far(), 0.0);
    }

    #[test]
    fn test_none_zone_far_is_zero() {
        assert_eq!(ZoneType::None.default_far(), 0.0);
    }

    #[test]
    fn test_road_maintenance_cost_scales_by_type() {
        // Path should be cheapest, Highway most expensive
        assert!(RoadType::Path.maintenance_cost() < RoadType::Local.maintenance_cost());
        assert!(RoadType::Local.maintenance_cost() < RoadType::Avenue.maintenance_cost());
        assert!(RoadType::Avenue.maintenance_cost() < RoadType::Boulevard.maintenance_cost());
        assert!(RoadType::Boulevard.maintenance_cost() < RoadType::Highway.maintenance_cost());
    }

    #[test]
    fn test_road_maintenance_cost_values() {
        assert!((RoadType::Path.maintenance_cost() - 0.1).abs() < f64::EPSILON);
        assert!((RoadType::Local.maintenance_cost() - 0.3).abs() < f64::EPSILON);
        assert!((RoadType::OneWay.maintenance_cost() - 0.4).abs() < f64::EPSILON);
        assert!((RoadType::Avenue.maintenance_cost() - 0.5).abs() < f64::EPSILON);
        assert!((RoadType::Boulevard.maintenance_cost() - 1.5).abs() < f64::EPSILON);
        assert!((RoadType::Highway.maintenance_cost() - 2.0).abs() < f64::EPSILON);
    }
}
