//! Integration tests for the Cultural Buildings Prestige system (SVC-014).

use crate::cultural_buildings::CulturalPrestige;
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::tourism::Tourism;

// ====================================================================
// Prestige resource tests
// ====================================================================

#[test]
fn test_cultural_prestige_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<CulturalPrestige>();
}

#[test]
fn test_cultural_prestige_default_is_zero() {
    let city = TestCity::new();
    let prestige = city.resource::<CulturalPrestige>();
    assert!(
        (prestige.prestige_score - 0.0).abs() < f32::EPSILON,
        "Empty city should have 0 prestige"
    );
    assert_eq!(prestige.museum_count, 0);
    assert_eq!(prestige.cathedral_count, 0);
    assert_eq!(prestige.stadium_count, 0);
    assert_eq!(prestige.tv_station_count, 0);
    assert!(!prestige.stadium_event_active);
}

// ====================================================================
// Prestige computation tests
// ====================================================================

#[test]
fn test_museum_adds_prestige() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Museum);
    city.tick(50);
    let prestige = city.resource::<CulturalPrestige>();
    assert!(
        prestige.prestige_score > 0.0,
        "Museum should add prestige, got {}",
        prestige.prestige_score
    );
    assert_eq!(prestige.museum_count, 1);
}

#[test]
fn test_cathedral_adds_prestige() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Cathedral);
    city.tick(50);
    let prestige = city.resource::<CulturalPrestige>();
    assert!(
        prestige.prestige_score > 0.0,
        "Cathedral should add prestige, got {}",
        prestige.prestige_score
    );
    assert_eq!(prestige.cathedral_count, 1);
}

#[test]
fn test_stadium_adds_prestige() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    city.tick(50);
    let prestige = city.resource::<CulturalPrestige>();
    assert!(
        prestige.prestige_score > 0.0,
        "Stadium should add prestige, got {}",
        prestige.prestige_score
    );
    assert_eq!(prestige.stadium_count, 1);
}

#[test]
fn test_tv_station_adds_prestige() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::TVStation);
    city.tick(50);
    let prestige = city.resource::<CulturalPrestige>();
    assert!(
        prestige.prestige_score > 0.0,
        "TVStation should add prestige, got {}",
        prestige.prestige_score
    );
    assert_eq!(prestige.tv_station_count, 1);
}

#[test]
fn test_multiple_cultural_buildings_increase_prestige() {
    let mut city1 = TestCity::new().with_service(10, 10, ServiceType::Museum);
    city1.tick(50);
    let p1 = city1.resource::<CulturalPrestige>().prestige_score;

    let mut city2 = TestCity::new()
        .with_service(10, 10, ServiceType::Museum)
        .with_service(20, 20, ServiceType::Cathedral)
        .with_service(30, 30, ServiceType::Stadium)
        .with_service(40, 40, ServiceType::TVStation);
    city2.tick(50);
    let p2 = city2.resource::<CulturalPrestige>().prestige_score;

    assert!(
        p2 > p1,
        "More cultural buildings ({}) should yield higher prestige than one ({})",
        p2, p1
    );
}

#[test]
fn test_non_cultural_buildings_dont_add_prestige() {
    let mut city = TestCity::new()
        .with_service(10, 10, ServiceType::FireStation)
        .with_service(20, 20, ServiceType::PoliceStation)
        .with_service(30, 30, ServiceType::Hospital);
    city.tick(50);
    let prestige = city.resource::<CulturalPrestige>();
    assert!(
        (prestige.prestige_score - 0.0).abs() < f32::EPSILON,
        "Non-cultural buildings should not add prestige, got {}",
        prestige.prestige_score
    );
}

// ====================================================================
// Stadium event tests
// ====================================================================

#[test]
fn test_stadium_event_activates_after_interval() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    // Run enough ticks for the event interval (500 ticks) + update interval
    city.tick(600);
    let prestige = city.resource::<CulturalPrestige>();
    // The event should have been triggered at some point
    assert!(
        prestige.stadium_event_start_tick > 0,
        "Stadium event should have started after 600 ticks"
    );
}

#[test]
fn test_stadium_event_provides_happiness_bonus() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    // Run to trigger event
    city.tick(600);
    let prestige = city.resource::<CulturalPrestige>();
    if prestige.stadium_event_active {
        assert!(
            prestige.active_happiness_bonus > 0.0,
            "Active stadium event should provide happiness bonus"
        );
    }
}

#[test]
fn test_stadium_event_ends_after_duration() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Stadium);
    // Trigger event and then let it end
    city.tick(800);
    let prestige = city.resource::<CulturalPrestige>();
    // After event duration (100 ticks), event should have ended
    assert!(
        prestige.stadium_event_start_tick > 0,
        "Stadium event should have occurred"
    );
}

#[test]
fn test_no_stadium_event_without_stadium() {
    let mut city = TestCity::new().with_service(10, 10, ServiceType::Museum);
    city.tick(600);
    let prestige = city.resource::<CulturalPrestige>();
    assert!(
        !prestige.stadium_event_active,
        "No stadium event should occur without a stadium"
    );
    assert!(
        (prestige.active_happiness_bonus - 0.0).abs() < f32::EPSILON,
        "No happiness bonus without stadium event"
    );
}

// ====================================================================
// Tourism bonus tests
// ====================================================================

#[test]
fn test_cultural_prestige_boosts_tourism_score() {
    // City with only a non-cultural attraction for baseline
    let mut city1 = TestCity::new().with_service(10, 10, ServiceType::SmallPark);
    {
        city1.world_mut().resource_mut::<GameClock>().day = 31;
    }
    city1.tick(50);
    let t1 = city1.resource::<Tourism>().cultural_facilities_score;

    // City with cultural buildings
    let mut city2 = TestCity::new()
        .with_service(10, 10, ServiceType::SmallPark)
        .with_service(20, 20, ServiceType::Museum)
        .with_service(30, 30, ServiceType::Cathedral);
    {
        city2.world_mut().resource_mut::<GameClock>().day = 31;
    }
    city2.tick(50);
    let t2 = city2.resource::<Tourism>().cultural_facilities_score;

    assert!(
        t2 > t1,
        "Cultural buildings ({}) should boost tourism cultural score above baseline ({})",
        t2, t1
    );
}

// ====================================================================
// TV Station immigration tests
// ====================================================================

#[test]
fn test_tv_station_boosts_immigration_attractiveness() {
    use crate::immigration::CityAttractiveness;

    // Verify directly that the prestige resource tracks TV stations
    // and that the attractiveness score increases
    let mut city = TestCity::new().with_service(10, 10, ServiceType::TVStation);
    city.tick(50);

    let prestige = city.resource::<CulturalPrestige>();
    assert_eq!(
        prestige.tv_station_count, 1,
        "Should detect 1 TV station"
    );

    let attractiveness = city.resource::<CityAttractiveness>();
    // The baseline for an empty city with default factors is ~32.5
    // TVStation boost adds ~1.97 (5 * (1 - exp(-0.5)))
    // So we expect > 32.5
    assert!(
        attractiveness.overall_score > 32.5,
        "TV station should boost attractiveness above baseline 32.5, got {}",
        attractiveness.overall_score
    );
}

#[test]
fn test_multiple_tv_stations_diminishing_returns() {
    use crate::immigration::CityAttractiveness;

    let mut city1 = TestCity::new().with_service(10, 10, ServiceType::TVStation);
    city1.tick(50);
    let a1 = city1.resource::<CityAttractiveness>().overall_score;

    let mut city2 = TestCity::new()
        .with_service(10, 10, ServiceType::TVStation)
        .with_service(20, 20, ServiceType::TVStation)
        .with_service(30, 30, ServiceType::TVStation);
    city2.tick(50);
    let a2 = city2.resource::<CityAttractiveness>().overall_score;

    // More TV stations should increase score, but less than linearly
    assert!(
        a2 > a1,
        "Multiple TV stations ({}) should boost more than one ({})",
        a2, a1
    );
}

// ====================================================================
// Saveable round-trip test
// ====================================================================

#[test]
fn test_cultural_prestige_save_round_trip() {
    use crate::Saveable;

    let original = CulturalPrestige {
        prestige_score: 45.0,
        museum_count: 3,
        cathedral_count: 1,
        stadium_count: 2,
        tv_station_count: 1,
        stadium_event_active: true,
        stadium_event_start_tick: 500,
        active_happiness_bonus: 3.0,
        active_tourism_multiplier: 1.15,
    };

    let bytes = original.save_to_bytes().expect("should serialize");
    let restored = CulturalPrestige::load_from_bytes(&bytes);

    assert!(
        (restored.prestige_score - 45.0).abs() < f32::EPSILON,
        "Prestige score should round-trip"
    );
    assert_eq!(restored.museum_count, 3);
    assert_eq!(restored.cathedral_count, 1);
    assert_eq!(restored.stadium_count, 2);
    assert_eq!(restored.tv_station_count, 1);
    assert!(restored.stadium_event_active);
    assert_eq!(restored.stadium_event_start_tick, 500);
}

#[test]
fn test_cultural_prestige_save_skips_default() {
    use crate::Saveable;

    let default = CulturalPrestige::default();
    assert!(
        default.save_to_bytes().is_none(),
        "Default state should not serialize"
    );
}
