use crate::services::ServiceType;
use crate::test_harness::TestCity;

// ====================================================================
// Airport system integration tests (TEST-069)
// ====================================================================

#[test]
fn test_airport_stats_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::airport::AirportStats>();
}

#[test]
fn test_airport_no_airports_stats_zero() {
    let mut city = TestCity::new();
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.total_airports, 0);
    assert_eq!(stats.airports_by_tier, [0, 0, 0]);
    assert_eq!(stats.passenger_flights_per_month, 0);
    assert_eq!(stats.cargo_flights_per_month, 0);
    assert!((stats.revenue).abs() < f64::EPSILON);
    assert!((stats.total_monthly_cost).abs() < f64::EPSILON);
}

#[test]
fn test_airport_no_airports_tourism_multiplier_one() {
    let mut city = TestCity::new();
    city.tick_slow_cycles(2);
    let tourism = city.resource::<crate::tourism::Tourism>();
    assert!(
        (tourism.airport_multiplier - 1.0).abs() < 0.01,
        "Without airports, airport_multiplier should be 1.0, got {}",
        tourism.airport_multiplier
    );
}

#[test]
fn test_airport_single_small_airstrip_counted() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::SmallAirstrip);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.total_airports, 1);
    assert_eq!(stats.airports_by_tier[0], 1);
    assert_eq!(stats.airports_by_tier[1], 0);
    assert_eq!(stats.airports_by_tier[2], 0);
}

#[test]
fn test_airport_single_regional_counted() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::RegionalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.total_airports, 1);
    assert_eq!(stats.airports_by_tier[1], 1);
}

#[test]
fn test_airport_single_international_counted() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.total_airports, 1);
    assert_eq!(stats.airports_by_tier[2], 1);
}

#[test]
fn test_airport_multiple_tiers_counted() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::SmallAirstrip)
        .with_service(60, 60, ServiceType::SmallAirstrip)
        .with_service(90, 90, ServiceType::RegionalAirport)
        .with_service(120, 120, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.total_airports, 4);
    assert_eq!(stats.airports_by_tier, [2, 1, 1]);
}

#[test]
fn test_airport_non_airport_services_not_counted() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::SmallAirstrip)
        .with_service(60, 60, ServiceType::FireStation)
        .with_service(70, 70, ServiceType::PoliceStation)
        .with_service(80, 80, ServiceType::Hospital);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.total_airports, 1);
}

#[test]
fn test_airport_tourism_multiplier_single_international() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    // 1 intl: bonus = 1.0 * sqrt(1) = 1.0, multiplier = 2.0
    assert!(
        (stats.tourism_multiplier - 2.0).abs() < 0.01,
        "got {}",
        stats.tourism_multiplier
    );
}

#[test]
fn test_airport_tourism_multiplier_applied_to_tourism_resource() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let tourism = city.resource::<crate::tourism::Tourism>();
    assert!(
        (tourism.airport_multiplier - 2.0).abs() < 0.01,
        "got {}",
        tourism.airport_multiplier
    );
}

#[test]
fn test_airport_tourism_multiplier_mixed_tiers() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::SmallAirstrip)
        .with_service(60, 60, ServiceType::RegionalAirport)
        .with_service(90, 90, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    // 0.10 + 0.30 + 1.00 = 1.40, multiplier = 2.40
    assert!(
        (stats.tourism_multiplier - 2.40).abs() < 0.01,
        "got {}",
        stats.tourism_multiplier
    );
}

#[test]
fn test_airport_tourism_diminishing_returns_two_international() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::InternationalAirport)
        .with_service(80, 80, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    // 2 intl: bonus = 1.0 * sqrt(2) ~ 1.414, multiplier ~ 2.414, NOT 3.0
    assert!(
        stats.tourism_multiplier < 3.0,
        "should have diminishing returns, got {}",
        stats.tourism_multiplier
    );
    assert!(
        (stats.tourism_multiplier - 2.414).abs() < 0.01,
        "got {}",
        stats.tourism_multiplier
    );
}

#[test]
fn test_airport_monthly_cost_single_small_airstrip() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::SmallAirstrip);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert!((stats.total_monthly_cost - 60.0).abs() < f64::EPSILON);
}

#[test]
fn test_airport_monthly_cost_multiple_airports() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::SmallAirstrip)
        .with_service(60, 60, ServiceType::RegionalAirport)
        .with_service(90, 90, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    // 60 + 100 + 150 = 310
    assert!((stats.total_monthly_cost - 310.0).abs() < f64::EPSILON);
}

#[test]
fn test_airport_zero_population_zero_revenue() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(stats.passenger_flights_per_month, 0);
    assert!((stats.revenue).abs() < f64::EPSILON);
}

#[test]
fn test_airport_fog_suspends_flights() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    {
        let world = city.world_mut();
        world
            .resource_mut::<crate::fog::FogState>()
            .flights_suspended = true;
        world
            .resource_mut::<crate::virtual_population::VirtualPopulation>()
            .total_virtual = 100_000;
    }
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(
        stats.passenger_flights_per_month, 0,
        "fog should suspend flights"
    );
    assert_eq!(stats.cargo_flights_per_month, 0);
}

#[test]
fn test_airport_clear_weather_allows_flights() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    {
        let world = city.world_mut();
        world
            .resource_mut::<crate::virtual_population::VirtualPopulation>()
            .total_virtual = 100_000;
        world
            .resource_mut::<crate::fog::FogState>()
            .flights_suspended = false;
    }
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert!(
        stats.passenger_flights_per_month > 0,
        "expected flights > 0, got {}",
        stats.passenger_flights_per_month
    );
}

#[test]
fn test_airport_capacity_limits_flights() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::SmallAirstrip);
    {
        let world = city.world_mut();
        world
            .resource_mut::<crate::virtual_population::VirtualPopulation>()
            .total_virtual = 1_000_000;
    }
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert_eq!(
        stats.passenger_flights_per_month, 500,
        "should be capped at 500, got {}",
        stats.passenger_flights_per_month
    );
}

#[test]
fn test_airport_revenue_positive_with_population() {
    let mut city = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    {
        let world = city.world_mut();
        world
            .resource_mut::<crate::virtual_population::VirtualPopulation>()
            .total_virtual = 50_000;
    }
    city.tick_slow_cycles(2);
    let stats = city.resource::<crate::airport::AirportStats>();
    assert!(
        stats.revenue > 0.0,
        "expected revenue > 0, got {}",
        stats.revenue
    );
}

#[test]
fn test_airport_noise_generated_around_small_airstrip() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::SmallAirstrip);
    city.tick_slow_cycles(2);
    let noise = city.resource::<crate::noise::NoisePollutionGrid>();
    let center = noise.get(128, 128);
    let far = noise.get(140, 128);
    assert!(center > 0, "center noise should be > 0, got {}", center);
    assert!(
        far < center,
        "far ({}) should be < center ({})",
        far,
        center
    );
}

#[test]
fn test_airport_noise_international_larger_radius() {
    let mut city = TestCity::new().with_service(128, 128, ServiceType::InternationalAirport);
    city.tick_slow_cycles(2);
    let noise = city.resource::<crate::noise::NoisePollutionGrid>();
    assert!(noise.get(128, 128) > 0, "center noise should be > 0");
    assert!(
        noise.get(135, 128) > 0,
        "noise at 7 cells should be > 0 for international"
    );
}

#[test]
fn test_airport_noise_international_more_intense_than_small() {
    let mut city_small = TestCity::new().with_service(128, 128, ServiceType::SmallAirstrip);
    city_small.tick_slow_cycles(2);
    let noise_small = city_small
        .resource::<crate::noise::NoisePollutionGrid>()
        .get(128, 128);

    let mut city_intl = TestCity::new().with_service(128, 128, ServiceType::InternationalAirport);
    city_intl.tick_slow_cycles(2);
    let noise_intl = city_intl
        .resource::<crate::noise::NoisePollutionGrid>()
        .get(128, 128);

    assert!(
        noise_intl > noise_small,
        "international ({}) should be louder than small ({})",
        noise_intl,
        noise_small
    );
}

#[test]
fn test_airport_economic_bonus_international_higher_revenue_than_small() {
    let mut city_small = TestCity::new().with_service(50, 50, ServiceType::SmallAirstrip);
    {
        let world = city_small.world_mut();
        world
            .resource_mut::<crate::virtual_population::VirtualPopulation>()
            .total_virtual = 10_000;
    }
    city_small.tick_slow_cycles(2);
    let rev_small = city_small
        .resource::<crate::airport::AirportStats>()
        .revenue;

    let mut city_intl = TestCity::new().with_service(50, 50, ServiceType::InternationalAirport);
    {
        let world = city_intl.world_mut();
        world
            .resource_mut::<crate::virtual_population::VirtualPopulation>()
            .total_virtual = 10_000;
    }
    city_intl.tick_slow_cycles(2);
    let rev_intl = city_intl.resource::<crate::airport::AirportStats>().revenue;

    assert!(
        rev_intl > rev_small,
        "intl revenue ({}) should exceed small ({})",
        rev_intl,
        rev_small
    );
}
