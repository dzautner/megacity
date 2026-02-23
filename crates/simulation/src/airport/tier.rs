use serde::{Deserialize, Serialize};

use crate::services::ServiceType;

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
