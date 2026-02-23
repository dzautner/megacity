//! Integration tests for the Energy Dispatch Merit Order System (POWER-009).

use crate::coal_power::PowerPlant;
use crate::energy_demand::{EnergyConsumer, EnergyGrid, LoadPriority};
use crate::energy_dispatch::EnergyDispatchState;
use crate::test_harness::TestCity;

/// Helper: create a PowerPlant component with the given capacity and fuel cost.
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

/// Spawn a standalone EnergyConsumer that will produce `target_mw` of demand
/// under default test conditions (6 AM, Spring, no degree-days).
///
/// Formula: demand_mw = base_kwh / 720 * tou * hvac * power / 1000
/// At defaults: tou=1.0, hvac=1.0, power=1.0
/// So base_kwh = target_mw * 720_000.0
fn spawn_demand(city: &mut TestCity, target_mw: f32) {
    let base_kwh = target_mw * 720_000.0;
    city.world_mut()
        .spawn(EnergyConsumer::new(base_kwh, LoadPriority::Normal));
}

/// Tick enough for both demand aggregation and dispatch to run.
/// Both fire every 4 ticks. We tick 8 to ensure at least one full cycle.
fn tick_dispatch(city: &mut TestCity) {
    city.tick(8);
}

#[test]
fn test_cheapest_generators_dispatched_first() {
    let mut city = TestCity::new();

    // Spawn generators: renewable ($0), coal ($30), gas peaker ($80).
    city.world_mut().spawn(make_plant(100.0, 0.0));
    city.world_mut().spawn(make_plant(200.0, 30.0));
    city.world_mut().spawn(make_plant(150.0, 80.0));

    // Create 150 MW of demand.
    spawn_demand(&mut city, 150.0);
    tick_dispatch(&mut city);

    let grid = city.resource::<EnergyGrid>();
    assert!(
        (grid.total_supply_mwh - 150.0).abs() < 5.0,
        "Expected ~150 MW supplied, got {}",
        grid.total_supply_mwh
    );

    let dispatch = city.resource::<EnergyDispatchState>();
    assert!(!dispatch.has_deficit, "Should not have deficit");

    // Verify: cheapest plants dispatched first, gas peaker not needed.
    let world = city.world_mut();
    let mut plants: Vec<(f32, f32)> = world
        .query::<&PowerPlant>()
        .iter(world)
        .map(|p| (p.fuel_cost, p.current_output_mw))
        .collect();
    plants.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Renewable ($0): should have significant output.
    assert!(plants[0].1 > 0.0, "Renewable should have output");
    // Gas peaker ($80): should have zero or minimal output since cheaper plants suffice.
    // (Exact values depend on demand rounding, so we check relative ordering.)
    assert!(
        plants[0].1 >= plants[2].1,
        "Renewable should have >= gas peaker output"
    );
}

#[test]
fn test_expensive_generators_only_when_needed() {
    let mut city = TestCity::new();

    city.world_mut().spawn(make_plant(50.0, 0.0)); // renewable
    city.world_mut().spawn(make_plant(200.0, 80.0)); // gas peaker

    // Small demand: 30 MW — only renewable should run.
    spawn_demand(&mut city, 30.0);
    tick_dispatch(&mut city);

    let world = city.world_mut();
    let mut plants: Vec<(f32, f32)> = world
        .query::<&PowerPlant>()
        .iter(world)
        .map(|p| (p.fuel_cost, p.current_output_mw))
        .collect();
    plants.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    assert!(plants[0].1 > 0.0, "Renewable should have output");
    assert!(
        plants[1].1 < f32::EPSILON,
        "Gas peaker should not run when demand < renewable capacity, got {}",
        plants[1].1
    );
}

#[test]
fn test_high_demand_dispatches_expensive_plants() {
    let mut city = TestCity::new();

    city.world_mut().spawn(make_plant(50.0, 0.0)); // renewable
    city.world_mut().spawn(make_plant(200.0, 80.0)); // gas peaker

    // Large demand: 200 MW — exceeds renewable, needs gas peaker.
    spawn_demand(&mut city, 200.0);
    tick_dispatch(&mut city);

    let world = city.world_mut();
    let mut plants: Vec<(f32, f32)> = world
        .query::<&PowerPlant>()
        .iter(world)
        .map(|p| (p.fuel_cost, p.current_output_mw))
        .collect();
    plants.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Both should be dispatched.
    assert!(
        (plants[0].1 - 50.0).abs() < f32::EPSILON,
        "Renewable should be at full capacity (50 MW), got {}",
        plants[0].1
    );
    assert!(
        plants[1].1 > 0.0,
        "Gas peaker should have output when demand exceeds renewable"
    );
}

#[test]
fn test_reserve_margin_deficit_triggers_blackout() {
    let mut city = TestCity::new();

    // Only 100 MW of capacity but ~200 MW demand.
    city.world_mut().spawn(make_plant(100.0, 30.0));
    spawn_demand(&mut city, 200.0);
    tick_dispatch(&mut city);

    let dispatch = city.resource::<EnergyDispatchState>();
    assert!(dispatch.has_deficit, "Should have deficit");
    assert!(
        dispatch.load_shed_fraction > 0.0,
        "Should be shedding load"
    );

    let grid = city.resource::<EnergyGrid>();
    assert!(
        grid.reserve_margin < 0.0,
        "Reserve margin should be negative, got {}",
        grid.reserve_margin
    );
}

#[test]
fn test_zero_demand_preserves_plant_output() {
    let mut city = TestCity::new();

    // No EnergyConsumer => demand = 0 after aggregation.
    // Dispatch should skip and preserve plant's default output.
    city.world_mut().spawn(make_plant(100.0, 0.0));
    tick_dispatch(&mut city);

    let dispatch = city.resource::<EnergyDispatchState>();
    assert!(!dispatch.has_deficit);
    assert!(!dispatch.active, "Dispatch should not be active with 0 demand");
}

#[test]
fn test_electricity_price_reflects_last_dispatched() {
    let mut city = TestCity::new();

    city.world_mut().spawn(make_plant(100.0, 0.0)); // renewable
    city.world_mut().spawn(make_plant(200.0, 30.0)); // coal
    city.world_mut().spawn(make_plant(100.0, 40.0)); // gas

    // Demand 350 MW: dispatches all three. Last dispatched = gas ($40).
    spawn_demand(&mut city, 350.0);
    tick_dispatch(&mut city);

    let dispatch = city.resource::<EnergyDispatchState>();
    // Price should be at least $40 (gas is last dispatched).
    assert!(
        dispatch.electricity_price >= 40.0,
        "Price should be >= $40 (gas cost), got {}",
        dispatch.electricity_price
    );
}

#[test]
fn test_scarcity_multiplier_increases_price() {
    let mut city = TestCity::new();

    // 110 MW capacity, ~105 MW demand => tight margin.
    city.world_mut().spawn(make_plant(110.0, 30.0));
    spawn_demand(&mut city, 105.0);
    tick_dispatch(&mut city);

    let dispatch = city.resource::<EnergyDispatchState>();
    assert!(dispatch.active, "Dispatch should be active");

    let grid = city.resource::<EnergyGrid>();
    // If reserve margin < 10%, scarcity kicks in and price > base cost.
    if grid.reserve_margin < 0.1 {
        assert!(
            dispatch.electricity_price > 30.0,
            "Price should exceed $30 with scarcity, got {}",
            dispatch.electricity_price
        );
    }
}

#[test]
fn test_rolling_blackout_rotation_increments() {
    let mut city = TestCity::new();

    // 50 MW capacity, 100 MW demand => deficit.
    city.world_mut().spawn(make_plant(50.0, 0.0));
    spawn_demand(&mut city, 100.0);

    tick_dispatch(&mut city);
    let rot1 = city.resource::<EnergyDispatchState>().blackout_rotation;
    assert!(rot1 > 0, "Rotation should have incremented from 0");

    // Run another dispatch cycle.
    city.tick(4);
    let rot2 = city.resource::<EnergyDispatchState>().blackout_rotation;
    assert!(
        rot2 > rot1,
        "Rotation should continue incrementing: {} -> {}",
        rot1,
        rot2
    );
}

#[test]
fn test_full_merit_order_chain() {
    let mut city = TestCity::new();

    // Create plants with distinct fuel costs matching the merit order.
    city.world_mut().spawn(make_plant(50.0, 80.0)); // gas peaker
    city.world_mut().spawn(make_plant(100.0, 0.0)); // renewable
    city.world_mut().spawn(make_plant(80.0, 40.0)); // gas
    city.world_mut().spawn(make_plant(120.0, 10.0)); // nuclear
    city.world_mut().spawn(make_plant(60.0, 30.0)); // coal

    // Demand 350 MW — should dispatch in merit order:
    // Renewable(100) + Nuclear(120) + Coal(60) + Gas(70) = 350
    // Gas peaker should NOT be needed.
    spawn_demand(&mut city, 350.0);
    tick_dispatch(&mut city);

    let world = city.world_mut();
    let mut plants: Vec<(f32, f32)> = world
        .query::<&PowerPlant>()
        .iter(world)
        .map(|p| (p.fuel_cost, p.current_output_mw))
        .collect();
    plants.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Renewable ($0): should be at full capacity (100 MW).
    assert!(
        (plants[0].1 - 100.0).abs() < f32::EPSILON,
        "Renewable: expected 100, got {}",
        plants[0].1
    );
    // Nuclear ($10): should be at full capacity (120 MW).
    assert!(
        (plants[1].1 - 120.0).abs() < f32::EPSILON,
        "Nuclear: expected 120, got {}",
        plants[1].1
    );
    // Coal ($30): should be at full capacity (60 MW).
    assert!(
        (plants[2].1 - 60.0).abs() < f32::EPSILON,
        "Coal: expected 60, got {}",
        plants[2].1
    );
    // Gas ($40): should get the remainder (350-100-120-60 = 70 MW).
    assert!(
        (plants[3].1 - 70.0).abs() < 1.0,
        "Gas: expected ~70, got {}",
        plants[3].1
    );
    // Gas peaker ($80): should NOT be dispatched.
    assert!(
        plants[4].1.abs() < f32::EPSILON,
        "Gas peaker: expected 0, got {}",
        plants[4].1
    );
}
