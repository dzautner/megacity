//! Integration tests for issue #1605: job_seeking must not overfill building
//! capacity even when both `job_seeking` and `job_matching` systems run.
//!
//! The fix ensures that `job_matching` (education_jobs) increments
//! `building.occupants` when assigning workers, keeping the counter in sync
//! with what `job_seeking` (life_simulation) uses for capacity checks.

use crate::buildings::Building;
use crate::citizen::WorkLocation;
use crate::grid::ZoneType;
use crate::stats::CityStats;
use crate::test_harness::TestCity;
use bevy::prelude::*;

/// Core regression test for #1605: spawn far more unemployed citizens than
/// a single building can hold, run enough ticks for both job_seeking and
/// job_matching to fire, then verify occupants <= capacity.
#[test]
fn test_job_overfill_single_building() {
    let home_pos = (10, 10);
    let work_pos = (15, 15);

    let job_capacity = Building::capacity_for_level(ZoneType::Industrial, 1); // 20

    // Spawn 3x the capacity in unemployed citizens
    let num_citizens = (job_capacity as usize) * 3;
    let mut city = TestCity::new()
        .with_building(home_pos.0, home_pos.1, ZoneType::ResidentialLow, 3)
        .with_building(work_pos.0, work_pos.1, ZoneType::Industrial, 1);

    for _ in 0..num_citizens {
        city = city.with_unemployed_citizen(home_pos);
    }

    // Fill residential so citizen_spawner doesn't create extra employed citizens.
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.zone_type.is_residential() {
                building.occupants = building.capacity;
            }
        }
    }

    // Run enough ticks to trigger both job_seeking (interval=300) and
    // job_matching (interval=20). Run multiple cycles to test cross-tick
    // accumulation.
    for _ in 0..5 {
        city.world_mut()
            .resource_mut::<CityStats>()
            .average_happiness = 60.0;
        city.tick(301);
    }

    // Verify: no job building should have occupants > capacity
    let world = city.world_mut();
    let mut query = world.query::<&Building>();
    for building in query.iter(world) {
        if building.zone_type.is_job_zone() {
            assert!(
                building.occupants <= building.capacity,
                "Overfill! Building at ({}, {}) zone {:?} has {} occupants but capacity is {}",
                building.grid_x,
                building.grid_y,
                building.zone_type,
                building.occupants,
                building.capacity,
            );
        }
    }
}

/// Verify that the number of WorkLocation components pointing at a building
/// never exceeds its capacity after multiple tick cycles.
#[test]
fn test_work_location_count_respects_capacity() {
    let home_pos = (10, 10);
    let work_pos = (20, 20);

    let job_capacity = Building::capacity_for_level(ZoneType::Industrial, 1); // 20
    let num_citizens = (job_capacity as usize) * 4;

    let mut city = TestCity::new()
        .with_building(home_pos.0, home_pos.1, ZoneType::ResidentialHigh, 3)
        .with_building(work_pos.0, work_pos.1, ZoneType::Industrial, 1);

    for _ in 0..num_citizens {
        city = city.with_unemployed_citizen(home_pos);
    }

    // Fill residential
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.zone_type.is_residential() {
                building.occupants = building.capacity;
            }
        }
    }

    // Run many tick cycles
    for _ in 0..10 {
        city.world_mut()
            .resource_mut::<CityStats>()
            .average_happiness = 60.0;
        city.tick(100);
    }

    // Count WorkLocations per building entity
    let world = city.world_mut();
    let mut building_query = world.query::<(Entity, &Building)>();
    let buildings: Vec<(Entity, u32)> = building_query
        .iter(world)
        .filter(|(_, b)| b.zone_type.is_job_zone())
        .map(|(e, b)| (e, b.capacity))
        .collect();

    let mut work_query = world.query::<&WorkLocation>();
    for (building_entity, capacity) in &buildings {
        let assigned = work_query
            .iter(world)
            .filter(|wl| wl.building == *building_entity)
            .count() as u32;
        assert!(
            assigned <= *capacity,
            "WorkLocation overfill! Building {:?} has {} workers assigned but capacity is {}",
            building_entity,
            assigned,
            capacity,
        );
    }
}

/// Test with multiple building types to ensure the invariant holds across
/// different zone types that both job systems interact with.
#[test]
fn test_job_overfill_multiple_building_types() {
    let home_pos = (10, 10);

    let mut city = TestCity::new()
        .with_building(home_pos.0, home_pos.1, ZoneType::ResidentialHigh, 3)
        .with_building(20, 10, ZoneType::Industrial, 1)   // cap 20
        .with_building(25, 10, ZoneType::CommercialLow, 1) // cap 8
        .with_building(30, 10, ZoneType::Office, 1);       // cap 30

    // Spawn many unemployed citizens (more than total capacity of all job buildings)
    for _ in 0..120 {
        city = city.with_unemployed_citizen(home_pos);
    }

    // Fill residential
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.zone_type.is_residential() {
                building.occupants = building.capacity;
            }
        }
    }

    // Run enough cycles for both job systems to fire repeatedly
    for _ in 0..8 {
        city.world_mut()
            .resource_mut::<CityStats>()
            .average_happiness = 60.0;
        city.tick(301);
    }

    // Verify invariant for all job buildings
    let world = city.world_mut();
    let mut query = world.query::<&Building>();
    for building in query.iter(world) {
        if building.zone_type.is_job_zone() {
            assert!(
                building.occupants <= building.capacity,
                "Overfill! {:?} at ({},{}) has {}/{} occupants",
                building.zone_type,
                building.grid_x,
                building.grid_y,
                building.occupants,
                building.capacity,
            );
        }
    }
}
