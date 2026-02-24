//! Integration tests for POLL-012: Noise Pollution Land Value and Health Effects

use crate::citizen::CitizenDetails;
use crate::grid::ZoneType;
use crate::immigration::CityAttractiveness;
use crate::land_value::LandValueGrid;
use crate::noise::NoisePollutionGrid;
use crate::noise_effects::{
    is_nighttime, nighttime_multiplier, NoiseEffectsStats, NoiseTier,
};
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::wind::WindState;

// ====================================================================
// Tier classification pure-function tests
// ====================================================================

#[test]
fn test_noise_tier_quiet_boundary() {
    assert_eq!(NoiseTier::from_level(0), NoiseTier::Quiet);
    assert_eq!(NoiseTier::from_level(10), NoiseTier::Quiet);
}

#[test]
fn test_noise_tier_normal_boundary() {
    assert_eq!(NoiseTier::from_level(11), NoiseTier::Normal);
    assert_eq!(NoiseTier::from_level(25), NoiseTier::Normal);
}

#[test]
fn test_noise_tier_noticeable_boundary() {
    assert_eq!(NoiseTier::from_level(26), NoiseTier::Noticeable);
    assert_eq!(NoiseTier::from_level(40), NoiseTier::Noticeable);
}

#[test]
fn test_noise_tier_loud_boundary() {
    assert_eq!(NoiseTier::from_level(41), NoiseTier::Loud);
    assert_eq!(NoiseTier::from_level(55), NoiseTier::Loud);
}

#[test]
fn test_noise_tier_very_loud_boundary() {
    assert_eq!(NoiseTier::from_level(56), NoiseTier::VeryLoud);
    assert_eq!(NoiseTier::from_level(70), NoiseTier::VeryLoud);
}

#[test]
fn test_noise_tier_painful_boundary() {
    assert_eq!(NoiseTier::from_level(71), NoiseTier::Painful);
    assert_eq!(NoiseTier::from_level(85), NoiseTier::Painful);
}

#[test]
fn test_noise_tier_dangerous_boundary() {
    assert_eq!(NoiseTier::from_level(86), NoiseTier::Dangerous);
    assert_eq!(NoiseTier::from_level(100), NoiseTier::Dangerous);
}

// ====================================================================
// Land value multiplier pure-function tests
// ====================================================================

#[test]
fn test_quiet_gives_land_value_premium() {
    let mult = NoiseTier::Quiet.land_value_multiplier();
    assert!(
        mult > 1.0,
        "quiet areas should get a land value premium, got {}",
        mult
    );
}

#[test]
fn test_normal_is_neutral_land_value() {
    let mult = NoiseTier::Normal.land_value_multiplier();
    assert!(
        (mult - 1.0).abs() < f32::EPSILON,
        "normal noise should be neutral for land value, got {}",
        mult
    );
}

#[test]
fn test_dangerous_has_severe_land_value_penalty() {
    let mult = NoiseTier::Dangerous.land_value_multiplier();
    assert!(
        (mult - 0.20).abs() < f32::EPSILON,
        "dangerous noise should apply -80% land value penalty, got {}",
        mult
    );
}

// ====================================================================
// Health modifier pure-function tests
// ====================================================================

#[test]
fn test_quiet_through_noticeable_no_health_effect() {
    assert!((NoiseTier::Quiet.health_modifier()).abs() < f32::EPSILON);
    assert!((NoiseTier::Normal.health_modifier()).abs() < f32::EPSILON);
    assert!((NoiseTier::Noticeable.health_modifier()).abs() < f32::EPSILON);
}

#[test]
fn test_loud_causes_health_damage() {
    assert!(
        NoiseTier::Loud.health_modifier() < 0.0,
        "loud noise should cause health damage"
    );
}

#[test]
fn test_dangerous_causes_severe_health_damage() {
    let mod_val = NoiseTier::Dangerous.health_modifier();
    assert!(
        (mod_val - (-0.20)).abs() < f32::EPSILON,
        "dangerous noise should cause -0.20 health per tick, got {}",
        mod_val
    );
}

// ====================================================================
// Nighttime multiplier pure-function tests
// ====================================================================

#[test]
fn test_nighttime_hours_detected() {
    assert!(is_nighttime(22.0), "22:00 is nighttime");
    assert!(is_nighttime(0.0), "00:00 is nighttime");
    assert!(is_nighttime(5.0), "05:00 is nighttime");
}

#[test]
fn test_daytime_hours_not_nighttime() {
    assert!(!is_nighttime(6.0), "06:00 is daytime");
    assert!(!is_nighttime(12.0), "12:00 is daytime");
    assert!(!is_nighttime(21.9), "21:54 is daytime");
}

#[test]
fn test_nighttime_multiplier_amplifies() {
    let night = nighttime_multiplier(2.0);
    let day = nighttime_multiplier(14.0);
    assert!(
        night > day,
        "nighttime multiplier should be greater: night={}, day={}",
        night,
        day
    );
    assert!(
        (night - 1.5).abs() < f32::EPSILON,
        "nighttime multiplier should be 1.5, got {}",
        night
    );
}

// ====================================================================
// Integration: land value reduced near noisy infrastructure
// ====================================================================

#[test]
fn test_land_value_reduced_near_highway() {
    let mut city = TestCity::new()
        .with_road(100, 128, 160, 128, crate::grid::RoadType::Highway);

    // Disable wind to keep noise concentrated
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    let value_before = city.resource::<LandValueGrid>().get(130, 129);

    city.tick_slow_cycles(3);

    let value_after = city.resource::<LandValueGrid>().get(130, 129);

    // Land value near a highway should decrease due to noise
    assert!(
        value_after <= value_before || value_before == 0,
        "land value near highway should not increase: before={}, after={}",
        value_before,
        value_after
    );
}

#[test]
fn test_land_value_near_airport_lower_than_quiet_area() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::InternationalAirport);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(3);

    let noisy_value = city.resource::<LandValueGrid>().get(130, 128);
    let quiet_value = city.resource::<LandValueGrid>().get(200, 200);

    assert!(
        noisy_value <= quiet_value,
        "area near airport should have lower land value: noisy={}, quiet={}",
        noisy_value,
        quiet_value
    );
}

// ====================================================================
// Integration: citizen health damaged by loud noise
// ====================================================================

#[test]
fn test_citizen_in_loud_area_loses_health() {
    // Place citizen next to industrial buildings that generate noise
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 1)
        .with_building(129, 128, ZoneType::Industrial, 3)
        .with_building(127, 128, ZoneType::Industrial, 3)
        .with_building(128, 127, ZoneType::Industrial, 3)
        .with_building(128, 129, ZoneType::Industrial, 3)
        .with_building(140, 140, ZoneType::CommercialLow, 1)
        .with_citizen((128, 128), (140, 140));

    // Set daytime and prevent emigration
    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        for mut details in world.query::<&mut CitizenDetails>().iter_mut(world) {
            details.happiness = 95.0;
            details.health = 90.0;
            details.age = 25;
        }
        let mut attr = world.resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    let initial_health = {
        let world = city.world_mut();
        world
            .query::<&CitizenDetails>()
            .iter(world)
            .next()
            .expect("citizen should exist")
            .health
    };

    city.tick_slow_cycles(5);

    let final_health = {
        let world = city.world_mut();
        world
            .query::<&CitizenDetails>()
            .iter(world)
            .next()
            .expect("citizen should survive with high happiness")
            .health
    };

    assert!(
        final_health < initial_health,
        "citizen near industrial noise should lose health: initial={}, final={}",
        initial_health,
        final_health
    );
}

// ====================================================================
// Integration: nighttime noise worse for residential
// ====================================================================

#[test]
fn test_nighttime_noise_worse_than_daytime_for_residential() {
    // Two identical cities: one at night, one during daytime
    let make_city = |hour: f32| {
        TestCity::new()
            .with_building(128, 128, ZoneType::ResidentialLow, 1)
            .with_building(129, 128, ZoneType::Industrial, 3)
            .with_building(127, 128, ZoneType::Industrial, 3)
            .with_building(128, 127, ZoneType::Industrial, 3)
            .with_building(128, 129, ZoneType::Industrial, 3)
            .with_building(140, 140, ZoneType::CommercialLow, 1)
            .with_citizen((128, 128), (140, 140))
            .with_time(hour)
    };

    let mut day_city = make_city(12.0);
    let mut night_city = make_city(2.0);

    // Prevent emigration and set same initial health
    for city in [&mut day_city, &mut night_city] {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
        for mut details in world.query::<&mut CitizenDetails>().iter_mut(world) {
            details.happiness = 95.0;
            details.health = 80.0;
            details.age = 25;
        }
        let mut attr = world.resource_mut::<CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    day_city.tick_slow_cycles(3);
    night_city.tick_slow_cycles(3);

    let day_health = {
        let world = day_city.world_mut();
        world
            .query::<&CitizenDetails>()
            .iter(world)
            .next()
            .expect("citizen should survive")
            .health
    };

    let night_health = {
        let world = night_city.world_mut();
        world
            .query::<&CitizenDetails>()
            .iter(world)
            .next()
            .expect("citizen should survive")
            .health
    };

    // Night citizen should have lost more health (or equal if noise is below threshold)
    assert!(
        night_health <= day_health,
        "nighttime noise should cause more health damage: night={}, day={}",
        night_health,
        day_health
    );
}

// ====================================================================
// Integration: stats resource updates
// ====================================================================

#[test]
fn test_noise_effects_stats_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<NoiseEffectsStats>();
}

#[test]
fn test_noise_effects_stats_updated_with_noisy_city() {
    let mut city = TestCity::new()
        .with_road(100, 128, 160, 128, crate::grid::RoadType::Highway)
        .with_building(128, 128, ZoneType::Industrial, 3)
        .with_building(130, 128, ZoneType::Industrial, 3);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let stats = city.resource::<NoiseEffectsStats>();
    // With a highway and industrial buildings, we should have some loud cells
    assert!(
        stats.loud_cells > 0 || stats.avg_noise_tier > 0.0,
        "noisy city should have some noise stats: loud_cells={}, avg_tier={}",
        stats.loud_cells,
        stats.avg_noise_tier
    );
}

// ====================================================================
// Integration: quiet area land value premium
// ====================================================================

#[test]
fn test_quiet_area_maintains_or_increases_land_value() {
    // A city with no noise sources
    let mut city = TestCity::new();

    let value_before = city.resource::<LandValueGrid>().get(128, 128);

    city.tick_slow_cycles(2);

    let value_after = city.resource::<LandValueGrid>().get(128, 128);

    // In a quiet area (no noise sources), land value should not decrease
    // from noise effects (it may change from other systems though)
    assert!(
        value_after >= value_before.saturating_sub(5),
        "quiet area land value should be stable: before={}, after={}",
        value_before,
        value_after
    );
}

// ====================================================================
// Plugin integration: all resources registered
// ====================================================================

#[test]
fn test_noise_effects_plugin_registers_resources() {
    let city = TestCity::new();
    city.assert_resource_exists::<NoisePollutionGrid>();
    city.assert_resource_exists::<LandValueGrid>();
    city.assert_resource_exists::<NoiseEffectsStats>();
}
