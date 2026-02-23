use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::services::ServiceType;

use super::attraction_formula::AttractionBreakdown;

/// Tourism tracking resource with the weighted attraction formula (SVC-018).
///
/// The attraction score is computed from six weighted components:
/// cultural_facilities * 0.3 + natural_beauty * 0.2 + hotel_capacity * 0.15
/// + transport_access * 0.15 + safety * 0.1 + entertainment * 0.1
#[derive(Resource, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Tourism {
    /// Overall attraction score (0–100), computed from weighted components.
    pub attractiveness: f32,
    /// Monthly tourist arrivals.
    pub monthly_visitors: u32,
    /// Monthly tourism income flowing to the city economy.
    pub monthly_tourism_income: f64,
    /// Last game day when monthly update occurred.
    pub last_update_day: u32,
    /// Multiplier from airport system (1.0 = no airports, >1.0 = airports boost).
    pub airport_multiplier: f32,
    /// Average tourist stay duration in days (1–5), derived from attraction score.
    pub average_stay_days: f32,
    /// Monthly tourist spending at commercial businesses (boosts commercial income).
    pub commercial_spending: f64,
    /// Cultural facilities score component (0–100).
    pub cultural_facilities_score: f32,
    /// Natural beauty score component (0–100).
    pub natural_beauty_score: f32,
    /// Hotel capacity score component (0–100).
    pub hotel_capacity_score: f32,
    /// Transport access score component (0–100).
    pub transport_access_score: f32,
    /// Safety score component (0–100).
    pub safety_score: f32,
    /// Entertainment score component (0–100).
    pub entertainment_score: f32,
}

impl Default for Tourism {
    fn default() -> Self {
        Self {
            attractiveness: 0.0,
            monthly_visitors: 0,
            monthly_tourism_income: 0.0,
            last_update_day: 0,
            airport_multiplier: 1.0,
            average_stay_days: 1.0,
            commercial_spending: 0.0,
            cultural_facilities_score: 0.0,
            natural_beauty_score: 0.0,
            hotel_capacity_score: 0.0,
            transport_access_score: 0.0,
            safety_score: 0.0,
            entertainment_score: 0.0,
        }
    }
}

impl Tourism {
    /// How many tourists a service type attracts per month (used for base visitor calc).
    pub(crate) fn tourism_draw(service_type: ServiceType) -> u32 {
        match service_type {
            ServiceType::Stadium => 500,
            ServiceType::Museum => 300,
            ServiceType::Cathedral => 200,
            ServiceType::CityHall => 100,
            ServiceType::TVStation => 150,
            ServiceType::LargePark => 100,
            ServiceType::SportsField => 50,
            ServiceType::Plaza => 80,
            _ => 0,
        }
    }

    /// Build an `AttractionBreakdown` from the current component scores.
    pub fn breakdown(&self) -> AttractionBreakdown {
        AttractionBreakdown {
            cultural_facilities: self.cultural_facilities_score,
            natural_beauty: self.natural_beauty_score,
            hotel_capacity: self.hotel_capacity_score,
            transport_access: self.transport_access_score,
            safety: self.safety_score,
            entertainment: self.entertainment_score,
        }
    }
}

/// Tourism events that can occur based on weather conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TourismWeatherEvent {
    /// Good-weather festival: occurs on Sunny days in Spring/Summer.
    Festival,
    /// Weather closure: occurs during Storm or extreme conditions.
    Closure,
}

// ---------------------------------------------------------------------------
// Saveable
// ---------------------------------------------------------------------------

impl crate::Saveable for Tourism {
    const SAVE_KEY: &'static str = "tourism";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.monthly_visitors == 0 && self.attractiveness == 0.0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
