//! Per-cell LOS grid covering the entire map.
//!
//! Stores a LOS grade for every grid cell, computed from the traffic density
//! relative to road capacity.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

use super::grades::LosGrade;

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

impl crate::Saveable for TrafficLosGrid {
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
