//! SVC-016: Telecom Infrastructure (Cell Towers, Data Centers)
//!
//! Cell towers provide mobile coverage to nearby areas with signal intensity
//! that falls off with distance. Data centers serve businesses by boosting
//! commercial productivity in their coverage area. Coverage affects citizen
//! satisfaction and commercial output.
//!
//! ## Tiers
//! - **Basic Cell Tower**: standard radius (15 cells), max signal 200
//! - **Data Center**: larger radius (40 cells), max signal 128, plus a
//!   commercial productivity boost for businesses in range

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::budget::ExtendedBudget;
use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::services::{ServiceBuilding, ServiceType};
use crate::Saveable;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Coverage grid
// ---------------------------------------------------------------------------

/// Per-cell mobile signal coverage level (0–255), where 255 = full coverage.
/// Multiple towers stack (saturating add) so overlapping coverage improves quality.
#[derive(Resource)]
pub struct TelecomCoverage {
    /// Mobile signal level per cell.
    pub signal: Vec<u8>,
    /// Commercial productivity multiplier from data centers (1.0 = no boost).
    /// Stored as a separate grid because it only applies to commercial zones.
    pub commercial_boost: Vec<f32>,
}

impl Default for TelecomCoverage {
    fn default() -> Self {
        let n = GRID_WIDTH * GRID_HEIGHT;
        Self {
            signal: vec![0; n],
            commercial_boost: vec![1.0; n],
        }
    }
}

impl TelecomCoverage {
    pub fn clear(&mut self) {
        self.signal.fill(0);
        self.commercial_boost.fill(1.0);
    }

    #[inline]
    pub fn idx(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    /// Get mobile signal level at a cell (0–255).
    #[inline]
    pub fn get_signal(&self, x: usize, y: usize) -> u8 {
        self.signal[Self::idx(x, y)]
    }

    /// Get commercial productivity multiplier at a cell (>= 1.0).
    #[inline]
    pub fn get_commercial_boost(&self, x: usize, y: usize) -> f32 {
        self.commercial_boost[Self::idx(x, y)]
    }

    /// Number of cells with any mobile signal.
    pub fn covered_cells(&self) -> u32 {
        self.signal.iter().filter(|&&v| v > 0).count() as u32
    }

    /// Number of cells with commercial boost above baseline.
    pub fn boosted_cells(&self) -> u32 {
        self.commercial_boost
            .iter()
            .filter(|&&v| v > 1.0)
            .count() as u32
    }
}

// ---------------------------------------------------------------------------
// Aggregate state (Saveable)
// ---------------------------------------------------------------------------

/// City-wide telecom statistics, persisted across save/load.
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct TelecomState {
    /// Number of cell towers placed.
    pub cell_tower_count: u32,
    /// Number of data centers placed.
    pub data_center_count: u32,
    /// Percentage of grid cells with mobile signal (0–100).
    pub coverage_percentage: f32,
    /// Total monthly maintenance cost for all telecom buildings.
    pub monthly_cost: f64,
}

impl Saveable for TelecomState {
    const SAVE_KEY: &'static str = "telecom";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum signal intensity for a basic cell tower.
const CELL_TOWER_MAX_SIGNAL: u8 = 200;

/// Maximum signal intensity for a data center (provides light mobile coverage).
const DATA_CENTER_MAX_SIGNAL: u8 = 128;

/// Maximum commercial productivity multiplier from a data center at its center.
const DATA_CENTER_MAX_BOOST: f32 = 1.25;

/// Happiness bonus per coverage level: at full signal (255) this gives +5.
pub const TELECOM_HAPPINESS_BONUS: f32 = 5.0;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Recompute the telecom coverage grids from placed CellTower and DataCenter
/// service buildings. Runs on the slow tick timer (~every 100 ticks).
pub fn update_telecom_coverage(
    slow_tick: Res<SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    ext_budget: Res<ExtendedBudget>,
    mut coverage: ResMut<TelecomCoverage>,
    mut state: ResMut<TelecomState>,
) {
    if !slow_tick.should_run() {
        return;
    }

    coverage.clear();

    let mut cell_tower_count: u32 = 0;
    let mut data_center_count: u32 = 0;
    let mut monthly_cost = 0.0_f64;

    for service in &services {
        if !ServiceBuilding::is_telecom(service.service_type) {
            continue;
        }

        monthly_cost += ServiceBuilding::monthly_maintenance(service.service_type);

        let budget_level = ext_budget.service_budgets.for_service(service.service_type);
        let effective_radius = service.radius * budget_level;
        let radius_cells = (effective_radius / CELL_SIZE).ceil() as i32;
        let sx = service.grid_x as i32;
        let sy = service.grid_y as i32;
        let r2 = effective_radius * effective_radius;

        match service.service_type {
            ServiceType::CellTower => {
                cell_tower_count += 1;
                stamp_signal(
                    &mut coverage.signal,
                    sx,
                    sy,
                    radius_cells,
                    r2,
                    CELL_TOWER_MAX_SIGNAL,
                );
            }
            ServiceType::DataCenter => {
                data_center_count += 1;
                // Data centers provide lighter mobile coverage...
                stamp_signal(
                    &mut coverage.signal,
                    sx,
                    sy,
                    radius_cells,
                    r2,
                    DATA_CENTER_MAX_SIGNAL,
                );
                // ...plus a commercial productivity boost.
                stamp_commercial_boost(
                    &mut coverage.commercial_boost,
                    sx,
                    sy,
                    radius_cells,
                    r2,
                );
            }
            _ => {}
        }
    }

    // Update aggregate stats.
    let total_cells = (GRID_WIDTH * GRID_HEIGHT) as f32;
    state.cell_tower_count = cell_tower_count;
    state.data_center_count = data_center_count;
    state.coverage_percentage = coverage.covered_cells() as f32 / total_cells * 100.0;
    state.monthly_cost = monthly_cost;
}

/// Stamp radial signal intensity onto the signal grid (saturating add).
fn stamp_signal(
    signal: &mut [u8],
    sx: i32,
    sy: i32,
    radius_cells: i32,
    r2: f32,
    max_signal: u8,
) {
    for dy in -radius_cells..=radius_cells {
        for dx in -radius_cells..=radius_cells {
            let cx = sx + dx;
            let cy = sy + dy;
            if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
                continue;
            }
            let wx = dx as f32 * CELL_SIZE;
            let wy = dy as f32 * CELL_SIZE;
            let dist_sq = wx * wx + wy * wy;
            if dist_sq > r2 {
                continue;
            }
            let dist_ratio = (dist_sq / r2).sqrt();
            let intensity = ((1.0 - dist_ratio) * max_signal as f32) as u8;
            let idx = TelecomCoverage::idx(cx as usize, cy as usize);
            signal[idx] = signal[idx].saturating_add(intensity);
        }
    }
}

/// Stamp commercial productivity boost onto the boost grid (additive above 1.0).
fn stamp_commercial_boost(
    boost: &mut [f32],
    sx: i32,
    sy: i32,
    radius_cells: i32,
    r2: f32,
) {
    let max_extra = DATA_CENTER_MAX_BOOST - 1.0; // 0.25
    for dy in -radius_cells..=radius_cells {
        for dx in -radius_cells..=radius_cells {
            let cx = sx + dx;
            let cy = sy + dy;
            if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
                continue;
            }
            let wx = dx as f32 * CELL_SIZE;
            let wy = dy as f32 * CELL_SIZE;
            let dist_sq = wx * wx + wy * wy;
            if dist_sq > r2 {
                continue;
            }
            let dist_ratio = (dist_sq / r2).sqrt();
            let extra = (1.0 - dist_ratio) * max_extra;
            let idx = TelecomCoverage::idx(cx as usize, cy as usize);
            boost[idx] += extra;
        }
    }
}

/// Compute the telecom happiness bonus for a citizen at a given grid position.
/// Returns a value between 0.0 and `TELECOM_HAPPINESS_BONUS`.
#[inline]
pub fn telecom_happiness_bonus(coverage: &TelecomCoverage, x: usize, y: usize) -> f32 {
    let level = coverage.get_signal(x, y) as f32;
    (level / 255.0) * TELECOM_HAPPINESS_BONUS
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct TelecomPlugin;

impl Plugin for TelecomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TelecomCoverage>();
        app.init_resource::<TelecomState>();

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<TelecomState>();

        app.add_systems(
            FixedUpdate,
            update_telecom_coverage
                .after(crate::traffic::update_traffic_density)
                .before(crate::happiness::update_service_coverage)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telecom_coverage_default() {
        let cov = TelecomCoverage::default();
        assert_eq!(cov.signal.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!(cov.signal.iter().all(|&v| v == 0));
        assert!(cov.commercial_boost.iter().all(|&v| (v - 1.0).abs() < f32::EPSILON));
    }

    #[test]
    fn test_telecom_coverage_clear() {
        let mut cov = TelecomCoverage::default();
        let idx = TelecomCoverage::idx(10, 10);
        cov.signal[idx] = 200;
        cov.commercial_boost[idx] = 1.5;
        cov.clear();
        assert_eq!(cov.signal[idx], 0);
        assert!((cov.commercial_boost[idx] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_telecom_happiness_bonus_zero() {
        let cov = TelecomCoverage::default();
        let bonus = telecom_happiness_bonus(&cov, 10, 10);
        assert_eq!(bonus, 0.0);
    }

    #[test]
    fn test_telecom_happiness_bonus_full() {
        let mut cov = TelecomCoverage::default();
        let idx = TelecomCoverage::idx(10, 10);
        cov.signal[idx] = 255;
        let bonus = telecom_happiness_bonus(&cov, 10, 10);
        assert!((bonus - TELECOM_HAPPINESS_BONUS).abs() < 0.01);
    }

    #[test]
    fn test_telecom_happiness_bonus_half() {
        let mut cov = TelecomCoverage::default();
        let idx = TelecomCoverage::idx(10, 10);
        cov.signal[idx] = 128;
        let bonus = telecom_happiness_bonus(&cov, 10, 10);
        assert!(bonus > 2.0 && bonus < 3.0);
    }

    #[test]
    fn test_telecom_state_default() {
        let state = TelecomState::default();
        assert_eq!(state.cell_tower_count, 0);
        assert_eq!(state.data_center_count, 0);
        assert_eq!(state.coverage_percentage, 0.0);
        assert_eq!(state.monthly_cost, 0.0);
    }

    #[test]
    fn test_coverage_idx() {
        let idx = TelecomCoverage::idx(5, 10);
        assert_eq!(idx, 10 * GRID_WIDTH + 5);
    }

    #[test]
    fn test_signal_saturating_add() {
        let mut cov = TelecomCoverage::default();
        let idx = TelecomCoverage::idx(15, 15);
        cov.signal[idx] = 200;
        cov.signal[idx] = cov.signal[idx].saturating_add(100);
        assert_eq!(cov.signal[idx], 255);
    }

    #[test]
    fn test_stamp_signal_center_gets_max() {
        let n = GRID_WIDTH * GRID_HEIGHT;
        let mut signal = vec![0u8; n];
        let radius = 10.0 * CELL_SIZE;
        stamp_signal(&mut signal, 50, 50, 10, radius * radius, 200);
        // Center cell should get the maximum signal (distance=0 -> intensity=200)
        let idx = TelecomCoverage::idx(50, 50);
        assert_eq!(signal[idx], 200);
    }

    #[test]
    fn test_stamp_signal_outside_radius_zero() {
        let n = GRID_WIDTH * GRID_HEIGHT;
        let mut signal = vec![0u8; n];
        let radius = 5.0 * CELL_SIZE;
        stamp_signal(&mut signal, 50, 50, 5, radius * radius, 200);
        // A cell far outside the radius should have zero signal
        let idx = TelecomCoverage::idx(100, 100);
        assert_eq!(signal[idx], 0);
    }

    #[test]
    fn test_stamp_commercial_boost_center() {
        let n = GRID_WIDTH * GRID_HEIGHT;
        let mut boost = vec![1.0f32; n];
        let radius = 10.0 * CELL_SIZE;
        stamp_commercial_boost(&mut boost, 50, 50, 10, radius * radius);
        let idx = TelecomCoverage::idx(50, 50);
        // Center gets max boost: 1.0 + 0.25 = 1.25
        assert!((boost[idx] - DATA_CENTER_MAX_BOOST).abs() < 0.01);
    }

    #[test]
    fn test_stamp_commercial_boost_outside_radius() {
        let n = GRID_WIDTH * GRID_HEIGHT;
        let mut boost = vec![1.0f32; n];
        let radius = 5.0 * CELL_SIZE;
        stamp_commercial_boost(&mut boost, 50, 50, 5, radius * radius);
        let idx = TelecomCoverage::idx(100, 100);
        assert!((boost[idx] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let state = TelecomState {
            cell_tower_count: 5,
            data_center_count: 2,
            coverage_percentage: 42.5,
            monthly_cost: 120.0,
        };
        let bytes = state.save_to_bytes().unwrap();
        let loaded = TelecomState::load_from_bytes(&bytes);
        assert_eq!(loaded.cell_tower_count, 5);
        assert_eq!(loaded.data_center_count, 2);
        assert!((loaded.coverage_percentage - 42.5).abs() < 0.01);
        assert!((loaded.monthly_cost - 120.0).abs() < 0.01);
    }

    #[test]
    fn test_covered_cells_count() {
        let mut cov = TelecomCoverage::default();
        assert_eq!(cov.covered_cells(), 0);
        cov.signal[TelecomCoverage::idx(10, 10)] = 100;
        cov.signal[TelecomCoverage::idx(20, 20)] = 50;
        assert_eq!(cov.covered_cells(), 2);
    }

    #[test]
    fn test_boosted_cells_count() {
        let mut cov = TelecomCoverage::default();
        assert_eq!(cov.boosted_cells(), 0);
        cov.commercial_boost[TelecomCoverage::idx(10, 10)] = 1.1;
        assert_eq!(cov.boosted_cells(), 1);
    }
}
