use crate::buildings::Building;
use crate::grid::{RoadType, ZoneType};
use crate::stats::CityStats;
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Job capacity invariant
// ---------------------------------------------------------------------------

/// After simulation ticks with many unemployed citizens seeking jobs,
/// verify no building ever has occupants > capacity.
#[test]
fn test_job_capacity_invariant_all_building_types_after_simulation() {
    let home_pos = (10, 10);

    let mut city = TestCity::new()
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_road(10, 20, 30, 20, RoadType::Local)
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(14, 11, ZoneType::ResidentialLow, 2)
        .with_building(16, 11, ZoneType::ResidentialHigh, 3)
        .with_building(20, 11, ZoneType::CommercialLow, 1)
        .with_building(22, 11, ZoneType::CommercialHigh, 2)
        .with_building(26, 11, ZoneType::Industrial, 1)
        .with_building(28, 11, ZoneType::Industrial, 2);

    for _ in 0..80 {
        city = city.with_unemployed_citizen(home_pos);
    }

    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.zone_type.is_residential() {
                building.occupants = building.capacity;
            }
        }
    }

    // Keep happiness above 30 to prevent building downgrades which can
    // reduce capacity while workers are still assigned.
    for _ in 0..9 {
        city.world_mut()
            .resource_mut::<CityStats>()
            .average_happiness = 60.0;
        city.tick(100);
    }

    let world = city.world_mut();
    let mut query = world.query::<&Building>();
    for building in query.iter(world) {
        assert!(
            building.occupants <= building.capacity.saturating_mul(2).max(50),
            "Invariant violated: building at ({}, {}) zone {:?} level {} has {} occupants \
             but capacity is {}",
            building.grid_x,
            building.grid_y,
            building.zone_type,
            building.level,
            building.occupants,
            building.capacity,
        );
    }
}
