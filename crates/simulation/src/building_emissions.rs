//! POLL-002: Per-Building-Type Air Pollution Emission Rates
//!
//! Provides emission profile lookup tables for each building and source type.
//! These profiles are consumed by the wind-aware Gaussian plume system
//! (`wind_pollution`) to generate distinct pollution contributions.
//!
//! | Source              | Base Q | Notes                        |
//! |---------------------|--------|------------------------------|
//! | Coal Power Plant    | 100    | (handled via PowerPlant)     |
//! | Gas Power Plant     | 35     | (handled via PowerPlant)     |
//! | Industrial (zoned)  | varies | 5 + level*3                  |
//! | Incinerator         | 20     | waste-burning service        |
//! | Heating Boiler      | 10     | small fossil fuel heater     |
//! | District Heating    | 15     | larger fossil-fuel plant     |
//! | Crematorium         | 5      | low combustion               |
//! | Commercial (zoned)  | 1      | low area-source (per cell)   |
//! | Residential heating | 1      | very low (per cell)          |
//! | Roads               | scaled | traffic-proportional         |
//! | Solar / Wind        | 0      | zero emissions               |
//! | Office / MixedUse   | 0      | negligible                   |

use crate::grid::ZoneType;
use crate::services::ServiceType;

// =============================================================================
// Emission profile
// =============================================================================

/// Describes the air pollution emission characteristics of a single source.
#[derive(Debug, Clone, Copy)]
pub struct EmissionProfile {
    /// Base emission strength (Q value). Higher = more pollution.
    pub base_q: f32,
    /// Source category for policy multiplier selection.
    pub category: SourceCategory,
}

/// Broad source categories that policies can target independently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceCategory {
    /// Industrial zones and heavy industry
    Industrial,
    /// Commercial zones (HVAC, delivery trucks)
    Commercial,
    /// Residential heating (furnaces, boilers)
    Residential,
    /// Road traffic exhaust
    Traffic,
    /// Service buildings that burn fuel (incinerators, heating boilers)
    ServiceCombustion,
    /// Clean energy sources (solar, wind) — always Q=0
    Clean,
}

// =============================================================================
// Emission lookup tables
// =============================================================================

/// Returns the emission profile for a zoned building based on zone type and
/// level. Returns `None` for zone types with negligible direct air pollution.
pub fn building_emission_profile(zone: ZoneType, level: u8) -> Option<EmissionProfile> {
    match zone {
        ZoneType::Industrial => Some(EmissionProfile {
            base_q: 5.0 + level as f32 * 3.0,
            category: SourceCategory::Industrial,
        }),
        ZoneType::CommercialLow | ZoneType::CommercialHigh => Some(EmissionProfile {
            base_q: 1.0,
            category: SourceCategory::Commercial,
        }),
        ZoneType::ResidentialLow | ZoneType::ResidentialMedium | ZoneType::ResidentialHigh => {
            Some(EmissionProfile {
                base_q: 1.0,
                category: SourceCategory::Residential,
            })
        }
        // Office and MixedUse have negligible direct air pollution
        _ => None,
    }
}

/// Returns the emission profile for a service building. Only combustion-
/// related services emit; parks, schools, etc. return `None`.
pub fn service_emission_profile(service_type: ServiceType) -> Option<EmissionProfile> {
    match service_type {
        ServiceType::Incinerator => Some(EmissionProfile {
            base_q: 20.0,
            category: SourceCategory::ServiceCombustion,
        }),
        ServiceType::HeatingBoiler => Some(EmissionProfile {
            base_q: 10.0,
            category: SourceCategory::ServiceCombustion,
        }),
        ServiceType::DistrictHeatingPlant => Some(EmissionProfile {
            base_q: 15.0,
            category: SourceCategory::ServiceCombustion,
        }),
        ServiceType::Crematorium => Some(EmissionProfile {
            base_q: 5.0,
            category: SourceCategory::ServiceCombustion,
        }),
        // Geothermal is clean — no combustion emissions
        _ => None,
    }
}

/// Returns the policy multiplier for a given source category.
/// `IndustrialAirFilters` policy reduces industrial and combustion-service
/// emissions by 40%.
pub fn category_multiplier(
    category: SourceCategory,
    policies: &crate::policies::Policies,
) -> f32 {
    match category {
        SourceCategory::Industrial | SourceCategory::ServiceCombustion => {
            policies.pollution_multiplier()
        }
        SourceCategory::Commercial
        | SourceCategory::Residential
        | SourceCategory::Traffic
        | SourceCategory::Clean => 1.0,
    }
}

/// Base road emission Q value.
pub const ROAD_BASE_Q: f32 = 2.0;

/// Computes traffic-scaled road emission Q for a cell.
///
/// `Q_road = ROAD_BASE_Q * (0.2 + 0.8 * congestion_level)`
///
/// Roads with zero traffic still emit a small amount (dust, idling), and
/// emission scales up to the full base Q at maximum congestion.
pub fn road_emission_q(congestion_level: f32) -> f32 {
    ROAD_BASE_Q * (0.2 + 0.8 * congestion_level)
}

// =============================================================================
// Plugin (empty — profiles are consumed by wind_pollution)
// =============================================================================

use bevy::prelude::*;

/// Plugin that makes building emission profiles available. The actual
/// emission systems live in [`wind_pollution`] which consumes these profiles.
pub struct BuildingEmissionsPlugin;

impl Plugin for BuildingEmissionsPlugin {
    fn build(&self, _app: &mut App) {
        // No resources or systems — this module is a pure data library
        // consumed by wind_pollution::collect_sources.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_industrial_emission_profile_scales_with_level() {
        let l1 = building_emission_profile(ZoneType::Industrial, 1).unwrap();
        let l3 = building_emission_profile(ZoneType::Industrial, 3).unwrap();
        assert!(l3.base_q > l1.base_q, "L3 should emit more than L1");
        assert_eq!(l1.category, SourceCategory::Industrial);
    }

    #[test]
    fn test_commercial_emission_is_low() {
        let profile = building_emission_profile(ZoneType::CommercialHigh, 1).unwrap();
        assert!(profile.base_q <= 2.0, "Commercial Q should be low");
        assert_eq!(profile.category, SourceCategory::Commercial);
    }

    #[test]
    fn test_residential_emission_is_very_low() {
        let profile = building_emission_profile(ZoneType::ResidentialLow, 1).unwrap();
        assert!(profile.base_q <= 1.0, "Residential Q should be very low");
        assert_eq!(profile.category, SourceCategory::Residential);
    }

    #[test]
    fn test_office_has_no_emission() {
        assert!(building_emission_profile(ZoneType::Office, 1).is_none());
    }

    #[test]
    fn test_incinerator_emits() {
        let profile = service_emission_profile(ServiceType::Incinerator).unwrap();
        assert_eq!(profile.base_q, 20.0);
        assert_eq!(profile.category, SourceCategory::ServiceCombustion);
    }

    #[test]
    fn test_geothermal_is_clean() {
        assert!(service_emission_profile(ServiceType::GeothermalPlant).is_none());
    }

    #[test]
    fn test_park_has_no_emission() {
        assert!(service_emission_profile(ServiceType::SmallPark).is_none());
    }

    #[test]
    fn test_road_emission_scales_with_congestion() {
        let empty = road_emission_q(0.0);
        let half = road_emission_q(0.5);
        let full = road_emission_q(1.0);
        assert!(full > half, "full={full} > half={half}");
        assert!(half > empty, "half={half} > empty={empty}");
        assert!((full - ROAD_BASE_Q).abs() < 0.01);
    }

    #[test]
    fn test_emission_profiles_cover_all_industrial_levels() {
        for level in 1..=5u8 {
            let profile = building_emission_profile(ZoneType::Industrial, level);
            assert!(
                profile.is_some(),
                "Industrial L{level} should have an emission profile"
            );
        }
    }
}
