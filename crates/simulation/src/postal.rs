use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::budget::ExtendedBudget;
use crate::buildings::Building;
use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

/// Per-cell postal coverage level (0-255), where 255 = full coverage.
#[derive(Resource)]
pub struct PostalCoverage {
    pub levels: Vec<u8>,
}

impl Default for PostalCoverage {
    fn default() -> Self {
        Self {
            levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
        }
    }
}

impl PostalCoverage {
    pub fn clear(&mut self) {
        self.levels.fill(0);
    }

    #[inline]
    pub fn idx(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[Self::idx(x, y)]
    }
}

/// Aggregate postal statistics for the city (saveable).
#[derive(Resource, Default, Clone, Serialize, Deserialize)]
pub struct PostalStats {
    pub total_covered_cells: u32,
    pub coverage_percentage: f32,
    pub monthly_cost: f64,
    /// Number of commercial buildings with postal coverage.
    pub covered_commercial_buildings: u32,
    /// Average commercial productivity multiplier from postal coverage (1.0 = no bonus).
    pub avg_commercial_productivity: f32,
}

impl crate::Saveable for PostalStats {
    const SAVE_KEY: &'static str = "postal_stats";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

/// Maximum happiness bonus from postal coverage at full coverage (255).
pub const POSTAL_HAPPINESS_BONUS: f32 = 5.0;

/// Slight happiness penalty for residential areas with no postal coverage.
pub const POSTAL_NO_COVERAGE_PENALTY: f32 = 2.0;

/// Maximum commercial productivity multiplier from full postal coverage.
/// A building with full postal coverage gets 15% more productivity.
pub const POSTAL_COMMERCIAL_MAX_BOOST: f32 = 0.15;

/// Update postal coverage grid based on PostOffice and MailSortingCenter service buildings.
/// Runs on SlowTickTimer (every 100 ticks).
///
/// Mail sorting centers double the effective radius of post offices within their radius.
/// Budget level affects coverage quality proportionally.
pub fn update_postal_coverage(
    slow_tick: Res<SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    ext_budget: Res<ExtendedBudget>,
    mut coverage: ResMut<PostalCoverage>,
    mut stats: ResMut<PostalStats>,
    buildings: Query<&Building>,
) {
    if !slow_tick.should_run() {
        return;
    }

    coverage.clear();

    // Collect mail sorting center positions and their effective radii
    let sorting_centers: Vec<(usize, usize, f32)> = services
        .iter()
        .filter(|s| s.service_type == ServiceType::MailSortingCenter)
        .map(|s| {
            let budget_level = ext_budget.service_budgets.for_service(s.service_type);
            let effective_radius = s.radius * budget_level;
            (s.grid_x, s.grid_y, effective_radius)
        })
        .collect();

    // For each post office, check if any sorting center is nearby (within the
    // sorting center's radius). If so, the post office's effective radius is doubled.
    let mut monthly_cost = 0.0f64;

    for service in &services {
        if !ServiceBuilding::is_postal(service.service_type) {
            continue;
        }

        monthly_cost += ServiceBuilding::monthly_maintenance(service.service_type);

        let budget_level = ext_budget.service_budgets.for_service(service.service_type);
        let base_radius = service.radius * budget_level;

        let effective_radius = if service.service_type == ServiceType::PostOffice {
            // Check if boosted by a nearby sorting center
            let boosted = sorting_centers.iter().any(|&(cx, cy, sc_radius)| {
                let dx = (service.grid_x as f32 - cx as f32) * CELL_SIZE;
                let dy = (service.grid_y as f32 - cy as f32) * CELL_SIZE;
                (dx * dx + dy * dy).sqrt() <= sc_radius
            });
            if boosted {
                base_radius * 2.0
            } else {
                base_radius
            }
        } else {
            // MailSortingCenter uses its own radius for coverage
            base_radius
        };

        apply_coverage_circle(
            &mut coverage,
            service.grid_x,
            service.grid_y,
            effective_radius,
            service.service_type,
        );
    }

    // Compute aggregate stats including commercial productivity
    let total_covered = coverage.levels.iter().filter(|&&v| v > 0).count() as u32;
    let total_cells = (GRID_WIDTH * GRID_HEIGHT) as f32;

    let mut covered_commercial = 0u32;
    let mut total_commercial = 0u32;
    let mut productivity_sum = 0.0f32;

    for b in &buildings {
        if b.zone_type.is_commercial() {
            total_commercial += 1;
            let level = coverage.get(b.grid_x, b.grid_y);
            if level > 0 {
                covered_commercial += 1;
            }
            productivity_sum += postal_commercial_multiplier_from_level(level);
        }
    }

    stats.total_covered_cells = total_covered;
    stats.coverage_percentage = total_covered as f32 / total_cells * 100.0;
    stats.monthly_cost = monthly_cost;
    stats.covered_commercial_buildings = covered_commercial;
    stats.avg_commercial_productivity = if total_commercial > 0 {
        productivity_sum / total_commercial as f32
    } else {
        1.0
    };
}

/// Apply a circular coverage footprint from a postal building to the grid.
fn apply_coverage_circle(
    coverage: &mut PostalCoverage,
    center_x: usize,
    center_y: usize,
    effective_radius: f32,
    service_type: ServiceType,
) {
    let radius_cells = (effective_radius / CELL_SIZE).ceil() as i32;
    let sx = center_x as i32;
    let sy = center_y as i32;
    let r2 = effective_radius * effective_radius;

    // Coverage intensity: sorting centers provide lighter coverage (128 max),
    // post offices provide full coverage (255 max)
    let max_coverage: u8 = match service_type {
        ServiceType::PostOffice => 255,
        ServiceType::MailSortingCenter => 128,
        _ => 0,
    };

    for dy in -radius_cells..=radius_cells {
        for dx in -radius_cells..=radius_cells {
            let cx = sx + dx;
            let cy = sy + dy;
            if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
                continue;
            }
            let wx_diff = dx as f32 * CELL_SIZE;
            let wy_diff = dy as f32 * CELL_SIZE;
            let dist_sq = wx_diff * wx_diff + wy_diff * wy_diff;
            if dist_sq > r2 {
                continue;
            }

            // Coverage falls off with distance from the center
            let dist_ratio = (dist_sq / r2).sqrt();
            let intensity = ((1.0 - dist_ratio) * max_coverage as f32) as u8;

            let idx = PostalCoverage::idx(cx as usize, cy as usize);
            // Saturating add: multiple sources stack up to 255
            coverage.levels[idx] = coverage.levels[idx].saturating_add(intensity);
        }
    }
}

/// Compute the postal happiness effect for a citizen at a given grid position.
///
/// Returns a value between `-POSTAL_NO_COVERAGE_PENALTY` (no coverage) and
/// `+POSTAL_HAPPINESS_BONUS` (full coverage at 255).
#[inline]
pub fn postal_happiness_bonus(coverage: &PostalCoverage, grid_x: usize, grid_y: usize) -> f32 {
    let level = coverage.get(grid_x, grid_y);
    if level == 0 {
        return -POSTAL_NO_COVERAGE_PENALTY;
    }
    (level as f32 / 255.0) * POSTAL_HAPPINESS_BONUS
}

/// Compute the commercial productivity multiplier from a postal coverage level.
///
/// Returns `1.0` (no boost) when coverage is 0, up to `1.0 + POSTAL_COMMERCIAL_MAX_BOOST`
/// (currently 1.15) at full coverage (255).
#[inline]
pub fn postal_commercial_multiplier(coverage: &PostalCoverage, x: usize, y: usize) -> f32 {
    postal_commercial_multiplier_from_level(coverage.get(x, y))
}

#[inline]
fn postal_commercial_multiplier_from_level(level: u8) -> f32 {
    1.0 + (level as f32 / 255.0) * POSTAL_COMMERCIAL_MAX_BOOST
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postal_coverage_default() {
        let cov = PostalCoverage::default();
        assert_eq!(cov.levels.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!(cov.levels.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_postal_coverage_clear() {
        let mut cov = PostalCoverage::default();
        let idx = PostalCoverage::idx(10, 10);
        cov.levels[idx] = 200;
        cov.clear();
        assert_eq!(cov.levels[idx], 0);
    }

    #[test]
    fn test_postal_happiness_bonus_zero_gives_penalty() {
        let cov = PostalCoverage::default();
        let bonus = postal_happiness_bonus(&cov, 10, 10);
        assert!((bonus + POSTAL_NO_COVERAGE_PENALTY).abs() < 0.01);
    }

    #[test]
    fn test_postal_happiness_bonus_full() {
        let mut cov = PostalCoverage::default();
        let idx = PostalCoverage::idx(10, 10);
        cov.levels[idx] = 255;
        let bonus = postal_happiness_bonus(&cov, 10, 10);
        assert!((bonus - POSTAL_HAPPINESS_BONUS).abs() < 0.01);
    }

    #[test]
    fn test_postal_happiness_bonus_half() {
        let mut cov = PostalCoverage::default();
        let idx = PostalCoverage::idx(10, 10);
        cov.levels[idx] = 128;
        let bonus = postal_happiness_bonus(&cov, 10, 10);
        // 128/255 * 5.0 ~ 2.51
        assert!(bonus > 2.0 && bonus < 3.0);
    }

    #[test]
    fn test_postal_stats_default() {
        let stats = PostalStats::default();
        assert_eq!(stats.total_covered_cells, 0);
        assert_eq!(stats.coverage_percentage, 0.0);
        assert_eq!(stats.monthly_cost, 0.0);
        assert_eq!(stats.covered_commercial_buildings, 0);
        assert_eq!(stats.avg_commercial_productivity, 0.0);
    }

    #[test]
    fn test_postal_coverage_idx() {
        let idx = PostalCoverage::idx(5, 10);
        assert_eq!(idx, 10 * GRID_WIDTH + 5);
    }

    #[test]
    fn test_postal_coverage_saturating_add() {
        let mut cov = PostalCoverage::default();
        let idx = PostalCoverage::idx(15, 15);
        cov.levels[idx] = 200;
        cov.levels[idx] = cov.levels[idx].saturating_add(100);
        assert_eq!(cov.levels[idx], 255); // capped at 255
    }

    #[test]
    fn test_commercial_multiplier_zero_coverage() {
        let cov = PostalCoverage::default();
        let mult = postal_commercial_multiplier(&cov, 10, 10);
        assert!((mult - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_commercial_multiplier_full_coverage() {
        let mut cov = PostalCoverage::default();
        let idx = PostalCoverage::idx(10, 10);
        cov.levels[idx] = 255;
        let mult = postal_commercial_multiplier(&cov, 10, 10);
        let expected = 1.0 + POSTAL_COMMERCIAL_MAX_BOOST;
        assert!((mult - expected).abs() < 0.001);
    }

    #[test]
    fn test_commercial_multiplier_half_coverage() {
        let mut cov = PostalCoverage::default();
        let idx = PostalCoverage::idx(10, 10);
        cov.levels[idx] = 128;
        let mult = postal_commercial_multiplier(&cov, 10, 10);
        // 1.0 + (128/255) * 0.15 ~ 1.075
        assert!(mult > 1.05 && mult < 1.10);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut stats = PostalStats::default();
        stats.total_covered_cells = 1000;
        stats.coverage_percentage = 42.5;
        stats.monthly_cost = 200.0;
        stats.covered_commercial_buildings = 50;
        stats.avg_commercial_productivity = 1.08;
        let bytes = stats.save_to_bytes().expect("should serialize");
        let restored = PostalStats::load_from_bytes(&bytes);
        assert_eq!(restored.total_covered_cells, 1000);
        assert!((restored.coverage_percentage - 42.5).abs() < 0.01);
        assert!((restored.monthly_cost - 200.0).abs() < 0.01);
        assert_eq!(restored.covered_commercial_buildings, 50);
        assert!((restored.avg_commercial_productivity - 1.08).abs() < 0.01);
    }

    #[test]
    fn test_apply_coverage_circle_basic() {
        let mut cov = PostalCoverage::default();
        apply_coverage_circle(&mut cov, 50, 50, CELL_SIZE * 5.0, ServiceType::PostOffice);
        assert!(cov.get(50, 50) > 0, "Center should have coverage");
        assert!(cov.get(52, 50) > 0, "Nearby cell should have coverage");
        assert_eq!(cov.get(200, 200), 0, "Far cell should have no coverage");
    }
}

pub struct PostalPlugin;

impl Plugin for PostalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PostalCoverage>()
            .init_resource::<PostalStats>()
            .add_systems(
                FixedUpdate,
                update_postal_coverage
                    .after(crate::traffic::update_traffic_density)
                    .before(crate::happiness::update_service_coverage)
                    .in_set(crate::SimulationSet::Simulation),
            );

        use save::SaveableAppExt;
        app.register_saveable::<PostalStats>();
    }
}
