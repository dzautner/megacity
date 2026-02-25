//! Integration tests for POL-004: Housing Affordability Crisis Mechanic.

use crate::buildings::Building;
use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::grid::ZoneType;
use crate::housing_affordability::{AffordabilityTier, HousingAffordability};
use crate::immigration::CityAttractiveness;
use crate::land_value::LandValueGrid;
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::test_harness::TestCity;
use crate::Saveable;

/// Spawn a residential building at (gx, gy) with given capacity and occupants.
fn spawn_residential(
    city: &mut TestCity,
    gx: usize,
    gy: usize,
    capacity: u32,
    occupants: u32,
) -> bevy::prelude::Entity {
    let entity = city
        .world_mut()
        .spawn(Building {
            zone_type: ZoneType::ResidentialHigh,
            grid_x: gx,
            grid_y: gy,
            level: 1,
            capacity,
            occupants,
            width: 1,
            height: 1,
        })
        .id();
    {
        let mut grid = city.world_mut().resource_mut::<crate::grid::WorldGrid>();
        grid.get_mut(gx, gy).building_id = Some(entity);
        grid.get_mut(gx, gy).zone = ZoneType::ResidentialHigh;
    }
    entity
}

/// Spawn a citizen with the given salary living in the given building at (gx, gy).
fn spawn_citizen_with_salary(
    city: &mut TestCity,
    salary: f32,
    home_building: bevy::prelude::Entity,
    gx: usize,
    gy: usize,
) {
    let (wx, wy) = crate::grid::WorldGrid::grid_to_world(gx, gy);
    city.world_mut().spawn((
        Citizen,
        Position { x: wx, y: wy },
        Velocity { x: 0.0, y: 0.0 },
        HomeLocation {
            grid_x: gx,
            grid_y: gy,
            building: home_building,
        },
        WorkLocation {
            grid_x: gx + 5,
            grid_y: gy,
            building: bevy::prelude::Entity::PLACEHOLDER,
        },
        CitizenStateComp(CitizenState::AtHome),
        PathCache::new(Vec::new()),
        CitizenDetails {
            age: 30,
            gender: Gender::Male,
            education: 2,
            happiness: 70.0,
            health: 80.0,
            salary,
            savings: salary * 3.0,
        },
        Personality::default(),
        Needs::default(),
        Family::default(),
        ActivityTimer::default(),
        ChosenTransportMode::default(),
    ));
}

/// Set land value for a specific cell.
fn set_land_value(city: &mut TestCity, x: usize, y: usize, value: u8) {
    let mut lv = city.world_mut().resource_mut::<LandValueGrid>();
    lv.set(x, y, value);
}

// -----------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------

#[test]
fn test_affordability_resource_initialized() {
    let city = TestCity::new();
    let aff = city.resource::<HousingAffordability>();
    assert!(!aff.crisis_active);
    assert_eq!(aff.crisis_duration, 0);
    assert_eq!(aff.tier, AffordabilityTier::Healthy);
    assert!((aff.severity - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_healthy_affordability_no_crisis() {
    let mut city = TestCity::new();

    // High salary citizens in a low land-value area -> healthy ratio
    let bldg = spawn_residential(&mut city, 50, 50, 10, 5);
    set_land_value(&mut city, 50, 50, 10); // low land value -> low rent

    for _ in 0..5 {
        spawn_citizen_with_salary(&mut city, 5000.0, bldg, 50, 50);
    }

    city.tick_slow_cycle();

    let aff = city.resource::<HousingAffordability>();
    // rent ~ 10 * 8 = 80, income = 5000, ratio = 0.016
    assert!(
        aff.affordability_ratio < HEALTHY_THRESHOLD,
        "Ratio {} should be below healthy threshold",
        aff.affordability_ratio
    );
    assert_eq!(aff.tier, AffordabilityTier::Healthy);
    assert!(!aff.crisis_active);
}

#[test]
fn test_stressed_affordability_tier() {
    let mut city = TestCity::new();

    // Moderate rent vs income -> stressed tier
    let bldg = spawn_residential(&mut city, 50, 50, 10, 5);
    set_land_value(&mut city, 50, 50, 100); // rent ~ 100 * 8 = 800

    for _ in 0..5 {
        // salary = 2000, ratio = 800/2000 = 0.4 -> stressed
        spawn_citizen_with_salary(&mut city, 2000.0, bldg, 50, 50);
    }

    city.tick_slow_cycle();

    let aff = city.resource::<HousingAffordability>();
    assert!(
        aff.affordability_ratio >= HEALTHY_THRESHOLD
            && aff.affordability_ratio < STRESSED_THRESHOLD,
        "Ratio {} should be in stressed range (0.3-0.5)",
        aff.affordability_ratio
    );
    assert_eq!(aff.tier, AffordabilityTier::Stressed);
}

#[test]
fn test_crisis_triggers_above_threshold() {
    let mut city = TestCity::new();

    // High rent vs low income -> crisis
    let bldg = spawn_residential(&mut city, 50, 50, 10, 5);
    set_land_value(&mut city, 50, 50, 200); // rent ~ 200 * 8 = 1600

    for _ in 0..5 {
        // salary = 2000, ratio = 1600/2000 = 0.8 -> crisis
        spawn_citizen_with_salary(&mut city, 2000.0, bldg, 50, 50);
    }

    city.tick_slow_cycle();

    let aff = city.resource::<HousingAffordability>();
    assert!(
        aff.affordability_ratio > CRISIS_TRIGGER,
        "Ratio {} should exceed crisis trigger {}",
        aff.affordability_ratio,
        CRISIS_TRIGGER
    );
    assert!(aff.crisis_active, "Crisis should be active");
    assert!(aff.crisis_duration >= 1, "Crisis duration should be >= 1");
    assert!(aff.severity > 0.0, "Severity should be positive");
}

#[test]
fn test_crisis_severity_increases_over_time() {
    let mut city = TestCity::new();

    let bldg = spawn_residential(&mut city, 50, 50, 10, 5);
    set_land_value(&mut city, 50, 50, 200);

    for _ in 0..5 {
        spawn_citizen_with_salary(&mut city, 2000.0, bldg, 50, 50);
    }

    city.tick_slow_cycle();
    let severity_1 = city.resource::<HousingAffordability>().severity;
    let duration_1 = city.resource::<HousingAffordability>().crisis_duration;

    city.tick_slow_cycle();
    let severity_2 = city.resource::<HousingAffordability>().severity;
    let duration_2 = city.resource::<HousingAffordability>().crisis_duration;

    assert!(
        duration_2 > duration_1,
        "Crisis duration should increase: {} -> {}",
        duration_1,
        duration_2
    );
    assert!(
        severity_2 > severity_1,
        "Severity should increase over time: {} -> {}",
        severity_1,
        severity_2
    );
}

#[test]
fn test_crisis_reduces_attractiveness() {
    let mut city = TestCity::new();

    let bldg = spawn_residential(&mut city, 50, 50, 10, 5);
    set_land_value(&mut city, 50, 50, 200);

    for _ in 0..5 {
        spawn_citizen_with_salary(&mut city, 2000.0, bldg, 50, 50);
    }

    // Manually set attractiveness high so we can measure the penalty
    city.world_mut()
        .resource_mut::<CityAttractiveness>()
        .overall_score = 80.0;
    city.world_mut()
        .resource_mut::<CityAttractiveness>()
        .housing_factor = 0.8;

    city.tick_slow_cycle();

    let attract = city.resource::<CityAttractiveness>();
    assert!(
        attract.overall_score < 80.0,
        "Attractiveness should be reduced by crisis, got {}",
        attract.overall_score
    );
    assert!(
        attract.housing_factor < 0.8,
        "Housing factor should be reduced by crisis, got {}",
        attract.housing_factor
    );
}

#[test]
fn test_crisis_resolves_when_ratio_drops() {
    let mut city = TestCity::new();

    // Start with crisis conditions
    let bldg = spawn_residential(&mut city, 50, 50, 10, 5);
    set_land_value(&mut city, 50, 50, 200);

    for _ in 0..5 {
        spawn_citizen_with_salary(&mut city, 2000.0, bldg, 50, 50);
    }

    city.tick_slow_cycle();
    assert!(
        city.resource::<HousingAffordability>().crisis_active,
        "Crisis should be active"
    );

    // Drop land value dramatically to resolve crisis
    set_land_value(&mut city, 50, 50, 5); // rent ~ 5 * 8 = 40, ratio = 40/2000 = 0.02

    city.tick_slow_cycle();

    let aff = city.resource::<HousingAffordability>();
    assert!(
        !aff.crisis_active,
        "Crisis should resolve when ratio drops below relief threshold"
    );
    assert_eq!(aff.crisis_duration, 0);
    assert!((aff.severity - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_no_citizens_no_crash() {
    let mut city = TestCity::new();

    // No citizens, no buildings — should not crash
    city.tick_slow_cycle();

    let aff = city.resource::<HousingAffordability>();
    assert!(!aff.crisis_active);
    assert!((aff.affordability_ratio - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_citizen_tier_counts() {
    let mut city = TestCity::new();

    let bldg = spawn_residential(&mut city, 50, 50, 20, 10);
    set_land_value(&mut city, 50, 50, 100); // rent ~ 800

    // Some high-income (healthy), some medium (stressed), some low (crisis)
    for _ in 0..3 {
        spawn_citizen_with_salary(&mut city, 10000.0, bldg, 50, 50); // ratio = 0.08, healthy
    }
    for _ in 0..3 {
        spawn_citizen_with_salary(&mut city, 2000.0, bldg, 50, 50); // ratio = 0.4, stressed
    }
    for _ in 0..3 {
        spawn_citizen_with_salary(&mut city, 1000.0, bldg, 50, 50); // ratio = 0.8, crisis
    }

    city.tick_slow_cycle();

    let aff = city.resource::<HousingAffordability>();
    assert_eq!(aff.citizens_healthy, 3, "Expected 3 healthy citizens");
    assert_eq!(aff.citizens_stressed, 3, "Expected 3 stressed citizens");
    assert_eq!(aff.citizens_crisis, 3, "Expected 3 crisis citizens");
}

#[test]
fn test_saveable_roundtrip() {
    let state = HousingAffordability {
        affordability_ratio: 0.65,
        tier: AffordabilityTier::Crisis,
        crisis_active: true,
        crisis_duration: 12,
        severity: 0.24,
        average_rent: 1200.0,
        average_income: 1846.0,
        citizens_healthy: 100,
        citizens_stressed: 200,
        citizens_crisis: 50,
    };

    let bytes = state.save_to_bytes().unwrap();
    let restored = HousingAffordability::load_from_bytes(&bytes);

    assert!(restored.crisis_active);
    assert_eq!(restored.crisis_duration, 12);
    assert!((restored.severity - 0.24).abs() < f32::EPSILON);
    assert!((restored.affordability_ratio - 0.65).abs() < f32::EPSILON);
    assert_eq!(restored.tier, AffordabilityTier::Crisis);
}

#[test]
fn test_saveable_skip_default_state() {
    let state = HousingAffordability::default();
    assert!(
        state.save_to_bytes().is_none(),
        "Default state should skip saving"
    );
}

// Thresholds used in tests (re-imported from module for clarity).
const HEALTHY_THRESHOLD: f32 = 0.3;
const STRESSED_THRESHOLD: f32 = 0.5;
const CRISIS_TRIGGER: f32 = 0.4;
