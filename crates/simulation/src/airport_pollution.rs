//! POLL-016: Airport Air Pollution Sources
//!
//! Airports emit area-source air pollution covering their building footprint.
//! Each airport tier emits Q=25.0 per footprint cell, added to the pollution
//! grid after the main Gaussian plume pass.
//!
//! | Source              | Base Q | Footprint |
//! |---------------------|--------|-----------|
//! | Small Airstrip      | 25.0   | 3×3       |
//! | Regional Airport    | 25.0   | 4×3       |
//! | International Airport | 25.0 | 4×4       |
//!
//! Seaport is not yet implemented as a building type, so only airport
//! pollution is included here.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::pollution::PollutionGrid;
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Base emission Q value for airport buildings (per footprint cell).
pub const AIRPORT_BASE_Q: f32 = 25.0;

// =============================================================================
// Helper
// =============================================================================

/// Returns true if the given service type is an airport.
fn is_airport(service_type: ServiceType) -> bool {
    matches!(
        service_type,
        ServiceType::SmallAirstrip
            | ServiceType::RegionalAirport
            | ServiceType::InternationalAirport
    )
}

/// Returns the emission Q for an airport service type.
/// Non-airport types return 0.
pub fn airport_emission_q(service_type: ServiceType) -> f32 {
    if is_airport(service_type) {
        AIRPORT_BASE_Q
    } else {
        0.0
    }
}

// =============================================================================
// System
// =============================================================================

/// Adds airport area-source pollution to the pollution grid.
///
/// For each airport service building, this system iterates over the footprint
/// cells and adds Q=25.0 (saturating) to each cell's pollution level.
/// Runs on the slow tick after the main wind pollution system.
pub fn apply_airport_pollution(
    slow_timer: Res<SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    mut pollution: ResMut<PollutionGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for service in &services {
        if !is_airport(service.service_type) {
            continue;
        }

        let q = airport_emission_q(service.service_type);
        if q <= 0.0 {
            continue;
        }

        let (fw, fh) = ServiceBuilding::footprint(service.service_type);
        let addition = (q as u8).max(1);

        for fy in 0..fh {
            for fx in 0..fw {
                let gx = service.grid_x + fx;
                let gy = service.grid_y + fy;
                if gx < GRID_WIDTH && gy < GRID_HEIGHT {
                    let current = pollution.get(gx, gy);
                    pollution.set(gx, gy, current.saturating_add(addition));
                }
            }
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct AirportPollutionPlugin;

impl Plugin for AirportPollutionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            apply_airport_pollution
                .after(crate::wind_pollution::update_pollution_gaussian_plume)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airport_emission_q_for_airports() {
        assert_eq!(airport_emission_q(ServiceType::SmallAirstrip), 25.0);
        assert_eq!(airport_emission_q(ServiceType::RegionalAirport), 25.0);
        assert_eq!(
            airport_emission_q(ServiceType::InternationalAirport),
            25.0
        );
    }

    #[test]
    fn test_airport_emission_q_non_airport_is_zero() {
        assert_eq!(airport_emission_q(ServiceType::SmallPark), 0.0);
        assert_eq!(airport_emission_q(ServiceType::Hospital), 0.0);
        assert_eq!(airport_emission_q(ServiceType::BusDepot), 0.0);
    }

    #[test]
    fn test_is_airport_identifies_all_tiers() {
        assert!(is_airport(ServiceType::SmallAirstrip));
        assert!(is_airport(ServiceType::RegionalAirport));
        assert!(is_airport(ServiceType::InternationalAirport));
    }

    #[test]
    fn test_is_airport_rejects_non_airports() {
        assert!(!is_airport(ServiceType::FireStation));
        assert!(!is_airport(ServiceType::TrainStation));
        assert!(!is_airport(ServiceType::LargePark));
    }

    #[test]
    fn test_airport_footprints_are_multi_cell() {
        let (w, h) = ServiceBuilding::footprint(ServiceType::SmallAirstrip);
        assert!(w * h > 1, "Small airstrip should have multi-cell footprint");

        let (w, h) = ServiceBuilding::footprint(ServiceType::RegionalAirport);
        assert!(
            w * h > 1,
            "Regional airport should have multi-cell footprint"
        );

        let (w, h) = ServiceBuilding::footprint(ServiceType::InternationalAirport);
        assert!(
            w * h > 1,
            "International airport should have multi-cell footprint"
        );
    }
}
