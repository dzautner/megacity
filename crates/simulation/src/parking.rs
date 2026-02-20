//! Parking minimum/maximum system (ZONE-011).
//!
//! Implements parking requirements as a zoning control. Parking minimums
//! consume land and increase construction costs, while parking maximums
//! encourage transit usage.
//!
//! ## Per-zone parking ratios
//! - Residential: 1-2 spaces per unit (low density=1, medium=1.5, high=2)
//! - Commercial: 1 space per 300 sqft (~3.3 per 1000 sqft)
//! - Industrial: 1 space per 500 sqft (~2.0 per 1000 sqft)
//! - Office: 1 space per 400 sqft (~2.5 per 1000 sqft)
//! - MixedUse: weighted average of residential + commercial ratios
//!
//! ## Cost impact
//! Each required parking space adds $5K-$20K to effective building cost
//! depending on zone type (surface lots are cheaper, structured parking
//! in dense areas is expensive).
//!
//! ## Policies
//! - **Eliminate parking minimums**: removes minimum requirements, reduces
//!   construction cost, increases transit dependency.
//! - **Parking maximum**: caps parking to a fraction of the minimum ratio,
//!   encouraging transit use and reducing land consumed by parking.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::grid::{WorldGrid, ZoneType};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Cost per required parking space for low-density zones (surface lots).
pub const PARKING_COST_LOW_DENSITY: f64 = 5_000.0;

/// Cost per required parking space for medium-density zones.
pub const PARKING_COST_MEDIUM_DENSITY: f64 = 10_000.0;

/// Cost per required parking space for high-density zones (structured parking).
pub const PARKING_COST_HIGH_DENSITY: f64 = 20_000.0;

/// Cost per required parking space for industrial zones.
pub const PARKING_COST_INDUSTRIAL: f64 = 5_000.0;

/// Cost per required parking space for office zones.
pub const PARKING_COST_OFFICE: f64 = 15_000.0;

/// Parking maximum cap as a fraction of the minimum ratio (e.g., 0.5 means
/// maximum is half the minimum requirement).
pub const PARKING_MAXIMUM_FRACTION: f32 = 0.5;

// =============================================================================
// Parking ratios per zone type
// =============================================================================

/// Returns the parking spaces required per unit/1000sqft for a given zone type.
/// - Residential Low: 1.0 per unit
/// - Residential Medium: 1.5 per unit
/// - Residential High: 2.0 per unit
/// - Commercial Low: 3.3 per 1000 sqft (1 per 300 sqft)
/// - Commercial High: 3.3 per 1000 sqft
/// - Industrial: 2.0 per 1000 sqft (1 per 500 sqft)
/// - Office: 2.5 per 1000 sqft (1 per 400 sqft)
/// - MixedUse: 2.5 (weighted average of residential and commercial)
pub fn parking_ratio(zone: ZoneType) -> f32 {
    match zone {
        ZoneType::ResidentialLow => 1.0,
        ZoneType::ResidentialMedium => 1.5,
        ZoneType::ResidentialHigh => 2.0,
        ZoneType::CommercialLow => 3.3,
        ZoneType::CommercialHigh => 3.3,
        ZoneType::Industrial => 2.0,
        ZoneType::Office => 2.5,
        ZoneType::MixedUse => 2.5,
        ZoneType::None => 0.0,
    }
}

/// Returns the cost per required parking space for a given zone type.
pub fn parking_cost_per_space(zone: ZoneType) -> f64 {
    match zone {
        ZoneType::ResidentialLow => PARKING_COST_LOW_DENSITY,
        ZoneType::ResidentialMedium => PARKING_COST_MEDIUM_DENSITY,
        ZoneType::ResidentialHigh => PARKING_COST_HIGH_DENSITY,
        ZoneType::CommercialLow => PARKING_COST_LOW_DENSITY,
        ZoneType::CommercialHigh => PARKING_COST_HIGH_DENSITY,
        ZoneType::Industrial => PARKING_COST_INDUSTRIAL,
        ZoneType::Office => PARKING_COST_OFFICE,
        ZoneType::MixedUse => PARKING_COST_HIGH_DENSITY,
        ZoneType::None => 0.0,
    }
}

// =============================================================================
// Resource: parking policy state
// =============================================================================

/// City-wide parking policy configuration.
///
/// Controls whether parking minimums are enforced and whether a parking
/// maximum cap is in effect. These policies affect building construction
/// costs and transit dependency.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct ParkingPolicyState {
    /// When true, parking minimums are eliminated (no required parking).
    /// Reduces construction costs but increases transit dependency.
    pub eliminate_minimums: bool,
    /// When true, a parking maximum is enforced (caps parking at a fraction
    /// of the minimum ratio). Encourages transit use.
    pub parking_maximum: bool,
}

// =============================================================================
// Resource: computed parking effects
// =============================================================================

/// Computed effects of parking policies, updated each slow tick.
///
/// Other simulation systems read this resource to determine parking-related
/// cost adjustments and transit dependency impacts.
#[derive(Resource, Debug, Clone, Default)]
pub struct ParkingEffects {
    /// Total required parking spaces across all buildings.
    pub total_required_spaces: u32,
    /// Total additional construction cost from parking requirements.
    pub total_parking_cost: f64,
    /// Average effective parking ratio (accounting for policy modifications).
    pub effective_ratio_multiplier: f32,
    /// Whether parking minimums are currently eliminated.
    pub minimums_eliminated: bool,
    /// Whether parking maximum is currently active.
    pub maximum_active: bool,
    /// Number of buildings affected by parking requirements.
    pub buildings_affected: u32,
}

// =============================================================================
// Pure computation functions
// =============================================================================

/// Calculate the effective parking ratio for a zone type given current policy.
///
/// - Default: returns the full parking_ratio for the zone.
/// - Eliminate minimums: returns 0 (no required parking).
/// - Parking maximum: caps ratio at `PARKING_MAXIMUM_FRACTION` of the minimum.
/// - Both: eliminate minimums takes precedence (ratio = 0).
pub fn effective_parking_ratio(zone: ZoneType, state: &ParkingPolicyState) -> f32 {
    if state.eliminate_minimums {
        return 0.0;
    }
    let base = parking_ratio(zone);
    if state.parking_maximum {
        base * PARKING_MAXIMUM_FRACTION
    } else {
        base
    }
}

/// Calculate the number of required parking spaces for a building.
///
/// For residential zones, the ratio is spaces-per-unit (capacity = units).
/// For commercial/industrial/office, we approximate units from capacity
/// (each capacity unit ~ 1 person ~ some sqft equivalent).
pub fn required_parking_spaces(zone: ZoneType, capacity: u32, state: &ParkingPolicyState) -> u32 {
    let ratio = effective_parking_ratio(zone, state);
    if ratio <= 0.0 {
        return 0;
    }
    // For all zone types, required spaces = capacity * ratio (rounded up)
    (capacity as f32 * ratio).ceil() as u32
}

/// Calculate the additional construction cost from parking requirements
/// for a single building.
pub fn parking_construction_cost(zone: ZoneType, capacity: u32, state: &ParkingPolicyState) -> f64 {
    let spaces = required_parking_spaces(zone, capacity, state);
    spaces as f64 * parking_cost_per_space(zone)
}

/// Calculate the ratio multiplier for the current policy state.
/// Returns 0.0 if minimums eliminated, 0.5 if maximum active, 1.0 otherwise.
pub fn ratio_multiplier(state: &ParkingPolicyState) -> f32 {
    if state.eliminate_minimums {
        0.0
    } else if state.parking_maximum {
        PARKING_MAXIMUM_FRACTION
    } else {
        1.0
    }
}

// =============================================================================
// System
// =============================================================================

/// System: update parking effects every slow tick.
///
/// Iterates all buildings and computes total required parking spaces and
/// aggregate construction cost impact from parking requirements.
pub fn update_parking_effects(
    timer: Res<SlowTickTimer>,
    grid: Res<WorldGrid>,
    buildings: Query<&Building>,
    state: Res<ParkingPolicyState>,
    mut effects: ResMut<ParkingEffects>,
) {
    if !timer.should_run() {
        return;
    }

    let mut total_spaces = 0u32;
    let mut total_cost = 0.0f64;
    let mut buildings_affected = 0u32;

    for cell in &grid.cells {
        if let Some(entity) = cell.building_id {
            if let Ok(building) = buildings.get(entity) {
                let spaces = required_parking_spaces(building.zone_type, building.capacity, &state);
                let cost = parking_construction_cost(building.zone_type, building.capacity, &state);

                if spaces > 0 {
                    buildings_affected += 1;
                }

                total_spaces += spaces;
                total_cost += cost;
            }
        }
    }

    effects.total_required_spaces = total_spaces;
    effects.total_parking_cost = total_cost;
    effects.effective_ratio_multiplier = ratio_multiplier(&state);
    effects.minimums_eliminated = state.eliminate_minimums;
    effects.maximum_active = state.parking_maximum;
    effects.buildings_affected = buildings_affected;
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for ParkingPolicyState {
    const SAVE_KEY: &'static str = "parking_policy";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if at default state (no policies active)
        if !self.eliminate_minimums && !self.parking_maximum {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct ParkingPlugin;

impl Plugin for ParkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParkingPolicyState>()
            .init_resource::<ParkingEffects>()
            .add_systems(FixedUpdate, update_parking_effects);

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<ParkingPolicyState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Parking ratio tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_residential_low_ratio() {
        assert!((parking_ratio(ZoneType::ResidentialLow) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_residential_medium_ratio() {
        assert!((parking_ratio(ZoneType::ResidentialMedium) - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_residential_high_ratio() {
        assert!((parking_ratio(ZoneType::ResidentialHigh) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_commercial_low_ratio() {
        assert!((parking_ratio(ZoneType::CommercialLow) - 3.3).abs() < 0.01);
    }

    #[test]
    fn test_commercial_high_ratio() {
        assert!((parking_ratio(ZoneType::CommercialHigh) - 3.3).abs() < 0.01);
    }

    #[test]
    fn test_industrial_ratio() {
        assert!((parking_ratio(ZoneType::Industrial) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_office_ratio() {
        assert!((parking_ratio(ZoneType::Office) - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mixed_use_ratio() {
        assert!((parking_ratio(ZoneType::MixedUse) - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_none_zone_ratio() {
        assert!((parking_ratio(ZoneType::None)).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Parking cost per space tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_residential_low_cost() {
        assert!((parking_cost_per_space(ZoneType::ResidentialLow) - 5_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_residential_high_cost() {
        assert!(
            (parking_cost_per_space(ZoneType::ResidentialHigh) - 20_000.0).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_industrial_cost() {
        assert!((parking_cost_per_space(ZoneType::Industrial) - 5_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_office_cost() {
        assert!((parking_cost_per_space(ZoneType::Office) - 15_000.0).abs() < f64::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Effective parking ratio tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_policy_full_ratio() {
        let state = ParkingPolicyState::default();
        let ratio = effective_parking_ratio(ZoneType::ResidentialLow, &state);
        assert!((ratio - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_eliminate_minimums_zero_ratio() {
        let state = ParkingPolicyState {
            eliminate_minimums: true,
            parking_maximum: false,
        };
        let ratio = effective_parking_ratio(ZoneType::ResidentialHigh, &state);
        assert!(ratio.abs() < f32::EPSILON);
    }

    #[test]
    fn test_parking_maximum_halves_ratio() {
        let state = ParkingPolicyState {
            eliminate_minimums: false,
            parking_maximum: true,
        };
        let ratio = effective_parking_ratio(ZoneType::ResidentialHigh, &state);
        // 2.0 * 0.5 = 1.0
        assert!((ratio - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_eliminate_minimums_overrides_maximum() {
        let state = ParkingPolicyState {
            eliminate_minimums: true,
            parking_maximum: true,
        };
        let ratio = effective_parking_ratio(ZoneType::CommercialHigh, &state);
        assert!(ratio.abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Required parking spaces tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_required_spaces_residential_low() {
        let state = ParkingPolicyState::default();
        // 10 units * 1.0 ratio = 10 spaces
        let spaces = required_parking_spaces(ZoneType::ResidentialLow, 10, &state);
        assert_eq!(spaces, 10);
    }

    #[test]
    fn test_required_spaces_residential_high() {
        let state = ParkingPolicyState::default();
        // 50 units * 2.0 ratio = 100 spaces
        let spaces = required_parking_spaces(ZoneType::ResidentialHigh, 50, &state);
        assert_eq!(spaces, 100);
    }

    #[test]
    fn test_required_spaces_commercial() {
        let state = ParkingPolicyState::default();
        // 30 units * 3.3 ratio = 99 spaces
        let spaces = required_parking_spaces(ZoneType::CommercialHigh, 30, &state);
        assert_eq!(spaces, 99);
    }

    #[test]
    fn test_required_spaces_industrial() {
        let state = ParkingPolicyState::default();
        // 20 units * 2.0 ratio = 40 spaces
        let spaces = required_parking_spaces(ZoneType::Industrial, 20, &state);
        assert_eq!(spaces, 40);
    }

    #[test]
    fn test_required_spaces_zero_with_eliminated_minimums() {
        let state = ParkingPolicyState {
            eliminate_minimums: true,
            parking_maximum: false,
        };
        let spaces = required_parking_spaces(ZoneType::ResidentialHigh, 100, &state);
        assert_eq!(spaces, 0);
    }

    #[test]
    fn test_required_spaces_reduced_with_maximum() {
        let state_default = ParkingPolicyState::default();
        let state_max = ParkingPolicyState {
            eliminate_minimums: false,
            parking_maximum: true,
        };
        let spaces_default = required_parking_spaces(ZoneType::Industrial, 20, &state_default);
        let spaces_max = required_parking_spaces(ZoneType::Industrial, 20, &state_max);
        assert!(spaces_max < spaces_default);
        // 20 * 2.0 * 0.5 = 20 (ceiling)
        assert_eq!(spaces_max, 20);
    }

    #[test]
    fn test_required_spaces_zero_capacity() {
        let state = ParkingPolicyState::default();
        let spaces = required_parking_spaces(ZoneType::ResidentialLow, 0, &state);
        assert_eq!(spaces, 0);
    }

    #[test]
    fn test_required_spaces_none_zone() {
        let state = ParkingPolicyState::default();
        let spaces = required_parking_spaces(ZoneType::None, 100, &state);
        assert_eq!(spaces, 0);
    }

    // -------------------------------------------------------------------------
    // Parking construction cost tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parking_cost_residential_low() {
        let state = ParkingPolicyState::default();
        // 10 units * 1.0 ratio = 10 spaces * $5K = $50K
        let cost = parking_construction_cost(ZoneType::ResidentialLow, 10, &state);
        assert!((cost - 50_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parking_cost_residential_high() {
        let state = ParkingPolicyState::default();
        // 50 units * 2.0 ratio = 100 spaces * $20K = $2M
        let cost = parking_construction_cost(ZoneType::ResidentialHigh, 50, &state);
        assert!((cost - 2_000_000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parking_cost_zero_with_eliminated_minimums() {
        let state = ParkingPolicyState {
            eliminate_minimums: true,
            parking_maximum: false,
        };
        let cost = parking_construction_cost(ZoneType::ResidentialHigh, 100, &state);
        assert!(cost.abs() < f64::EPSILON);
    }

    #[test]
    fn test_parking_cost_reduced_with_maximum() {
        let state_default = ParkingPolicyState::default();
        let state_max = ParkingPolicyState {
            eliminate_minimums: false,
            parking_maximum: true,
        };
        let cost_default = parking_construction_cost(ZoneType::ResidentialHigh, 50, &state_default);
        let cost_max = parking_construction_cost(ZoneType::ResidentialHigh, 50, &state_max);
        assert!(cost_max < cost_default);
    }

    // -------------------------------------------------------------------------
    // Ratio multiplier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_ratio_multiplier_default() {
        let state = ParkingPolicyState::default();
        assert!((ratio_multiplier(&state) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ratio_multiplier_eliminated() {
        let state = ParkingPolicyState {
            eliminate_minimums: true,
            parking_maximum: false,
        };
        assert!(ratio_multiplier(&state).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ratio_multiplier_maximum() {
        let state = ParkingPolicyState {
            eliminate_minimums: false,
            parking_maximum: true,
        };
        assert!((ratio_multiplier(&state) - PARKING_MAXIMUM_FRACTION).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Default state tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_policy_state() {
        let state = ParkingPolicyState::default();
        assert!(!state.eliminate_minimums);
        assert!(!state.parking_maximum);
    }

    #[test]
    fn test_default_effects() {
        let effects = ParkingEffects::default();
        assert_eq!(effects.total_required_spaces, 0);
        assert!(effects.total_parking_cost.abs() < f64::EPSILON);
        assert!(effects.effective_ratio_multiplier.abs() < f32::EPSILON);
        assert!(!effects.minimums_eliminated);
        assert!(!effects.maximum_active);
        assert_eq!(effects.buildings_affected, 0);
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let state = ParkingPolicyState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_active() {
        use crate::Saveable;
        let state = ParkingPolicyState {
            eliminate_minimums: true,
            parking_maximum: false,
        };
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let state = ParkingPolicyState {
            eliminate_minimums: true,
            parking_maximum: true,
        };
        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = ParkingPolicyState::load_from_bytes(&bytes);
        assert_eq!(restored.eliminate_minimums, state.eliminate_minimums);
        assert_eq!(restored.parking_maximum, state.parking_maximum);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(ParkingPolicyState::SAVE_KEY, "parking_policy");
    }

    // -------------------------------------------------------------------------
    // Constant verification tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constant_values() {
        assert_eq!(PARKING_COST_LOW_DENSITY, 5_000.0);
        assert_eq!(PARKING_COST_MEDIUM_DENSITY, 10_000.0);
        assert_eq!(PARKING_COST_HIGH_DENSITY, 20_000.0);
        assert_eq!(PARKING_COST_INDUSTRIAL, 5_000.0);
        assert_eq!(PARKING_COST_OFFICE, 15_000.0);
        assert_eq!(PARKING_MAXIMUM_FRACTION, 0.5);
    }

    // -------------------------------------------------------------------------
    // Integration-style tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_all_zones_have_parking_ratios() {
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zone in zones {
            assert!(
                parking_ratio(zone) > 0.0,
                "Zone {:?} should have a positive parking ratio",
                zone
            );
        }
    }

    #[test]
    fn test_all_zones_have_parking_costs() {
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zone in zones {
            assert!(
                parking_cost_per_space(zone) > 0.0,
                "Zone {:?} should have a positive parking cost",
                zone
            );
        }
    }

    #[test]
    fn test_cost_in_valid_range() {
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zone in zones {
            let cost = parking_cost_per_space(zone);
            assert!(
                (5_000.0..=20_000.0).contains(&cost),
                "Zone {:?} parking cost ${} should be between $5K and $20K",
                zone,
                cost
            );
        }
    }

    #[test]
    fn test_high_density_more_expensive_than_low() {
        assert!(
            parking_cost_per_space(ZoneType::ResidentialHigh)
                > parking_cost_per_space(ZoneType::ResidentialLow)
        );
    }

    #[test]
    fn test_eliminate_minimums_removes_all_costs() {
        let state = ParkingPolicyState {
            eliminate_minimums: true,
            parking_maximum: false,
        };
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zone in zones {
            let cost = parking_construction_cost(zone, 100, &state);
            assert!(
                cost.abs() < f64::EPSILON,
                "Zone {:?} should have zero parking cost with minimums eliminated",
                zone
            );
        }
    }

    #[test]
    fn test_parking_maximum_reduces_but_nonzero() {
        let state = ParkingPolicyState {
            eliminate_minimums: false,
            parking_maximum: true,
        };
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zone in zones {
            let cost_max = parking_construction_cost(zone, 100, &state);
            let cost_default = parking_construction_cost(zone, 100, &ParkingPolicyState::default());
            assert!(
                cost_max < cost_default,
                "Zone {:?} parking cost with maximum (${}) should be less than default (${})",
                zone,
                cost_max,
                cost_default
            );
            assert!(
                cost_max > 0.0,
                "Zone {:?} parking cost with maximum should still be positive",
                zone
            );
        }
    }

    #[test]
    fn test_residential_ratios_in_range() {
        // Issue spec: Residential 1-2 per unit
        assert!(parking_ratio(ZoneType::ResidentialLow) >= 1.0);
        assert!(parking_ratio(ZoneType::ResidentialLow) <= 2.0);
        assert!(parking_ratio(ZoneType::ResidentialMedium) >= 1.0);
        assert!(parking_ratio(ZoneType::ResidentialMedium) <= 2.0);
        assert!(parking_ratio(ZoneType::ResidentialHigh) >= 1.0);
        assert!(parking_ratio(ZoneType::ResidentialHigh) <= 2.0);
    }
}
