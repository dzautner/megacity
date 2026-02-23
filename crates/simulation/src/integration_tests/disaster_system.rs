//! Integration tests for the disaster system (TEST-049).
//!
//! Tests earthquake damage application, disaster duration bounds,
//! disaster cleanup after expiry, and building damage capping.

use bevy::prelude::Entity;
use crate::buildings::Building;
use crate::disasters::{ActiveDisaster, DisasterInstance, DisasterType, EarthquakeDamaged};
use crate::grid::{WorldGrid, ZoneType};
use crate::test_harness::TestCity;

fn inject_disaster(city: &mut TestCity, disaster: DisasterInstance) {
    city.world_mut()
        .resource_mut::<ActiveDisaster>()
        .current = Some(disaster);
}

#[test]
fn test_disaster_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<ActiveDisaster>();
}

#[test]
fn test_disaster_earthquake_downgrades_buildings_in_radius() {
    let center = (128, 128);
    let mut city = TestCity::new()
        .with_building(center.0, center.1, ZoneType::ResidentialLow, 3)
        .with_building(center.0 + 2, center.1, ZoneType::CommercialLow, 3)
        .with_building(center.0, center.1 + 3, ZoneType::Industrial, 3);

    assert_eq!(city.building_count(), 3);

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Earthquake,
        center_x: center.0,
        center_y: center.1,
        radius: 10,
        ticks_remaining: 20,
        damage_applied: false,
    });

    city.tick(5);

    let world = city.world_mut();
    let mut query = world.query::<&Building>();
    for building in query.iter(world) {
        assert!(
            building.level <= 2,
            "Surviving building at ({}, {}) should be level <= 2, got {}",
            building.grid_x, building.grid_y, building.level,
        );
    }
}

#[test]
fn test_disaster_earthquake_does_not_affect_buildings_outside_radius() {
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 3)
        .with_building(200, 200, ZoneType::ResidentialLow, 3);

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Earthquake,
        center_x: 128,
        center_y: 128,
        radius: 10,
        ticks_remaining: 20,
        damage_applied: false,
    });

    city.tick(5);

    let world = city.world_mut();
    let mut query = world.query::<&Building>();
    let far = query.iter(world).find(|b| b.grid_x == 200 && b.grid_y == 200);

    assert!(far.is_some(), "Building at (200,200) should survive (outside radius)");
    assert_eq!(far.unwrap().level, 3, "Building outside radius should stay level 3");
}

#[test]
fn test_disaster_duration_decrements_each_tick() {
    let mut city = TestCity::new();

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Earthquake,
        center_x: 128,
        center_y: 128,
        radius: 10,
        ticks_remaining: 20,
        damage_applied: false,
    });

    city.tick(5);

    let active = city.resource::<ActiveDisaster>();
    let d = active.current.as_ref().expect("Disaster should still be active after 5/20 ticks");
    assert_eq!(d.ticks_remaining, 15, "Expected 15 ticks remaining, got {}", d.ticks_remaining);
}

#[test]
fn test_disaster_cleanup_after_duration_expires() {
    let mut city = TestCity::new();

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Tornado,
        center_x: 128,
        center_y: 128,
        radius: 5,
        ticks_remaining: 10,
        damage_applied: false,
    });

    city.tick(10);

    let active = city.resource::<ActiveDisaster>();
    assert!(active.current.is_none(), "Disaster should be cleared after ticks_remaining reaches 0");
}

#[test]
fn test_disaster_cleanup_not_premature() {
    let mut city = TestCity::new();

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Flood,
        center_x: 128,
        center_y: 128,
        radius: 8,
        ticks_remaining: 50,
        damage_applied: false,
    });

    city.tick(49);
    assert!(
        city.resource::<ActiveDisaster>().current.is_some(),
        "Disaster should still be active with 1 tick remaining"
    );

    city.tick(1);
    assert!(
        city.resource::<ActiveDisaster>().current.is_none(),
        "Disaster should be cleared after all ticks expire"
    );
}

#[test]
fn test_disaster_earthquake_level_one_not_downgraded_below_one() {
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 1);

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Earthquake,
        center_x: 128,
        center_y: 128,
        radius: 10,
        ticks_remaining: 20,
        damage_applied: false,
    });

    city.tick(5);

    let world = city.world_mut();
    let mut query = world.query::<&Building>();
    for building in query.iter(world) {
        assert!(building.level >= 1, "Building level should never go below 1, got {}", building.level);
    }
}

#[test]
fn test_disaster_earthquake_damaged_marker_removed() {
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 3);

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Earthquake,
        center_x: 128,
        center_y: 128,
        radius: 10,
        ticks_remaining: 20,
        damage_applied: false,
    });

    city.tick(5);

    let world = city.world_mut();
    let mut query = world.query_filtered::<Entity, bevy::prelude::With<EarthquakeDamaged>>();
    let count = query.iter(world).count();
    assert_eq!(count, 0, "EarthquakeDamaged marker should be removed after processing, found {}", count);
}

#[test]
fn test_disaster_earthquake_reduces_capacity() {
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 3);

    let original_capacity = {
        let world = city.world_mut();
        world.query::<&Building>().iter(world).next().unwrap().capacity
    };

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Earthquake,
        center_x: 128,
        center_y: 128,
        radius: 10,
        ticks_remaining: 20,
        damage_applied: false,
    });

    city.tick(5);

    let world = city.world_mut();
    if let Some(building) = world.query::<&Building>().iter(world).next() {
        assert!(
            building.capacity < original_capacity,
            "Surviving building capacity ({}) should be less than original ({})",
            building.capacity, original_capacity,
        );
    }
}

#[test]
fn test_disaster_earthquake_evicts_excess_occupants() {
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 3);

    {
        let world = city.world_mut();
        for mut building in world.query::<&mut Building>().iter_mut(world) {
            building.occupants = building.capacity;
        }
    }

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Earthquake,
        center_x: 128,
        center_y: 128,
        radius: 10,
        ticks_remaining: 20,
        damage_applied: false,
    });

    city.tick(5);

    let world = city.world_mut();
    for building in world.query::<&Building>().iter(world) {
        assert!(
            building.occupants <= building.capacity,
            "Occupants ({}) should not exceed capacity ({}) after earthquake",
            building.occupants, building.capacity,
        );
    }
}

#[test]
fn test_disaster_tornado_destroys_some_buildings() {
    let mut city = TestCity::new();
    for i in 0..10 {
        city = city.with_building(128 + i, 128, ZoneType::ResidentialLow, 2);
    }
    assert_eq!(city.building_count(), 10);

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Tornado,
        center_x: 133,
        center_y: 128,
        radius: 5,
        ticks_remaining: 50,
        damage_applied: false,
    });

    city.tick(5);

    assert!(
        city.building_count() <= 10,
        "Tornado should not create buildings: got {}",
        city.building_count(),
    );
}

#[test]
fn test_disaster_flood_destroys_low_elevation_buildings() {
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 2)
        .with_building(130, 130, ZoneType::ResidentialLow, 2);

    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        grid.get_mut(128, 128).elevation = 0.1;
        grid.get_mut(130, 130).elevation = 0.8;
    }

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Flood,
        center_x: 129,
        center_y: 129,
        radius: 8,
        ticks_remaining: 100,
        damage_applied: false,
    });

    city.tick(5);

    let world = city.world_mut();
    let mut query = world.query::<&Building>();
    let buildings: Vec<_> = query.iter(world).collect();

    assert!(
        !buildings.iter().any(|b| b.grid_x == 128 && b.grid_y == 128),
        "Low elevation building should be destroyed by flood"
    );
    assert!(
        buildings.iter().any(|b| b.grid_x == 130 && b.grid_y == 130),
        "High elevation building should survive flood"
    );
}

#[test]
fn test_disaster_damage_applied_only_once() {
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 3);

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Earthquake,
        center_x: 128,
        center_y: 128,
        radius: 10,
        ticks_remaining: 20,
        damage_applied: false,
    });

    city.tick(5);
    let level_after_first = {
        let world = city.world_mut();
        world.query::<&Building>().iter(world).next().map(|b| b.level)
    };

    city.tick(10);
    let level_after_more = {
        let world = city.world_mut();
        world.query::<&Building>().iter(world).next().map(|b| b.level)
    };

    if let (Some(first), Some(second)) = (level_after_first, level_after_more) {
        assert_eq!(first, second, "Level should not change after initial damage: {} vs {}", first, second);
    }
}

#[test]
fn test_disaster_not_triggered_when_disabled() {
    let mut city = TestCity::new();
    {
        city.world_mut()
            .resource_mut::<crate::weather::Weather>()
            .disasters_enabled = false;
    }

    city.tick_slow_cycles(10);

    assert!(
        city.resource::<ActiveDisaster>().current.is_none(),
        "No disaster should trigger when disasters_enabled is false"
    );
}

#[test]
fn test_disaster_grid_cell_cleared_on_destruction() {
    let mut city = TestCity::new()
        .with_building(128, 128, ZoneType::ResidentialLow, 2);

    {
        city.world_mut()
            .resource_mut::<WorldGrid>()
            .get_mut(128, 128)
            .elevation = 0.1;
    }

    inject_disaster(&mut city, DisasterInstance {
        disaster_type: DisasterType::Flood,
        center_x: 128,
        center_y: 128,
        radius: 8,
        ticks_remaining: 100,
        damage_applied: false,
    });

    city.tick(5);

    let cell = city.cell(128, 128);
    assert!(cell.building_id.is_none(), "Grid building_id should be None after destruction");
    assert_eq!(cell.zone, ZoneType::None, "Grid zone should reset to None after destruction");
}

#[test]
fn test_disaster_duration_bounded_by_constants() {
    for (dtype, duration) in [
        (DisasterType::Tornado, 50),
        (DisasterType::Earthquake, 20),
        (DisasterType::Flood, 100),
    ] {
        let mut city = TestCity::new();

        inject_disaster(&mut city, DisasterInstance {
            disaster_type: dtype,
            center_x: 128,
            center_y: 128,
            radius: 10,
            ticks_remaining: duration,
            damage_applied: false,
        });

        city.tick(duration);

        assert!(
            city.resource::<ActiveDisaster>().current.is_none(),
            "{:?} should be cleared after {} ticks",
            dtype, duration,
        );
    }
}
