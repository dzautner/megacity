//! Unit tests for walkability scoring.

#[cfg(test)]
mod tests {
    use crate::grid::ZoneType;
    use crate::services::ServiceType;
    use crate::walkability::categories::*;
    use crate::walkability::grid::*;
    use crate::walkability::scoring::*;

    // -------------------------------------------------------------------------
    // Distance decay tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_decay_within_full_radius() {
        assert!((distance_decay(0.0) - 1.0).abs() < f32::EPSILON);
        assert!((distance_decay(10.0) - 1.0).abs() < f32::EPSILON);
        assert!((distance_decay(25.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_decay_at_max_radius() {
        assert!((distance_decay(100.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_decay_beyond_max() {
        assert!((distance_decay(150.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_decay_midpoint() {
        // At 62.5 cells (halfway between 25 and 100)
        let mid = (FULL_SCORE_RADIUS + MAX_WALK_RADIUS) / 2.0;
        let expected = 0.5;
        assert!((distance_decay(mid) - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn test_decay_monotonic() {
        let mut prev = distance_decay(0.0);
        for d in 1..=110 {
            let current = distance_decay(d as f32);
            assert!(
                current <= prev,
                "decay should be monotonically non-increasing"
            );
            prev = current;
        }
    }

    // -------------------------------------------------------------------------
    // Category weight tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_weights_sum_to_one() {
        let sum = WEIGHT_GROCERY
            + WEIGHT_SCHOOL
            + WEIGHT_HEALTHCARE
            + WEIGHT_PARK
            + WEIGHT_TRANSIT
            + WEIGHT_EMPLOYMENT;
        assert!(
            (sum - 1.0).abs() < f32::EPSILON,
            "category weights must sum to 1.0, got {}",
            sum
        );
    }

    #[test]
    fn test_category_weights_match_constants() {
        assert!((WalkabilityCategory::Grocery.weight() - WEIGHT_GROCERY).abs() < f32::EPSILON);
        assert!((WalkabilityCategory::School.weight() - WEIGHT_SCHOOL).abs() < f32::EPSILON);
        assert!(
            (WalkabilityCategory::Healthcare.weight() - WEIGHT_HEALTHCARE).abs() < f32::EPSILON
        );
        assert!((WalkabilityCategory::Park.weight() - WEIGHT_PARK).abs() < f32::EPSILON);
        assert!((WalkabilityCategory::Transit.weight() - WEIGHT_TRANSIT).abs() < f32::EPSILON);
        assert!(
            (WalkabilityCategory::Employment.weight() - WEIGHT_EMPLOYMENT).abs() < f32::EPSILON
        );
    }

    // -------------------------------------------------------------------------
    // Classification tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_classify_hospital_as_healthcare() {
        assert_eq!(
            classify_service(ServiceType::Hospital),
            Some(WalkabilityCategory::Healthcare)
        );
    }

    #[test]
    fn test_classify_clinic_as_healthcare() {
        assert_eq!(
            classify_service(ServiceType::MedicalClinic),
            Some(WalkabilityCategory::Healthcare)
        );
    }

    #[test]
    fn test_classify_school_as_school() {
        assert_eq!(
            classify_service(ServiceType::ElementarySchool),
            Some(WalkabilityCategory::School)
        );
        assert_eq!(
            classify_service(ServiceType::HighSchool),
            Some(WalkabilityCategory::School)
        );
        assert_eq!(
            classify_service(ServiceType::University),
            Some(WalkabilityCategory::School)
        );
    }

    #[test]
    fn test_classify_park_as_park() {
        assert_eq!(
            classify_service(ServiceType::SmallPark),
            Some(WalkabilityCategory::Park)
        );
        assert_eq!(
            classify_service(ServiceType::LargePark),
            Some(WalkabilityCategory::Park)
        );
        assert_eq!(
            classify_service(ServiceType::Playground),
            Some(WalkabilityCategory::Park)
        );
    }

    #[test]
    fn test_classify_transit() {
        assert_eq!(
            classify_service(ServiceType::BusDepot),
            Some(WalkabilityCategory::Transit)
        );
        assert_eq!(
            classify_service(ServiceType::TrainStation),
            Some(WalkabilityCategory::Transit)
        );
        assert_eq!(
            classify_service(ServiceType::SubwayStation),
            Some(WalkabilityCategory::Transit)
        );
    }

    #[test]
    fn test_classify_fire_station_is_none() {
        assert_eq!(classify_service(ServiceType::FireStation), None);
    }

    #[test]
    fn test_classify_commercial_zone_as_grocery() {
        assert_eq!(
            classify_zone(ZoneType::CommercialLow),
            Some(WalkabilityCategory::Grocery)
        );
        assert_eq!(
            classify_zone(ZoneType::CommercialHigh),
            Some(WalkabilityCategory::Grocery)
        );
        assert_eq!(
            classify_zone(ZoneType::MixedUse),
            Some(WalkabilityCategory::Grocery)
        );
    }

    #[test]
    fn test_classify_industrial_as_employment() {
        assert_eq!(
            classify_zone(ZoneType::Industrial),
            Some(WalkabilityCategory::Employment)
        );
        assert_eq!(
            classify_zone(ZoneType::Office),
            Some(WalkabilityCategory::Employment)
        );
    }

    #[test]
    fn test_classify_residential_is_none() {
        assert_eq!(classify_zone(ZoneType::ResidentialLow), None);
        assert_eq!(classify_zone(ZoneType::ResidentialHigh), None);
    }

    // -------------------------------------------------------------------------
    // Category score tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_category_score_no_amenities() {
        let positions: Vec<(usize, usize)> = vec![];
        let score = category_score_for_cell(128, 128, &positions);
        assert!(score.abs() < f32::EPSILON);
    }

    #[test]
    fn test_category_score_adjacent_amenity() {
        let positions = vec![(128, 128)];
        let score = category_score_for_cell(128, 128, &positions);
        assert!((score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_category_score_nearby_amenity() {
        let positions = vec![(128, 138)]; // 10 cells away
        let score = category_score_for_cell(128, 128, &positions);
        assert!((score - 1.0).abs() < f32::EPSILON); // within full-score radius
    }

    #[test]
    fn test_category_score_distant_amenity() {
        let positions = vec![(128, 228)]; // 100 cells away
        let score = category_score_for_cell(128, 128, &positions);
        assert!(score.abs() < 0.01); // at or beyond max walk radius
    }

    #[test]
    fn test_category_score_picks_nearest() {
        let positions = vec![(128, 228), (128, 130)]; // far and near
        let score = category_score_for_cell(128, 128, &positions);
        // Should pick the near one (2 cells away -> full score)
        assert!((score - 1.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Composite score tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_composite_all_categories_full() {
        // If all categories score 1.0, composite should be 1.0 (100)
        let composite = 1.0 * WEIGHT_GROCERY
            + 1.0 * WEIGHT_SCHOOL
            + 1.0 * WEIGHT_HEALTHCARE
            + 1.0 * WEIGHT_PARK
            + 1.0 * WEIGHT_TRANSIT
            + 1.0 * WEIGHT_EMPLOYMENT;
        assert!((composite - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_composite_no_categories() {
        // If all categories score 0, composite should be 0
        let composite = 0.0 * WEIGHT_GROCERY
            + 0.0 * WEIGHT_SCHOOL
            + 0.0 * WEIGHT_HEALTHCARE
            + 0.0 * WEIGHT_PARK
            + 0.0 * WEIGHT_TRANSIT
            + 0.0 * WEIGHT_EMPLOYMENT;
        assert!(composite.abs() < f32::EPSILON);
    }

    #[test]
    fn test_composite_only_grocery() {
        // Only grocery scores 1.0, rest are 0
        let composite = 1.0 * WEIGHT_GROCERY;
        let expected = WEIGHT_GROCERY;
        assert!((composite - expected).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Grid tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_grid_default_all_zero() {
        let grid = WalkabilityGrid::default();
        assert!(grid.scores.iter().all(|&s| s == 0));
        assert!(grid.city_average.abs() < f32::EPSILON);
    }

    #[test]
    fn test_grid_get_set() {
        let mut grid = WalkabilityGrid::default();
        grid.set(10, 20, 75);
        assert_eq!(grid.get(10, 20), 75);
    }

    #[test]
    fn test_grid_fraction() {
        let mut grid = WalkabilityGrid::default();
        grid.set(5, 5, 50);
        assert!((grid.fraction(5, 5) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_grid_fraction_full() {
        let mut grid = WalkabilityGrid::default();
        grid.set(5, 5, 100);
        assert!((grid.fraction(5, 5) - 1.0).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Saveable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skips_default() {
        use crate::Saveable;
        let grid = WalkabilityGrid::default();
        assert!(grid.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_saves_when_non_zero() {
        use crate::Saveable;
        let mut grid = WalkabilityGrid::default();
        grid.set(10, 10, 50);
        assert!(grid.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut grid = WalkabilityGrid::default();
        grid.set(50, 50, 85);
        grid.city_average = 42.5;

        let bytes = grid.save_to_bytes().expect("should serialize");
        let restored = WalkabilityGrid::load_from_bytes(&bytes);

        assert_eq!(restored.get(50, 50), 85);
        assert!((restored.city_average - 42.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_saveable_key() {
        use crate::Saveable;
        assert_eq!(WalkabilityGrid::SAVE_KEY, "walkability");
    }

    // -------------------------------------------------------------------------
    // Constant value verification
    // -------------------------------------------------------------------------

    #[test]
    fn test_full_score_radius() {
        assert_eq!(FULL_SCORE_RADIUS, 25.0);
    }

    #[test]
    fn test_max_walk_radius() {
        assert_eq!(MAX_WALK_RADIUS, 100.0);
    }

    #[test]
    fn test_happiness_bonus_positive() {
        assert!(WALKABILITY_HAPPINESS_BONUS > 0.0);
    }

    #[test]
    fn test_land_value_bonus_positive() {
        assert!(WALKABILITY_LAND_VALUE_BONUS > 0);
    }
}
