//! Integration tests for life simulation: aging, death, education advancement,
//! and home building validity (TEST-047).

use bevy::prelude::*;

use crate::buildings::Building;
use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity,
};
use crate::death_care::DeathCareStats;
use crate::education::EducationGrid;
use crate::grid::{WorldGrid, ZoneType};
use crate::immigration::CityAttractiveness;
use crate::life_simulation::LifeSimTimer;
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;
use crate::utilities::UtilityType;
use crate::TestSafetyNet;
use crate::virtual_population::VirtualPopulation;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Spawn a citizen with specific age, education, health, and gender.
fn spawn_citizen_with(
    world: &mut World,
    home_building: Entity,
    home_gx: usize,
    home_gy: usize,
    age: u8,
    education: u8,
    health: f32,
    gender: Gender,
) -> Entity {
    let (wx, wy) = WorldGrid::grid_to_world(home_gx, home_gy);
    world
        .spawn((
            Citizen,
            Position { x: wx, y: wy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: home_gx,
                grid_y: home_gy,
                building: home_building,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age,
                gender,
                education,
                happiness: 90.0,
                health,
                salary: CitizenDetails::base_salary_for_education(education),
                savings: 5000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs {
                hunger: 100.0,
                energy: 100.0,
                social: 100.0,
                fun: 100.0,
                comfort: 100.0,
            },
            Family::default(),
            ActivityTimer::default(),
            ChosenTransportMode::default(),
        ))
        .id()
}

/// Create a test city with a residential building at (50,50) plus utilities.
/// Removes TestSafetyNet so aging/death systems can run for life simulation tests.
fn setup_city_with_home() -> (TestCity, Entity) {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 3)
        .with_utility(52, 52, UtilityType::PowerPlant)
        .with_utility(54, 54, UtilityType::WaterTower);
    city.world_mut().remove_resource::<TestSafetyNet>();
    let building = city.grid().get(50, 50).building_id.unwrap();
    (city, building)
}

/// Prevent emigration by setting city attractiveness high.
fn prevent_emigration(world: &mut World) {
    if let Some(mut attr) = world.get_resource_mut::<CityAttractiveness>() {
        attr.overall_score = 80.0;
    }
}

// ---------------------------------------------------------------------------
// Test: aging increments age by 1 per aging tick
// ---------------------------------------------------------------------------

#[test]
fn test_aging_increments_age_by_one_per_year() {
    let (mut city, building) = setup_city_with_home();
    let ages: Vec<u8> = vec![0, 10, 25, 50, 65];
    let mut entities = Vec::new();
    for &age in &ages {
        let e = spawn_citizen_with(
            city.world_mut(),
            building,
            50,
            50,
            age,
            0,
            100.0,
            Gender::Male,
        );
        entities.push(e);
    }
    prevent_emigration(city.world_mut());

    let initial_ages: Vec<u8> = {
        let world = city.world_mut();
        entities
            .iter()
            .map(|&e| world.get::<CitizenDetails>(e).unwrap().age)
            .collect()
    };

    // Aging triggers when clock.day >= last_aging_day + 365.
    city.world_mut().resource_mut::<GameClock>().day = 365;
    city.tick(1);

    for (i, &entity) in entities.iter().enumerate() {
        let world = city.world_mut();
        if let Some(details) = world.get::<CitizenDetails>(entity) {
            assert_eq!(
                details.age,
                initial_ages[i] + 1,
                "Citizen starting at age {} should now be {}, got {}",
                initial_ages[i],
                initial_ages[i] + 1,
                details.age
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Test: multiple aging cycles increment cumulatively
// ---------------------------------------------------------------------------

#[test]
fn test_aging_multiple_years_cumulative() {
    let (mut city, building) = setup_city_with_home();
    let entity = spawn_citizen_with(
        city.world_mut(),
        building,
        50,
        50,
        20,
        2,
        100.0,
        Gender::Female,
    );
    prevent_emigration(city.world_mut());

    for year in 1..=3 {
        city.world_mut().resource_mut::<GameClock>().day = 365 * year;
        city.tick(1);
    }

    let age = city
        .world_mut()
        .get::<CitizenDetails>(entity)
        .unwrap()
        .age;
    assert_eq!(
        age, 23,
        "Citizen should be 23 after 3 aging cycles from age 20"
    );
}

// ---------------------------------------------------------------------------
// Test: citizens at MAX_AGE (100) always die
// ---------------------------------------------------------------------------

#[test]
fn test_death_certainty_at_max_age() {
    let (mut city, building) = setup_city_with_home();

    let mut entities = Vec::new();
    for _ in 0..10 {
        let e = spawn_citizen_with(
            city.world_mut(),
            building,
            50,
            50,
            99,
            0,
            100.0,
            Gender::Male,
        );
        entities.push(e);
    }
    city.world_mut()
        .resource_mut::<VirtualPopulation>()
        .total_virtual = 10;
    prevent_emigration(city.world_mut());

    let count_before = city.citizen_count();
    assert_eq!(count_before, 10);

    // Trigger aging: 99 -> 100 => MAX_AGE => guaranteed despawn
    city.world_mut().resource_mut::<GameClock>().day = 365;
    city.tick(1);

    let count_after = city.citizen_count();
    assert_eq!(
        count_after, 0,
        "All citizens at age 100 should die (guaranteed death at MAX_AGE)"
    );

    let stats = city.resource::<DeathCareStats>();
    assert_eq!(stats.total_deaths_this_month, 10, "Should record 10 deaths");
}

// ---------------------------------------------------------------------------
// Test: death probability is higher for older citizens vs younger
// ---------------------------------------------------------------------------

#[test]
fn test_death_probability_increases_with_age() {
    let mut deaths_old = 0u32;
    let mut deaths_young_old = 0u32;
    let trials = 20;

    for _ in 0..trials {
        // Age 95 -> 96: death_chance = (96-70)/60 = 0.433
        let (mut city, building) = setup_city_with_home();
        let e = spawn_citizen_with(
            city.world_mut(),
            building,
            50,
            50,
            95,
            0,
            100.0,
            Gender::Male,
        );
        city.world_mut()
            .resource_mut::<VirtualPopulation>()
            .total_virtual = 1;
        prevent_emigration(city.world_mut());
        city.world_mut().resource_mut::<GameClock>().day = 365;
        city.tick(1);
        if city.world_mut().get::<CitizenDetails>(e).is_none() {
            deaths_old += 1;
        }

        // Age 69 -> 70: death_chance = (70-70)/60 = 0.0
        let (mut city2, building2) = setup_city_with_home();
        let e2 = spawn_citizen_with(
            city2.world_mut(),
            building2,
            50,
            50,
            69,
            0,
            100.0,
            Gender::Female,
        );
        city2
            .world_mut()
            .resource_mut::<VirtualPopulation>()
            .total_virtual = 1;
        prevent_emigration(city2.world_mut());
        city2.world_mut().resource_mut::<GameClock>().day = 365;
        city2.tick(1);
        if city2.world_mut().get::<CitizenDetails>(e2).is_none() {
            deaths_young_old += 1;
        }
    }

    assert!(
        deaths_old > deaths_young_old,
        "Citizens aged 96 should die more often than those aged 70: \
         old_deaths={deaths_old}, young_old_deaths={deaths_young_old} over {trials} trials"
    );
}

// ---------------------------------------------------------------------------
// Test: education advancement from school coverage
// ---------------------------------------------------------------------------

#[test]
fn test_education_advancement_from_school_coverage() {
    let (mut city, building) = setup_city_with_home();

    // Citizen aged 20, education=0: eligible for all education levels
    let entity = spawn_citizen_with(
        city.world_mut(),
        building,
        50,
        50,
        20,
        0,
        100.0,
        Gender::Female,
    );
    prevent_emigration(city.world_mut());

    // Directly write education level 3 (University) to the EducationGrid
    // at the citizen's home cell. This avoids running tick_slow_cycle()
    // and the associated emigration/attractiveness side effects.
    city.world_mut()
        .resource_mut::<EducationGrid>()
        .set(50, 50, 3);

    // Advance the education timer to just below threshold, then tick once
    // so education_advancement fires exactly once.
    city.world_mut()
        .resource_mut::<LifeSimTimer>()
        .education_tick = 1439; // EDUCATION_INTERVAL - 1

    city.tick(1);

    let details = city
        .world_mut()
        .get::<CitizenDetails>(entity)
        .expect("citizen should exist after a single education tick");
    assert!(
        details.education > 0,
        "Citizen at university-covered cell should advance education, got {}",
        details.education
    );
}

// ---------------------------------------------------------------------------
// Test: education requires eligible age
// ---------------------------------------------------------------------------

#[test]
fn test_education_requires_eligible_age() {
    let (mut city, building) = setup_city_with_home();

    // Child aged 3: NOT eligible for university (requires age >= 18)
    let child = spawn_citizen_with(
        city.world_mut(),
        building,
        50,
        50,
        3,
        0,
        100.0,
        Gender::Male,
    );
    // Adult aged 22: eligible for university
    let adult = spawn_citizen_with(
        city.world_mut(),
        building,
        50,
        50,
        22,
        0,
        100.0,
        Gender::Female,
    );
    prevent_emigration(city.world_mut());

    // Directly set education grid to level 3 (University) at home cell
    city.world_mut()
        .resource_mut::<EducationGrid>()
        .set(50, 50, 3);

    // Fire education_advancement once
    city.world_mut()
        .resource_mut::<LifeSimTimer>()
        .education_tick = 1439;
    city.tick(1);

    let child_edu = city
        .world_mut()
        .get::<CitizenDetails>(child)
        .expect("child should exist after 1 tick")
        .education;
    let adult_edu = city
        .world_mut()
        .get::<CitizenDetails>(adult)
        .expect("adult should exist after 1 tick")
        .education;

    assert_eq!(
        child_edu, 0,
        "Child aged 3 should NOT get university education"
    );
    assert!(
        adult_edu > 0,
        "Adult aged 22 should advance education, got {adult_edu}"
    );
}

// ---------------------------------------------------------------------------
// Test: citizens with homes reference valid buildings
// ---------------------------------------------------------------------------

#[test]
fn test_citizens_home_references_valid_building() {
    let mut city = TestCity::new()
        .with_building(50, 50, ZoneType::ResidentialLow, 3)
        .with_building(60, 60, ZoneType::ResidentialLow, 2)
        .with_utility(55, 55, UtilityType::PowerPlant)
        .with_utility(57, 57, UtilityType::WaterTower);

    let building_50 = city.grid().get(50, 50).building_id.unwrap();
    let building_60 = city.grid().get(60, 60).building_id.unwrap();

    for _ in 0..5 {
        spawn_citizen_with(
            city.world_mut(),
            building_50,
            50,
            50,
            30,
            1,
            90.0,
            Gender::Male,
        );
    }
    for _ in 0..3 {
        spawn_citizen_with(
            city.world_mut(),
            building_60,
            60,
            60,
            25,
            2,
            95.0,
            Gender::Female,
        );
    }
    prevent_emigration(city.world_mut());
    city.tick(10);

    let world = city.world_mut();
    let mut query = world.query_filtered::<&HomeLocation, With<Citizen>>();
    let homes: Vec<(usize, usize, Entity)> = query
        .iter(world)
        .map(|h| (h.grid_x, h.grid_y, h.building))
        .collect();

    assert!(!homes.is_empty(), "should have citizens with homes");
    for (gx, gy, building_entity) in &homes {
        let building = world.get::<Building>(*building_entity);
        assert!(
            building.is_some(),
            "Home building {:?} at ({},{}) should be a valid Building",
            building_entity,
            gx,
            gy
        );
        let b = building.unwrap();
        assert_eq!(
            (b.grid_x, b.grid_y),
            (*gx, *gy),
            "Building grid position should match citizen's home location"
        );
    }
}
