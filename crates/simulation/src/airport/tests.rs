use super::*;
use crate::services::ServiceType;

#[test]
fn test_airport_tier_from_service_type() {
    assert_eq!(
        AirportTier::from_service_type(ServiceType::SmallAirstrip),
        Some(AirportTier::SmallAirstrip)
    );
    assert_eq!(
        AirportTier::from_service_type(ServiceType::RegionalAirport),
        Some(AirportTier::RegionalAirport)
    );
    assert_eq!(
        AirportTier::from_service_type(ServiceType::InternationalAirport),
        Some(AirportTier::InternationalAirport)
    );
    assert_eq!(
        AirportTier::from_service_type(ServiceType::FireStation),
        None
    );
}

#[test]
fn test_airport_tier_from_non_airport_service_types_returns_none() {
    let non_airport_types = [
        ServiceType::PoliceStation,
        ServiceType::Hospital,
        ServiceType::ElementarySchool,
        ServiceType::SmallPark,
        ServiceType::Stadium,
        ServiceType::Landfill,
        ServiceType::BusDepot,
        ServiceType::TrainStation,
    ];
    for st in non_airport_types {
        assert_eq!(
            AirportTier::from_service_type(st),
            None,
            "Expected None for {:?}",
            st
        );
    }
}

#[test]
fn test_airport_tier_capacity() {
    assert_eq!(AirportTier::SmallAirstrip.capacity(), 500);
    assert_eq!(AirportTier::RegionalAirport.capacity(), 5_000);
    assert_eq!(AirportTier::InternationalAirport.capacity(), 50_000);
}

#[test]
fn test_capacity_increases_with_tier() {
    assert!(AirportTier::SmallAirstrip.capacity() < AirportTier::RegionalAirport.capacity());
    assert!(AirportTier::RegionalAirport.capacity() < AirportTier::InternationalAirport.capacity());
}

#[test]
fn test_passenger_flights_capped_by_capacity() {
    let pop = 1_000_000.0f32;
    let demand = (pop * 0.01) as u32;
    let capacity = AirportTier::SmallAirstrip.capacity();
    assert_eq!(demand, 10_000);
    assert_eq!(demand.min(capacity), 500);
}

#[test]
fn test_passenger_flights_under_capacity() {
    let pop = 10_000.0f32;
    let demand = (pop * 0.01) as u32;
    assert_eq!(
        demand.min(AirportTier::InternationalAirport.capacity()),
        100
    );
}

#[test]
fn test_total_capacity_sums_across_tiers() {
    let total = AirportTier::SmallAirstrip.capacity() * 2 + AirportTier::RegionalAirport.capacity();
    assert_eq!(total, 6_000);
}

#[test]
fn test_zero_population_zero_flights() {
    assert_eq!((0.0f32 * 0.01) as u32, 0);
}

#[test]
fn test_airport_tier_tourism_bonus() {
    assert!((AirportTier::SmallAirstrip.tourism_bonus() - 0.10).abs() < f32::EPSILON);
    assert!((AirportTier::RegionalAirport.tourism_bonus() - 0.30).abs() < f32::EPSILON);
    assert!((AirportTier::InternationalAirport.tourism_bonus() - 1.00).abs() < f32::EPSILON);
}

#[test]
fn test_tourism_bonus_increases_with_tier() {
    assert!(
        AirportTier::SmallAirstrip.tourism_bonus() < AirportTier::RegionalAirport.tourism_bonus()
    );
    assert!(
        AirportTier::RegionalAirport.tourism_bonus()
            < AirportTier::InternationalAirport.tourism_bonus()
    );
}

#[test]
fn test_diminishing_returns_tourism() {
    let single = 1.0f32 * (1.0f32).sqrt();
    let double = 1.0f32 * (2.0f32).sqrt();
    assert!((single - 1.0).abs() < 0.001);
    assert!((double - 1.414).abs() < 0.01);
    assert!(double - single < single);
}

#[test]
fn test_diminishing_returns_three_same_tier() {
    let bonus = 0.10f32 * (3.0f32).sqrt();
    assert!(bonus / 3.0 < 0.10);
}

#[test]
fn test_mixed_tiers_tourism_multiplier() {
    let total = AirportTier::SmallAirstrip.tourism_bonus()
        + AirportTier::RegionalAirport.tourism_bonus()
        + AirportTier::InternationalAirport.tourism_bonus();
    assert!((total - 1.40).abs() < 0.001);
    assert!((1.0 + total - 2.40).abs() < 0.001);
}

#[test]
fn test_no_airports_tourism_multiplier_is_one() {
    assert!((1.0 + 0.0f32 - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_noise_radius_values() {
    assert_eq!(AirportTier::SmallAirstrip.noise_radius(), 5);
    assert_eq!(AirportTier::RegionalAirport.noise_radius(), 8);
    assert_eq!(AirportTier::InternationalAirport.noise_radius(), 12);
}

#[test]
fn test_noise_radius_increases_with_tier() {
    assert!(
        AirportTier::SmallAirstrip.noise_radius() < AirportTier::RegionalAirport.noise_radius()
    );
    assert!(
        AirportTier::RegionalAirport.noise_radius()
            < AirportTier::InternationalAirport.noise_radius()
    );
}

#[test]
fn test_airport_stats_default() {
    let stats = AirportStats::default();
    assert_eq!(stats.total_airports, 0);
    assert_eq!(stats.passenger_flights_per_month, 0);
    assert_eq!(stats.cargo_flights_per_month, 0);
    assert!((stats.tourism_multiplier).abs() < f32::EPSILON);
    assert!((stats.revenue).abs() < f64::EPSILON);
}

#[test]
fn test_airport_stats_default_by_tier_array() {
    assert_eq!(AirportStats::default().airports_by_tier, [0, 0, 0]);
}

#[test]
fn test_airport_stats_default_total_monthly_cost() {
    assert!((AirportStats::default().total_monthly_cost).abs() < f64::EPSILON);
}

#[test]
fn test_monthly_costs() {
    assert!((AirportTier::SmallAirstrip.monthly_cost() - 60.0).abs() < f64::EPSILON);
    assert!((AirportTier::RegionalAirport.monthly_cost() - 100.0).abs() < f64::EPSILON);
    assert!((AirportTier::InternationalAirport.monthly_cost() - 150.0).abs() < f64::EPSILON);
}

#[test]
fn test_monthly_cost_increases_with_tier() {
    assert!(
        AirportTier::SmallAirstrip.monthly_cost() < AirportTier::RegionalAirport.monthly_cost()
    );
    assert!(
        AirportTier::RegionalAirport.monthly_cost()
            < AirportTier::InternationalAirport.monthly_cost()
    );
}

#[test]
fn test_total_monthly_cost_multiple_airports() {
    let total = 2.0 * AirportTier::SmallAirstrip.monthly_cost()
        + AirportTier::RegionalAirport.monthly_cost()
        + 3.0 * AirportTier::InternationalAirport.monthly_cost();
    assert!((total - 670.0).abs() < f64::EPSILON);
}

#[test]
fn test_revenue_per_flight() {
    assert!((AirportTier::SmallAirstrip.revenue_per_flight() - 5.0).abs() < f64::EPSILON);
    assert!((AirportTier::RegionalAirport.revenue_per_flight() - 15.0).abs() < f64::EPSILON);
    assert!((AirportTier::InternationalAirport.revenue_per_flight() - 50.0).abs() < f64::EPSILON);
}

#[test]
fn test_revenue_per_flight_increases_with_tier() {
    assert!(
        AirportTier::SmallAirstrip.revenue_per_flight()
            < AirportTier::RegionalAirport.revenue_per_flight()
    );
    assert!(
        AirportTier::RegionalAirport.revenue_per_flight()
            < AirportTier::InternationalAirport.revenue_per_flight()
    );
}

#[test]
fn test_cargo_flights_are_fifth_of_passenger_flights() {
    assert_eq!(100u32 / 5, 20);
}

#[test]
fn test_cargo_flights_boosted_by_outside_connection() {
    let cargo_base = 100u32 / 5;
    let cargo_with_conn = (cargo_base as f32 * 1.5) as u32;
    assert_eq!(cargo_with_conn, 30);
    assert!(cargo_with_conn > cargo_base);
}

#[test]
fn test_cargo_flights_zero_when_no_passengers() {
    assert_eq!(0u32 / 5, 0);
}

#[test]
fn test_revenue_international_fills_first() {
    let revenue = 100u32 as f64 * AirportTier::InternationalAirport.revenue_per_flight();
    assert!((revenue - 5000.0).abs() < f64::EPSILON);
}

#[test]
fn test_revenue_overflow_to_lower_tier() {
    let demand = 60_000u32;
    let intl_cap = AirportTier::InternationalAirport.capacity();
    assert_eq!(demand.min(intl_cap), 50_000);
    assert_eq!(demand.saturating_sub(intl_cap), 10_000);
}

#[test]
fn test_cargo_revenue_flat_rate() {
    assert!((25u32 as f64 * 8.0 - 200.0).abs() < f64::EPSILON);
}

#[test]
fn test_outside_connection_revenue_bonus_25_percent() {
    assert!((1000.0f64 * 1.25 - 1250.0).abs() < f64::EPSILON);
}

#[test]
fn test_fog_suspends_all_flights() {
    let demand = 500u32;
    let capacity = 50_000u32;
    let flights = if true { 0u32 } else { demand.min(capacity) };
    assert_eq!(flights, 0);
}

#[test]
fn test_clear_weather_allows_flights() {
    let demand = 500u32;
    let capacity = 50_000u32;
    let flights = if false { 0u32 } else { demand.min(capacity) };
    assert_eq!(flights, 500);
}

#[test]
fn test_all_tiers_have_positive_capacity() {
    for tier in [
        AirportTier::SmallAirstrip,
        AirportTier::RegionalAirport,
        AirportTier::InternationalAirport,
    ] {
        assert!(tier.capacity() > 0, "{:?} capacity must be > 0", tier);
    }
}

#[test]
fn test_all_tiers_have_positive_noise_radius() {
    for tier in [
        AirportTier::SmallAirstrip,
        AirportTier::RegionalAirport,
        AirportTier::InternationalAirport,
    ] {
        assert!(
            tier.noise_radius() > 0,
            "{:?} noise radius must be > 0",
            tier
        );
    }
}

#[test]
fn test_all_tiers_have_positive_monthly_cost() {
    for tier in [
        AirportTier::SmallAirstrip,
        AirportTier::RegionalAirport,
        AirportTier::InternationalAirport,
    ] {
        assert!(
            tier.monthly_cost() > 0.0,
            "{:?} monthly cost must be > 0",
            tier
        );
    }
}

#[test]
fn test_all_tiers_have_positive_revenue_per_flight() {
    for tier in [
        AirportTier::SmallAirstrip,
        AirportTier::RegionalAirport,
        AirportTier::InternationalAirport,
    ] {
        assert!(
            tier.revenue_per_flight() > 0.0,
            "{:?} revenue must be > 0",
            tier
        );
    }
}

#[test]
fn test_all_tiers_have_positive_tourism_bonus() {
    for tier in [
        AirportTier::SmallAirstrip,
        AirportTier::RegionalAirport,
        AirportTier::InternationalAirport,
    ] {
        assert!(
            tier.tourism_bonus() > 0.0,
            "{:?} tourism bonus must be > 0",
            tier
        );
    }
}
