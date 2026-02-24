//! Green infrastructure runoff reduction.
//!
//! Trees and parks reduce effective stormwater runoff in their cells and
//! nearby neighbors, simulating the absorption capacity of vegetation.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::stormwater::StormwaterGrid;
use crate::trees::TreeGrid;

/// Fraction of runoff absorbed by a cell containing a tree.
pub(crate) const TREE_RUNOFF_REDUCTION: f32 = 0.30;

/// Fraction of runoff absorbed by a park cell (ServiceType::Park).
pub(crate) const PARK_RUNOFF_REDUCTION: f32 = 0.25;

/// Maximum combined green infrastructure reduction per cell.
pub(crate) const MAX_GREEN_REDUCTION: f32 = 0.50;

/// Apply green infrastructure runoff reductions to the stormwater grid.
///
/// Trees and parks absorb a portion of runoff from their cells, reducing
/// effective stormwater accumulation. This is called after the main stormwater
/// update to model nature-based drainage solutions.
///
/// Returns the total runoff volume absorbed by green infrastructure this tick.
pub(crate) fn apply_green_infrastructure(
    stormwater: &mut StormwaterGrid,
    tree_grid: &TreeGrid,
    park_cells: &[bool],
) -> f32 {
    let mut total_absorbed = 0.0_f32;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let current_runoff = stormwater.runoff[idx];
            if current_runoff <= 0.0 {
                continue;
            }

            let mut reduction = 0.0_f32;

            if tree_grid.has_tree(x, y) {
                reduction += TREE_RUNOFF_REDUCTION;
            }

            if idx < park_cells.len() && park_cells[idx] {
                reduction += PARK_RUNOFF_REDUCTION;
            }

            if reduction > 0.0 {
                let effective_reduction = reduction.min(MAX_GREEN_REDUCTION);
                let absorbed = current_runoff * effective_reduction;
                stormwater.runoff[idx] -= absorbed;
                total_absorbed += absorbed;
            }
        }
    }

    total_absorbed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_reduces_runoff_by_30_percent() {
        let mut sw = StormwaterGrid::default();
        sw.set(10, 10, 100.0);

        let mut trees = TreeGrid::default();
        trees.set(10, 10, true);

        let park_cells = vec![false; GRID_WIDTH * GRID_HEIGHT];

        let absorbed = apply_green_infrastructure(&mut sw, &trees, &park_cells);

        let expected_remaining = 100.0 * (1.0 - TREE_RUNOFF_REDUCTION);
        assert!(
            (sw.get(10, 10) - expected_remaining).abs() < 0.01,
            "expected ~{}, got {}",
            expected_remaining,
            sw.get(10, 10)
        );
        assert!(absorbed > 29.0 && absorbed < 31.0, "absorbed {}", absorbed);
    }

    #[test]
    fn test_park_reduces_runoff_by_25_percent() {
        let mut sw = StormwaterGrid::default();
        sw.set(20, 20, 200.0);

        let trees = TreeGrid::default();

        let mut park_cells = vec![false; GRID_WIDTH * GRID_HEIGHT];
        park_cells[20 * GRID_WIDTH + 20] = true;

        let absorbed = apply_green_infrastructure(&mut sw, &trees, &park_cells);

        let expected_remaining = 200.0 * (1.0 - PARK_RUNOFF_REDUCTION);
        assert!(
            (sw.get(20, 20) - expected_remaining).abs() < 0.1,
            "expected ~{}, got {}",
            expected_remaining,
            sw.get(20, 20)
        );
        assert!(absorbed > 49.0 && absorbed < 51.0, "absorbed {}", absorbed);
    }

    #[test]
    fn test_combined_tree_and_park_capped_at_max() {
        let mut sw = StormwaterGrid::default();
        sw.set(5, 5, 100.0);

        let mut trees = TreeGrid::default();
        trees.set(5, 5, true);

        let mut park_cells = vec![false; GRID_WIDTH * GRID_HEIGHT];
        park_cells[5 * GRID_WIDTH + 5] = true;

        let absorbed = apply_green_infrastructure(&mut sw, &trees, &park_cells);

        // Tree (0.30) + Park (0.25) = 0.55, capped at 0.50
        let expected_remaining = 100.0 * (1.0 - MAX_GREEN_REDUCTION);
        assert!(
            (sw.get(5, 5) - expected_remaining).abs() < 0.01,
            "expected ~{}, got {}",
            expected_remaining,
            sw.get(5, 5)
        );
        assert!(
            (absorbed - 100.0 * MAX_GREEN_REDUCTION).abs() < 0.01,
            "absorbed {}",
            absorbed
        );
    }

    #[test]
    fn test_no_green_infra_no_change() {
        let mut sw = StormwaterGrid::default();
        sw.set(15, 15, 50.0);

        let trees = TreeGrid::default();
        let park_cells = vec![false; GRID_WIDTH * GRID_HEIGHT];

        let absorbed = apply_green_infrastructure(&mut sw, &trees, &park_cells);

        assert!((sw.get(15, 15) - 50.0).abs() < f32::EPSILON);
        assert!((absorbed - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_zero_runoff_not_affected() {
        let mut sw = StormwaterGrid::default();
        // All zeros by default

        let mut trees = TreeGrid::default();
        trees.set(0, 0, true);

        let park_cells = vec![false; GRID_WIDTH * GRID_HEIGHT];

        let absorbed = apply_green_infrastructure(&mut sw, &trees, &park_cells);
        assert!((absorbed - 0.0).abs() < f32::EPSILON);
    }
}
