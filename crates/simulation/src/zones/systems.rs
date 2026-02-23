use bevy::prelude::*;

use crate::buildings::{Building, MixedUseBuilding};
use crate::game_params::GameParams;
use crate::grid::{CellType, WorldGrid};

use super::demand::ZoneDemand;
use super::market::{compute_market_demand_with_params, vacancy_rate};
use super::stats::gather_zone_stats;

// ---------------------------------------------------------------------------
// ECS system
// ---------------------------------------------------------------------------

pub fn update_zone_demand(
    slow_tick: Res<crate::SlowTickTimer>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    mixed_use_buildings: Query<&MixedUseBuilding>,
    mut demand: ResMut<ZoneDemand>,
    game_params: Res<GameParams>,
) {
    if !slow_tick.should_run() {
        return;
    }

    let zs = gather_zone_stats(&grid, &buildings, &mixed_use_buildings);

    // Update tracked vacancy rates.
    demand.vacancy_residential = vacancy_rate(zs.residential_capacity, zs.residential_occupants);
    demand.vacancy_commercial = vacancy_rate(zs.commercial_capacity, zs.commercial_occupants);
    demand.vacancy_industrial = vacancy_rate(zs.industrial_capacity, zs.industrial_occupants);
    demand.vacancy_office = vacancy_rate(zs.office_capacity, zs.office_occupants);

    // Compute raw target demand values using configurable parameters.
    let zdp = &game_params.zone_demand;
    let (r_target, c_target, i_target, o_target) = compute_market_demand_with_params(&zs, zdp);

    // Apply damping: smoothly interpolate toward target to avoid oscillation.
    let damping = zdp.damping;
    demand.residential += (r_target - demand.residential) * damping;
    demand.commercial += (c_target - demand.commercial) * damping;
    demand.industrial += (i_target - demand.industrial) * damping;
    demand.office += (o_target - demand.office) * damping;

    // Ensure final values stay clamped.
    demand.residential = demand.residential.clamp(0.0, 1.0);
    demand.commercial = demand.commercial.clamp(0.0, 1.0);
    demand.industrial = demand.industrial.clamp(0.0, 1.0);
    demand.office = demand.office.clamp(0.0, 1.0);
}

pub fn is_adjacent_to_road(grid: &WorldGrid, x: usize, y: usize) -> bool {
    // Check within 2-cell radius so interior block cells can also have buildings
    for dy in -2i32..=2 {
        for dx in -2i32..=2 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0
                && ny >= 0
                && (nx as usize) < grid.width
                && (ny as usize) < grid.height
                && grid.get(nx as usize, ny as usize).cell_type == CellType::Road
            {
                return true;
            }
        }
    }
    false
}
