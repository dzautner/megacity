//! Integration tests verifying that zone tax rate changes affect actual
//! revenue collection. This validates the fix for issue #1770 where the
//! UI tax slider was disconnected from the economy system.

use crate::budget::ExtendedBudget;
use crate::grid::ZoneType;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

/// Helper: set the game clock day to force a tax collection cycle.
fn force_clock_to_day(city: &mut TestCity, day: u32) {
    let world = city.world_mut();
    world.resource_mut::<GameClock>().day = day;
}

/// Changing the residential zone tax rate changes residential tax income.
#[test]
fn test_residential_tax_rate_change_affects_income() {
    // Collect at low rate
    let mut city_low = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 2)
        .with_budget(10_000.0);
    {
        let world = city_low.world_mut();
        world
            .resource_mut::<ExtendedBudget>()
            .zone_taxes
            .residential = 0.05;
    }
    force_clock_to_day(&mut city_low, 32);
    city_low.tick(10);
    let low_tax = city_low
        .resource::<ExtendedBudget>()
        .income_breakdown
        .residential_tax;

    // Collect at high rate
    let mut city_high = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 2)
        .with_budget(10_000.0);
    {
        let world = city_high.world_mut();
        world
            .resource_mut::<ExtendedBudget>()
            .zone_taxes
            .residential = 0.20;
    }
    force_clock_to_day(&mut city_high, 32);
    city_high.tick(10);
    let high_tax = city_high
        .resource::<ExtendedBudget>()
        .income_breakdown
        .residential_tax;

    // Higher rate should produce more tax
    assert!(
        high_tax > low_tax,
        "Higher residential tax rate should produce more revenue: \
         low({low_tax}) should be < high({high_tax})"
    );

    // With 4x rate difference, tax should be proportional
    let ratio = high_tax / low_tax;
    assert!(
        (ratio - 4.0).abs() < 0.1,
        "Tax should scale proportionally with rate: ratio={ratio}, expected ~4.0"
    );
}

/// Changing the commercial zone tax rate changes commercial tax income.
#[test]
fn test_commercial_tax_rate_change_affects_income() {
    let mut city_low = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialLow, 2)
        .with_budget(10_000.0);
    {
        let world = city_low.world_mut();
        world
            .resource_mut::<ExtendedBudget>()
            .zone_taxes
            .commercial = 0.05;
    }
    force_clock_to_day(&mut city_low, 32);
    city_low.tick(10);
    let low_tax = city_low
        .resource::<ExtendedBudget>()
        .income_breakdown
        .commercial_tax;

    let mut city_high = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialLow, 2)
        .with_budget(10_000.0);
    {
        let world = city_high.world_mut();
        world
            .resource_mut::<ExtendedBudget>()
            .zone_taxes
            .commercial = 0.15;
    }
    force_clock_to_day(&mut city_high, 32);
    city_high.tick(10);
    let high_tax = city_high
        .resource::<ExtendedBudget>()
        .income_breakdown
        .commercial_tax;

    assert!(
        high_tax > low_tax,
        "Higher commercial tax rate should produce more revenue: \
         low({low_tax}) should be < high({high_tax})"
    );
}

/// Verify zero tax rate produces zero revenue for a zone.
#[test]
fn test_zero_tax_rate_produces_zero_revenue() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 3)
        .with_budget(10_000.0);
    {
        let world = city.world_mut();
        world
            .resource_mut::<ExtendedBudget>()
            .zone_taxes
            .residential = 0.0;
    }
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let tax = city
        .resource::<ExtendedBudget>()
        .income_breakdown
        .residential_tax;
    assert!(
        tax.abs() < 0.001,
        "Zero tax rate should produce zero revenue, got {tax}"
    );
}

/// Different zone types use their own rates independently.
#[test]
fn test_each_zone_uses_its_own_rate() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 2)
        .with_building(60, 60, ZoneType::CommercialLow, 2)
        .with_building(70, 70, ZoneType::Industrial, 2)
        .with_building(80, 80, ZoneType::Office, 2)
        .with_budget(100_000.0);

    {
        let world = city.world_mut();
        let mut ext = world.resource_mut::<ExtendedBudget>();
        ext.zone_taxes.residential = 0.05;
        ext.zone_taxes.commercial = 0.10;
        ext.zone_taxes.industrial = 0.15;
        ext.zone_taxes.office = 0.20;
    }
    force_clock_to_day(&mut city, 32);
    city.tick(10);

    let ext = city.resource::<ExtendedBudget>();
    let res_tax = ext.income_breakdown.residential_tax;
    let com_tax = ext.income_breakdown.commercial_tax;
    let ind_tax = ext.income_breakdown.industrial_tax;
    let off_tax = ext.income_breakdown.office_tax;

    // All should be positive
    assert!(res_tax > 0.0, "Residential tax should be positive");
    assert!(com_tax > 0.0, "Commercial tax should be positive");
    assert!(ind_tax > 0.0, "Industrial tax should be positive");
    assert!(off_tax > 0.0, "Office tax should be positive");

    // Rates are 1:2:3:4, so taxes should scale similarly
    // (assuming similar land values for all locations)
    assert!(
        com_tax > res_tax,
        "Commercial (10%) should produce more tax than residential (5%)"
    );
    assert!(
        ind_tax > com_tax,
        "Industrial (15%) should produce more tax than commercial (10%)"
    );
    assert!(
        off_tax > ind_tax,
        "Office (20%) should produce more tax than industrial (15%)"
    );
}
