use bevy::prelude::*;
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

pub fn building_spawner(
    mut commands: Commands,
    mut grid: ResMut<WorldGrid>,
    demand: Res<ZoneDemand>,
    mut timer: ResMut<BuildingSpawnTimer>,
) {
    timer.0 += 1;
    if timer.0 < SPAWN_INTERVAL {
        return;
    }
    timer.0 = 0;

    let mut rng = rand::thread_rng();

    // Try to spawn buildings in zoned cells with demand
    for zone in [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
    ] {
        if demand.demand_for(zone) < 0.1 {
            continue;
        }

        let spawn_chance = demand.demand_for(zone);
        let mut spawned = 0;
        let max_per_tick = 50;

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if spawned >= max_per_tick {
                    break;
                }
                let cell = grid.get(x, y);
                if cell.zone != zone || cell.building_id.is_some() {
                    continue;
                }
                if cell.cell_type != CellType::Grass {
                    continue;
                }
                if !is_adjacent_to_road(&grid, x, y) {
                    continue;
                }
                // Require power and water infrastructure
                if !cell.has_power || !cell.has_water {
                    continue;
                }

                if rng.gen::<f32>() > spawn_chance {
                    continue;
                }

                let capacity = Building::capacity_for_level(zone, 1);
                let construction_ticks = 100; // ~10 seconds at 10Hz
                let entity = commands
                    .spawn((
                        Building {
                            zone_type: zone,
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

                grid.get_mut(x, y).building_id = Some(entity);
                spawned += 1;
            }
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
}
