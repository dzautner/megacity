use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{WorldGrid, ZoneType};

// =============================================================================
// Helpers
// =============================================================================

/// Returns the valid 4-connected neighbors of cell (x, y).
pub(crate) fn neighbors4(x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(4);
    if x > 0 {
        result.push((x - 1, y));
    }
    if x + 1 < GRID_WIDTH {
        result.push((x + 1, y));
    }
    if y > 0 {
        result.push((x, y - 1));
    }
    if y + 1 < GRID_HEIGHT {
        result.push((x, y + 1));
    }
    result
}

/// Returns the valid 8-connected neighbors of cell (x, y).
pub(crate) fn neighbors8(x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(8);
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                result.push((nx as usize, ny as usize));
            }
        }
    }
    result
}

/// Checks if there is an industrial zone within `radius` cells of (x, y).
pub(crate) fn is_near_industrial(grid: &WorldGrid, x: usize, y: usize, radius: i32) -> bool {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx.abs() + dy.abs() > radius {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0
                && ny >= 0
                && (nx as usize) < GRID_WIDTH
                && (ny as usize) < GRID_HEIGHT
                && grid.get(nx as usize, ny as usize).zone == ZoneType::Industrial
            {
                return true;
            }
        }
    }
    false
}
