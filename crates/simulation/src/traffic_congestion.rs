//! Density-based traffic congestion simulation.
//!
//! Tracks per-cell speed multipliers derived from the ratio of current traffic
//! volume to road capacity. When a road cell's occupancy approaches its
//! capacity, citizens on that cell slow down, creating visible congestion.
//!
//! Speed formula: `multiplier = (1.0 - occupancy_ratio²).max(MIN_SPEED_MULTIPLIER)`
//!
//! The movement system reads these multipliers to scale citizen speed each tick.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::traffic::TrafficGrid;

/// Minimum speed multiplier so citizens never fully stop (avoids deadlocks).
const MIN_SPEED_MULTIPLIER: f32 = 0.1;

/// Per-cell speed multipliers derived from traffic density vs road capacity.
#[derive(Resource)]
pub struct TrafficCongestion {
    /// Speed multiplier per cell in [MIN_SPEED_MULTIPLIER, 1.0].
    /// 1.0 = free flow, MIN_SPEED_MULTIPLIER = near-gridlock.
    pub speed_multipliers: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl Default for TrafficCongestion {
    fn default() -> Self {
        Self {
            speed_multipliers: vec![1.0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl TrafficCongestion {
    /// Get the speed multiplier for a cell. Returns 1.0 for out-of-bounds.
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.speed_multipliers[y * self.width + x]
        } else {
            1.0
        }
    }

    /// Set the speed multiplier for a cell.
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        if x < self.width && y < self.height {
            self.speed_multipliers[y * self.width + x] = val;
        }
    }
}

/// Compute the congestion speed multiplier from an occupancy ratio.
///
/// `occupancy_ratio` = current_volume / capacity, clamped to [0, inf).
/// Returns a value in [MIN_SPEED_MULTIPLIER, 1.0].
///
/// Formula: `(1.0 - ratio²).max(MIN_SPEED_MULTIPLIER)`
/// This gives a smooth quadratic slowdown that becomes severe near capacity.
#[inline]
pub fn congestion_speed_multiplier(occupancy_ratio: f32) -> f32 {
    let ratio_sq = occupancy_ratio * occupancy_ratio;
    (1.0 - ratio_sq).max(MIN_SPEED_MULTIPLIER)
}

/// System: recompute per-cell speed multipliers from traffic density and road capacity.
///
/// Runs every 5 ticks (same cadence as `update_traffic_density`) to stay in sync.
pub fn update_congestion_multipliers(
    tick: Res<crate::TickCounter>,
    traffic: Res<TrafficGrid>,
    grid: Res<WorldGrid>,
    mut congestion: ResMut<TrafficCongestion>,
) {
    // Sync with traffic density update cadence
    if !tick.0.is_multiple_of(5) {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Road {
                // Non-road cells: free flow (no congestion)
                congestion.speed_multipliers[y * GRID_WIDTH + x] = 1.0;
                continue;
            }

            let volume = traffic.get(x, y) as f32;
            let capacity = cell.road_type.capacity() as f32;

            if capacity <= 0.0 || volume <= 0.0 {
                congestion.speed_multipliers[y * GRID_WIDTH + x] = 1.0;
                continue;
            }

            let ratio = volume / capacity;
            congestion.speed_multipliers[y * GRID_WIDTH + x] = congestion_speed_multiplier(ratio);
        }
    }
}

pub struct TrafficCongestionPlugin;

impl Plugin for TrafficCongestionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrafficCongestion>().add_systems(
            FixedUpdate,
            // Reads TrafficGrid; must run after traffic density is computed.
            // Only writes TrafficCongestion (private resource).
            update_congestion_multipliers
                .after(crate::traffic::update_traffic_density)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_congestion_speed_multiplier_zero_traffic() {
        // No traffic = full speed
        let m = congestion_speed_multiplier(0.0);
        assert!((m - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_congestion_speed_multiplier_half_capacity() {
        // At 50% capacity: 1.0 - 0.25 = 0.75
        let m = congestion_speed_multiplier(0.5);
        assert!((m - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_congestion_speed_multiplier_at_capacity() {
        // At 100% capacity: 1.0 - 1.0 = 0.0 -> clamped to MIN
        let m = congestion_speed_multiplier(1.0);
        assert!((m - MIN_SPEED_MULTIPLIER).abs() < f32::EPSILON);
    }

    #[test]
    fn test_congestion_speed_multiplier_over_capacity() {
        // Over capacity should still clamp to MIN
        let m = congestion_speed_multiplier(1.5);
        assert!((m - MIN_SPEED_MULTIPLIER).abs() < f32::EPSILON);
    }

    #[test]
    fn test_congestion_speed_multiplier_monotonically_decreasing() {
        let ratios = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        for pair in ratios.windows(2) {
            let m1 = congestion_speed_multiplier(pair[0]);
            let m2 = congestion_speed_multiplier(pair[1]);
            assert!(
                m1 >= m2,
                "multiplier should decrease: ratio {} -> {}, but {} < {}",
                pair[0],
                pair[1],
                m1,
                m2
            );
        }
    }

    #[test]
    fn test_congestion_resource_default() {
        let c = TrafficCongestion::default();
        assert_eq!(c.speed_multipliers.len(), GRID_WIDTH * GRID_HEIGHT);
        // All cells should start at 1.0 (free flow)
        assert!(c
            .speed_multipliers
            .iter()
            .all(|&v| (v - 1.0).abs() < f32::EPSILON));
    }

    #[test]
    fn test_congestion_get_out_of_bounds() {
        let c = TrafficCongestion::default();
        // Out-of-bounds should return 1.0
        assert!((c.get(999, 999) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_congestion_set_and_get() {
        let mut c = TrafficCongestion::default();
        c.set(10, 20, 0.5);
        assert!((c.get(10, 20) - 0.5).abs() < f32::EPSILON);
    }
}
