use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::services::ServiceType;

/// Tourism tracking
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Tourism {
    pub attractiveness: f32, // 0-100 score
    pub monthly_visitors: u32,
    pub monthly_tourism_income: f64,
    pub last_update_day: u32,
    /// Multiplier from airport system (1.0 = no airports, >1.0 = airports boost tourism).
    pub airport_multiplier: f32,
}

impl Default for Tourism {
    fn default() -> Self {
        Self {
            attractiveness: 0.0,
            monthly_visitors: 0,
            monthly_tourism_income: 0.0,
            last_update_day: 0,
            airport_multiplier: 1.0,
        }
    }
}

impl Tourism {
    /// How many tourists a service type attracts per month
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
}

/// Tourism events that can occur based on weather conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TourismWeatherEvent {
    /// Good-weather festival: occurs on Sunny days in Spring/Summer.
    Festival,
    /// Weather closure: occurs during Storm or extreme conditions.
    Closure,
}
