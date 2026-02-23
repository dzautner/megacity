//! Per-segment LOS tracking and city-wide distribution statistics.
//!
//! `TrafficLosState` tracks the LOS grade for each road segment (by SegmentId),
//! computed from the average V/C ratio across the segment's rasterized cells.
//! `LosDistribution` summarizes city-wide LOS counts/percentages.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use std::collections::HashMap;

use crate::road_segments::SegmentId;

use super::grades::LosGrade;

/// Per-segment LOS grade tracking.
#[derive(Resource, Default, Encode, Decode)]
pub struct TrafficLosState {
    /// LOS grade for each road segment, keyed by segment id.
    pub segment_grades: HashMap<u32, u8>,
}

impl TrafficLosState {
    /// Get the LOS grade for a segment.
    pub fn get(&self, id: SegmentId) -> LosGrade {
        match self.segment_grades.get(&id.0).copied().unwrap_or(0) {
            0 => LosGrade::A,
            1 => LosGrade::B,
            2 => LosGrade::C,
            3 => LosGrade::D,
            4 => LosGrade::E,
            _ => LosGrade::F,
        }
    }

    /// Set the LOS grade for a segment.
    pub fn set(&mut self, id: SegmentId, grade: LosGrade) {
        self.segment_grades.insert(id.0, grade as u8);
    }

    /// Remove a segment (e.g. after bulldozing).
    pub fn remove(&mut self, id: SegmentId) {
        self.segment_grades.remove(&id.0);
    }

    /// Number of segments being tracked.
    pub fn segment_count(&self) -> usize {
        self.segment_grades.len()
    }
}

impl crate::Saveable for TrafficLosState {
    const SAVE_KEY: &'static str = "traffic_los_state";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.segment_grades.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

/// City-wide LOS distribution statistics.
#[derive(Resource, Default, Clone, Debug)]
pub struct LosDistribution {
    /// Count of segments at each LOS grade.
    pub counts: [u32; 6],
    /// Total number of graded segments.
    pub total: u32,
}

impl LosDistribution {
    /// Percentage of segments at the given grade (0.0-100.0).
    pub fn percentage(&self, grade: LosGrade) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        self.counts[grade as usize] as f32 / self.total as f32 * 100.0
    }

    /// Weighted average grade (0.0=all A, 5.0=all F).
    pub fn weighted_average(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        let sum: u32 = self
            .counts
            .iter()
            .enumerate()
            .map(|(i, &c)| i as u32 * c)
            .sum();
        sum as f32 / self.total as f32
    }

    /// Percentage of segments at LOS D, E, or F (congested).
    pub fn congested_percentage(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        let congested = self.counts[3] + self.counts[4] + self.counts[5];
        congested as f32 / self.total as f32 * 100.0
    }

    /// Recompute from the current `TrafficLosState`.
    pub fn recompute(&mut self, state: &TrafficLosState) {
        self.counts = [0; 6];
        self.total = state.segment_grades.len() as u32;
        for &raw_grade in state.segment_grades.values() {
            let idx = (raw_grade as usize).min(5);
            self.counts[idx] += 1;
        }
    }
}
