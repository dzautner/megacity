//! PROG-007: Water Physics and Flood Simulation (Timberborn inspired).
//!
//! Implements a per-cell shallow-water model where water flows over terrain
//! following elevation gradients. Rain adds water, evaporation removes it, and
//! buildings in flooded cells take damage.
//!
//! ## Resources
//! - [`WaterGrid`] — per-cell water depth (metres)
//! - [`WaterPhysicsState`] — aggregate statistics and configuration
//!
//! ## Systems
//! - [`add_rainfall`] — distributes precipitation onto the grid each slow tick
//! - [`simulate_water_flow`] — spreads water from high to low surface elevation
//! - [`apply_evaporation`] — removes water at a configurable rate
//! - [`detect_floods_and_damage`] — marks flooded cells and damages buildings

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::WorldGrid;
use crate::weather::Weather;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum water depth (metres) to consider a cell "flooded".
pub const FLOOD_DEPTH_THRESHOLD: f32 = 0.15;

/// Fraction of water that can flow out of a cell per iteration.
const FLOW_RATE: f32 = 0.25;

/// Number of flow iterations per slow tick.
const FLOW_ITERATIONS: u32 = 4;

/// Base evaporation rate (metres per slow tick) at 20 C.
const BASE_EVAPORATION: f32 = 0.005;

/// Extra evaporation per degree Celsius above 20 C.
const EVAPORATION_TEMP_FACTOR: f32 = 0.0005;

/// Conversion: precipitation_intensity (in/hr) to metres per slow tick.
/// precipitation_intensity maxes around 4.0; slow tick ~ 10 game-seconds.
/// 1 in/hr = 0.0254 m/hr; at 10s ~ 0.0254/360 ≈ 7e-5; scaled up for gameplay.
const PRECIP_TO_METRES: f32 = 0.008;

/// Dollar damage per metre of flood depth per building capacity unit per tick.
const DAMAGE_PER_METRE: f64 = 50.0;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Per-cell water depth grid (metres above terrain surface).
#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct WaterGrid {
    pub cells: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl Default for WaterGrid {
    fn default() -> Self {
        Self {
            cells: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl WaterGrid {
    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.cells[self.index(x, y)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        let idx = self.index(x, y);
        self.cells[idx] = val;
    }

    /// True if any cell exceeds the flood threshold.
    pub fn has_flooding(&self) -> bool {
        self.cells.iter().any(|&d| d >= FLOOD_DEPTH_THRESHOLD)
    }

    /// Reset all depths to zero.
    pub fn clear(&mut self) {
        self.cells.iter_mut().for_each(|d| *d = 0.0);
    }
}

/// Aggregate statistics and configuration for the water physics simulation.
#[derive(Resource, Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct WaterPhysicsState {
    /// Number of cells with water depth >= [`FLOOD_DEPTH_THRESHOLD`].
    pub flooded_cell_count: u32,
    /// Maximum water depth across all cells (metres).
    pub max_depth: f32,
    /// Total volume of water on the grid (sum of depths).
    pub total_volume: f32,
    /// Cumulative building damage from flooding ($).
    pub cumulative_damage: f64,
    /// Whether the simulation is active. Disabled during drought.
    pub enabled: bool,
}

impl Default for WaterPhysicsState {
    fn default() -> Self {
        Self {
            flooded_cell_count: 0,
            max_depth: 0.0,
            total_volume: 0.0,
            cumulative_damage: 0.0,
            enabled: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Saveable
// ---------------------------------------------------------------------------

impl crate::Saveable for WaterPhysicsState {
    const SAVE_KEY: &'static str = "water_physics";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// System: rainfall input
// ---------------------------------------------------------------------------

/// Distributes precipitation from the weather system onto the water grid.
/// Only adds water to non-water terrain cells (natural water bodies are static).
pub fn add_rainfall(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    state: Res<WaterPhysicsState>,
    world_grid: Res<WorldGrid>,
    mut water: ResMut<WaterGrid>,
) {
    if !timer.should_run() || !state.enabled {
        return;
    }

    let intensity = weather.precipitation_intensity;
    if intensity <= 0.0 {
        return;
    }

    let rain_depth = intensity * PRECIP_TO_METRES;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            // Skip natural water bodies (they have a fixed water surface)
            if world_grid.get(x, y).cell_type == crate::grid::CellType::Water {
                continue;
            }
            let idx = water.index(x, y);
            water.cells[idx] += rain_depth;
        }
    }
}

// ---------------------------------------------------------------------------
// System: shallow-water flow
// ---------------------------------------------------------------------------

/// Spreads water from cells with higher surface elevation to lower neighbours.
///
/// Surface elevation = terrain elevation + water depth. Water flows proportionally
/// to the surface height difference, capped by [`FLOW_RATE`].
pub fn simulate_water_flow(
    timer: Res<SlowTickTimer>,
    state: Res<WaterPhysicsState>,
    world_grid: Res<WorldGrid>,
    mut water: ResMut<WaterGrid>,
) {
    if !timer.should_run() || !state.enabled {
        return;
    }

    for _ in 0..FLOW_ITERATIONS {
        let snapshot: Vec<f32> = water.cells.clone();

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let idx = y * water.width + x;
                let depth = snapshot[idx];
                if depth <= 0.0 {
                    continue;
                }

                let elev = world_grid.get(x, y).elevation;
                let surface = elev + depth;

                // Gather lower neighbours (4-connected)
                let (neighbors, count) = world_grid.neighbors4(x, y);
                let mut lower: [(usize, usize, f32); 4] = [(0, 0, 0.0); 4];
                let mut lower_count = 0usize;
                let mut total_diff = 0.0_f32;

                for &(nx, ny) in &neighbors[..count] {
                    let n_idx = ny * water.width + nx;
                    let n_elev = world_grid.get(nx, ny).elevation;
                    let n_surface = n_elev + snapshot[n_idx];

                    if n_surface < surface {
                        let diff = surface - n_surface;
                        lower[lower_count] = (nx, ny, diff);
                        lower_count += 1;
                        total_diff += diff;
                    }
                }

                if lower_count == 0 || total_diff <= 0.0 {
                    continue;
                }

                let transferable = depth * FLOW_RATE;
                water.cells[idx] -= transferable;

                for &(nx, ny, diff) in &lower[..lower_count] {
                    let fraction = diff / total_diff;
                    let n_idx = ny * water.width + nx;
                    water.cells[n_idx] += transferable * fraction;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// System: evaporation
// ---------------------------------------------------------------------------

/// Removes water at a temperature-dependent rate each slow tick.
pub fn apply_evaporation(
    timer: Res<SlowTickTimer>,
    weather: Res<Weather>,
    state: Res<WaterPhysicsState>,
    mut water: ResMut<WaterGrid>,
) {
    if !timer.should_run() || !state.enabled {
        return;
    }

    let temp_bonus = (weather.temperature - 20.0).max(0.0) * EVAPORATION_TEMP_FACTOR;
    let evap = BASE_EVAPORATION + temp_bonus;

    for depth in water.cells.iter_mut() {
        if *depth > 0.0 {
            *depth = (*depth - evap).max(0.0);
        }
    }
}

// ---------------------------------------------------------------------------
// System: flood detection and building damage
// ---------------------------------------------------------------------------

/// Detects flooded cells, applies damage to buildings, and updates aggregate stats.
pub fn detect_floods_and_damage(
    timer: Res<SlowTickTimer>,
    mut state: ResMut<WaterPhysicsState>,
    water: Res<WaterGrid>,
    buildings: Query<&Building>,
) {
    if !timer.should_run() || !state.enabled {
        return;
    }

    let mut flooded = 0u32;
    let mut max_depth = 0.0f32;
    let mut total_volume = 0.0f32;
    let mut tick_damage = 0.0f64;

    // Aggregate grid stats
    for &depth in &water.cells {
        if depth >= FLOOD_DEPTH_THRESHOLD {
            flooded += 1;
        }
        if depth > max_depth {
            max_depth = depth;
        }
        total_volume += depth;
    }

    // Building damage
    for building in &buildings {
        let bx = building.grid_x;
        let by = building.grid_y;
        if bx >= GRID_WIDTH || by >= GRID_HEIGHT {
            continue;
        }
        let depth = water.get(bx, by);
        if depth < FLOOD_DEPTH_THRESHOLD {
            continue;
        }
        tick_damage += depth as f64 * building.capacity as f64 * DAMAGE_PER_METRE;
    }

    state.flooded_cell_count = flooded;
    state.max_depth = max_depth;
    state.total_volume = total_volume;
    state.cumulative_damage += tick_damage;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct WaterPhysicsPlugin;

impl Plugin for WaterPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterGrid>()
            .init_resource::<WaterPhysicsState>()
            .add_systems(
                FixedUpdate,
                (
                    add_rainfall,
                    simulate_water_flow.after(add_rainfall),
                    apply_evaporation.after(simulate_water_flow),
                    detect_floods_and_damage.after(apply_evaporation),
                )
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<WaterPhysicsState>();
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_water_grid_default() {
        let g = WaterGrid::default();
        assert_eq!(g.cells.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!(g.cells.iter().all(|&d| d == 0.0));
    }

    #[test]
    fn test_water_grid_get_set() {
        let mut g = WaterGrid::default();
        g.set(10, 20, 1.5);
        assert!((g.get(10, 20) - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_water_grid_has_flooding() {
        let mut g = WaterGrid::default();
        assert!(!g.has_flooding());
        g.set(5, 5, FLOOD_DEPTH_THRESHOLD);
        assert!(g.has_flooding());
    }

    #[test]
    fn test_water_grid_has_flooding_below_threshold() {
        let mut g = WaterGrid::default();
        g.set(5, 5, FLOOD_DEPTH_THRESHOLD - 0.01);
        assert!(!g.has_flooding());
    }

    #[test]
    fn test_water_grid_clear() {
        let mut g = WaterGrid::default();
        g.set(10, 10, 5.0);
        g.set(20, 20, 3.0);
        g.clear();
        assert!(g.cells.iter().all(|&d| d == 0.0));
    }

    #[test]
    fn test_water_grid_index() {
        let g = WaterGrid::default();
        assert_eq!(g.index(0, 0), 0);
        assert_eq!(g.index(1, 0), 1);
        assert_eq!(g.index(0, 1), GRID_WIDTH);
    }

    #[test]
    fn test_state_default() {
        let s = WaterPhysicsState::default();
        assert_eq!(s.flooded_cell_count, 0);
        assert!((s.max_depth).abs() < f32::EPSILON);
        assert!((s.total_volume).abs() < f32::EPSILON);
        assert!((s.cumulative_damage).abs() < f64::EPSILON);
        assert!(s.enabled);
    }

    #[test]
    fn test_saveable_key() {
        assert_eq!(
            <WaterPhysicsState as crate::Saveable>::SAVE_KEY,
            "water_physics"
        );
    }

    #[test]
    fn test_serde_roundtrip_state() {
        let s = WaterPhysicsState {
            flooded_cell_count: 42,
            max_depth: 1.5,
            total_volume: 100.0,
            cumulative_damage: 9999.0,
            enabled: true,
        };
        let bytes = bitcode::encode(&s);
        let restored: WaterPhysicsState = bitcode::decode(&bytes).unwrap();
        assert_eq!(restored.flooded_cell_count, 42);
        assert!((restored.max_depth - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_serde_roundtrip_grid() {
        let mut g = WaterGrid::default();
        g.set(5, 5, 2.0);
        let json = serde_json::to_string(&g).expect("serialize");
        let restored: WaterGrid = serde_json::from_str(&json).expect("deserialize");
        assert!((restored.get(5, 5) - 2.0).abs() < f32::EPSILON);
        assert!((restored.get(0, 0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_flood_depth_threshold_positive() {
        assert!(FLOOD_DEPTH_THRESHOLD > 0.0);
    }

    #[test]
    fn test_flow_rate_in_range() {
        assert!(FLOW_RATE > 0.0 && FLOW_RATE <= 1.0);
    }
}

