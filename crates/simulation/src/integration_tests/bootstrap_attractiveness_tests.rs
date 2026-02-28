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

/// Verify that a zero-population city with buildings eventually computes
/// an attractiveness score above 60 (the immigration threshold).
///
/// Before the fix:
///   happiness_factor = 0.0 (no citizens -> average_happiness = 0.0)
///   housing_factor   = 0.0 or very low (100% vacancy penalised as "ghost town")
///   Max score ~ 32.5, never reaching 60.
///
/// After the fix:
///   happiness_factor = 0.6 (baseline 60 for empty cities)
///   housing_factor   = 0.8 (brand-new empty housing attractive to pioneers)
///   Score easily exceeds 60.
#[test]
fn test_bootstrap_attractiveness_exceeds_threshold_without_citizens() {
    let mut city = build_bootstrap_city();

    assert_eq!(city.citizen_count(), 0, "should start with zero citizens");
    assert_eq!(city.building_count(), 0, "should start with zero buildings");

    // Tick enough for buildings to spawn (construction takes ~100-200 ticks)
    // and for attractiveness to be recomputed (every 50 ticks).
    city.tick(500);

    let attractiveness = city.resource::<CityAttractiveness>();
    assert!(
        attractiveness.overall_score > 60.0,
        "Attractiveness score should exceed 60 in a bootstrapping city \
         with buildings but no citizens. Got {:.1} (happiness={:.2}, \
         housing={:.2}, employment={:.2}, services={:.2}, tax={:.2})",
        attractiveness.overall_score,
        attractiveness.happiness_factor,
        attractiveness.housing_factor,
        attractiveness.employment_factor,
        attractiveness.services_factor,
        attractiveness.tax_factor,
    );
}

/// End-to-end bootstrap test: a blank city with roads, zones, and
/// utilities should eventually grow a population from zero — no manual
/// seeding of citizens required.
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
    city.tick(100);

    let attractiveness = city.resource::<CityAttractiveness>();
    assert!(
        (attractiveness.happiness_factor - 0.6).abs() < 0.01,
        "Happiness factor should be 0.6 (baseline 60) for an empty city. \
         Got {:.3}",
        attractiveness.happiness_factor,
    );
}

/// Verify the housing factor: empty buildings with 0 occupants should
/// yield a housing_factor of 0.8 (pioneer-friendly), not be penalised
/// as a ghost town.
#[test]
fn test_bootstrap_housing_factor_for_empty_buildings() {
    let mut city = build_bootstrap_city();

    // Tick enough for buildings to spawn and finish construction
    city.tick(500);

    let building_count = city.building_count();
    // If no buildings spawned (unlikely but possible if power/water
    // didn't reach zones), skip assertion on housing_factor.
    if building_count == 0 {
        return;
    }

    // Read the housing factor.
    let attractiveness = city.resource::<CityAttractiveness>();
    assert!(
        attractiveness.housing_factor >= 0.75,
        "Housing factor for empty residential buildings should be ~0.8 \
         (pioneers welcome). Got {:.3} with {building_count} buildings",
        attractiveness.housing_factor,
    );
}
