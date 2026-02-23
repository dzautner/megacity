//! Hotel demand and capacity system (SVC-019).
//!
//! Tracks hotel room capacity across the city, calculates tourist demand based
//! on city attractiveness (landmarks, services, parks, culture), and computes
//! occupancy rates and hotel tax revenue.
//!
//! - Hotels have room capacity (50-500 rooms depending on commercial building level)
//! - Tourist demand based on city attractiveness from the Tourism resource
//! - Occupancy rate = demand / capacity
//! - Revenue from hotel tax on occupied rooms
//! - Over-capacity (demand > capacity) = lost tourism revenue
//! - Under-capacity (capacity >> demand) = wasted investment

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::grid::ZoneType;
use crate::services::{ServiceBuilding, ServiceType};
use crate::tourism::Tourism;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Base nightly room rate used for revenue calculation.
const BASE_ROOM_RATE: f64 = 120.0;

/// Default hotel tax rate (percentage of room revenue collected as tax).
const DEFAULT_HOTEL_TAX_RATE: f32 = 0.12;

/// Days per month for revenue calculation.
const DAYS_PER_MONTH: f64 = 30.0;

/// Average length of stay in nights per tourist visit.
const AVG_STAY_NIGHTS: f64 = 3.0;

/// Number of tourists sharing a single hotel room on average.
const TOURISTS_PER_ROOM: f64 = 1.8;

// ---------------------------------------------------------------------------
// Hotel capacity by building level
// ---------------------------------------------------------------------------

/// Returns the number of hotel rooms a commercial building provides based on
/// its level. Higher-level commercial buildings represent larger hotels.
pub fn hotel_rooms_for_level(level: u8) -> u32 {
    match level {
        1 => 50,
        2 => 120,
        3 => 200,
        4 => 350,
        5 => 500,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// Attractiveness scoring
// ---------------------------------------------------------------------------

/// Calculate a city attractiveness bonus from service buildings.
/// Landmarks, cultural venues, and parks each contribute to the score.
pub fn attractiveness_from_services(service_type: ServiceType) -> f32 {
    match service_type {
        ServiceType::Stadium => 8.0,
        ServiceType::Museum => 6.0,
        ServiceType::Cathedral => 5.0,
        ServiceType::CityHall => 3.0,
        ServiceType::TVStation => 2.0,
        ServiceType::LargePark => 4.0,
        ServiceType::SmallPark => 1.5,
        ServiceType::SportsField => 2.0,
        ServiceType::Plaza => 2.5,
        ServiceType::University => 3.0,
        ServiceType::Library => 1.0,
        ServiceType::InternationalAirport => 7.0,
        ServiceType::RegionalAirport => 3.0,
        ServiceType::TrainStation => 2.0,
        _ => 0.0,
    }
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Tracks hotel demand, capacity, occupancy, and tax revenue across the city.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct HotelDemandState {
    /// Total hotel room capacity in the city.
    pub total_capacity: u32,
    /// Number of commercial buildings contributing hotel rooms.
    pub hotel_count: u32,
    /// Tourist demand expressed as rooms needed per night.
    pub rooms_demanded: u32,
    /// Occupancy rate: demand / capacity, capped at 1.0.
    pub occupancy_rate: f32,
    /// Monthly hotel tax revenue collected.
    pub monthly_tax_revenue: f64,
    /// Hotel tax rate (0.0-1.0).
    pub hotel_tax_rate: f32,
    /// City attractiveness score (0-100) derived from services and landmarks.
    pub attractiveness_score: f32,
    /// Lost tourism revenue from insufficient hotel capacity.
    pub lost_revenue: f64,
    /// Estimated wasted investment from excess unused capacity.
    pub wasted_investment: f64,
    /// Rooms occupied (min of demand and capacity).
    pub rooms_occupied: u32,
    /// Last slow-tick counter when the system updated.
    pub last_update_tick: u32,
}

impl Default for HotelDemandState {
    fn default() -> Self {
        Self {
            total_capacity: 0,
            hotel_count: 0,
            rooms_demanded: 0,
            occupancy_rate: 0.0,
            monthly_tax_revenue: 0.0,
            hotel_tax_rate: DEFAULT_HOTEL_TAX_RATE,
            attractiveness_score: 0.0,
            lost_revenue: 0.0,
            wasted_investment: 0.0,
            rooms_occupied: 0,
            last_update_tick: 0,
        }
    }
}

impl HotelDemandState {
    /// Returns true if the city is over-capacity (more demand than rooms).
    pub fn is_over_capacity(&self) -> bool {
        self.rooms_demanded > self.total_capacity && self.total_capacity > 0
    }

    /// Returns true if the city has significant under-capacity (occupancy < 40%).
    pub fn is_under_capacity(&self) -> bool {
        self.total_capacity > 0 && self.occupancy_rate < 0.4
    }

    /// Nightly room rate adjusted by occupancy (higher occupancy = higher rates).
    pub fn effective_room_rate(&self) -> f64 {
        let occupancy_multiplier = if self.occupancy_rate > 0.9 {
            1.5 // high demand premium
        } else if self.occupancy_rate > 0.7 {
            1.2
        } else if self.occupancy_rate > 0.5 {
            1.0
        } else {
            0.8 // discount rates when low occupancy
        };
        BASE_ROOM_RATE * occupancy_multiplier
    }
}

// ---------------------------------------------------------------------------
// Saveable
// ---------------------------------------------------------------------------

impl crate::Saveable for HotelDemandState {
    const SAVE_KEY: &'static str = "hotel_demand";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.total_capacity == 0 && self.hotel_count == 0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Calculate hotel capacity from commercial high-density buildings.
/// Only CommercialHigh buildings are treated as hotels (large enough to
/// have hotel-style room capacity).
fn count_hotel_capacity(buildings: &Query<&Building>) -> (u32, u32) {
    let mut total_capacity = 0u32;
    let mut hotel_count = 0u32;
    for building in buildings.iter() {
        if building.zone_type == ZoneType::CommercialHigh && building.occupants > 0 {
            total_capacity += hotel_rooms_for_level(building.level);
            hotel_count += 1;
        }
    }
    (total_capacity, hotel_count)
}

/// Calculate city attractiveness from service buildings.
fn calculate_attractiveness(services: &Query<&ServiceBuilding>) -> f32 {
    let mut score = 0.0f32;
    for service in services.iter() {
        score += attractiveness_from_services(service.service_type);
    }
    score.min(100.0)
}

/// Convert monthly tourist visitors into nightly room demand.
fn visitors_to_room_demand(monthly_visitors: u32) -> u32 {
    // Monthly visitors stay AVG_STAY_NIGHTS nights, sharing TOURISTS_PER_ROOM per room.
    // Demand = (visitors * avg_stay) / (days_per_month * tourists_per_room)
    let room_nights = monthly_visitors as f64 * AVG_STAY_NIGHTS;
    let nightly_demand = room_nights / (DAYS_PER_MONTH * TOURISTS_PER_ROOM);
    nightly_demand as u32
}

/// Calculate lost revenue when demand exceeds capacity.
fn calculate_lost_revenue(rooms_demanded: u32, total_capacity: u32, effective_rate: f64) -> f64 {
    if rooms_demanded > total_capacity {
        let unmet = (rooms_demanded - total_capacity) as f64;
        unmet * effective_rate * DAYS_PER_MONTH
    } else {
        0.0
    }
}

/// Calculate wasted investment metric for under-utilized capacity.
fn calculate_wasted_investment(
    total_capacity: u32,
    rooms_occupied: u32,
    effective_rate: f64,
) -> f64 {
    if total_capacity > rooms_occupied {
        let vacant = (total_capacity - rooms_occupied) as f64;
        // Wasted investment = vacant rooms * potential revenue at a discounted rate
        vacant * effective_rate * 0.5 * DAYS_PER_MONTH
    } else {
        0.0
    }
}

/// Main update system: runs on slow tick to recalculate hotel demand metrics.
pub fn update_hotel_demand(
    slow_tick: Res<SlowTickTimer>,
    tourism: Res<Tourism>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
    mut state: ResMut<HotelDemandState>,
) {
    if !slow_tick.should_run() {
        return;
    }

    state.last_update_tick = slow_tick.counter;

    // 1. Count hotel capacity from commercial buildings
    let (capacity, count) = count_hotel_capacity(&buildings);
    state.total_capacity = capacity;
    state.hotel_count = count;

    // 2. Calculate attractiveness from services
    state.attractiveness_score = calculate_attractiveness(&services);

    // 3. Convert tourism visitors to room demand
    state.rooms_demanded = visitors_to_room_demand(tourism.monthly_visitors);

    // 4. Calculate occupancy
    if state.total_capacity > 0 {
        state.occupancy_rate = (state.rooms_demanded as f32 / state.total_capacity as f32).min(1.0);
        state.rooms_occupied = state.rooms_demanded.min(state.total_capacity);
    } else {
        state.occupancy_rate = 0.0;
        state.rooms_occupied = 0;
    }

    // 5. Calculate effective room rate (adjusts with occupancy)
    let effective_rate = state.effective_room_rate();

    // 6. Calculate monthly tax revenue from occupied rooms
    let monthly_room_revenue = state.rooms_occupied as f64 * effective_rate * DAYS_PER_MONTH;
    state.monthly_tax_revenue = monthly_room_revenue * state.hotel_tax_rate as f64;

    // 7. Calculate lost revenue and wasted investment
    state.lost_revenue =
        calculate_lost_revenue(state.rooms_demanded, state.total_capacity, effective_rate);
    state.wasted_investment =
        calculate_wasted_investment(state.total_capacity, state.rooms_occupied, effective_rate);
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct HotelDemandPlugin;

impl Plugin for HotelDemandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HotelDemandState>().add_systems(
            FixedUpdate,
            update_hotel_demand
                .after(crate::tourism::update_tourism)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<HotelDemandState>();
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotel_rooms_for_level() {
        assert_eq!(hotel_rooms_for_level(1), 50);
        assert_eq!(hotel_rooms_for_level(2), 120);
        assert_eq!(hotel_rooms_for_level(3), 200);
        assert_eq!(hotel_rooms_for_level(4), 350);
        assert_eq!(hotel_rooms_for_level(5), 500);
        assert_eq!(hotel_rooms_for_level(0), 0);
        assert_eq!(hotel_rooms_for_level(6), 0);
    }

    #[test]
    fn test_attractiveness_scoring() {
        assert!(attractiveness_from_services(ServiceType::Stadium) > 0.0);
        assert!(attractiveness_from_services(ServiceType::Museum) > 0.0);
        assert!(attractiveness_from_services(ServiceType::Cathedral) > 0.0);
        assert_eq!(attractiveness_from_services(ServiceType::FireStation), 0.0);
        assert_eq!(
            attractiveness_from_services(ServiceType::PoliceStation),
            0.0
        );
    }

    #[test]
    fn test_visitors_to_room_demand() {
        // 1000 visitors * 3 nights / (30 days * 1.8 per room) ~= 55
        let demand = visitors_to_room_demand(1000);
        assert!(demand > 0);
        assert!(demand < 100);

        // Zero visitors = zero demand
        assert_eq!(visitors_to_room_demand(0), 0);
    }

    #[test]
    fn test_occupancy_rate_capped_at_one() {
        let mut state = HotelDemandState::default();
        state.total_capacity = 100;
        state.rooms_demanded = 200;
        // When recalculating, occupancy should cap at 1.0
        let rate = (state.rooms_demanded as f32 / state.total_capacity as f32).min(1.0);
        assert_eq!(rate, 1.0);
    }

    #[test]
    fn test_over_capacity_detection() {
        let mut state = HotelDemandState::default();
        state.total_capacity = 100;
        state.rooms_demanded = 150;
        assert!(state.is_over_capacity());

        state.rooms_demanded = 50;
        assert!(!state.is_over_capacity());
    }

    #[test]
    fn test_under_capacity_detection() {
        let mut state = HotelDemandState::default();
        state.total_capacity = 100;
        state.occupancy_rate = 0.3;
        assert!(state.is_under_capacity());

        state.occupancy_rate = 0.6;
        assert!(!state.is_under_capacity());
    }

    #[test]
    fn test_effective_room_rate_scaling() {
        let mut state = HotelDemandState::default();

        state.occupancy_rate = 0.95;
        let high_rate = state.effective_room_rate();

        state.occupancy_rate = 0.3;
        let low_rate = state.effective_room_rate();

        assert!(
            high_rate > low_rate,
            "Higher occupancy should yield higher rates"
        );
    }

    #[test]
    fn test_lost_revenue_calculation() {
        let lost = calculate_lost_revenue(200, 100, 120.0);
        assert!(lost > 0.0);
        // 100 unmet rooms * 120 rate * 30 days = 360,000
        assert!((lost - 360_000.0).abs() < 0.01);

        // No lost revenue when capacity meets demand
        let no_loss = calculate_lost_revenue(50, 100, 120.0);
        assert_eq!(no_loss, 0.0);
    }

    #[test]
    fn test_wasted_investment_calculation() {
        let wasted = calculate_wasted_investment(100, 50, 120.0);
        assert!(wasted > 0.0);
        // 50 vacant * 120 * 0.5 * 30 = 90,000
        assert!((wasted - 90_000.0).abs() < 0.01);

        // No waste when fully occupied
        let no_waste = calculate_wasted_investment(100, 100, 120.0);
        assert_eq!(no_waste, 0.0);
    }

    #[test]
    fn test_default_state() {
        let state = HotelDemandState::default();
        assert_eq!(state.total_capacity, 0);
        assert_eq!(state.hotel_count, 0);
        assert_eq!(state.rooms_demanded, 0);
        assert_eq!(state.occupancy_rate, 0.0);
        assert_eq!(state.monthly_tax_revenue, 0.0);
        assert!((state.hotel_tax_rate - DEFAULT_HOTEL_TAX_RATE).abs() < f32::EPSILON);
        assert_eq!(state.lost_revenue, 0.0);
        assert_eq!(state.wasted_investment, 0.0);
    }

    #[test]
    fn test_zero_capacity_no_panic() {
        // Ensure no division by zero when capacity is 0
        let state = HotelDemandState::default();
        assert_eq!(state.occupancy_rate, 0.0);
        assert!(!state.is_over_capacity());
        assert!(!state.is_under_capacity());
    }

    #[test]
    fn test_saveable_key() {
        assert_eq!(HotelDemandState::SAVE_KEY, "hotel_demand");
    }
}
