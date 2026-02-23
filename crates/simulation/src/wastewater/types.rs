//! Wastewater types, constants, and helper functions.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::water_demand::WaterDemand;

// =============================================================================
// Constants
// =============================================================================

/// Fraction of water consumption that becomes sewage.
pub(crate) const SEWAGE_FRACTION: f32 = 0.80;

/// Treatment capacity per sewage plant in gallons per day.
pub(crate) const TREATMENT_CAPACITY_PER_PLANT: f32 = 50_000.0;

/// Pollution amount added to each water cell within discharge radius when overflow occurs.
pub(crate) const DISCHARGE_POLLUTION_AMOUNT: u8 = 15;

/// Radius (in grid cells) around a sewage plant within which buildings are considered serviced.
pub(crate) const SEWAGE_SERVICE_RADIUS: i32 = 20;

/// Radius (in grid cells) around the city center used to find water cells for discharge.
pub(crate) const DISCHARGE_SEARCH_RADIUS: i32 = 30;

/// Health penalty per slow tick for citizens living near uncollected sewage.
pub(crate) const HEALTH_PENALTY_PER_TICK: f32 = 1.5;

/// Happiness penalty per slow tick for citizens living near uncollected sewage.
pub(crate) const HAPPINESS_PENALTY_PER_TICK: f32 = 3.0;

/// Distance (in grid cells) from a building without sewage service that triggers health penalty.
pub(crate) const UNCOLLECTED_PENALTY_RADIUS: i32 = 5;

// =============================================================================
// Wastewater state resource
// =============================================================================

/// City-wide wastewater and sewage tracking resource.
#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct WastewaterState {
    /// Total sewage generated across all buildings in gallons per day.
    pub total_sewage_generated: f32,
    /// Total treatment capacity from all sewage plants in gallons per day.
    pub total_treatment_capacity: f32,
    /// Untreated sewage overflow amount in gallons per day (excess beyond capacity).
    pub overflow_amount: f32,
    /// Fraction of buildings with sewage service coverage (0.0..=1.0).
    pub coverage_ratio: f32,
    /// Number of raw sewage discharge events (incremented each period with overflow).
    pub pollution_events: u32,
    /// Whether a health penalty is currently active due to uncollected sewage.
    pub health_penalty_active: bool,
}

// =============================================================================
// Helper functions
// =============================================================================

/// Compute sewage generation for a building based on its water demand.
/// Returns sewage in gallons per day (80% of water consumption).
pub(crate) fn sewage_for_demand(demand: &WaterDemand) -> f32 {
    demand.demand_gpd * SEWAGE_FRACTION
}

/// Check if a building at (bx, by) is within the service radius of any sewage plant.
pub(crate) fn is_serviced_by_sewage_plant(
    bx: usize,
    by: usize,
    sewage_plants: &[(usize, usize)],
) -> bool {
    for &(px, py) in sewage_plants {
        let dx = (bx as i32 - px as i32).abs();
        let dy = (by as i32 - py as i32).abs();
        if dx + dy <= SEWAGE_SERVICE_RADIUS {
            return true;
        }
    }
    false
}

/// Find water cells near a set of sewage plant locations for discharge.
/// Returns a list of (x, y) water cell coordinates.
pub(crate) fn find_discharge_water_cells(
    grid: &WorldGrid,
    sewage_plants: &[(usize, usize)],
) -> Vec<(usize, usize)> {
    let mut water_cells = Vec::new();

    // If there are sewage plants, search around them for water cells
    // Otherwise, search from the center of the map
    let search_centers: Vec<(usize, usize)> = if sewage_plants.is_empty() {
        vec![(GRID_WIDTH / 2, GRID_HEIGHT / 2)]
    } else {
        sewage_plants.to_vec()
    };

    for &(cx, cy) in &search_centers {
        for dy in -DISCHARGE_SEARCH_RADIUS..=DISCHARGE_SEARCH_RADIUS {
            for dx in -DISCHARGE_SEARCH_RADIUS..=DISCHARGE_SEARCH_RADIUS {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;
                if grid.get(ux, uy).cell_type == CellType::Water {
                    water_cells.push((ux, uy));
                }
            }
        }
    }

    water_cells.dedup();
    water_cells
}
