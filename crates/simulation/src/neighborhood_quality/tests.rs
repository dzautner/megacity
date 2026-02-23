//! Tests for the neighborhood quality index module.

#[cfg(test)]
mod tests {
    use crate::crime::CrimeGrid;
    use crate::districts::{DISTRICTS_X, DISTRICTS_Y};
    use crate::happiness::{
        ServiceCoverageGrid, COVERAGE_EDUCATION, COVERAGE_FIRE, COVERAGE_HEALTH, COVERAGE_PARK,
        COVERAGE_POLICE,
    };
    use crate::noise::NoisePollutionGrid;
    use crate::pollution::PollutionGrid;

    use crate::neighborhood_quality::compute::{
        compute_composite_index, compute_environment_quality, compute_park_access, compute_safety,
        compute_service_coverage,
    };
    use crate::neighborhood_quality::types::{
        DistrictQuality, NeighborhoodQualityIndex, WEIGHT_BUILDING_QUALITY, WEIGHT_CRIME,
        WEIGHT_ENVIRONMENT, WEIGHT_PARK_ACCESS, WEIGHT_SERVICE_COVERAGE, WEIGHT_WALKABILITY,
    };

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
