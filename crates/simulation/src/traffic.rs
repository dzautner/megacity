use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::citizen::{Citizen, CitizenStateComp, PathCache};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::RoadType;

#[derive(Resource, Serialize, Deserialize)]
pub struct TrafficGrid {
    pub density: Vec<u16>,
    pub width: usize,
    pub height: usize,
}

impl Default for TrafficGrid {
    fn default() -> Self {
        Self {
            density: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl TrafficGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u16 {
        self.density[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u16) {
        self.density[y * self.width + x] = val;
    }

    pub fn congestion_level(&self, x: usize, y: usize) -> f32 {
        let d = self.get(x, y) as f32;
        // 0 = free, 1.0 = fully congested
        (d / 20.0).min(1.0)
    }

    pub fn path_cost(&self, x: usize, y: usize) -> u32 {
        let base = 1u32;
        let congestion_penalty = (self.congestion_level(x, y) * 5.0) as u32;
        base + congestion_penalty
    }

    /// Path cost factoring in road type speed
    pub fn path_cost_with_road(&self, x: usize, y: usize, road_type: RoadType) -> u32 {
        let speed = road_type.speed();
        // Higher speed = lower cost. Normalize: local(30)=base, highway(100)=0.3x
        let speed_factor = 30.0 / speed;
        let base = (speed_factor * 1.0) as u32 + 1;
        let congestion_penalty = (self.congestion_level(x, y) * 5.0) as u32;
        base + congestion_penalty
    }

    pub fn clear(&mut self) {
        self.density.fill(0);
    }

    /// Compute the LOS grade for a cell based on its congestion level.
    pub fn los_grade(&self, x: usize, y: usize) -> LosGrade {
        LosGrade::from_congestion(self.congestion_level(x, y))
    }
}

/// Highway Capacity Manual Level of Service grades (A through F).
/// Each grade represents a range of volume/capacity ratio (congestion level).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LosGrade {
    /// Free flow, minimal delay
    A,
    /// Stable flow, slight delays
    B,
    /// Stable flow, acceptable delays
    C,
    /// Approaching unstable flow
    D,
    /// Unstable flow, significant delays
    E,
    /// Forced/breakdown flow, gridlock
    F,
}

impl LosGrade {
    /// Convert a congestion level (0.0 = free, 1.0 = gridlocked) to a LOS grade.
    pub fn from_congestion(congestion: f32) -> Self {
        if congestion < 0.15 {
            LosGrade::A
        } else if congestion < 0.30 {
            LosGrade::B
        } else if congestion < 0.50 {
            LosGrade::C
        } else if congestion < 0.70 {
            LosGrade::D
        } else if congestion < 0.85 {
            LosGrade::E
        } else {
            LosGrade::F
        }
    }

    /// RGBA color for this LOS grade (green-to-red ramp).
    /// Returns [r, g, b, a] in sRGB space, suitable for vertex colors or UI display.
    pub fn color(self) -> [f32; 4] {
        match self {
            LosGrade::A => [0.20, 0.78, 0.35, 0.85], // green — free flow
            LosGrade::B => [0.55, 0.82, 0.25, 0.85], // yellow-green — stable
            LosGrade::C => [0.92, 0.85, 0.20, 0.85], // yellow — acceptable
            LosGrade::D => [0.95, 0.60, 0.15, 0.85], // orange — approaching unstable
            LosGrade::E => [0.92, 0.30, 0.12, 0.85], // red-orange — unstable
            LosGrade::F => [0.78, 0.10, 0.10, 0.85], // red — gridlock
        }
    }

    /// Short label for display (e.g. in legends).
    pub fn label(self) -> &'static str {
        match self {
            LosGrade::A => "A - Free Flow",
            LosGrade::B => "B - Stable",
            LosGrade::C => "C - Acceptable",
            LosGrade::D => "D - Near Unstable",
            LosGrade::E => "E - Unstable",
            LosGrade::F => "F - Gridlock",
        }
    }

    /// Single letter for compact display.
    pub fn letter(self) -> char {
        match self {
            LosGrade::A => 'A',
            LosGrade::B => 'B',
            LosGrade::C => 'C',
            LosGrade::D => 'D',
            LosGrade::E => 'E',
            LosGrade::F => 'F',
        }
    }

    /// All grades in order from best to worst.
    pub fn all() -> [LosGrade; 6] {
        [
            LosGrade::A,
            LosGrade::B,
            LosGrade::C,
            LosGrade::D,
            LosGrade::E,
            LosGrade::F,
        ]
    }

    /// Interpolate LOS color smoothly for a given congestion level (0.0 to 1.0).
    /// This produces smooth gradients between the discrete LOS grade colors.
    pub fn interpolated_color(congestion: f32) -> [f32; 4] {
        let c = congestion.clamp(0.0, 1.0);
        // Define breakpoints matching the grade thresholds
        let breakpoints: [(f32, [f32; 4]); 6] = [
            (0.0, LosGrade::A.color()),
            (0.15, LosGrade::B.color()),
            (0.30, LosGrade::C.color()),
            (0.50, LosGrade::D.color()),
            (0.70, LosGrade::E.color()),
            (1.0, LosGrade::F.color()),
        ];

        // Find the two breakpoints to interpolate between
        for i in 0..breakpoints.len() - 1 {
            let (t0, c0) = breakpoints[i];
            let (t1, c1) = breakpoints[i + 1];
            if c <= t1 {
                let frac = if (t1 - t0).abs() < 1e-6 {
                    0.0
                } else {
                    (c - t0) / (t1 - t0)
                };
                return [
                    c0[0] + (c1[0] - c0[0]) * frac,
                    c0[1] + (c1[1] - c0[1]) * frac,
                    c0[2] + (c1[2] - c0[2]) * frac,
                    c0[3] + (c1[3] - c0[3]) * frac,
                ];
            }
        }
        LosGrade::F.color()
    }
}

pub fn update_traffic_density(
    tick: Res<crate::TickCounter>,
    mut traffic: ResMut<TrafficGrid>,
    citizens: Query<(&CitizenStateComp, &PathCache), With<Citizen>>,
) {
    if !tick.0.is_multiple_of(5) {
        return;
    }
    traffic.clear();

    for (state, path) in &citizens {
        if !state.0.is_commuting() {
            continue;
        }

        // Mark current path segment on traffic grid
        if let Some(target) = path.current_target() {
            let x = target.0.min(GRID_WIDTH - 1);
            let y = target.1.min(GRID_HEIGHT - 1);
            let current = traffic.get(x, y);
            traffic.set(x, y, current.saturating_add(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_density_math() {
        let mut traffic = TrafficGrid::default();
        traffic.set(10, 10, 10);
        assert_eq!(traffic.get(10, 10), 10);
        assert!((traffic.congestion_level(10, 10) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_congestion_bounds() {
        let mut traffic = TrafficGrid::default();
        traffic.set(5, 5, 100);
        assert!(traffic.congestion_level(5, 5) <= 1.0);

        traffic.set(5, 5, 0);
        assert_eq!(traffic.congestion_level(5, 5), 0.0);
    }

    #[test]
    fn test_path_cost_increases_with_congestion() {
        let mut traffic = TrafficGrid::default();
        let cost_empty = traffic.path_cost(10, 10);

        traffic.set(10, 10, 20);
        let cost_congested = traffic.path_cost(10, 10);

        assert!(cost_congested > cost_empty);
    }

    #[test]
    fn test_los_grade_from_congestion() {
        assert_eq!(LosGrade::from_congestion(0.0), LosGrade::A);
        assert_eq!(LosGrade::from_congestion(0.10), LosGrade::A);
        assert_eq!(LosGrade::from_congestion(0.20), LosGrade::B);
        assert_eq!(LosGrade::from_congestion(0.35), LosGrade::C);
        assert_eq!(LosGrade::from_congestion(0.55), LosGrade::D);
        assert_eq!(LosGrade::from_congestion(0.75), LosGrade::E);
        assert_eq!(LosGrade::from_congestion(0.90), LosGrade::F);
        assert_eq!(LosGrade::from_congestion(1.0), LosGrade::F);
    }

    #[test]
    fn test_los_grade_colors_are_valid() {
        for grade in LosGrade::all() {
            let c = grade.color();
            for &v in &c {
                assert!(
                    v >= 0.0 && v <= 1.0,
                    "color component out of range for {:?}",
                    grade
                );
            }
        }
    }

    #[test]
    fn test_los_interpolated_color_endpoints() {
        let c_min = LosGrade::interpolated_color(0.0);
        let c_max = LosGrade::interpolated_color(1.0);
        // At 0.0, should match grade A
        let a = LosGrade::A.color();
        for i in 0..4 {
            assert!((c_min[i] - a[i]).abs() < 0.01);
        }
        // At 1.0, should match grade F
        let f = LosGrade::F.color();
        for i in 0..4 {
            assert!((c_max[i] - f[i]).abs() < 0.01);
        }
    }

    #[test]
    fn test_traffic_grid_los_grade() {
        let mut traffic = TrafficGrid::default();
        traffic.set(10, 10, 0);
        assert_eq!(traffic.los_grade(10, 10), LosGrade::A);

        traffic.set(10, 10, 20); // congestion = 1.0
        assert_eq!(traffic.los_grade(10, 10), LosGrade::F);
    }
}
