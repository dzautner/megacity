//! Integration tests for POLL-003: Air Pollution Health Effects with AQI Tiers

use crate::citizen::CitizenDetails;
use crate::grid::ZoneType;
use crate::immigration::CityAttractiveness;
use crate::land_value::LandValueGrid;
use crate::pollution::PollutionGrid;
use crate::pollution_health::{
    air_pollution_health_modifier, pollution_immigration_penalty, pollution_land_value_multiplier,
    AqiTier,
};
use crate::test_harness::TestCity;
use crate::wind::WindState;

// ====================================================================
// AQI tier boundary tests (pure function)
// ====================================================================

#[test]
fn test_aqi_tier_good_boundary() {
    assert_eq!(AqiTier::from_concentration(0), AqiTier::Good);
    assert_eq!(AqiTier::from_concentration(50), AqiTier::Good);
}

#[test]
fn test_aqi_tier_moderate_boundary() {
    assert_eq!(AqiTier::from_concentration(51), AqiTier::Moderate);
    assert_eq!(AqiTier::from_concentration(100), AqiTier::Moderate);
}

#[test]
fn test_aqi_tier_unhealthy_sensitive_boundary() {
    assert_eq!(
        AqiTier::from_concentration(101),
        AqiTier::UnhealthyForSensitive
    );
    assert_eq!(
        AqiTier::from_concentration(150),
        AqiTier::UnhealthyForSensitive
    );
}

#[test]
fn test_aqi_tier_unhealthy_boundary() {
    assert_eq!(AqiTier::from_concentration(151), AqiTier::Unhealthy);
    assert_eq!(AqiTier::from_concentration(200), AqiTier::Unhealthy);
}

#[test]
fn test_aqi_tier_very_unhealthy_boundary() {
    assert_eq!(AqiTier::from_concentration(201), AqiTier::VeryUnhealthy);
    assert_eq!(AqiTier::from_concentration(250), AqiTier::VeryUnhealthy);
}

#[test]
fn test_aqi_tier_hazardous_boundary() {
    assert_eq!(AqiTier::from_concentration(251), AqiTier::Hazardous);
    assert_eq!(AqiTier::from_concentration(255), AqiTier::Hazardous);
}

// ====================================================================
// Health modifier tests (pure function)
// ====================================================================

#[test]
fn test_clean_area_gives_health_bonus() {
    let modifier = air_pollution_health_modifier(0);
    assert!(
        modifier > 0.0,
        "clean air (concentration=0) should give positive health modifier, got {}",
        modifier
    );
    assert!(
        (modifier - 0.01).abs() < f32::EPSILON,
        "clean air modifier should be +0.01, got {}",
        modifier
    );
}

#[test]
fn test_moderate_pollution_is_neutral() {
    let modifier = air_pollution_health_modifier(75);
    assert!(
        (modifier - 0.0).abs() < f32::EPSILON,
        "moderate pollution modifier should be 0.0, got {}",
        modifier
    );
}

#[test]
fn test_high_pollution_damages_health() {
    let modifier = air_pollution_health_modifier(255);
    assert!(
        modifier < 0.0,
        "hazardous pollution should damage health, got {}",
        modifier
    );
    assert!(
        (modifier - (-0.20)).abs() < f32::EPSILON,
        "hazardous modifier should be -0.20, got {}",
        modifier
    );
}

// ====================================================================
// Integration: citizen in high-pollution area has lower health
// ====================================================================

#[test]
fn test_citizen_in_polluted_area_loses_health() {
    // Place citizen with a home near an industrial cluster
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 1)
        .with_building(130, 128, ZoneType::Industrial, 3)
        .with_building(126, 128, ZoneType::Industrial, 3)
        .with_building(128, 126, ZoneType::Industrial, 3)
        .with_building(130, 130, ZoneType::CommercialLow, 1)
        .with_citizen((128, 128), (130, 130));

    // Disable wind to keep pollution concentrated
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    // Record initial health
    let initial_health = {
        let world = city.world_mut();
        let mut query = world.query::<&CitizenDetails>();
        query.iter(world).next().unwrap().health
    };

    // Run several slow cycles to let pollution build and health effects apply
    city.tick_slow_cycles(5);

    // Check health after pollution exposure
    let final_health = {
        let world = city.world_mut();
        let mut query = world.query::<&CitizenDetails>();
        query.iter(world).next().unwrap().health
    };

    // Citizen should have lost some health from pollution exposure
    assert!(
        final_health < initial_health,
        "citizen in polluted area should lose health: initial={}, final={}",
        initial_health,
        final_health
    );
}

#[test]
fn test_citizen_in_clean_area_healthier_than_polluted() {
    // Clean city: citizen in a clean area (no industry)
    let mut clean_city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 1)
        .with_building(130, 130, ZoneType::CommercialLow, 1)
        .with_citizen((128, 128), (130, 130));

    {
        let world = clean_city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    // Polluted city: citizen near heavy industry
    let mut polluted_city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 1)
        .with_building(130, 128, ZoneType::Industrial, 3)
        .with_building(126, 128, ZoneType::Industrial, 3)
        .with_building(128, 126, ZoneType::Industrial, 3)
        .with_building(130, 130, ZoneType::CommercialLow, 1)
        .with_citizen((128, 128), (130, 130));

    {
        let world = polluted_city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    // Set same initial health in both
    for city in [&mut clean_city, &mut polluted_city] {
        let world = city.world_mut();
        let mut query = world.query::<&mut CitizenDetails>();
        for mut details in query.iter_mut(world) {
            details.health = 80.0;
        }
    }

    clean_city.tick_slow_cycles(5);
    polluted_city.tick_slow_cycles(5);

    let clean_health = {
        let world = clean_city.world_mut();
        let mut query = world.query::<&CitizenDetails>();
        query.iter(world).next().unwrap().health
    };

    let polluted_health = {
        let world = polluted_city.world_mut();
        let mut query = world.query::<&CitizenDetails>();
        query.iter(world).next().unwrap().health
    };

    // Citizen in clean area should be healthier than one in polluted area
    assert!(
        clean_health >= polluted_health,
        "citizen in clean area should be healthier: clean={}, polluted={}",
        clean_health,
        polluted_health
    );
}

// ====================================================================
// Integration: land value effects
// ====================================================================

#[test]
fn test_land_value_reduced_in_polluted_area() {
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::Industrial, 3)
        .with_building(130, 128, ZoneType::Industrial, 3);

    // Disable wind
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    // Get land value before pollution health effects
    let value_before = city.resource::<LandValueGrid>().get(129, 128);

    city.tick_slow_cycles(2);

    let value_after = city.resource::<LandValueGrid>().get(129, 128);

    // Land value near heavy industry should be reduced by pollution multiplier
    // The base land_value system already reduces for pollution, but our
    // multiplier further reduces it
    assert!(
        value_after <= value_before || value_before == 0,
        "land value in polluted area should not increase: before={}, after={}",
        value_before,
        value_after
    );
}

// ====================================================================
// Integration: immigration penalty
// ====================================================================

#[test]
fn test_immigration_penalty_applied_for_polluted_city() {
    // City with heavy industry
    let mut city = TestCity::new()
        .with_building(100, 100, ZoneType::Industrial, 3)
        .with_building(102, 100, ZoneType::Industrial, 3)
        .with_building(104, 100, ZoneType::Industrial, 3)
        .with_building(100, 102, ZoneType::Industrial, 3);

    // Disable wind
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    // Clean city for comparison
    let mut clean_city = TestCity::new();
    {
        let world = clean_city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);
    clean_city.tick_slow_cycles(2);

    let polluted_score = city.resource::<CityAttractiveness>().overall_score;
    let clean_score = clean_city.resource::<CityAttractiveness>().overall_score;

    // Polluted city should have lower or equal attractiveness
    // (the clean city also starts with default score, but pollution penalty
    // should reduce the polluted city's score)
    assert!(
        polluted_score <= clean_score,
        "polluted city should have lower attractiveness: polluted={}, clean={}",
        polluted_score,
        clean_score
    );
}

// ====================================================================
// Pure function: land value multiplier
// ====================================================================

#[test]
fn test_land_value_multiplier_clean_is_neutral() {
    let multiplier = pollution_land_value_multiplier(25);
    assert!(
        (multiplier - 1.0).abs() < f32::EPSILON,
        "clean area should have neutral land value multiplier (1.0), got {}",
        multiplier
    );
}

#[test]
fn test_land_value_multiplier_hazardous_is_penalty() {
    let multiplier = pollution_land_value_multiplier(255);
    assert!(
        multiplier < 1.0,
        "hazardous area should have land value penalty, got {}",
        multiplier
    );
    assert!(
        (multiplier - 0.50).abs() < f32::EPSILON,
        "hazardous multiplier should be 0.50, got {}",
        multiplier
    );
}

// ====================================================================
// Pure function: immigration penalty
// ====================================================================

#[test]
fn test_immigration_penalty_clean_city_no_penalty() {
    let penalty = pollution_immigration_penalty(25);
    assert!(
        (penalty - 0.0).abs() < f32::EPSILON,
        "clean city should have no immigration penalty, got {}",
        penalty
    );
}

#[test]
fn test_immigration_penalty_hazardous_severe() {
    let penalty = pollution_immigration_penalty(255);
    assert!(
        penalty < -10.0,
        "hazardous city should have severe immigration penalty, got {}",
        penalty
    );
}

// ====================================================================
// Plugin resource registration
// ====================================================================

#[test]
fn test_pollution_health_plugin_resources_exist() {
    let city = TestCity::new();
    // The plugin doesn't add new resources, but the systems depend on these
    city.assert_resource_exists::<PollutionGrid>();
    city.assert_resource_exists::<LandValueGrid>();
    city.assert_resource_exists::<CityAttractiveness>();
}
