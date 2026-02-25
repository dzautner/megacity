//! Integration tests for POLL-031: residential and commercial heating air pollution.

use crate::grid::ZoneType;
use crate::heating_emissions::{HeatingEmissionsStats, HeatingFuelMix};
use crate::pollution::PollutionGrid;
use crate::test_harness::TestCity;
use crate::wind::WindState;

// ====================================================================
// Heating emissions only active in cold weather
// ====================================================================

#[test]
fn test_heating_emissions_only_active_in_cold_weather() {
    // Warm weather: no heating emissions
    let mut city_warm = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 1)
        .with_weather(25.0);
    {
        let world = city_warm.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_warm.tick_slow_cycle();

    let stats_warm = city_warm.resource::<HeatingEmissionsStats>();
    assert_eq!(
        stats_warm.emitting_buildings, 0,
        "No heating emissions expected in warm weather (25C)"
    );
    assert_eq!(stats_warm.total_emission_q, 0.0);

    // Cold weather: heating emissions active
    let mut city_cold = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 1)
        .with_weather(-5.0);
    {
        let world = city_cold.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_cold.tick_slow_cycle();

    let stats_cold = city_cold.resource::<HeatingEmissionsStats>();
    assert!(
        stats_cold.emitting_buildings > 0,
        "Heating emissions expected in cold weather (-5C), got 0"
    );
    assert!(
        stats_cold.total_emission_q > 0.0,
        "Total emission Q should be positive in cold weather"
    );
}

// ====================================================================
// Electric heating produces zero emissions
// ====================================================================

#[test]
fn test_heating_emissions_electric_produces_zero() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 1)
        .with_weather(-5.0);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        // Set fuel mix to all-electric
        *world.resource_mut::<HeatingFuelMix>() = HeatingFuelMix {
            gas_fraction: 0.0,
            oil_fraction: 0.0,
            wood_fraction: 0.0,
            electric_fraction: 1.0,
        };
    }
    city.tick_slow_cycle();

    let stats = city.resource::<HeatingEmissionsStats>();
    assert_eq!(
        stats.emitting_buildings, 0,
        "All-electric heating should produce zero emitting buildings"
    );
    assert_eq!(
        stats.total_emission_q, 0.0,
        "All-electric heating should produce zero emission Q"
    );
}

// ====================================================================
// Dense residential emits more than sparse
// ====================================================================

#[test]
fn test_heating_emissions_dense_more_than_sparse() {
    // High-density residential
    let mut city_high = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 1)
        .with_weather(-5.0);
    {
        let world = city_high.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_high.tick_slow_cycle();
    let stats_high = city_high.resource::<HeatingEmissionsStats>();
    let q_high = stats_high.total_emission_q;

    // Low-density residential
    let mut city_low = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_weather(-5.0);
    {
        let world = city_low.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_low.tick_slow_cycle();
    let stats_low = city_low.resource::<HeatingEmissionsStats>();
    let q_low = stats_low.total_emission_q;

    assert!(
        q_high > q_low,
        "High-density ({q_high}) should emit more than low-density ({q_low})"
    );
}

// ====================================================================
// Commercial buildings also emit heating pollution
// ====================================================================

#[test]
fn test_heating_emissions_commercial_emits() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialHigh, 1)
        .with_weather(-5.0);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city.tick_slow_cycle();

    let stats = city.resource::<HeatingEmissionsStats>();
    assert!(
        stats.emitting_buildings > 0,
        "Commercial buildings should emit heating pollution in cold weather"
    );
}

// ====================================================================
// Industrial and office buildings do NOT emit heating pollution
// ====================================================================

#[test]
fn test_heating_emissions_industrial_does_not_emit() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 1)
        .with_weather(-5.0);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city.tick_slow_cycle();

    let stats = city.resource::<HeatingEmissionsStats>();
    // Industrial heating emissions should be zero (covered by building_emissions)
    assert_eq!(
        stats.emitting_buildings, 0,
        "Industrial buildings should not emit heating pollution (base_q = None)"
    );
}

#[test]
fn test_heating_emissions_office_does_not_emit() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Office, 1)
        .with_weather(-5.0);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city.tick_slow_cycle();

    let stats = city.resource::<HeatingEmissionsStats>();
    assert_eq!(
        stats.emitting_buildings, 0,
        "Office buildings should not emit heating pollution"
    );
}

// ====================================================================
// Heating pollution adds to PollutionGrid
// ====================================================================

#[test]
fn test_heating_emissions_adds_to_pollution_grid() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 1)
        .with_weather(-5.0);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_building = pollution.get(50, 50);
    assert!(
        at_building > 0,
        "Heating emissions should add pollution at building cell, got {at_building}"
    );
}

// ====================================================================
// Wood fuel mix produces more pollution than gas
// ====================================================================

#[test]
fn test_heating_emissions_wood_more_than_gas() {
    // All-wood fuel mix
    let mut city_wood = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 1)
        .with_weather(-5.0);
    {
        let world = city_wood.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        *world.resource_mut::<HeatingFuelMix>() = HeatingFuelMix {
            gas_fraction: 0.0,
            oil_fraction: 0.0,
            wood_fraction: 1.0,
            electric_fraction: 0.0,
        };
    }
    city_wood.tick_slow_cycle();
    let q_wood = city_wood.resource::<HeatingEmissionsStats>().total_emission_q;

    // All-gas fuel mix
    let mut city_gas = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 1)
        .with_weather(-5.0);
    {
        let world = city_gas.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        *world.resource_mut::<HeatingFuelMix>() = HeatingFuelMix {
            gas_fraction: 1.0,
            oil_fraction: 0.0,
            wood_fraction: 0.0,
            electric_fraction: 0.0,
        };
    }
    city_gas.tick_slow_cycle();
    let q_gas = city_gas.resource::<HeatingEmissionsStats>().total_emission_q;

    assert!(
        q_wood > q_gas,
        "Wood fuel ({q_wood}) should produce more emissions than gas ({q_gas})"
    );
}

// ====================================================================
// Emissions scale with heating demand (colder = more)
// ====================================================================

#[test]
fn test_heating_emissions_scale_with_demand() {
    // Mildly cold: 5C
    let mut city_mild = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 1)
        .with_weather(5.0);
    {
        let world = city_mild.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_mild.tick_slow_cycle();
    let q_mild = city_mild
        .resource::<HeatingEmissionsStats>()
        .total_emission_q;

    // Very cold: -10C
    let mut city_cold = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 1)
        .with_weather(-10.0);
    {
        let world = city_cold.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_cold.tick_slow_cycle();
    let q_cold = city_cold
        .resource::<HeatingEmissionsStats>()
        .total_emission_q;

    assert!(
        q_cold > q_mild,
        "Very cold ({q_cold}) should produce more emissions than mildly cold ({q_mild})"
    );
}

// ====================================================================
// Multiple buildings accumulate emissions
// ====================================================================

#[test]
fn test_heating_emissions_multiple_buildings() {
    // Single building
    let mut city_one = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 1)
        .with_weather(-5.0);
    {
        let world = city_one.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_one.tick_slow_cycle();
    let q_one = city_one
        .resource::<HeatingEmissionsStats>()
        .total_emission_q;

    // Three buildings
    let mut city_three = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialHigh, 1)
        .with_building(52, 52, ZoneType::ResidentialHigh, 1)
        .with_building(54, 54, ZoneType::ResidentialHigh, 1)
        .with_weather(-5.0);
    {
        let world = city_three.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_three.tick_slow_cycle();
    let q_three = city_three
        .resource::<HeatingEmissionsStats>()
        .total_emission_q;

    assert!(
        q_three > q_one,
        "Three buildings ({q_three}) should emit more than one ({q_one})"
    );
    let count = city_three
        .resource::<HeatingEmissionsStats>()
        .emitting_buildings;
    assert_eq!(count, 3, "Should have 3 emitting buildings, got {count}");
}
