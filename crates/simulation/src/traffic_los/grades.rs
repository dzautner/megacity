//! Level of Service grade enum and V/C ratio classification.
//!
//! Grades follow the Highway Capacity Manual (HCM) convention:
//! - A: Free flow (V/C < 0.35)
//! - B: Stable flow (V/C 0.35-0.55)
//! - C: Stable flow, some restriction (V/C 0.55-0.77)
//! - D: Approaching unstable (V/C 0.77-0.93)
//! - E: Unstable flow (V/C 0.93-1.00)
//! - F: Forced flow / breakdown (V/C >= 1.00)

use bitcode::{Decode, Encode};

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
        } else if vc < 0.93 {
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
