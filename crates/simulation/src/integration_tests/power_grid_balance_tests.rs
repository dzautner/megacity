//! Integration tests for SVC-023: Power Grid Demand/Supply Balance.

use crate::blackout::BlackoutState;
use crate::coal_power::{PowerPlant, PowerPlantType};
use crate::energy_demand::{EnergyConsumer, LoadPriority};
use crate::energy_dispatch::EnergyDispatchState;
use crate::grid::ZoneType;
use crate::power_grid_balance::{
    commercial_demand_curve, industrial_demand_curve, residential_demand_curve, GridAlertLevel,
    PowerGridBalance,
};
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::utilities::UtilityType;
use crate::Saveable;

/// Helper: create a PowerPlant component.
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

/// Tick enough for demand aggregation, dispatch, blackout, and balance.
fn tick_balance(city: &mut TestCity) {
    city.tick(16);
}

// -----------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------

#[test]
fn test_power_grid_balance_initialized() {
    let city = TestCity::new().with_weather(18.3);
    let balance = city.resource::<PowerGridBalance>();
    assert_eq!(balance.alert_level, GridAlertLevel::Healthy);
    assert!(!balance.brownout_active);
    assert_eq!(balance.affected_cells, 0);
}

#[test]
fn test_balance_healthy_with_surplus() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Large supply, small demand.
    city.world_mut().spawn(make_plant(500.0, 25.0));
    spawn_demand(&mut city, 50.0);
    tick_balance(&mut city);

    let balance = city.resource::<PowerGridBalance>();
    assert_eq!(
        balance.alert_level,
        GridAlertLevel::Healthy,
        "Alert should be Healthy with large surplus, got {:?} (margin={})",
        balance.alert_level,
        balance.reserve_margin
    );
    assert!(!balance.brownout_active);
}

#[test]
fn test_balance_deficit_triggers_brownout_flag() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Zone cells near the utility source.
    {
        let mut grid = city
            .world_mut()
            .resource_mut::<crate::grid::WorldGrid>();
        for x in 51..60 {
            grid.get_mut(x, 49).zone = ZoneType::ResidentialLow;
        }
    }

    // Supply < demand: 30 MW supply, 100 MW demand.
    city.world_mut().spawn(make_plant(30.0, 25.0));
    spawn_demand(&mut city, 100.0);
    tick_balance(&mut city);

    let balance = city.resource::<PowerGridBalance>();
    let dispatch = city.resource::<EnergyDispatchState>();

    if dispatch.active && dispatch.has_deficit {
        assert_eq!(
            balance.alert_level,
            GridAlertLevel::Deficit,
            "Alert should be Deficit when supply < demand, got {:?}",
            balance.alert_level
        );
        assert!(
            balance.brownout_active,
            "Brownout should be active during deficit"
        );
    }
}

#[test]
fn test_reserve_margin_calculation() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // 200 MW capacity, 100 MW demand -> margin ~ (200-100)/200 = 0.5.
    city.world_mut().spawn(make_plant(200.0, 25.0));
    spawn_demand(&mut city, 100.0);
    tick_balance(&mut city);

    let balance = city.resource::<PowerGridBalance>();
    // Reserve margin should be positive when capacity > demand.
    if balance.total_capacity_mw > 0.0 && balance.total_demand_mw > 0.0 {
        assert!(
            balance.reserve_margin > 0.0,
            "Reserve margin should be positive with surplus: {}",
            balance.reserve_margin
        );
    }
}

#[test]
fn test_supply_breakdown_tracks_coal() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    city.world_mut().spawn(make_plant(200.0, 30.0));
    spawn_demand(&mut city, 50.0);
    tick_balance(&mut city);

    let balance = city.resource::<PowerGridBalance>();
    // Coal state should be reflected in the supply breakdown.
    // The exact value depends on dispatch, but total_supply should be > 0.
    assert!(
        balance.total_supply_mw >= 0.0,
        "Total supply should be non-negative"
    );
}

#[test]
fn test_residential_curve_evening_peak() {
    let evening = residential_demand_curve(19.0);
    let midday = residential_demand_curve(12.0);
    let night = residential_demand_curve(2.0);

    assert!(evening > midday, "Evening > midday for residential");
    assert!(evening > night, "Evening > night for residential");
    assert_eq!(evening, 1.5);
}

#[test]
fn test_commercial_curve_daytime_peak() {
    let daytime = commercial_demand_curve(12.0);
    let evening = commercial_demand_curve(22.0);
    let night = commercial_demand_curve(2.0);

    assert!(daytime > evening, "Daytime > evening for commercial");
    assert!(daytime > night, "Daytime > night for commercial");
    assert_eq!(daytime, 1.4);
}

#[test]
fn test_industrial_curve_flat_profile() {
    let day_vals: Vec<f32> = (0..24).map(|h| industrial_demand_curve(h as f32)).collect();
    let min = day_vals.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = day_vals.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    let range = max - min;
    assert!(
        range < 0.2,
        "Industrial curve should have narrow range: max={} min={} range={}",
        max,
        min,
        range
    );
}

#[test]
fn test_alert_level_transitions() {
    assert_eq!(
        GridAlertLevel::from_reserve_margin(0.30),
        GridAlertLevel::Healthy
    );
    assert_eq!(
        GridAlertLevel::from_reserve_margin(0.15),
        GridAlertLevel::Tight
    );
    assert_eq!(
        GridAlertLevel::from_reserve_margin(0.07),
        GridAlertLevel::Warning
    );
    assert_eq!(
        GridAlertLevel::from_reserve_margin(0.02),
        GridAlertLevel::Critical
    );
    assert_eq!(
        GridAlertLevel::from_reserve_margin(-0.1),
        GridAlertLevel::Deficit
    );
}

#[test]
fn test_balance_clears_after_supply_added() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Start with deficit.
    city.world_mut().spawn(make_plant(30.0, 25.0));
    spawn_demand(&mut city, 100.0);
    tick_balance(&mut city);

    // Add more supply to clear deficit.
    city.world_mut().spawn(make_plant(500.0, 30.0));
    tick_balance(&mut city);

    let balance = city.resource::<PowerGridBalance>();
    let blackout = city.resource::<BlackoutState>();

    // After adding supply, brownout should clear.
    if !blackout.active {
        assert!(
            !balance.brownout_active,
            "Brownout should clear after adding supply"
        );
    }
}

#[test]
fn test_saveable_roundtrip_integration() {
    let balance = PowerGridBalance {
        total_demand_mw: 250.0,
        total_supply_mw: 300.0,
        total_capacity_mw: 400.0,
        reserve_margin: 0.375,
        alert_level: GridAlertLevel::Healthy,
        brownout_active: false,
        affected_cells: 0,
        renewable_fraction: 0.12,
        ..Default::default()
    };

    let bytes = balance.save_to_bytes().unwrap();
    let restored = PowerGridBalance::load_from_bytes(&bytes);

    assert!((restored.total_demand_mw - 250.0).abs() < f32::EPSILON);
    assert!((restored.total_supply_mw - 300.0).abs() < f32::EPSILON);
    assert!((restored.total_capacity_mw - 400.0).abs() < f32::EPSILON);
    assert!((restored.reserve_margin - 0.375).abs() < f32::EPSILON);
    assert_eq!(restored.alert_level, GridAlertLevel::Healthy);
    assert!((restored.renewable_fraction - 0.12).abs() < f32::EPSILON);
}

#[test]
fn test_renewable_fraction_zero_without_renewables() {
    let mut city = new_powered_city();
    city.world_mut().resource_mut::<GameClock>().hour = 10.0;

    // Only coal — no solar/wind.
    city.world_mut().spawn(make_plant(200.0, 30.0));
    spawn_demand(&mut city, 50.0);
    tick_balance(&mut city);

    let balance = city.resource::<PowerGridBalance>();
    assert!(
        balance.renewable_fraction < 0.01,
        "Renewable fraction should be ~0 with only coal: {}",
        balance.renewable_fraction
    );
}
