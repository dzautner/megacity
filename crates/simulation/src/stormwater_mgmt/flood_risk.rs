//! Flood risk overlay grid: per-cell risk score combining elevation,
//! imperviousness, and drainage coverage.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

/// Per-cell flood risk score (0 = no risk, 255 = maximum risk).
///
/// Computed as a weighted combination of:
/// - Low elevation (lower = higher risk)
/// - High imperviousness (more impervious = more runoff)
/// - Low drainage coverage (fewer storm drains = higher risk)
#[derive(Resource, Serialize, Deserialize)]
pub struct FloodRiskGrid {
    pub risk: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for FloodRiskGrid {
    fn default() -> Self {
        Self {
            risk: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl FloodRiskGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.risk[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.risk[y * self.width + x] = val;
    }

    /// Average risk score across the entire grid.
    pub fn average_risk(&self) -> f32 {
        if self.risk.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.risk.iter().map(|&v| v as u64).sum();
        sum as f32 / self.risk.len() as f32
    }
}

// =============================================================================
// Risk calculation constants
// =============================================================================

/// Weight for elevation component (lower elevation = higher risk).
pub(crate) const ELEVATION_WEIGHT: f32 = 0.4;

/// Weight for imperviousness component (more impervious = higher risk).
pub(crate) const IMPERVIOUSNESS_WEIGHT: f32 = 0.35;

/// Weight for drainage deficit component (less drainage = higher risk).
pub(crate) const DRAINAGE_WEIGHT: f32 = 0.25;

/// Compute flood risk score for a single cell.
///
/// - `elevation`: terrain elevation (0.0 to 1.0, normalized)
/// - `imperviousness`: fraction of surface that is impervious (0.0 to 1.0)
/// - `drainage_coverage`: fraction of the area covered by storm drains (0.0 to 1.0)
///
/// Returns a score 0..=255 where higher = more flood risk.
pub fn compute_cell_risk(elevation: f32, imperviousness: f32, drainage_coverage: f32) -> u8 {
    // Lower elevation → higher risk (invert)
    let elev_risk = 1.0 - elevation.clamp(0.0, 1.0);
    // Higher imperviousness → higher risk
    let imperv_risk = imperviousness.clamp(0.0, 1.0);
    // Lower drainage → higher risk (invert)
    let drain_risk = 1.0 - drainage_coverage.clamp(0.0, 1.0);

    let combined = elev_risk * ELEVATION_WEIGHT
        + imperv_risk * IMPERVIOUSNESS_WEIGHT
        + drain_risk * DRAINAGE_WEIGHT;

    (combined * 255.0).clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flood_risk_grid_default() {
        let grid = FloodRiskGrid::default();
        assert_eq!(grid.risk.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!(grid.risk.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_flood_risk_grid_get_set() {
        let mut grid = FloodRiskGrid::default();
        grid.set(10, 10, 200);
        assert_eq!(grid.get(10, 10), 200);
    }

    #[test]
    fn test_low_elevation_high_imperv_no_drainage_max_risk() {
        let risk = compute_cell_risk(0.0, 1.0, 0.0);
        // All factors at maximum risk: 0.4 + 0.35 + 0.25 = 1.0 → 255
        assert_eq!(risk, 255);
    }

    #[test]
    fn test_high_elevation_low_imperv_full_drainage_min_risk() {
        let risk = compute_cell_risk(1.0, 0.0, 1.0);
        // All factors at minimum risk: 0.0 + 0.0 + 0.0 = 0.0 → 0
        assert_eq!(risk, 0);
    }

    #[test]
    fn test_medium_risk_values() {
        let risk = compute_cell_risk(0.5, 0.5, 0.5);
        // Each factor at 0.5: 0.5*0.4 + 0.5*0.35 + 0.5*0.25 = 0.5 → 127
        assert!(risk > 120 && risk < 135, "got {}", risk);
    }

    #[test]
    fn test_risk_increases_with_lower_elevation() {
        let high_elev = compute_cell_risk(0.9, 0.5, 0.5);
        let low_elev = compute_cell_risk(0.1, 0.5, 0.5);
        assert!(low_elev > high_elev);
    }

    #[test]
    fn test_risk_increases_with_higher_imperviousness() {
        let low_imperv = compute_cell_risk(0.5, 0.1, 0.5);
        let high_imperv = compute_cell_risk(0.5, 0.9, 0.5);
        assert!(high_imperv > low_imperv);
    }

    #[test]
    fn test_risk_increases_with_lower_drainage() {
        let good_drainage = compute_cell_risk(0.5, 0.5, 0.9);
        let poor_drainage = compute_cell_risk(0.5, 0.5, 0.1);
        assert!(poor_drainage > good_drainage);
    }

    #[test]
    fn test_average_risk_empty() {
        let mut grid = FloodRiskGrid::default();
        grid.risk.clear();
        assert!((grid.average_risk() - 0.0).abs() < f32::EPSILON);
    }
}
