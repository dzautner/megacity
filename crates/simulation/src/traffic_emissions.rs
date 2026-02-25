//! POLL-015: Road Traffic Vehicle Emission Pollution Source
//!
//! Implements traffic-based air pollution where each road cell emits pollution
//! proportional to its traffic volume and road type. Higher-capacity roads
//! produce more emissions at baseline, and actual output scales with traffic.
//!
//! ## Emission formula
//!
//! `Q_road = base_Q * traffic_scaling_factor`
//!
//! ### Base Q by road type
//!
//! | Road Type   | Base Q | Mapping            |
//! |-------------|--------|--------------------|
//! | Highway     | 8.0    | Highway            |
//! | Boulevard   | 4.0    | Arterial equivalent|
//! | Avenue      | 2.0    | Collector equivalent|
//! | Local       | 1.0    | Local road         |
//! | OneWay      | 1.0    | Local equivalent   |
//! | Path        | 0.0    | No vehicle traffic |
//!
//! ### Traffic scaling factors
//!
//! | Utilization (traffic / capacity) | Factor |
//! |----------------------------------|--------|
//! | 0.0 (empty)                      | 0.1    |
//! | < 0.5 (moderate)                 | 0.5    |
//! | < 1.0 (congested)                | 1.0    |
//! | >= 1.0 (over-capacity)           | 1.2    |

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid};
use crate::pollution::PollutionGrid;
use crate::traffic::TrafficGrid;
use crate::SlowTickTimer;

// =============================================================================
// Base emission rates by road type
// =============================================================================

/// Returns the base emission Q value for a given road type.
///
/// Higher-capacity roads produce more baseline vehicle emissions.
pub fn base_emission_q(road_type: RoadType) -> f32 {
    match road_type {
        RoadType::Highway => 8.0,
        RoadType::Boulevard => 4.0,
        RoadType::Avenue => 2.0,
        RoadType::Local => 1.0,
        RoadType::OneWay => 1.0,
        RoadType::Path => 0.0,
    }
}

// =============================================================================
// Traffic scaling
// =============================================================================

/// Computes the traffic scaling factor based on the ratio of traffic volume
/// to road capacity.
///
/// - Empty road (ratio == 0): 0.1x (idle/dust emissions)
/// - Moderate (ratio < 0.5): 0.5x
/// - Congested (ratio < 1.0): 1.0x
/// - Over-capacity (ratio >= 1.0): 1.2x
pub fn traffic_scaling_factor(traffic_volume: u16, road_type: RoadType) -> f32 {
    let capacity = road_type.capacity();
    if capacity == 0 || traffic_volume == 0 {
        return 0.1;
    }

    let ratio = traffic_volume as f32 / capacity as f32;

    if ratio >= 1.0 {
        1.2
    } else if ratio >= 0.5 {
        1.0
    } else {
        0.5
    }
}

/// Computes the full traffic emission Q for a road cell.
///
/// `Q_road = base_Q * traffic_scaling_factor`
pub fn traffic_emission_q(road_type: RoadType, traffic_volume: u16) -> f32 {
    let base = base_emission_q(road_type);
    if base <= 0.0 {
        return 0.0;
    }
    base * traffic_scaling_factor(traffic_volume, road_type)
}

// =============================================================================
// System
// =============================================================================

/// Adds traffic-based emission pollution to the pollution grid.
///
/// This system runs on the slow tick and iterates all road cells, computing
/// traffic-scaled emissions based on road type and current traffic volume.
/// The computed values are added (saturating) to the existing pollution grid.
pub fn apply_traffic_emissions(
    slow_timer: Res<SlowTickTimer>,
    grid: Res<WorldGrid>,
    traffic: Res<TrafficGrid>,
    mut pollution: ResMut<PollutionGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Road {
                continue;
            }

            let q = traffic_emission_q(cell.road_type, traffic.get(x, y));
            if q <= 0.0 {
                continue;
            }

            // Saturating add to pollution grid
            let current = pollution.get(x, y);
            let addition = (q as u8).max(1);
            pollution.set(x, y, current.saturating_add(addition));
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct TrafficEmissionsPlugin;

impl Plugin for TrafficEmissionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            apply_traffic_emissions
                .after(crate::traffic::update_traffic_density)
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
    fn test_base_emission_q_by_road_type() {
        assert_eq!(base_emission_q(RoadType::Highway), 8.0);
        assert_eq!(base_emission_q(RoadType::Boulevard), 4.0);
        assert_eq!(base_emission_q(RoadType::Avenue), 2.0);
        assert_eq!(base_emission_q(RoadType::Local), 1.0);
        assert_eq!(base_emission_q(RoadType::OneWay), 1.0);
        assert_eq!(base_emission_q(RoadType::Path), 0.0);
    }

    #[test]
    fn test_traffic_scaling_empty_road() {
        // Zero traffic -> 0.1x
        let factor = traffic_scaling_factor(0, RoadType::Local);
        assert!((factor - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_traffic_scaling_moderate() {
        // Local capacity=20, traffic=5 -> ratio=0.25 -> moderate=0.5x
        let factor = traffic_scaling_factor(5, RoadType::Local);
        assert!((factor - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_traffic_scaling_congested() {
        // Local capacity=20, traffic=15 -> ratio=0.75 -> congested=1.0x
        let factor = traffic_scaling_factor(15, RoadType::Local);
        assert!((factor - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_traffic_scaling_over_capacity() {
        // Local capacity=20, traffic=25 -> ratio=1.25 -> over-capacity=1.2x
        let factor = traffic_scaling_factor(25, RoadType::Local);
        assert!((factor - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_traffic_scaling_path_always_idle() {
        // Path capacity=5, but base Q is 0 so it doesn't matter
        let factor = traffic_scaling_factor(0, RoadType::Path);
        assert!((factor - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_traffic_emission_q_highway_full() {
        // Highway at capacity: 8.0 * 1.0 = 8.0
        let q = traffic_emission_q(RoadType::Highway, 60);
        assert!((q - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_traffic_emission_q_highway_over_capacity() {
        // Highway over capacity: 8.0 * 1.2 = 9.6
        let q = traffic_emission_q(RoadType::Highway, 100);
        assert!((q - 9.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_traffic_emission_q_local_empty() {
        // Local empty: 1.0 * 0.1 = 0.1
        let q = traffic_emission_q(RoadType::Local, 0);
        assert!((q - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_traffic_emission_q_path_is_zero() {
        // Path has base Q=0, so any traffic produces 0
        let q = traffic_emission_q(RoadType::Path, 10);
        assert!((q - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_highway_full_vs_local_empty() {
        let highway_full = traffic_emission_q(RoadType::Highway, 60);
        let local_empty = traffic_emission_q(RoadType::Local, 0);
        // Highway at full traffic should emit much more than empty local
        assert!(
            highway_full > local_empty * 8.0,
            "Highway at full traffic ({highway_full}) should emit far more \
             than empty local ({local_empty})"
        );
    }

    #[test]
    fn test_emission_scales_with_traffic() {
        let q_empty = traffic_emission_q(RoadType::Avenue, 0);
        let q_moderate = traffic_emission_q(RoadType::Avenue, 10);
        let q_congested = traffic_emission_q(RoadType::Avenue, 30);
        let q_over = traffic_emission_q(RoadType::Avenue, 50);

        assert!(q_moderate > q_empty, "Moderate > empty");
        assert!(q_congested > q_moderate, "Congested > moderate");
        assert!(q_over > q_congested, "Over-capacity > congested");
    }

    #[test]
    fn test_all_road_types_have_defined_base_q() {
        let types = [
            RoadType::Highway,
            RoadType::Boulevard,
            RoadType::Avenue,
            RoadType::Local,
            RoadType::OneWay,
            RoadType::Path,
        ];
        for rt in types {
            let q = base_emission_q(rt);
            assert!(q >= 0.0, "{rt:?} should have non-negative base_q, got {q}");
        }
    }
}
