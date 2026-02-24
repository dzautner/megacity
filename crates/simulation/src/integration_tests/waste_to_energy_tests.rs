//! Integration tests for the Waste-to-Energy power plant (POWER-014).

use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::{EnergyConsumer, EnergyGrid, LoadPriority};
use crate::garbage::WasteSystem;
use crate::test_harness::TestCity;
use crate::waste_composition::WasteComposition;
use crate::waste_to_energy::{
    calculate_wte_output_mw, WtePlant, WteState, WTE_ASH_FRACTION,
    WTE_DEFAULT_WASTE_TONS, WTE_OPERATING_COST_PER_TON, WTE_POLLUTION_Q_RAW,
    WTE_POLLUTION_Q_SCRUBBED, WTE_TIPPING_FEE_PER_TON,
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

/// Set the waste system's period_generated_tons to simulate available waste.
fn set_available_waste(city: &mut TestCity, tons: f64) {
    let world = city.world_mut();
    let mut waste = world.resource_mut::<WasteSystem>();
    waste.period_generated_tons = tons;
}

/// Tick enough for slow tick + dispatch to run.
fn tick_wte(city: &mut TestCity) {
    city.tick_slow_cycle();
}

#[test]
fn test_wte_plant_produces_energy() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_demand(&mut city, 50.0);
    set_available_waste(&mut city, 500.0);

    tick_wte(&mut city);

    let state = city.resource::<WteState>();
    assert_eq!(state.plant_count, 1, "Should have 1 WTE plant");
    assert!(
        state.total_output_mw > 0.0,
        "WTE should produce energy, got {} MW",
        state.total_output_mw
    );
}

#[test]
fn test_wte_plant_consumes_waste() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_demand(&mut city, 50.0);
    set_available_waste(&mut city, 500.0);

    tick_wte(&mut city);

    let state = city.resource::<WteState>();
    assert!(
        state.total_waste_consumed_tons > 0.0,
        "WTE should consume waste, got {} tons",
        state.total_waste_consumed_tons
    );
    assert!(
        state.total_waste_consumed_tons <= 500.0,
        "Should not consume more waste than available"
    );
}

#[test]
fn test_wte_ash_residue_is_ten_percent() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_demand(&mut city, 50.0);
    set_available_waste(&mut city, 500.0);

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
fn test_wte_tipping_fees_exceed_operating_cost() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_demand(&mut city, 50.0);
    set_available_waste(&mut city, 500.0);

    tick_wte(&mut city);

    let state = city.resource::<WteState>();
    if state.total_waste_consumed_tons > 0.0 {
        assert!(
            state.total_tipping_revenue > state.total_operating_cost,
            "Tipping revenue ({}) should exceed operating cost ({})",
            state.total_tipping_revenue,
            state.total_operating_cost
        );
    }
}

#[test]
fn test_wte_no_waste_no_output() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_demand(&mut city, 50.0);
    set_available_waste(&mut city, 0.0);

    tick_wte(&mut city);

    let state = city.resource::<WteState>();
    assert!(
        state.total_output_mw.abs() < f32::EPSILON,
        "No waste = no output, got {} MW",
        state.total_output_mw
    );
}

#[test]
fn test_wte_scrubbers_reduce_pollution() {
    let state_scrubbed = WteState {
        scrubbers_installed: true,
        ..Default::default()
    };
    let state_raw = WteState {
        scrubbers_installed: false,
        ..Default::default()
    };

    let q_scrubbed = if state_scrubbed.scrubbers_installed {
        WTE_POLLUTION_Q_SCRUBBED
    } else {
        WTE_POLLUTION_Q_RAW
    };
    let q_raw = if state_raw.scrubbers_installed {
        WTE_POLLUTION_Q_SCRUBBED
    } else {
        WTE_POLLUTION_Q_RAW
    };

    assert!(
        q_scrubbed < q_raw,
        "Scrubbed Q ({q_scrubbed}) should be less than raw Q ({q_raw})"
    );
}

#[test]
fn test_wte_output_formula_matches_spec() {
    // Verify formula: waste_tons * BTU_per_lb * 2000 * boiler_eff * gen_eff / 3412000
    let btu = WasteComposition::default().energy_content_btu_per_lb();
    let output = calculate_wte_output_mw(WTE_DEFAULT_WASTE_TONS, btu);

    // Manual calculation for 500 tons/day:
    let expected = WTE_DEFAULT_WASTE_TONS * btu * 2000.0 * 0.80 * 0.33 / 3_412_000.0;
    assert!(
        (output - expected).abs() < 0.01,
        "Formula mismatch: got {output}, expected {expected}"
    );

    // Should be in the ~15 MW range
    assert!(
        output > 10.0 && output < 25.0,
        "Default output should be ~15 MW, got {output}"
    );
}

#[test]
fn test_wte_adds_to_energy_grid() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_demand(&mut city, 50.0);
    set_available_waste(&mut city, 500.0);

    tick_wte(&mut city);

    let grid = city.resource::<EnergyGrid>();
    assert!(
        grid.total_supply_mwh > 0.0,
        "Energy grid supply should increase, got {}",
        grid.total_supply_mwh
    );
}

#[test]
fn test_multiple_wte_plants() {
    let mut city = new_baseline_city();
    spawn_wte_plant(&mut city, 10, 10);
    spawn_wte_plant(&mut city, 20, 20);
    spawn_demand(&mut city, 100.0);
    set_available_waste(&mut city, 1000.0);

    tick_wte(&mut city);

    let state = city.resource::<WteState>();
    assert_eq!(state.plant_count, 2, "Should have 2 WTE plants");
    assert!(
        state.total_output_mw > 0.0,
        "Multiple plants should produce energy"
    );
}

#[test]
fn test_wte_waste_capped_at_capacity() {
    let mut city = new_baseline_city();
    // Spawn a plant with default 500 ton/day capacity
    spawn_wte_plant(&mut city, 10, 10);
    spawn_demand(&mut city, 50.0);
    // Provide way more waste than the plant can handle
    set_available_waste(&mut city, 10_000.0);

    tick_wte(&mut city);

    let state = city.resource::<WteState>();
    assert!(
        state.total_waste_consumed_tons <= WTE_DEFAULT_WASTE_TONS + 0.1,
        "Should not consume more than capacity: got {} tons",
        state.total_waste_consumed_tons
    );
}

#[test]
fn test_wte_state_saveable_roundtrip() {
    use crate::Saveable;

    let state = WteState {
        plant_count: 3,
        total_waste_consumed_tons: 1200.0,
        total_output_mw: 36.9,
        total_ash_tons: 120.0,
        total_operating_cost: 60000.0,
        total_tipping_revenue: 78000.0,
        scrubbers_installed: false,
        current_pollution_q: WTE_POLLUTION_Q_RAW,
    };

    let bytes = state.save_to_bytes().expect("should produce bytes");
    let loaded = WteState::load_from_bytes(&bytes);

    assert_eq!(loaded.plant_count, 3);
    assert!((loaded.total_output_mw - 36.9).abs() < 0.1);
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
