//! Types and constants for the energy dashboard.

use bevy::prelude::*;

/// Resource controlling whether the energy dashboard window is visible.
#[derive(Resource, Default)]
pub struct EnergyDashboardVisible(pub bool);

/// Number of history samples to keep (24 game-hours at 1 sample per game-hour).
pub const HISTORY_CAPACITY: usize = 24;

/// Ring buffer storing demand and supply history over the last 24 game-hours.
#[derive(Resource, Debug, Clone)]
pub struct EnergyHistory {
    /// Demand samples in MW (oldest first when reading sequentially).
    pub demand: Vec<f32>,
    /// Supply samples in MW (oldest first when reading sequentially).
    pub supply: Vec<f32>,
    /// Write index into the ring buffer.
    pub write_idx: usize,
    /// Total number of samples written (may exceed capacity).
    pub sample_count: usize,
    /// Last game-hour at which a sample was recorded.
    pub last_recorded_hour: u32,
}

impl Default for EnergyHistory {
    fn default() -> Self {
        Self {
            demand: vec![0.0; HISTORY_CAPACITY],
            supply: vec![0.0; HISTORY_CAPACITY],
            write_idx: 0,
            sample_count: 0,
            last_recorded_hour: u32::MAX,
        }
    }
}

impl EnergyHistory {
    /// Record a new demand/supply sample.
    pub fn push(&mut self, demand: f32, supply: f32) {
        self.demand[self.write_idx] = demand;
        self.supply[self.write_idx] = supply;
        self.write_idx = (self.write_idx + 1) % HISTORY_CAPACITY;
        self.sample_count += 1;
    }

    /// Returns samples in chronological order (oldest first).
    pub fn ordered_demand(&self) -> Vec<f32> {
        self.ordered_samples(&self.demand)
    }

    /// Returns samples in chronological order (oldest first).
    pub fn ordered_supply(&self) -> Vec<f32> {
        self.ordered_samples(&self.supply)
    }

    /// Number of valid samples (capped at HISTORY_CAPACITY).
    pub fn valid_count(&self) -> usize {
        self.sample_count.min(HISTORY_CAPACITY)
    }

    fn ordered_samples(&self, buf: &[f32]) -> Vec<f32> {
        let count = self.valid_count();
        if count < HISTORY_CAPACITY {
            // Buffer not yet full: samples are in order from index 0.
            buf[..count].to_vec()
        } else {
            // Buffer full: oldest sample is at write_idx.
            let mut result = Vec::with_capacity(HISTORY_CAPACITY);
            result.extend_from_slice(&buf[self.write_idx..]);
            result.extend_from_slice(&buf[..self.write_idx]);
            result
        }
    }
}

/// Aggregated generation mix data by plant type for display.
#[derive(Debug, Default, Clone)]
pub struct GenerationMix {
    pub coal_mw: f32,
    pub gas_mw: f32,
    pub wind_mw: f32,
    pub battery_mw: f32,
}

impl GenerationMix {
    /// Total generation across all plant types.
    pub fn total(&self) -> f32 {
        self.coal_mw + self.gas_mw + self.wind_mw + self.battery_mw
    }
}
