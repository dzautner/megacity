//! Integration tests for the Park District System (SERV-007).

use crate::park_districts::{ParkDistrictEffects, ParkDistrictState, ParkType};
use crate::services::ServiceType;
use crate::test_harness::TestCity;

#[test]
fn test_park_district_creation_and_level_1_effects() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::SmallPark);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<ParkDistrictState>();
        state.create_district(ParkType::CityPark, 50, 50);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<ParkDistrictEffects>();
    let happiness = effects.happiness_at(50, 50);
    assert!(
        happiness >= 3.0,
        "Level 1 park district should provide >= 3.0 happiness at center, got {happiness}"
    );

    let land_val = effects.land_value_at(50, 50);
    assert!(
        land_val >= 2.0,
        "Level 1 park district should provide >= 2.0 land value at center, got {land_val}"
    );
}

#[test]
fn test_park_district_levels_up_with_attractions_and_visitors() {
    // Place 5 attractions to meet L3 threshold
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::SmallPark)
        .with_service(51, 50, ServiceType::LargePark)
        .with_service(52, 50, ServiceType::Playground)
        .with_service(53, 50, ServiceType::SportsField)
        .with_service(54, 50, ServiceType::Plaza);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<ParkDistrictState>();
        let id = state.create_district(ParkType::CityPark, 50, 50);
        // Manually set visitors to meet L3 threshold (200+)
        if let Some(d) = state.get_district_mut(id) {
            d.total_visitors = 200;
        }
    }

    city.tick_slow_cycle();

    let state = city.resource::<ParkDistrictState>();
    let district = state.get_district(1).expect("district should exist");
    // With 5 attractions and 200+ visitors, should be at least L3
    assert!(
        district.level >= 3,
        "District with 5 attractions and 200 visitors should be >= L3, got L{}",
        district.level
    );
}

#[test]
fn test_park_district_happiness_radius() {
    let mut city = TestCity::new()
        .with_service(128, 128, ServiceType::SmallPark);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<ParkDistrictState>();
        state.create_district(ParkType::CityPark, 128, 128);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<ParkDistrictEffects>();

    // Center should have effect
    let center_happiness = effects.happiness_at(128, 128);
    assert!(center_happiness > 0.0, "Center should have happiness bonus");

    // Nearby cell (within L1 radius of 6) should have effect
    let nearby = effects.happiness_at(131, 128);
    assert!(nearby > 0.0, "Cell within radius should have happiness bonus");

    // Far cell (well outside L1 radius of 6) should have no effect
    let far = effects.happiness_at(150, 128);
    assert!(
        far < f32::EPSILON,
        "Cell far outside radius should have no happiness, got {far}"
    );
}

#[test]
fn test_park_district_land_value_bonus() {
    let mut city = TestCity::new()
        .with_service(100, 100, ServiceType::LargePark)
        .with_service(101, 100, ServiceType::SmallPark);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<ParkDistrictState>();
        state.create_district(ParkType::CityPark, 100, 100);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<ParkDistrictEffects>();
    let lv = effects.land_value_at(100, 100);
    assert!(
        lv >= 2.0,
        "Park district should provide land value bonus, got {lv}"
    );
}

#[test]
fn test_nature_reserve_pollution_reduction() {
    let mut city = TestCity::new()
        .with_service(80, 80, ServiceType::LargePark);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<ParkDistrictState>();
        state.create_district(ParkType::NatureReserve, 80, 80);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<ParkDistrictEffects>();
    let idx = ParkDistrictEffects::idx(80, 80);
    assert!(
        effects.pollution_reduction[idx] > 0,
        "NatureReserve should provide pollution reduction at center"
    );
}

#[test]
fn test_zoo_education_bonus() {
    let mut city = TestCity::new()
        .with_service(60, 60, ServiceType::SmallPark);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<ParkDistrictState>();
        state.create_district(ParkType::Zoo, 60, 60);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<ParkDistrictEffects>();
    let idx = ParkDistrictEffects::idx(60, 60);
    assert!(
        effects.education_bonus[idx] > 0.0,
        "Zoo should provide education bonus at center, got {}",
        effects.education_bonus[idx]
    );
}

#[test]
fn test_amusement_park_higher_tourism() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::SmallPark)
        .with_service(51, 50, ServiceType::LargePark)
        .with_service(100, 100, ServiceType::SmallPark)
        .with_service(101, 100, ServiceType::LargePark);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<ParkDistrictState>();
        let id1 = state.create_district(ParkType::CityPark, 50, 50);
        let id2 = state.create_district(ParkType::AmusementPark, 100, 100);
        // Give both enough visitors to reach at least L2
        if let Some(d) = state.get_district_mut(id1) {
            d.total_visitors = 100;
        }
        if let Some(d) = state.get_district_mut(id2) {
            d.total_visitors = 100;
        }
    }

    city.tick_slow_cycle();

    let state = city.resource::<ParkDistrictState>();
    let city_park = state.get_district(1).unwrap();
    let amusement = state.get_district(2).unwrap();

    assert!(
        amusement.tourism_score() >= city_park.tourism_score(),
        "AmusementPark tourism ({}) should be >= CityPark tourism ({})",
        amusement.tourism_score(),
        city_park.tourism_score()
    );
}

#[test]
fn test_entry_fee_generates_revenue() {
    use crate::grid::{RoadType, ZoneType};

    // Need citizens (population) for visitors to be nonzero
    let mut city = TestCity::new()
        .with_road(48, 48, 48, 58, RoadType::Local)
        .with_zone_rect(49, 48, 50, 58, ZoneType::ResidentialLow)
        .with_building(49, 50, ZoneType::ResidentialLow, 1)
        .with_building(50, 50, ZoneType::ResidentialLow, 1)
        .with_citizen((49, 50), (50, 50))
        .with_citizen((49, 50), (50, 50))
        .with_citizen((49, 50), (50, 50))
        .with_citizen((49, 50), (50, 50))
        .with_service(50, 54, ServiceType::SmallPark);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<ParkDistrictState>();
        let id = state.create_district(ParkType::CityPark, 50, 54);
        if let Some(d) = state.get_district_mut(id) {
            d.entry_fee = 3.0;
        }
    }

    // Run slow cycle so stats update (population counted) then park districts update
    city.tick_slow_cycle();

    let state = city.resource::<ParkDistrictState>();
    let district = state.get_district(1).unwrap();
    assert!(
        district.total_revenue > 0.0,
        "District with entry fee and population should generate revenue, got {}",
        district.total_revenue
    );
}

#[test]
fn test_district_removal_clears_effects() {
    let mut city = TestCity::new()
        .with_service(50, 50, ServiceType::SmallPark);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<ParkDistrictState>();
        state.create_district(ParkType::CityPark, 50, 50);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<ParkDistrictEffects>();
    assert!(effects.happiness_at(50, 50) > 0.0);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<ParkDistrictState>();
        state.remove_district(1);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<ParkDistrictEffects>();
    assert!(
        effects.happiness_at(50, 50) < f32::EPSILON,
        "Effects should clear after district removal"
    );
}

#[test]
fn test_multiple_districts_independent_effects() {
    let mut city = TestCity::new()
        .with_service(30, 30, ServiceType::SmallPark)
        .with_service(200, 200, ServiceType::SmallPark);

    {
        let world = city.world_mut();
        let mut state = world.resource_mut::<ParkDistrictState>();
        state.create_district(ParkType::CityPark, 30, 30);
        state.create_district(ParkType::Zoo, 200, 200);
    }

    city.tick_slow_cycle();

    let effects = city.resource::<ParkDistrictEffects>();

    assert!(effects.happiness_at(30, 30) > 0.0, "District 1 should affect its center");
    assert!(effects.happiness_at(200, 200) > 0.0, "District 2 should affect its center");

    let idx_zoo = ParkDistrictEffects::idx(200, 200);
    let idx_city = ParkDistrictEffects::idx(30, 30);
    assert!(
        effects.education_bonus[idx_zoo] > 0.0,
        "Zoo district should have education bonus"
    );
    assert!(
        effects.education_bonus[idx_city] < f32::EPSILON,
        "CityPark district should not have education bonus"
    );
}
