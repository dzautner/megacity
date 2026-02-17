use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::services::{ServiceBuilding, ServiceType};
use crate::stats::CityStats;

/// Tourism tracking
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tourism {
    pub attractiveness: f32,      // 0-100 score
    pub monthly_visitors: u32,
    pub monthly_tourism_income: f64,
    pub last_update_day: u32,
}

impl Tourism {
    /// How many tourists a service type attracts per month
    fn tourism_draw(service_type: ServiceType) -> u32 {
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

pub fn update_tourism(
    clock: Res<crate::time_of_day::GameClock>,
    mut tourism: ResMut<Tourism>,
    services: Query<&ServiceBuilding>,
    stats: Res<CityStats>,
) {
    // Update monthly
    if clock.day <= tourism.last_update_day + 30 {
        return;
    }
    tourism.last_update_day = clock.day;

    // Calculate attractiveness from landmarks and entertainment
    let mut total_draw = 0u32;
    for service in &services {
        total_draw += Tourism::tourism_draw(service.service_type);
    }

    // Attractiveness scales with city size and landmarks
    let pop_factor = (stats.population as f32 / 10000.0).min(5.0);
    tourism.attractiveness = (total_draw as f32 * 0.1 + pop_factor * 10.0).min(100.0);

    // Visitors based on attractiveness
    tourism.monthly_visitors = (tourism.attractiveness * 50.0) as u32;

    // Tourism income: visitors spend money at commercial buildings
    tourism.monthly_tourism_income = tourism.monthly_visitors as f64 * 2.0;
}
