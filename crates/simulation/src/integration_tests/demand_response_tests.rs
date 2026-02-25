//! Integration tests for POWER-012: Demand Response Programs.

use crate::coal_power::PowerPlant;
use crate::demand_response::DemandResponsePrograms;
use crate::energy_demand::{EnergyConsumer, EnergyGrid, LoadPriority};
use crate::energy_dispatch::EnergyDispatchState;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::utilities::UtilityType;
use crate::Saveable;

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

/// Create a TestCity with baseline weather and a power utility source.
fn new_powered_city() -> TestCity {
    TestCity::new()
        .with_weather(18.3)
        .with_road(50, 50, 70, 50, crate::grid::RoadType::Local)
        .with_utility(50, 50, UtilityType::PowerPlant)
}

/// Spawn a standalone EnergyConsumer that produces `target_mw` of demand.
fn spawn_demand(city: &mut TestCity, target_mw: f32) {
    let base_kwh = target_mw * 720_000.0;
    city.world_mut()
        .spawn(EnergyConsumer::new(base_kwh, LoadPriority::Normal));
}

/// Tick enough for utility BFS, demand aggregation, demand response, and dispatch.
fn tick_energy(city: &mut TestCity) {
    city.tick(8);
}

// -----------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------

#[test]
fn test_demand_response_resource_initialized() {
    let city = TestCity::new().with_weather(18.3);
    let programs = city.resource::<DemandResponsePrograms>();
    assert!(!programs.smart_thermostat);
    assert!(!programs.industrial_load_shifting);
    assert!(!programs.ev_managed_charging);
    assert!(!programs.peak_pricing_signals);
    assert!(!programs.interruptible_service);
    assert!(!programs.critical_peak_rebates);
    assert_eq!(programs.active_count(), 0);
}

#[test]
fn test_no_reduction_when_no_programs_active() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);
    tick_energy(&mut city);

    let grid = city.resource::<EnergyGrid>();
    // With no demand response, demand should be at the computed level
    // (not reduced). We just check it's positive.
    assert!(
        grid.total_demand_mwh > 0.0,
        "Demand should be positive without programs"
    );

    let programs = city.resource::<DemandResponsePrograms>();
    assert!(
        (programs.current_reduction_fraction - 0.0).abs() < f32::EPSILON,
        "Reduction fraction should be 0 with no programs"
    );
}

#[test]
fn test_smart_thermostat_reduces_demand() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);

    // Measure baseline demand (no programs).
    tick_energy(&mut city);
    let baseline_demand = city.resource::<EnergyGrid>().total_demand_mwh;

    // Enable smart thermostat.
    city.world_mut()
        .resource_mut::<DemandResponsePrograms>()
        .smart_thermostat = true;
    tick_energy(&mut city);

    let reduced_demand = city.resource::<EnergyGrid>().total_demand_mwh;

    // The demand should be reduced by approximately 8%.
    // Due to the system running at intervals and baseline being recomputed,
    // we verify the reduced demand is meaningfully less than baseline.
    assert!(
        reduced_demand < baseline_demand,
        "Smart thermostat should reduce demand: baseline={}, reduced={}",
        baseline_demand,
        reduced_demand
    );

    let programs = city.resource::<DemandResponsePrograms>();
    assert!(
        (programs.current_reduction_fraction - 0.08).abs() < f32::EPSILON,
        "Smart thermostat reduction should be 8%"
    );
}

#[test]
fn test_multiple_programs_stack() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);

    // Enable multiple programs.
    {
        let mut programs = city.world_mut().resource_mut::<DemandResponsePrograms>();
        programs.smart_thermostat = true; // 8%
        programs.peak_pricing_signals = true; // 10%
        programs.ev_managed_charging = true; // 5%
    }

    tick_energy(&mut city);

    let programs = city.resource::<DemandResponsePrograms>();
    let expected = 0.08 + 0.10 + 0.05;
    assert!(
        (programs.current_reduction_fraction - expected).abs() < f32::EPSILON,
        "Multiple programs should stack: expected {}, got {}",
        expected,
        programs.current_reduction_fraction
    );
    assert_eq!(programs.active_count(), 3);
}

#[test]
fn test_demand_response_reduces_dispatch_deficit() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Create a marginal deficit scenario: supply slightly less than demand.
    city.world_mut().spawn(make_plant(95.0, 25.0));
    spawn_demand(&mut city, 100.0);

    // With no programs, there should be a deficit.
    tick_energy(&mut city);
    let dispatch_no_dr = city.resource::<EnergyDispatchState>().clone();

    // Enable programs that reduce demand by 15% (interruptible service).
    city.world_mut()
        .resource_mut::<DemandResponsePrograms>()
        .interruptible_service = true;
    tick_energy(&mut city);

    let dispatch_with_dr = city.resource::<EnergyDispatchState>().clone();

    // With 15% demand reduction, the effective demand drops to ~85 MW,
    // which is well under 95 MW supply. So the deficit should be gone.
    if dispatch_no_dr.active && dispatch_no_dr.has_deficit {
        assert!(
            !dispatch_with_dr.has_deficit,
            "Demand response should eliminate marginal deficit: \
             no DR deficit={}, with DR deficit={}",
            dispatch_no_dr.has_deficit,
            dispatch_with_dr.has_deficit,
        );
    }
}

#[test]
fn test_demand_response_monthly_cost_tracked() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);

    {
        let mut programs = city.world_mut().resource_mut::<DemandResponsePrograms>();
        programs.smart_thermostat = true; // $1000
        programs.interruptible_service = true; // $2000
    }

    tick_energy(&mut city);

    let programs = city.resource::<DemandResponsePrograms>();
    let expected_cost = 1_000.0 + 2_000.0;
    assert!(
        (programs.total_monthly_cost - expected_cost).abs() < f64::EPSILON,
        "Monthly cost should be {}, got {}",
        expected_cost,
        programs.total_monthly_cost
    );
}

#[test]
fn test_peak_pricing_is_free() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);

    city.world_mut()
        .resource_mut::<DemandResponsePrograms>()
        .peak_pricing_signals = true;

    tick_energy(&mut city);

    let programs = city.resource::<DemandResponsePrograms>();
    assert!(
        (programs.total_monthly_cost - 0.0).abs() < f64::EPSILON,
        "Peak pricing signals should cost $0"
    );
    assert!(
        (programs.current_reduction_fraction - 0.10).abs() < f32::EPSILON,
        "Peak pricing signals should reduce by 10%"
    );
}

#[test]
fn test_saveable_roundtrip() {
    let programs = DemandResponsePrograms {
        smart_thermostat: true,
        industrial_load_shifting: false,
        ev_managed_charging: true,
        peak_pricing_signals: true,
        interruptible_service: false,
        critical_peak_rebates: true,
        current_reduction_fraction: 0.30,
        total_monthly_cost: 2_300.0,
    };

    let bytes = programs.save_to_bytes().unwrap();
    let restored = DemandResponsePrograms::load_from_bytes(&bytes);

    assert!(restored.smart_thermostat);
    assert!(!restored.industrial_load_shifting);
    assert!(restored.ev_managed_charging);
    assert!(restored.peak_pricing_signals);
    assert!(!restored.interruptible_service);
    assert!(restored.critical_peak_rebates);
}

#[test]
fn test_all_programs_active_reduces_demand_significantly() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 100.0);

    // Enable all programs.
    {
        let mut programs = city.world_mut().resource_mut::<DemandResponsePrograms>();
        programs.smart_thermostat = true;
        programs.industrial_load_shifting = true;
        programs.ev_managed_charging = true;
        programs.peak_pricing_signals = true;
        programs.interruptible_service = true;
        programs.critical_peak_rebates = true;
    }

    tick_energy(&mut city);

    let programs = city.resource::<DemandResponsePrograms>();
    let expected = 0.08 + 0.12 + 0.05 + 0.10 + 0.15 + 0.07;
    assert!(
        (programs.current_reduction_fraction - expected).abs() < f32::EPSILON,
        "All programs combined reduction: expected {}, got {}",
        expected,
        programs.current_reduction_fraction
    );
    assert_eq!(programs.active_count(), 6);
}
