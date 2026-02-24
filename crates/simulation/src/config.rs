pub const GRID_WIDTH: usize = 256;
pub const GRID_HEIGHT: usize = 256;
pub const CELL_SIZE: f32 = 16.0;
pub const CHUNK_SIZE: usize = 8;
pub const CHUNKS_X: usize = GRID_WIDTH / CHUNK_SIZE;
pub const CHUNKS_Y: usize = GRID_HEIGHT / CHUNK_SIZE;
pub const WATER_THRESHOLD: f32 = 0.35;
pub const TERRAIN_OCTAVES: i32 = 6;
pub const TERRAIN_PERSISTENCE: f32 = 0.45;
pub const TERRAIN_LACUNARITY: f32 = 2.0;
pub const TERRAIN_BASE_FREQUENCY: f32 = 0.008;
pub const WORLD_WIDTH: f32 = GRID_WIDTH as f32 * CELL_SIZE;
pub const WORLD_HEIGHT: f32 = GRID_HEIGHT as f32 * CELL_SIZE;

/// Maximum terrain height in world units. Elevation [0,1] maps to [0, TERRAIN_HEIGHT_SCALE].
pub const TERRAIN_HEIGHT_SCALE: f32 = 40.0;

/// World-space Y level for the water surface. Cells below WATER_THRESHOLD are
/// rendered at this fixed height so water appears as a flat plane.
pub const WATER_LEVEL_Y: f32 = WATER_THRESHOLD * TERRAIN_HEIGHT_SCALE;
