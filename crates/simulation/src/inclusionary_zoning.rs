//! Inclusionary Zoning Requirements (ZONE-010).
//!
//! Implements inclusionary zoning as a district policy requiring new residential
//! developments to reserve a percentage of units as affordable housing.
//!
//! Features:
//! - **District policy toggle**: Inclusionary Zoning can be enabled per player-defined district
//! - **Affordable unit percentage**: 10-20% of residential units reserved as affordable housing
//! - **FAR bonus**: +10-20% Floor Area Ratio bonus to offset affordable unit costs
//! - **Affordable units**: House lower-income citizens who would otherwise be priced out
//! - **Profitability impact**: Affects building profitability and construction rate
//!
//! The system tracks which districts have inclusionary zoning enabled and computes
//! per-district effects (effective capacity reduction, FAR bonus, affordable unit counts).

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use std::collections::HashMap;

use crate::buildings::Building;
use crate::districts::DistrictMap;
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Minimum affordable housing percentage (10%).
pub const MIN_AFFORDABLE_PERCENTAGE: f32 = 0.10;

/// Maximum affordable housing percentage (20%).
pub const MAX_AFFORDABLE_PERCENTAGE: f32 = 0.20;

/// Default affordable housing percentage when policy is first enabled (15%).
pub const DEFAULT_AFFORDABLE_PERCENTAGE: f32 = 0.15;

/// Minimum FAR bonus granted to offset affordable unit cost (10%).
pub const MIN_FAR_BONUS: f32 = 0.10;

/// Maximum FAR bonus granted to offset affordable unit cost (20%).
pub const MAX_FAR_BONUS: f32 = 0.20;

/// Construction rate penalty multiplier when inclusionary zoning is active.
/// Developers build slightly slower due to reduced profitability.
pub const CONSTRUCTION_RATE_PENALTY: f32 = 0.90;

/// Monthly administrative cost per district with inclusionary zoning.
pub const MONTHLY_ADMIN_COST_PER_DISTRICT: f64 = 8_000.0;

// =============================================================================
// Per-district configuration
// =============================================================================

/// Per-district inclusionary zoning configuration.
#[derive(Debug, Clone, Encode, Decode)]
pub struct DistrictInclusionaryConfig {
    /// Whether inclusionary zoning is enabled for this district.
    pub enabled: bool,
    /// Percentage of units that must be affordable (0.10 - 0.20).
    pub affordable_percentage: f32,
}

impl Default for DistrictInclusionaryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            affordable_percentage: DEFAULT_AFFORDABLE_PERCENTAGE,
        }
    }
}

// =============================================================================
// Resource: Inclusionary Zoning State
// =============================================================================

/// Tracks inclusionary zoning policy state across all player-defined districts.
#[derive(Resource, Debug, Clone, Default, Encode, Decode)]
pub struct InclusionaryZoningState {
    /// Per-district inclusionary zoning configurations.
    /// Key is the district index in the DistrictMap.
    pub district_configs: HashMap<usize, DistrictInclusionaryConfig>,
    /// Total affordable units across all districts (computed).
    pub total_affordable_units: u32,
    /// Total residential units in affected districts (computed).
    pub total_affected_units: u32,
    /// Total monthly admin cost (computed).
    pub total_monthly_cost: f64,
}

impl InclusionaryZoningState {
    /// Enable inclusionary zoning for a district with the default percentage.
    pub fn enable(&mut self, district_idx: usize) {
        let config = self
            .district_configs
            .entry(district_idx)
            .or_insert_with(DistrictInclusionaryConfig::default);
        config.enabled = true;
    }

    /// Disable inclusionary zoning for a district.
    pub fn disable(&mut self, district_idx: usize) {
        if let Some(config) = self.district_configs.get_mut(&district_idx) {
            config.enabled = false;
        }
    }

    /// Check if inclusionary zoning is enabled for a district.
    pub fn is_enabled(&self, district_idx: usize) -> bool {
        self.district_configs
            .get(&district_idx)
            .is_some_and(|c| c.enabled)
    }

    /// Set the affordable percentage for a district (clamped to valid range).
    pub fn set_affordable_percentage(&mut self, district_idx: usize, pct: f32) {
        let clamped = pct.clamp(MIN_AFFORDABLE_PERCENTAGE, MAX_AFFORDABLE_PERCENTAGE);
        let config = self
            .district_configs
            .entry(district_idx)
            .or_insert_with(DistrictInclusionaryConfig::default);
        config.affordable_percentage = clamped;
    }

    /// Get the affordable percentage for a district (returns 0.0 if not enabled).
    pub fn affordable_percentage(&self, district_idx: usize) -> f32 {
        self.district_configs
            .get(&district_idx)
            .filter(|c| c.enabled)
            .map(|c| c.affordable_percentage)
            .unwrap_or(0.0)
    }

    /// Get the number of enabled districts.
    pub fn enabled_district_count(&self) -> usize {
        self.district_configs.values().filter(|c| c.enabled).count()
    }
}

// =============================================================================
// Pure helper functions (testable without ECS)
// =============================================================================

/// Calculate the FAR bonus for a given affordable percentage.
/// The bonus scales linearly from MIN_FAR_BONUS at MIN_AFFORDABLE_PERCENTAGE
/// to MAX_FAR_BONUS at MAX_AFFORDABLE_PERCENTAGE.
pub fn calculate_far_bonus(affordable_pct: f32) -> f32 {
    if affordable_pct <= 0.0 {
        return 0.0;
    }
    let clamped = affordable_pct.clamp(MIN_AFFORDABLE_PERCENTAGE, MAX_AFFORDABLE_PERCENTAGE);
    let t = (clamped - MIN_AFFORDABLE_PERCENTAGE)
        / (MAX_AFFORDABLE_PERCENTAGE - MIN_AFFORDABLE_PERCENTAGE);
    MIN_FAR_BONUS + t * (MAX_FAR_BONUS - MIN_FAR_BONUS)
}

/// Calculate the number of affordable units for a building given its capacity
/// and the district's affordable percentage.
pub fn calculate_affordable_units(capacity: u32, affordable_pct: f32) -> u32 {
    if affordable_pct <= 0.0 {
        return 0;
    }
    // At least 1 affordable unit if the building has any capacity and policy is active
    let raw = (capacity as f32 * affordable_pct).ceil() as u32;
    raw.min(capacity)
}

/// Calculate the effective (market-rate) capacity after removing affordable units.
pub fn calculate_effective_capacity(capacity: u32, affordable_pct: f32) -> u32 {
    let affordable = calculate_affordable_units(capacity, affordable_pct);
    capacity.saturating_sub(affordable)
}

/// Calculate the monthly admin cost for the given number of enabled districts.
pub fn calculate_monthly_admin_cost(enabled_count: usize) -> f64 {
    enabled_count as f64 * MONTHLY_ADMIN_COST_PER_DISTRICT
}

/// Check if a cell is in a district with inclusionary zoning enabled.
pub fn is_cell_in_inclusionary_district(
    x: usize,
    y: usize,
    state: &InclusionaryZoningState,
    district_map: &DistrictMap,
) -> bool {
    district_map
        .get_district_index_at(x, y)
        .is_some_and(|di| state.is_enabled(di))
}

/// Get the affordable percentage for the cell's district, or 0.0 if not in
/// an inclusionary zoning district.
pub fn affordable_percentage_for_cell(
    x: usize,
    y: usize,
    state: &InclusionaryZoningState,
    district_map: &DistrictMap,
) -> f32 {
    district_map
        .get_district_index_at(x, y)
        .map(|di| state.affordable_percentage(di))
        .unwrap_or(0.0)
}

// =============================================================================
// Systems
// =============================================================================

/// System: update inclusionary zoning computed effects every slow tick.
///
/// Iterates all residential buildings, checks if they are in a district
/// with inclusionary zoning enabled, and aggregates total affordable/affected
/// unit counts and admin costs.
pub fn update_inclusionary_zoning(
    timer: Res<SlowTickTimer>,
    mut state: ResMut<InclusionaryZoningState>,
    district_map: Res<DistrictMap>,
    buildings: Query<&Building>,
) {
    if !timer.should_run() {
        return;
    }

    let mut total_affordable = 0u32;
    let mut total_affected = 0u32;

    for building in &buildings {
        if !building.zone_type.is_residential() && !building.zone_type.is_mixed_use() {
            continue;
        }

        let di = district_map.get_district_index_at(building.grid_x, building.grid_y);
        let affordable_pct = di
            .map(|idx| state.affordable_percentage(idx))
            .unwrap_or(0.0);

        if affordable_pct > 0.0 {
            let res_capacity = if building.zone_type.is_mixed_use() {
                let (_, res_cap) =
                    crate::buildings::MixedUseBuilding::capacities_for_level(building.level);
                res_cap
            } else {
                building.capacity
            };
            total_affected += res_capacity;
            total_affordable += calculate_affordable_units(res_capacity, affordable_pct);
        }
    }

    state.total_affordable_units = total_affordable;
    state.total_affected_units = total_affected;
    state.total_monthly_cost = calculate_monthly_admin_cost(state.enabled_district_count());
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for InclusionaryZoningState {
    const SAVE_KEY: &'static str = "inclusionary_zoning";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no districts have ever been configured
        if self.district_configs.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        bitcode::decode(bytes).unwrap_or_default()
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct InclusionaryZoningPlugin;

impl Plugin for InclusionaryZoningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InclusionaryZoningState>().add_systems(
            FixedUpdate,
            update_inclusionary_zoning.after(crate::buildings::progress_construction),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<InclusionaryZoningState>();
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
        let state = InclusionaryZoningState::default();
        assert!(state.district_configs.is_empty());
        assert_eq!(state.total_affordable_units, 0);
        assert_eq!(state.total_affected_units, 0);
        assert_eq!(state.total_monthly_cost, 0.0);
    }

    #[test]
    fn test_default_config() {
        let config = DistrictInclusionaryConfig::default();
        assert!(!config.enabled);
        assert!(
            (config.affordable_percentage - DEFAULT_AFFORDABLE_PERCENTAGE).abs() < f32::EPSILON
        );
    }

    // -------------------------------------------------------------------------
    // Enable/disable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_enable_district() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        assert!(state.is_enabled(0));
        assert!(!state.is_enabled(1));
    }

    #[test]
    fn test_disable_district() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.disable(0);
        assert!(!state.is_enabled(0));
    }

    #[test]
    fn test_enable_multiple_districts() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.enable(3);
        state.enable(5);
        assert!(state.is_enabled(0));
        assert!(!state.is_enabled(1));
        assert!(state.is_enabled(3));
        assert!(state.is_enabled(5));
        assert_eq!(state.enabled_district_count(), 3);
    }

    #[test]
    fn test_enable_idempotent() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.enable(0);
        assert_eq!(state.enabled_district_count(), 1);
    }

    #[test]
    fn test_disable_nonexistent() {
        let mut state = InclusionaryZoningState::default();
        state.disable(5); // never enabled
        assert!(!state.is_enabled(5));
    }

    // -------------------------------------------------------------------------
    // Affordable percentage tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_affordable_percentage() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        let pct = state.affordable_percentage(0);
        assert!((pct - DEFAULT_AFFORDABLE_PERCENTAGE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_affordable_percentage() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.set_affordable_percentage(0, 0.18);
        assert!((state.affordable_percentage(0) - 0.18).abs() < f32::EPSILON);
    }

    #[test]
    fn test_affordable_percentage_clamped_min() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.set_affordable_percentage(0, 0.01); // below min
        assert!((state.affordable_percentage(0) - MIN_AFFORDABLE_PERCENTAGE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_affordable_percentage_clamped_max() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.set_affordable_percentage(0, 0.50); // above max
        assert!((state.affordable_percentage(0) - MAX_AFFORDABLE_PERCENTAGE).abs() < f32::EPSILON);
    }

    #[test]
    fn test_affordable_percentage_zero_when_disabled() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.set_affordable_percentage(0, 0.15);
        state.disable(0);
        assert_eq!(state.affordable_percentage(0), 0.0);
    }

    #[test]
    fn test_affordable_percentage_zero_when_not_configured() {
        let state = InclusionaryZoningState::default();
        assert_eq!(state.affordable_percentage(99), 0.0);
    }

    // -------------------------------------------------------------------------
    // FAR bonus tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_far_bonus_at_min() {
        let bonus = calculate_far_bonus(MIN_AFFORDABLE_PERCENTAGE);
        assert!((bonus - MIN_FAR_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_far_bonus_at_max() {
        let bonus = calculate_far_bonus(MAX_AFFORDABLE_PERCENTAGE);
        assert!((bonus - MAX_FAR_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_far_bonus_at_midpoint() {
        let mid_pct = (MIN_AFFORDABLE_PERCENTAGE + MAX_AFFORDABLE_PERCENTAGE) / 2.0;
        let expected = (MIN_FAR_BONUS + MAX_FAR_BONUS) / 2.0;
        let bonus = calculate_far_bonus(mid_pct);
        assert!(
            (bonus - expected).abs() < 0.001,
            "midpoint bonus should be ~{}: got {}",
            expected,
            bonus
        );
    }

    #[test]
    fn test_far_bonus_zero_when_no_policy() {
        let bonus = calculate_far_bonus(0.0);
        assert!(bonus.abs() < f32::EPSILON);
    }

    #[test]
    fn test_far_bonus_clamped_below_min() {
        // Even a very small percentage gets clamped to MIN range
        let bonus = calculate_far_bonus(0.05);
        assert!((bonus - MIN_FAR_BONUS).abs() < f32::EPSILON);
    }

    #[test]
    fn test_far_bonus_clamped_above_max() {
        let bonus = calculate_far_bonus(0.50);
        assert!((bonus - MAX_FAR_BONUS).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Affordable units calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_affordable_units_basic() {
        // 100 units at 15% = 15 affordable
        let units = calculate_affordable_units(100, 0.15);
        assert_eq!(units, 15);
    }

    #[test]
    fn test_affordable_units_rounds_up() {
        // 10 units at 15% = 1.5, ceil = 2
        let units = calculate_affordable_units(10, 0.15);
        assert_eq!(units, 2);
    }

    #[test]
    fn test_affordable_units_zero_capacity() {
        let units = calculate_affordable_units(0, 0.15);
        assert_eq!(units, 0);
    }

    #[test]
    fn test_affordable_units_zero_percentage() {
        let units = calculate_affordable_units(100, 0.0);
        assert_eq!(units, 0);
    }

    #[test]
    fn test_affordable_units_at_min() {
        // 100 units at 10% = 10
        let units = calculate_affordable_units(100, MIN_AFFORDABLE_PERCENTAGE);
        assert_eq!(units, 10);
    }

    #[test]
    fn test_affordable_units_at_max() {
        // 100 units at 20% = 20
        let units = calculate_affordable_units(100, MAX_AFFORDABLE_PERCENTAGE);
        assert_eq!(units, 20);
    }

    #[test]
    fn test_affordable_units_capped_at_capacity() {
        // Edge case: very high percentage shouldn't exceed capacity
        let units = calculate_affordable_units(5, 0.20);
        assert!(units <= 5);
    }

    // -------------------------------------------------------------------------
    // Effective capacity tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_effective_capacity_basic() {
        // 100 units at 15% affordable = 85 effective
        let effective = calculate_effective_capacity(100, 0.15);
        assert_eq!(effective, 85);
    }

    #[test]
    fn test_effective_capacity_zero_percentage() {
        let effective = calculate_effective_capacity(100, 0.0);
        assert_eq!(effective, 100);
    }

    #[test]
    fn test_effective_capacity_small_building() {
        // 10 units at 15% = 2 affordable, 8 effective
        let effective = calculate_effective_capacity(10, 0.15);
        assert_eq!(effective, 8);
    }

    #[test]
    fn test_effective_capacity_at_max() {
        // 100 units at 20% = 20 affordable, 80 effective
        let effective = calculate_effective_capacity(100, MAX_AFFORDABLE_PERCENTAGE);
        assert_eq!(effective, 80);
    }

    // -------------------------------------------------------------------------
    // Monthly admin cost tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_admin_cost_none() {
        let cost = calculate_monthly_admin_cost(0);
        assert!(cost.abs() < f64::EPSILON);
    }

    #[test]
    fn test_admin_cost_one_district() {
        let cost = calculate_monthly_admin_cost(1);
        assert!((cost - MONTHLY_ADMIN_COST_PER_DISTRICT).abs() < f64::EPSILON);
    }

    #[test]
    fn test_admin_cost_multiple_districts() {
        let cost = calculate_monthly_admin_cost(3);
        assert!((cost - 3.0 * MONTHLY_ADMIN_COST_PER_DISTRICT).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Cell query tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_cell_in_inclusionary_district() {
        let mut state = InclusionaryZoningState::default();
        let mut district_map = DistrictMap::default();

        district_map.assign_cell_to_district(10, 10, 0);

        // Not enabled yet
        assert!(!is_cell_in_inclusionary_district(
            10,
            10,
            &state,
            &district_map
        ));

        // Enable
        state.enable(0);
        assert!(is_cell_in_inclusionary_district(
            10,
            10,
            &state,
            &district_map
        ));

        // Cell not in any district
        assert!(!is_cell_in_inclusionary_district(
            100,
            100,
            &state,
            &district_map
        ));
    }

    #[test]
    fn test_affordable_percentage_for_cell() {
        let mut state = InclusionaryZoningState::default();
        let mut district_map = DistrictMap::default();

        district_map.assign_cell_to_district(10, 10, 0);
        state.enable(0);
        state.set_affordable_percentage(0, 0.18);

        let pct = affordable_percentage_for_cell(10, 10, &state, &district_map);
        assert!((pct - 0.18).abs() < f32::EPSILON);

        // Cell not in district
        let pct2 = affordable_percentage_for_cell(100, 100, &state, &district_map);
        assert_eq!(pct2, 0.0);
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let state = InclusionaryZoningState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_active() {
        use crate::Saveable;
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        state.set_affordable_percentage(0, 0.18);
        state.enable(3);
        state.total_affordable_units = 42;
        state.total_affected_units = 200;
        state.total_monthly_cost = 16_000.0;

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = InclusionaryZoningState::load_from_bytes(&bytes);

        assert!(restored.is_enabled(0));
        assert!((restored.affordable_percentage(0) - 0.18).abs() < f32::EPSILON);
        assert!(restored.is_enabled(3));
        assert!(!restored.is_enabled(1));
        assert_eq!(restored.total_affordable_units, 42);
        assert_eq!(restored.total_affected_units, 200);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(InclusionaryZoningState::SAVE_KEY, "inclusionary_zoning");
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_are_reasonable() {
        assert!(MIN_AFFORDABLE_PERCENTAGE > 0.0);
        assert!(MIN_AFFORDABLE_PERCENTAGE < MAX_AFFORDABLE_PERCENTAGE);
        assert!(MAX_AFFORDABLE_PERCENTAGE <= 1.0);
        assert!(DEFAULT_AFFORDABLE_PERCENTAGE >= MIN_AFFORDABLE_PERCENTAGE);
        assert!(DEFAULT_AFFORDABLE_PERCENTAGE <= MAX_AFFORDABLE_PERCENTAGE);
        assert!(MIN_FAR_BONUS > 0.0);
        assert!(MIN_FAR_BONUS <= MAX_FAR_BONUS);
        assert!(CONSTRUCTION_RATE_PENALTY > 0.0);
        assert!(CONSTRUCTION_RATE_PENALTY <= 1.0);
        assert!(MONTHLY_ADMIN_COST_PER_DISTRICT > 0.0);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_enable_disable_cycle() {
        let mut state = InclusionaryZoningState::default();
        state.enable(0);
        assert!(state.is_enabled(0));
        assert_eq!(state.enabled_district_count(), 1);

        state.disable(0);
        assert!(!state.is_enabled(0));
        // Config still exists but is disabled
        assert!(state.district_configs.contains_key(&0));

        // Re-enable preserves custom percentage
        state.set_affordable_percentage(0, 0.18);
        state.enable(0);
        assert!((state.affordable_percentage(0) - 0.18).abs() < f32::EPSILON);
    }

    #[test]
    fn test_far_bonus_scales_with_affordable_percentage() {
        // Higher affordable percentage should give a higher FAR bonus
        let bonus_10 = calculate_far_bonus(0.10);
        let bonus_15 = calculate_far_bonus(0.15);
        let bonus_20 = calculate_far_bonus(0.20);
        assert!(bonus_10 < bonus_15);
        assert!(bonus_15 < bonus_20);
    }

    #[test]
    fn test_effective_capacity_plus_affordable_equals_total() {
        // Effective + affordable should always equal the original capacity
        for capacity in [10, 50, 100, 500, 1000] {
            for pct in [0.10, 0.15, 0.20] {
                let affordable = calculate_affordable_units(capacity, pct);
                let effective = calculate_effective_capacity(capacity, pct);
                assert_eq!(
                    affordable + effective,
                    capacity,
                    "capacity={}, pct={}: affordable={} + effective={} != {}",
                    capacity,
                    pct,
                    affordable,
                    effective,
                    capacity
                );
            }
        }
    }
}
