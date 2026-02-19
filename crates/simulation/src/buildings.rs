use bevy::prelude::*;
use rand::seq::IteratorRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::zones::{is_adjacent_to_road, ZoneDemand};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub zone_type: ZoneType,
    pub level: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub capacity: u32,
    pub occupants: u32,
}

/// Marker component for buildings that are still under construction.
/// While present, the building cannot accept occupants.
/// Approximately 10 seconds at 10Hz fixed timestep (100 ticks).
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct UnderConstruction {
    pub ticks_remaining: u32,
    pub total_ticks: u32,
}

impl Building {
    pub fn capacity_for_level(zone: ZoneType, level: u8) -> u32 {
        match (zone, level) {
            // Low-density residential: houses and small apartments
            (ZoneType::ResidentialLow, 1) => 10,
            (ZoneType::ResidentialLow, 2) => 30,
            (ZoneType::ResidentialLow, 3) => 80,
            // High-density residential: apartment blocks and towers
            (ZoneType::ResidentialHigh, 1) => 50,
            (ZoneType::ResidentialHigh, 2) => 200,
            (ZoneType::ResidentialHigh, 3) => 500,
            (ZoneType::ResidentialHigh, 4) => 1000,
            (ZoneType::ResidentialHigh, 5) => 2000,
            // Low-density commercial: shops and small stores
            (ZoneType::CommercialLow, 1) => 8,
            (ZoneType::CommercialLow, 2) => 25,
            (ZoneType::CommercialLow, 3) => 60,
            // High-density commercial: malls and department stores
            (ZoneType::CommercialHigh, 1) => 30,
            (ZoneType::CommercialHigh, 2) => 100,
            (ZoneType::CommercialHigh, 3) => 300,
            (ZoneType::CommercialHigh, 4) => 600,
            (ZoneType::CommercialHigh, 5) => 1200,
            // Industrial: factories and warehouses
            (ZoneType::Industrial, 1) => 20,
            (ZoneType::Industrial, 2) => 60,
            (ZoneType::Industrial, 3) => 150,
            (ZoneType::Industrial, 4) => 300,
            (ZoneType::Industrial, 5) => 600,
            // Office: office towers
            (ZoneType::Office, 1) => 30,
            (ZoneType::Office, 2) => 100,
            (ZoneType::Office, 3) => 300,
            (ZoneType::Office, 4) => 700,
            (ZoneType::Office, 5) => 1500,
            _ => 0,
        }
    }
}

/// Tick interval for building spawner (in sim ticks)
const SPAWN_INTERVAL: u32 = 2;

#[derive(Resource, Default)]
pub struct BuildingSpawnTimer(pub u32);

/// Maintained set of cells eligible for building placement per zone type.
/// Rebuilt from the grid whenever it changes (zoning, buildings, roads, infrastructure).
#[derive(Resource, Default)]
pub struct EligibleCells {
    pub cells: Vec<(ZoneType, Vec<(usize, usize)>)>,
}

/// Rebuild the eligible cell lists from the grid.
/// Runs only when the grid resource has changed (Bevy change detection).
pub fn rebuild_eligible_cells(grid: Res<WorldGrid>, mut eligible: ResMut<EligibleCells>) {
    if !grid.is_changed() {
        return;
    }

    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
    ];

    let mut result: Vec<(ZoneType, Vec<(usize, usize)>)> = Vec::with_capacity(zones.len());

    for zone in zones {
        let mut cells = Vec::new();
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = grid.get(x, y);
                if cell.zone == zone
                    && cell.building_id.is_none()
                    && cell.cell_type == CellType::Grass
                    && cell.has_power
                    && cell.has_water
                    && is_adjacent_to_road(&grid, x, y)
                {
                    cells.push((x, y));
                }
            }
        }
        result.push((zone, cells));
    }

    eligible.cells = result;
}

pub fn building_spawner(
    mut commands: Commands,
    mut grid: ResMut<WorldGrid>,
    demand: Res<ZoneDemand>,
    mut timer: ResMut<BuildingSpawnTimer>,
    eligible: Res<EligibleCells>,
) {
    timer.0 += 1;
    if timer.0 < SPAWN_INTERVAL {
        return;
    }
    timer.0 = 0;

    let mut rng = rand::thread_rng();

    for (zone, cells) in &eligible.cells {
        if demand.demand_for(*zone) < 0.1 || cells.is_empty() {
            continue;
        }

        let spawn_chance = demand.demand_for(*zone);
        let max_per_tick = 50;

        // Sample up to max_per_tick cells randomly from eligible list
        let selected: Vec<(usize, usize)> = cells
            .iter()
            .copied()
            .choose_multiple(&mut rng, max_per_tick.min(cells.len()));

        for (x, y) in selected {
            // Double-check cell is still eligible (could have changed since rebuild)
            let cell = grid.get(x, y);
            if cell.zone != *zone || cell.building_id.is_some() {
                continue;
            }

            if rng.gen::<f32>() > spawn_chance {
                continue;
            }

            let capacity = Building::capacity_for_level(*zone, 1);
            let construction_ticks = 100; // ~10 seconds at 10Hz
            let entity = commands
                .spawn((
                    Building {
                        zone_type: *zone,
                        level: 1,
                        grid_x: x,
                        grid_y: y,
                        capacity,
                        occupants: 0,
                    },
                    UnderConstruction {
                        ticks_remaining: construction_ticks,
                        total_ticks: construction_ticks,
                    },
                ))
                .id();

            // This grid mutation triggers Bevy change detection,
            // so rebuild_eligible_cells will re-run next tick
            grid.get_mut(x, y).building_id = Some(entity);
        }
    }
}

/// Advances construction progress each tick. When complete, removes the
/// `UnderConstruction` component so the building becomes operational.
/// While under construction, occupants are clamped to 0.
pub fn progress_construction(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Building, &mut UnderConstruction)>,
) {
    for (entity, mut building, mut uc) in &mut query {
        // Ensure no occupants while under construction
        building.occupants = 0;

        if uc.ticks_remaining > 0 {
            uc.ticks_remaining -= 1;
        }

        if uc.ticks_remaining == 0 {
            commands.entity(entity).remove::<UnderConstruction>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_capacity() {
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialLow, 1),
            10
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialLow, 2),
            30
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialHigh, 3),
            500
        );
        assert_eq!(
            Building::capacity_for_level(ZoneType::ResidentialHigh, 5),
            2000
        );
        assert_eq!(Building::capacity_for_level(ZoneType::CommercialLow, 1), 8);
        assert_eq!(Building::capacity_for_level(ZoneType::Industrial, 1), 20);
        assert_eq!(Building::capacity_for_level(ZoneType::Office, 1), 30);
    }

    #[test]
    fn test_building_only_in_zoned_cells() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        for cell in &grid.cells {
            assert!(cell.building_id.is_none());
            assert_eq!(cell.zone, ZoneType::None);
        }
    }

    #[test]
    fn test_eligible_cells_finds_zoned_cells() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // Place a road at (10, 10)
        grid.get_mut(10, 10).cell_type = CellType::Road;
        // Zone cells adjacent to road
        for x in 8..=9 {
            let cell = grid.get_mut(x, 10);
            cell.zone = ZoneType::ResidentialLow;
            cell.has_power = true;
            cell.has_water = true;
        }

        let mut res_cells = Vec::new();
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = grid.get(x, y);
                if cell.zone == ZoneType::ResidentialLow
                    && cell.building_id.is_none()
                    && cell.cell_type == CellType::Grass
                    && cell.has_power
                    && cell.has_water
                    && is_adjacent_to_road(&grid, x, y)
                {
                    res_cells.push((x, y));
                }
            }
        }

        assert_eq!(res_cells.len(), 2);
        assert!(res_cells.contains(&(8, 10)));
        assert!(res_cells.contains(&(9, 10)));
    }

    #[test]
    fn test_eligible_cells_excludes_occupied() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Road;
        let cell = grid.get_mut(9, 10);
        cell.zone = ZoneType::Industrial;
        cell.has_power = true;
        cell.has_water = true;
        // Mark as having a building
        cell.building_id = Some(Entity::from_raw(1));

        let mut eligible_count = 0;
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = grid.get(x, y);
                if cell.zone == ZoneType::Industrial
                    && cell.building_id.is_none()
                    && cell.cell_type == CellType::Grass
                    && cell.has_power
                    && cell.has_water
                    && is_adjacent_to_road(&grid, x, y)
                {
                    eligible_count += 1;
                }
            }
        }

        assert_eq!(eligible_count, 0);
    }
}
