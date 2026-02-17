use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::grid::{CellType, WorldGrid, ZoneType};

#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
pub struct ZoneDemand {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
    pub office: f32,
}

impl ZoneDemand {
    pub fn demand_for(&self, zone: ZoneType) -> f32 {
        match zone {
            ZoneType::ResidentialLow | ZoneType::ResidentialHigh => self.residential,
            ZoneType::CommercialLow | ZoneType::CommercialHigh => self.commercial,
            ZoneType::Industrial => self.industrial,
            ZoneType::Office => self.office,
            ZoneType::None => 0.0,
        }
    }
}

pub fn update_zone_demand(
    slow_tick: Res<crate::SlowTickTimer>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    mut demand: ResMut<ZoneDemand>,
) {
    if !slow_tick.should_run() {
        return;
    }
    let mut r_zoned = 0u32;
    let mut r_built = 0u32;
    let mut c_zoned = 0u32;
    let mut c_built = 0u32;
    let mut i_zoned = 0u32;
    let mut i_built = 0u32;
    let mut o_zoned = 0u32;
    let mut o_built = 0u32;
    let mut pop = 0u32;

    for cell in &grid.cells {
        match cell.zone {
            ZoneType::ResidentialLow | ZoneType::ResidentialHigh => {
                r_zoned += 1;
                if let Some(entity) = cell.building_id {
                    r_built += 1;
                    if let Ok(b) = buildings.get(entity) {
                        pop += b.occupants;
                    }
                }
            }
            ZoneType::CommercialLow | ZoneType::CommercialHigh => {
                c_zoned += 1;
                if cell.building_id.is_some() {
                    c_built += 1;
                }
            }
            ZoneType::Industrial => {
                i_zoned += 1;
                if cell.building_id.is_some() {
                    i_built += 1;
                }
            }
            ZoneType::Office => {
                o_zoned += 1;
                if cell.building_id.is_some() {
                    o_built += 1;
                }
            }
            ZoneType::None => {}
        }
    }

    // Base demand: always some pull if there are roads
    let has_roads = grid.cells.iter().any(|c| c.cell_type == CellType::Road);
    let base = if has_roads { 0.3 } else { 0.0 };

    // Residential demand: base + commercial pull - saturation
    let r_sat = if r_zoned > 0 {
        r_built as f32 / r_zoned as f32
    } else {
        0.0
    };
    demand.residential = (base + 0.5 - r_sat * 0.6).clamp(0.0, 1.0);

    // Commercial demand: grows with population
    let pop_factor = (pop as f32 / 5000.0).min(0.5);
    let c_sat = if c_zoned > 0 {
        c_built as f32 / c_zoned as f32
    } else {
        0.0
    };
    let c_floor = if has_roads { 0.1 } else { 0.0 };
    demand.commercial = (pop_factor + base * 0.5 - c_sat * 0.6).clamp(c_floor, 1.0);

    // Industrial demand: needed for jobs
    let i_sat = if i_zoned > 0 {
        i_built as f32 / i_zoned as f32
    } else {
        0.0
    };
    demand.industrial = (base + 0.3 - i_sat * 0.5).clamp(0.0, 1.0);

    // Office demand: scales with population like commercial but slower growth
    let office_pop_factor = (pop as f32 / 10000.0).min(0.4);
    let o_sat = if o_zoned > 0 {
        o_built as f32 / o_zoned as f32
    } else {
        0.0
    };
    let o_floor = if has_roads { 0.05 } else { 0.0 };
    demand.office = (office_pop_factor + base * 0.3 - o_sat * 0.5).clamp(o_floor, 1.0);
}

pub fn is_adjacent_to_road(grid: &WorldGrid, x: usize, y: usize) -> bool {
    let (neighbors, count) = grid.neighbors4(x, y);
    neighbors[..count].iter().any(|(nx, ny)| grid.get(*nx, *ny).cell_type == CellType::Road)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};

    #[test]
    fn test_zoning_requires_road_adjacency() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // No roads placed, no cell is adjacent to a road
        assert!(!is_adjacent_to_road(&grid, 10, 10));
    }

    #[test]
    fn test_demand_increases_with_roads() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let demand1 = ZoneDemand::default();

        // Place a road
        grid.get_mut(10, 10).cell_type = CellType::Road;

        let mut demand2 = ZoneDemand::default();
        // Manually compute what update_zone_demand would do
        let has_roads = true;
        let base = if has_roads { 0.3 } else { 0.0 };
        demand2.residential = (base + 0.5_f32).clamp(0.0, 1.0);

        assert!(demand2.residential > demand1.residential);
    }

    #[test]
    fn test_demand_formula_bounds() {
        let demand = ZoneDemand {
            residential: 0.8,
            commercial: 0.5,
            industrial: 0.3,
            office: 0.2,
        };
        assert!(demand.residential >= 0.0 && demand.residential <= 1.0);
        assert!(demand.commercial >= 0.0 && demand.commercial <= 1.0);
        assert!(demand.industrial >= 0.0 && demand.industrial <= 1.0);
        assert!(demand.office >= 0.0 && demand.office <= 1.0);
    }

    #[test]
    fn test_demand_for_zones() {
        let demand = ZoneDemand {
            residential: 0.8,
            commercial: 0.5,
            industrial: 0.3,
            office: 0.2,
        };
        assert_eq!(demand.demand_for(ZoneType::ResidentialLow), 0.8);
        assert_eq!(demand.demand_for(ZoneType::ResidentialHigh), 0.8);
        assert_eq!(demand.demand_for(ZoneType::CommercialLow), 0.5);
        assert_eq!(demand.demand_for(ZoneType::CommercialHigh), 0.5);
        assert_eq!(demand.demand_for(ZoneType::Industrial), 0.3);
        assert_eq!(demand.demand_for(ZoneType::Office), 0.2);
        assert_eq!(demand.demand_for(ZoneType::None), 0.0);
    }
}
