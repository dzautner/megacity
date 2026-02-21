use crate::grid::{RoadType, ZoneType};
use crate::road_segments::RoadSegmentStore;
use crate::test_harness::TestCity;

// ====================================================================
// Mode Choice (TRAF-007) integration tests
// ====================================================================

#[test]
fn test_mode_choice_resource_initialized() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::mode_choice::ModeShareStats>();
    city.assert_resource_exists::<crate::mode_choice::ModeInfrastructureCache>();
}

#[test]
fn test_mode_choice_default_stats() {
    let city = TestCity::new();
    let stats = city.resource::<crate::mode_choice::ModeShareStats>();
    assert_eq!(stats.total(), 0);
    // Default: 100% drive when no trips active
    assert!((stats.drive_pct - 100.0).abs() < f32::EPSILON);
}

#[test]
fn test_mode_choice_citizen_has_component() {
    use crate::mode_choice::ChosenTransportMode;
    use bevy::prelude::Entity;

    let mut city = TestCity::new()
        .with_road(100, 128, 110, 128, RoadType::Local)
        .with_building(101, 127, ZoneType::ResidentialLow, 1)
        .with_building(109, 127, ZoneType::CommercialLow, 1)
        .with_citizen((101, 127), (109, 127));

    // Verify the citizen has a ChosenTransportMode component
    let world = city.world_mut();
    let count = world
        .query_filtered::<Entity, bevy::prelude::With<ChosenTransportMode>>()
        .iter(world)
        .count();
    assert_eq!(
        count, 1,
        "citizen should have ChosenTransportMode component"
    );
}

#[test]
fn test_bicycle_lanes_add_lane_to_road_segment() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_budget(100_000.0);

    // Find the segment ID
    let seg_id = {
        let store = city.resource::<RoadSegmentStore>();
        assert!(
            !store.segments.is_empty(),
            "should have at least one road segment"
        );
        store.segments[0].id
    };

    // Add bike lane
    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.add_bike_lane(seg_id);
    }

    city.tick_slow_cycle();

    let coverage = city.resource::<crate::bicycle_lanes::BicycleCoverageGrid>();
    assert!(
        coverage.city_average > 0.0,
        "city with bike lane should have nonzero cycling coverage, got {}",
        coverage.city_average
    );
}

#[test]
fn test_bicycle_lanes_mode_share_positive_with_infrastructure() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_budget(100_000.0);

    // Add bike lane to the road
    let seg_id = {
        let store = city.resource::<RoadSegmentStore>();
        store.segments[0].id
    };

    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.add_bike_lane(seg_id);
    }

    city.tick_slow_cycle();

    let coverage = city.resource::<crate::bicycle_lanes::BicycleCoverageGrid>();
    assert!(
        coverage.cycling_mode_share > 0.0,
        "cycling mode share should be positive with bike lanes, got {}",
        coverage.cycling_mode_share
    );
}

#[test]
fn test_bicycle_lanes_remove_lane_drops_coverage() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_budget(100_000.0);

    let seg_id = {
        let store = city.resource::<RoadSegmentStore>();
        store.segments[0].id
    };

    // Add and verify
    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.add_bike_lane(seg_id);
    }
    city.tick_slow_cycle();

    let coverage_with = city
        .resource::<crate::bicycle_lanes::BicycleCoverageGrid>()
        .city_average;
    assert!(coverage_with > 0.0);

    // Remove and verify drop
    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.remove_bike_lane(seg_id);
    }
    city.tick_slow_cycle();

    let coverage_without = city
        .resource::<crate::bicycle_lanes::BicycleCoverageGrid>()
        .city_average;
    assert!(
        coverage_without < coverage_with,
        "removing bike lane should reduce coverage: with={}, without={}",
        coverage_with,
        coverage_without
    );
}

#[test]
fn test_bicycle_lanes_maintenance_cost() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Local)
        .with_budget(100_000.0);

    let seg_id = {
        let store = city.resource::<RoadSegmentStore>();
        store.segments[0].id
    };

    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.add_bike_lane(seg_id);
    }

    city.tick_slow_cycle();

    let coverage = city.resource::<crate::bicycle_lanes::BicycleCoverageGrid>();
    assert!(
        coverage.total_maintenance_cost > 0.0,
        "bike lanes should have positive maintenance cost, got {}",
        coverage.total_maintenance_cost
    );
}

#[test]
fn test_bicycle_lanes_unsupported_road_type_ignored() {
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Highway)
        .with_budget(100_000.0);

    let seg_id = {
        let store = city.resource::<RoadSegmentStore>();
        store.segments[0].id
    };

    // Add bike lane to highway (should not produce coverage)
    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.add_bike_lane(seg_id);
    }

    city.tick_slow_cycle();

    let bike_state = city.resource::<crate::bicycle_lanes::BicycleLaneState>();
    assert!(
        bike_state.has_bike_lane(seg_id),
        "bike lane flag should be set even on unsupported type"
    );

    // The coverage system checks supports_bike_lane(), so highway bike lanes
    // don't contribute to coverage. But Path cells may still contribute.
    // This test just verifies no panic occurs.
}

#[test]
fn test_bicycle_lanes_encourage_biking_policy_boost() {
    let mut city = TestCity::new()
        .with_road(50, 50, 70, 50, RoadType::Local)
        .with_budget(100_000.0);

    let seg_id = {
        let store = city.resource::<RoadSegmentStore>();
        store.segments[0].id
    };

    {
        let world = city.world_mut();
        let mut bike_state = world
            .get_resource_mut::<crate::bicycle_lanes::BicycleLaneState>()
            .unwrap();
        bike_state.add_bike_lane(seg_id);
    }

    // Measure without policy
    city.tick_slow_cycle();
    let share_without = city
        .resource::<crate::bicycle_lanes::BicycleCoverageGrid>()
        .cycling_mode_share;

    // Enable Encourage Biking policy
    {
        let world = city.world_mut();
        let mut policies = world
            .get_resource_mut::<crate::policies::Policies>()
            .unwrap();
        policies.toggle(crate::policies::Policy::EncourageBiking);
    }

    city.tick_slow_cycle();
    let share_with = city
        .resource::<crate::bicycle_lanes::BicycleCoverageGrid>()
        .cycling_mode_share;

    assert!(
        share_with > share_without,
        "Encourage Biking policy should increase cycling mode share: \
         without={}, with={}",
        share_without,
        share_with
    );
}

#[test]
fn test_bicycle_lanes_saveable_roundtrip() {
    use crate::bicycle_lanes::BicycleLaneState;
    use crate::road_segments::SegmentId;
    use crate::Saveable;

    let mut state = BicycleLaneState::default();
    state.add_bike_lane(SegmentId(10));
    state.add_bike_lane(SegmentId(20));

    let bytes = state.save_to_bytes().expect("non-empty should save");
    let restored = BicycleLaneState::load_from_bytes(&bytes);

    assert_eq!(restored.lane_count(), 2);
    assert!(restored.has_bike_lane(SegmentId(10)));
    assert!(restored.has_bike_lane(SegmentId(20)));
}

#[test]
fn test_bicycle_lanes_path_road_gives_implicit_coverage() {
    // Path roads are pedestrian/bike paths — they should provide coverage
    // even without explicitly adding bike lanes
    let mut city = TestCity::new()
        .with_road(50, 50, 60, 50, RoadType::Path)
        .with_budget(100_000.0);

    city.tick_slow_cycle();

    let coverage = city.resource::<crate::bicycle_lanes::BicycleCoverageGrid>();
    assert!(
        coverage.city_average > 0.0,
        "Path roads should provide implicit bike coverage, got {}",
        coverage.city_average
    );
}
