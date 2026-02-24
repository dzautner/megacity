use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::fire::FireGrid;
use crate::grid::{CellType, WorldGrid};
use crate::land_value::LandValueGrid;
use crate::trees::TreeGrid;
use crate::weather::{Weather, WeatherCondition};
use crate::wind::WindState;
use crate::TickCounter;

use super::constants::*;
use super::helpers::{is_near_industrial, neighbors4, neighbors8};
use super::resources::{ForestFireGrid, ForestFireStats};

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
    if !tick.0.is_multiple_of(FIRE_UPDATE_INTERVAL) {
        return;
    }

    let is_storm = weather.current_event == WeatherCondition::Storm;
    let is_rain = weather.current_event.is_precipitation();
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
                    spread_chance /= 4;
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
// Plugin
// =============================================================================

pub struct ForestFirePlugin;

impl Plugin for ForestFirePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ForestFireGrid>()
            .init_resource::<ForestFireStats>()
            .add_systems(
                FixedUpdate,
                // Writes LandValueGrid, TreeGrid, FireGrid; must run after
                // fire_damage and after base land value is computed.
                update_forest_fire
                    .after(crate::fire::fire_damage)
                    .after(crate::land_value::update_land_value)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
