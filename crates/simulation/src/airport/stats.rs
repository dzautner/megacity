use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
