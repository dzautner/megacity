// =============================================================================
// Zoning: Tel Aviv neighborhood zone assignment.
// =============================================================================

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid, ZoneType};

use super::coastline_x;

// =============================================================================
// Zoning (Tel Aviv neighborhoods)
// =============================================================================

#[allow(dead_code)]
fn zone_tel_aviv(grid: &WorldGrid, commands: &mut bevy::prelude::Commands) {
    // We need mutable grid but also read it for adjacency checks.
    // Clone zone assignments, then apply.
    let mut zone_map: Vec<(usize, usize, ZoneType)> = Vec::new();

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Grass || cell.zone != ZoneType::None {
                continue;
            }

            // Must be adjacent to a road
            let (n4, n4c) = grid.neighbors4(x, y);
            let has_road = n4[..n4c]
                .iter()
                .any(|&(nx, ny)| grid.get(nx, ny).cell_type == CellType::Road);
            if !has_road {
                continue;
            }

            let xf = x as f32;
            let yf = y as f32;
            let hash = x.wrapping_mul(31).wrapping_add(y.wrapping_mul(37));

            // Check if near coast
            let near_coast = xf < coastline_x(yf) + 12.0;

            let zone = if yf < 70.0 && xf < 80.0 {
                // Jaffa & Neve Tzedek: mixed old neighborhood
                match hash % 6 {
                    0..=2 => ZoneType::ResidentialLow,
                    3..=4 => ZoneType::CommercialLow,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if near_coast && yf < 160.0 {
                // Coastal strip: hotels, commercial, high-end residential
                match hash % 5 {
                    0..=1 => ZoneType::CommercialHigh,
                    2..=3 => ZoneType::ResidentialHigh,
                    _ => ZoneType::Office,
                }
            } else if xf > 70.0 && xf < 145.0 && yf > 70.0 && yf < 120.0 {
                // Central Tel Aviv / White City: dense residential + commercial
                match hash % 8 {
                    0..=3 => ZoneType::ResidentialHigh,
                    4..=5 => ZoneType::CommercialLow,
                    6 => ZoneType::Office,
                    _ => ZoneType::CommercialHigh,
                }
            } else if xf > 100.0 && xf < 150.0 && yf > 100.0 && yf < 115.0 {
                // Azrieli / Hashalom area: office towers
                match hash % 4 {
                    0..=1 => ZoneType::Office,
                    2 => ZoneType::CommercialHigh,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if xf > 145.0 && xf < 185.0 {
                // East of center, along Ayalon: industrial + commercial
                match hash % 8 {
                    0..=2 => ZoneType::Industrial,
                    3..=5 => ZoneType::CommercialLow,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if yf > 192.0 {
                // Ramat Aviv: residential suburbs + university area
                match hash % 6 {
                    0..=3 => ZoneType::ResidentialLow,
                    4 => ZoneType::ResidentialHigh,
                    _ => ZoneType::CommercialLow,
                }
            } else if xf > 70.0 && xf < 150.0 && yf > 120.0 && yf < 170.0 {
                // North-central: residential with some commercial
                match hash % 8 {
                    0..=4 => ZoneType::ResidentialHigh,
                    5..=6 => ZoneType::CommercialLow,
                    _ => ZoneType::Office,
                }
            } else {
                // Fallback: residential
                if hash % 3 == 0 {
                    ZoneType::ResidentialLow
                } else {
                    ZoneType::ResidentialHigh
                }
            };

            zone_map.push((x, y, zone));
        }
    }

    // Apply (need to drop immutable borrow first -- we use commands for deferred grid mutation)
    // Actually we can't mutate grid here since we took it as &WorldGrid.
    // We'll apply zones after this function returns. Store them and apply in init_world.
    // For now, let's use a different approach: store zone_map as a resource and apply later.
    // Actually, simpler: just pass &mut WorldGrid. Let me fix the signature.
    let _ = commands;
    let _ = zone_map;
}

#[allow(dead_code)]
pub fn apply_zones(grid: &mut WorldGrid) {
    // Precompute which cells are near roads (within manhattan distance 5)
    let zone_depth: isize = 5;
    let mut near_road = vec![false; GRID_WIDTH * GRID_HEIGHT];
    for ry in 0..GRID_HEIGHT {
        for rx in 0..GRID_WIDTH {
            if grid.get(rx, ry).cell_type != CellType::Road {
                continue;
            }
            for dy in -zone_depth..=zone_depth {
                for dx in -zone_depth..=zone_depth {
                    if dx.abs() + dy.abs() > zone_depth {
                        continue;
                    }
                    let nx = rx as isize + dx;
                    let ny = ry as isize + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                    {
                        near_road[ny as usize * GRID_WIDTH + nx as usize] = true;
                    }
                }
            }
        }
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell_type = grid.get(x, y).cell_type;
            let current_zone = grid.get(x, y).zone;
            if cell_type != CellType::Grass || current_zone != ZoneType::None {
                continue;
            }

            // Must be within zone_depth cells of a road
            if !near_road[y * GRID_WIDTH + x] {
                continue;
            }

            let xf = x as f32;
            let yf = y as f32;
            let hash = x.wrapping_mul(31).wrapping_add(y.wrapping_mul(37));
            let near_coast = xf < coastline_x(yf) + 12.0;

            let zone = if yf < 70.0 && xf < 80.0 {
                match hash % 6 {
                    0..=2 => ZoneType::ResidentialLow,
                    3..=4 => ZoneType::CommercialLow,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if near_coast && yf < 160.0 {
                match hash % 5 {
                    0..=1 => ZoneType::CommercialHigh,
                    2..=3 => ZoneType::ResidentialHigh,
                    _ => ZoneType::Office,
                }
            } else if xf > 70.0 && xf < 145.0 && yf > 70.0 && yf < 120.0 {
                match hash % 8 {
                    0..=3 => ZoneType::ResidentialHigh,
                    4..=5 => ZoneType::CommercialLow,
                    6 => ZoneType::Office,
                    _ => ZoneType::CommercialHigh,
                }
            } else if xf > 100.0 && xf < 150.0 && yf > 100.0 && yf < 115.0 {
                match hash % 4 {
                    0..=1 => ZoneType::Office,
                    2 => ZoneType::CommercialHigh,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if xf > 145.0 && xf < 185.0 {
                match hash % 8 {
                    0..=2 => ZoneType::Industrial,
                    3..=5 => ZoneType::CommercialLow,
                    _ => ZoneType::ResidentialHigh,
                }
            } else if yf > 192.0 {
                match hash % 6 {
                    0..=3 => ZoneType::ResidentialLow,
                    4 => ZoneType::ResidentialHigh,
                    _ => ZoneType::CommercialLow,
                }
            } else if xf > 70.0 && xf < 150.0 && yf > 120.0 && yf < 170.0 {
                match hash % 8 {
                    0..=4 => ZoneType::ResidentialHigh,
                    5..=6 => ZoneType::CommercialLow,
                    _ => ZoneType::Office,
                }
            } else if hash % 3 == 0 {
                ZoneType::ResidentialLow
            } else {
                ZoneType::ResidentialHigh
            };

            grid.get_mut(x, y).zone = zone;
        }
    }
}
