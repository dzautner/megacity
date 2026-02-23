//! Tourism attraction formula (SVC-018).
//!
//! Computes a weighted attraction score from six city metrics:
//!
//! ```text
//! attraction = cultural_facilities * 0.3
//!            + natural_beauty     * 0.2
//!            + hotel_capacity     * 0.15
//!            + transport_access   * 0.15
//!            + safety             * 0.1
//!            + entertainment      * 0.1
//! ```
//!
//! Each component is normalized to 0–100 before weighting, so the final
//! attraction score is also 0–100.

use crate::services::ServiceType;

// ---------------------------------------------------------------------------
// Weights (must sum to 1.0)
// ---------------------------------------------------------------------------

const W_CULTURAL: f32 = 0.30;
const W_NATURE: f32 = 0.20;
const W_HOTEL: f32 = 0.15;
const W_TRANSPORT: f32 = 0.15;
const W_SAFETY: f32 = 0.10;
const W_ENTERTAINMENT: f32 = 0.10;

// ---------------------------------------------------------------------------
// Component scoring helpers
// ---------------------------------------------------------------------------

/// Score cultural facilities from service buildings.
///
/// Returns a raw score that is later normalized to 0–100.
/// Museums, cathedrals, libraries, and universities contribute.
pub fn cultural_facility_score(service_type: ServiceType) -> f32 {
    match service_type {
        ServiceType::Museum => 15.0,
        ServiceType::Cathedral => 12.0,
        ServiceType::University => 8.0,
        ServiceType::Library => 5.0,
        ServiceType::CityHall => 4.0,
        _ => 0.0,
    }
}

/// Score entertainment venues from service buildings.
///
/// Stadiums, sports fields, plazas, and TV stations contribute.
pub fn entertainment_score(service_type: ServiceType) -> f32 {
    match service_type {
        ServiceType::Stadium => 18.0,
        ServiceType::SportsField => 6.0,
        ServiceType::Plaza => 5.0,
        ServiceType::TVStation => 4.0,
        _ => 0.0,
    }
}

/// Score transport access from service buildings.
///
/// Airports, train stations, bus depots, and subway stations contribute.
pub fn transport_access_score(service_type: ServiceType) -> f32 {
    match service_type {
        ServiceType::InternationalAirport => 20.0,
        ServiceType::RegionalAirport => 10.0,
        ServiceType::TrainStation => 8.0,
        ServiceType::SubwayStation => 5.0,
        ServiceType::BusDepot => 4.0,
        ServiceType::TramDepot => 3.0,
        ServiceType::FerryPier => 6.0,
        _ => 0.0,
    }
}

/// Score natural beauty from parks and tree coverage.
///
/// `park_score` comes from service buildings; `tree_fraction` is the fraction
/// of grid cells covered by trees (0.0–1.0).
pub fn natural_beauty_score(service_type: ServiceType) -> f32 {
    match service_type {
        ServiceType::LargePark => 12.0,
        ServiceType::SmallPark => 5.0,
        ServiceType::Playground => 3.0,
        _ => 0.0,
    }
}

// ---------------------------------------------------------------------------
// Breakdown for UI / debugging
// ---------------------------------------------------------------------------

/// Intermediate breakdown of the six attraction formula components (0–100 each).
#[derive(Debug, Clone, Copy, Default)]
pub struct AttractionBreakdown {
    pub cultural_facilities: f32,
    pub natural_beauty: f32,
    pub hotel_capacity: f32,
    pub transport_access: f32,
    pub safety: f32,
    pub entertainment: f32,
}

impl AttractionBreakdown {
    /// Compute the final weighted attraction score (0–100).
    pub fn total(&self) -> f32 {
        let raw = self.cultural_facilities * W_CULTURAL
            + self.natural_beauty * W_NATURE
            + self.hotel_capacity * W_HOTEL
            + self.transport_access * W_TRANSPORT
            + self.safety * W_SAFETY
            + self.entertainment * W_ENTERTAINMENT;
        raw.clamp(0.0, 100.0)
    }
}

/// Normalize a raw score to 0–100 using a soft cap (diminishing returns).
///
/// `raw` is the accumulated score; `half_point` is the raw value that maps to
/// 50/100.  Uses `score = 100 * raw / (raw + half_point)`.
pub fn normalize_score(raw: f32, half_point: f32) -> f32 {
    if raw <= 0.0 || half_point <= 0.0 {
        return 0.0;
    }
    (100.0 * raw / (raw + half_point)).min(100.0)
}

// ---------------------------------------------------------------------------
// Tourist stay duration and spending
// ---------------------------------------------------------------------------

/// Average stay duration in days for tourists.
/// Higher attraction scores lead to longer stays (1–5 days).
pub fn average_stay_days(attraction_score: f32) -> f32 {
    // Linear interpolation: score 0 -> 1 day, score 100 -> 5 days
    (1.0 + attraction_score * 0.04).clamp(1.0, 5.0)
}

/// Daily spending per tourist at commercial businesses (in currency units).
///
/// Base spending scales with attraction score (better cities charge more).
pub fn daily_tourist_spending(attraction_score: f32) -> f64 {
    // Base: $50/day, scaling up to $150/day at max attraction
    let base = 50.0;
    let bonus = (attraction_score as f64 / 100.0) * 100.0;
    base + bonus
}

/// Calculate total monthly tourist spending that flows to commercial businesses.
///
/// `monthly_visitors` * `average_stay_days` * `daily_spending`.
pub fn monthly_tourist_commercial_spending(
    monthly_visitors: u32,
    attraction_score: f32,
) -> f64 {
    let stay = average_stay_days(attraction_score) as f64;
    let daily = daily_tourist_spending(attraction_score);
    monthly_visitors as f64 * stay * daily
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weights_sum_to_one() {
        let sum = W_CULTURAL + W_NATURE + W_HOTEL + W_TRANSPORT + W_SAFETY + W_ENTERTAINMENT;
        assert!(
            (sum - 1.0).abs() < f32::EPSILON,
            "Weights should sum to 1.0, got {}",
            sum
        );
    }

    #[test]
    fn test_cultural_scores_positive_for_cultural_types() {
        assert!(cultural_facility_score(ServiceType::Museum) > 0.0);
        assert!(cultural_facility_score(ServiceType::Cathedral) > 0.0);
        assert!(cultural_facility_score(ServiceType::University) > 0.0);
        assert!(cultural_facility_score(ServiceType::Library) > 0.0);
    }

    #[test]
    fn test_cultural_scores_zero_for_non_cultural() {
        assert_eq!(cultural_facility_score(ServiceType::FireStation), 0.0);
        assert_eq!(cultural_facility_score(ServiceType::Stadium), 0.0);
    }

    #[test]
    fn test_entertainment_scores() {
        assert!(entertainment_score(ServiceType::Stadium) > 0.0);
        assert!(entertainment_score(ServiceType::SportsField) > 0.0);
        assert_eq!(entertainment_score(ServiceType::Museum), 0.0);
    }

    #[test]
    fn test_transport_scores() {
        assert!(transport_access_score(ServiceType::InternationalAirport) > 0.0);
        assert!(transport_access_score(ServiceType::TrainStation) > 0.0);
        assert_eq!(transport_access_score(ServiceType::Museum), 0.0);
    }

    #[test]
    fn test_natural_beauty_scores() {
        assert!(natural_beauty_score(ServiceType::LargePark) > 0.0);
        assert!(natural_beauty_score(ServiceType::SmallPark) > 0.0);
        assert_eq!(natural_beauty_score(ServiceType::Stadium), 0.0);
    }

    #[test]
    fn test_normalize_score_zero_raw() {
        assert_eq!(normalize_score(0.0, 30.0), 0.0);
    }

    #[test]
    fn test_normalize_score_at_half_point() {
        let score = normalize_score(30.0, 30.0);
        assert!(
            (score - 50.0).abs() < 0.01,
            "At half_point, score should be 50, got {}",
            score
        );
    }

    #[test]
    fn test_normalize_score_high_raw() {
        let score = normalize_score(1000.0, 30.0);
        assert!(score > 95.0, "Very high raw should approach 100, got {}", score);
    }

    #[test]
    fn test_normalize_score_negative_raw() {
        assert_eq!(normalize_score(-5.0, 30.0), 0.0);
    }

    #[test]
    fn test_breakdown_total_all_zero() {
        let b = AttractionBreakdown::default();
        assert_eq!(b.total(), 0.0);
    }

    #[test]
    fn test_breakdown_total_all_max() {
        let b = AttractionBreakdown {
            cultural_facilities: 100.0,
            natural_beauty: 100.0,
            hotel_capacity: 100.0,
            transport_access: 100.0,
            safety: 100.0,
            entertainment: 100.0,
        };
        assert!(
            (b.total() - 100.0).abs() < 0.01,
            "All 100 should yield 100, got {}",
            b.total()
        );
    }

    #[test]
    fn test_breakdown_total_weighted() {
        let b = AttractionBreakdown {
            cultural_facilities: 50.0,
            natural_beauty: 50.0,
            hotel_capacity: 50.0,
            transport_access: 50.0,
            safety: 50.0,
            entertainment: 50.0,
        };
        assert!(
            (b.total() - 50.0).abs() < 0.01,
            "All 50 should yield 50, got {}",
            b.total()
        );
    }

    #[test]
    fn test_average_stay_days_low_attraction() {
        let days = average_stay_days(0.0);
        assert!((days - 1.0).abs() < 0.01, "Zero attraction = 1 day stay");
    }

    #[test]
    fn test_average_stay_days_high_attraction() {
        let days = average_stay_days(100.0);
        assert!((days - 5.0).abs() < 0.01, "Max attraction = 5 day stay");
    }

    #[test]
    fn test_average_stay_days_mid_attraction() {
        let days = average_stay_days(50.0);
        assert!((days - 3.0).abs() < 0.01, "Mid attraction = 3 day stay");
    }

    #[test]
    fn test_daily_tourist_spending_scales() {
        let low = daily_tourist_spending(0.0);
        let high = daily_tourist_spending(100.0);
        assert!(high > low, "Higher attraction should mean higher spending");
        assert!((low - 50.0).abs() < 0.01, "Base spending should be ~50");
        assert!(
            (high - 150.0).abs() < 0.01,
            "Max spending should be ~150, got {}",
            high
        );
    }

    #[test]
    fn test_monthly_commercial_spending_zero_visitors() {
        assert_eq!(monthly_tourist_commercial_spending(0, 50.0), 0.0);
    }

    #[test]
    fn test_monthly_commercial_spending_positive() {
        let spending = monthly_tourist_commercial_spending(100, 50.0);
        // 100 visitors * 3.0 days * $100/day = $30,000
        assert!(
            (spending - 30_000.0).abs() < 1.0,
            "Expected ~30000, got {}",
            spending
        );
    }
}
