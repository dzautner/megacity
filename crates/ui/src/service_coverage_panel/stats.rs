//! Coverage computation and color helpers.

use bevy_egui::egui;

use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{WorldGrid, ZoneType};
use simulation::happiness::ServiceCoverageGrid;
use simulation::services::{ServiceBuilding, ServiceType};

use super::categories::ServiceCategory;

// =============================================================================
// Computed coverage data (updated each frame the panel is visible)
// =============================================================================

/// Per-category coverage statistics.
#[derive(Debug, Clone, Default)]
pub struct CategoryStats {
    /// Percentage of developed cells covered (0.0..1.0).
    pub coverage_pct: f64,
    /// Number of service buildings in this category (capacity proxy).
    pub building_count: u32,
    /// Number of developed/zoned cells that want service (demand proxy).
    pub demand_cells: u32,
    /// Number of those demand cells that are covered.
    pub covered_cells: u32,
    /// Total monthly maintenance cost for this category.
    pub monthly_maintenance: f64,
}

/// Per-service-type statistics within a category.
#[derive(Debug, Clone, Default)]
pub struct ServiceTypeStats {
    pub count: u32,
    pub monthly_maintenance: f64,
}

// =============================================================================
// Coverage computation
// =============================================================================

/// Computes the coverage percentage for a single category.
///
/// Coverage = (developed cells with the category's coverage bit set) / (total developed cells).
/// "Developed" means the cell has a non-None zone type.
pub fn compute_category_stats(
    category: ServiceCategory,
    grid: &WorldGrid,
    coverage: &ServiceCoverageGrid,
    services: &[&ServiceBuilding],
) -> CategoryStats {
    let bit = category.coverage_bit();
    let mut demand_cells: u32 = 0;
    let mut covered_cells: u32 = 0;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.zone == ZoneType::None {
                continue;
            }
            demand_cells += 1;
            let idx = ServiceCoverageGrid::idx(x, y);
            if coverage.flags[idx] & bit != 0 {
                covered_cells += 1;
            }
        }
    }

    let coverage_pct = if demand_cells > 0 {
        covered_cells as f64 / demand_cells as f64
    } else {
        0.0
    };

    let mut building_count = 0u32;
    let mut monthly_maintenance = 0.0f64;
    for s in services {
        if category.matches_service(s.service_type) {
            building_count += 1;
            monthly_maintenance += ServiceBuilding::monthly_maintenance(s.service_type);
        }
    }

    CategoryStats {
        coverage_pct,
        building_count,
        demand_cells,
        covered_cells,
        monthly_maintenance,
    }
}

/// Computes per-service-type stats within a list of services.
pub fn compute_service_type_stats(
    service_type: ServiceType,
    services: &[&ServiceBuilding],
) -> ServiceTypeStats {
    let mut count = 0u32;
    let mut monthly_maintenance = 0.0f64;
    for s in services {
        if s.service_type == service_type {
            count += 1;
            monthly_maintenance += ServiceBuilding::monthly_maintenance(s.service_type);
        }
    }
    ServiceTypeStats {
        count,
        monthly_maintenance,
    }
}

// =============================================================================
// Color helpers
// =============================================================================

/// Returns the egui color for a coverage percentage.
/// Green (>80%), yellow (50-80%), red (<50%).
pub fn coverage_color(pct: f64) -> egui::Color32 {
    if pct > 0.80 {
        egui::Color32::from_rgb(80, 200, 80) // green
    } else if pct >= 0.50 {
        egui::Color32::from_rgb(220, 200, 50) // yellow
    } else {
        egui::Color32::from_rgb(255, 60, 60) // red
    }
}

/// Returns a label describing the coverage level.
pub fn coverage_label(pct: f64) -> &'static str {
    if pct > 0.80 {
        "Good"
    } else if pct >= 0.50 {
        "Moderate"
    } else {
        "Poor"
    }
}
