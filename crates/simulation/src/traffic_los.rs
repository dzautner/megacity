//! Traffic Level of Service (LOS) grading system.
//!
//! Computes LOS grades A through F for each road cell based on traffic density
//! relative to road capacity. LOS A represents free flow, while LOS F represents
//! gridlock conditions.
//!
//! The grades follow the Highway Capacity Manual (HCM) convention:
//! - A: Free flow (v/c ratio < 0.35)
//! - B: Stable flow (v/c ratio < 0.55)
//! - C: Stable flow, some restriction (v/c ratio < 0.77)
//! - D: Approaching unstable (v/c ratio < 0.90)
//! - E: Unstable flow (v/c ratio < 1.00)
//! - F: Forced flow / breakdown (v/c ratio >= 1.00)

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::traffic::TrafficGrid;
use crate::Saveable;

/// Level of Service grade from A (best) to F (worst).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Encode, Decode)]
#[repr(u8)]
pub enum LosGrade {
    #[default]
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
}

impl LosGrade {
    /// Convert a volume-to-capacity ratio to a LOS grade.
    pub fn from_vc_ratio(vc: f32) -> Self {
        if vc < 0.35 {
            LosGrade::A
        } else if vc < 0.55 {
            LosGrade::B
        } else if vc < 0.77 {
            LosGrade::C
        } else if vc < 0.90 {
            LosGrade::D
        } else if vc < 1.00 {
            LosGrade::E
        } else {
            LosGrade::F
        }
    }

    /// Return a normalized 0.0..1.0 value for use in color ramps.
    /// A=0.0, F=1.0.
    pub fn as_t(self) -> f32 {
        self as u8 as f32 / 5.0
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            LosGrade::A => "LOS A (Free Flow)",
            LosGrade::B => "LOS B (Stable Flow)",
            LosGrade::C => "LOS C (Restricted Flow)",
            LosGrade::D => "LOS D (Approaching Unstable)",
            LosGrade::E => "LOS E (Unstable Flow)",
            LosGrade::F => "LOS F (Breakdown)",
        }
    }

    /// RGBA color for overlay rendering.
    /// Green (A) -> Yellow (C) -> Red (F).
    pub fn color(self) -> [f32; 4] {
        match self {
            LosGrade::A => [0.0, 0.8, 0.0, 0.6], // green
            LosGrade::B => [0.4, 0.8, 0.0, 0.6], // yellow-green
            LosGrade::C => [0.8, 0.8, 0.0, 0.6], // yellow
            LosGrade::D => [1.0, 0.5, 0.0, 0.6], // orange
            LosGrade::E => [1.0, 0.2, 0.0, 0.6], // red-orange
            LosGrade::F => [0.8, 0.0, 0.0, 0.6], // red
        }
    }

    /// Single-character grade letter.
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
}

/// Per-cell LOS grade grid covering the entire map.
#[derive(Resource, Encode, Decode)]
pub struct TrafficLosGrid {
    pub grades: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for TrafficLosGrid {
    fn default() -> Self {
        Self {
            grades: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl TrafficLosGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> LosGrade {
        let raw = self.grades[y * self.width + x];
        match raw {
            0 => LosGrade::A,
            1 => LosGrade::B,
            2 => LosGrade::C,
            3 => LosGrade::D,
            4 => LosGrade::E,
            _ => LosGrade::F,
        }
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, grade: LosGrade) {
        self.grades[y * self.width + x] = grade as u8;
    }

    /// Return the LOS as a normalized float 0.0..1.0 for color mapping.
    #[inline]
    pub fn get_t(&self, x: usize, y: usize) -> f32 {
        self.get(x, y).as_t()
    }
}

impl Saveable for TrafficLosGrid {
    const SAVE_KEY: &'static str = "traffic_los";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if all grades are A (default)
        if self.grades.iter().all(|&g| g == 0) {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

/// System that computes LOS grades for all road cells.
/// Runs every 10 ticks, after traffic density is updated.
pub fn update_traffic_los(
    tick: Res<crate::TickCounter>,
    grid: Res<WorldGrid>,
    traffic: Res<TrafficGrid>,
    mut los_grid: ResMut<TrafficLosGrid>,
) {
    // Run every 10 ticks (aligned with traffic updates which run every 5)
    if !tick.0.is_multiple_of(10) {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Road {
                // Non-road cells default to A
                los_grid.set(x, y, LosGrade::A);
                continue;
            }

            let density = traffic.get(x, y) as f32;
            let capacity = cell.road_type.capacity() as f32;
            let vc_ratio = density / capacity;
            let grade = LosGrade::from_vc_ratio(vc_ratio);
            los_grid.set(x, y, grade);
        }
    }
}

pub struct TrafficLosPlugin;

impl Plugin for TrafficLosPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrafficLosGrid>().add_systems(
            FixedUpdate,
            update_traffic_los
                .after(crate::traffic::update_traffic_density)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<TrafficLosGrid>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::RoadType;

    #[test]
    fn test_los_grade_from_vc_ratio() {
        // LOS A: v/c < 0.35
        assert_eq!(LosGrade::from_vc_ratio(0.0), LosGrade::A);
        assert_eq!(LosGrade::from_vc_ratio(0.20), LosGrade::A);
        assert_eq!(LosGrade::from_vc_ratio(0.34), LosGrade::A);
        // LOS B: v/c < 0.55
        assert_eq!(LosGrade::from_vc_ratio(0.35), LosGrade::B);
        assert_eq!(LosGrade::from_vc_ratio(0.54), LosGrade::B);
        // LOS C: v/c < 0.77
        assert_eq!(LosGrade::from_vc_ratio(0.55), LosGrade::C);
        assert_eq!(LosGrade::from_vc_ratio(0.76), LosGrade::C);
        // LOS D: v/c < 0.90
        assert_eq!(LosGrade::from_vc_ratio(0.77), LosGrade::D);
        assert_eq!(LosGrade::from_vc_ratio(0.89), LosGrade::D);
        // LOS E: v/c < 1.00
        assert_eq!(LosGrade::from_vc_ratio(0.90), LosGrade::E);
        assert_eq!(LosGrade::from_vc_ratio(0.99), LosGrade::E);
        // LOS F: v/c >= 1.00
        assert_eq!(LosGrade::from_vc_ratio(1.00), LosGrade::F);
        assert_eq!(LosGrade::from_vc_ratio(2.50), LosGrade::F);
    }

    #[test]
    fn test_los_grade_as_t() {
        assert!((LosGrade::A.as_t() - 0.0).abs() < f32::EPSILON);
        assert!((LosGrade::B.as_t() - 0.2).abs() < f32::EPSILON);
        assert!((LosGrade::C.as_t() - 0.4).abs() < f32::EPSILON);
        assert!((LosGrade::D.as_t() - 0.6).abs() < f32::EPSILON);
        assert!((LosGrade::E.as_t() - 0.8).abs() < f32::EPSILON);
        assert!((LosGrade::F.as_t() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_los_grid_default() {
        let grid = TrafficLosGrid::default();
        assert_eq!(grid.grades.len(), GRID_WIDTH * GRID_HEIGHT);
        for y in 0..3 {
            for x in 0..3 {
                assert_eq!(grid.get(x, y), LosGrade::A);
            }
        }
    }

    #[test]
    fn test_los_grid_set_get() {
        let mut grid = TrafficLosGrid::default();
        grid.set(5, 5, LosGrade::C);
        assert_eq!(grid.get(5, 5), LosGrade::C);
        grid.set(5, 5, LosGrade::F);
        assert_eq!(grid.get(5, 5), LosGrade::F);
    }

    #[test]
    fn test_los_grid_get_t() {
        let mut grid = TrafficLosGrid::default();
        grid.set(0, 0, LosGrade::A);
        assert!((grid.get_t(0, 0) - 0.0).abs() < f32::EPSILON);
        grid.set(0, 0, LosGrade::F);
        assert!((grid.get_t(0, 0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_road_capacity_ordering() {
        // Higher road types should have more capacity
        assert!(RoadType::Path.capacity() < RoadType::Local.capacity());
        assert!(RoadType::Local.capacity() < RoadType::Avenue.capacity());
        assert!(RoadType::Avenue.capacity() < RoadType::Boulevard.capacity());
        assert!(RoadType::Boulevard.capacity() < RoadType::Highway.capacity());
    }

    #[test]
    fn test_los_label_non_empty() {
        let grades = [
            LosGrade::A,
            LosGrade::B,
            LosGrade::C,
            LosGrade::D,
            LosGrade::E,
            LosGrade::F,
        ];
        for grade in &grades {
            assert!(!grade.label().is_empty());
        }
    }

    #[test]
    fn test_los_color_has_alpha() {
        let grades = [
            LosGrade::A,
            LosGrade::B,
            LosGrade::C,
            LosGrade::D,
            LosGrade::E,
            LosGrade::F,
        ];
        for grade in &grades {
            let color = grade.color();
            assert!(color[3] > 0.0, "Alpha should be > 0 for {grade:?}");
        }
    }

    #[test]
    fn test_los_letter() {
        assert_eq!(LosGrade::A.letter(), 'A');
        assert_eq!(LosGrade::F.letter(), 'F');
    }

    #[test]
    fn test_saveable_skip_default() {
        let grid = TrafficLosGrid::default();
        assert!(
            grid.save_to_bytes().is_none(),
            "Default state should skip saving"
        );
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut grid = TrafficLosGrid::default();
        grid.set(10, 10, LosGrade::D);
        grid.set(20, 20, LosGrade::F);

        let bytes = grid.save_to_bytes().expect("Non-default should save");
        let restored = TrafficLosGrid::load_from_bytes(&bytes);

        assert_eq!(restored.get(10, 10), LosGrade::D);
        assert_eq!(restored.get(20, 20), LosGrade::F);
        assert_eq!(restored.get(0, 0), LosGrade::A);
    }

    #[test]
    fn test_highway_needs_more_traffic_for_congestion() {
        // A highway with 15 vehicles should have better LOS than a local road
        // (highway: 15/80=0.19 -> A, local: 15/20=0.75 -> C)
        let highway_vc = 15.0 / RoadType::Highway.capacity() as f32;
        let local_vc = 15.0 / RoadType::Local.capacity() as f32;

        let highway_grade = LosGrade::from_vc_ratio(highway_vc);
        let local_grade = LosGrade::from_vc_ratio(local_vc);

        assert!(
            (highway_grade as u8) < (local_grade as u8),
            "Highway should have better LOS than local road with same traffic"
        );
    }

    #[test]
    fn test_boundary_thresholds_match_spec() {
        // Verify exact boundary values from the TRAF-002 spec
        assert_eq!(LosGrade::from_vc_ratio(0.349), LosGrade::A);
        assert_eq!(LosGrade::from_vc_ratio(0.351), LosGrade::B);
        assert_eq!(LosGrade::from_vc_ratio(0.549), LosGrade::B);
        assert_eq!(LosGrade::from_vc_ratio(0.551), LosGrade::C);
        assert_eq!(LosGrade::from_vc_ratio(0.769), LosGrade::C);
        assert_eq!(LosGrade::from_vc_ratio(0.771), LosGrade::D);
        assert_eq!(LosGrade::from_vc_ratio(0.899), LosGrade::D);
        assert_eq!(LosGrade::from_vc_ratio(0.901), LosGrade::E);
        assert_eq!(LosGrade::from_vc_ratio(0.999), LosGrade::E);
        assert_eq!(LosGrade::from_vc_ratio(1.001), LosGrade::F);
    }
}
