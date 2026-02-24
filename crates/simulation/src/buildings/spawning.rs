use bevy::prelude::*;
use rand::seq::IteratorRandom;
use rand::Rng;
use crate::sim_rng::SimRng;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::cumulative_zoning::{select_effective_zone, CumulativeZoningState};
use crate::game_params::GameParams;
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::zones::{is_adjacent_to_road, ZoneDemand};

use super::types::{max_level_for_far, Building, MixedUseBuilding, UnderConstruction};

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
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
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
    cumulative_zoning: Res<CumulativeZoningState>,
    game_params: Res<GameParams>,
    mut rng: ResMut<SimRng>,
) {
    timer.0 += 1;
    if timer.0 < game_params.building.spawn_interval_ticks {
        return;
    }
    timer.0 = 0;


    for (zone, cells) in &eligible.cells {
        if demand.demand_for(*zone) < 0.1 || cells.is_empty() {
            continue;
        }

        let spawn_chance = demand.demand_for(*zone);
        let max_per_tick = game_params.building.max_buildings_per_zone_per_tick as usize;

        // Sample up to max_per_tick cells randomly from eligible list
        let selected: Vec<(usize, usize)> = cells
            .iter()
            .copied()
            .choose_multiple(&mut rng.0, max_per_tick.min(cells.len()));

        for (x, y) in selected {
            // Double-check cell is still eligible (could have changed since rebuild)
            let cell = grid.get(x, y);
            if cell.zone != *zone || cell.building_id.is_some() {
                continue;
            }

            if rng.0.gen::<f32>() > spawn_chance {
                continue;
            }

            // When cumulative zoning is enabled, select the highest-value
            // permitted use based on market demand. Otherwise use the
            // cell's own zone type (exclusive zoning).
            let effective_zone = if cumulative_zoning.enabled {
                select_effective_zone(*zone, &demand)
            } else {
                *zone
            };

            // Cap initial level by FAR constraint (initial level is 1, but
            // max_level_for_far is guaranteed >= 1, so this is a safety check)
            let far_cap = max_level_for_far(effective_zone) as u8;
            let initial_level = 1u8.min(far_cap);
            let capacity = Building::capacity_for_level(effective_zone, initial_level);
            let construction_ticks = game_params.building.construction_ticks;
            let entity = if effective_zone == ZoneType::MixedUse {
                let (comm_cap, res_cap) = MixedUseBuilding::capacities_for_level(initial_level);
                commands
                    .spawn((
                        Building {
                            zone_type: effective_zone,
                            level: initial_level,
                            grid_x: x,
                            grid_y: y,
                            capacity,
                            occupants: 0,
                        },
                        MixedUseBuilding {
                            commercial_capacity: comm_cap,
                            commercial_occupants: 0,
                            residential_capacity: res_cap,
                            residential_occupants: 0,
                        },
                        UnderConstruction {
                            ticks_remaining: construction_ticks,
                            total_ticks: construction_ticks,
                        },
                    ))
                    .id()
            } else {
                commands
                    .spawn((
                        Building {
                            zone_type: effective_zone,
                            level: initial_level,
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
                    .id()
            };

            // This grid mutation triggers Bevy change detection,
            // so rebuild_eligible_cells will re-run next tick
            grid.get_mut(x, y).building_id = Some(entity);
        }
    }
}
