//! Bulldoze Refund System
//!
//! When a player bulldozes roads, service buildings, or utility buildings,
//! they receive a partial refund of the original placement cost. This prevents
//! an economic soft-lock where players cannot recover from bankruptcy caused
//! by high maintenance costs, since they can now "downsize" by bulldozing
//! expensive infrastructure and recouping some of the investment.

use bevy::prelude::*;

use crate::grid::RoadType;
use crate::services::{ServiceBuilding, ServiceType};
use crate::utilities::UtilityType;

/// Fraction of original placement cost refunded when bulldozing (50%).
pub const REFUND_RATE: f64 = 0.5;

/// Compute the refund amount for bulldozing a road cell of the given type.
pub fn refund_for_road(road_type: RoadType) -> f64 {
    road_type.cost() * REFUND_RATE
}

/// Compute the refund amount for bulldozing a service building.
pub fn refund_for_service(service_type: ServiceType) -> f64 {
    ServiceBuilding::cost(service_type) * REFUND_RATE
}

/// Compute the refund amount for bulldozing a utility source.
pub fn refund_for_utility(utility_type: UtilityType) -> f64 {
    crate::services::utility_cost(utility_type) * REFUND_RATE
}

pub struct BulldozeRefundPlugin;

impl Plugin for BulldozeRefundPlugin {
    fn build(&self, _app: &mut App) {
        // Refund logic is applied directly in the bulldoze input handlers
        // (rendering crate) and batch bulldoze (ui crate). This plugin exists
        // to follow the per-feature plugin convention and to house the refund
        // constants and helpers.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_road_refund_is_half_cost() {
        for road_type in [
            RoadType::Local,
            RoadType::Avenue,
            RoadType::Boulevard,
            RoadType::Highway,
            RoadType::OneWay,
            RoadType::Path,
        ] {
            let refund = refund_for_road(road_type);
            let expected = road_type.cost() * REFUND_RATE;
            assert!(
                (refund - expected).abs() < 0.001,
                "Road {:?} refund {refund} != expected {expected}",
                road_type
            );
        }
    }

    #[test]
    fn test_service_refund_is_half_cost() {
        let refund = refund_for_service(ServiceType::Hospital);
        let expected = ServiceBuilding::cost(ServiceType::Hospital) * REFUND_RATE;
        assert!(
            (refund - expected).abs() < 0.001,
            "Hospital refund {refund} != expected {expected}"
        );
    }

    #[test]
    fn test_utility_refund_is_half_cost() {
        let refund = refund_for_utility(UtilityType::PowerPlant);
        let expected = crate::services::utility_cost(UtilityType::PowerPlant) * REFUND_RATE;
        assert!(
            (refund - expected).abs() < 0.001,
            "PowerPlant refund {refund} != expected {expected}"
        );
    }

    #[test]
    fn test_refund_rate_is_positive_and_less_than_one() {
        assert!(REFUND_RATE > 0.0);
        assert!(REFUND_RATE < 1.0);
    }
}
