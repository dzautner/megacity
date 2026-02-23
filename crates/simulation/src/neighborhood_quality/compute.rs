//! Pure computation functions for neighborhood quality sub-scores.
//!
//! These functions are free of ECS dependencies and can be tested in isolation.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::crime::CrimeGrid;
use crate::grid::{CellType, WorldGrid};
use crate::happiness::{
    ServiceCoverageGrid, COVERAGE_EDUCATION, COVERAGE_FIRE, COVERAGE_HEALTH, COVERAGE_PARK,
    COVERAGE_POLICE,
};
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;

use super::types::{
    DistrictQuality, WEIGHT_BUILDING_QUALITY, WEIGHT_CRIME, WEIGHT_ENVIRONMENT, WEIGHT_PARK_ACCESS,
    WEIGHT_SERVICE_COVERAGE, WEIGHT_WALKABILITY,
};

/// Compute walkability score for a district region.
///
/// Walkability is the fraction of cells in the district that have a road or
/// path cell within a 2-cell radius. Higher road density near buildings means
/// better walkability.
pub fn compute_walkability(grid: &WorldGrid, start_x: usize, start_y: usize, size: usize) -> f32 {
    let end_x = (start_x + size).min(GRID_WIDTH);
    let end_y = (start_y + size).min(GRID_HEIGHT);
    let mut walkable_cells = 0u32;
    let mut total_cells = 0u32;

    for y in start_y..end_y {
        for x in start_x..end_x {
            total_cells += 1;
            // Check if any cell within radius 2 is a road
            let radius = 2i32;
            let mut has_road_nearby = false;
            for dy in -radius..=radius {
                if has_road_nearby {
                    break;
                }
                for dx in -radius..=radius {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                        && grid.get(nx as usize, ny as usize).cell_type == CellType::Road
                    {
                        has_road_nearby = true;
                        break;
                    }
                }
            }
            if has_road_nearby {
                walkable_cells += 1;
            }
        }
    }

    if total_cells == 0 {
        return 0.0;
    }
    walkable_cells as f32 / total_cells as f32
}

/// Compute service coverage score for a district region.
///
/// Returns the average fraction of cells covered by health, education, police,
/// and fire services.
pub fn compute_service_coverage(
    coverage: &ServiceCoverageGrid,
    start_x: usize,
    start_y: usize,
    size: usize,
) -> f32 {
    let end_x = (start_x + size).min(GRID_WIDTH);
    let end_y = (start_y + size).min(GRID_HEIGHT);
    let mut health_count = 0u32;
    let mut edu_count = 0u32;
    let mut police_count = 0u32;
    let mut fire_count = 0u32;
    let mut total = 0u32;

    for y in start_y..end_y {
        for x in start_x..end_x {
            total += 1;
            let idx = ServiceCoverageGrid::idx(x, y);
            let flags = coverage.flags[idx];
            if flags & COVERAGE_HEALTH != 0 {
                health_count += 1;
            }
            if flags & COVERAGE_EDUCATION != 0 {
                edu_count += 1;
            }
            if flags & COVERAGE_POLICE != 0 {
                police_count += 1;
            }
            if flags & COVERAGE_FIRE != 0 {
                fire_count += 1;
            }
        }
    }

    if total == 0 {
        return 0.0;
    }
    let health_frac = health_count as f32 / total as f32;
    let edu_frac = edu_count as f32 / total as f32;
    let police_frac = police_count as f32 / total as f32;
    let fire_frac = fire_count as f32 / total as f32;
    ((health_frac + edu_frac + police_frac + fire_frac) / 4.0).clamp(0.0, 1.0)
}

/// Compute environment quality score for a district region.
///
/// Environment quality is the inverse of average pollution and noise levels.
/// A district with zero pollution and noise scores 1.0.
pub fn compute_environment_quality(
    pollution: &PollutionGrid,
    noise: &NoisePollutionGrid,
    start_x: usize,
    start_y: usize,
    size: usize,
) -> f32 {
    let end_x = (start_x + size).min(GRID_WIDTH);
    let end_y = (start_y + size).min(GRID_HEIGHT);
    let mut pollution_sum = 0u32;
    let mut noise_sum = 0u32;
    let mut total = 0u32;

    for y in start_y..end_y {
        for x in start_x..end_x {
            total += 1;
            pollution_sum += pollution.get(x, y) as u32;
            noise_sum += noise.get(x, y) as u32;
        }
    }

    if total == 0 {
        return 0.0;
    }
    // Normalize pollution: 0 avg -> 1.0, 100+ avg -> 0.0
    let avg_pollution = pollution_sum as f32 / total as f32;
    let pollution_score = (1.0 - avg_pollution / 100.0).clamp(0.0, 1.0);

    // Normalize noise: 0 avg -> 1.0, 100 avg -> 0.0
    let avg_noise = noise_sum as f32 / total as f32;
    let noise_score = (1.0 - avg_noise / 100.0).clamp(0.0, 1.0);

    // Equal weight between pollution and noise
    ((pollution_score + noise_score) / 2.0).clamp(0.0, 1.0)
}

/// Compute safety score (inverse of crime) for a district region.
///
/// A district with zero crime scores 1.0; one with average crime 50+ scores 0.0.
pub fn compute_safety(crime: &CrimeGrid, start_x: usize, start_y: usize, size: usize) -> f32 {
    let end_x = (start_x + size).min(GRID_WIDTH);
    let end_y = (start_y + size).min(GRID_HEIGHT);
    let mut crime_sum = 0u32;
    let mut total = 0u32;

    for y in start_y..end_y {
        for x in start_x..end_x {
            total += 1;
            crime_sum += crime.get(x, y) as u32;
        }
    }

    if total == 0 {
        return 0.0;
    }
    let avg_crime = crime_sum as f32 / total as f32;
    (1.0 - avg_crime / 50.0).clamp(0.0, 1.0)
}

/// Compute park access score for a district region.
///
/// Returns the fraction of cells that have park coverage.
pub fn compute_park_access(
    coverage: &ServiceCoverageGrid,
    start_x: usize,
    start_y: usize,
    size: usize,
) -> f32 {
    let end_x = (start_x + size).min(GRID_WIDTH);
    let end_y = (start_y + size).min(GRID_HEIGHT);
    let mut park_count = 0u32;
    let mut total = 0u32;

    for y in start_y..end_y {
        for x in start_x..end_x {
            total += 1;
            let idx = ServiceCoverageGrid::idx(x, y);
            if coverage.flags[idx] & COVERAGE_PARK != 0 {
                park_count += 1;
            }
        }
    }

    if total == 0 {
        return 0.0;
    }
    (park_count as f32 / total as f32).clamp(0.0, 1.0)
}

/// Compute the composite quality index from sub-scores.
pub fn compute_composite_index(quality: &DistrictQuality) -> f32 {
    let raw = quality.walkability * WEIGHT_WALKABILITY
        + quality.service_coverage * WEIGHT_SERVICE_COVERAGE
        + quality.environment * WEIGHT_ENVIRONMENT
        + quality.safety * WEIGHT_CRIME
        + quality.park_access * WEIGHT_PARK_ACCESS
        + quality.building_quality * WEIGHT_BUILDING_QUALITY;
    (raw * 100.0).clamp(0.0, 100.0)
}
