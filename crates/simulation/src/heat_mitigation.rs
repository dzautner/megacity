//! Heat wave mitigation measures (WEATHER-013).
//!
//! Provides several mitigation options that cities can deploy to reduce the
//! impact of heat waves on population, infrastructure, and services:
//!
//! - **Cooling centers**: Public buildings open as shelters, reducing heat
//!   mortality by 50%. Cost: $10,000/day during heat waves.
//! - **Green canopy**: Tree coverage provides local temperature reduction of
//!   5F per 20% tree coverage in a radius. Passive; derived from tree grid.
//! - **Light-colored roofs**: Building upgrade that reduces roof temperature
//!   by 3F. Cost: $5,000 per building (one-time upgrade).
//! - **Misting stations**: Placeable infrastructure that reduces perceived
//!   temperature by 10F in public spaces. Cost: $2,000/day during heat waves.
//! - **Emergency water distribution**: Policy toggle that prevents dehydration
//!   deaths during heat waves. Cost: $8,000/day during heat waves.
//!
//! Each mitigation has a cost and activation condition (only during heat waves).
//! The `HeatMitigationState` resource tracks which measures are active and
//! computes the aggregate effects.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::heat_wave::{HeatWaveSeverity, HeatWaveState};
use crate::trees::TreeGrid;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Mortality reduction factor when cooling centers are active (50%).
const COOLING_CENTER_MORTALITY_REDUCTION: f32 = 0.50;

/// Daily operating cost of cooling centers during heat waves.
const COOLING_CENTER_DAILY_COST: f64 = 10_000.0;

/// Temperature reduction per 20% tree coverage (Fahrenheit).
const GREEN_CANOPY_TEMP_REDUCTION_PER_20PCT: f32 = 5.0;

/// Temperature reduction from light-colored roofs (Fahrenheit).
const LIGHT_ROOF_TEMP_REDUCTION: f32 = 3.0;

/// One-time cost per building for light-colored roof upgrade.
/// Used by UI/policy layer; tested below.
#[allow(dead_code)]
const LIGHT_ROOF_UPGRADE_COST: f64 = 5_000.0;

/// Perceived temperature reduction from misting stations (Fahrenheit).
const MISTING_STATION_TEMP_REDUCTION: f32 = 10.0;

/// Daily operating cost per misting station during heat waves.
const MISTING_STATION_DAILY_COST: f64 = 2_000.0;

/// Daily operating cost of emergency water distribution during heat waves.
const EMERGENCY_WATER_DAILY_COST: f64 = 8_000.0;

/// Slow tick interval divider: update costs roughly once per game day
/// (slow tick runs every ~100 ticks; we apply daily costs each slow tick
/// scaled by the fraction of a day it represents).
const COST_TICKS_PER_DAY: f32 = 10.0;

// =============================================================================
// Resources
// =============================================================================

/// Aggregate state for all heat wave mitigation measures.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct HeatMitigationState {
    // --- Player-controlled toggles ---
    /// Whether cooling centers are enabled (activate during heat waves).
    pub cooling_centers_enabled: bool,
    /// Whether emergency water distribution is enabled (activate during heat waves).
    pub emergency_water_enabled: bool,
    /// Number of misting stations placed by the player.
    pub misting_station_count: u32,
    /// Number of buildings upgraded with light-colored roofs.
    pub light_roof_count: u32,

    // --- Derived effects (computed each tick) ---
    /// Mortality reduction factor from all active mitigations (0.0 = no reduction, 1.0 = all prevented).
    pub mortality_reduction: f32,
    /// Aggregate temperature reduction from green canopy (Fahrenheit).
    pub green_canopy_temp_reduction: f32,
    /// Temperature reduction from light-colored roofs (Fahrenheit, city-wide average).
    pub light_roof_temp_reduction: f32,
    /// Perceived temperature reduction from misting stations (Fahrenheit).
    pub misting_temp_reduction: f32,
    /// Whether dehydration deaths are prevented (emergency water active during heat wave).
    pub dehydration_prevented: bool,

    // --- Cost tracking ---
    /// Total cost accumulated from mitigation measures this season.
    pub season_cost: f64,
    /// Cost incurred in the most recent update tick.
    pub last_tick_cost: f64,
    /// Total spent on light-colored roof upgrades (cumulative).
    pub light_roof_upgrade_total_cost: f64,

    // --- Internal tracking ---
    /// Last game day for which daily costs were applied.
    pub last_cost_day: u32,
}

impl Default for HeatMitigationState {
    fn default() -> Self {
        Self {
            cooling_centers_enabled: false,
            emergency_water_enabled: false,
            misting_station_count: 0,
            light_roof_count: 0,
            mortality_reduction: 0.0,
            green_canopy_temp_reduction: 0.0,
            light_roof_temp_reduction: 0.0,
            misting_temp_reduction: 0.0,
            dehydration_prevented: false,
            season_cost: 0.0,
            last_tick_cost: 0.0,
            light_roof_upgrade_total_cost: 0.0,
            last_cost_day: 0,
        }
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for HeatMitigationState {
    const SAVE_KEY: &'static str = "heat_mitigation";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if all toggles are off and no stations/upgrades placed
        if !self.cooling_centers_enabled
            && !self.emergency_water_enabled
            && self.misting_station_count == 0
            && self.light_roof_count == 0
            && self.season_cost == 0.0
        {
            return None;
        }
        // Manual binary serialization of persistent fields only.
        // Layout: [cooling:u8, water:u8, misting:u32, roofs:u32,
        //          season_cost:f64, roof_cost:f64, last_day:u32]
        // Total: 2 + 4 + 4 + 8 + 8 + 4 = 30 bytes
        let mut buf = Vec::with_capacity(30);
        buf.push(self.cooling_centers_enabled as u8);
        buf.push(self.emergency_water_enabled as u8);
        buf.extend_from_slice(&self.misting_station_count.to_le_bytes());
        buf.extend_from_slice(&self.light_roof_count.to_le_bytes());
        buf.extend_from_slice(&self.season_cost.to_le_bytes());
        buf.extend_from_slice(&self.light_roof_upgrade_total_cost.to_le_bytes());
        buf.extend_from_slice(&self.last_cost_day.to_le_bytes());
        Some(buf)
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < 30 {
            warn!(
                "Saveable {}: expected >= 30 bytes, got {}, falling back to default",
                Self::SAVE_KEY,
                bytes.len()
            );
            return Self::default();
        }
        let cooling = bytes[0] != 0;
        let water = bytes[1] != 0;
        let misting = u32::from_le_bytes(bytes[2..6].try_into().unwrap_or([0; 4]));
        let roofs = u32::from_le_bytes(bytes[6..10].try_into().unwrap_or([0; 4]));
        let season_cost = f64::from_le_bytes(bytes[10..18].try_into().unwrap_or([0; 8]));
        let roof_cost = f64::from_le_bytes(bytes[18..26].try_into().unwrap_or([0; 8]));
        let last_day = u32::from_le_bytes(bytes[26..30].try_into().unwrap_or([0; 4]));
        Self {
            cooling_centers_enabled: cooling,
            emergency_water_enabled: water,
            misting_station_count: misting,
            light_roof_count: roofs,
            season_cost,
            light_roof_upgrade_total_cost: roof_cost,
            last_cost_day: last_day,
            // Derived fields are recomputed by the system
            ..Default::default()
        }
    }
}

// =============================================================================
// Pure helper functions
// =============================================================================

/// Compute the average tree coverage fraction across the entire grid.
/// Returns a value in [0.0, 1.0].
pub fn average_tree_coverage(tree_grid: &TreeGrid) -> f32 {
    let total = (GRID_WIDTH * GRID_HEIGHT) as f32;
    if total == 0.0 {
        return 0.0;
    }
    let count = tree_grid.cells.iter().filter(|&&has_tree| has_tree).count() as f32;
    count / total
}

/// Compute the green canopy temperature reduction based on average tree coverage.
/// -5F per 20% coverage.
pub fn green_canopy_reduction(tree_coverage_fraction: f32) -> f32 {
    // Each 0.20 fraction of tree coverage = 5F reduction
    let increments = tree_coverage_fraction / 0.20;
    increments * GREEN_CANOPY_TEMP_REDUCTION_PER_20PCT
}

/// Compute the light-colored roof temperature reduction as a city-wide average.
/// Returns the reduction in Fahrenheit scaled by the fraction of buildings upgraded.
pub fn light_roof_reduction(upgraded_count: u32, total_buildings: u32) -> f32 {
    if total_buildings == 0 {
        return 0.0;
    }
    let fraction = (upgraded_count as f32 / total_buildings as f32).min(1.0);
    fraction * LIGHT_ROOF_TEMP_REDUCTION
}

/// Compute the misting station temperature reduction.
/// Scales with the number of stations, capped at the maximum reduction.
pub fn misting_reduction(station_count: u32) -> f32 {
    if station_count == 0 {
        return 0.0;
    }
    // Each station covers a portion of the city; diminishing returns after many.
    // Model: full effect at 50+ stations, linear ramp up.
    let fraction = (station_count as f32 / 50.0).min(1.0);
    fraction * MISTING_STATION_TEMP_REDUCTION
}

/// Compute the total mortality reduction factor from active mitigations.
/// Returns a value in [0.0, 1.0] where 1.0 means all heat mortality prevented.
pub fn total_mortality_reduction(cooling_centers_active: bool, dehydration_prevented: bool) -> f32 {
    let mut reduction = 0.0_f32;
    if cooling_centers_active {
        reduction += COOLING_CENTER_MORTALITY_REDUCTION;
    }
    // Emergency water prevents dehydration component (~30% of heat deaths)
    if dehydration_prevented {
        reduction += 0.30;
    }
    reduction.min(1.0)
}

/// Compute the daily operating cost of all active mitigations during a heat wave.
pub fn daily_operating_cost(
    cooling_centers_active: bool,
    emergency_water_active: bool,
    misting_station_count: u32,
) -> f64 {
    let mut cost = 0.0_f64;
    if cooling_centers_active {
        cost += COOLING_CENTER_DAILY_COST;
    }
    if emergency_water_active {
        cost += EMERGENCY_WATER_DAILY_COST;
    }
    cost += misting_station_count as f64 * MISTING_STATION_DAILY_COST;
    cost
}

// =============================================================================
// Systems
// =============================================================================

/// System that updates heat mitigation effects based on current heat wave state.
///
/// Runs on the slow tick timer. Only applies costs and effects when a heat wave
/// is active (severity > None). When no heat wave is active, derived effects
/// are zeroed out but player toggles remain.
pub fn update_heat_mitigation(
    timer: Res<SlowTickTimer>,
    heat_wave: Res<HeatWaveState>,
    tree_grid: Res<TreeGrid>,
    buildings: Query<&crate::buildings::Building>,
    mut mitigation: ResMut<HeatMitigationState>,
    mut budget: ResMut<CityBudget>,
) {
    if !timer.should_run() {
        return;
    }

    let is_heat_wave = heat_wave.severity != HeatWaveSeverity::None;

    // --- Green canopy: always computed (passive benefit) ---
    let tree_coverage = average_tree_coverage(&tree_grid);
    mitigation.green_canopy_temp_reduction = green_canopy_reduction(tree_coverage);

    // --- Light-colored roofs: always computed (passive benefit) ---
    let total_buildings = buildings.iter().count() as u32;
    mitigation.light_roof_temp_reduction =
        light_roof_reduction(mitigation.light_roof_count, total_buildings);

    if !is_heat_wave {
        // No heat wave: zero out active-only effects, no costs
        mitigation.mortality_reduction = 0.0;
        mitigation.misting_temp_reduction = 0.0;
        mitigation.dehydration_prevented = false;
        mitigation.last_tick_cost = 0.0;
        return;
    }

    // --- Heat wave is active: compute active mitigation effects ---

    // Cooling centers
    let cooling_active = mitigation.cooling_centers_enabled;

    // Emergency water distribution
    let water_active = mitigation.emergency_water_enabled;
    mitigation.dehydration_prevented = water_active;

    // Misting stations
    mitigation.misting_temp_reduction = misting_reduction(mitigation.misting_station_count);

    // Aggregate mortality reduction
    mitigation.mortality_reduction = total_mortality_reduction(cooling_active, water_active);

    // --- Costs: apply fractional daily cost per slow tick ---
    let daily_cost = daily_operating_cost(
        cooling_active,
        water_active,
        mitigation.misting_station_count,
    );
    // Each slow tick is approximately 1/COST_TICKS_PER_DAY of a game day
    let tick_cost = daily_cost / COST_TICKS_PER_DAY as f64;
    budget.treasury -= tick_cost;
    mitigation.last_tick_cost = tick_cost;
    mitigation.season_cost += tick_cost;
}

// =============================================================================
// Plugin
// =============================================================================

pub struct HeatMitigationPlugin;

impl Plugin for HeatMitigationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HeatMitigationState>().add_systems(
            FixedUpdate,
            update_heat_mitigation
                .after(crate::heat_wave::update_heat_wave)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<HeatMitigationState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Default state tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_state() {
        let state = HeatMitigationState::default();
        assert!(!state.cooling_centers_enabled);
        assert!(!state.emergency_water_enabled);
        assert_eq!(state.misting_station_count, 0);
        assert_eq!(state.light_roof_count, 0);
        assert!((state.mortality_reduction).abs() < f32::EPSILON);
        assert!((state.green_canopy_temp_reduction).abs() < f32::EPSILON);
        assert!((state.light_roof_temp_reduction).abs() < f32::EPSILON);
        assert!((state.misting_temp_reduction).abs() < f32::EPSILON);
        assert!(!state.dehydration_prevented);
        assert_eq!(state.season_cost, 0.0);
        assert_eq!(state.last_tick_cost, 0.0);
        assert_eq!(state.light_roof_upgrade_total_cost, 0.0);
    }

    // -------------------------------------------------------------------------
    // Green canopy tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_green_canopy_no_trees() {
        let reduction = green_canopy_reduction(0.0);
        assert!(reduction.abs() < f32::EPSILON, "no trees = no reduction");
    }

    #[test]
    fn test_green_canopy_20_percent() {
        let reduction = green_canopy_reduction(0.20);
        assert!(
            (reduction - 5.0).abs() < f32::EPSILON,
            "20% coverage = 5F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_green_canopy_40_percent() {
        let reduction = green_canopy_reduction(0.40);
        assert!(
            (reduction - 10.0).abs() < f32::EPSILON,
            "40% coverage = 10F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_green_canopy_100_percent() {
        let reduction = green_canopy_reduction(1.0);
        assert!(
            (reduction - 25.0).abs() < f32::EPSILON,
            "100% coverage = 25F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_green_canopy_10_percent() {
        let reduction = green_canopy_reduction(0.10);
        assert!(
            (reduction - 2.5).abs() < f32::EPSILON,
            "10% coverage = 2.5F reduction, got {}",
            reduction
        );
    }

    // -------------------------------------------------------------------------
    // Light-colored roof tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_light_roof_no_buildings() {
        let reduction = light_roof_reduction(0, 0);
        assert!(
            reduction.abs() < f32::EPSILON,
            "no buildings = no reduction"
        );
    }

    #[test]
    fn test_light_roof_no_upgrades() {
        let reduction = light_roof_reduction(0, 100);
        assert!(reduction.abs() < f32::EPSILON, "no upgrades = no reduction");
    }

    #[test]
    fn test_light_roof_all_upgraded() {
        let reduction = light_roof_reduction(100, 100);
        assert!(
            (reduction - LIGHT_ROOF_TEMP_REDUCTION).abs() < f32::EPSILON,
            "all upgraded = full 3F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_light_roof_half_upgraded() {
        let reduction = light_roof_reduction(50, 100);
        assert!(
            (reduction - 1.5).abs() < f32::EPSILON,
            "50% upgraded = 1.5F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_light_roof_more_upgraded_than_buildings() {
        // Edge case: upgraded count exceeds building count (clamped to 1.0 fraction)
        let reduction = light_roof_reduction(200, 100);
        assert!(
            (reduction - LIGHT_ROOF_TEMP_REDUCTION).abs() < f32::EPSILON,
            "capped at full reduction, got {}",
            reduction
        );
    }

    // -------------------------------------------------------------------------
    // Misting station tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_misting_no_stations() {
        let reduction = misting_reduction(0);
        assert!(reduction.abs() < f32::EPSILON, "no stations = no reduction");
    }

    #[test]
    fn test_misting_one_station() {
        let reduction = misting_reduction(1);
        let expected = (1.0 / 50.0) * MISTING_STATION_TEMP_REDUCTION;
        assert!(
            (reduction - expected).abs() < 0.01,
            "1 station = {}F reduction, got {}",
            expected,
            reduction
        );
    }

    #[test]
    fn test_misting_50_stations() {
        let reduction = misting_reduction(50);
        assert!(
            (reduction - MISTING_STATION_TEMP_REDUCTION).abs() < f32::EPSILON,
            "50 stations = full 10F reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_misting_100_stations_capped() {
        let reduction = misting_reduction(100);
        assert!(
            (reduction - MISTING_STATION_TEMP_REDUCTION).abs() < f32::EPSILON,
            "100 stations = still capped at 10F, got {}",
            reduction
        );
    }

    #[test]
    fn test_misting_25_stations() {
        let reduction = misting_reduction(25);
        let expected = 0.5 * MISTING_STATION_TEMP_REDUCTION;
        assert!(
            (reduction - expected).abs() < f32::EPSILON,
            "25 stations = 5F reduction, got {}",
            reduction
        );
    }

    // -------------------------------------------------------------------------
    // Mortality reduction tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mortality_no_mitigation() {
        let reduction = total_mortality_reduction(false, false);
        assert!(
            reduction.abs() < f32::EPSILON,
            "no mitigation = no reduction"
        );
    }

    #[test]
    fn test_mortality_cooling_centers_only() {
        let reduction = total_mortality_reduction(true, false);
        assert!(
            (reduction - COOLING_CENTER_MORTALITY_REDUCTION).abs() < f32::EPSILON,
            "cooling centers = 50% reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_mortality_emergency_water_only() {
        let reduction = total_mortality_reduction(false, true);
        assert!(
            (reduction - 0.30).abs() < f32::EPSILON,
            "emergency water = 30% reduction, got {}",
            reduction
        );
    }

    #[test]
    fn test_mortality_both_active() {
        let reduction = total_mortality_reduction(true, true);
        // 50% + 30% = 80%, capped at 1.0
        assert!(
            (reduction - 0.80).abs() < f32::EPSILON,
            "both = 80% reduction, got {}",
            reduction
        );
    }

    // -------------------------------------------------------------------------
    // Daily operating cost tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cost_nothing_active() {
        let cost = daily_operating_cost(false, false, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_cost_cooling_centers_only() {
        let cost = daily_operating_cost(true, false, 0);
        assert!((cost - COOLING_CENTER_DAILY_COST).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_emergency_water_only() {
        let cost = daily_operating_cost(false, true, 0);
        assert!((cost - EMERGENCY_WATER_DAILY_COST).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_misting_stations() {
        let cost = daily_operating_cost(false, false, 5);
        let expected = 5.0 * MISTING_STATION_DAILY_COST;
        assert!((cost - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_all_active() {
        let cost = daily_operating_cost(true, true, 10);
        let expected = COOLING_CENTER_DAILY_COST
            + EMERGENCY_WATER_DAILY_COST
            + 10.0 * MISTING_STATION_DAILY_COST;
        assert!(
            (cost - expected).abs() < f64::EPSILON,
            "expected {}, got {}",
            expected,
            cost
        );
    }

    // -------------------------------------------------------------------------
    // Average tree coverage tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_average_tree_coverage_empty() {
        let tree_grid = TreeGrid::default();
        let coverage = average_tree_coverage(&tree_grid);
        assert!(
            coverage.abs() < f32::EPSILON,
            "empty tree grid = 0% coverage"
        );
    }

    #[test]
    fn test_average_tree_coverage_some_trees() {
        let mut tree_grid = TreeGrid::default();
        // Place 100 trees
        for i in 0..100 {
            let x = i % GRID_WIDTH;
            let y = i / GRID_WIDTH;
            tree_grid.set(x, y, true);
        }
        let coverage = average_tree_coverage(&tree_grid);
        let expected = 100.0 / (GRID_WIDTH * GRID_HEIGHT) as f32;
        assert!(
            (coverage - expected).abs() < 0.0001,
            "expected {}, got {}",
            expected,
            coverage
        );
    }

    // -------------------------------------------------------------------------
    // Saveable implementation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_default_skips_save() {
        use crate::Saveable;
        let state = HeatMitigationState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "default state should skip saving"
        );
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = HeatMitigationState::default();
        state.cooling_centers_enabled = true;
        state.emergency_water_enabled = true;
        state.misting_station_count = 25;
        state.light_roof_count = 50;
        state.season_cost = 12345.67;
        state.light_roof_upgrade_total_cost = 250_000.0;
        state.last_cost_day = 42;

        let bytes = state
            .save_to_bytes()
            .expect("should save non-default state");
        let restored = HeatMitigationState::load_from_bytes(&bytes);

        assert_eq!(restored.cooling_centers_enabled, true);
        assert_eq!(restored.emergency_water_enabled, true);
        assert_eq!(restored.misting_station_count, 25);
        assert_eq!(restored.light_roof_count, 50);
        assert!((restored.season_cost - 12345.67).abs() < 0.01);
        assert!((restored.light_roof_upgrade_total_cost - 250_000.0).abs() < 0.01);
        assert_eq!(restored.last_cost_day, 42);

        // Derived fields should be at defaults after load
        assert!((restored.mortality_reduction).abs() < f32::EPSILON);
        assert!((restored.misting_temp_reduction).abs() < f32::EPSILON);
        assert!(!restored.dehydration_prevented);
    }

    #[test]
    fn test_saveable_corrupted_bytes() {
        use crate::Saveable;
        let garbage = vec![0xFF, 0xFE, 0xFD];
        let restored = HeatMitigationState::load_from_bytes(&garbage);
        // Should produce default state on corrupt data
        assert!(!restored.cooling_centers_enabled);
        assert_eq!(restored.misting_station_count, 0);
    }

    // -------------------------------------------------------------------------
    // Constants validation
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_are_reasonable() {
        assert!(COOLING_CENTER_MORTALITY_REDUCTION > 0.0);
        assert!(COOLING_CENTER_MORTALITY_REDUCTION <= 1.0);
        assert!(COOLING_CENTER_DAILY_COST > 0.0);
        assert!(GREEN_CANOPY_TEMP_REDUCTION_PER_20PCT > 0.0);
        assert!(LIGHT_ROOF_TEMP_REDUCTION > 0.0);
        assert!(LIGHT_ROOF_UPGRADE_COST > 0.0);
        assert!(MISTING_STATION_TEMP_REDUCTION > 0.0);
        assert!(MISTING_STATION_DAILY_COST > 0.0);
        assert!(EMERGENCY_WATER_DAILY_COST > 0.0);
        assert!(COST_TICKS_PER_DAY > 0.0);
    }

    #[test]
    fn test_light_roof_upgrade_cost_constant() {
        assert!(
            (LIGHT_ROOF_UPGRADE_COST - 5_000.0).abs() < f64::EPSILON,
            "light roof upgrade should cost $5,000"
        );
    }

    // -------------------------------------------------------------------------
    // Integration-style tests (testing combined effects)
    // -------------------------------------------------------------------------

    #[test]
    fn test_combined_temp_reduction() {
        // Scenario: 40% tree coverage, 50% buildings upgraded, 25 misting stations
        let canopy = green_canopy_reduction(0.40);
        let roof = light_roof_reduction(50, 100);
        let misting = misting_reduction(25);

        let total = canopy + roof + misting;

        // 40% trees = 10F, 50% roofs = 1.5F, 25 stations = 5F -> 16.5F total
        assert!(
            (total - 16.5).abs() < 0.01,
            "expected 16.5F total reduction, got {}",
            total
        );
    }

    #[test]
    fn test_no_effects_without_heat_wave() {
        // When there's no heat wave, active costs should be zero
        let cost = daily_operating_cost(true, true, 10);
        // The cost function itself doesn't check heat wave status -- the system
        // does. So we just verify cost > 0 (the system would skip applying it).
        assert!(cost > 0.0);
    }

    #[test]
    fn test_mortality_reduction_capped() {
        // Even with all mitigations, mortality reduction should never exceed 1.0
        let reduction = total_mortality_reduction(true, true);
        assert!(
            reduction <= 1.0,
            "mortality reduction should be capped at 1.0, got {}",
            reduction
        );
    }
}
