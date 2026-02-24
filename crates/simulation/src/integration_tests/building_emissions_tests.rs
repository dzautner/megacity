//! Integration tests for POLL-002: per-building-type air pollution emission rates.

use crate::building_emissions::{building_emission_profile, service_emission_profile};
use crate::grid::ZoneType;
use crate::policies::{Policies, Policy};
use crate::pollution::PollutionGrid;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::traffic::TrafficGrid;
use crate::wind::WindState;

// ====================================================================
// Industrial buildings emit pollution that scales with level
// ====================================================================

#[test]
fn test_building_emissions_industrial_emits() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 1);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_building = pollution.get(50, 50);
    assert!(
        at_building > 0,
        "Industrial L1 building should emit pollution, got {at_building}"
    );
}

#[test]
fn test_building_emissions_industrial_scales_with_level() {
    let mut city_l1 = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 1);
    {
        let world = city_l1.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_l1.tick_slow_cycle();
    let p_l1 = city_l1.resource::<PollutionGrid>().get(50, 50);

    let mut city_l3 = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 3);
    {
        let world = city_l3.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        world.resource_mut::<crate::stats::CityStats>().average_happiness = 50.0;
    }
    city_l3.tick_slow_cycle();
    let p_l3 = city_l3.resource::<PollutionGrid>().get(50, 50);

    assert!(
        p_l3 > p_l1,
        "L3 industrial ({p_l3}) should produce more pollution than L1 ({p_l1})"
    );
}

// ====================================================================
// Commercial buildings emit low pollution
// ====================================================================

#[test]
fn test_building_emissions_commercial_lower_than_industrial() {
    let mut city_com = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialHigh, 1);
    {
        let world = city_com.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_com.tick_slow_cycle();
    let p_com = city_com.resource::<PollutionGrid>().get(50, 50);

    let mut city_ind = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 1);
    {
        let world = city_ind.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_ind.tick_slow_cycle();
    let p_ind = city_ind.resource::<PollutionGrid>().get(50, 50);

    assert!(
        p_ind > p_com,
        "Industrial ({p_ind}) should pollute more than commercial ({p_com})"
    );
}

// ====================================================================
// Office buildings do not emit
// ====================================================================

#[test]
fn test_building_emissions_office_does_not_emit() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Office, 1);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_building = pollution.get(50, 50);
    assert_eq!(
        at_building, 0,
        "Office building should not emit pollution, got {at_building}"
    );
}

// ====================================================================
// Service building emissions (incinerator, heating boiler)
// ====================================================================

#[test]
fn test_building_emissions_incinerator_emits() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::Incinerator);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_service = pollution.get(50, 50);
    assert!(
        at_service > 0,
        "Incinerator should emit pollution, got {at_service}"
    );
}

#[test]
fn test_building_emissions_heating_boiler_emits() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::HeatingBoiler);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_service = pollution.get(50, 50);
    assert!(
        at_service > 0,
        "Heating boiler should emit pollution, got {at_service}"
    );
}

#[test]
fn test_building_emissions_geothermal_does_not_emit() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::GeothermalPlant);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycle();

    let pollution = city.resource::<PollutionGrid>();
    let at_service = pollution.get(50, 50);
    assert_eq!(
        at_service, 0,
        "Geothermal should not emit pollution, got {at_service}"
    );
}

// ====================================================================
// Road emissions scale with traffic
// ====================================================================

#[test]
fn test_building_emissions_congested_road_more_than_empty() {
    use crate::grid::RoadType;

    // City with road and no traffic
    let mut city_empty = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local);
    {
        let world = city_empty.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }
    city_empty.tick_slow_cycle();

    let p_empty: u32 = (50..=60)
        .map(|x| city_empty.resource::<PollutionGrid>().get(x, 50) as u32)
        .sum();

    // City with road and artificial traffic
    let mut city_busy = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local);
    {
        let world = city_busy.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        let mut traffic = world.resource_mut::<TrafficGrid>();
        for x in 50..=60 {
            traffic.set(x, 50, 20); // fully congested
        }
    }
    city_busy.tick_slow_cycle();

    let p_busy: u32 = (50..=60)
        .map(|x| city_busy.resource::<PollutionGrid>().get(x, 50) as u32)
        .sum();

    assert!(
        p_busy > p_empty,
        "Congested road ({p_busy}) should produce more pollution than empty road ({p_empty})"
    );
}

// ====================================================================
// Policy multiplier reduces industrial emissions
// ====================================================================

#[test]
fn test_building_emissions_air_filters_reduces_industrial() {
    // Without policy
    let mut city_no_policy = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 3);
    {
        let world = city_no_policy.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        world.resource_mut::<crate::stats::CityStats>().average_happiness = 50.0;
    }
    city_no_policy.tick_slow_cycle();
    let p_no_policy = city_no_policy.resource::<PollutionGrid>().get(50, 50);

    // With IndustrialAirFilters policy
    let mut city_with_policy = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 3);
    {
        let world = city_with_policy.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        world.resource_mut::<crate::stats::CityStats>().average_happiness = 50.0;
        world.resource_mut::<Policies>().toggle(Policy::IndustrialAirFilters);
    }
    city_with_policy.tick_slow_cycle();
    let p_with_policy = city_with_policy.resource::<PollutionGrid>().get(50, 50);

    assert!(
        p_with_policy < p_no_policy,
        "Air filters should reduce pollution: with={p_with_policy}, without={p_no_policy}"
    );
}

// ====================================================================
// Emission profile table validation
// ====================================================================

#[test]
fn test_building_emissions_profiles_cover_industrial() {
    for level in 1..=5u8 {
        let profile = building_emission_profile(ZoneType::Industrial, level);
        assert!(
            profile.is_some(),
            "Industrial L{level} should have an emission profile"
        );
        let p = profile.unwrap();
        assert!(p.base_q > 0.0, "Industrial L{level} base_q should be > 0");
    }
}

#[test]
fn test_building_emissions_clean_services() {
    assert!(service_emission_profile(ServiceType::SmallPark).is_none());
    assert!(service_emission_profile(ServiceType::Hospital).is_none());
    assert!(service_emission_profile(ServiceType::Library).is_none());
    assert!(service_emission_profile(ServiceType::GeothermalPlant).is_none());
}

// ====================================================================
// Park reduction still works after adding new sources
// ====================================================================

#[test]
fn test_building_emissions_park_reduces_nearby() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 3)
        .with_service(54, 50, ServiceType::SmallPark);
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        world.resource_mut::<crate::stats::CityStats>().average_happiness = 50.0;
    }

    city.tick_slow_cycle();

    let at_park = city.resource::<PollutionGrid>().get(54, 50);

    // Compare with city without park
    let mut city_no_park = TestCity::new()
        .with_building(50, 50, ZoneType::Industrial, 3);
    {
        let world = city_no_park.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        world.resource_mut::<crate::stats::CityStats>().average_happiness = 50.0;
    }
    city_no_park.tick_slow_cycle();
    let at_same_no_park = city_no_park.resource::<PollutionGrid>().get(54, 50);

    assert!(
        at_park < at_same_no_park,
        "Park should reduce pollution: with_park={at_park}, without={at_same_no_park}"
    );
}
