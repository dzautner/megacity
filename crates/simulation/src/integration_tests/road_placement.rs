use crate::grid::RoadType;
use crate::test_harness::TestCity;

#[test]
fn road_placement_creates_road_cells() {
    let city = TestCity::new().with_road(100, 100, 100, 110, RoadType::Local);

    let road_count = city.road_cell_count();
    assert!(
        road_count > 0,
        "placing a road should create road cells, got {road_count}"
    );
}

#[test]
fn road_placement_creates_road_nodes_in_network() {
    let city = TestCity::new().with_road(100, 100, 100, 110, RoadType::Local);

    let network = city.road_network();
    assert!(
        !network.edges.is_empty(),
        "placing a road should add nodes to the RoadNetwork"
    );
}

#[test]
fn road_placement_creates_segments() {
    let city = TestCity::new().with_road(100, 100, 100, 110, RoadType::Local);

    let segments = city.road_segments();
    assert!(
        !segments.segments.is_empty(),
        "placing a road should create road segments"
    );
}

#[test]
fn road_cells_are_connected_in_network() {
    let city = TestCity::new().with_road(100, 100, 110, 100, RoadType::Local);

    let network = city.road_network();
    let connected_nodes = network
        .edges
        .values()
        .filter(|neighbors| !neighbors.is_empty())
        .count();
    assert!(
        connected_nodes > 0,
        "road nodes should be connected to each other"
    );
}

#[test]
fn different_road_types_create_correct_cells() {
    for road_type in [
        RoadType::Local,
        RoadType::Avenue,
        RoadType::Boulevard,
        RoadType::Highway,
    ] {
        let city = TestCity::new().with_road(100, 50, 100, 60, road_type);

        let road_count = city.road_cell_count();
        assert!(
            road_count > 0,
            "road type {:?} should create road cells, got {road_count}",
            road_type
        );
    }
}

#[test]
fn multiple_roads_form_grid() {
    let city = TestCity::new()
        .with_road(100, 100, 120, 100, RoadType::Local)
        .with_road(110, 95, 110, 105, RoadType::Local);

    let road_count = city.road_cell_count();
    assert!(
        road_count > 15,
        "two intersecting roads should create many road cells, got {road_count}"
    );
}
