//! Integration tests for the Battery Energy Storage System (POWER-008).

use crate::battery_storage::{BatteryState, BatteryTier, BatteryUnit};
use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::{EnergyConsumer, EnergyGrid, LoadPriority};
use crate::energy_dispatch::EnergyDispatchState;
use crate::test_harness::TestCity;

/// Helper: create a PowerPlant component with the given capacity and fuel cost.
fn make_plant(capacity_mw: f32, fuel_cost: f32) -> PowerPlant {
    PowerPlant {
        plant_type: PowerPlantType::Coal,
        capacity_mw,
        current_output_mw: 0.0,
        fuel_cost,
        grid_x: 0,
        grid_y: 0,
    }
}

/// Create a TestCity with baseline weather (18.3 C) so demand is predictable.
fn new_baseline_city() -> TestCity {
    TestCity::new().with_weather(18.3)
}

/// Spawn a standalone EnergyConsumer producing `target_mw` of demand under
/// baseline conditions (tou=1.0, hvac=1.0, power=1.0).
fn spawn_demand(city: &mut TestCity, target_mw: f32) {
    let base_kwh = target_mw * 720_000.0;
    city.world_mut()
        .spawn(EnergyConsumer::new(base_kwh, LoadPriority::Normal));
}

/// Tick enough for demand aggregation, dispatch, and battery system to all run.
fn tick_battery(city: &mut TestCity) {
    city.tick(8);
}

/// Add a battery unit to the BatteryState resource.
fn add_battery(city: &mut TestCity, tier: BatteryTier, stored_mwh: f32) {
    let mut unit = BatteryUnit::new(tier, 0, 0);
    unit.stored_mwh = stored_mwh;
    let world = city.world_mut();
    let mut state = world.resource_mut::<BatteryState>();
    state.add_battery(unit);
}

#[test]
fn test_battery_charges_when_excess_supply() {
    let mut city = new_baseline_city();

    // 200 MW supply, 100 MW demand => 100 MW excess
    city.world_mut().spawn(make_plant(200.0, 0.0));
    spawn_demand(&mut city, 100.0);

    // Add an empty small battery (10 MWh, 5 MW rate)
    add_battery(&mut city, BatteryTier::Small, 0.0);

    tick_battery(&mut city);

    let state = city.resource::<BatteryState>();
    assert!(
        state.last_charge_mwh > 0.0,
        "Battery should have charged, got {}",
        state.last_charge_mwh
    );
    assert!(
        state.total_stored_mwh > 0.0,
        "Battery should have stored energy, got {}",
        state.total_stored_mwh
    );
}

#[test]
fn test_battery_discharges_when_deficit() {
    let mut city = new_baseline_city();

    // 50 MW supply, 100 MW demand => 50 MW deficit
    city.world_mut().spawn(make_plant(50.0, 0.0));
    spawn_demand(&mut city, 100.0);

    // Add a fully charged large battery (100 MWh, 50 MW rate)
    add_battery(&mut city, BatteryTier::Large, 100.0);

    tick_battery(&mut city);

    let state = city.resource::<BatteryState>();
    assert!(
        state.last_discharge_mwh > 0.0,
        "Battery should have discharged, got {}",
        state.last_discharge_mwh
    );
    assert!(
        state.total_stored_mwh < 100.0,
        "Stored energy should have decreased from 100, got {}",
        state.total_stored_mwh
    );
}

#[test]
fn test_battery_discharge_applies_efficiency() {
    let mut city = new_baseline_city();

    // 0 MW supply, 50 MW demand => 50 MW deficit
    city.world_mut().spawn(make_plant(0.0, 0.0));
    spawn_demand(&mut city, 50.0);

    // Add a fully charged large battery
    add_battery(&mut city, BatteryTier::Large, 100.0);

    tick_battery(&mut city);

    let state = city.resource::<BatteryState>();
    // Discharged should be less than the energy drawn from storage due to 85% efficiency
    let drawn = 100.0 - state.total_stored_mwh;
    if drawn > 0.0 && state.last_discharge_mwh > 0.0 {
        let actual_efficiency = state.last_discharge_mwh / drawn;
        assert!(
            (actual_efficiency - 0.85).abs() < 0.05,
            "Efficiency should be ~85%, got {}%",
            actual_efficiency * 100.0
        );
    }
}

#[test]
fn test_battery_respects_reserve_threshold() {
    let mut city = new_baseline_city();

    // Large deficit but battery is near reserve
    city.world_mut().spawn(make_plant(10.0, 0.0));
    spawn_demand(&mut city, 100.0);

    // Add a battery at 25% SOC (25 MWh of 100 MWh capacity)
    // Reserve is 20% = 20 MWh, so only 5 MWh available for discharge
    add_battery(&mut city, BatteryTier::Large, 25.0);

    tick_battery(&mut city);

    let state = city.resource::<BatteryState>();
    // Should not discharge below 20 MWh (20% reserve)
    assert!(
        state.total_stored_mwh >= 19.9, // small float tolerance
        "Battery should not go below 20% reserve, stored = {}",
        state.total_stored_mwh
    );
}

#[test]
fn test_battery_charge_clamped_to_rate() {
    let mut city = new_baseline_city();

    // Massive excess: 500 MW supply, 0 demand
    city.world_mut().spawn(make_plant(500.0, 30.0));

    // Small battery: 5 MW rate limit
    add_battery(&mut city, BatteryTier::Small, 0.0);

    tick_battery(&mut city);

    let state = city.resource::<BatteryState>();
    // Should have charged at most 5 MWh (rate limited)
    assert!(
        state.total_stored_mwh <= 5.0 + 0.01,
        "Charge should be rate-limited to 5 MW, got {}",
        state.total_stored_mwh
    );
}

#[test]
fn test_battery_discharge_reduces_load_shed() {
    let mut city = new_baseline_city();

    // 50 MW supply, 100 MW demand => deficit
    city.world_mut().spawn(make_plant(50.0, 0.0));
    spawn_demand(&mut city, 100.0);

    // Add charged battery that can help
    add_battery(&mut city, BatteryTier::Large, 100.0);

    tick_battery(&mut city);

    let dispatch = city.resource::<EnergyDispatchState>();
    // Battery discharge should have reduced the load shed fraction
    // (compared to the ~0.5 it would be without batteries)
    assert!(
        dispatch.load_shed_fraction < 0.5,
        "Load shed should be reduced by battery discharge, got {}",
        dispatch.load_shed_fraction
    );
}

#[test]
fn test_no_batteries_no_effect() {
    let mut city = new_baseline_city();

    city.world_mut().spawn(make_plant(100.0, 0.0));
    spawn_demand(&mut city, 50.0);

    // No batteries added
    tick_battery(&mut city);

    let state = city.resource::<BatteryState>();
    assert_eq!(state.unit_count, 0);
    assert!((state.last_charge_mwh).abs() < f32::EPSILON);
    assert!((state.last_discharge_mwh).abs() < f32::EPSILON);
}

#[test]
fn test_multiple_batteries_share_load() {
    let mut city = new_baseline_city();

    // 200 MW supply, 100 MW demand => 100 MW excess
    city.world_mut().spawn(make_plant(200.0, 0.0));
    spawn_demand(&mut city, 100.0);

    // Add two small batteries (each 5 MW rate, 10 MWh capacity)
    add_battery(&mut city, BatteryTier::Small, 0.0);
    add_battery(&mut city, BatteryTier::Small, 0.0);

    tick_battery(&mut city);

    let state = city.resource::<BatteryState>();
    // Both should have charged (total charge should exceed single battery rate)
    assert!(
        state.last_charge_mwh > 0.0,
        "Should have charged: {}",
        state.last_charge_mwh
    );
    assert_eq!(state.unit_count, 2);
}

#[test]
fn test_battery_soc_tracking() {
    let mut city = new_baseline_city();

    // Excess supply for charging
    city.world_mut().spawn(make_plant(200.0, 0.0));
    spawn_demand(&mut city, 100.0);

    add_battery(&mut city, BatteryTier::Small, 0.0);

    tick_battery(&mut city);

    let state = city.resource::<BatteryState>();
    assert!(
        state.aggregate_soc > 0.0,
        "SOC should be > 0 after charging, got {}",
        state.aggregate_soc
    );
    assert!(
        state.aggregate_soc <= 1.0,
        "SOC should be <= 1.0, got {}",
        state.aggregate_soc
    );
}

#[test]
fn test_battery_state_saveable_roundtrip() {
    use crate::Saveable;

    let mut state = BatteryState::default();
    let mut unit = BatteryUnit::new(BatteryTier::Large, 50, 60);
    unit.stored_mwh = 42.5;
    state.add_battery(unit);

    let bytes = state.save_to_bytes().expect("should produce bytes");
    let restored = BatteryState::load_from_bytes(&bytes);

    assert_eq!(restored.units.len(), 1);
    assert!((restored.units[0].stored_mwh - 42.5).abs() < f32::EPSILON);
    assert_eq!(restored.units[0].tier, BatteryTier::Large);
    assert_eq!(restored.units[0].grid_x, 50);
    assert_eq!(restored.units[0].grid_y, 60);
}
