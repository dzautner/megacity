use crate::grid::{RoadType, WorldGrid, ZoneType};
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// SimulationSet phase ordering
// ---------------------------------------------------------------------------

/// Verify that the SimulationSet and SimulationUpdateSet phase ordering is
/// correctly configured by running a full tick.  If the set chain is broken
/// Bevy would panic with an ambiguity error or the systems would not run.
#[test]
fn test_simulation_set_phases_configured() {
    use crate::test_harness::TestCity;

    // Build a minimal city and run a few ticks.  If the phase ordering is
    // misconfigured (e.g. circular dependency, missing configure_sets) this
    // will panic.
    let mut city = TestCity::new()
        .with_road(128, 128, 128, 131, RoadType::Local)
        .with_zone(129, 128, ZoneType::ResidentialLow)
        .with_zone(129, 130, ZoneType::CommercialLow);
    city.tick(5);

    // Sanity: game clock should have advanced (PreSim systems ran)
    assert!(city.clock().hour > 6.0 || city.clock().day > 1);
}

#[test]
fn test_traffic_los_resource_initialized() {
    use crate::traffic_los::{LosGrade, TrafficLosGrid};

    let city = TestCity::new();

    // The TrafficLosPlugin should register the TrafficLosGrid resource
    let los = city.resource::<TrafficLosGrid>();
    assert_eq!(
        los.get(0, 0),
        LosGrade::A,
        "Default LOS should be A (free flow)"
    );
}

#[test]
fn test_traffic_los_empty_roads_grade_a() {
    use crate::traffic_los::{LosGrade, TrafficLosGrid};

    let mut city = TestCity::new().with_road(10, 10, 20, 10, RoadType::Local);

    // Run enough ticks for the LOS system to fire (runs every 10 ticks)
    city.tick(10);

    // With no citizens commuting, traffic density is 0, so roads should be LOS A
    let los = city.resource::<TrafficLosGrid>();
    assert_eq!(
        los.get(15, 10),
        LosGrade::A,
        "Empty road should be LOS A (free flow)"
    );
}

#[test]
fn test_traffic_los_grading_uses_road_type_capacity() {
    use crate::traffic_los::LosGrade;

    let city = TestCity::new()
        .with_road(10, 10, 20, 10, RoadType::Local)
        .with_road(10, 15, 20, 15, RoadType::Highway);

    // Verify that the road types have different capacities (needed for LOS)
    let grid = city.resource::<WorldGrid>();
    let local_capacity = grid.get(15, 10).road_type.capacity();
    let highway_capacity = grid.get(15, 15).road_type.capacity();
    assert!(
        highway_capacity > local_capacity,
        "Highway capacity ({highway_capacity}) should exceed Local capacity ({local_capacity})"
    );

    // Verify that the LOS grading function correctly distinguishes load levels
    // Same traffic volume on different road types yields different grades
    let traffic_volume = local_capacity as f32; // saturate local road
    let local_vc = traffic_volume / local_capacity as f32; // 1.0 -> LOS F
    let highway_vc = traffic_volume / highway_capacity as f32; // < 1.0

    let local_grade = LosGrade::from_vc_ratio(local_vc);
    let highway_grade = LosGrade::from_vc_ratio(highway_vc);

    assert_eq!(
        local_grade,
        LosGrade::F,
        "Local at capacity should be LOS F"
    );
    assert!(
        (highway_grade as u8) < (local_grade as u8),
        "Highway ({highway_grade:?}) should have better LOS than Local ({local_grade:?}) at same traffic volume"
    );
}
