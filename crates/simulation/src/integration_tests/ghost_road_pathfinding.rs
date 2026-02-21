use crate::citizen::{CitizenState, CitizenStateComp, PathCache};
use crate::grid::{RoadType, ZoneType};
use crate::roads::RoadNode;
use crate::test_harness::TestCity;

#[test]
fn test_road_removal_invalidates_citizen_path_cache() {
    // Build a city with a straight road from (100,100) to (100,115)
    // and a citizen with home and work buildings.
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 115, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (100, 115));

    // Manually set the citizen to CommutingToWork with a path through
    // road nodes that includes (100, 105), which we will then delete.
    {
        let world = city.world_mut();
        let mut query = world.query::<(&mut PathCache, &mut CitizenStateComp)>();
        for (mut path, mut state) in query.iter_mut(world) {
            *path = PathCache::new(vec![
                RoadNode(100, 101),
                RoadNode(100, 102),
                RoadNode(100, 103),
                RoadNode(100, 104),
                RoadNode(100, 105),
                RoadNode(100, 106),
                RoadNode(100, 107),
            ]);
            state.0 = CitizenState::CommutingToWork;
        }
    }

    // Verify the citizen is commuting with a non-empty path
    assert_eq!(city.citizens_in_state(CitizenState::CommutingToWork), 1);

    // Bulldoze road cell (100, 105) -- this is in the middle of the path
    city.remove_road_at(100, 105);

    // Run one tick so the invalidation system fires
    city.tick(1);

    // The citizen should have been sent home because their path contained
    // a deleted road node.
    assert_eq!(
        city.citizens_in_state(CitizenState::CommutingToWork),
        0,
        "citizen should no longer be commuting after road deletion"
    );
    assert_eq!(
        city.citizens_in_state(CitizenState::AtHome),
        1,
        "citizen should be sent home after path invalidation"
    );

    // Verify the path cache was cleared
    {
        let world = city.world_mut();
        let mut query = world.query::<&PathCache>();
        for path in query.iter(world) {
            assert!(
                path.is_complete(),
                "path cache should be empty/complete after invalidation"
            );
        }
    }
}

#[test]
fn test_road_removal_does_not_affect_citizens_on_other_roads() {
    // Build a city with two separate roads
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 115, RoadType::Local)
        .with_road(120, 100, 120, 115, RoadType::Local)
        .with_building(120, 100, ZoneType::ResidentialLow, 1)
        .with_building(120, 115, ZoneType::CommercialLow, 1)
        .with_citizen((120, 100), (120, 115));

    // Set citizen path along the SECOND road (120, y)
    {
        let world = city.world_mut();
        let mut query = world.query::<(&mut PathCache, &mut CitizenStateComp)>();
        for (mut path, mut state) in query.iter_mut(world) {
            *path = PathCache::new(vec![
                RoadNode(120, 101),
                RoadNode(120, 102),
                RoadNode(120, 103),
                RoadNode(120, 104),
                RoadNode(120, 105),
            ]);
            state.0 = CitizenState::CommutingToWork;
        }
    }

    // Bulldoze a road cell on the FIRST road (100, 105) -- unrelated to citizen's path
    city.remove_road_at(100, 105);

    // Run one tick
    city.tick(1);

    // The citizen should still be commuting -- their path is on a different road
    assert_eq!(
        city.citizens_in_state(CitizenState::CommutingToWork),
        1,
        "citizen on unrelated road should still be commuting"
    );
}

#[test]
fn test_road_removal_only_affects_commuting_citizens() {
    // Build a city with a road and a citizen at home
    let mut city = TestCity::new()
        .with_road(100, 100, 100, 115, RoadType::Local)
        .with_building(100, 100, ZoneType::ResidentialLow, 1)
        .with_building(100, 115, ZoneType::CommercialLow, 1)
        .with_citizen((100, 100), (100, 115));

    // Citizen is AtHome with a stale path (leftover from previous trip).
    // This should NOT be affected by road removal since they are not commuting.
    {
        let world = city.world_mut();
        let mut query = world.query::<(&mut PathCache, &mut CitizenStateComp)>();
        for (mut path, mut state) in query.iter_mut(world) {
            *path = PathCache::new(vec![
                RoadNode(100, 103),
                RoadNode(100, 104),
                RoadNode(100, 105),
            ]);
            state.0 = CitizenState::AtHome;
        }
    }

    city.remove_road_at(100, 105);
    city.tick(1);

    // Should remain at home -- not affected because they aren't commuting
    assert_eq!(
        city.citizens_in_state(CitizenState::AtHome),
        1,
        "at-home citizen should not be affected by road removal"
    );
}
