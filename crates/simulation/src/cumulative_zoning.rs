//! Cumulative hierarchy zoning rules (ZONE-004).
//!
//! Implements Euclidean cumulative hierarchy where higher-intensity zones
//! implicitly allow lower-intensity uses. The hierarchy is:
//!
//!   R-1 (ResidentialLow) < R-2 (ResidentialMedium) < R-3 (ResidentialHigh)
//!   < R-4 (MixedUse) < C-1 (CommercialLow) < C-2 (CommercialHigh)
//!   < M-1 (Industrial) < M-2 (Office)
//!
//! When cumulative mode is enabled (policy toggle), a zone permits all uses
//! at or below its hierarchy level. For example, a CommercialHigh zone also
//! permits residential and low-commercial buildings.
//!
//! The building spawner consults this module to determine the effective
//! building type to place, selecting the highest-value permitted use based
//! on current market demand.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::grid::ZoneType;
use crate::zones::ZoneDemand;

// =============================================================================
// Hierarchy levels
// =============================================================================

/// Returns the numeric hierarchy level for a zone type.
///
/// Higher values represent higher-intensity uses. A zone with cumulative
/// zoning enabled permits all uses with a hierarchy level <= its own level.
///
/// Hierarchy:
///   R-1 (ResidentialLow) = 1
///   R-2 (ResidentialMedium) = 2
///   R-3 (ResidentialHigh) = 3
///   R-4 (MixedUse) = 4
///   C-1 (CommercialLow) = 5
///   C-2 (CommercialHigh) = 6
///   M-1 (Industrial) = 7
///   M-2 (Office) = 8
pub fn hierarchy_level(zone: ZoneType) -> u8 {
    match zone {
        ZoneType::ResidentialLow => 1,
        ZoneType::ResidentialMedium => 2,
        ZoneType::ResidentialHigh => 3,
        ZoneType::MixedUse => 4,
        ZoneType::CommercialLow => 5,
        ZoneType::CommercialHigh => 6,
        ZoneType::Industrial => 7,
        ZoneType::Office => 8,
        ZoneType::None => 0,
    }
}

// =============================================================================
// Permission queries
// =============================================================================

/// Returns `true` if `zone` permits residential buildings under cumulative zoning.
///
/// Residential is the lowest-intensity use category (levels 1-3), so any
/// zone at hierarchy level >= 1 permits residential.
pub fn permits_residential(zone: ZoneType) -> bool {
    hierarchy_level(zone) >= 1
}

/// Returns `true` if `zone` permits commercial buildings under cumulative zoning.
///
/// Commercial uses start at hierarchy level 5 (CommercialLow).
/// MixedUse (level 4) also permits commercial as a special case since it
/// inherently combines residential and commercial.
pub fn permits_commercial(zone: ZoneType) -> bool {
    let level = hierarchy_level(zone);
    level >= 5 || zone == ZoneType::MixedUse
}

/// Returns `true` if `zone` permits industrial buildings under cumulative zoning.
///
/// Industrial uses start at hierarchy level 7.
pub fn permits_industrial(zone: ZoneType) -> bool {
    hierarchy_level(zone) >= 7
}

/// Returns all zone types permitted in the given zone under cumulative hierarchy.
///
/// The result is ordered from lowest to highest intensity.
pub fn permitted_zone_types(zone: ZoneType) -> Vec<ZoneType> {
    let level = hierarchy_level(zone);
    let all_zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::MixedUse,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
    ];

    all_zones
        .iter()
        .copied()
        .filter(|z| hierarchy_level(*z) <= level)
        .collect()
}

// =============================================================================
// Cumulative zoning policy state (toggleable)
// =============================================================================

/// City-wide cumulative zoning policy state.
///
/// When `enabled` is `true`, the building spawner uses cumulative hierarchy
/// rules: zones permit all lower-intensity uses and the spawner selects the
/// highest-value permitted use based on market demand.
///
/// When `enabled` is `false` (default), exclusive zoning applies: each zone
/// only permits its own building type.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct CumulativeZoningState {
    /// Whether cumulative zoning is enabled city-wide.
    pub enabled: bool,
}

// =============================================================================
// Effective zone selection
// =============================================================================

/// Given a cell's zone type and current market demand, returns the effective
/// zone type that the building spawner should use when cumulative zoning is
/// enabled.
///
/// The spawner selects the zone type with the highest demand among all
/// permitted uses for the given zone. If no permitted use has positive
/// demand, returns the original zone type.
pub fn select_effective_zone(zone: ZoneType, demand: &ZoneDemand) -> ZoneType {
    let permitted = permitted_zone_types(zone);
    if permitted.is_empty() {
        return zone;
    }

    let mut best_zone = zone;
    let mut best_demand = -1.0_f32;

    for candidate in &permitted {
        let d = demand.demand_for(*candidate);
        if d > best_demand {
            best_demand = d;
            best_zone = *candidate;
        }
    }

    // Only override if the best alternative has meaningful demand
    if best_demand > 0.0 {
        best_zone
    } else {
        zone
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for CumulativeZoningState {
    const SAVE_KEY: &'static str = "cumulative_zoning";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if at default state (disabled)
        if !self.enabled {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        bitcode::decode(bytes).unwrap_or_default()
    }
}

// =============================================================================
// System: sync policy toggle with state resource
// =============================================================================

/// Syncs the `CumulativeZoning` policy toggle from [`Policies`] with the
/// [`CumulativeZoningState`] resource. This allows the policy panel UI to
/// control cumulative zoning via the standard policy toggle mechanism.
fn sync_cumulative_zoning_policy(
    policies: Res<crate::policies::Policies>,
    mut state: ResMut<CumulativeZoningState>,
) {
    let should_be_enabled = policies.is_active(crate::policies::Policy::CumulativeZoning);
    if state.enabled != should_be_enabled {
        state.enabled = should_be_enabled;
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct CumulativeZoningPlugin;

impl Plugin for CumulativeZoningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CumulativeZoningState>()
            .add_systems(FixedUpdate, sync_cumulative_zoning_policy);

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<CumulativeZoningState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Hierarchy level tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_hierarchy_levels_ordered() {
        assert!(
            hierarchy_level(ZoneType::ResidentialLow)
                < hierarchy_level(ZoneType::ResidentialMedium)
        );
        assert!(
            hierarchy_level(ZoneType::ResidentialMedium)
                < hierarchy_level(ZoneType::ResidentialHigh)
        );
        assert!(hierarchy_level(ZoneType::ResidentialHigh) < hierarchy_level(ZoneType::MixedUse));
        assert!(hierarchy_level(ZoneType::MixedUse) < hierarchy_level(ZoneType::CommercialLow));
        assert!(
            hierarchy_level(ZoneType::CommercialLow) < hierarchy_level(ZoneType::CommercialHigh)
        );
        assert!(hierarchy_level(ZoneType::CommercialHigh) < hierarchy_level(ZoneType::Industrial));
        assert!(hierarchy_level(ZoneType::Industrial) < hierarchy_level(ZoneType::Office));
    }

    #[test]
    fn test_hierarchy_none_is_zero() {
        assert_eq!(hierarchy_level(ZoneType::None), 0);
    }

    #[test]
    fn test_hierarchy_all_zones_have_levels() {
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::MixedUse,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
        ];
        for zone in zones {
            assert!(
                hierarchy_level(zone) > 0,
                "{:?} should have level > 0",
                zone
            );
        }
    }

    // -------------------------------------------------------------------------
    // Permission tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_residential_low_permits_only_residential() {
        assert!(permits_residential(ZoneType::ResidentialLow));
        assert!(!permits_commercial(ZoneType::ResidentialLow));
        assert!(!permits_industrial(ZoneType::ResidentialLow));
    }

    #[test]
    fn test_commercial_high_permits_residential_and_commercial() {
        assert!(permits_residential(ZoneType::CommercialHigh));
        assert!(permits_commercial(ZoneType::CommercialHigh));
        assert!(!permits_industrial(ZoneType::CommercialHigh));
    }

    #[test]
    fn test_industrial_permits_all() {
        assert!(permits_residential(ZoneType::Industrial));
        assert!(permits_commercial(ZoneType::Industrial));
        assert!(permits_industrial(ZoneType::Industrial));
    }

    #[test]
    fn test_office_permits_all() {
        assert!(permits_residential(ZoneType::Office));
        assert!(permits_commercial(ZoneType::Office));
        assert!(permits_industrial(ZoneType::Office));
    }

    #[test]
    fn test_mixed_use_permits_residential_and_commercial() {
        assert!(permits_residential(ZoneType::MixedUse));
        assert!(permits_commercial(ZoneType::MixedUse));
        assert!(!permits_industrial(ZoneType::MixedUse));
    }

    #[test]
    fn test_none_permits_nothing() {
        assert!(!permits_residential(ZoneType::None));
        assert!(!permits_commercial(ZoneType::None));
        assert!(!permits_industrial(ZoneType::None));
    }

    // -------------------------------------------------------------------------
    // Permitted zone types tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_permitted_zones_residential_low() {
        let permitted = permitted_zone_types(ZoneType::ResidentialLow);
        assert_eq!(permitted, vec![ZoneType::ResidentialLow]);
    }

    #[test]
    fn test_permitted_zones_residential_high() {
        let permitted = permitted_zone_types(ZoneType::ResidentialHigh);
        assert_eq!(
            permitted,
            vec![
                ZoneType::ResidentialLow,
                ZoneType::ResidentialMedium,
                ZoneType::ResidentialHigh,
            ]
        );
    }

    #[test]
    fn test_permitted_zones_commercial_low() {
        let permitted = permitted_zone_types(ZoneType::CommercialLow);
        assert_eq!(
            permitted,
            vec![
                ZoneType::ResidentialLow,
                ZoneType::ResidentialMedium,
                ZoneType::ResidentialHigh,
                ZoneType::MixedUse,
                ZoneType::CommercialLow,
            ]
        );
    }

    #[test]
    fn test_permitted_zones_office_includes_all() {
        let permitted = permitted_zone_types(ZoneType::Office);
        assert_eq!(permitted.len(), 8);
        assert_eq!(
            permitted,
            vec![
                ZoneType::ResidentialLow,
                ZoneType::ResidentialMedium,
                ZoneType::ResidentialHigh,
                ZoneType::MixedUse,
                ZoneType::CommercialLow,
                ZoneType::CommercialHigh,
                ZoneType::Industrial,
                ZoneType::Office,
            ]
        );
    }

    #[test]
    fn test_permitted_zones_none_is_empty() {
        let permitted = permitted_zone_types(ZoneType::None);
        assert!(permitted.is_empty());
    }

    // -------------------------------------------------------------------------
    // Effective zone selection tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_select_effective_zone_picks_highest_demand() {
        let demand = ZoneDemand {
            residential: 0.3,
            commercial: 0.8,
            industrial: 0.1,
            office: 0.0,
            ..Default::default()
        };

        // CommercialHigh zone permits residential, mixed-use, and commercial.
        // Commercial demand (0.8) > residential demand (0.3).
        // MixedUse demand_for returns max(residential, commercial) = 0.8,
        // which ties with CommercialLow/CommercialHigh. MixedUse appears
        // first in iteration order (hierarchy level 4), so it wins.
        let effective = select_effective_zone(ZoneType::CommercialHigh, &demand);
        assert!(
            effective == ZoneType::MixedUse
                || effective == ZoneType::CommercialLow
                || effective == ZoneType::CommercialHigh,
            "Expected MixedUse or commercial zone, got {:?}",
            effective
        );
    }

    #[test]
    fn test_select_effective_zone_residential_dominant() {
        let demand = ZoneDemand {
            residential: 0.9,
            commercial: 0.2,
            industrial: 0.1,
            office: 0.0,
            ..Default::default()
        };

        // Industrial zone permits everything. Residential has highest demand.
        let effective = select_effective_zone(ZoneType::Industrial, &demand);
        assert!(
            effective.is_residential(),
            "Expected residential zone, got {:?}",
            effective
        );
    }

    #[test]
    fn test_select_effective_zone_exclusive_returns_same() {
        let demand = ZoneDemand {
            residential: 0.5,
            commercial: 0.5,
            industrial: 0.5,
            office: 0.5,
            ..Default::default()
        };

        // ResidentialLow only permits itself, so effective should be same.
        let effective = select_effective_zone(ZoneType::ResidentialLow, &demand);
        assert_eq!(effective, ZoneType::ResidentialLow);
    }

    #[test]
    fn test_select_effective_zone_no_demand_returns_original() {
        let demand = ZoneDemand::default();

        let effective = select_effective_zone(ZoneType::CommercialHigh, &demand);
        assert_eq!(effective, ZoneType::CommercialHigh);
    }

    // -------------------------------------------------------------------------
    // State and policy toggle tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_state_disabled() {
        let state = CumulativeZoningState::default();
        assert!(!state.enabled);
    }

    #[test]
    fn test_state_can_be_enabled() {
        let mut state = CumulativeZoningState::default();
        state.enabled = true;
        assert!(state.enabled);
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let state = CumulativeZoningState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_enabled() {
        use crate::Saveable;
        let mut state = CumulativeZoningState::default();
        state.enabled = true;
        assert!(state.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = CumulativeZoningState::default();
        state.enabled = true;

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = CumulativeZoningState::load_from_bytes(&bytes);

        assert_eq!(restored.enabled, state.enabled);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(CumulativeZoningState::SAVE_KEY, "cumulative_zoning");
    }
}
