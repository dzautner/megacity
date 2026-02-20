use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::citizen::{Citizen, CitizenDetails, Position, WorkLocation};
use crate::config::CELL_SIZE;
use crate::crime::CrimeGrid;
use crate::grid::WorldGrid;
use crate::homelessness::{HomelessShelter, HomelessnessStats};
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Tracks city-wide welfare statistics.
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
pub struct WelfareStats {
    /// Total citizens currently sheltered in homeless shelters.
    pub total_sheltered: u32,
    /// Total citizens receiving welfare office benefits (job training, etc.).
    pub total_welfare_recipients: u32,
    /// Monthly cost of welfare programs.
    pub monthly_cost: f64,
    /// Total shelter bed capacity across all shelters.
    pub shelter_capacity: u32,
    /// Current shelter occupancy across all shelters.
    pub shelter_occupancy: u32,
    /// Number of welfare offices in the city.
    pub welfare_office_count: u32,
    /// Number of homeless shelters in the city.
    pub shelter_count: u32,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Capacity (beds) per homeless shelter building.
const SHELTER_CAPACITY_PER_BUILDING: u32 = 50;

/// Crime reduction per welfare office in its coverage radius.
const WELFARE_CRIME_REDUCTION: u8 = 8;

/// Employment chance bonus for unemployed citizens near a welfare office.
/// Applied as a multiplier to the citizen's salary (simulating job training benefit).
const JOB_TRAINING_SALARY_BONUS: f32 = 50.0;

// ---------------------------------------------------------------------------
// System: update_welfare
// ---------------------------------------------------------------------------

/// Updates welfare statistics and applies welfare effects.
///
/// Runs on SlowTickTimer (every 100 ticks):
/// - Counts homeless shelters and their total capacity
/// - Welfare offices provide job training bonus (increase salary for unemployed citizens in radius)
/// - Welfare offices reduce crime in radius (social safety net effect)
/// - Tracks shelter occupancy vs capacity
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_welfare(
    slow_timer: Res<SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    shelters: Query<&HomelessShelter>,
    homeless_stats: Res<HomelessnessStats>,
    mut welfare_stats: ResMut<WelfareStats>,
    mut crime_grid: ResMut<CrimeGrid>,
    mut unemployed_citizens: Query<
        (&Position, &mut CitizenDetails),
        (With<Citizen>, Without<WorkLocation>),
    >,
    grid: Res<WorldGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Count shelters and capacity ---
    let mut shelter_count = 0u32;
    let mut total_capacity = 0u32;
    let mut total_occupancy = 0u32;

    for shelter in &shelters {
        shelter_count += 1;
        total_capacity += shelter.capacity;
        total_occupancy += shelter.current_occupants;
    }

    // Also count shelters from ServiceBuilding entities that might not have
    // a HomelessShelter component yet (placed but not yet spawned with component).
    // In practice, these are tracked via the HomelessShelter component in homelessness.rs,
    // but we track service buildings separately for stats display.
    let mut service_shelter_count = 0u32;
    let mut welfare_office_count = 0u32;

    // Collect welfare office positions for radius-based effects
    let mut welfare_offices: Vec<(usize, usize, f32)> = Vec::new();

    for service in &services {
        match service.service_type {
            ServiceType::HomelessShelter => {
                service_shelter_count += 1;
            }
            ServiceType::WelfareOffice => {
                welfare_office_count += 1;
                welfare_offices.push((service.grid_x, service.grid_y, service.radius));
            }
            _ => {}
        }
    }

    // Use the max of component-tracked vs service-tracked shelter count
    let effective_shelter_count = shelter_count.max(service_shelter_count);

    // If service buildings exist without HomelessShelter components, estimate capacity
    if service_shelter_count > shelter_count {
        let extra = service_shelter_count - shelter_count;
        total_capacity += extra * SHELTER_CAPACITY_PER_BUILDING;
    }

    // --- Welfare office effects ---

    // 1. Crime reduction in welfare office radius
    for &(gx, gy, radius) in &welfare_offices {
        let radius_cells = (radius / CELL_SIZE) as i32;
        for dy in -radius_cells..=radius_cells {
            for dx in -radius_cells..=radius_cells {
                let nx = gx as i32 + dx;
                let ny = gy as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < grid.width && (ny as usize) < grid.height {
                    let dist = dx.abs() + dy.abs();
                    // Falloff with distance
                    let effect = WELFARE_CRIME_REDUCTION.saturating_sub(dist as u8 / 2);
                    let idx = ny as usize * grid.width + nx as usize;
                    crime_grid.levels[idx] = crime_grid.levels[idx].saturating_sub(effect);
                }
            }
        }
    }

    // 2. Job training bonus: boost salary for unemployed citizens near welfare offices
    //    This makes them more employable in the job_seeking system.
    let mut welfare_recipients = 0u32;

    if !welfare_offices.is_empty() {
        for (pos, mut details) in &mut unemployed_citizens {
            // Check if citizen is within any welfare office radius
            for &(gx, gy, radius) in &welfare_offices {
                let (office_wx, office_wy) = WorldGrid::grid_to_world(gx, gy);
                let dx = pos.x - office_wx;
                let dy = pos.y - office_wy;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= radius * radius {
                    // Apply job training benefit: increase effective salary
                    // This makes the citizen more competitive in the job market
                    details.salary = (details.salary + JOB_TRAINING_SALARY_BONUS).min(10000.0);
                    welfare_recipients += 1;
                    break; // Only count once per citizen
                }
            }
        }
    }

    // --- Calculate monthly cost ---
    let shelter_maintenance: f64 = effective_shelter_count as f64
        * ServiceBuilding::monthly_maintenance(ServiceType::HomelessShelter);
    let welfare_maintenance: f64 = welfare_office_count as f64
        * ServiceBuilding::monthly_maintenance(ServiceType::WelfareOffice);
    let monthly_cost = shelter_maintenance + welfare_maintenance;

    // --- Update stats resource ---
    welfare_stats.total_sheltered = homeless_stats.sheltered;
    welfare_stats.total_welfare_recipients = welfare_recipients;
    welfare_stats.monthly_cost = monthly_cost;
    welfare_stats.shelter_capacity = total_capacity;
    welfare_stats.shelter_occupancy = total_occupancy;
    welfare_stats.welfare_office_count = welfare_office_count;
    welfare_stats.shelter_count = effective_shelter_count;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn welfare_stats_default() {
        let stats = WelfareStats::default();
        assert_eq!(stats.total_sheltered, 0);
        assert_eq!(stats.total_welfare_recipients, 0);
        assert_eq!(stats.monthly_cost, 0.0);
        assert_eq!(stats.shelter_capacity, 0);
        assert_eq!(stats.shelter_occupancy, 0);
        assert_eq!(stats.welfare_office_count, 0);
        assert_eq!(stats.shelter_count, 0);
    }

    #[test]
    fn welfare_office_service_type() {
        assert_eq!(ServiceType::WelfareOffice.name(), "Welfare Office");
        assert!(ServiceBuilding::is_welfare(ServiceType::WelfareOffice));
        assert!(ServiceBuilding::is_welfare(ServiceType::HomelessShelter));
        assert!(!ServiceBuilding::is_welfare(ServiceType::Hospital));
        assert!(!ServiceBuilding::is_welfare(ServiceType::PoliceStation));
    }

    #[test]
    fn welfare_office_coverage_radius() {
        let radius = ServiceBuilding::coverage_radius(ServiceType::WelfareOffice);
        assert!(radius > 0.0);
        // Should be 20.0 * CELL_SIZE = 320.0
        assert_eq!(radius, 20.0 * CELL_SIZE);
    }

    #[test]
    fn welfare_office_cost() {
        let cost = ServiceBuilding::cost(ServiceType::WelfareOffice);
        assert_eq!(cost, 600.0);
    }

    #[test]
    fn welfare_office_maintenance() {
        let maint = ServiceBuilding::monthly_maintenance(ServiceType::WelfareOffice);
        assert_eq!(maint, 20.0);
    }

    #[test]
    fn shelter_capacity_constant() {
        assert_eq!(SHELTER_CAPACITY_PER_BUILDING, 50);
    }

    #[test]
    fn crime_reduction_constant() {
        assert!(WELFARE_CRIME_REDUCTION > 0);
        assert!(WELFARE_CRIME_REDUCTION <= 20);
    }
}

pub struct WelfarePlugin;

impl Plugin for WelfarePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WelfareStats>().add_systems(
            FixedUpdate,
            update_welfare.after(crate::homelessness::recover_from_homelessness),
        );
    }
}
