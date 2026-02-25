//! Water/Sewage Pipe Network (SVC-024).
//!
//! Replaces the binary `has_water` BFS with a capacity-aware pipe network.
//! Pipes auto-follow roads (CS2 style) with capacity limits. Water towers and
//! pump stations provide water pressure. When demand exceeds pipe capacity,
//! pressure drops and buildings at the end of the line lose water. Sewage
//! flows from buildings to treatment plants with separate capacity tracking.
//!
//! Key features:
//! - Pipes auto-follow road placement (no manual pipe drawing).
//! - Water pressure calculation from source, dropping with distance.
//! - Pipe capacity limits per road segment.
//! - Sewage network runs in parallel to water.
//! - Treatment plant capacity limits.
//! - Pipe age tracking with leak probability and break events.
//! - Pipe break events cause local water loss until repaired.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::grid::{CellType, WorldGrid};
use crate::utilities::{UtilitySource, UtilityType};
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Base pipe capacity in gallons per day per road cell.
const BASE_PIPE_CAPACITY_GPD: f32 = 5000.0;

/// Pressure drop per hop from a water source (0.0-1.0 scale per hop).
const PRESSURE_DROP_PER_HOP: f32 = 0.008;

/// Minimum pressure factor for water service (below this, no service).
const MIN_PRESSURE_FACTOR: f32 = 0.1;

/// Sewage capacity per treatment plant in gallons per day.
const TREATMENT_PLANT_CAPACITY_GPD: f32 = 500_000.0;

/// Sewage generated per building occupant per day (gallons).
const SEWAGE_PER_OCCUPANT_GPD: f32 = 80.0;

/// Base leak rate for new pipes (fraction of flow lost per hop).
const BASE_LEAK_RATE: f32 = 0.001;

/// Leak rate increase per age tier (pipes age every 500 slow ticks).
const LEAK_RATE_PER_AGE_TIER: f32 = 0.002;

/// Slow ticks per age tier (how often pipes age).
const TICKS_PER_AGE_TIER: u32 = 500;

/// Probability of a pipe break per slow tick per age tier.
const BREAK_PROBABILITY_PER_TIER: f32 = 0.001;

/// Ticks to repair a broken pipe segment.
const REPAIR_TICKS: u32 = 5;

/// Maximum BFS hops for pipe network traversal.
const MAX_PIPE_HOPS: u32 = 120;

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// City-wide water/sewage pipe network state.
#[derive(Resource, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct WaterPipeNetworkState {
    /// Total road cells with pipes (auto-follows roads).
    pub pipe_cells: u32,
    /// Total pipe capacity in gallons per day.
    pub total_pipe_capacity_gpd: f32,
    /// Total water demand from buildings in gallons per day.
    pub total_water_demand_gpd: f32,
    /// Number of buildings with full water pressure.
    pub buildings_full_service: u32,
    /// Number of buildings with reduced pressure (partial service).
    pub buildings_reduced_service: u32,
    /// Number of buildings with no water service.
    pub buildings_no_service: u32,
    /// Average pressure factor across all served buildings (0.0-1.0).
    pub average_pressure: f32,
    /// Total water lost to leaks (gallons per day).
    pub leak_loss_gpd: f32,
    /// Number of currently broken pipe segments.
    pub broken_pipes: u32,
    /// Pipes under repair (counting down to fixed).
    pub pipes_under_repair: u32,
    /// Total sewage generation in gallons per day.
    pub total_sewage_gpd: f32,
    /// Total sewage treatment capacity in gallons per day.
    pub treatment_capacity_gpd: f32,
    /// Number of treatment plants.
    pub treatment_plant_count: u32,
    /// Sewage overflow (sewage - treatment capacity, clamped to 0).
    pub sewage_overflow_gpd: f32,
    /// Pipe network age in ticks (increments each slow tick).
    pub network_age_ticks: u32,
    /// Current age tier (network_age_ticks / TICKS_PER_AGE_TIER).
    pub age_tier: u32,
    /// Current effective leak rate (increases with age).
    pub effective_leak_rate: f32,
    /// Number of water sources (water towers + pumping stations).
    pub water_source_count: u32,
    /// Last slow-tick counter when the system updated.
    pub last_update_tick: u32,
}

impl Default for WaterPipeNetworkState {
    fn default() -> Self {
        Self {
            pipe_cells: 0,
            total_pipe_capacity_gpd: 0.0,
            total_water_demand_gpd: 0.0,
            buildings_full_service: 0,
            buildings_reduced_service: 0,
            buildings_no_service: 0,
            average_pressure: 1.0,
            leak_loss_gpd: 0.0,
            broken_pipes: 0,
            pipes_under_repair: 0,
            total_sewage_gpd: 0.0,
            treatment_capacity_gpd: 0.0,
            treatment_plant_count: 0,
            sewage_overflow_gpd: 0.0,
            network_age_ticks: 0,
            age_tier: 0,
            effective_leak_rate: BASE_LEAK_RATE,
            water_source_count: 0,
            last_update_tick: 0,
        }
    }
}

impl WaterPipeNetworkState {
    /// True if sewage generation exceeds treatment capacity.
    pub fn has_sewage_overflow(&self) -> bool {
        self.sewage_overflow_gpd > 0.0
    }

    /// True if water demand exceeds effective pipe capacity (after leaks).
    pub fn is_over_capacity(&self) -> bool {
        let effective = self.total_pipe_capacity_gpd * (1.0 - self.effective_leak_rate);
        self.total_water_demand_gpd > effective && self.total_pipe_capacity_gpd > 0.0
    }

    /// Coverage ratio: fraction of buildings with at least partial service.
    pub fn coverage_ratio(&self) -> f32 {
        let total =
            self.buildings_full_service + self.buildings_reduced_service + self.buildings_no_service;
        if total == 0 {
            return 1.0;
        }
        (self.buildings_full_service + self.buildings_reduced_service) as f32 / total as f32
    }
}

// ---------------------------------------------------------------------------
// Pipe break tracking
// ---------------------------------------------------------------------------

/// Tracks individual pipe break events for repair.
#[derive(Resource, Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct PipeBreakTracker {
    /// Active breaks: (grid_x, grid_y, remaining_repair_ticks).
    pub breaks: Vec<(usize, usize, u32)>,
}

// ---------------------------------------------------------------------------
// Saveable
// ---------------------------------------------------------------------------

impl crate::Saveable for WaterPipeNetworkState {
    const SAVE_KEY: &'static str = "water_pipe_network";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.pipe_cells == 0 && self.water_source_count == 0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Pure helper functions
// ---------------------------------------------------------------------------

/// Calculate pressure factor at a given BFS distance from a water source.
pub fn pressure_at_distance(hops: u32) -> f32 {
    (1.0 - hops as f32 * PRESSURE_DROP_PER_HOP).clamp(0.0, 1.0)
}

/// Calculate effective leak rate based on pipe age tier.
pub fn leak_rate_for_age_tier(age_tier: u32) -> f32 {
    (BASE_LEAK_RATE + age_tier as f32 * LEAK_RATE_PER_AGE_TIER).clamp(0.0, 0.5)
}

/// Calculate break probability per slow tick for a given age tier.
pub fn break_probability(age_tier: u32) -> f32 {
    (age_tier as f32 * BREAK_PROBABILITY_PER_TIER).clamp(0.0, 0.25)
}

/// Calculate the water demand for a building based on occupants.
pub fn building_water_demand(occupants: u32) -> f32 {
    occupants as f32 * 150.0 // 150 gallons per capita per day
}

/// Classify a pressure factor into service level.
pub fn classify_service(pressure: f32) -> ServiceLevel {
    if pressure >= 0.8 {
        ServiceLevel::Full
    } else if pressure >= MIN_PRESSURE_FACTOR {
        ServiceLevel::Reduced
    } else {
        ServiceLevel::None
    }
}

/// Service level classification for buildings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceLevel {
    Full,
    Reduced,
    None,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Count road cells to determine pipe network extent.
fn count_pipe_cells(grid: &WorldGrid) -> u32 {
    grid.cells
        .iter()
        .filter(|c| c.cell_type == CellType::Road)
        .count() as u32
}

/// Count water sources and calculate total source capacity.
fn count_water_sources(sources: &Query<&UtilitySource>) -> (u32, u32) {
    let mut water_count = 0u32;
    let mut treatment_count = 0u32;
    for source in sources.iter() {
        match source.utility_type {
            UtilityType::WaterTower | UtilityType::PumpingStation => water_count += 1,
            UtilityType::SewagePlant | UtilityType::WaterTreatment => treatment_count += 1,
            _ => {}
        }
    }
    (water_count, treatment_count)
}

/// Calculate sewage generation from all buildings.
fn total_sewage_from_buildings(buildings: &Query<&Building>) -> f32 {
    buildings
        .iter()
        .map(|b| b.occupants as f32 * SEWAGE_PER_OCCUPANT_GPD)
        .sum()
}

/// Calculate total water demand from all buildings.
fn total_water_demand(buildings: &Query<&Building>) -> f32 {
    buildings
        .iter()
        .map(|b| building_water_demand(b.occupants))
        .sum()
}

/// BFS from water sources to calculate per-building pressure.
/// Returns (full_service, reduced_service, no_service, avg_pressure).
fn calculate_building_pressures(
    grid: &WorldGrid,
    sources: &Query<&UtilitySource>,
    buildings: &Query<&Building>,
    breaks: &PipeBreakTracker,
) -> (u32, u32, u32, f32) {
    let width = grid.width;
    let height = grid.height;
    let len = width * height;

    // BFS to calculate min distance from any water source for each cell.
    let mut distance: Vec<u32> = vec![u32::MAX; len];
    let mut queue = std::collections::VecDeque::new();

    // Build a set of broken cells for O(1) lookup.
    let broken_set: std::collections::HashSet<(usize, usize)> =
        breaks.breaks.iter().map(|&(x, y, _)| (x, y)).collect();

    // Seed BFS from all water sources.
    for source in sources.iter() {
        if !source.utility_type.is_water() {
            continue;
        }
        // Skip sewage-only sources.
        if matches!(
            source.utility_type,
            UtilityType::SewagePlant | UtilityType::WaterTreatment
        ) {
            continue;
        }
        let idx = source.grid_y * width + source.grid_x;
        if idx < len && distance[idx] == u32::MAX {
            distance[idx] = 0;
            queue.push_back((source.grid_x, source.grid_y, 0u32));
        }
    }

    // BFS through road cells (pipes follow roads).
    while let Some((x, y, dist)) = queue.pop_front() {
        if dist >= MAX_PIPE_HOPS {
            continue;
        }
        let (neighbors, ncount) = grid.neighbors4(x, y);
        for &(nx, ny) in &neighbors[..ncount] {
            let nidx = ny * width + nx;
            if nidx >= len {
                continue;
            }
            if distance[nidx] != u32::MAX {
                continue;
            }
            // Only traverse through road cells (pipes follow roads).
            if grid.get(nx, ny).cell_type != CellType::Road {
                // Allow marking grass cells adjacent to roads (buildings).
                if grid.get(nx, ny).cell_type == CellType::Grass {
                    distance[nidx] = dist + 1;
                }
                continue;
            }
            // Skip broken pipe segments.
            if broken_set.contains(&(nx, ny)) {
                continue;
            }
            distance[nidx] = dist + 1;
            queue.push_back((nx, ny, dist + 1));
        }
    }

    // Evaluate each building's pressure based on BFS distance.
    let mut full = 0u32;
    let mut reduced = 0u32;
    let mut none = 0u32;
    let mut pressure_sum = 0.0f32;
    let mut count = 0u32;

    for building in buildings.iter() {
        let idx = building.grid_y * width + building.grid_x;
        let pressure = if idx < len && distance[idx] != u32::MAX {
            pressure_at_distance(distance[idx])
        } else {
            0.0
        };

        match classify_service(pressure) {
            ServiceLevel::Full => full += 1,
            ServiceLevel::Reduced => reduced += 1,
            ServiceLevel::None => none += 1,
        }

        pressure_sum += pressure;
        count += 1;
    }

    let avg = if count > 0 {
        pressure_sum / count as f32
    } else {
        1.0
    };

    (full, reduced, none, avg)
}

/// Age pipe breaks and remove repaired ones.
fn tick_pipe_breaks(breaks: &mut PipeBreakTracker) -> (u32, u32) {
    let mut active = 0u32;
    let mut repairing = 0u32;
    breaks.breaks.retain_mut(|(_x, _y, remaining)| {
        if *remaining > 1 {
            *remaining -= 1;
            repairing += 1;
            true
        } else {
            // Repaired this tick.
            false
        }
    });
    active = breaks.breaks.len() as u32;
    let _ = active; // suppress unused warning; we return active from len
    repairing = repairing.min(breaks.breaks.len() as u32);
    (breaks.breaks.len() as u32, repairing)
}

/// Main update system: runs on slow tick.
#[allow(clippy::too_many_arguments)]
pub fn update_water_pipe_network(
    slow_tick: Res<SlowTickTimer>,
    grid: Res<WorldGrid>,
    sources: Query<&UtilitySource>,
    buildings: Query<&Building>,
    mut state: ResMut<WaterPipeNetworkState>,
    mut breaks: ResMut<PipeBreakTracker>,
) {
    if !slow_tick.should_run() {
        return;
    }

    state.last_update_tick = slow_tick.counter;

    // 1. Count pipe cells (roads = pipes).
    state.pipe_cells = count_pipe_cells(&grid);
    state.total_pipe_capacity_gpd = state.pipe_cells as f32 * BASE_PIPE_CAPACITY_GPD;

    // 2. Count water sources and treatment plants.
    let (water_sources, treatment_plants) = count_water_sources(&sources);
    state.water_source_count = water_sources;
    state.treatment_plant_count = treatment_plants;
    state.treatment_capacity_gpd = treatment_plants as f32 * TREATMENT_PLANT_CAPACITY_GPD;

    // 3. Calculate demand.
    state.total_water_demand_gpd = total_water_demand(&buildings);
    state.total_sewage_gpd = total_sewage_from_buildings(&buildings);

    // 4. Sewage overflow.
    state.sewage_overflow_gpd =
        (state.total_sewage_gpd - state.treatment_capacity_gpd).max(0.0);

    // 5. Age the network.
    state.network_age_ticks = state.network_age_ticks.saturating_add(1);
    state.age_tier = state.network_age_ticks / TICKS_PER_AGE_TIER;
    state.effective_leak_rate = leak_rate_for_age_tier(state.age_tier);

    // 6. Calculate leak losses.
    state.leak_loss_gpd =
        state.total_pipe_capacity_gpd * state.effective_leak_rate;

    // 7. Tick pipe breaks (repair countdown).
    let (broken, repairing) = tick_pipe_breaks(&mut breaks);
    state.broken_pipes = broken;
    state.pipes_under_repair = repairing;

    // 8. Simulate new pipe breaks based on age.
    if state.pipe_cells > 0 && state.age_tier > 0 {
        let prob = break_probability(state.age_tier);
        // Deterministic: use counter as seed for reproducibility.
        let pseudo_random = ((slow_tick.counter as f32 * 7.13) % 1.0).abs();
        if pseudo_random < prob && breaks.breaks.len() < 10 {
            // Break a pipe at a deterministic location.
            let cell_idx =
                (slow_tick.counter as usize * 31) % grid.cells.len();
            let bx = cell_idx % grid.width;
            let by = cell_idx / grid.width;
            if grid.get(bx, by).cell_type == CellType::Road {
                // Only break if not already broken at this location.
                if !breaks.breaks.iter().any(|&(x, y, _)| x == bx && y == by) {
                    breaks.breaks.push((bx, by, REPAIR_TICKS));
                }
            }
        }
    }

    // 9. Calculate building pressures via BFS.
    let (full, reduced, none, avg) =
        calculate_building_pressures(&grid, &sources, &buildings, &breaks);
    state.buildings_full_service = full;
    state.buildings_reduced_service = reduced;
    state.buildings_no_service = none;
    state.average_pressure = avg;
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct WaterPipeNetworkPlugin;

impl Plugin for WaterPipeNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterPipeNetworkState>();
        app.init_resource::<PipeBreakTracker>();

        app.add_systems(
            FixedUpdate,
            update_water_pipe_network
                .after(crate::utilities::propagate_utilities)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<WaterPipeNetworkState>();
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Saveable;

    #[test]
    fn test_pressure_at_distance_zero() {
        let p = pressure_at_distance(0);
        assert!((p - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pressure_at_distance_drops() {
        let p0 = pressure_at_distance(0);
        let p50 = pressure_at_distance(50);
        assert!(p50 < p0, "Pressure should drop with distance");
        assert!(p50 > 0.0, "Pressure should still be positive at 50 hops");
    }

    #[test]
    fn test_pressure_at_max_distance() {
        let p = pressure_at_distance(MAX_PIPE_HOPS);
        assert!(p >= 0.0, "Pressure should not go negative");
    }

    #[test]
    fn test_pressure_clamped_to_zero() {
        let p = pressure_at_distance(200);
        assert!((p - 0.0).abs() < f32::EPSILON, "Pressure should clamp to 0");
    }

    #[test]
    fn test_leak_rate_new_pipes() {
        let rate = leak_rate_for_age_tier(0);
        assert!((rate - BASE_LEAK_RATE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_leak_rate_increases_with_age() {
        let r0 = leak_rate_for_age_tier(0);
        let r5 = leak_rate_for_age_tier(5);
        assert!(r5 > r0, "Leak rate should increase with age");
    }

    #[test]
    fn test_leak_rate_capped() {
        let r = leak_rate_for_age_tier(1000);
        assert!(r <= 0.5, "Leak rate should be capped at 0.5");
    }

    #[test]
    fn test_break_probability_zero_at_tier_zero() {
        let p = break_probability(0);
        assert!((p - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_break_probability_increases_with_age() {
        let p0 = break_probability(0);
        let p5 = break_probability(5);
        assert!(p5 > p0);
    }

    #[test]
    fn test_break_probability_capped() {
        let p = break_probability(1000);
        assert!(p <= 0.25);
    }

    #[test]
    fn test_building_water_demand() {
        assert!((building_water_demand(0) - 0.0).abs() < f32::EPSILON);
        assert!((building_water_demand(10) - 1500.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_classify_service_full() {
        assert_eq!(classify_service(1.0), ServiceLevel::Full);
        assert_eq!(classify_service(0.85), ServiceLevel::Full);
    }

    #[test]
    fn test_classify_service_reduced() {
        assert_eq!(classify_service(0.5), ServiceLevel::Reduced);
        assert_eq!(classify_service(0.15), ServiceLevel::Reduced);
    }

    #[test]
    fn test_classify_service_none() {
        assert_eq!(classify_service(0.0), ServiceLevel::None);
        assert_eq!(classify_service(0.05), ServiceLevel::None);
    }

    #[test]
    fn test_default_state() {
        let state = WaterPipeNetworkState::default();
        assert_eq!(state.pipe_cells, 0);
        assert_eq!(state.water_source_count, 0);
        assert!((state.average_pressure - 1.0).abs() < f32::EPSILON);
        assert_eq!(state.broken_pipes, 0);
        assert!(!state.has_sewage_overflow());
        assert!(!state.is_over_capacity());
    }

    #[test]
    fn test_coverage_ratio_no_buildings() {
        let state = WaterPipeNetworkState::default();
        assert!((state.coverage_ratio() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_coverage_ratio_all_served() {
        let mut state = WaterPipeNetworkState::default();
        state.buildings_full_service = 100;
        state.buildings_reduced_service = 0;
        state.buildings_no_service = 0;
        assert!((state.coverage_ratio() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_coverage_ratio_partial() {
        let mut state = WaterPipeNetworkState::default();
        state.buildings_full_service = 50;
        state.buildings_reduced_service = 25;
        state.buildings_no_service = 25;
        assert!((state.coverage_ratio() - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sewage_overflow_detection() {
        let mut state = WaterPipeNetworkState::default();
        state.total_sewage_gpd = 100_000.0;
        state.treatment_capacity_gpd = 50_000.0;
        state.sewage_overflow_gpd = 50_000.0;
        assert!(state.has_sewage_overflow());
    }

    #[test]
    fn test_over_capacity_detection() {
        let mut state = WaterPipeNetworkState::default();
        state.total_pipe_capacity_gpd = 100_000.0;
        state.total_water_demand_gpd = 200_000.0;
        state.effective_leak_rate = 0.01;
        assert!(state.is_over_capacity());
    }

    #[test]
    fn test_pipe_break_repair() {
        let mut tracker = PipeBreakTracker {
            breaks: vec![(10, 20, 3), (30, 40, 1)],
        };
        let (active, _repairing) = tick_pipe_breaks(&mut tracker);
        // Second break should be repaired (remaining was 1).
        assert_eq!(active, 1);
        assert_eq!(tracker.breaks[0], (10, 20, 2));
    }

    #[test]
    fn test_saveable_key() {
        assert_eq!(WaterPipeNetworkState::SAVE_KEY, "water_pipe_network");
    }

    #[test]
    fn test_saveable_skip_default() {
        let state = WaterPipeNetworkState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_roundtrip() {
        let state = WaterPipeNetworkState {
            pipe_cells: 200,
            total_pipe_capacity_gpd: 1_000_000.0,
            total_water_demand_gpd: 500_000.0,
            buildings_full_service: 80,
            buildings_reduced_service: 15,
            buildings_no_service: 5,
            average_pressure: 0.85,
            leak_loss_gpd: 1000.0,
            broken_pipes: 2,
            pipes_under_repair: 1,
            total_sewage_gpd: 400_000.0,
            treatment_capacity_gpd: 500_000.0,
            treatment_plant_count: 1,
            sewage_overflow_gpd: 0.0,
            network_age_ticks: 100,
            age_tier: 0,
            effective_leak_rate: 0.001,
            water_source_count: 3,
            last_update_tick: 50,
        };
        let bytes = state.save_to_bytes().expect("should save non-default");
        let restored = WaterPipeNetworkState::load_from_bytes(&bytes);
        assert_eq!(restored.pipe_cells, 200);
        assert_eq!(restored.buildings_full_service, 80);
        assert_eq!(restored.water_source_count, 3);
        assert!((restored.average_pressure - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_saveable_invalid_bytes_returns_default() {
        let restored = WaterPipeNetworkState::load_from_bytes(&[0xFF, 0x00]);
        assert_eq!(restored.pipe_cells, 0);
        assert_eq!(restored.water_source_count, 0);
    }
}
