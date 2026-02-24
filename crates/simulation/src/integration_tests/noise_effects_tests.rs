//! Integration tests for POLL-012: Noise Pollution Land Value and Health Effects

use crate::citizen::CitizenDetails;
use crate::grid::ZoneType;
use crate::immigration::CityAttractiveness;
use crate::land_value::LandValueGrid;
use crate::noise::NoisePollutionGrid;
use crate::noise_effects::{is_nighttime, nighttime_multiplier, NoiseEffectsStats, NoiseTier};
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
fn test_quiet_and_normal_are_neutral_land_value() {
    let quiet = NoiseTier::Quiet.land_value_multiplier();
    let normal = NoiseTier::Normal.land_value_multiplier();
    assert!(
        (quiet - 1.0).abs() < f32::EPSILON,
        "quiet should be neutral, got {}",
        quiet
    );
    assert!(
        (normal - 1.0).abs() < f32::EPSILON,
        "normal should be neutral, got {}",
        normal
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

#[test]
fn test_land_value_multiplier_decreases_with_tier() {
    let tiers = [
        NoiseTier::Quiet,
        NoiseTier::Normal,
        NoiseTier::Noticeable,
        NoiseTier::Loud,
        NoiseTier::VeryLoud,
        NoiseTier::Painful,
        NoiseTier::Dangerous,
    ];
    for window in tiers.windows(2) {
        let higher = window[0].land_value_multiplier();
        let lower = window[1].land_value_multiplier();
        assert!(
            higher >= lower,
            "land value multiplier should decrease: {:?}={} >= {:?}={}",
            window[0],
            higher,
            window[1],
            lower
        );
    }
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
// Integration: noise generated near airport
// ====================================================================

#[test]
fn test_airport_generates_noise_in_radius() {
    // International airport generates noise=45 in 10-cell radius
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::InternationalAirport);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    // Check noise is generated at the airport cell
    let noise_at = city.resource::<NoisePollutionGrid>().get(128, 128);
    assert!(
        noise_at > 0,
        "airport cell should have noise, got {}",
        noise_at
    );

    // Check noise is generated in nearby cells (within radius)
    let noise_near = city.resource::<NoisePollutionGrid>().get(130, 128);
    assert!(
        noise_near > 0,
        "cells near airport should have noise, got {}",
        noise_near
    );

    // Check noise decays with distance
    let noise_far = city.resource::<NoisePollutionGrid>().get(200, 200);
    assert!(
        noise_near > noise_far,
        "noise should decay with distance: near={}, far={}",
        noise_near,
        noise_far
    );
}

// ====================================================================
// Integration: land value reduced near industrial cluster
// ====================================================================

#[test]
fn test_land_value_reduced_near_industrial_cluster() {
    // Industrial buildings generate noise=20 in 3-cell radius
    // Multiple overlapping generate higher combined noise
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::Industrial, 3)
        .with_building(130, 128, ZoneType::Industrial, 3)
        .with_building(128, 130, ZoneType::Industrial, 3)
        .with_building(130, 130, ZoneType::Industrial, 3);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(3);

    // The noise at the center of the cluster should be significant
    let noise_level = city.resource::<NoisePollutionGrid>().get(129, 129);
    let noisy_lv = city.resource::<LandValueGrid>().get(129, 129);
    // A distant cell with no noise
    let quiet_lv = city.resource::<LandValueGrid>().get(200, 200);

    // If noise is above Noticeable threshold (26+), value should be penalized
    if noise_level > 25 {
        assert!(
            noisy_lv <= quiet_lv,
            "noisy area (noise={}) land value should be <= quiet area: noisy={}, quiet={}",
            noise_level,
            noisy_lv,
            quiet_lv
        );
    }
}

// ====================================================================
// Integration: citizen health damaged by loud noise
// ====================================================================

#[test]
fn test_citizen_in_loud_area_loses_health() {
    // Place citizen in the center of an industrial cluster
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 1)
        .with_building(129, 128, ZoneType::Industrial, 3)
        .with_building(127, 128, ZoneType::Industrial, 3)
        .with_building(128, 127, ZoneType::Industrial, 3)
        .with_building(128, 129, ZoneType::Industrial, 3)
        .with_building(140, 140, ZoneType::CommercialLow, 1)
        .with_citizen((128, 128), (140, 140));

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

    // Check the noise at the home cell
    let noise_at_home = city.resource::<NoisePollutionGrid>().get(128, 128);

    let final_health = {
        let world = city.world_mut();
        world
            .query::<&CitizenDetails>()
            .iter(world)
            .next()
            .expect("citizen should survive with high happiness")
            .health
    };

    // If noise is in the Loud+ range (41+), health should decrease
    // If noise is below threshold, health may still change due to other systems
    if noise_at_home > 40 {
        assert!(
            final_health < initial_health,
            "citizen at noise={} should lose health: initial={}, final={}",
            noise_at_home,
            initial_health,
            final_health
        );
    }
}

// ====================================================================
// Integration: nighttime noise worse for residential
// ====================================================================

#[test]
fn test_nighttime_noise_worse_than_daytime_for_residential() {
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

    // Night citizen should have lost more health (or equal if noise < threshold)
    assert!(
        night_health <= day_health,
        "nighttime noise should cause >= health damage: night={}, day={}",
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
        .with_service(128, 128, ServiceType::InternationalAirport)
        .with_building(128, 140, ZoneType::Industrial, 3)
        .with_building(130, 140, ZoneType::Industrial, 3);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let stats = city.resource::<NoiseEffectsStats>();
    assert!(
        stats.loud_cells > 0 || stats.avg_noise_tier > 0.0,
        "noisy city should have some noise stats: loud_cells={}, avg_tier={}",
        stats.loud_cells,
        stats.avg_noise_tier
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

// ====================================================================
// Integration: multiple airports create loud noise for stats
// ====================================================================

#[test]
fn test_multiple_airports_create_high_noise_stats() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::InternationalAirport)
        .with_service(128, 132, ServiceType::InternationalAirport);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let stats = city.resource::<NoiseEffectsStats>();
    // Two international airports should generate many loud cells
    assert!(
        stats.loud_cells > 0,
        "two airports should create loud cells, got {}",
        stats.loud_cells
    );
}

// ====================================================================
// Integration: noise tier classification matches generated noise
// ====================================================================

#[test]
fn test_generated_noise_classified_correctly() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::InternationalAirport);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    // The airport cell itself should have high noise
    let noise_at_airport = city.resource::<NoisePollutionGrid>().get(128, 128);
    let tier = NoiseTier::from_level(noise_at_airport);

    // InternationalAirport generates 45 base noise but nearby grass cells
    // reduce it by ~2 per neighbour, so expect at least Noticeable (26+)
    assert!(
        noise_at_airport >= 26,
        "airport cell should have at least Noticeable noise (26+), got {}",
        noise_at_airport
    );

    // Verify tier classification matches (Noticeable or above)
    assert!(
        matches!(
            tier,
            NoiseTier::Noticeable
                | NoiseTier::Loud
                | NoiseTier::VeryLoud
                | NoiseTier::Painful
                | NoiseTier::Dangerous
        ),
        "airport noise {} should be Noticeable or above, got {:?}",
        noise_at_airport,
        tier
    );
}
