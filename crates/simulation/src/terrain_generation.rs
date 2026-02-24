//! REND-002: Procedural Terrain Generation with fBm + Hydraulic Erosion.
//!
//! Generates terrain procedurally using fractional Brownian motion (fBm) noise
//! with hydraulic erosion simulation. Supports seed-based deterministic
//! generation, water body placement, river generation, and biome assignment.

use bevy::prelude::*;
use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};
use serde::{Deserialize, Serialize};

use crate::config::{
    GRID_HEIGHT, GRID_WIDTH, TERRAIN_BASE_FREQUENCY, TERRAIN_LACUNARITY, TERRAIN_OCTAVES,
    TERRAIN_PERSISTENCE, WATER_THRESHOLD,
};
use crate::grid::{CellType, WorldGrid};

// ---------------------------------------------------------------------------
// Terrain configuration resource
// ---------------------------------------------------------------------------

/// Configuration for procedural terrain generation.
///
/// Saved/loaded via the `Saveable` trait so generated terrain parameters persist.
#[derive(
    Resource, Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode, Default,
)]
pub struct TerrainConfig {
    /// Seed for deterministic terrain generation.
    pub seed: u64,
    /// Number of hydraulic erosion iterations.
    pub erosion_iterations: u32,
    /// Whether terrain has been generated for the current seed.
    pub generated: bool,
}

/// Per-cell biome classification based on elevation and moisture.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, bitcode::Encode,
    bitcode::Decode,
)]
pub enum Biome {
    DeepWater,
    ShallowWater,
    Beach,
    Grassland,
    Forest,
    Highland,
    Mountain,
}

// ---------------------------------------------------------------------------
// fBm noise generation
// ---------------------------------------------------------------------------

/// Generate base elevation using fBm noise.
///
/// Populates a flat `Vec<f32>` (row-major, width x height) with elevation
/// values in [0, 1]. Uses the constants from `config.rs`.
fn generate_fbm_elevation(width: usize, height: usize, seed: i32) -> Vec<f32> {
    let mut noise = FastNoiseLite::with_seed(seed);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise.set_frequency(Some(TERRAIN_BASE_FREQUENCY));
    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(TERRAIN_OCTAVES));
    noise.set_fractal_gain(Some(TERRAIN_PERSISTENCE));
    noise.set_fractal_lacunarity(Some(TERRAIN_LACUNARITY));

    let mut elevations = vec![0.0_f32; width * height];
    for y in 0..height {
        for x in 0..width {
            let raw = noise.get_noise_2d(x as f32, y as f32);
            // fBm with OpenSimplex2 outputs in [-1, 1]; normalize to [0, 1]
            elevations[y * width + x] = ((raw + 1.0) * 0.5).clamp(0.0, 1.0);
        }
    }
    elevations
}

// ---------------------------------------------------------------------------
// Hydraulic erosion
// ---------------------------------------------------------------------------

/// Simulate hydraulic erosion on the elevation map.
///
/// Each iteration: a virtual raindrop is placed at a random position (seeded),
/// flows downhill depositing or eroding sediment based on slope. Produces
/// realistic valleys and ridges.
fn hydraulic_erosion(
    elevations: &mut [f32],
    width: usize,
    height: usize,
    iterations: u32,
    seed: u64,
) {
    const INERTIA: f32 = 0.05;
    const CAPACITY_FACTOR: f32 = 4.0;
    const DEPOSITION_RATE: f32 = 0.3;
    const EROSION_RATE: f32 = 0.3;
    const EVAPORATION: f32 = 0.01;
    const GRAVITY: f32 = 4.0;
    const MAX_LIFETIME: u32 = 30;
    const MIN_SEDIMENT_CAPACITY: f32 = 0.01;

    // Simple seeded PRNG (xorshift64) for drop placement.
    let mut rng_state = seed.wrapping_add(1);
    let mut next_rand = || -> f32 {
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 7;
        rng_state ^= rng_state << 17;
        (rng_state & 0x00FF_FFFF) as f32 / 0x00FF_FFFF as f32
    };

    for _ in 0..iterations {
        let mut pos_x = next_rand() * (width as f32 - 2.0) + 0.5;
        let mut pos_y = next_rand() * (height as f32 - 2.0) + 0.5;
        let mut dir_x = 0.0_f32;
        let mut dir_y = 0.0_f32;
        let mut speed = 1.0_f32;
        let mut water = 1.0_f32;
        let mut sediment = 0.0_f32;

        for _ in 0..MAX_LIFETIME {
            let ix = pos_x as usize;
            let iy = pos_y as usize;

            if ix < 1 || iy < 1 || ix >= width - 1 || iy >= height - 1 {
                break;
            }

            // Compute gradient using neighbors
            let idx = iy * width + ix;
            let h = elevations[idx];
            let h_right = elevations[idx + 1];
            let h_below = elevations[(iy + 1) * width + ix];
            let grad_x = h_right - h;
            let grad_y = h_below - h;

            // Update direction with inertia
            dir_x = dir_x * INERTIA - grad_x * (1.0 - INERTIA);
            dir_y = dir_y * INERTIA - grad_y * (1.0 - INERTIA);

            let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
            if len < 1e-6 {
                break;
            }
            dir_x /= len;
            dir_y /= len;

            // Move
            let new_x = pos_x + dir_x;
            let new_y = pos_y + dir_y;

            let nix = new_x as usize;
            let niy = new_y as usize;
            if nix >= width - 1 || niy >= height - 1 {
                break;
            }

            let new_h = elevations[niy * width + nix];
            let delta_h = new_h - h;

            // Compute sediment capacity
            let capacity =
                (-delta_h * speed * water * CAPACITY_FACTOR).max(MIN_SEDIMENT_CAPACITY);

            if sediment > capacity || delta_h > 0.0 {
                // Deposit
                let deposit = if delta_h > 0.0 {
                    sediment.min(delta_h)
                } else {
                    (sediment - capacity) * DEPOSITION_RATE
                };
                sediment -= deposit;
                elevations[idx] += deposit;
            } else {
                // Erode
                let erode = ((capacity - sediment) * EROSION_RATE).min(-delta_h);
                sediment += erode;
                elevations[idx] -= erode;
            }

            // Update speed and water
            speed = (speed * speed + delta_h * GRAVITY).max(0.0).sqrt();
            water *= 1.0 - EVAPORATION;

            pos_x = new_x;
            pos_y = new_y;
        }
    }

    // Re-clamp all elevations to [0, 1]
    for e in elevations.iter_mut() {
        *e = e.clamp(0.0, 1.0);
    }
}

// ---------------------------------------------------------------------------
// River generation
// ---------------------------------------------------------------------------

/// Trace rivers from high-elevation sources downhill to water bodies.
///
/// Finds the N highest land cells and traces steepest-descent paths to water.
/// Marks traversed cells as `CellType::Water`.
fn generate_rivers(
    elevations: &[f32],
    grid: &mut WorldGrid,
    width: usize,
    height: usize,
    num_sources: usize,
) {
    // Collect land cells sorted by elevation (descending)
    let mut land_cells: Vec<(usize, usize, f32)> = Vec::new();
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let e = elevations[y * width + x];
            if e >= WATER_THRESHOLD {
                land_cells.push((x, y, e));
            }
        }
    }
    land_cells.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Take top N distinct high-elevation sources, spaced apart
    let mut sources = Vec::new();
    for &(x, y, _) in &land_cells {
        let too_close = sources.iter().any(|&(sx, sy): &(usize, usize)| {
            let dx = x as i32 - sx as i32;
            let dy = y as i32 - sy as i32;
            dx * dx + dy * dy < 900 // min 30 cells apart
        });
        if !too_close {
            sources.push((x, y));
            if sources.len() >= num_sources {
                break;
            }
        }
    }

    // Trace each river
    for (sx, sy) in sources {
        trace_river(elevations, grid, width, height, sx, sy);
    }
}

/// Trace a single river from (start_x, start_y) downhill via steepest descent.
fn trace_river(
    elevations: &[f32],
    grid: &mut WorldGrid,
    width: usize,
    height: usize,
    start_x: usize,
    start_y: usize,
) {
    let mut x = start_x;
    let mut y = start_y;
    let max_steps = width + height;

    for _ in 0..max_steps {
        let idx = y * width + x;
        let current_elev = elevations[idx];

        // Already reached water -- done
        if current_elev < WATER_THRESHOLD {
            break;
        }

        // Mark as water (river cell)
        grid.get_mut(x, y).cell_type = CellType::Water;
        grid.get_mut(x, y).elevation = (WATER_THRESHOLD - 0.02).max(0.1);

        // Find steepest descent neighbor (8-connected)
        let mut best_x = x;
        let mut best_y = y;
        let mut best_elev = current_elev;

        for dy in [-1_i32, 0, 1] {
            for dx in [-1_i32, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                    continue;
                }
                let ne = elevations[ny as usize * width + nx as usize];
                if ne < best_elev {
                    best_elev = ne;
                    best_x = nx as usize;
                    best_y = ny as usize;
                }
            }
        }

        // No downhill neighbor -- stuck at local minimum
        if best_x == x && best_y == y {
            break;
        }

        x = best_x;
        y = best_y;
    }
}

// ---------------------------------------------------------------------------
// Biome assignment
// ---------------------------------------------------------------------------

/// Assign a biome to a cell based on elevation and a moisture estimate.
pub fn classify_biome(elevation: f32, moisture: f32) -> Biome {
    if elevation < WATER_THRESHOLD - 0.1 {
        Biome::DeepWater
    } else if elevation < WATER_THRESHOLD {
        Biome::ShallowWater
    } else if elevation < WATER_THRESHOLD + 0.05 {
        Biome::Beach
    } else if elevation > 0.85 {
        Biome::Mountain
    } else if elevation > 0.7 {
        Biome::Highland
    } else if moisture > 0.5 {
        Biome::Forest
    } else {
        Biome::Grassland
    }
}

/// Generate a moisture map using a second noise pass (different frequency/seed).
fn generate_moisture_map(width: usize, height: usize, seed: i32) -> Vec<f32> {
    let mut noise = FastNoiseLite::with_seed(seed.wrapping_add(9999));
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise.set_frequency(Some(0.012));
    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(4));
    noise.set_fractal_gain(Some(0.5));
    noise.set_fractal_lacunarity(Some(2.0));

    let mut moisture = vec![0.0_f32; width * height];
    for y in 0..height {
        for x in 0..width {
            let raw = noise.get_noise_2d(x as f32, y as f32);
            moisture[y * width + x] = ((raw + 1.0) * 0.5).clamp(0.0, 1.0);
        }
    }
    moisture
}

// ---------------------------------------------------------------------------
// Biome grid resource
// ---------------------------------------------------------------------------

/// Grid of biome assignments, one per cell.
#[derive(Resource, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct BiomeGrid {
    pub biomes: Vec<Biome>,
    pub width: usize,
    pub height: usize,
}

impl Default for BiomeGrid {
    fn default() -> Self {
        Self {
            biomes: vec![Biome::Grassland; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl BiomeGrid {
    pub fn get(&self, x: usize, y: usize) -> Biome {
        self.biomes[y * self.width + x]
    }
}

// ---------------------------------------------------------------------------
// Top-level generation orchestrator
// ---------------------------------------------------------------------------

/// Run the full procedural terrain pipeline:
/// 1. fBm noise for base elevation
/// 2. Hydraulic erosion
/// 3. Write elevations + water cells to WorldGrid
/// 4. River generation
/// 5. Biome assignment
pub fn generate_procedural_terrain(
    grid: &mut WorldGrid,
    seed: u64,
    erosion_iterations: u32,
) -> BiomeGrid {
    let width = grid.width;
    let height = grid.height;
    let noise_seed = seed as i32;

    // 1. fBm base elevation
    let mut elevations = generate_fbm_elevation(width, height, noise_seed);

    // 2. Hydraulic erosion
    if erosion_iterations > 0 {
        hydraulic_erosion(&mut elevations, width, height, erosion_iterations, seed);
    }

    // 3. Write to WorldGrid
    for y in 0..height {
        for x in 0..width {
            let e = elevations[y * width + x];
            let cell = grid.get_mut(x, y);
            cell.elevation = e;
            cell.cell_type = if e < WATER_THRESHOLD {
                CellType::Water
            } else {
                CellType::Grass
            };
        }
    }

    // 4. Rivers
    generate_rivers(&elevations, grid, width, height, 5);

    // 5. Moisture + biome assignment
    let moisture = generate_moisture_map(width, height, noise_seed);
    let mut biome_grid = BiomeGrid {
        biomes: vec![Biome::Grassland; width * height],
        width,
        height,
    };

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let cell = grid.get(x, y);
            let m = moisture[idx];
            let biome = if cell.cell_type == CellType::Water {
                if cell.elevation < WATER_THRESHOLD - 0.1 {
                    Biome::DeepWater
                } else {
                    Biome::ShallowWater
                }
            } else {
                classify_biome(cell.elevation, m)
            };
            biome_grid.biomes[idx] = biome;
        }
    }

    biome_grid
}

// ---------------------------------------------------------------------------
// Saveable implementations
// ---------------------------------------------------------------------------

impl crate::Saveable for TerrainConfig {
    const SAVE_KEY: &'static str = "terrain_config";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

impl crate::Saveable for BiomeGrid {
    const SAVE_KEY: &'static str = "biome_grid";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct TerrainGenerationPlugin;

impl Plugin for TerrainGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainConfig>();
        app.init_resource::<BiomeGrid>();

        app.init_resource::<crate::SaveableRegistry>();
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<TerrainConfig>();
        registry.register::<BiomeGrid>();
    }
}
