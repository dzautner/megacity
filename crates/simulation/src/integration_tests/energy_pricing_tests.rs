//! Integration tests for POWER-010: Time-of-Use Electricity Pricing and Revenue.

use crate::coal_power::PowerPlant;
use crate::energy_demand::{EnergyConsumer, LoadPriority};
use crate::energy_pricing::{
    EnergyEconomics, EnergyPricingConfig, TimeOfUsePeriod,
};
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

/// Helper: create a PowerPlant component.
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

/// Create a TestCity with baseline weather (18.3C) for predictable demand.
fn new_baseline_city() -> TestCity {
    TestCity::new().with_weather(18.3)
}

/// Spawn a standalone EnergyConsumer that produces `target_mw` of demand
/// under baseline conditions (6 AM Spring, T=18.3C).
fn spawn_demand(city: &mut TestCity, target_mw: f32) {
    let base_kwh = target_mw * 720_000.0;
    city.world_mut()
        .spawn(EnergyConsumer::new(base_kwh, LoadPriority::Normal));
}

/// Tick enough for demand aggregation, dispatch, and pricing to all run.
fn tick_pricing(city: &mut TestCity) {
    city.tick(8);
}

#[test]
fn test_energy_pricing_resources_initialized() {
    let city = new_baseline_city();
    let config = city.resource::<EnergyPricingConfig>();
    assert!(
        (config.base_rate_per_kwh - 0.12).abs() < f32::EPSILON,
        "Base rate should be $0.12/kWh"
    );

    let econ = city.resource::<EnergyEconomics>();
    assert!(
        (econ.current_price_per_kwh - 0.12).abs() < f32::EPSILON,
        "Default price should be $0.12/kWh"
    );
}

#[test]
fn test_energy_pricing_mid_peak_no_scarcity() {
    let mut city = new_baseline_city();

    // Set clock to 10:00 (mid-peak)
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Create generators with surplus capacity.
    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);
    tick_pricing(&mut city);

    let econ = city.resource::<EnergyEconomics>();
    // Mid-peak multiplier = 1.0, reserve margin > 20% => scarcity = 1.0
    // Price should be $0.12 * 1.0 * 1.0 = $0.12
    assert!(
        (econ.current_price_per_kwh - 0.12).abs() < 0.01,
        "Mid-peak no-scarcity price should be ~$0.12, got {}",
        econ.current_price_per_kwh
    );
    assert_eq!(econ.current_period, TimeOfUsePeriod::MidPeak);
    assert!((econ.tou_multiplier - 1.0).abs() < f32::EPSILON);
    assert!((econ.scarcity_multiplier - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_energy_pricing_on_peak_period() {
    let mut city = new_baseline_city();

    // Set clock to 16:00 (on-peak)
    city.world_mut().resource_mut::<GameClock>().hour = 16.0;

    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);
    tick_pricing(&mut city);

    let econ = city.resource::<EnergyEconomics>();
    // On-peak multiplier = 1.5, no scarcity => $0.12 * 1.5 = $0.18
    assert!(
        (econ.current_price_per_kwh - 0.18).abs() < 0.01,
        "On-peak price should be ~$0.18, got {}",
        econ.current_price_per_kwh
    );
    assert_eq!(econ.current_period, TimeOfUsePeriod::OnPeak);
    assert!((econ.tou_multiplier - 1.5).abs() < f32::EPSILON);
}

#[test]
fn test_energy_pricing_off_peak_period() {
    let mut city = new_baseline_city();

    // Set clock to 2:00 (off-peak)
    city.world_mut().resource_mut::<GameClock>().hour = 2.0;

    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);
    tick_pricing(&mut city);

    let econ = city.resource::<EnergyEconomics>();
    // Off-peak multiplier = 0.6, no scarcity => $0.12 * 0.6 = $0.072
    assert!(
        (econ.current_price_per_kwh - 0.072).abs() < 0.01,
        "Off-peak price should be ~$0.072, got {}",
        econ.current_price_per_kwh
    );
    assert_eq!(econ.current_period, TimeOfUsePeriod::OffPeak);
}

#[test]
fn test_scarcity_increases_price() {
    let mut city = new_baseline_city();

    // Set clock to 10:00 (mid-peak) so TOU = 1.0
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Tight supply: 110 MW capacity, ~105 MW demand => ~5% margin
    city.world_mut().spawn(make_plant(110.0, 25.0));
    spawn_demand(&mut city, 105.0);
    tick_pricing(&mut city);

    let econ = city.resource::<EnergyEconomics>();
    // With tight margin, scarcity multiplier should be > 1.0
    assert!(
        econ.scarcity_multiplier > 1.0,
        "Scarcity multiplier should be > 1.0 with tight margin, got {}",
        econ.scarcity_multiplier
    );
    assert!(
        econ.current_price_per_kwh > 0.12,
        "Price should exceed base rate with scarcity, got {}",
        econ.current_price_per_kwh
    );
}

#[test]
fn test_deficit_scarcity_multiplier_is_three() {
    let mut city = new_baseline_city();

    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Deficit: 50 MW capacity, 100 MW demand
    city.world_mut().spawn(make_plant(50.0, 25.0));
    spawn_demand(&mut city, 100.0);
    tick_pricing(&mut city);

    let econ = city.resource::<EnergyEconomics>();
    // Deficit => scarcity_multiplier = 3.0
    assert!(
        (econ.scarcity_multiplier - 3.0).abs() < f32::EPSILON,
        "Deficit scarcity multiplier should be 3.0, got {}",
        econ.scarcity_multiplier
    );
    // Price = $0.12 * 1.0 * 3.0 = $0.36
    assert!(
        (econ.current_price_per_kwh - 0.36).abs() < 0.02,
        "Deficit mid-peak price should be ~$0.36, got {}",
        econ.current_price_per_kwh
    );
}

#[test]
fn test_revenue_accumulates_with_demand() {
    let mut city = new_baseline_city();

    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);

    // Tick multiple times to accumulate revenue.
    tick_pricing(&mut city);
    tick_pricing(&mut city);

    let econ = city.resource::<EnergyEconomics>();
    assert!(
        econ.total_revenue > 0.0,
        "Revenue should be positive with demand, got {}",
        econ.total_revenue
    );
    assert!(
        econ.total_consumption_mwh > 0.0,
        "Consumption should be tracked, got {}",
        econ.total_consumption_mwh
    );
}

#[test]
fn test_net_income_equals_revenue_minus_costs() {
    let mut city = new_baseline_city();

    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);
    tick_pricing(&mut city);

    let econ = city.resource::<EnergyEconomics>();
    let expected_net = econ.total_revenue - econ.total_costs;
    assert!(
        (econ.net_income - expected_net).abs() < 0.01,
        "Net income ({}) should equal revenue ({}) - costs ({})",
        econ.net_income,
        econ.total_revenue,
        econ.total_costs
    );
}

#[test]
fn test_zero_demand_no_revenue() {
    let mut city = new_baseline_city();

    // No consumers, no generators.
    tick_pricing(&mut city);

    let econ = city.resource::<EnergyEconomics>();
    assert!(
        econ.total_revenue.abs() < f64::EPSILON,
        "No demand => no revenue, got {}",
        econ.total_revenue
    );
}

#[test]
fn test_citizen_cost_burden_increases_with_scarcity() {
    let mut city = new_baseline_city();

    city.world_mut().resource_mut::<GameClock>().hour = 16.0; // on-peak

    // Deficit scenario
    city.world_mut().spawn(make_plant(50.0, 25.0));
    spawn_demand(&mut city, 100.0);
    tick_pricing(&mut city);

    let econ = city.resource::<EnergyEconomics>();
    // On-peak (1.5) * deficit (3.0) = 4.5 burden
    assert!(
        econ.citizen_cost_burden > 1.0,
        "Citizen cost burden should exceed 1.0 with scarcity and on-peak, got {}",
        econ.citizen_cost_burden
    );
}

#[test]
fn test_custom_config_affects_pricing() {
    let mut city = new_baseline_city();

    // Override config with custom rates.
    city.world_mut().insert_resource(EnergyPricingConfig {
        base_rate_per_kwh: 0.20,
        off_peak_multiplier: 0.5,
        mid_peak_multiplier: 1.0,
        on_peak_multiplier: 2.0,
        generation_cost_per_mwh: 30.0,
    });

    city.world_mut().resource_mut::<GameClock>().hour = 16.0; // on-peak

    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);
    tick_pricing(&mut city);

    let econ = city.resource::<EnergyEconomics>();
    // $0.20 * 2.0 * 1.0 = $0.40
    assert!(
        (econ.current_price_per_kwh - 0.40).abs() < 0.02,
        "Custom on-peak price should be ~$0.40, got {}",
        econ.current_price_per_kwh
    );
}
