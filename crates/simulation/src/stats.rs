use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::citizen::Citizen;
use crate::grid::{CellType, WorldGrid, ZoneType};

#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
pub struct CityStats {
    pub population: u32,
    pub residential_buildings: u32,
    pub commercial_buildings: u32,
    pub industrial_buildings: u32,
    pub office_buildings: u32,
    pub road_cells: u32,
    pub employed: u32,
    pub average_happiness: f32,
}

pub fn update_stats(
    slow_tick: Res<crate::SlowTickTimer>,
    grid: Res<WorldGrid>,
    citizens: Query<&crate::citizen::CitizenDetails, With<Citizen>>,
    _buildings: Query<&Building>,
    mut stats: ResMut<CityStats>,
    virtual_pop: Res<crate::virtual_population::VirtualPopulation>,
) {
    if !slow_tick.should_run() {
        return;
    }
    let mut r = 0u32;
    let mut c = 0u32;
    let mut i = 0u32;
    let mut o = 0u32;
    let mut roads = 0u32;

    for cell in &grid.cells {
        if cell.cell_type == CellType::Road {
            roads += 1;
        }
        if cell.building_id.is_some() {
            let zone = cell.zone;
            if zone.is_residential() {
                r += 1;
            } else if zone.is_commercial() {
                c += 1;
            } else if zone == ZoneType::Industrial {
                i += 1;
            } else if zone == ZoneType::Office {
                o += 1;
            }
        }
    }

    let pop = citizens.iter().count() as u32;
    let total_happiness: f32 = citizens.iter().map(|c| c.happiness).sum();
    let avg_happiness = if pop > 0 {
        total_happiness / pop as f32
    } else {
        0.0
    };

    stats.population = pop + virtual_pop.total_virtual;
    stats.residential_buildings = r;
    stats.commercial_buildings = c;
    stats.industrial_buildings = i;
    stats.office_buildings = o;
    stats.road_cells = roads;
    stats.average_happiness = avg_happiness;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_default() {
        let stats = CityStats::default();
        assert_eq!(stats.population, 0);
        assert_eq!(stats.road_cells, 0);
    }
}
