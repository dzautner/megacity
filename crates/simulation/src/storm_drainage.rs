//! Storm drain and retention pond infrastructure (WATER-006).
//!
//! Storm drains follow road placement and remove runoff capacity (0.5 in/hr each).
//! Retention ponds are 4x4 buildings that store 500,000 gallons, slowly releasing stored water.
//! Rain gardens are 1x1 buildings that absorb 100% of local cell runoff and 50% from 4 neighbors.
//! The system tracks drainage network capacity vs. runoff, triggering flooding when exceeded.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::stormwater::StormwaterGrid;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Runoff capacity removed per storm drain, in inches/hr.
const DRAIN_CAPACITY_PER_DRAIN: f32 = 0.5;

/// Maximum gallons a single retention pond can store.
const RETENTION_POND_CAPACITY: f32 = 500_000.0;

/// Fraction of stored retention water released per slow tick.
/// Retention ponds slowly release stored water between storms.
const RETENTION_RELEASE_RATE: f32 = 0.05;

/// Conversion factor from stormwater grid runoff units to inches/hr equivalent.
/// The stormwater grid stores runoff as `rainfall_intensity * imperviousness * CELL_AREA`.
/// We normalise to inches/hr for comparison with drain capacity.
const RUNOFF_TO_INCHES_PER_HR: f32 = 0.01;

/// Conversion factor from stormwater grid runoff units to gallons for retention storage.
const RUNOFF_TO_GALLONS: f32 = 100.0;

/// Fraction of a rain garden's 4 cardinal neighbors' runoff that it absorbs.
const RAIN_GARDEN_NEIGHBOR_ABSORB: f32 = 0.50;

/// Runoff threshold (inches/hr equivalent) above which a cell is considered flooding
/// when drainage capacity is exceeded.
const FLOOD_THRESHOLD: f32 = 0.1;

// =============================================================================
// Infrastructure type enum
// =============================================================================

/// The kind of storm drainage infrastructure placed in the city.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StormDrainageType {
    /// Storm drain placed along roads. Removes 0.5 in/hr capacity.
    StormDrain,
    /// 4x4 retention pond. Stores up to 500,000 gallons, slowly releases.
    RetentionPond,
    /// 1x1 rain garden. Absorbs 100% of local cell runoff + 50% from 4 neighbors.
    RainGarden,
}

// =============================================================================
// Storm drainage infrastructure component
// =============================================================================

/// Component attached to entities representing storm drainage infrastructure.
/// Used to query all drains, ponds, and gardens in the ECS world.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct StormDrainageInfrastructure {
    /// What kind of drainage infrastructure this is.
    pub drainage_type: StormDrainageType,
    /// Grid X position of this infrastructure.
    pub grid_x: usize,
    /// Grid Y position of this infrastructure.
    pub grid_y: usize,
}

// =============================================================================
// Storm drainage state resource
// =============================================================================

/// City-wide storm drainage state, tracking infrastructure counts, capacity, and overflow.
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct StormDrainageState {
    /// Total drain capacity in inches/hr removed by all storm drains.
    pub total_drain_capacity: f32,
    /// Total retention pond storage capacity in gallons.
    pub total_retention_capacity: f32,
    /// Gallons of stormwater currently stored in retention ponds.
    pub current_retention_stored: f32,
    /// Number of storm drains placed in the city.
    pub drain_count: u32,
    /// Number of retention ponds placed in the city.
    pub retention_pond_count: u32,
    /// Number of rain gardens placed in the city.
    pub rain_garden_count: u32,
    /// Number of cells where runoff exceeds drainage capacity (flooding).
    pub overflow_cells: u32,
    /// Fraction of road cells that have at least one drain (0.0..=1.0).
    pub drainage_coverage: f32,
}

// =============================================================================
// System
// =============================================================================

/// Updates storm drainage infrastructure state each slow tick.
///
/// 1. Counts storm drains, retention ponds, and rain gardens from infrastructure queries.
/// 2. Computes total drain capacity and retention capacity.
/// 3. Reads the StormwaterGrid to determine per-cell runoff.
/// 4. Rain gardens absorb local + neighbor runoff.
/// 5. Fills retention ponds with excess runoff beyond drain capacity.
/// 6. Slowly releases retention pond stored water.
/// 7. Tracks overflow cells where runoff exceeds all drainage.
/// 8. Computes drainage coverage as fraction of road cells with nearby drains.
pub fn update_storm_drainage(
    slow_timer: Res<SlowTickTimer>,
    mut drainage_state: ResMut<StormDrainageState>,
    stormwater: Res<StormwaterGrid>,
    grid: Res<WorldGrid>,
    infrastructure: Query<&StormDrainageInfrastructure>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Count infrastructure and compute capacities ---
    let mut drain_count: u32 = 0;
    let mut retention_pond_count: u32 = 0;
    let mut rain_garden_count: u32 = 0;

    // Track drain positions for coverage calculation
    let mut drain_positions: Vec<(usize, usize)> = Vec::new();
    // Track rain garden positions for runoff absorption
    let mut rain_garden_positions: Vec<(usize, usize)> = Vec::new();

    for infra in &infrastructure {
        match infra.drainage_type {
            StormDrainageType::StormDrain => {
                drain_count += 1;
                drain_positions.push((infra.grid_x, infra.grid_y));
            }
            StormDrainageType::RetentionPond => {
                retention_pond_count += 1;
            }
            StormDrainageType::RainGarden => {
                rain_garden_count += 1;
                rain_garden_positions.push((infra.grid_x, infra.grid_y));
            }
        }
    }

    let total_drain_capacity = drain_count as f32 * DRAIN_CAPACITY_PER_DRAIN;
    let total_retention_capacity = retention_pond_count as f32 * RETENTION_POND_CAPACITY;

    // --- Phase 2: Compute effective runoff per cell (after rain garden absorption) ---
    let total_cells = GRID_WIDTH * GRID_HEIGHT;
    let mut effective_runoff = vec![0.0_f32; total_cells];

    // Copy raw runoff from the stormwater grid
    for i in 0..total_cells {
        effective_runoff[i] = stormwater.runoff[i];
    }

    // Rain gardens absorb 100% of their own cell + 50% from 4 cardinal neighbors
    for &(gx, gy) in &rain_garden_positions {
        if gx < GRID_WIDTH && gy < GRID_HEIGHT {
            // Absorb 100% of local cell runoff
            let idx = gy * GRID_WIDTH + gx;
            effective_runoff[idx] = 0.0;

            // Absorb 50% from each cardinal neighbor
            let (neighbors, count) = grid.neighbors4(gx, gy);
            for &(nx, ny) in &neighbors[..count] {
                let nidx = ny * GRID_WIDTH + nx;
                effective_runoff[nidx] *= 1.0 - RAIN_GARDEN_NEIGHBOR_ABSORB;
            }
        }
    }

    // --- Phase 3: Compute total effective runoff in inches/hr ---
    let mut total_effective_runoff_in_hr = 0.0_f32;
    for val in &effective_runoff {
        total_effective_runoff_in_hr += val * RUNOFF_TO_INCHES_PER_HR;
    }

    // --- Phase 4: Determine excess runoff beyond drain capacity ---
    let excess_runoff_in_hr = (total_effective_runoff_in_hr - total_drain_capacity).max(0.0);

    // Convert excess to gallons for retention storage
    let excess_gallons = excess_runoff_in_hr * RUNOFF_TO_GALLONS;

    // --- Phase 5: Fill retention ponds ---
    let mut current_stored = drainage_state.current_retention_stored;
    let available_storage = (total_retention_capacity - current_stored).max(0.0);
    let stored_this_tick = excess_gallons.min(available_storage);
    current_stored += stored_this_tick;

    // --- Phase 6: Slowly release stored water ---
    let released = current_stored * RETENTION_RELEASE_RATE;
    current_stored = (current_stored - released).max(0.0);

    // --- Phase 7: Count overflow cells ---
    // A cell overflows when its effective runoff exceeds the per-cell drain
    // capacity share AND retention ponds are full (cannot absorb more).
    let per_cell_drain_capacity = if drain_count > 0 {
        total_drain_capacity / total_cells as f32
    } else {
        0.0
    };

    let retention_full = current_stored >= total_retention_capacity * 0.99;

    let mut overflow_cells: u32 = 0;
    for val in &effective_runoff {
        let cell_runoff_in_hr = val * RUNOFF_TO_INCHES_PER_HR;
        if cell_runoff_in_hr > per_cell_drain_capacity + FLOOD_THRESHOLD && retention_full {
            overflow_cells += 1;
        }
    }

    // --- Phase 8: Compute drainage coverage ---
    // Fraction of road cells that have a storm drain on them or adjacent (Manhattan dist <= 1)
    let mut road_cell_count: u32 = 0;
    let mut covered_road_cells: u32 = 0;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type == CellType::Road {
                road_cell_count += 1;

                // Check if any drain is at this cell or adjacent
                let is_covered = drain_positions.iter().any(|&(dx, dy)| {
                    let dist_x = (x as i32 - dx as i32).unsigned_abs() as usize;
                    let dist_y = (y as i32 - dy as i32).unsigned_abs() as usize;
                    dist_x + dist_y <= 1
                });

                if is_covered {
                    covered_road_cells += 1;
                }
            }
        }
    }

    let drainage_coverage = if road_cell_count > 0 {
        covered_road_cells as f32 / road_cell_count as f32
    } else {
        0.0
    };

    // --- Update state ---
    drainage_state.drain_count = drain_count;
    drainage_state.retention_pond_count = retention_pond_count;
    drainage_state.rain_garden_count = rain_garden_count;
    drainage_state.total_drain_capacity = total_drain_capacity;
    drainage_state.total_retention_capacity = total_retention_capacity;
    drainage_state.current_retention_stored = current_stored;
    drainage_state.overflow_cells = overflow_cells;
    drainage_state.drainage_coverage = drainage_coverage;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // StormDrainageType tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_storm_drainage_type_equality() {
        assert_eq!(StormDrainageType::StormDrain, StormDrainageType::StormDrain);
        assert_ne!(
            StormDrainageType::StormDrain,
            StormDrainageType::RetentionPond
        );
        assert_ne!(
            StormDrainageType::RetentionPond,
            StormDrainageType::RainGarden
        );
    }

    #[test]
    fn test_storm_drainage_type_clone() {
        let t = StormDrainageType::RetentionPond;
        let t2 = t;
        assert_eq!(t, t2);
    }

    // -------------------------------------------------------------------------
    // StormDrainageState default tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_default() {
        let state = StormDrainageState::default();
        assert_eq!(state.total_drain_capacity, 0.0);
        assert_eq!(state.total_retention_capacity, 0.0);
        assert_eq!(state.current_retention_stored, 0.0);
        assert_eq!(state.drain_count, 0);
        assert_eq!(state.retention_pond_count, 0);
        assert_eq!(state.rain_garden_count, 0);
        assert_eq!(state.overflow_cells, 0);
        assert_eq!(state.drainage_coverage, 0.0);
    }

    #[test]
    fn test_state_clone() {
        let mut state = StormDrainageState::default();
        state.drain_count = 5;
        state.total_drain_capacity = 2.5;
        state.current_retention_stored = 1000.0;
        let cloned = state.clone();
        assert_eq!(cloned.drain_count, 5);
        assert_eq!(cloned.total_drain_capacity, 2.5);
        assert_eq!(cloned.current_retention_stored, 1000.0);
    }

    // -------------------------------------------------------------------------
    // StormDrainageInfrastructure component tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_infrastructure_component_fields() {
        let infra = StormDrainageInfrastructure {
            drainage_type: StormDrainageType::StormDrain,
            grid_x: 10,
            grid_y: 20,
        };
        assert_eq!(infra.drainage_type, StormDrainageType::StormDrain);
        assert_eq!(infra.grid_x, 10);
        assert_eq!(infra.grid_y, 20);
    }

    #[test]
    fn test_infrastructure_component_clone() {
        let infra = StormDrainageInfrastructure {
            drainage_type: StormDrainageType::RainGarden,
            grid_x: 5,
            grid_y: 15,
        };
        let cloned = infra.clone();
        assert_eq!(cloned.drainage_type, StormDrainageType::RainGarden);
        assert_eq!(cloned.grid_x, 5);
        assert_eq!(cloned.grid_y, 15);
    }

    // -------------------------------------------------------------------------
    // Drain capacity calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_drain_capacity_per_drain() {
        assert!((DRAIN_CAPACITY_PER_DRAIN - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_total_drain_capacity_scales_linearly() {
        let counts = [0u32, 1, 5, 10, 100];
        for count in counts {
            let capacity = count as f32 * DRAIN_CAPACITY_PER_DRAIN;
            let expected = count as f32 * 0.5;
            assert!(
                (capacity - expected).abs() < f32::EPSILON,
                "Drain capacity for {} drains should be {}, got {}",
                count,
                expected,
                capacity
            );
        }
    }

    // -------------------------------------------------------------------------
    // Retention pond capacity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_retention_pond_capacity() {
        assert!((RETENTION_POND_CAPACITY - 500_000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_total_retention_capacity_scales() {
        let pond_count = 3u32;
        let total = pond_count as f32 * RETENTION_POND_CAPACITY;
        assert!((total - 1_500_000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_retention_fill_clamps_to_capacity() {
        let capacity = RETENTION_POND_CAPACITY;
        let current_stored = 400_000.0_f32;
        let excess_gallons = 200_000.0_f32;
        let available = (capacity - current_stored).max(0.0);
        let stored = excess_gallons.min(available);
        // Available is 100,000, so we can only store 100,000 of the 200,000 excess
        assert!((stored - 100_000.0).abs() < f32::EPSILON);
        assert!((current_stored + stored - 500_000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_retention_fill_when_empty() {
        let capacity = RETENTION_POND_CAPACITY;
        let current_stored = 0.0_f32;
        let excess_gallons = 300_000.0_f32;
        let available = (capacity - current_stored).max(0.0);
        let stored = excess_gallons.min(available);
        // All 300,000 can fit
        assert!((stored - 300_000.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Retention release rate tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_retention_release_rate() {
        let stored = 100_000.0_f32;
        let released = stored * RETENTION_RELEASE_RATE;
        assert!((released - 5_000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_retention_release_reduces_stored() {
        let mut stored = 100_000.0_f32;
        let released = stored * RETENTION_RELEASE_RATE;
        stored = (stored - released).max(0.0);
        assert!((stored - 95_000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_retention_release_zero_when_empty() {
        let stored = 0.0_f32;
        let released = stored * RETENTION_RELEASE_RATE;
        assert!(released.abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Rain garden absorption tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_rain_garden_absorbs_local_cell_completely() {
        let local_runoff = 5.0_f32;
        let absorbed = local_runoff; // 100% absorption
        let remaining = local_runoff - absorbed;
        assert!(remaining.abs() < f32::EPSILON);
    }

    #[test]
    fn test_rain_garden_absorbs_neighbor_50_percent() {
        let neighbor_runoff = 4.0_f32;
        let remaining = neighbor_runoff * (1.0 - RAIN_GARDEN_NEIGHBOR_ABSORB);
        assert!((remaining - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rain_garden_total_absorption_with_neighbors() {
        // Rain garden at center with 4 neighbors each having 4.0 runoff
        let local_runoff = 3.0_f32;
        let neighbor_runoff = 4.0_f32;
        let neighbor_count = 4;

        let total_absorbed =
            local_runoff + (neighbor_runoff * RAIN_GARDEN_NEIGHBOR_ABSORB * neighbor_count as f32);
        // 3.0 + (4.0 * 0.5 * 4) = 3.0 + 8.0 = 11.0
        assert!((total_absorbed - 11.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Overflow / flooding tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_overflow_when_no_drains() {
        // With no drains and some runoff, per_cell_drain_capacity is 0
        let per_cell_drain_capacity = 0.0_f32;
        let cell_runoff_in_hr = 0.5_f32;
        let retention_full = true;
        let floods =
            cell_runoff_in_hr > per_cell_drain_capacity + FLOOD_THRESHOLD && retention_full;
        assert!(
            floods,
            "Cell should flood with no drains and runoff above threshold"
        );
    }

    #[test]
    fn test_no_overflow_when_drains_sufficient() {
        // Enough drain capacity should prevent overflow
        let drain_count = 100u32;
        let total_cells = GRID_WIDTH * GRID_HEIGHT;
        let total_capacity = drain_count as f32 * DRAIN_CAPACITY_PER_DRAIN;
        let per_cell = total_capacity / total_cells as f32;
        // With a very small per-cell runoff that's below threshold + per_cell capacity
        let cell_runoff_in_hr = per_cell * 0.5; // well below threshold
        let retention_full = true;
        let floods = cell_runoff_in_hr > per_cell + FLOOD_THRESHOLD && retention_full;
        assert!(!floods, "Cell should not flood when drains are sufficient");
    }

    #[test]
    fn test_no_overflow_when_retention_not_full() {
        // Even with excess runoff, if retention ponds have space, no overflow
        let per_cell_drain_capacity = 0.0_f32;
        let cell_runoff_in_hr = 1.0_f32;
        let retention_full = false; // ponds still have space
        let floods =
            cell_runoff_in_hr > per_cell_drain_capacity + FLOOD_THRESHOLD && retention_full;
        assert!(
            !floods,
            "Cell should not flood when retention ponds have space"
        );
    }

    // -------------------------------------------------------------------------
    // Drainage coverage tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_drainage_coverage_no_roads() {
        let road_cell_count = 0u32;
        let covered = 0u32;
        let coverage = if road_cell_count > 0 {
            covered as f32 / road_cell_count as f32
        } else {
            0.0
        };
        assert!(coverage.abs() < f32::EPSILON);
    }

    #[test]
    fn test_drainage_coverage_all_covered() {
        let road_cell_count = 50u32;
        let covered = 50u32;
        let coverage = covered as f32 / road_cell_count as f32;
        assert!((coverage - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_drainage_coverage_partial() {
        let road_cell_count = 100u32;
        let covered = 25u32;
        let coverage = covered as f32 / road_cell_count as f32;
        assert!((coverage - 0.25).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Drain adjacency tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_drain_adjacency_manhattan_distance() {
        let drain_x: usize = 10;
        let drain_y: usize = 10;

        // Same cell: distance 0 <= 1, covered
        let dist = (10i32 - drain_x as i32).unsigned_abs() as usize
            + (10i32 - drain_y as i32).unsigned_abs() as usize;
        assert!(dist <= 1);

        // Adjacent cell: distance 1 <= 1, covered
        let dist = (11i32 - drain_x as i32).unsigned_abs() as usize
            + (10i32 - drain_y as i32).unsigned_abs() as usize;
        assert!(dist <= 1);

        // Diagonal cell: distance 2 > 1, NOT covered
        let dist = (11i32 - drain_x as i32).unsigned_abs() as usize
            + (11i32 - drain_y as i32).unsigned_abs() as usize;
        assert!(dist > 1);
    }

    // -------------------------------------------------------------------------
    // Excess runoff calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_excess_runoff_with_sufficient_capacity() {
        let total_runoff_in_hr = 10.0_f32;
        let total_drain_capacity = 15.0_f32;
        let excess = (total_runoff_in_hr - total_drain_capacity).max(0.0);
        assert!(
            excess.abs() < f32::EPSILON,
            "No excess when capacity exceeds runoff"
        );
    }

    #[test]
    fn test_excess_runoff_with_insufficient_capacity() {
        let total_runoff_in_hr = 20.0_f32;
        let total_drain_capacity = 15.0_f32;
        let excess = (total_runoff_in_hr - total_drain_capacity).max(0.0);
        assert!((excess - 5.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Serde round-trip test
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_serde_roundtrip() {
        let mut state = StormDrainageState::default();
        state.drain_count = 42;
        state.retention_pond_count = 3;
        state.rain_garden_count = 7;
        state.total_drain_capacity = 21.0;
        state.total_retention_capacity = 1_500_000.0;
        state.current_retention_stored = 250_000.0;
        state.overflow_cells = 10;
        state.drainage_coverage = 0.75;

        let json = serde_json::to_string(&state).expect("serialize");
        let deserialized: StormDrainageState = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.drain_count, 42);
        assert_eq!(deserialized.retention_pond_count, 3);
        assert_eq!(deserialized.rain_garden_count, 7);
        assert!((deserialized.total_drain_capacity - 21.0).abs() < f32::EPSILON);
        assert!((deserialized.total_retention_capacity - 1_500_000.0).abs() < f32::EPSILON);
        assert!((deserialized.current_retention_stored - 250_000.0).abs() < f32::EPSILON);
        assert_eq!(deserialized.overflow_cells, 10);
        assert!((deserialized.drainage_coverage - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_infrastructure_serde_roundtrip() {
        let infra = StormDrainageInfrastructure {
            drainage_type: StormDrainageType::RetentionPond,
            grid_x: 42,
            grid_y: 99,
        };
        let json = serde_json::to_string(&infra).expect("serialize");
        let deserialized: StormDrainageInfrastructure =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.drainage_type, StormDrainageType::RetentionPond);
        assert_eq!(deserialized.grid_x, 42);
        assert_eq!(deserialized.grid_y, 99);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_full_drainage_cycle_no_infrastructure() {
        // Simulate a cycle with no infrastructure: all runoff becomes potential overflow
        let total_runoff_in_hr = 50.0_f32;
        let drain_capacity = 0.0_f32;
        let retention_capacity = 0.0_f32;
        let mut stored = 0.0_f32;

        let excess_in_hr = (total_runoff_in_hr - drain_capacity).max(0.0);
        let excess_gallons = excess_in_hr * RUNOFF_TO_GALLONS;
        let available = (retention_capacity - stored).max(0.0);
        let stored_tick = excess_gallons.min(available);
        stored += stored_tick;

        // Nothing stored because capacity is 0
        assert!(stored.abs() < f32::EPSILON);
        // All runoff is excess
        assert!((excess_in_hr - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_full_drainage_cycle_with_infrastructure() {
        // 10 drains (5.0 in/hr capacity), 2 retention ponds (1,000,000 gal)
        let drain_count = 10u32;
        let pond_count = 2u32;
        let drain_capacity = drain_count as f32 * DRAIN_CAPACITY_PER_DRAIN; // 5.0
        let retention_capacity = pond_count as f32 * RETENTION_POND_CAPACITY; // 1,000,000
        let mut stored = 200_000.0_f32;

        let total_runoff_in_hr = 8.0_f32;
        let excess_in_hr = (total_runoff_in_hr - drain_capacity).max(0.0); // 3.0
        let excess_gallons = excess_in_hr * RUNOFF_TO_GALLONS; // 300.0

        let available = (retention_capacity - stored).max(0.0); // 800,000
        let stored_tick = excess_gallons.min(available); // 300.0
        stored += stored_tick; // 200,300

        // Release
        let released = stored * RETENTION_RELEASE_RATE;
        stored = (stored - released).max(0.0);

        assert!((drain_capacity - 5.0).abs() < f32::EPSILON);
        assert!((excess_in_hr - 3.0).abs() < f32::EPSILON);
        assert!(stored > 0.0);
        assert!(stored < retention_capacity);
    }

    #[test]
    fn test_multiple_rain_gardens_reduce_runoff() {
        // Simulate effective runoff reduction from multiple rain gardens
        let grid_size = 5; // small test grid
        let mut runoff = vec![4.0_f32; grid_size * grid_size];

        // Place rain garden at (2, 2) in a 5x5 grid
        let gx = 2usize;
        let gy = 2usize;

        // Absorb 100% of local cell
        runoff[gy * grid_size + gx] = 0.0;

        // Absorb 50% from 4 cardinal neighbors
        let neighbors = [(1, 2), (3, 2), (2, 1), (2, 3)];
        for &(nx, ny) in &neighbors {
            let idx = ny * grid_size + nx;
            runoff[idx] *= 1.0 - RAIN_GARDEN_NEIGHBOR_ABSORB;
        }

        // Local cell is 0
        assert!(runoff[gy * grid_size + gx].abs() < f32::EPSILON);
        // Each neighbor is reduced to 2.0
        for &(nx, ny) in &neighbors {
            assert!((runoff[ny * grid_size + nx] - 2.0).abs() < f32::EPSILON);
        }
        // Non-adjacent cells remain 4.0
        assert!((runoff[0] - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_constants_are_positive() {
        assert!(DRAIN_CAPACITY_PER_DRAIN > 0.0);
        assert!(RETENTION_POND_CAPACITY > 0.0);
        assert!(RETENTION_RELEASE_RATE > 0.0);
        assert!(RETENTION_RELEASE_RATE < 1.0);
        assert!(RUNOFF_TO_INCHES_PER_HR > 0.0);
        assert!(RUNOFF_TO_GALLONS > 0.0);
        assert!(RAIN_GARDEN_NEIGHBOR_ABSORB > 0.0);
        assert!(RAIN_GARDEN_NEIGHBOR_ABSORB <= 1.0);
        assert!(FLOOD_THRESHOLD > 0.0);
    }
}
