//! Tourism update system using the weighted attraction formula (SVC-018).

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::crime::CrimeGrid;
use crate::grid::ZoneType;
use crate::hotel_demand::HotelDemandState;
use crate::services::ServiceBuilding;
use crate::stats::CityStats;
use crate::trees::TreeGrid;
use crate::weather::Weather;

use super::attraction_formula::{
    average_stay_days, cultural_facility_score, entertainment_score,
    monthly_tourist_commercial_spending, natural_beauty_score, normalize_score,
    transport_access_score,
};
use super::{tourism_seasonal_modifier, Tourism};

/// Half-point constants for the normalized score curves.
/// These tune how quickly each component saturates.
const CULTURAL_HALF: f32 = 30.0;
const NATURE_HALF: f32 = 25.0;
const HOTEL_HALF: f32 = 200.0; // hotel rooms
const TRANSPORT_HALF: f32 = 20.0;
const ENTERTAINMENT_HALF: f32 = 25.0;

/// Main tourism update system. Runs once per ~30 game days.
///
/// Reads service buildings, crime grid, hotel capacity, tree coverage, and
/// weather to compute the six-component attraction formula and derive tourist
/// arrivals, stay duration, and commercial spending.
#[allow(clippy::too_many_arguments)]
pub fn update_tourism(
    clock: Res<crate::time_of_day::GameClock>,
    mut tourism: ResMut<Tourism>,
    services: Query<&ServiceBuilding>,
    stats: Res<CityStats>,
    weather: Res<Weather>,
    crime_grid: Res<CrimeGrid>,
    trees: Res<TreeGrid>,
    hotel_state: Res<HotelDemandState>,
) {
    // Update monthly
    if clock.day <= tourism.last_update_day + 30 {
        return;
    }
    tourism.last_update_day = clock.day;

    // ---------------------------------------------------------------
    // 1. Accumulate raw scores from service buildings
    // ---------------------------------------------------------------
    let mut raw_cultural = 0.0f32;
    let mut raw_entertainment = 0.0f32;
    let mut raw_transport = 0.0f32;
    let mut raw_nature_services = 0.0f32;
    let mut total_draw = 0u32;

    for service in &services {
        let st = service.service_type;
        raw_cultural += cultural_facility_score(st);
        raw_entertainment += entertainment_score(st);
        raw_transport += transport_access_score(st);
        raw_nature_services += natural_beauty_score(st);
        total_draw += Tourism::tourism_draw(st);
    }

    // ---------------------------------------------------------------
    // 2. Normalize each component to 0–100
    // ---------------------------------------------------------------

    // Cultural facilities
    tourism.cultural_facilities_score = normalize_score(raw_cultural, CULTURAL_HALF);

    // Entertainment
    tourism.entertainment_score = normalize_score(raw_entertainment, ENTERTAINMENT_HALF);

    // Transport access (also boosted by airport multiplier)
    let transport_raw = raw_transport * tourism.airport_multiplier;
    tourism.transport_access_score = normalize_score(transport_raw, TRANSPORT_HALF);

    // Natural beauty: parks + tree coverage fraction
    let total_cells = (GRID_WIDTH * GRID_HEIGHT) as f32;
    let tree_count = trees.cells.iter().filter(|&&t| t).count() as f32;
    let tree_fraction = tree_count / total_cells;
    // Tree coverage adds up to 30 raw points at 15% coverage
    let tree_bonus = (tree_fraction / 0.15).min(1.0) * 30.0;
    let raw_nature = raw_nature_services + tree_bonus;
    tourism.natural_beauty_score = normalize_score(raw_nature, NATURE_HALF);

    // Hotel capacity score (from HotelDemandState)
    tourism.hotel_capacity_score =
        normalize_score(hotel_state.total_capacity as f32, HOTEL_HALF);

    // Safety score: inverse of average crime level
    let total_crime: u64 = crime_grid.levels.iter().map(|&c| c as u64).sum();
    let avg_crime = total_crime as f32 / total_cells;
    // Max crime level per cell is 255; lower crime = higher safety
    let safety_raw = 100.0 - (avg_crime / 255.0 * 100.0);
    tourism.safety_score = safety_raw.clamp(0.0, 100.0);

    // ---------------------------------------------------------------
    // 3. Compute weighted attraction score
    // ---------------------------------------------------------------
    let breakdown = tourism.breakdown();
    let attraction = breakdown.total();
    tourism.attractiveness = attraction;

    // ---------------------------------------------------------------
    // 4. Calculate visitor arrivals (backward-compatible path)
    // ---------------------------------------------------------------
    // Use the legacy draw-based formula scaled by the new attraction score
    // so existing economy integration keeps working.
    let pop_factor = (stats.population as f32 / 10000.0).min(5.0);
    let base_attractiveness = (total_draw as f32 * 0.1 + pop_factor * 10.0).min(100.0);
    // Blend: use the formula-based attraction but keep population influence
    let effective_attractiveness = (attraction * 0.7 + base_attractiveness * 0.3).min(100.0);

    let base_visitors = (effective_attractiveness * 50.0) as u32;
    let season_weather_modifier = tourism_seasonal_modifier(weather.season, &weather);
    tourism.monthly_visitors =
        (base_visitors as f32 * tourism.airport_multiplier * season_weather_modifier) as u32;

    // ---------------------------------------------------------------
    // 5. Stay duration and spending
    // ---------------------------------------------------------------
    tourism.average_stay_days = average_stay_days(attraction);
    tourism.commercial_spending =
        monthly_tourist_commercial_spending(tourism.monthly_visitors, attraction);

    // Legacy income field (used by economy.rs): visitor-count-based spending
    // Now uses the richer per-visitor spending from the attraction formula
    let spending_per_visitor = 2.0 * tourism.airport_multiplier as f64;
    tourism.monthly_tourism_income = tourism.monthly_visitors as f64 * spending_per_visitor
        + tourism.commercial_spending * 0.1; // 10% of commercial spending as city tax
}
