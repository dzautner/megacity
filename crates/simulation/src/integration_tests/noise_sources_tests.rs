//! Integration tests for POLL-019: Complete Noise Source Type Table
//!
//! Verifies that the 17-source noise emission table correctly integrates
//! with the simulation, including activity-pattern modulation based on
//! time of day.

use crate::noise::NoisePollutionGrid;
use crate::noise_sources::{
    effective_db, lookup_source, NoiseSourceTableRes, NoiseSourceType,
};
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;
use crate::wind::WindState;

// ====================================================================
// Table completeness
// ====================================================================

#[test]
fn test_noise_source_table_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<NoiseSourceTableRes>();
}

#[test]
fn test_noise_source_table_has_all_17_entries() {
    let city = TestCity::new();
    let table = city.resource::<NoiseSourceTableRes>();
    assert_eq!(table.entries.len(), 17);
}

// ====================================================================
// Fire station generates noise (new source not in base system)
// ====================================================================

#[test]
fn test_fire_station_generates_noise_during_day() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::FireStation)
        .with_time(12.0);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let noise = city.resource::<NoisePollutionGrid>().get(128, 128);
    assert!(
        noise > 0,
        "fire station should generate noise at source, got {}",
        noise
    );
}

#[test]
fn test_fire_station_noise_always_active() {
    // Fire station is Always active, so it should generate noise at night too
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::FireStation)
        .with_time(2.0); // 2 AM

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let noise = city.resource::<NoisePollutionGrid>().get(128, 128);
    assert!(
        noise > 0,
        "fire station should generate noise at night (Always), got {}",
        noise
    );
}

// ====================================================================
// Power plant generates noise
// ====================================================================

#[test]
fn test_power_plant_generates_noise() {
    let mut city = TestCity::new()
        .with_utility(128, 128, UtilityType::PowerPlant)
        .with_time(12.0);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let noise = city.resource::<NoisePollutionGrid>().get(128, 128);
    assert!(
        noise > 0,
        "power plant should generate noise, got {}",
        noise
    );
}

// ====================================================================
// School generates noise only during daytime
// ====================================================================

#[test]
fn test_school_noisy_during_day() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::ElementarySchool)
        .with_time(10.0); // 10 AM school hours

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let noise = city.resource::<NoisePollutionGrid>().get(128, 128);
    assert!(
        noise > 0,
        "school should generate noise during day, got {}",
        noise
    );
}

#[test]
fn test_school_quiet_at_night() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::ElementarySchool)
        .with_time(23.0); // 11 PM

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    // School is Daytime only -- effective_db returns 0 at night
    let eff = effective_db(NoiseSourceType::School, 23.0);
    assert!(
        eff < f32::EPSILON,
        "school effective dB at night should be 0, got {}",
        eff
    );
    // Noise at the cell should not include school contribution
    let noise = city.resource::<NoisePollutionGrid>().get(128, 128);
    assert!(
        noise < 10,
        "school cell at night should be quiet, got {}",
        noise
    );
}

// ====================================================================
// Park generates very low noise
// ====================================================================

#[test]
fn test_park_generates_low_noise_during_day() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::SmallPark)
        .with_time(12.0);

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    // Park is 35 dB -- very quiet
    let noise = city.resource::<NoisePollutionGrid>().get(128, 128);
    assert!(
        noise <= 50,
        "park noise should be low, got {}",
        noise
    );
}

// ====================================================================
// Train station generates noise (24h)
// ====================================================================

#[test]
fn test_train_station_generates_noise() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::TrainStation)
        .with_time(3.0); // 3 AM -- train station is Always active

    {
        let world = city.world_mut();
        world.resource_mut::<WindState>().speed = 0.0;
    }

    city.tick_slow_cycles(2);

    let noise = city.resource::<NoisePollutionGrid>().get(128, 128);
    assert!(
        noise > 0,
        "train station should generate noise at 3 AM, got {}",
        noise
    );
}

// ====================================================================
// Activity pattern correctness via effective_db
// ====================================================================

#[test]
fn test_nightclub_only_active_at_night() {
    assert!(effective_db(NoiseSourceType::Nightclub, 0.0) > 0.0);
    assert!(effective_db(NoiseSourceType::Nightclub, 3.0) > 0.0);
    assert!(effective_db(NoiseSourceType::Nightclub, 23.0) > 0.0);
    assert!((effective_db(NoiseSourceType::Nightclub, 12.0)).abs() < f32::EPSILON);
    assert!((effective_db(NoiseSourceType::Nightclub, 8.0)).abs() < f32::EPSILON);
}

#[test]
fn test_construction_only_active_during_day() {
    assert!(effective_db(NoiseSourceType::Construction, 8.0) > 0.0);
    assert!(effective_db(NoiseSourceType::Construction, 14.0) > 0.0);
    assert!((effective_db(NoiseSourceType::Construction, 23.0)).abs() < f32::EPSILON);
    assert!((effective_db(NoiseSourceType::Construction, 3.0)).abs() < f32::EPSILON);
}

#[test]
fn test_stadium_active_during_events() {
    assert!(effective_db(NoiseSourceType::Stadium, 20.0) > 0.0);
    assert!((effective_db(NoiseSourceType::Stadium, 8.0)).abs() < f32::EPSILON);
}

#[test]
fn test_highway_always_active() {
    for h in [0.0, 6.0, 12.0, 18.0, 23.0] {
        assert!(
            effective_db(NoiseSourceType::Highway, h) > 0.0,
            "highway should be active at hour {}",
            h
        );
    }
}

// ====================================================================
// All 17 source types have correct dB in table
// ====================================================================

#[test]
fn test_all_17_source_levels_match_spec() {
    let spec: &[(NoiseSourceType, f32)] = &[
        (NoiseSourceType::Highway, 75.0),
        (NoiseSourceType::Arterial, 70.0),
        (NoiseSourceType::LocalRoad, 55.0),
        (NoiseSourceType::RailCorridor, 80.0),
        (NoiseSourceType::Airport, 105.0),
        (NoiseSourceType::Construction, 90.0),
        (NoiseSourceType::HeavyIndustry, 85.0),
        (NoiseSourceType::LightIndustry, 70.0),
        (NoiseSourceType::CommercialHvac, 60.0),
        (NoiseSourceType::Nightclub, 95.0),
        (NoiseSourceType::FireStation, 80.0),
        (NoiseSourceType::PowerPlant, 75.0),
        (NoiseSourceType::Stadium, 95.0),
        (NoiseSourceType::School, 70.0),
        (NoiseSourceType::Park, 35.0),
        (NoiseSourceType::ParkingStructure, 65.0),
        (NoiseSourceType::TrainStation, 75.0),
    ];
    for (st, expected_db) in spec {
        let entry = lookup_source(*st).expect(&format!("missing {:?}", st));
        assert!(
            (entry.db_level - expected_db).abs() < f32::EPSILON,
            "{:?}: expected {} dB, got {}",
            st,
            expected_db,
            entry.db_level
        );
    }
}
