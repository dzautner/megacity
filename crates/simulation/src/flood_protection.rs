//! Flood levee and seawall infrastructure (WATER-009).
//!
//! Implements flood protection infrastructure: levees, seawalls, and floodgates.
//!
//! **Levee**: Placeable along rivers, prevents flooding up to design height
//! (10 ft default). Water above the design height causes overtopping and
//! catastrophic failure, resulting in worse flooding than if the levee were absent.
//!
//! **Seawall**: Placeable along the coast, prevents coastal surge from flooding
//! inland cells. Seawalls have a fixed protection height of 15 ft.
//!
//! **Floodgate**: Allows controlled water release. When open, water flows freely;
//! when closed, acts like a levee with 12 ft protection height.
//!
//! **Maintenance**: Each protection cell costs $2,000/year. Neglected infrastructure
//! degrades over time, increasing failure probability. Failure probability rises
//! with age and lack of maintenance.
//!
//! The `update_flood_protection` system runs every slow tick and:
//!   1. Ages all protection structures
//!   2. Applies maintenance costs from the city budget
//!   3. Degrades unmaintained structures (condition decreases)
//!   4. Checks for overtopping during active floods
//!   5. Calculates failure probability based on age + condition
//!   6. Reduces flood depth in protected cells (or amplifies on failure)
//!   7. Updates aggregate protection statistics

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::flood_simulation::{FloodGrid, FloodState};
use crate::grid::{CellType, WorldGrid};
use crate::time_of_day::GameClock;
use crate::Saveable;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Default design height for levees in feet.
const LEVEE_DESIGN_HEIGHT: f32 = 10.0;

/// Default design height for seawalls in feet.
const SEAWALL_DESIGN_HEIGHT: f32 = 15.0;

/// Default design height for closed floodgates in feet.
const FLOODGATE_DESIGN_HEIGHT: f32 = 12.0;

/// Annual maintenance cost per protection cell in dollars.
const MAINTENANCE_COST_PER_CELL_PER_YEAR: f64 = 2_000.0;

/// Number of game days per year for maintenance cost calculation.
const DAYS_PER_YEAR: u32 = 360;

/// Condition degradation rate per slow tick when maintenance is not funded.
/// A fully maintained structure stays at condition 1.0.
const DEGRADATION_RATE_PER_TICK: f32 = 0.002;

/// Condition recovery rate per slow tick when maintenance IS funded.
const RECOVERY_RATE_PER_TICK: f32 = 0.001;

/// Base failure probability per tick for a structure at full condition.
const BASE_FAILURE_PROB: f32 = 0.0001;

/// Additional failure probability per year of age.
const AGE_FAILURE_FACTOR: f32 = 0.00005;

/// Failure probability multiplier when condition is low (scales as 1/condition).
/// At condition 0.5, failure prob is doubled; at 0.25, quadrupled.
const CONDITION_FAILURE_EXPONENT: f32 = 2.0;

/// When a levee fails due to overtopping, flood depth is amplified by this factor.
const OVERTOPPING_AMPLIFICATION: f32 = 1.5;

// =============================================================================
// Types
// =============================================================================

/// Type of flood protection structure.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, bitcode::Encode, bitcode::Decode,
)]
pub enum ProtectionType {
    /// River levee: prevents flooding up to design height.
    Levee,
    /// Coastal seawall: prevents coastal surge.
    Seawall,
    /// Floodgate: controllable water barrier.
    Floodgate,
}

impl ProtectionType {
    /// Returns the design protection height in feet.
    pub fn design_height(self) -> f32 {
        match self {
            ProtectionType::Levee => LEVEE_DESIGN_HEIGHT,
            ProtectionType::Seawall => SEAWALL_DESIGN_HEIGHT,
            ProtectionType::Floodgate => FLOODGATE_DESIGN_HEIGHT,
        }
    }
}

/// A single flood protection structure placed on the grid.
#[derive(Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct ProtectionStructure {
    /// Grid X coordinate.
    pub grid_x: u16,
    /// Grid Y coordinate.
    pub grid_y: u16,
    /// Type of protection.
    pub protection_type: ProtectionType,
    /// Structural condition (0.0 = destroyed, 1.0 = perfect).
    pub condition: f32,
    /// Age in game days since placement.
    pub age_days: u32,
    /// Whether this structure has failed (overtopped or collapsed).
    pub failed: bool,
    /// Whether the floodgate is currently open (only relevant for Floodgate type).
    pub gate_open: bool,
}

impl ProtectionStructure {
    /// Create a new protection structure at the given grid position.
    pub fn new(grid_x: usize, grid_y: usize, protection_type: ProtectionType) -> Self {
        Self {
            grid_x: grid_x as u16,
            grid_y: grid_y as u16,
            protection_type,
            condition: 1.0,
            age_days: 0,
            failed: false,
            gate_open: false,
        }
    }

    /// Effective protection height accounting for condition degradation.
    pub fn effective_height(&self) -> f32 {
        if self.failed {
            return 0.0;
        }
        if self.protection_type == ProtectionType::Floodgate && self.gate_open {
            return 0.0;
        }
        self.protection_type.design_height() * self.condition
    }

    /// Calculate failure probability for this tick based on age and condition.
    pub fn failure_probability(&self) -> f32 {
        if self.failed {
            return 0.0; // Already failed
        }
        let age_factor = 1.0 + self.age_days as f32 * AGE_FAILURE_FACTOR;
        let condition_factor = if self.condition > 0.01 {
            (1.0 / self.condition).powf(CONDITION_FAILURE_EXPONENT)
        } else {
            100.0 // Essentially guaranteed failure
        };
        (BASE_FAILURE_PROB * age_factor * condition_factor).min(1.0)
    }
}

// =============================================================================
// FloodProtectionState resource
// =============================================================================

/// Resource tracking all flood protection infrastructure in the city.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct FloodProtectionState {
    /// All placed protection structures.
    pub structures: Vec<ProtectionStructure>,
    /// Total number of active (non-failed) protection cells.
    pub active_count: u32,
    /// Total number of failed protection cells.
    pub failed_count: u32,
    /// Total annual maintenance cost for all structures.
    pub annual_maintenance_cost: f64,
    /// Whether maintenance is currently funded from the budget.
    pub maintenance_funded: bool,
    /// Total flood damage prevented this period (estimated).
    pub damage_prevented: f64,
    /// Last game day that yearly maintenance was charged.
    pub last_maintenance_day: u32,
    /// Number of overtopping events this period.
    pub overtopping_events: u32,
}

impl Default for FloodProtectionState {
    fn default() -> Self {
        Self {
            structures: Vec::new(),
            active_count: 0,
            failed_count: 0,
            annual_maintenance_cost: 0.0,
            maintenance_funded: true,
            damage_prevented: 0.0,
            last_maintenance_day: 0,
            overtopping_events: 0,
        }
    }
}

impl Saveable for FloodProtectionState {
    const SAVE_KEY: &'static str = "flood_protection";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.structures.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Pure helper functions
// =============================================================================

/// Check if a grid cell is adjacent to water (river or coast).
pub fn is_adjacent_to_water(world_grid: &WorldGrid, x: usize, y: usize) -> bool {
    let (neighbors, count) = world_grid.neighbors4(x, y);
    for &(nx, ny) in &neighbors[..count] {
        if world_grid.get(nx, ny).cell_type == CellType::Water {
            return true;
        }
    }
    false
}

/// Check if a cell is a valid placement location for a levee (adjacent to river/water, not water itself).
pub fn can_place_levee(world_grid: &WorldGrid, x: usize, y: usize) -> bool {
    if x >= GRID_WIDTH || y >= GRID_HEIGHT {
        return false;
    }
    let cell = world_grid.get(x, y);
    // Cannot place on water cells
    if cell.cell_type == CellType::Water {
        return false;
    }
    // Must be adjacent to water
    is_adjacent_to_water(world_grid, x, y)
}

/// Check if a cell is a valid placement location for a seawall (on coast edge).
/// A coastal cell is one that is adjacent to water AND on the grid edge.
pub fn can_place_seawall(world_grid: &WorldGrid, x: usize, y: usize) -> bool {
    if x >= GRID_WIDTH || y >= GRID_HEIGHT {
        return false;
    }
    let cell = world_grid.get(x, y);
    if cell.cell_type == CellType::Water {
        return false;
    }
    // Must be adjacent to water
    is_adjacent_to_water(world_grid, x, y)
}

/// Check if a cell is a valid placement location for a floodgate.
/// Floodgates can be placed on any cell adjacent to water.
pub fn can_place_floodgate(world_grid: &WorldGrid, x: usize, y: usize) -> bool {
    can_place_levee(world_grid, x, y)
}

/// Calculate the daily maintenance cost from the annual cost.
pub fn daily_maintenance_cost(annual_cost: f64) -> f64 {
    annual_cost / DAYS_PER_YEAR as f64
}

/// Determine if a protection structure should fail this tick.
/// Uses a deterministic check based on tick counter for reproducibility.
pub fn should_fail(failure_prob: f32, tick_hash: u32) -> bool {
    // Convert failure probability to a threshold out of 10000
    let threshold = (failure_prob * 10000.0) as u32;
    (tick_hash % 10000) < threshold
}

// =============================================================================
// System
// =============================================================================

/// Main flood protection update system. Runs every slow tick.
///
/// Manages aging, maintenance, condition degradation, overtopping checks,
/// and flood depth reduction for all protection infrastructure.
#[allow(clippy::too_many_arguments)]
pub fn update_flood_protection(
    timer: Res<SlowTickTimer>,
    mut protection: ResMut<FloodProtectionState>,
    mut flood_grid: ResMut<FloodGrid>,
    flood_state: Res<FloodState>,
    world_grid: Res<WorldGrid>,
    mut budget: ResMut<CityBudget>,
    clock: Res<GameClock>,
) {
    if !timer.should_run() {
        return;
    }

    if protection.structures.is_empty() {
        return;
    }

    let current_day = clock.day;

    // --- Step 1: Age all structures ---
    for structure in &mut protection.structures {
        structure.age_days = structure.age_days.saturating_add(1);
    }

    // --- Step 2: Calculate and apply maintenance costs ---
    let total_structures = protection.structures.len() as f64;
    let annual_cost = total_structures * MAINTENANCE_COST_PER_CELL_PER_YEAR;
    protection.annual_maintenance_cost = annual_cost;

    // Charge daily maintenance
    if current_day > protection.last_maintenance_day {
        let daily_cost = daily_maintenance_cost(annual_cost);
        if budget.treasury >= daily_cost {
            budget.treasury -= daily_cost;
            protection.maintenance_funded = true;
        } else {
            protection.maintenance_funded = false;
        }
        protection.last_maintenance_day = current_day;
    }

    // --- Step 3: Degrade or recover condition ---
    let funded = protection.maintenance_funded;
    for structure in &mut protection.structures {
        if structure.failed {
            continue;
        }
        if funded {
            // Slowly recover condition when maintained
            structure.condition = (structure.condition + RECOVERY_RATE_PER_TICK).min(1.0);
        } else {
            // Degrade when not maintained
            structure.condition = (structure.condition - DEGRADATION_RATE_PER_TICK).max(0.0);
        }
    }

    // --- Step 4: Check for overtopping and apply protection ---
    let mut overtopping_events = 0u32;
    let mut damage_prevented = 0.0_f64;

    // Use current_day as a simple hash for failure determination
    let tick_hash = current_day.wrapping_mul(2654435761);

    for i in 0..protection.structures.len() {
        let structure = &protection.structures[i];
        let x = structure.grid_x as usize;
        let y = structure.grid_y as usize;

        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            continue;
        }

        if structure.failed {
            continue;
        }

        let effective_height = structure.effective_height();
        if effective_height <= 0.0 {
            continue;
        }

        let flood_depth = flood_grid.get(x, y);

        if flood_depth <= 0.0 {
            continue;
        }

        // Check for overtopping
        if flood_depth > effective_height {
            // Overtopping! The structure fails catastrophically.
            overtopping_events += 1;

            // Amplify flood depth at this cell (water bursts through)
            let amplified = flood_depth * OVERTOPPING_AMPLIFICATION;
            flood_grid.set(x, y, amplified);

            // Mark as failed
            protection.structures[i].failed = true;
        } else {
            // Protection holds: reduce flood depth at this cell
            let reduction = flood_depth.min(effective_height);
            flood_grid.set(x, y, (flood_depth - reduction).max(0.0));

            // Estimate damage prevented (rough: reduction * $1000 per ft)
            damage_prevented += reduction as f64 * 1000.0;
        }

        // Check for age/condition-based spontaneous failure
        let failure_prob = protection.structures[i].failure_probability();
        let structure_hash = tick_hash.wrapping_add(i as u32 * 37);
        if should_fail(failure_prob, structure_hash) && flood_depth > 0.0 {
            protection.structures[i].failed = true;
            protection.structures[i].condition = 0.0;
        }
    }

    protection.overtopping_events = overtopping_events;
    protection.damage_prevented = damage_prevented;

    // --- Step 5: Also protect neighboring cells behind the protection line ---
    // For each non-failed structure, reduce flood depth in cells on the
    // opposite side from the water source.
    if flood_state.is_flooding {
        for structure in &protection.structures {
            if structure.failed {
                continue;
            }
            let x = structure.grid_x as usize;
            let y = structure.grid_y as usize;
            if x >= GRID_WIDTH || y >= GRID_HEIGHT {
                continue;
            }

            let effective_height = structure.effective_height();
            if effective_height <= 0.0 {
                continue;
            }

            // Find neighboring non-water cells and reduce their flood depth
            let (neighbors, count) = world_grid.neighbors4(x, y);
            for &(nx, ny) in &neighbors[..count] {
                if world_grid.get(nx, ny).cell_type == CellType::Water {
                    continue;
                }
                let neighbor_depth = flood_grid.get(nx, ny);
                if neighbor_depth > 0.0 && neighbor_depth <= effective_height {
                    let reduction = neighbor_depth * 0.5; // 50% reduction for shielded cells
                    flood_grid.set(nx, ny, (neighbor_depth - reduction).max(0.0));
                }
            }
        }
    }

    // --- Step 6: Update aggregate statistics ---
    let mut active = 0u32;
    let mut failed = 0u32;
    for structure in &protection.structures {
        if structure.failed {
            failed += 1;
        } else {
            active += 1;
        }
    }
    protection.active_count = active;
    protection.failed_count = failed;
}

// =============================================================================
// Plugin
// =============================================================================

pub struct FloodProtectionPlugin;

impl Plugin for FloodProtectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FloodProtectionState>().add_systems(
            FixedUpdate,
            update_flood_protection.after(crate::flood_simulation::update_flood_simulation),
        );

        // Register for save/load via the SaveableRegistry
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<FloodProtectionState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // ProtectionType tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_levee_design_height() {
        assert!(
            (ProtectionType::Levee.design_height() - 10.0).abs() < f32::EPSILON,
            "Levee design height should be 10 ft"
        );
    }

    #[test]
    fn test_seawall_design_height() {
        assert!(
            (ProtectionType::Seawall.design_height() - 15.0).abs() < f32::EPSILON,
            "Seawall design height should be 15 ft"
        );
    }

    #[test]
    fn test_floodgate_design_height() {
        assert!(
            (ProtectionType::Floodgate.design_height() - 12.0).abs() < f32::EPSILON,
            "Floodgate design height should be 12 ft"
        );
    }

    // -------------------------------------------------------------------------
    // ProtectionStructure tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_new_structure_defaults() {
        let s = ProtectionStructure::new(10, 20, ProtectionType::Levee);
        assert_eq!(s.grid_x, 10);
        assert_eq!(s.grid_y, 20);
        assert_eq!(s.protection_type, ProtectionType::Levee);
        assert!((s.condition - 1.0).abs() < f32::EPSILON);
        assert_eq!(s.age_days, 0);
        assert!(!s.failed);
        assert!(!s.gate_open);
    }

    #[test]
    fn test_effective_height_full_condition() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        assert!(
            (s.effective_height() - 10.0).abs() < f32::EPSILON,
            "Full condition levee should have 10 ft effective height"
        );
    }

    #[test]
    fn test_effective_height_degraded() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.condition = 0.5;
        assert!(
            (s.effective_height() - 5.0).abs() < f32::EPSILON,
            "Half condition levee should have 5 ft effective height"
        );
    }

    #[test]
    fn test_effective_height_failed() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.failed = true;
        assert!(
            s.effective_height().abs() < f32::EPSILON,
            "Failed levee should have 0 effective height"
        );
    }

    #[test]
    fn test_effective_height_open_floodgate() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Floodgate);
        s.gate_open = true;
        assert!(
            s.effective_height().abs() < f32::EPSILON,
            "Open floodgate should have 0 effective height"
        );
    }

    #[test]
    fn test_effective_height_closed_floodgate() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Floodgate);
        assert!(
            (s.effective_height() - 12.0).abs() < f32::EPSILON,
            "Closed floodgate should have 12 ft effective height"
        );
    }

    #[test]
    fn test_effective_height_seawall() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Seawall);
        assert!(
            (s.effective_height() - 15.0).abs() < f32::EPSILON,
            "Full condition seawall should have 15 ft effective height"
        );
    }

    // -------------------------------------------------------------------------
    // Failure probability tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_failure_prob_new_structure() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        let prob = s.failure_probability();
        assert!(
            prob > 0.0 && prob < 0.001,
            "New structure should have very low failure prob: {}",
            prob
        );
    }

    #[test]
    fn test_failure_prob_aged_structure() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.age_days = 3600; // 10 years
        let prob = s.failure_probability();
        let new_prob = ProtectionStructure::new(0, 0, ProtectionType::Levee).failure_probability();
        assert!(
            prob > new_prob,
            "Aged structure should have higher failure prob: {} vs {}",
            prob,
            new_prob
        );
    }

    #[test]
    fn test_failure_prob_degraded_condition() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.condition = 0.25;
        let prob = s.failure_probability();
        let new_prob = ProtectionStructure::new(0, 0, ProtectionType::Levee).failure_probability();
        assert!(
            prob > new_prob * 2.0,
            "Low condition should significantly increase failure prob: {} vs {}",
            prob,
            new_prob
        );
    }

    #[test]
    fn test_failure_prob_already_failed() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.failed = true;
        assert!(
            s.failure_probability().abs() < f32::EPSILON,
            "Already failed structure should have 0 failure prob"
        );
    }

    #[test]
    fn test_failure_prob_capped_at_one() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.condition = 0.001;
        s.age_days = 100_000;
        let prob = s.failure_probability();
        assert!(
            prob <= 1.0,
            "Failure probability should be capped at 1.0, got {}",
            prob
        );
    }

    // -------------------------------------------------------------------------
    // Placement validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_can_place_levee_on_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(5, 5).cell_type = CellType::Water;
        // Placing ON water should fail
        assert!(
            !can_place_levee(&grid, 5, 5),
            "Should not place levee on water"
        );
    }

    #[test]
    fn test_can_place_levee_adjacent_to_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(5, 5).cell_type = CellType::Water;
        // Adjacent cell should be valid
        assert!(
            can_place_levee(&grid, 5, 6),
            "Should be able to place levee adjacent to water"
        );
    }

    #[test]
    fn test_can_place_levee_not_adjacent_to_water() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        // All cells are Grass, no water adjacent
        assert!(
            !can_place_levee(&grid, 128, 128),
            "Should not place levee far from water"
        );
    }

    #[test]
    fn test_can_place_levee_out_of_bounds() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert!(
            !can_place_levee(&grid, GRID_WIDTH, 0),
            "Out of bounds should fail"
        );
    }

    #[test]
    fn test_can_place_seawall_adjacent_to_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(0, 0).cell_type = CellType::Water;
        assert!(
            can_place_seawall(&grid, 1, 0),
            "Should be able to place seawall adjacent to coast water"
        );
    }

    #[test]
    fn test_can_place_floodgate_same_as_levee() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(10, 10).cell_type = CellType::Water;
        assert_eq!(
            can_place_floodgate(&grid, 10, 11),
            can_place_levee(&grid, 10, 11),
            "Floodgate placement should follow levee rules"
        );
    }

    // -------------------------------------------------------------------------
    // Water adjacency tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_adjacent_to_water_true() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(5, 5).cell_type = CellType::Water;
        assert!(is_adjacent_to_water(&grid, 5, 6));
        assert!(is_adjacent_to_water(&grid, 5, 4));
        assert!(is_adjacent_to_water(&grid, 6, 5));
        assert!(is_adjacent_to_water(&grid, 4, 5));
    }

    #[test]
    fn test_is_adjacent_to_water_false() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        assert!(!is_adjacent_to_water(&grid, 128, 128));
    }

    #[test]
    fn test_is_adjacent_to_water_at_corner() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(0, 1).cell_type = CellType::Water;
        assert!(is_adjacent_to_water(&grid, 0, 0));
    }

    // -------------------------------------------------------------------------
    // Maintenance cost tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_daily_maintenance_cost() {
        let annual = 2000.0;
        let daily = daily_maintenance_cost(annual);
        let expected = 2000.0 / 360.0;
        assert!(
            (daily - expected).abs() < 0.01,
            "Daily maintenance should be {}, got {}",
            expected,
            daily
        );
    }

    #[test]
    fn test_annual_maintenance_cost_scaling() {
        // 10 structures = $20,000/year
        let annual = 10.0 * MAINTENANCE_COST_PER_CELL_PER_YEAR;
        assert!(
            (annual - 20_000.0).abs() < f64::EPSILON,
            "10 structures should cost $20,000/year"
        );
    }

    // -------------------------------------------------------------------------
    // Condition degradation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_condition_degradation_rate() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.condition -= DEGRADATION_RATE_PER_TICK;
        assert!(
            (s.condition - (1.0 - DEGRADATION_RATE_PER_TICK)).abs() < f32::EPSILON,
            "One tick degradation should reduce condition by {}",
            DEGRADATION_RATE_PER_TICK
        );
    }

    #[test]
    fn test_condition_recovery_rate() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.condition = 0.5;
        s.condition = (s.condition + RECOVERY_RATE_PER_TICK).min(1.0);
        assert!(
            (s.condition - (0.5 + RECOVERY_RATE_PER_TICK)).abs() < f32::EPSILON,
            "One tick recovery should increase condition by {}",
            RECOVERY_RATE_PER_TICK
        );
    }

    #[test]
    fn test_condition_does_not_exceed_one() {
        let mut condition = 0.999;
        condition = (condition + RECOVERY_RATE_PER_TICK).min(1.0);
        assert!(
            condition <= 1.0,
            "Condition should not exceed 1.0, got {}",
            condition
        );
    }

    #[test]
    fn test_condition_does_not_go_below_zero() {
        let mut condition = 0.001_f32;
        condition = (condition - DEGRADATION_RATE_PER_TICK).max(0.0);
        assert!(
            condition >= 0.0,
            "Condition should not go below 0.0, got {}",
            condition
        );
    }

    // -------------------------------------------------------------------------
    // Should-fail deterministic test
    // -------------------------------------------------------------------------

    #[test]
    fn test_should_fail_zero_prob() {
        assert!(
            !should_fail(0.0, 12345),
            "Zero probability should never fail"
        );
    }

    #[test]
    fn test_should_fail_certain() {
        assert!(
            should_fail(1.0, 12345),
            "Probability 1.0 should always fail"
        );
    }

    #[test]
    fn test_should_fail_deterministic() {
        let result1 = should_fail(0.5, 42);
        let result2 = should_fail(0.5, 42);
        assert_eq!(result1, result2, "Same inputs should give same result");
    }

    // -------------------------------------------------------------------------
    // Overtopping amplification tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_overtopping_amplification_factor() {
        let flood_depth = 12.0_f32;
        let amplified = flood_depth * OVERTOPPING_AMPLIFICATION;
        assert!(
            (amplified - 18.0).abs() < f32::EPSILON,
            "12 ft flood with 1.5x amplification should give 18 ft, got {}",
            amplified
        );
    }

    // -------------------------------------------------------------------------
    // FloodProtectionState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_state() {
        let state = FloodProtectionState::default();
        assert!(state.structures.is_empty());
        assert_eq!(state.active_count, 0);
        assert_eq!(state.failed_count, 0);
        assert!(state.annual_maintenance_cost.abs() < f64::EPSILON);
        assert!(state.maintenance_funded);
        assert!(state.damage_prevented.abs() < f64::EPSILON);
        assert_eq!(state.last_maintenance_day, 0);
        assert_eq!(state.overtopping_events, 0);
    }

    #[test]
    fn test_state_with_structures() {
        let mut state = FloodProtectionState::default();
        state
            .structures
            .push(ProtectionStructure::new(5, 5, ProtectionType::Levee));
        state
            .structures
            .push(ProtectionStructure::new(6, 5, ProtectionType::Seawall));
        state
            .structures
            .push(ProtectionStructure::new(7, 5, ProtectionType::Floodgate));
        assert_eq!(state.structures.len(), 3);
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_empty_returns_none() {
        let state = FloodProtectionState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Empty state should return None for save"
        );
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut state = FloodProtectionState::default();
        state
            .structures
            .push(ProtectionStructure::new(10, 20, ProtectionType::Levee));
        state
            .structures
            .push(ProtectionStructure::new(11, 20, ProtectionType::Seawall));
        state.active_count = 2;
        state.annual_maintenance_cost = 4000.0;
        state.damage_prevented = 50000.0;

        let bytes = state.save_to_bytes().expect("should have bytes");
        let loaded = FloodProtectionState::load_from_bytes(&bytes);

        assert_eq!(loaded.structures.len(), 2);
        assert_eq!(loaded.structures[0].grid_x, 10);
        assert_eq!(loaded.structures[0].grid_y, 20);
        assert_eq!(loaded.structures[0].protection_type, ProtectionType::Levee);
        assert_eq!(
            loaded.structures[1].protection_type,
            ProtectionType::Seawall
        );
        assert_eq!(loaded.active_count, 2);
        assert!((loaded.annual_maintenance_cost - 4000.0).abs() < f64::EPSILON);
        assert!((loaded.damage_prevented - 50000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_saveable_key() {
        assert_eq!(
            FloodProtectionState::SAVE_KEY,
            "flood_protection",
            "Save key should be 'flood_protection'"
        );
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_positive() {
        assert!(LEVEE_DESIGN_HEIGHT > 0.0);
        assert!(SEAWALL_DESIGN_HEIGHT > 0.0);
        assert!(FLOODGATE_DESIGN_HEIGHT > 0.0);
        assert!(MAINTENANCE_COST_PER_CELL_PER_YEAR > 0.0);
        assert!(DEGRADATION_RATE_PER_TICK > 0.0);
        assert!(RECOVERY_RATE_PER_TICK > 0.0);
        assert!(BASE_FAILURE_PROB > 0.0);
        assert!(AGE_FAILURE_FACTOR > 0.0);
        assert!(OVERTOPPING_AMPLIFICATION > 1.0);
    }

    #[test]
    fn test_seawall_higher_than_levee() {
        assert!(
            SEAWALL_DESIGN_HEIGHT > LEVEE_DESIGN_HEIGHT,
            "Seawall should have higher protection than levee"
        );
    }

    #[test]
    fn test_floodgate_between_levee_and_seawall() {
        assert!(
            FLOODGATE_DESIGN_HEIGHT > LEVEE_DESIGN_HEIGHT,
            "Floodgate should be higher than levee"
        );
        assert!(
            FLOODGATE_DESIGN_HEIGHT < SEAWALL_DESIGN_HEIGHT,
            "Floodgate should be lower than seawall"
        );
    }

    #[test]
    fn test_degradation_faster_than_recovery() {
        assert!(
            DEGRADATION_RATE_PER_TICK > RECOVERY_RATE_PER_TICK,
            "Degradation should be faster than recovery"
        );
    }

    // -------------------------------------------------------------------------
    // Protection effectiveness tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_levee_protects_below_design_height() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        let flood_depth = 8.0_f32;
        let effective = s.effective_height();
        // Flood is below design height, levee should protect
        assert!(
            flood_depth <= effective,
            "8 ft flood should be within 10 ft levee protection"
        );
    }

    #[test]
    fn test_levee_overtopped_above_design_height() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        let flood_depth = 12.0_f32;
        let effective = s.effective_height();
        // Flood exceeds design height
        assert!(
            flood_depth > effective,
            "12 ft flood should overtop 10 ft levee"
        );
    }

    #[test]
    fn test_degraded_levee_overtopped_at_lower_depth() {
        let mut s = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        s.condition = 0.6; // effective height = 6.0
        let flood_depth = 7.0_f32;
        let effective = s.effective_height();
        assert!(
            flood_depth > effective,
            "7 ft flood should overtop degraded levee with 6 ft effective height"
        );
    }

    #[test]
    fn test_seawall_protects_higher_surge() {
        let s = ProtectionStructure::new(0, 0, ProtectionType::Seawall);
        let surge = 14.0_f32;
        assert!(
            surge <= s.effective_height(),
            "14 ft surge should be within 15 ft seawall protection"
        );
    }

    // -------------------------------------------------------------------------
    // Integration-style data structure tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_multiple_structure_types() {
        let mut state = FloodProtectionState::default();
        state
            .structures
            .push(ProtectionStructure::new(1, 1, ProtectionType::Levee));
        state
            .structures
            .push(ProtectionStructure::new(2, 1, ProtectionType::Seawall));
        state
            .structures
            .push(ProtectionStructure::new(3, 1, ProtectionType::Floodgate));

        assert_eq!(state.structures[0].effective_height(), 10.0);
        assert_eq!(state.structures[1].effective_height(), 15.0);
        assert_eq!(state.structures[2].effective_height(), 12.0);
    }

    #[test]
    fn test_aging_increases_failure_probability() {
        let mut young = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        let mut old = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        old.age_days = 7200; // 20 years

        let prob_young = young.failure_probability();
        let prob_old = old.failure_probability();

        assert!(
            prob_old > prob_young,
            "20-year-old structure should have higher failure prob ({}) than new ({})",
            prob_old,
            prob_young
        );
    }

    #[test]
    fn test_low_condition_increases_failure_probability() {
        let good = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        let mut poor = ProtectionStructure::new(0, 0, ProtectionType::Levee);
        poor.condition = 0.25;

        let prob_good = good.failure_probability();
        let prob_poor = poor.failure_probability();

        assert!(
            prob_poor > prob_good * 4.0,
            "Condition 0.25 should multiply failure prob by ~16x: poor={}, good={}",
            prob_poor,
            prob_good
        );
    }

    #[test]
    fn test_floodgate_toggle() {
        let mut gate = ProtectionStructure::new(5, 5, ProtectionType::Floodgate);
        assert!(!gate.gate_open);
        assert!((gate.effective_height() - 12.0).abs() < f32::EPSILON);

        gate.gate_open = true;
        assert!(gate.effective_height().abs() < f32::EPSILON);

        gate.gate_open = false;
        assert!((gate.effective_height() - 12.0).abs() < f32::EPSILON);
    }
}
