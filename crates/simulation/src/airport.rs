use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::fog::FogState;
use crate::outside_connections::OutsideConnections;
use crate::services::{ServiceBuilding, ServiceType};
use crate::stats::CityStats;
use crate::tourism::Tourism;
use crate::SlowTickTimer;

// =============================================================================
// Airport Tier
// =============================================================================

/// Tier classification for airport buildings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AirportTier {
    SmallAirstrip,
    RegionalAirport,
    InternationalAirport,
}

impl AirportTier {
    /// Maximum passenger capacity per month.
    pub fn capacity(self) -> u32 {
        match self {
            AirportTier::SmallAirstrip => 500,
            AirportTier::RegionalAirport => 5_000,
            AirportTier::InternationalAirport => 50_000,
        }
    }

    /// Tourism bonus multiplier added by this tier.
    pub fn tourism_bonus(self) -> f32 {
        match self {
            AirportTier::SmallAirstrip => 0.10,
            AirportTier::RegionalAirport => 0.30,
            AirportTier::InternationalAirport => 1.00,
        }
    }

    /// Noise pollution radius in grid cells.
    pub fn noise_radius(self) -> i32 {
        match self {
            AirportTier::SmallAirstrip => 5,
            AirportTier::RegionalAirport => 8,
            AirportTier::InternationalAirport => 12,
        }
    }

    /// Monthly operating cost.
    pub fn monthly_cost(self) -> f64 {
        match self {
            AirportTier::SmallAirstrip => 60.0,
            AirportTier::RegionalAirport => 100.0,
            AirportTier::InternationalAirport => 150.0,
        }
    }

    /// Derive tier from ServiceType.
    pub fn from_service_type(st: ServiceType) -> Option<AirportTier> {
        match st {
            ServiceType::SmallAirstrip => Some(AirportTier::SmallAirstrip),
            ServiceType::RegionalAirport => Some(AirportTier::RegionalAirport),
            ServiceType::InternationalAirport => Some(AirportTier::InternationalAirport),
            _ => None,
        }
    }

    /// Revenue per passenger flight.
    pub fn revenue_per_flight(self) -> f64 {
        match self {
            AirportTier::SmallAirstrip => 5.0,
            AirportTier::RegionalAirport => 15.0,
            AirportTier::InternationalAirport => 50.0,
        }
    }
}

// =============================================================================
// Airport Stats Resource
// =============================================================================

/// City-wide airport statistics, updated every slow tick.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct AirportStats {
    /// Total number of airport buildings across all tiers.
    pub total_airports: u32,
    /// Number of airports per tier: [SmallAirstrip, RegionalAirport, InternationalAirport].
    pub airports_by_tier: [u32; 3],
    /// Estimated passenger flights per month.
    pub passenger_flights_per_month: u32,
    /// Estimated cargo flights per month (driven by trade connections).
    pub cargo_flights_per_month: u32,
    /// Aggregate tourism multiplier from all airports (diminishing returns).
    pub tourism_multiplier: f32,
    /// Monthly revenue from airport operations (passenger + cargo).
    pub revenue: f64,
    /// Total monthly operating costs.
    pub total_monthly_cost: f64,
}

// =============================================================================
// System
// =============================================================================

/// Update airport statistics every slow tick (100 ticks).
///
/// - Counts airport buildings from `ServiceBuilding` query
/// - Calculates tourism multiplier with diminishing returns per additional airport
/// - Generates flight revenue based on population and airport tier
/// - Updates the Tourism resource with the airport multiplier
pub fn update_airports(
    slow_timer: Res<SlowTickTimer>,
    mut airport_stats: ResMut<AirportStats>,
    services: Query<&ServiceBuilding>,
    stats: Res<CityStats>,
    mut tourism: ResMut<Tourism>,
    outside: Res<OutsideConnections>,
    fog: Res<FogState>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // -------------------------------------------------------------------------
    // 1. Count airports by tier
    // -------------------------------------------------------------------------
    let mut by_tier = [0u32; 3];
    for service in &services {
        match AirportTier::from_service_type(service.service_type) {
            Some(AirportTier::SmallAirstrip) => by_tier[0] += 1,
            Some(AirportTier::RegionalAirport) => by_tier[1] += 1,
            Some(AirportTier::InternationalAirport) => by_tier[2] += 1,
            None => {}
        }
    }

    let total = by_tier[0] + by_tier[1] + by_tier[2];

    // -------------------------------------------------------------------------
    // 2. Tourism multiplier with diminishing returns
    // -------------------------------------------------------------------------
    // Each airport adds its tourism_bonus, but stacking the same tier
    // gives diminishing returns: bonus * (1 / sqrt(count_of_that_tier)).
    let mut tourism_mult = 0.0f32;
    let tiers = [
        (AirportTier::SmallAirstrip, by_tier[0]),
        (AirportTier::RegionalAirport, by_tier[1]),
        (AirportTier::InternationalAirport, by_tier[2]),
    ];
    for (tier, count) in &tiers {
        if *count > 0 {
            let base = tier.tourism_bonus();
            // Diminishing returns: first airport full bonus, each additional sqrt-scaled
            let effective = base * (*count as f32).sqrt();
            tourism_mult += effective;
        }
    }

    // -------------------------------------------------------------------------
    // 3. Calculate passenger and cargo flights
    // -------------------------------------------------------------------------
    let pop = stats.population as f32;

    // Passenger flights scale with population and airport capacity
    let mut total_capacity: u32 = 0;
    for (tier, count) in &tiers {
        total_capacity += tier.capacity() * count;
    }
    // Demand is a fraction of population (1% of pop wants to fly per month)
    let demand = (pop * 0.01) as u32;
    // Dense fog suspends all flight operations
    let passenger_flights = if fog.flights_suspended {
        0
    } else {
        demand.min(total_capacity)
    };

    // Cargo flights are a fraction of passenger flights, boosted by outside connections
    let has_airport_connection =
        outside.has_connection(crate::outside_connections::ConnectionType::Airport);
    let cargo_base = passenger_flights / 5;
    let cargo_flights = if has_airport_connection {
        (cargo_base as f32 * 1.5) as u32
    } else {
        cargo_base
    };

    // -------------------------------------------------------------------------
    // 4. Revenue calculation
    // -------------------------------------------------------------------------
    let mut revenue = 0.0f64;
    // Passenger revenue distributed by tier
    let mut remaining_passengers = passenger_flights;
    // Fill international first (highest revenue), then regional, then small
    for &(tier, count) in tiers.iter().rev() {
        if count == 0 || remaining_passengers == 0 {
            continue;
        }
        let tier_capacity = tier.capacity() * count;
        let served = remaining_passengers.min(tier_capacity);
        revenue += served as f64 * tier.revenue_per_flight();
        remaining_passengers = remaining_passengers.saturating_sub(served);
    }
    // Cargo revenue (flat rate per cargo flight)
    revenue += cargo_flights as f64 * 8.0;

    // Outside connection bonus: if airport connection active, +25% revenue
    if has_airport_connection {
        revenue *= 1.25;
    }

    // -------------------------------------------------------------------------
    // 5. Monthly costs
    // -------------------------------------------------------------------------
    let mut total_cost = 0.0f64;
    for (tier, count) in &tiers {
        total_cost += tier.monthly_cost() * *count as f64;
    }

    // -------------------------------------------------------------------------
    // 6. Apply tourism multiplier to Tourism resource
    // -------------------------------------------------------------------------
    tourism.airport_multiplier = 1.0 + tourism_mult;

    // -------------------------------------------------------------------------
    // 7. Update stats resource
    // -------------------------------------------------------------------------
    airport_stats.total_airports = total;
    airport_stats.airports_by_tier = by_tier;
    airport_stats.passenger_flights_per_month = passenger_flights;
    airport_stats.cargo_flights_per_month = cargo_flights;
    airport_stats.tourism_multiplier = 1.0 + tourism_mult;
    airport_stats.revenue = revenue;
    airport_stats.total_monthly_cost = total_cost;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_airport_tier_capacity() {
        assert_eq!(AirportTier::SmallAirstrip.capacity(), 500);
        assert_eq!(AirportTier::RegionalAirport.capacity(), 5_000);
        assert_eq!(AirportTier::InternationalAirport.capacity(), 50_000);
    }

    #[test]
    fn test_airport_tier_tourism_bonus() {
        assert!((AirportTier::SmallAirstrip.tourism_bonus() - 0.10).abs() < f32::EPSILON);
        assert!((AirportTier::RegionalAirport.tourism_bonus() - 0.30).abs() < f32::EPSILON);
        assert!((AirportTier::InternationalAirport.tourism_bonus() - 1.00).abs() < f32::EPSILON);
    }

    #[test]
    fn test_diminishing_returns_tourism() {
        // Single international airport: tourism_bonus = 1.0 * sqrt(1) = 1.0
        // Two international airports: tourism_bonus = 1.0 * sqrt(2) ~ 1.414
        // This means second airport adds only ~0.414 instead of 1.0
        let single = 1.0f32 * (1.0f32).sqrt();
        let double = 1.0f32 * (2.0f32).sqrt();
        assert!((single - 1.0).abs() < 0.001);
        assert!((double - 1.414).abs() < 0.01);
        // Second airport contribution is less than first
        assert!(double - single < single);
    }

    #[test]
    fn test_airport_stats_default() {
        let stats = AirportStats::default();
        assert_eq!(stats.total_airports, 0);
        assert_eq!(stats.passenger_flights_per_month, 0);
        assert_eq!(stats.cargo_flights_per_month, 0);
        assert!((stats.tourism_multiplier - 0.0).abs() < f32::EPSILON);
        assert!((stats.revenue - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_monthly_costs() {
        assert!((AirportTier::SmallAirstrip.monthly_cost() - 60.0).abs() < f64::EPSILON);
        assert!((AirportTier::RegionalAirport.monthly_cost() - 100.0).abs() < f64::EPSILON);
        assert!((AirportTier::InternationalAirport.monthly_cost() - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_revenue_per_flight() {
        assert!((AirportTier::SmallAirstrip.revenue_per_flight() - 5.0).abs() < f64::EPSILON);
        assert!((AirportTier::RegionalAirport.revenue_per_flight() - 15.0).abs() < f64::EPSILON);
        assert!(
            (AirportTier::InternationalAirport.revenue_per_flight() - 50.0).abs() < f64::EPSILON
        );
    }
}

pub struct AirportPlugin;

impl Plugin for AirportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AirportStats>().add_systems(
            FixedUpdate,
            update_airports.after(crate::tourism::update_tourism),
        );
    }
}
