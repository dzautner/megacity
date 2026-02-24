//! Precomputed coverage metrics for the info panel.
//!
//! Instead of the UI recomputing coverage every frame (or on a wall-clock
//! timer), we compute coverage aggregates here on the `SlowTickTimer` cadence
//! and expose them as a `CoverageMetrics` resource that the UI reads directly.

use bevy::prelude::*;

use crate::config::CELL_SIZE;
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::services::ServiceBuilding;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Cached coverage fractions (0.0..=1.0) for each service/utility category.
///
/// Updated every `SlowTickTimer::INTERVAL` ticks in the simulation layer so
/// the UI never needs to iterate the grid or query service buildings itself.
#[derive(Resource, Default, Debug, Clone)]
pub struct CoverageMetrics {
    pub power: f32,
    pub water: f32,
    pub education: f32,
    pub fire: f32,
    pub police: f32,
    pub health: f32,
    pub telecom: f32,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct CoverageMetricsPlugin;

impl Plugin for CoverageMetricsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CoverageMetrics>();
        app.add_systems(Update, update_coverage_metrics);
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

fn update_coverage_metrics(
    timer: Res<SlowTickTimer>,
    mut metrics: ResMut<CoverageMetrics>,
    grid: Res<WorldGrid>,
    services: Query<&ServiceBuilding>,
) {
    if !timer.should_run() {
        return;
    }

    let (power, water) = compute_utility_coverage(&grid);
    metrics.power = power;
    metrics.water = water;
    metrics.education = compute_service_coverage(&services, &grid, ServiceCategory::Education);
    metrics.fire = compute_service_coverage(&services, &grid, ServiceCategory::Fire);
    metrics.police = compute_service_coverage(&services, &grid, ServiceCategory::Police);
    metrics.health = compute_service_coverage(&services, &grid, ServiceCategory::Health);
    metrics.telecom = compute_service_coverage(&services, &grid, ServiceCategory::Telecom);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

enum ServiceCategory {
    Education,
    Fire,
    Police,
    Health,
    Telecom,
}

fn compute_utility_coverage(grid: &WorldGrid) -> (f32, f32) {
    let mut total = 0u32;
    let mut powered = 0u32;
    let mut watered = 0u32;
    for cell in &grid.cells {
        if cell.cell_type == CellType::Grass && cell.zone != ZoneType::None {
            total += 1;
            if cell.has_power {
                powered += 1;
            }
            if cell.has_water {
                watered += 1;
            }
        }
    }
    if total == 0 {
        return (1.0, 1.0);
    }
    (powered as f32 / total as f32, watered as f32 / total as f32)
}

fn compute_service_coverage(
    services: &Query<&ServiceBuilding>,
    grid: &WorldGrid,
    category: ServiceCategory,
) -> f32 {
    let total_zoned = grid
        .cells
        .iter()
        .filter(|c| c.zone != ZoneType::None)
        .count() as f32;
    if total_zoned == 0.0 {
        return 0.0;
    }

    let mut covered_cells = 0u32;
    for service in services.iter() {
        let matches = match category {
            ServiceCategory::Education => ServiceBuilding::is_education(service.service_type),
            ServiceCategory::Fire => ServiceBuilding::is_fire(service.service_type),
            ServiceCategory::Police => ServiceBuilding::is_police(service.service_type),
            ServiceCategory::Health => ServiceBuilding::is_health(service.service_type),
            ServiceCategory::Telecom => ServiceBuilding::is_telecom(service.service_type),
        };
        if matches {
            let radius_cells = service.radius / CELL_SIZE;
            covered_cells += (std::f32::consts::PI * radius_cells * radius_cells) as u32;
        }
    }

    (covered_cells as f32 / total_zoned).min(1.0)
}
