//! SVC-009: Integration tests for Postal Service Coverage.

use crate::grid::ZoneType;
use crate::postal::{
    postal_commercial_multiplier, postal_happiness_bonus, PostalCoverage, PostalStats,
    POSTAL_COMMERCIAL_MAX_BOOST, POSTAL_HAPPINESS_BONUS, POSTAL_NO_COVERAGE_PENALTY,
};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

fn tick_slow(city: &mut TestCity) {
    city.tick_slow_cycles(1);
}

// ====================================================================
// 1. Resource initialization
// ====================================================================

#[test]
fn test_postal_resources_initialized() {
    let city = TestCity::new();
    let coverage = city.resource::<PostalCoverage>();
    assert!(
        coverage.levels.iter().all(|&v| v == 0),
        "All coverage should start at zero"
    );
    let stats = city.resource::<PostalStats>();
    assert_eq!(stats.total_covered_cells, 0);
    assert_eq!(stats.coverage_percentage, 0.0);
    assert_eq!(stats.monthly_cost, 0.0);
    assert_eq!(stats.covered_commercial_buildings, 0);
}

// ====================================================================
// 2. Post office coverage calculation
// ====================================================================

#[test]
fn test_post_office_provides_coverage() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::PostOffice);
    tick_slow(&mut city);
    let coverage = city.resource::<PostalCoverage>();
    assert!(
        coverage.get(50, 50) > 0,
        "Post office cell should have postal coverage"
    );
}

#[test]
fn test_post_office_coverage_radius() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::PostOffice);
    tick_slow(&mut city);
    let coverage = city.resource::<PostalCoverage>();

    // PostOffice radius is 12 * CELL_SIZE = 192 world units = 12 cells
    assert!(
        coverage.get(55, 50) > 0,
        "Cell 5 away should be within post office radius"
    );
    assert!(
        coverage.get(58, 50) > 0,
        "Cell 8 away should be within post office radius"
    );
    assert_eq!(
        coverage.get(200, 200),
        0,
        "Far-away cell should have no coverage"
    );
}

#[test]
fn test_coverage_falls_off_with_distance() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::PostOffice);
    tick_slow(&mut city);
    let coverage = city.resource::<PostalCoverage>();

    let center = coverage.get(50, 50);
    let nearby = coverage.get(53, 50);
    let farther = coverage.get(58, 50);
    assert!(
        center > nearby,
        "Center ({}) should have more coverage than nearby ({})",
        center,
        nearby
    );
    assert!(
        nearby > farther,
        "Nearby ({}) should have more coverage than farther ({})",
        nearby,
        farther
    );
}

#[test]
fn test_no_coverage_without_postal_buildings() {
    let mut city = TestCity::new();
    tick_slow(&mut city);
    let coverage = city.resource::<PostalCoverage>();
    assert!(
        coverage.levels.iter().all(|&v| v == 0),
        "No postal buildings means no coverage"
    );
}

// ====================================================================
// 3. Mail sorting center doubles post office radius
// ====================================================================

#[test]
fn test_mail_sorting_center_provides_coverage() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::MailSortingCenter);
    tick_slow(&mut city);
    let coverage = city.resource::<PostalCoverage>();
    assert!(
        coverage.get(50, 50) > 0,
        "Mail sorting center should provide some coverage"
    );
}

#[test]
fn test_sorting_center_boosts_post_office_radius() {
    // Post office alone has radius 12 cells
    let mut city_solo = TestCity::new().with_service(50, 50, ServiceType::PostOffice);
    tick_slow(&mut city_solo);
    let cov_solo = city_solo.resource::<PostalCoverage>();
    let solo_far = cov_solo.get(65, 50); // 15 cells away

    // Post office + sorting center nearby: radius should double to 24 cells
    let mut city_boosted = TestCity::new()
        .with_service(50, 50, ServiceType::PostOffice)
        .with_service(51, 50, ServiceType::MailSortingCenter);
    tick_slow(&mut city_boosted);
    let cov_boosted = city_boosted.resource::<PostalCoverage>();
    let boosted_far = cov_boosted.get(65, 50);

    assert!(
        boosted_far > solo_far,
        "Post office boosted by sorting center should cover farther cells: boosted={}, solo={}",
        boosted_far,
        solo_far
    );
}

// ====================================================================
// 4. Happiness effects
// ====================================================================

#[test]
fn test_happiness_bonus_with_coverage() {
    let mut cov = PostalCoverage::default();
    let idx = PostalCoverage::idx(50, 50);
    cov.levels[idx] = 255;
    let bonus = postal_happiness_bonus(&cov, 50, 50);
    assert!(
        (bonus - POSTAL_HAPPINESS_BONUS).abs() < 0.01,
        "Full coverage should give max bonus: got {}",
        bonus
    );
}

#[test]
fn test_happiness_penalty_without_coverage() {
    let cov = PostalCoverage::default();
    let penalty = postal_happiness_bonus(&cov, 50, 50);
    assert!(
        (penalty + POSTAL_NO_COVERAGE_PENALTY).abs() < 0.01,
        "No coverage should give penalty: got {}",
        penalty
    );
}

#[test]
fn test_happiness_partial_coverage() {
    let mut cov = PostalCoverage::default();
    let idx = PostalCoverage::idx(50, 50);
    cov.levels[idx] = 128;
    let bonus = postal_happiness_bonus(&cov, 50, 50);
    assert!(
        bonus > 0.0 && bonus < POSTAL_HAPPINESS_BONUS,
        "Partial coverage should give partial bonus: got {}",
        bonus
    );
}

// ====================================================================
// 5. Commercial productivity boost
// ====================================================================

#[test]
fn test_commercial_productivity_no_coverage() {
    let cov = PostalCoverage::default();
    let mult = postal_commercial_multiplier(&cov, 50, 50);
    assert!(
        (mult - 1.0).abs() < 0.001,
        "No coverage should give 1.0x multiplier"
    );
}

#[test]
fn test_commercial_productivity_full_coverage() {
    let mut cov = PostalCoverage::default();
    let idx = PostalCoverage::idx(50, 50);
    cov.levels[idx] = 255;
    let mult = postal_commercial_multiplier(&cov, 50, 50);
    let expected = 1.0 + POSTAL_COMMERCIAL_MAX_BOOST;
    assert!(
        (mult - expected).abs() < 0.001,
        "Full coverage should give {}x multiplier, got {}",
        expected,
        mult
    );
}

#[test]
fn test_commercial_stats_tracked() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::PostOffice)
        .with_building(51, 50, ZoneType::CommercialLow, 1)
        .with_building(200, 200, ZoneType::CommercialLow, 1);
    tick_slow(&mut city);

    let stats = city.resource::<PostalStats>();
    assert_eq!(
        stats.covered_commercial_buildings, 1,
        "Only the commercial building near the post office should be covered"
    );
    assert!(
        stats.avg_commercial_productivity > 1.0,
        "Average productivity should be above 1.0 with one covered building: got {}",
        stats.avg_commercial_productivity
    );
}

// ====================================================================
// 6. Postal stats aggregate tracking
// ====================================================================

#[test]
fn test_postal_stats_update_on_slow_tick() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::PostOffice);
    tick_slow(&mut city);

    let stats = city.resource::<PostalStats>();
    assert!(
        stats.total_covered_cells > 0,
        "Should have covered cells after placing a post office"
    );
    assert!(
        stats.coverage_percentage > 0.0,
        "Coverage percentage should be positive"
    );
    assert!(
        stats.monthly_cost > 0.0,
        "Monthly cost should be positive with a post office"
    );
}

#[test]
fn test_postal_stats_zero_without_buildings() {
    let mut city = TestCity::new();
    tick_slow(&mut city);

    let stats = city.resource::<PostalStats>();
    assert_eq!(stats.total_covered_cells, 0);
    assert_eq!(stats.monthly_cost, 0.0);
}

// ====================================================================
// 7. Saveable roundtrip
// ====================================================================

#[test]
fn test_postal_stats_saveable_roundtrip() {
    use crate::Saveable;
    let mut stats = PostalStats::default();
    stats.total_covered_cells = 5000;
    stats.coverage_percentage = 75.5;
    stats.monthly_cost = 350.0;
    stats.covered_commercial_buildings = 42;
    stats.avg_commercial_productivity = 1.12;
    let bytes = stats.save_to_bytes().expect("should serialize");
    let restored = PostalStats::load_from_bytes(&bytes);
    assert_eq!(restored.total_covered_cells, 5000);
    assert!((restored.coverage_percentage - 75.5).abs() < 0.01);
    assert!((restored.monthly_cost - 350.0).abs() < 0.01);
    assert_eq!(restored.covered_commercial_buildings, 42);
    assert!((restored.avg_commercial_productivity - 1.12).abs() < 0.01);
}

// ====================================================================
// 8. Constants validation
// ====================================================================

#[test]
fn test_postal_constants_reasonable() {
    assert!(POSTAL_HAPPINESS_BONUS > 0.0);
    assert!(POSTAL_HAPPINESS_BONUS <= 10.0);
    assert!(POSTAL_NO_COVERAGE_PENALTY > 0.0);
    assert!(POSTAL_NO_COVERAGE_PENALTY <= 5.0);
    assert!(POSTAL_COMMERCIAL_MAX_BOOST > 0.0);
    assert!(POSTAL_COMMERCIAL_MAX_BOOST <= 0.5);
}

// ====================================================================
// 9. Multiple post offices stack coverage
// ====================================================================

#[test]
fn test_multiple_post_offices_stack_coverage() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::PostOffice)
        .with_service(55, 50, ServiceType::PostOffice);
    tick_slow(&mut city);
    let coverage = city.resource::<PostalCoverage>();

    // Cell between two post offices should have stacked coverage
    let between = coverage.get(52, 50);
    assert!(
        between > 0,
        "Cell between two post offices should have coverage"
    );

    // Compare with single post office
    let mut city_single = TestCity::new().with_service(50, 50, ServiceType::PostOffice);
    tick_slow(&mut city_single);
    let cov_single = city_single.resource::<PostalCoverage>();
    let single = cov_single.get(52, 50);

    assert!(
        between >= single,
        "Stacked coverage ({}) should be >= single ({})",
        between,
        single
    );
}
