use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::fire::FireGrid;
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::land_value::LandValueGrid;
use crate::trees::TreeGrid;
use crate::weather::{Weather, WeatherEvent};
use crate::wind::WindState;
use crate::TickCounter;

// =============================================================================
// Resources
// =============================================================================

/// Per-cell forest fire intensity grid. 0 = no fire, 1-255 = burning intensity.
#[derive(Resource)]
pub struct ForestFireGrid {
    pub intensities: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for ForestFireGrid {
    fn default() -> Self {
        Self {
            intensities: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl ForestFireGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.intensities[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.intensities[y * self.width + x] = val;
    }
}

/// Tracks forest fire statistics over time.
#[derive(Resource, Default, Debug)]
pub struct ForestFireStats {
    pub active_fires: u32,
    pub total_area_burned: u64,
    pub fires_this_month: u32,
}

// =============================================================================
// Constants
// =============================================================================

/// How often (in ticks) the forest fire system runs.
const FIRE_UPDATE_INTERVAL: u64 = 10;

/// Lightning strike chance per cell during storms (per update cycle).
/// Applied only to cells with trees. Very low: checked against hash % 100_000.
const LIGHTNING_CHANCE_PER_CELL: u64 = 2; // 2 in 100_000

/// Chance for fire to spread from a burning building (FireGrid) to adjacent forest cells.
/// Checked against hash % 1000.
const BUILDING_FIRE_SPREAD_THRESHOLD: u64 = 50; // 5% per neighbor

/// Chance for spontaneous ignition near industrial zones in hot weather.
/// Checked against hash % 100_000.
const INDUSTRIAL_IGNITION_THRESHOLD: u64 = 5; // 5 in 100_000

/// Base fire intensity when a cell first ignites.
const INITIAL_INTENSITY: u8 = 30;

/// How much intensity decreases per update tick (natural burnout).
const BURNOUT_RATE: u8 = 2;

/// How much rain reduces fire intensity per update tick.
const RAIN_REDUCTION: u8 = 8;

/// How much storm reduces fire intensity per update tick.
const STORM_REDUCTION: u8 = 15;

/// Threshold above which a forest fire can ignite a building.
const BUILDING_IGNITION_THRESHOLD: u8 = 100;

/// Land value penalty applied per burning cell in radius 3.
const LAND_VALUE_PENALTY: u8 = 5;

// =============================================================================
// Deterministic pseudo-random helper
// =============================================================================

/// Deterministic hash for a given tick and cell index.
/// Returns a value in [0, modulus).
#[inline]
fn fire_hash(tick: u64, cell_index: usize, salt: u64) -> u64 {
    tick.wrapping_mul(7919)
        .wrapping_add(cell_index as u64)
        .wrapping_add(salt)
        .wrapping_mul(2654435761)
}

// =============================================================================
// Systems
// =============================================================================

/// Main forest fire update system.
/// Handles ignition, spread, burnout, rain suppression, and damage.
#[allow(clippy::too_many_arguments)]
pub fn update_forest_fire(
    tick: Res<TickCounter>,
    mut forest_fire: ResMut<ForestFireGrid>,
    mut fire_grid: ResMut<FireGrid>,
    mut tree_grid: ResMut<TreeGrid>,
    mut land_value: ResMut<LandValueGrid>,
    grid: Res<WorldGrid>,
    weather: Res<Weather>,
    wind: Res<WindState>,
    mut stats: ResMut<ForestFireStats>,
) {
    if tick.0 % FIRE_UPDATE_INTERVAL != 0 {
        return;
    }

    let is_storm = weather.current_event == WeatherEvent::Storm;
    let is_rain = weather.current_event == WeatherEvent::Rain;
    let is_hot = weather.temperature > 30.0;

    // Wind direction vector for spread bias
    let (wind_dx, wind_dy) = wind.direction_vector();
    let wind_speed = wind.speed;

    // We need to read the current state before modifying, so snapshot the intensities.
    let snapshot: Vec<u8> = forest_fire.intensities.clone();

    let mut active_fires: u32 = 0;
    let mut new_ignitions: u32 = 0;

    // --- Phase 1: Check for new ignitions ---
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let cell = grid.get(x, y);

            // Skip cells that are already burning
            if snapshot[idx] > 0 {
                continue;
            }

            // Water blocks fire completely
            if cell.cell_type == CellType::Water {
                continue;
            }

            let has_tree = tree_grid.has_tree(x, y);

            // Lightning strikes during storms (only on tree cells)
            if is_storm && has_tree {
                let h = fire_hash(tick.0, idx, 0) % 100_000;
                if h < LIGHTNING_CHANCE_PER_CELL {
                    forest_fire.set(x, y, INITIAL_INTENSITY);
                    new_ignitions += 1;
                    continue;
                }
            }

            // Spread from burning buildings (FireGrid) to adjacent forest cells
            if has_tree && fire_grid.get(x, y) == 0 {
                // Check if any neighbor4 is a burning building
                let neighbors = neighbors4(x, y);
                for (nx, ny) in neighbors {
                    if fire_grid.get(nx, ny) > 0 {
                        let h = fire_hash(tick.0, idx, 1) % 1000;
                        if h < BUILDING_FIRE_SPREAD_THRESHOLD {
                            forest_fire.set(x, y, INITIAL_INTENSITY);
                            new_ignitions += 1;
                            break;
                        }
                    }
                }
            }

            // Industrial zone spontaneous ignition in hot/dry weather
            if has_tree && is_hot && !is_rain && !is_storm {
                // Check if near industrial zone (within distance 3)
                if is_near_industrial(&grid, x, y, 3) {
                    let h = fire_hash(tick.0, idx, 2) % 100_000;
                    if h < INDUSTRIAL_IGNITION_THRESHOLD {
                        forest_fire.set(x, y, INITIAL_INTENSITY);
                        new_ignitions += 1;
                    }
                }
            }
        }
    }

    // --- Phase 2: Spread existing fires to adjacent cells ---
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let intensity = snapshot[idx];
            if intensity == 0 {
                continue;
            }

            // Try to spread to each of the 8 neighbors
            let neighbors = neighbors8(x, y);
            for (nx, ny) in neighbors {
                let nidx = ny * GRID_WIDTH + nx;
                let ncell = grid.get(nx, ny);

                // Already burning - skip
                if snapshot[nidx] > 0 || forest_fire.get(nx, ny) > 0 {
                    continue;
                }

                // Water blocks fire completely
                if ncell.cell_type == CellType::Water {
                    continue;
                }

                // Calculate spread probability based on fuel, wind, terrain
                let mut spread_chance: u64 = 0;

                // Trees are highly flammable
                if tree_grid.has_tree(nx, ny) {
                    spread_chance += 80; // 8% base for forested
                } else if ncell.cell_type == CellType::Grass {
                    spread_chance += 20; // 2% base for grass
                }

                // Roads slow fire spread significantly
                if ncell.cell_type == CellType::Road {
                    spread_chance = spread_chance / 4;
                }

                // Wind influence: higher chance downwind
                let dir_x = nx as f32 - x as f32;
                let dir_y = ny as f32 - y as f32;
                let alignment = dir_x * wind_dx + dir_y * wind_dy;
                if alignment > 0.0 {
                    // Downwind: increase spread chance based on wind speed
                    spread_chance += (wind_speed * 40.0) as u64;
                } else if alignment < 0.0 {
                    // Upwind: decrease spread chance
                    spread_chance = spread_chance.saturating_sub((wind_speed * 20.0) as u64);
                }

                // Fire intensity influences spread
                spread_chance = spread_chance * (intensity as u64) / 255;

                // Roll for spread
                let h = fire_hash(tick.0, nidx, 3) % 1000;
                if h < spread_chance {
                    forest_fire.set(nx, ny, INITIAL_INTENSITY);
                    new_ignitions += 1;
                }

                // If the neighbor cell has a building and fire is intense enough,
                // set the building on fire via FireGrid
                if intensity >= BUILDING_IGNITION_THRESHOLD && ncell.building_id.is_some() {
                    let h2 = fire_hash(tick.0, nidx, 4) % 1000;
                    if h2 < 30 {
                        // 3% chance to ignite building
                        fire_grid.set(nx, ny, 10);
                    }
                }
            }
        }
    }

    // --- Phase 3: Update intensities (burnout, rain suppression, damage) ---
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let mut intensity = forest_fire.intensities[idx];

            if intensity == 0 {
                continue;
            }

            // Natural burnout
            intensity = intensity.saturating_sub(BURNOUT_RATE);

            // Rain and storm reduce intensity
            if is_rain {
                intensity = intensity.saturating_sub(RAIN_REDUCTION);
            }
            if is_storm {
                intensity = intensity.saturating_sub(STORM_REDUCTION);
            }

            // Intensity growth for cells with fuel (trees)
            if tree_grid.has_tree(x, y) && intensity > 0 && intensity < 200 {
                // Fire grows while there's fuel
                intensity = intensity.saturating_add(3);
            }

            forest_fire.intensities[idx] = intensity;

            if intensity > 0 {
                active_fires += 1;

                // Damage: burn trees when intensity is high enough
                if intensity > 80 {
                    let h = fire_hash(tick.0, idx, 5) % 100;
                    if h < 20 {
                        // 20% chance per high-intensity tick to destroy tree
                        tree_grid.set(x, y, false);
                    }
                }

                // Reduce land value around fires
                let penalty_radius = 3i32;
                for dy in -penalty_radius..=penalty_radius {
                    for dx in -penalty_radius..=penalty_radius {
                        let lx = x as i32 + dx;
                        let ly = y as i32 + dy;
                        if lx >= 0
                            && ly >= 0
                            && (lx as usize) < GRID_WIDTH
                            && (ly as usize) < GRID_HEIGHT
                        {
                            let cur = land_value.get(lx as usize, ly as usize);
                            land_value.set(
                                lx as usize,
                                ly as usize,
                                cur.saturating_sub(LAND_VALUE_PENALTY),
                            );
                        }
                    }
                }
            }
        }
    }

    // --- Phase 4: Update stats ---
    stats.active_fires = active_fires;
    stats.total_area_burned += (active_fires as u64).saturating_add(new_ignitions as u64);
    stats.fires_this_month += new_ignitions;
}

// =============================================================================
// Helpers
// =============================================================================

/// Returns the valid 4-connected neighbors of cell (x, y).
fn neighbors4(x: usize, y: usize) -> Vec<(usize, usize)> {
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
fn neighbors8(x: usize, y: usize) -> Vec<(usize, usize)> {
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
fn is_near_industrial(grid: &WorldGrid, x: usize, y: usize, radius: i32) -> bool {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx.abs() + dy.abs() > radius {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                if grid.get(nx as usize, ny as usize).zone == ZoneType::Industrial {
                    return true;
                }
            }
        }
    }
    false
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forest_fire_grid_default() {
        let grid = ForestFireGrid::default();
        assert_eq!(grid.width, GRID_WIDTH);
        assert_eq!(grid.height, GRID_HEIGHT);
        assert_eq!(grid.intensities.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!(grid.intensities.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_forest_fire_grid_get_set() {
        let mut grid = ForestFireGrid::default();
        assert_eq!(grid.get(10, 20), 0);
        grid.set(10, 20, 150);
        assert_eq!(grid.get(10, 20), 150);
    }

    #[test]
    fn test_forest_fire_grid_boundary() {
        let mut grid = ForestFireGrid::default();
        grid.set(0, 0, 255);
        assert_eq!(grid.get(0, 0), 255);
        grid.set(GRID_WIDTH - 1, GRID_HEIGHT - 1, 100);
        assert_eq!(grid.get(GRID_WIDTH - 1, GRID_HEIGHT - 1), 100);
    }

    #[test]
    fn test_forest_fire_stats_default() {
        let stats = ForestFireStats::default();
        assert_eq!(stats.active_fires, 0);
        assert_eq!(stats.total_area_burned, 0);
        assert_eq!(stats.fires_this_month, 0);
    }

    #[test]
    fn test_fire_hash_deterministic() {
        let a = fire_hash(100, 5000, 0);
        let b = fire_hash(100, 5000, 0);
        assert_eq!(a, b);
    }

    #[test]
    fn test_fire_hash_varies_with_inputs() {
        let a = fire_hash(100, 5000, 0);
        let b = fire_hash(101, 5000, 0);
        let c = fire_hash(100, 5001, 0);
        let d = fire_hash(100, 5000, 1);
        // All should be different (extremely high probability)
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn test_neighbors4_center() {
        let n = neighbors4(128, 128);
        assert_eq!(n.len(), 4);
        assert!(n.contains(&(127, 128)));
        assert!(n.contains(&(129, 128)));
        assert!(n.contains(&(128, 127)));
        assert!(n.contains(&(128, 129)));
    }

    #[test]
    fn test_neighbors4_corner() {
        let n = neighbors4(0, 0);
        assert_eq!(n.len(), 2);
        assert!(n.contains(&(1, 0)));
        assert!(n.contains(&(0, 1)));
    }

    #[test]
    fn test_neighbors8_center() {
        let n = neighbors8(128, 128);
        assert_eq!(n.len(), 8);
        // Check all 8 directions
        assert!(n.contains(&(127, 127)));
        assert!(n.contains(&(128, 127)));
        assert!(n.contains(&(129, 127)));
        assert!(n.contains(&(127, 128)));
        assert!(n.contains(&(129, 128)));
        assert!(n.contains(&(127, 129)));
        assert!(n.contains(&(128, 129)));
        assert!(n.contains(&(129, 129)));
    }

    #[test]
    fn test_neighbors8_corner() {
        let n = neighbors8(0, 0);
        assert_eq!(n.len(), 3);
        assert!(n.contains(&(1, 0)));
        assert!(n.contains(&(0, 1)));
        assert!(n.contains(&(1, 1)));
    }

    #[test]
    fn test_constants_valid() {
        assert!(FIRE_UPDATE_INTERVAL > 0);
        assert!(INITIAL_INTENSITY > 0);
        assert!(BURNOUT_RATE > 0);
        assert!(RAIN_REDUCTION > BURNOUT_RATE);
        assert!(STORM_REDUCTION > RAIN_REDUCTION);
        assert!(BUILDING_IGNITION_THRESHOLD > INITIAL_INTENSITY);
    }

    #[test]
    fn test_is_near_industrial() {
        use crate::grid::WorldGrid;
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // No industrial zones by default
        assert!(!is_near_industrial(&grid, 128, 128, 3));

        // Place an industrial zone
        grid.get_mut(130, 128).zone = ZoneType::Industrial;
        assert!(is_near_industrial(&grid, 128, 128, 3));
        assert!(!is_near_industrial(&grid, 128, 128, 1));
    }

    #[test]
    fn test_burnout_reduces_intensity() {
        // Simulate burnout: intensity should decrease by BURNOUT_RATE
        let intensity: u8 = 50;
        let after_burnout = intensity.saturating_sub(BURNOUT_RATE);
        assert_eq!(after_burnout, 50 - BURNOUT_RATE);
    }

    #[test]
    fn test_rain_extinguishes_small_fires() {
        // A small fire (intensity = 5) should be extinguished by rain
        let intensity: u8 = 5;
        let after = intensity
            .saturating_sub(BURNOUT_RATE)
            .saturating_sub(RAIN_REDUCTION);
        assert_eq!(after, 0);
    }

    #[test]
    fn test_storm_extinguishes_moderate_fires() {
        // A moderate fire (intensity = 15) should be extinguished by storm
        let intensity: u8 = 15;
        let after = intensity
            .saturating_sub(BURNOUT_RATE)
            .saturating_sub(STORM_REDUCTION);
        assert_eq!(after, 0);
    }
}
