//! Integration tests for the bootstrap deadlock fix in city attractiveness.
//!
//! Verifies that a blank city (no citizens, no buildings) can attract its
//! first residents once roads, zones, and utilities are placed. Before
//! this fix, the attractiveness score could never reach 60 in a zero-
//! population city because happiness_factor and housing_factor both
//! evaluated to 0.0.

use crate::grid::{RoadType, ZoneType};
use crate::immigration::CityAttractiveness;
use crate::stats::CityStats;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

/// Build a small bootstrapping city with roads, zones, and utilities but
/// NO citizens and NO pre-placed buildings.  The simulation should spawn
/// buildings on the zoned cells, compute an attractiveness > 60 thanks
/// to the bootstrap fixes, and then immigration should kick in.
fn build_bootstrap_city() -> TestCity {
    TestCity::new()
        .with_budget(500_000.0)
        // Horizontal road through the middle
        .with_road(110, 128, 146, 128, RoadType::Avenue)
        // Residential zones above the road (adjacent for utility BFS)
        .with_zone_rect(112, 125, 144, 127, ZoneType::ResidentialLow)
        // Commercial zones below the road (adjacent for utility BFS)
        .with_zone_rect(112, 129, 130, 131, ZoneType::CommercialLow)
        // Industrial zones further along (for jobs diversity)
        .with_zone_rect(132, 129, 144, 131, ZoneType::Industrial)
        // Power and water on the road for BFS propagation
        .with_utility(110, 128, UtilityType::PowerPlant)
        .with_utility(146, 128, UtilityType::WaterTower)
}

/// End-to-end bootstrap test: a blank city with roads, zones, and
/// utilities should eventually grow a population from zero — no manual
/// seeding of citizens required.
///
/// This is THE test for the bootstrap deadlock fix. Before the fix,
/// population would remain at 0 forever because the attractiveness score
/// could never reach 60.
#[test]
fn test_bootstrap_blank_city_attracts_first_residents() {
    let mut city = build_bootstrap_city();

    assert_eq!(city.citizen_count(), 0, "should start with zero citizens");

    // Run for 2000 ticks:
    //   ~200 ticks for buildings to spawn and finish construction
    //   ~50 ticks for attractiveness to be recomputed (> 60 with fix)
    //   ~100 ticks for the first immigration wave
    //   Additional buffer for multi-wave growth.
    city.tick(2000);

    let stats = city.resource::<CityStats>();
    assert!(
        stats.population > 0,
        "A blank city with zones and utilities should eventually attract \
         residents. Population is still 0 after 2000 ticks."
    );

    let citizen_count = city.citizen_count();
    assert!(
        citizen_count > 0,
        "Citizen entities should exist after immigration. Got {citizen_count}"
    );
}

/// Verify the happiness baseline: with population=0, the happiness
/// factor should be 0.6 (= 60/100), not 0.0.
#[test]
fn test_bootstrap_happiness_baseline_for_empty_city() {
    let mut city = build_bootstrap_city();

    assert_eq!(city.citizen_count(), 0, "should start with zero citizens");

    // Tick past the first attractiveness computation (every 50 ticks)
    // but not long enough for buildings to spawn and immigration to fire.
    city.tick(100);

    let attractiveness = city.resource::<CityAttractiveness>();
    assert!(
        (attractiveness.happiness_factor - 0.6).abs() < 0.01,
        "Happiness factor should be 0.6 (baseline 60) for an empty city. \
         Got {:.3}",
        attractiveness.happiness_factor,
    );
}

/// Verify that the attractiveness score exceeds 60 at some point during
/// the bootstrap phase (before or just after first immigration).
///
/// We check incrementally — after each attractiveness recomputation
/// interval (50 ticks) — to catch the moment the score first exceeds 60.
/// This avoids the race where, by 500 ticks, immigrants have already
/// arrived and changed the factors.
#[test]
fn test_bootstrap_attractiveness_exceeds_threshold_during_growth() {
    let mut city = build_bootstrap_city();

    assert_eq!(city.citizen_count(), 0, "should start with zero citizens");
    assert_eq!(city.building_count(), 0, "should start with zero buildings");

    let mut score_exceeded_60 = false;

    // Check in 50-tick increments (attractiveness recomputes every 50 ticks)
    // over a 2000-tick window.
    for _ in 0..40 {
        city.tick(50);
        let attractiveness = city.resource::<CityAttractiveness>();
        if attractiveness.overall_score > 60.0 {
            score_exceeded_60 = true;
            break;
        }
    }

    assert!(
        score_exceeded_60,
        "Attractiveness score should exceed 60 at some point during the \
         bootstrap phase. Final score: {:.1}",
        city.resource::<CityAttractiveness>().overall_score,
    );
}
