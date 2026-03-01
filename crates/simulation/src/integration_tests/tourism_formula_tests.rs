//! Integration tests for the tourism attraction formula (SVC-018).
//!
//! Tests the weighted formula: cultural * 0.3 + nature * 0.2 + hotel * 0.15
//! + transport * 0.15 + safety * 0.1 + entertainment * 0.1

use crate::grid::ZoneType;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::tourism::Tourism;
use crate::weather::{Season, Weather, WeatherCondition};

/// Advance the game clock past the 30-day tourism update threshold and tick.
fn trigger_tourism_update(city: &mut TestCity) {
    city.world_mut().resource_mut::<GameClock>().day = 31;
    city.tick(1);
}

// ====================================================================
// Resource and default state
// ====================================================================

#[test]
fn test_tourism_formula_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<Tourism>();
}

#[test]
fn test_tourism_formula_default_component_scores() {
    let city = TestCity::new();
    let t = city.resource::<Tourism>();
    assert!((t.cultural_facilities_score - 0.0).abs() < f32::EPSILON);
    assert!((t.natural_beauty_score - 0.0).abs() < f32::EPSILON);
    assert!((t.hotel_capacity_score - 0.0).abs() < f32::EPSILON);
    assert!((t.transport_access_score - 0.0).abs() < f32::EPSILON);
    assert!((t.safety_score - 0.0).abs() < f32::EPSILON);
    assert!((t.entertainment_score - 0.0).abs() < f32::EPSILON);
}

// ====================================================================
// Cultural facilities component
// ====================================================================

#[test]
fn test_tourism_formula_museum_boosts_cultural_score() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Museum);
    trigger_tourism_update(&mut city);
    let t = city.resource::<Tourism>();
    assert!(
        t.cultural_facilities_score > 0.0,
        "Museum should boost cultural score, got {}",
        t.cultural_facilities_score
    );
}

#[test]
fn test_tourism_formula_more_cultural_facilities_higher_score() {
    let mut city1 = TestCity::new().with_service(10, 10, ServiceType::Museum);
    trigger_tourism_update(&mut city1);
    let score1 = city1.resource::<Tourism>().cultural_facilities_score;

    let mut city2 = TestCity::new()
        .with_service(10, 10, ServiceType::Museum)
        .with_service(20, 20, ServiceType::Cathedral)
        .with_service(30, 30, ServiceType::Library);
    trigger_tourism_update(&mut city2);
    let score2 = city2.resource::<Tourism>().cultural_facilities_score;

    assert!(
        score2 > score1,
        "More cultural facilities should yield higher score: {} vs {}",
        score2,
        score1
    );
}

// ====================================================================
// Entertainment component
// ====================================================================

#[test]
fn test_tourism_formula_stadium_boosts_entertainment() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    trigger_tourism_update(&mut city);
    let t = city.resource::<Tourism>();
    assert!(
        t.entertainment_score > 0.0,
        "Stadium should boost entertainment, got {}",
        t.entertainment_score
    );
}

// ====================================================================
// Transport access component
// ====================================================================

#[test]
fn test_tourism_formula_train_boosts_transport() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::TrainStation);
    trigger_tourism_update(&mut city);
    let t = city.resource::<Tourism>();
    assert!(
        t.transport_access_score > 0.0,
        "Train station should boost transport access, got {}",
        t.transport_access_score
    );
}

// ====================================================================
// Natural beauty component
// ====================================================================

#[test]
fn test_tourism_formula_parks_boost_nature() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::LargePark);
    trigger_tourism_update(&mut city);
    let t = city.resource::<Tourism>();
    assert!(
        t.natural_beauty_score > 0.0,
        "Large park should boost natural beauty, got {}",
        t.natural_beauty_score
    );
}

// ====================================================================
// Safety component
// ====================================================================

#[test]
fn test_tourism_formula_default_safety_is_high() {
    // With no crime, safety should be high (close to 100)
    let mut city = TestCity::new();
    trigger_tourism_update(&mut city);
    let t = city.resource::<Tourism>();
    assert!(
        t.safety_score > 90.0,
        "No-crime city should have high safety score, got {}",
        t.safety_score
    );
}

// ====================================================================
// Hotel capacity component
// ====================================================================

#[test]
fn test_tourism_formula_hotels_boost_capacity_score() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::CommercialHigh, 1);
    // Need to tick slow cycles to allow hotel_demand to compute capacity first
    trigger_tourism_update(&mut city);
    // The hotel_demand system also needs to run; tick slow cycles
    city.tick_slow_cycles(2);
    // Re-trigger tourism update
    city.world_mut().resource_mut::<GameClock>().day = 62;
    city.tick(1);
    let t = city.resource::<Tourism>();
    assert!(
        t.hotel_capacity_score > 0.0,
        "Commercial high building should contribute hotel capacity score, got {}",
        t.hotel_capacity_score
    );
}

// ====================================================================
// Weighted formula integration
// ====================================================================

#[test]
fn test_tourism_formula_attractiveness_from_weighted_components() {
    let mut city = TestCity::new()
        .with_service(10, 10, ServiceType::Museum)
        .with_service(20, 20, ServiceType::Stadium)
        .with_service(30, 30, ServiceType::LargePark)
        .with_service(40, 40, ServiceType::TrainStation);
    trigger_tourism_update(&mut city);
    let t = city.resource::<Tourism>();
    assert!(
        t.attractiveness > 0.0,
        "City with diverse attractions should have positive attractiveness"
    );
    // The weighted formula should produce a reasonable score
    assert!(
        t.attractiveness <= 100.0,
        "Attractiveness should cap at 100, got {}",
        t.attractiveness
    );
}

// ====================================================================
// Tourist stay duration
// ====================================================================

#[test]
fn test_tourism_formula_stay_duration_scales_with_attraction() {
    // Low-attraction city
    let mut city_low = TestCity::new();
    trigger_tourism_update(&mut city_low);
    let stay_low = city_low.resource::<Tourism>().average_stay_days;

    // High-attraction city
    let mut city_high = TestCity::new()
        .with_service(10, 10, ServiceType::Stadium)
        .with_service(20, 20, ServiceType::Museum)
        .with_service(30, 30, ServiceType::Cathedral)
        .with_service(40, 40, ServiceType::LargePark)
        .with_service(50, 50, ServiceType::TrainStation);
    trigger_tourism_update(&mut city_high);
    let stay_high = city_high.resource::<Tourism>().average_stay_days;

    assert!(
        stay_high >= stay_low,
        "Higher attraction should yield longer stays: {} vs {}",
        stay_high,
        stay_low
    );
    assert!(
        stay_low >= 1.0,
        "Minimum stay should be 1 day, got {}",
        stay_low
    );
    assert!(
        stay_high <= 5.0,
        "Maximum stay should be 5 days, got {}",
        stay_high
    );
}

// ====================================================================
// Commercial spending
// ====================================================================

#[test]
fn test_tourism_formula_commercial_spending_with_visitors() {
    let mut city = TestCity::new()
        .with_service(10, 10, ServiceType::Stadium)
        .with_service(20, 20, ServiceType::Museum);
    trigger_tourism_update(&mut city);
    let t = city.resource::<Tourism>();
    if t.monthly_visitors > 0 {
        assert!(
            t.commercial_spending > 0.0,
            "Tourists should generate commercial spending"
        );
    }
}

#[test]
fn test_tourism_formula_no_visitors_no_spending() {
    let city = TestCity::new();
    let t = city.resource::<Tourism>();
    assert!(
        (t.commercial_spending - 0.0).abs() < f64::EPSILON,
        "No visitors should mean no commercial spending"
    );
}

// ====================================================================
// Seasonal variation
// ====================================================================

#[test]
fn test_tourism_formula_summer_boosts_visitors() {
    let mut city_summer = TestCity::new()
        .with_service(10, 10, ServiceType::Stadium)
        .with_service(20, 20, ServiceType::Museum);
    {
        let w = city_summer.world_mut();
        let mut wt = w.resource_mut::<Weather>();
        wt.season = Season::Summer;
        wt.current_event = WeatherCondition::Sunny;
        wt.temperature = 25.0;
    }
    trigger_tourism_update(&mut city_summer);
    let summer_visitors = city_summer.resource::<Tourism>().monthly_visitors;

    let mut city_winter = TestCity::new()
        .with_service(10, 10, ServiceType::Stadium)
        .with_service(20, 20, ServiceType::Museum);
    {
        let w = city_winter.world_mut();
        let mut wt = w.resource_mut::<Weather>();
        wt.season = Season::Winter;
        wt.current_event = WeatherCondition::Snow;
        wt.temperature = 2.0;
    }
    trigger_tourism_update(&mut city_winter);
    let winter_visitors = city_winter.resource::<Tourism>().monthly_visitors;

    assert!(
        summer_visitors >= winter_visitors,
        "Summer ({}) should have at least as many visitors as winter ({})",
        summer_visitors,
        winter_visitors
    );
}

// ====================================================================
// Diverse city test (all components contributing)
// ====================================================================

#[test]
fn test_tourism_formula_diverse_city_all_components() {
    let mut city = TestCity::new()
        // Cultural
        .with_service(10, 10, ServiceType::Museum)
        .with_service(12, 12, ServiceType::Cathedral)
        // Entertainment
        .with_service(20, 20, ServiceType::Stadium)
        .with_service(22, 22, ServiceType::SportsField)
        // Transport
        .with_service(30, 30, ServiceType::TrainStation)
        // Nature
        .with_service(40, 40, ServiceType::LargePark)
        .with_service(42, 42, ServiceType::SmallPark)
        // Hotel capacity
        .with_building(50, 50, ZoneType::CommercialHigh, 2);
    trigger_tourism_update(&mut city);
    let t = city.resource::<Tourism>();

    assert!(t.cultural_facilities_score > 0.0, "Cultural should be positive");
    assert!(t.entertainment_score > 0.0, "Entertainment should be positive");
    assert!(t.transport_access_score > 0.0, "Transport should be positive");
    assert!(t.natural_beauty_score > 0.0, "Nature should be positive");
    assert!(t.safety_score > 0.0, "Safety should be positive");
    assert!(t.attractiveness > 0.0, "Overall attractiveness should be positive");
    assert!(t.monthly_visitors > 0, "Should have visitors");
    assert!(t.average_stay_days >= 1.0, "Stay should be at least 1 day");
}
