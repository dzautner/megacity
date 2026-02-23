use bevy::prelude::*;

use super::stats::AirportStats;
use super::tier::AirportTier;
use crate::fog::FogState;
use crate::outside_connections::OutsideConnections;
use crate::services::ServiceBuilding;
use crate::stats::CityStats;
use crate::tourism::Tourism;
use crate::SlowTickTimer;

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
    let mut tourism_mult = 0.0f32;
    let tiers = [
        (AirportTier::SmallAirstrip, by_tier[0]),
        (AirportTier::RegionalAirport, by_tier[1]),
        (AirportTier::InternationalAirport, by_tier[2]),
    ];
    for (tier, count) in &tiers {
        if *count > 0 {
            let base = tier.tourism_bonus();
            let effective = base * (*count as f32).sqrt();
            tourism_mult += effective;
        }
    }

    // -------------------------------------------------------------------------
    // 3. Calculate passenger and cargo flights
    // -------------------------------------------------------------------------
    let pop = stats.population as f32;

    let mut total_capacity: u32 = 0;
    for (tier, count) in &tiers {
        total_capacity += tier.capacity() * count;
    }
    let demand = (pop * 0.01) as u32;
    let passenger_flights = if fog.flights_suspended {
        0
    } else {
        demand.min(total_capacity)
    };

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
    let mut remaining_passengers = passenger_flights;
    for &(tier, count) in tiers.iter().rev() {
        if count == 0 || remaining_passengers == 0 {
            continue;
        }
        let tier_capacity = tier.capacity() * count;
        let served = remaining_passengers.min(tier_capacity);
        revenue += served as f64 * tier.revenue_per_flight();
        remaining_passengers = remaining_passengers.saturating_sub(served);
    }
    revenue += cargo_flights as f64 * 8.0;

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

pub struct AirportPlugin;

impl Plugin for AirportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AirportStats>().add_systems(
            FixedUpdate,
            update_airports
                .after(crate::tourism::update_tourism)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
