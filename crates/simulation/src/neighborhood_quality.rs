//! Neighborhood Quality Index (ZONE-014).
//!
//! Computes a composite quality index per statistical district combining:
//! - Walkability (20%): road connectivity, paths, and sidewalk density
//! - Service coverage (20%): health, education, police, fire, park coverage
//! - Environment quality (20%): inverse of pollution and noise levels
//! - Crime rate (15%): inverse of crime level
//! - Park access (15%): fraction of cells with park coverage
//! - Building quality average (10%): average building level in district
//!
//! Computed per district on the slow tick. Affects immigration attractiveness
//! at the district level -- high-quality neighborhoods attract higher-income citizens.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::crime::CrimeGrid;
use crate::districts::{Districts, DISTRICTS_X, DISTRICTS_Y, DISTRICT_SIZE};
use crate::grid::{CellType, WorldGrid};
use crate::happiness::{
    ServiceCoverageGrid, COVERAGE_EDUCATION, COVERAGE_FIRE, COVERAGE_HEALTH, COVERAGE_PARK,
    COVERAGE_POLICE,
};
use crate::noise::NoisePollutionGrid;
use crate::pollution::PollutionGrid;
use crate::SlowTickTimer;

// =============================================================================
// Weight constants (must sum to 1.0)
// =============================================================================

/// Walkability weight in the composite index.
pub const WEIGHT_WALKABILITY: f32 = 0.20;
/// Service coverage weight in the composite index.
pub const WEIGHT_SERVICE_COVERAGE: f32 = 0.20;
/// Environment quality (inverse pollution/noise) weight.
pub const WEIGHT_ENVIRONMENT: f32 = 0.20;
/// Crime rate (inverse) weight.
pub const WEIGHT_CRIME: f32 = 0.15;
/// Park access weight.
pub const WEIGHT_PARK_ACCESS: f32 = 0.15;
/// Building quality average weight.
pub const WEIGHT_BUILDING_QUALITY: f32 = 0.10;

/// Maximum building level used for normalization.
const MAX_BUILDING_LEVEL: f32 = 5.0;

// =============================================================================
// Per-district quality data
// =============================================================================

/// Quality index data for a single statistical district.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct DistrictQuality {
    /// Composite quality index (0.0 to 100.0).
    pub overall: f32,
    /// Walkability sub-score (0.0 to 1.0).
    pub walkability: f32,
    /// Service coverage sub-score (0.0 to 1.0).
    pub service_coverage: f32,
    /// Environment quality sub-score (0.0 to 1.0).
    pub environment: f32,
    /// Safety sub-score (inverse of crime, 0.0 to 1.0).
    pub safety: f32,
    /// Park access sub-score (0.0 to 1.0).
    pub park_access: f32,
    /// Building quality sub-score (0.0 to 1.0).
    pub building_quality: f32,
}

// =============================================================================
// Resource: neighborhood quality index per district
// =============================================================================

/// Resource holding the neighborhood quality index for every statistical district.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct NeighborhoodQualityIndex {
    /// One entry per statistical district (DISTRICTS_X * DISTRICTS_Y).
    pub districts: Vec<DistrictQuality>,
    /// City-wide average quality index (0.0 to 100.0).
    pub city_average: f32,
}

impl Default for NeighborhoodQualityIndex {
    fn default() -> Self {
        Self {
            districts: vec![DistrictQuality::default(); DISTRICTS_X * DISTRICTS_Y],
            city_average: 0.0,
        }
    }
}

impl NeighborhoodQualityIndex {
    /// Get the quality data for a given statistical district.
    pub fn get(&self, dx: usize, dy: usize) -> &DistrictQuality {
        &self.districts[dy * DISTRICTS_X + dx]
    }

    /// Get the quality index for the district containing a grid cell.
    pub fn quality_at_cell(&self, gx: usize, gy: usize) -> f32 {
        let (dx, dy) = Districts::district_for_grid(gx, gy);
        if dx < DISTRICTS_X && dy < DISTRICTS_Y {
            self.districts[dy * DISTRICTS_X + dx].overall
        } else {
            0.0
        }
    }
}

// =============================================================================
// Pure computation functions (testable without ECS)
// =============================================================================

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

// =============================================================================
// System: update neighborhood quality index
// =============================================================================

/// System that computes the neighborhood quality index per district on the slow tick.
#[allow(clippy::too_many_arguments)]
pub fn update_neighborhood_quality(
    timer: Res<SlowTickTimer>,
    grid: Res<WorldGrid>,
    coverage: Res<ServiceCoverageGrid>,
    pollution: Res<PollutionGrid>,
    noise: Res<NoisePollutionGrid>,
    crime: Res<CrimeGrid>,
    buildings: Query<&Building>,
    mut quality_index: ResMut<NeighborhoodQualityIndex>,
) {
    if !timer.should_run() {
        return;
    }

    // Pre-compute per-district building quality accumulators
    let num_districts = DISTRICTS_X * DISTRICTS_Y;
    let mut building_level_sum = vec![0u32; num_districts];
    let mut building_count = vec![0u32; num_districts];

    for building in &buildings {
        let (dx, dy) = Districts::district_for_grid(building.grid_x, building.grid_y);
        if dx < DISTRICTS_X && dy < DISTRICTS_Y {
            let idx = dy * DISTRICTS_X + dx;
            building_level_sum[idx] += building.level as u32;
            building_count[idx] += 1;
        }
    }

    let mut city_quality_sum = 0.0f32;
    let mut city_district_count = 0u32;

    for dy in 0..DISTRICTS_Y {
        for dx in 0..DISTRICTS_X {
            let idx = dy * DISTRICTS_X + dx;
            let start_x = dx * DISTRICT_SIZE;
            let start_y = dy * DISTRICT_SIZE;

            let walkability = compute_walkability(&grid, start_x, start_y, DISTRICT_SIZE);
            let service_cov = compute_service_coverage(&coverage, start_x, start_y, DISTRICT_SIZE);
            let environment =
                compute_environment_quality(&pollution, &noise, start_x, start_y, DISTRICT_SIZE);
            let safety = compute_safety(&crime, start_x, start_y, DISTRICT_SIZE);
            let park = compute_park_access(&coverage, start_x, start_y, DISTRICT_SIZE);

            let bq = if building_count[idx] > 0 {
                (building_level_sum[idx] as f32 / building_count[idx] as f32 / MAX_BUILDING_LEVEL)
                    .clamp(0.0, 1.0)
            } else {
                0.0
            };

            let mut dq = DistrictQuality {
                overall: 0.0,
                walkability,
                service_coverage: service_cov,
                environment,
                safety,
                park_access: park,
                building_quality: bq,
            };
            dq.overall = compute_composite_index(&dq);
            quality_index.districts[idx] = dq;

            city_quality_sum += quality_index.districts[idx].overall;
            city_district_count += 1;
        }
    }

    quality_index.city_average = if city_district_count > 0 {
        city_quality_sum / city_district_count as f32
    } else {
        0.0
    };
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl crate::Saveable for NeighborhoodQualityIndex {
    const SAVE_KEY: &'static str = "neighborhood_quality";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if all districts are at default (overall == 0.0)
        let has_data = self.districts.iter().any(|d| d.overall > 0.0);
        if !has_data {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct NeighborhoodQualityPlugin;

impl Plugin for NeighborhoodQualityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NeighborhoodQualityIndex>().add_systems(
            FixedUpdate,
            update_neighborhood_quality.after(crate::crime::update_crime),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<NeighborhoodQualityIndex>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Weight tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_weights_sum_to_one() {
        let total = WEIGHT_WALKABILITY
            + WEIGHT_SERVICE_COVERAGE
            + WEIGHT_ENVIRONMENT
            + WEIGHT_CRIME
            + WEIGHT_PARK_ACCESS
            + WEIGHT_BUILDING_QUALITY;
        assert!(
            (total - 1.0).abs() < f32::EPSILON,
            "weights sum to {} instead of 1.0",
            total
        );
    }

    #[test]
    fn test_weight_walkability() {
        assert!((WEIGHT_WALKABILITY - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn test_weight_service_coverage() {
        assert!((WEIGHT_SERVICE_COVERAGE - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn test_weight_environment() {
        assert!((WEIGHT_ENVIRONMENT - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn test_weight_crime() {
        assert!((WEIGHT_CRIME - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn test_weight_park_access() {
        assert!((WEIGHT_PARK_ACCESS - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn test_weight_building_quality() {
        assert!((WEIGHT_BUILDING_QUALITY - 0.10).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Composite index tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_composite_all_zero() {
        let dq = DistrictQuality::default();
        let index = compute_composite_index(&dq);
        assert!(index.abs() < f32::EPSILON);
    }

    #[test]
    fn test_composite_all_perfect() {
        let dq = DistrictQuality {
            overall: 0.0,
            walkability: 1.0,
            service_coverage: 1.0,
            environment: 1.0,
            safety: 1.0,
            park_access: 1.0,
            building_quality: 1.0,
        };
        let index = compute_composite_index(&dq);
        assert!(
            (index - 100.0).abs() < f32::EPSILON,
            "perfect scores should yield 100.0, got {}",
            index
        );
    }

    #[test]
    fn test_composite_half_scores() {
        let dq = DistrictQuality {
            overall: 0.0,
            walkability: 0.5,
            service_coverage: 0.5,
            environment: 0.5,
            safety: 0.5,
            park_access: 0.5,
            building_quality: 0.5,
        };
        let index = compute_composite_index(&dq);
        assert!(
            (index - 50.0).abs() < f32::EPSILON,
            "half scores should yield 50.0, got {}",
            index
        );
    }

    #[test]
    fn test_composite_only_walkability() {
        let dq = DistrictQuality {
            overall: 0.0,
            walkability: 1.0,
            service_coverage: 0.0,
            environment: 0.0,
            safety: 0.0,
            park_access: 0.0,
            building_quality: 0.0,
        };
        let index = compute_composite_index(&dq);
        let expected = WEIGHT_WALKABILITY * 100.0;
        assert!(
            (index - expected).abs() < f32::EPSILON,
            "expected {}, got {}",
            expected,
            index
        );
    }

    // -------------------------------------------------------------------------
    // Safety computation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_safety_no_crime() {
        let crime = CrimeGrid::default();
        let score = compute_safety(&crime, 0, 0, 16);
        assert!(
            (score - 1.0).abs() < f32::EPSILON,
            "zero crime should yield safety 1.0, got {}",
            score
        );
    }

    #[test]
    fn test_safety_max_crime() {
        let mut crime = CrimeGrid::default();
        // Set all cells in district (0,0) to crime level 50
        for y in 0..16 {
            for x in 0..16 {
                crime.set(x, y, 50);
            }
        }
        let score = compute_safety(&crime, 0, 0, 16);
        assert!(
            score.abs() < f32::EPSILON,
            "max crime (50) should yield safety 0.0, got {}",
            score
        );
    }

    #[test]
    fn test_safety_partial_crime() {
        let mut crime = CrimeGrid::default();
        // Set all cells in district (0,0) to crime level 25
        for y in 0..16 {
            for x in 0..16 {
                crime.set(x, y, 25);
            }
        }
        let score = compute_safety(&crime, 0, 0, 16);
        assert!(
            (score - 0.5).abs() < f32::EPSILON,
            "half crime should yield safety 0.5, got {}",
            score
        );
    }

    // -------------------------------------------------------------------------
    // Environment quality tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_environment_clean() {
        let pollution = PollutionGrid::default();
        let noise = NoisePollutionGrid::default();
        let score = compute_environment_quality(&pollution, &noise, 0, 0, 16);
        assert!(
            (score - 1.0).abs() < f32::EPSILON,
            "zero pollution/noise should yield 1.0, got {}",
            score
        );
    }

    #[test]
    fn test_environment_polluted() {
        let mut pollution = PollutionGrid::default();
        let noise = NoisePollutionGrid::default();
        for y in 0..16 {
            for x in 0..16 {
                pollution.set(x, y, 100);
            }
        }
        let score = compute_environment_quality(&pollution, &noise, 0, 0, 16);
        // Pollution score = 0.0, noise score = 1.0, average = 0.5
        assert!(
            (score - 0.5).abs() < f32::EPSILON,
            "max pollution, no noise should yield 0.5, got {}",
            score
        );
    }

    #[test]
    fn test_environment_noisy() {
        let pollution = PollutionGrid::default();
        let mut noise = NoisePollutionGrid::default();
        for y in 0..16 {
            for x in 0..16 {
                noise.set(x, y, 100);
            }
        }
        let score = compute_environment_quality(&pollution, &noise, 0, 0, 16);
        // Pollution score = 1.0, noise score = 0.0, average = 0.5
        assert!(
            (score - 0.5).abs() < f32::EPSILON,
            "no pollution, max noise should yield 0.5, got {}",
            score
        );
    }

    #[test]
    fn test_environment_both_max() {
        let mut pollution = PollutionGrid::default();
        let mut noise = NoisePollutionGrid::default();
        for y in 0..16 {
            for x in 0..16 {
                pollution.set(x, y, 100);
                noise.set(x, y, 100);
            }
        }
        let score = compute_environment_quality(&pollution, &noise, 0, 0, 16);
        assert!(
            score.abs() < f32::EPSILON,
            "max pollution and noise should yield 0.0, got {}",
            score
        );
    }

    // -------------------------------------------------------------------------
    // Park access tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_park_access_none() {
        let coverage = ServiceCoverageGrid::default();
        let score = compute_park_access(&coverage, 0, 0, 16);
        assert!(
            score.abs() < f32::EPSILON,
            "no park coverage should yield 0.0, got {}",
            score
        );
    }

    #[test]
    fn test_park_access_full() {
        let mut coverage = ServiceCoverageGrid::default();
        for y in 0..16 {
            for x in 0..16 {
                let idx = ServiceCoverageGrid::idx(x, y);
                coverage.flags[idx] |= COVERAGE_PARK;
            }
        }
        let score = compute_park_access(&coverage, 0, 0, 16);
        assert!(
            (score - 1.0).abs() < f32::EPSILON,
            "full park coverage should yield 1.0, got {}",
            score
        );
    }

    // -------------------------------------------------------------------------
    // Service coverage tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_service_coverage_none() {
        let coverage = ServiceCoverageGrid::default();
        let score = compute_service_coverage(&coverage, 0, 0, 16);
        assert!(
            score.abs() < f32::EPSILON,
            "no service coverage should yield 0.0, got {}",
            score
        );
    }

    #[test]
    fn test_service_coverage_full() {
        let mut coverage = ServiceCoverageGrid::default();
        for y in 0..16 {
            for x in 0..16 {
                let idx = ServiceCoverageGrid::idx(x, y);
                coverage.flags[idx] =
                    COVERAGE_HEALTH | COVERAGE_EDUCATION | COVERAGE_POLICE | COVERAGE_FIRE;
            }
        }
        let score = compute_service_coverage(&coverage, 0, 0, 16);
        assert!(
            (score - 1.0).abs() < f32::EPSILON,
            "full service coverage should yield 1.0, got {}",
            score
        );
    }

    #[test]
    fn test_service_coverage_partial() {
        let mut coverage = ServiceCoverageGrid::default();
        for y in 0..16 {
            for x in 0..16 {
                let idx = ServiceCoverageGrid::idx(x, y);
                // Only health and education
                coverage.flags[idx] = COVERAGE_HEALTH | COVERAGE_EDUCATION;
            }
        }
        let score = compute_service_coverage(&coverage, 0, 0, 16);
        // 2 out of 4 services at 100% each => 0.5
        assert!(
            (score - 0.5).abs() < f32::EPSILON,
            "half service coverage should yield 0.5, got {}",
            score
        );
    }

    // -------------------------------------------------------------------------
    // Default resource tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_quality_index() {
        let index = NeighborhoodQualityIndex::default();
        assert_eq!(index.districts.len(), DISTRICTS_X * DISTRICTS_Y);
        assert!(index.city_average.abs() < f32::EPSILON);
        for d in &index.districts {
            assert!(d.overall.abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_quality_at_cell() {
        let mut index = NeighborhoodQualityIndex::default();
        // Set district (1, 1) to quality 75.0
        index.districts[1 * DISTRICTS_X + 1].overall = 75.0;
        // Cell (16, 16) maps to district (1, 1)
        let q = index.quality_at_cell(16, 16);
        assert!((q - 75.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_quality_at_cell_out_of_bounds() {
        let index = NeighborhoodQualityIndex::default();
        let q = index.quality_at_cell(999, 999);
        assert!(q.abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let index = NeighborhoodQualityIndex::default();
        assert!(index.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_has_data() {
        use crate::Saveable;
        let mut index = NeighborhoodQualityIndex::default();
        index.districts[0].overall = 50.0;
        assert!(index.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut index = NeighborhoodQualityIndex::default();
        index.districts[0].overall = 75.0;
        index.districts[0].walkability = 0.8;
        index.districts[0].safety = 0.9;
        index.city_average = 42.0;

        let bytes = index.save_to_bytes().expect("should serialize");
        let restored = NeighborhoodQualityIndex::load_from_bytes(&bytes);

        assert!((restored.districts[0].overall - 75.0).abs() < f32::EPSILON);
        assert!((restored.districts[0].walkability - 0.8).abs() < f32::EPSILON);
        assert!((restored.districts[0].safety - 0.9).abs() < f32::EPSILON);
        assert!((restored.city_average - 42.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(NeighborhoodQualityIndex::SAVE_KEY, "neighborhood_quality");
    }
}
