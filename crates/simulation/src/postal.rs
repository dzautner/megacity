use bevy::prelude::*;

use crate::budget::ExtendedBudget;
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

/// Aggregate postal statistics for the city.
#[derive(Resource, Default)]
pub struct PostalStats {
    pub total_covered_cells: u32,
    pub coverage_percentage: f32,
    pub monthly_cost: f64,
}

/// Maximum happiness bonus from postal coverage at full coverage (255).
pub const POSTAL_HAPPINESS_BONUS: f32 = 5.0;

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

        let radius_cells = (effective_radius / CELL_SIZE).ceil() as i32;
        let sx = service.grid_x as i32;
        let sy = service.grid_y as i32;
        let r2 = effective_radius * effective_radius;

        // Coverage intensity: sorting centers provide lighter coverage (128 max),
        // post offices provide full coverage (255 max)
        let max_coverage: u8 = match service.service_type {
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

    // Compute stats
    let total_covered = coverage.levels.iter().filter(|&&v| v > 0).count() as u32;
    let total_cells = (GRID_WIDTH * GRID_HEIGHT) as f32;

    stats.total_covered_cells = total_covered;
    stats.coverage_percentage = total_covered as f32 / total_cells * 100.0;
    stats.monthly_cost = monthly_cost;
}

/// Compute the postal happiness bonus for a citizen at a given grid position.
/// Returns a value between 0.0 and POSTAL_HAPPINESS_BONUS (5.0).
#[inline]
pub fn postal_happiness_bonus(coverage: &PostalCoverage, grid_x: usize, grid_y: usize) -> f32 {
    let level = coverage.get(grid_x, grid_y) as f32;
    (level / 255.0) * POSTAL_HAPPINESS_BONUS
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
    fn test_postal_happiness_bonus_zero() {
        let cov = PostalCoverage::default();
        let bonus = postal_happiness_bonus(&cov, 10, 10);
        assert_eq!(bonus, 0.0);
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
}
