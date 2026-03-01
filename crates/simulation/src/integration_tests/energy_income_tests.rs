//! Integration tests for energy billing revenue visibility in income
//! observations (issue #1982).
//!
//! Verifies that energy net income from the billing cycle is included in
//! both `IncomeProjection.projected_income` and `CityBudget.monthly_income`.

use crate::coal_power::PowerPlant;
use crate::energy_demand::{EnergyConsumer, LoadPriority};
use crate::energy_pricing::EnergyEconomics;
use crate::income_projection::IncomeProjection;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

/// Helper: create a PowerPlant component with given capacity.
fn make_plant(capacity_mw: f32, fuel_cost: f32) -> PowerPlant {
    PowerPlant {
        plant_type: crate::coal_power::PowerPlantType::Coal,
        capacity_mw,
        current_output_mw: 0.0,
        fuel_cost,
        grid_x: 0,
        grid_y: 0,
    }
}

/// Spawn a standalone EnergyConsumer producing `target_mw` of demand.
fn spawn_demand(city: &mut TestCity, target_mw: f32) {
    let base_kwh = target_mw * 720_000.0;
    city.world_mut()
        .spawn(EnergyConsumer::new(base_kwh, LoadPriority::Normal));
}

#[test]
fn test_energy_revenue_reflected_in_income_projection() {
    let mut city = TestCity::new().with_weather(18.3);

    // Set clock to 10:00 (mid-peak).
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Spawn a power plant with surplus capacity and some demand.
    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);

    // Tick enough for pricing to accumulate revenue.
    city.tick(8);

    // Verify energy economics has accumulated non-zero net income.
    let net_before = city.resource::<EnergyEconomics>().net_income;
    assert!(
        net_before != 0.0,
        "Energy net_income should be non-zero after pricing ticks, got {}",
        net_before,
    );

    // Now trigger a billing cycle by advancing the game clock past 30 days.
    city.world_mut().resource_mut::<GameClock>().day = 32;
    city.tick(8);

    // After billing, last_cycle_net_income should be set.
    let last_cycle = city.resource::<EnergyEconomics>().last_cycle_net_income;
    assert!(
        last_cycle != 0.0,
        "last_cycle_net_income should be set after billing cycle, got {}",
        last_cycle,
    );

    // Now run a slow tick cycle so the income projection updates.
    city.tick_slow_cycle();

    // The projected income should include energy revenue.
    let projected = city.resource::<IncomeProjection>().projected_income;
    assert!(
        projected >= last_cycle,
        "projected_income ({}) should include energy revenue ({})",
        projected,
        last_cycle,
    );
}

#[test]
fn test_energy_revenue_included_in_monthly_income() {
    let mut city = TestCity::new().with_weather(18.3);

    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Spawn power plant and demand.
    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);

    // Tick to accumulate energy revenue.
    city.tick(8);

    // Trigger billing cycle.
    city.world_mut().resource_mut::<GameClock>().day = 32;
    city.tick(8);

    let energy_income = city.resource::<EnergyEconomics>().last_cycle_net_income;
    assert!(
        energy_income != 0.0,
        "Energy income should be non-zero after billing"
    );

    // Trigger tax collection by advancing the day past the collection
    // interval (default 30 days from day 1).
    city.world_mut().resource_mut::<GameClock>().day = 62;
    city.tick(8);

    let monthly_income = city.budget().monthly_income;
    assert!(
        monthly_income >= energy_income,
        "monthly_income ({}) should include energy revenue ({})",
        monthly_income,
        energy_income,
    );
}

#[test]
fn test_energy_income_projection_uses_current_accumulator_before_first_cycle() {
    let mut city = TestCity::new().with_weather(18.3);

    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Spawn power plant and demand.
    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);

    // Tick to accumulate some energy revenue (no billing cycle yet).
    city.tick(8);

    let net_income = city.resource::<EnergyEconomics>().net_income;
    assert!(
        net_income > 0.0,
        "net_income should be positive before first billing cycle"
    );
    let last_cycle = city.resource::<EnergyEconomics>().last_cycle_net_income;
    assert!(
        last_cycle == 0.0,
        "last_cycle_net_income should be zero before first billing cycle"
    );

    // Run a slow tick so income projection updates.
    city.tick_slow_cycle();

    let projected = city.resource::<IncomeProjection>().projected_income;
    // The projection should use the current accumulator as a fallback.
    assert!(
        projected > 0.0,
        "projected_income should be positive even before first billing cycle, got {}",
        projected,
    );
}
