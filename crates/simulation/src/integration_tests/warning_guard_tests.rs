//! Integration tests for PowerShortage/WaterShortage warning guards (#1985).
//!
//! Verifies that utility shortage warnings do NOT fire when there are zero
//! buildings in the city, since `compute_utility_coverage()` returns (0.0, 0.0)
//! in that case and the low coverage is not meaningful.

use crate::city_observation::CityWarning;
use crate::observation_builder::CurrentObservation;
use crate::test_harness::TestCity;

#[test]
fn test_no_power_shortage_warning_with_zero_buildings() {
    let mut city = TestCity::new();
    // An empty city has 0 buildings and 0 coverage — should NOT warn.
    city.tick(5);
    let warnings = &city.resource::<CurrentObservation>().observation.warnings;
    assert!(
        !warnings.contains(&CityWarning::PowerShortage),
        "PowerShortage should not fire with 0 buildings, got: {:?}",
        warnings,
    );
}

#[test]
fn test_no_water_shortage_warning_with_zero_buildings() {
    let mut city = TestCity::new();
    city.tick(5);
    let warnings = &city.resource::<CurrentObservation>().observation.warnings;
    assert!(
        !warnings.contains(&CityWarning::WaterShortage),
        "WaterShortage should not fire with 0 buildings, got: {:?}",
        warnings,
    );
}

#[test]
fn test_no_utility_warnings_empty_city() {
    let mut city = TestCity::new();
    city.tick(10);
    let obs = &city.resource::<CurrentObservation>().observation;
    // With zero buildings, neither power nor water shortage should appear.
    assert_eq!(
        obs.building_count, 0,
        "empty city should have 0 buildings"
    );
    let has_power_warning = obs.warnings.contains(&CityWarning::PowerShortage);
    let has_water_warning = obs.warnings.contains(&CityWarning::WaterShortage);
    assert!(
        !has_power_warning && !has_water_warning,
        "no utility shortage warnings expected with 0 buildings, got: {:?}",
        obs.warnings,
    );
}
