//! Integration tests for the `HappinessBreakdown` resource.

use crate::grid::ZoneType;
use crate::happiness_breakdown::HappinessBreakdown;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

#[test]
fn test_happiness_breakdown_empty_city_has_no_factors() {
    let city = TestCity::new();
    let breakdown = city.resource::<HappinessBreakdown>();
    assert!(
        breakdown.factors.is_empty(),
        "Expected empty breakdown for a city with no citizens"
    );
}

#[test]
fn test_happiness_breakdown_populated_after_ticks() {
    let mut city = TestCity::new()
        .with_utility(5, 10, UtilityType::PowerPlant)
        .with_utility(5, 12, UtilityType::WaterTower)
        .with_building(10, 10, ZoneType::ResidentialLow, 1)
        .with_building(12, 10, ZoneType::CommercialLow, 1)
        .with_citizen((10, 10), (12, 10));

    // Run enough ticks for the happiness breakdown to compute.
    // HAPPINESS_UPDATE_INTERVAL is 20, so we need at least that many ticks.
    // Use a few extra slow cycles to let utility coverage propagate.
    city.tick_slow_cycles(2);

    let breakdown = city.resource::<HappinessBreakdown>();
    assert!(
        !breakdown.factors.is_empty(),
        "Expected non-empty breakdown after spawning citizens and ticking"
    );

    // Employment should be positive since the citizen has a work location
    let employment = breakdown
        .factors
        .iter()
        .find(|(name, _)| name == "employment")
        .map(|(_, v)| *v);
    assert!(
        employment.is_some() && employment.unwrap() > 0.0,
        "Expected positive employment factor, got {:?}",
        employment
    );
}

