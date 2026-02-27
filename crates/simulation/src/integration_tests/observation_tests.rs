//! Integration tests for the CityObservation snapshot system (#1880).

use crate::city_observation::CityWarning;
use crate::economy::CityBudget;
use crate::observation_builder::CurrentObservation;
use crate::test_harness::TestCity;

#[test]
fn test_observation_populated_after_ticking() {
    let mut city = TestCity::new();
    city.tick(10);
    let obs = &city.resource::<CurrentObservation>().observation;
    // Tick counter should have advanced (first update + 10 ticks)
    assert!(obs.tick > 0, "tick counter should be > 0 after ticking");
    assert!(obs.day >= 1, "day should be >= 1");
}

#[test]
fn test_observation_treasury_matches_budget() {
    let mut city = TestCity::new();
    city.tick(1);
    let budget_treasury = city.resource::<CityBudget>().treasury;
    let obs_treasury = city.resource::<CurrentObservation>().observation.treasury;
    assert!(
        (budget_treasury - obs_treasury).abs() < f64::EPSILON,
        "observation treasury ({}) should match CityBudget treasury ({})",
        obs_treasury,
        budget_treasury,
    );
}

#[test]
fn test_observation_population_non_negative() {
    let mut city = TestCity::new();
    city.tick(5);
    let pop = &city.resource::<CurrentObservation>().observation.population;
    // All population fields should be non-negative (u32 guarantees this, but
    // we verify the snapshot is coherent: total >= employed, etc.)
    assert!(
        pop.total >= pop.employed,
        "total ({}) should be >= employed ({})",
        pop.total,
        pop.employed,
    );
}

#[test]
fn test_observation_happiness_valid_range() {
    let mut city = TestCity::new();
    city.tick(5);
    let happiness = city
        .resource::<CurrentObservation>()
        .observation
        .happiness
        .overall;
    // Happiness should be in [0, 100] range
    assert!(
        (0.0..=100.0).contains(&happiness),
        "happiness ({}) should be in [0.0, 100.0]",
        happiness,
    );
}

#[test]
fn test_observation_coverage_valid_range() {
    let mut city = TestCity::new();
    city.tick(5);
    let obs = &city.resource::<CurrentObservation>().observation;
    // Coverage values should be in [0.0, 1.0]
    for (name, val) in [
        ("power", obs.power_coverage),
        ("water", obs.water_coverage),
        ("fire", obs.services.fire),
        ("police", obs.services.police),
        ("health", obs.services.health),
        ("education", obs.services.education),
    ] {
        assert!(
            (0.0..=1.0).contains(&val),
            "{} coverage ({}) should be in [0.0, 1.0]",
            name,
            val,
        );
    }
}

#[test]
fn test_observation_negative_budget_warning() {
    let mut city = TestCity::new();
    // Set treasury to deeply negative so the warning fires
    city.world_mut()
        .resource_mut::<CityBudget>()
        .treasury = -10_000.0;
    city.world_mut()
        .resource_mut::<CityBudget>()
        .monthly_income = 100.0;
    city.world_mut()
        .resource_mut::<CityBudget>()
        .monthly_expenses = 500.0;
    city.tick(1);
    let warnings = &city.resource::<CurrentObservation>().observation.warnings;
    assert!(
        warnings.contains(&CityWarning::NegativeBudget),
        "expected NegativeBudget warning when treasury is negative and expenses > income, got: {:?}",
        warnings,
    );
}

#[test]
fn test_observation_zone_demand_populated() {
    let mut city = TestCity::new();
    city.tick(5);
    let zd = &city.resource::<CurrentObservation>().observation.zone_demand;
    // Demand values should be finite (not NaN or infinity)
    assert!(zd.residential.is_finite(), "residential demand should be finite");
    assert!(zd.commercial.is_finite(), "commercial demand should be finite");
    assert!(zd.industrial.is_finite(), "industrial demand should be finite");
    assert!(zd.office.is_finite(), "office demand should be finite");
}

#[test]
fn test_observation_serializes_to_json() {
    let mut city = TestCity::new();
    city.tick(5);
    let obs = &city.resource::<CurrentObservation>().observation;
    let json = serde_json::to_string(obs);
    assert!(json.is_ok(), "observation should serialize to JSON");
    let json_str = json.unwrap();
    assert!(json_str.contains("\"tick\""), "JSON should contain tick field");
    assert!(json_str.contains("\"treasury\""), "JSON should contain treasury field");
}
