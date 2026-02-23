//! Integration tests for the Energy Dispatch Merit Order System (POWER-009).

use crate::coal_power::PowerPlant;
use crate::energy_demand::EnergyGrid;
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

#[test]
fn test_cheapest_generators_dispatched_first() {
    let mut city = TestCity::new();

    // Spawn generators with different fuel costs (merit order).
    // Renewable-like ($0), coal ($30), gas peaker ($80).
    city.world_mut().spawn(make_plant(100.0, 0.0));
    city.world_mut().spawn(make_plant(200.0, 30.0));
    city.world_mut().spawn(make_plant(150.0, 80.0));

    // Set demand to 150 MW — should dispatch renewable (100) + coal (50).
    city.world_mut()
        .resource_mut::<EnergyGrid>()
        .total_demand_mwh = 150.0;

    city.tick(4);

    let grid = city.resource::<EnergyGrid>();
    assert!(
        (grid.total_supply_mwh - 150.0).abs() < 1.0,
        "Expected ~150 MW supplied, got {}",
        grid.total_supply_mwh
    );

    let dispatch = city.resource::<EnergyDispatchState>();
    assert!(!dispatch.has_deficit, "Should not have deficit");

    // Verify individual plant outputs by fuel cost.
    let world = city.world_mut();
    let mut plants: Vec<(f32, f32)> = world
        .query::<&PowerPlant>()
        .iter(world)
        .map(|p| (p.fuel_cost, p.current_output_mw))
        .collect();
    plants.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Renewable ($0): fully dispatched (100 MW).
    assert!(
        (plants[0].1 - 100.0).abs() < f32::EPSILON,
        "Renewable should output 100 MW, got {}",
        plants[0].1
    );
    // Coal ($30): partial (50 MW).
    assert!(
        (plants[1].1 - 50.0).abs() < f32::EPSILON,
        "Coal should output 50 MW, got {}",
        plants[1].1
    );
    // Gas peaker ($80): not dispatched.
    assert!(
        plants[2].1.abs() < f32::EPSILON,
        "Gas peaker should output 0 MW, got {}",
        plants[2].1
    );
}

#[test]
fn test_expensive_generators_only_when_needed() {
    let mut city = TestCity::new();

    city.world_mut().spawn(make_plant(50.0, 0.0));
    city.world_mut().spawn(make_plant(200.0, 80.0));

    // Demand 30 MW — only renewable should run.
    city.world_mut()
        .resource_mut::<EnergyGrid>()
        .total_demand_mwh = 30.0;

    city.tick(4);

    let world = city.world_mut();
    let mut plants: Vec<(f32, f32)> = world
        .query::<&PowerPlant>()
        .iter(world)
        .map(|p| (p.fuel_cost, p.current_output_mw))
        .collect();
    plants.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    assert!(
        (plants[0].1 - 30.0).abs() < f32::EPSILON,
        "Renewable should output 30 MW, got {}",
        plants[0].1
    );
    assert!(
        plants[1].1.abs() < f32::EPSILON,
        "Gas peaker should not run, got {}",
        plants[1].1
    );

    // Now increase demand beyond renewable capacity.
    city.world_mut()
        .resource_mut::<EnergyGrid>()
        .total_demand_mwh = 180.0;

    city.tick(4);

    let world = city.world_mut();
    let mut plants: Vec<(f32, f32)> = world
        .query::<&PowerPlant>()
        .iter(world)
        .map(|p| (p.fuel_cost, p.current_output_mw))
        .collect();
    plants.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    assert!(
        (plants[0].1 - 50.0).abs() < f32::EPSILON,
        "Renewable should output full 50 MW, got {}",
        plants[0].1
    );
    assert!(
        (plants[1].1 - 130.0).abs() < f32::EPSILON,
        "Gas peaker should output 130 MW, got {}",
        plants[1].1
    );
}

#[test]
fn test_reserve_margin_deficit_triggers_blackout() {
    let mut city = TestCity::new();

    // Only 100 MW of capacity but 200 MW demand.
    city.world_mut().spawn(make_plant(100.0, 30.0));

    city.world_mut()
        .resource_mut::<EnergyGrid>()
        .total_demand_mwh = 200.0;

    city.tick(4);

    let dispatch = city.resource::<EnergyDispatchState>();
    assert!(dispatch.has_deficit, "Should have deficit");

    let grid = city.resource::<EnergyGrid>();
    assert!(
        grid.reserve_margin < 0.0,
        "Reserve margin should be negative, got {}",
        grid.reserve_margin
    );

    assert!(
        (dispatch.load_shed_fraction - 0.5).abs() < 0.01,
        "Should shed 50% of load, got {}",
        dispatch.load_shed_fraction
    );

    assert!(
        (grid.total_supply_mwh - 100.0).abs() < f32::EPSILON,
        "Supply should be capped at 100 MW, got {}",
        grid.total_supply_mwh
    );
}

#[test]
fn test_no_generators_no_crash() {
    let mut city = TestCity::new();

    city.world_mut()
        .resource_mut::<EnergyGrid>()
        .total_demand_mwh = 100.0;

    city.tick(4);

    let dispatch = city.resource::<EnergyDispatchState>();
    assert!(dispatch.has_deficit, "Should have deficit with no generators");

    let grid = city.resource::<EnergyGrid>();
    assert_eq!(grid.total_supply_mwh, 0.0);

    assert!(
        (dispatch.load_shed_fraction - 1.0).abs() < f32::EPSILON,
        "Should shed 100% load, got {}",
        dispatch.load_shed_fraction
    );
}

#[test]
fn test_zero_demand_no_dispatch() {
    let mut city = TestCity::new();

    city.world_mut().spawn(make_plant(100.0, 0.0));

    city.world_mut()
        .resource_mut::<EnergyGrid>()
        .total_demand_mwh = 0.0;

    city.tick(4);

    let dispatch = city.resource::<EnergyDispatchState>();
    assert!(!dispatch.has_deficit);
    assert_eq!(dispatch.electricity_price, 0.0);

    let grid = city.resource::<EnergyGrid>();
    assert_eq!(grid.total_supply_mwh, 0.0);

    let world = city.world_mut();
    let output: f32 = world
        .query::<&PowerPlant>()
        .iter(world)
        .map(|p| p.current_output_mw)
        .sum();
    assert!(output.abs() < f32::EPSILON, "No output expected");
}

#[test]
fn test_electricity_price_reflects_last_dispatched() {
    let mut city = TestCity::new();

    city.world_mut().spawn(make_plant(100.0, 0.0));
    city.world_mut().spawn(make_plant(200.0, 30.0));
    city.world_mut().spawn(make_plant(100.0, 40.0));

    // Demand 350 MW: renewable(100) + coal(200) + gas(50) = 350.
    // Last dispatched = Gas at $40/MWh.
    city.world_mut()
        .resource_mut::<EnergyGrid>()
        .total_demand_mwh = 350.0;

    city.tick(4);

    let dispatch = city.resource::<EnergyDispatchState>();
    // Reserve margin = (400 - 350) / 350 = 0.143 > threshold, so multiplier = 1.0.
    assert!(
        (dispatch.electricity_price - 40.0).abs() < 1.0,
        "Price should be ~$40 (gas marginal cost), got {}",
        dispatch.electricity_price
    );
}

#[test]
fn test_scarcity_multiplier_increases_price() {
    let mut city = TestCity::new();

    // 110 MW capacity, 105 MW demand => reserve margin ~4.8% < 10% threshold.
    city.world_mut().spawn(make_plant(110.0, 30.0));

    city.world_mut()
        .resource_mut::<EnergyGrid>()
        .total_demand_mwh = 105.0;

    city.tick(4);

    let dispatch = city.resource::<EnergyDispatchState>();
    assert!(
        dispatch.electricity_price > 30.0,
        "Price should exceed base coal cost ($30) due to scarcity, got {}",
        dispatch.electricity_price
    );

    let grid = city.resource::<EnergyGrid>();
    assert!(
        grid.reserve_margin < 0.1,
        "Reserve margin should be below threshold"
    );
}

#[test]
fn test_rolling_blackout_rotation_increments() {
    let mut city = TestCity::new();

    city.world_mut().spawn(make_plant(50.0, 0.0));

    city.world_mut()
        .resource_mut::<EnergyGrid>()
        .total_demand_mwh = 100.0;

    let initial = city.resource::<EnergyDispatchState>().blackout_rotation;

    city.tick(4);
    let rot1 = city.resource::<EnergyDispatchState>().blackout_rotation;
    assert_eq!(rot1, initial.wrapping_add(1), "Rotation should increment");

    city.tick(4);
    let rot2 = city.resource::<EnergyDispatchState>().blackout_rotation;
    assert_eq!(rot2, rot1.wrapping_add(1), "Should increment again");
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

    // Demand 350 MW — should dispatch:
    // Renewable(100) + Nuclear(120) + Coal(60) + Gas(70) = 350
    city.world_mut()
        .resource_mut::<EnergyGrid>()
        .total_demand_mwh = 350.0;

    city.tick(4);

    let world = city.world_mut();
    let mut plants: Vec<(f32, f32)> = world
        .query::<&PowerPlant>()
        .iter(world)
        .map(|p| (p.fuel_cost, p.current_output_mw))
        .collect();
    plants.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Renewable ($0): 100 MW (full).
    assert!(
        (plants[0].1 - 100.0).abs() < f32::EPSILON,
        "Renewable: expected 100, got {}",
        plants[0].1
    );
    // Nuclear ($10): 120 MW (full).
    assert!(
        (plants[1].1 - 120.0).abs() < f32::EPSILON,
        "Nuclear: expected 120, got {}",
        plants[1].1
    );
    // Coal ($30): 60 MW (full).
    assert!(
        (plants[2].1 - 60.0).abs() < f32::EPSILON,
        "Coal: expected 60, got {}",
        plants[2].1
    );
    // Gas ($40): 70 MW (partial, 350 - 100 - 120 - 60 = 70).
    assert!(
        (plants[3].1 - 70.0).abs() < f32::EPSILON,
        "Gas: expected 70, got {}",
        plants[3].1
    );
    // Gas peaker ($80): 0 MW (not needed).
    assert!(
        plants[4].1.abs() < f32::EPSILON,
        "Gas peaker: expected 0, got {}",
        plants[4].1
    );

    // Price should be $40 (gas cost) since gas was the last dispatched.
    let dispatch = world.resource::<EnergyDispatchState>();
    // Reserve margin: (410 - 350) / 350 = 0.171 > 0.1, so no scarcity.
    assert!(
        (dispatch.electricity_price - 40.0).abs() < 1.0,
        "Price should be ~$40, got {}",
        dispatch.electricity_price
    );
}
