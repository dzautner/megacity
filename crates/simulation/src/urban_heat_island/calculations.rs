use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::trees::TreeGrid;

use super::constants::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `true` when the given hour of day is considered nighttime for UHI
/// amplification purposes.
pub(crate) fn is_nighttime(hour: u32) -> bool {
    hour >= NIGHT_START_HOUR || hour <= NIGHT_END_HOUR
}

/// Compute the surface heat factor for a cell based on its type, zone, and
/// tree coverage.
pub(crate) fn surface_heat_factor(cell_type: CellType, zone: ZoneType, has_tree: bool) -> f32 {
    match cell_type {
        CellType::Water => SURFACE_WATER,
        CellType::Road => SURFACE_ASPHALT,
        CellType::Grass => {
            if has_tree {
                SURFACE_VEGETATION
            } else {
                match zone {
                    // Buildings with different roof types based on zone density.
                    ZoneType::Industrial => SURFACE_ASPHALT, // dark roofs
                    ZoneType::ResidentialHigh
                    | ZoneType::CommercialHigh
                    | ZoneType::Office
                    | ZoneType::MixedUse => SURFACE_CONCRETE, // concrete/mixed
                    ZoneType::ResidentialLow | ZoneType::CommercialLow => SURFACE_LIGHT_ROOF,
                    ZoneType::ResidentialMedium => SURFACE_CONCRETE,
                    ZoneType::None => {
                        // Undeveloped grass -- slightly negative (vegetation)
                        SURFACE_VEGETATION
                    }
                }
            }
        }
    }
}

/// Compute the local green fraction in a 5x5 neighborhood centered on `(cx, cy)`.
/// Green cells include trees and undeveloped grass (no building, no road).
pub(crate) fn local_green_fraction(
    grid: &WorldGrid,
    tree_grid: &TreeGrid,
    cx: usize,
    cy: usize,
) -> f32 {
    let mut green_count: u32 = 0;
    let mut total: u32 = 0;

    let radius = 2i32; // 5x5 neighbourhood
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                continue;
            }
            let ux = nx as usize;
            let uy = ny as usize;
            total += 1;

            let cell = grid.get(ux, uy);
            if tree_grid.has_tree(ux, uy)
                || (cell.cell_type == CellType::Grass
                    && cell.zone == ZoneType::None
                    && cell.building_id.is_none())
                || cell.cell_type == CellType::Water
            {
                green_count += 1;
            }
        }
    }

    if total == 0 {
        0.0
    } else {
        green_count as f32 / total as f32
    }
}
