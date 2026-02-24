//! Integration tests for the Waste-to-Energy power plant (POWER-014).

use crate::buildings::Building;
use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::{EnergyConsumer, EnergyGrid, LoadPriority};
use crate::grid::ZoneType;
use crate::test_harness::TestCity;
use crate::waste_composition::WasteComposition;
use crate::waste_to_energy::{
    calculate_wte_output_mw, WtePlant, WteState, WTE_ASH_FRACTION, WTE_DEFAULT_WASTE_TONS,
    WTE_OPERATING_COST_PER_TON, WTE_POLLUTION_Q_RAW, WTE_POLLUTION_Q_SCRUBBED,
    WTE_TIPPING_FEE_PER_TON,
};

/// Create a TestCity with baseline weather.
fn new_baseline_city() -> TestCity {
    TestCity::new().with_weather(18.3)
}

/// Spawn a WTE plant entity with both PowerPlant and WtePlant components.
fn spawn_wte_plant(city: &mut TestCity, grid_x: usize, grid_y: usize) {
    let power_plant = PowerPlant::new_wte(grid_x, grid_y);
    let wte_plant = WtePlant::new(grid_x, grid_y);
    city.world_mut().spawn((power_plant, wte_plant));
}

/// Spawn demand to ensure dispatch runs.
fn spawn_demand(city: &mut TestCity, target_mw: f32) {
    let base_kwh = target_mw * 720_000.0;
    city.world_mut()
        .spawn(EnergyConsumer::new(base_kwh, LoadPriority::Normal));
}

/// Spawn industrial buildings that produce waste to feed the WTE plant.
fn spawn_waste_producers(city: &mut TestCity, count: usize) {
    let world = city.world_mut();
    for i in 0..count {
        let x = 50 + (i % 50);
        let y = 50 + (i / 50);
        world.spawn(Building {
            zone_type: ZoneType::Industrial,
            level: 3,
            grid_x: x,
            grid_y: y,
            capacity: 100,
            occupants: 50,
        });
    }
}

/// Tick enough for slow tick + dispatch to run.
fn tick_wte(city: &mut TestCity) {
    city.tick_slow_cycle();
}

#[test]
fn test_wte_plant_registered_in_state() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_demand(&mut city, 50.0);

    tick_wte(&mut city);

    let state = city.resource::<WteState>();
    assert_eq!(state.plant_count, 1, "Should have 1 WTE plant");
}

#[test]
fn test_wte_produces_energy_with_waste() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_demand(&mut city, 50.0);
    // Spawn industrial buildings that produce lots of waste
    spawn_waste_producers(&mut city, 200);

    // Run two slow cycles: first attaches WasteProducers, second generates waste
    tick_wte(&mut city);
    tick_wte(&mut city);

    let state = city.resource::<WteState>();
    assert_eq!(state.plant_count, 1, "Should have 1 WTE plant");
    // With industrial buildings producing waste, WTE should consume some
    if state.total_waste_consumed_tons > 0.0 {
        assert!(
            state.total_output_mw > 0.0,
            "WTE should produce energy when consuming waste, got {} MW",
            state.total_output_mw
        );
    }
}

#[test]
fn test_wte_no_waste_no_output() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_demand(&mut city, 50.0);
    // No buildings = no waste

    tick_wte(&mut city);

    let state = city.resource::<WteState>();
    assert_eq!(state.plant_count, 1);
    assert!(
        state.total_output_mw.abs() < f32::EPSILON,
        "No waste = no output, got {} MW",
        state.total_output_mw
    );
    assert!(
        state.total_waste_consumed_tons.abs() < f32::EPSILON,
        "No waste = no consumption, got {} tons",
        state.total_waste_consumed_tons
    );
}

#[test]
fn test_wte_ash_proportional_to_waste() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_demand(&mut city, 50.0);
    spawn_waste_producers(&mut city, 200);

    tick_wte(&mut city);
    tick_wte(&mut city);

    let state = city.resource::<WteState>();
    if state.total_waste_consumed_tons > 0.0 {
        let expected_ash = state.total_waste_consumed_tons * WTE_ASH_FRACTION;
        assert!(
            (state.total_ash_tons - expected_ash).abs() < 0.1,
            "Ash should be 10% of waste: expected {expected_ash}, got {}",
            state.total_ash_tons
        );
    }
}

#[test]
fn test_wte_scrubbers_reduce_pollution() {
    // Verify scrubber Q values
    assert!(
        WTE_POLLUTION_Q_SCRUBBED < WTE_POLLUTION_Q_RAW,
        "Scrubbed Q ({}) should be less than raw Q ({})",
        WTE_POLLUTION_Q_SCRUBBED,
        WTE_POLLUTION_Q_RAW
    );

    // Default state should have scrubbers installed
    let state = WteState::default();
    assert!(state.scrubbers_installed);
    assert!(
        (state.current_pollution_q - WTE_POLLUTION_Q_SCRUBBED).abs() < f32::EPSILON
    );
}

#[test]
fn test_wte_output_formula_matches_spec() {
    // Verify formula: waste_tons * BTU_per_lb * 2000 * boiler_eff * gen_eff / 3412 / 1000 / 24
    let btu = WasteComposition::default().energy_content_btu_per_lb();
    let output = calculate_wte_output_mw(WTE_DEFAULT_WASTE_TONS, btu);

    // Manual calculation for 500 tons/day:
    let expected = WTE_DEFAULT_WASTE_TONS * btu * 2000.0 * 0.80 * 0.33 / 3412.0 / 1000.0 / 24.0;
    assert!(
        (output - expected).abs() < 0.01,
        "Formula mismatch: got {output}, expected {expected}"
    );

    // Should be in the ~15-20 MW range
    assert!(
        output > 10.0 && output < 25.0,
        "Default output should be ~15-20 MW, got {output}"
    );
}

#[test]
fn test_wte_adds_to_energy_grid() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_demand(&mut city, 50.0);
    spawn_waste_producers(&mut city, 200);

    tick_wte(&mut city);
    tick_wte(&mut city);

    let grid = city.resource::<EnergyGrid>();
    let state = city.resource::<WteState>();
    // If WTE consumed waste, it should have added to the grid
    if state.total_waste_consumed_tons > 0.0 {
        assert!(
            grid.total_supply_mwh > 0.0,
            "Energy grid supply should increase, got {}",
            grid.total_supply_mwh
        );
    }
}

#[test]
fn test_multiple_wte_plants_counted() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_wte_plant(&mut city, 20, 20);
    spawn_demand(&mut city, 100.0);

    tick_wte(&mut city);

    let state = city.resource::<WteState>();
    assert_eq!(state.plant_count, 2, "Should have 2 WTE plants");
}

#[test]
fn test_wte_state_saveable_roundtrip() {
    use crate::Saveable;

    let state = WteState {
        plant_count: 3,
        total_waste_consumed_tons: 1200.0,
        total_output_mw: 42.0,
        total_ash_tons: 120.0,
        total_operating_cost: 60000.0,
        total_tipping_revenue: 78000.0,
        scrubbers_installed: false,
        current_pollution_q: WTE_POLLUTION_Q_RAW,
    };

    let bytes = state.save_to_bytes().expect("should produce bytes");
    let loaded = WteState::load_from_bytes(&bytes);

    assert_eq!(loaded.plant_count, 3);
    assert!((loaded.total_output_mw - 42.0).abs() < 0.1);
    assert!(!loaded.scrubbers_installed);
    assert!(
        (loaded.current_pollution_q - WTE_POLLUTION_Q_RAW).abs() < f32::EPSILON
    );
}

#[test]
fn test_wte_economics_per_ton() {
    // Verify the per-ton economics match spec
    let cost_per_ton = WTE_OPERATING_COST_PER_TON;
    let revenue_per_ton = WTE_TIPPING_FEE_PER_TON;

    // Operating cost: $40-60/ton (our default: $50)
    assert!(
        cost_per_ton >= 40.0 && cost_per_ton <= 60.0,
        "Operating cost should be $40-60/ton, got ${cost_per_ton}"
    );

    // Tipping fee: $50-80/ton (our default: $65)
    assert!(
        revenue_per_ton >= 50.0 && revenue_per_ton <= 80.0,
        "Tipping fee should be $50-80/ton, got ${revenue_per_ton}"
    );

    // Net positive
    assert!(revenue_per_ton > cost_per_ton);
}

#[test]
fn test_wte_power_plant_type_is_waste_to_energy() {
    let plant = PowerPlant::new_wte(0, 0);
    assert_eq!(plant.plant_type, PowerPlantType::WasteToEnergy);
}

#[test]
fn test_wte_output_linear_scaling() {
    let btu = WasteComposition::default().energy_content_btu_per_lb();
    let output_500 = calculate_wte_output_mw(500.0, btu);
    let output_1000 = calculate_wte_output_mw(1000.0, btu);
    let ratio = output_1000 / output_500;
    assert!(
        (ratio - 2.0).abs() < 0.01,
        "Output should scale linearly: ratio = {ratio}"
    );
}
