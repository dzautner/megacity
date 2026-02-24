//! Unit tests for lane-marking geometry generation.

use bevy::prelude::*;

use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{RoadType, WorldGrid};

use super::line_primitives::{bezier_tangent, eval_bezier, road_half_width};
use super::mesh_builder::build_lane_marking_mesh;

/// Create a flat grid for tests (all elevation = 0.5).
fn test_grid() -> WorldGrid {
    WorldGrid::new(GRID_WIDTH, GRID_HEIGHT)
}

#[test]
fn test_road_half_width_values() {
    assert!((road_half_width(RoadType::Path) - 1.5).abs() < f32::EPSILON);
    assert!((road_half_width(RoadType::OneWay) - 3.0).abs() < f32::EPSILON);
    assert!((road_half_width(RoadType::Local) - 4.0).abs() < f32::EPSILON);
    assert!((road_half_width(RoadType::Avenue) - 6.0).abs() < f32::EPSILON);
    assert!((road_half_width(RoadType::Boulevard) - 8.0).abs() < f32::EPSILON);
    assert!((road_half_width(RoadType::Highway) - 10.0).abs() < f32::EPSILON);
}

#[test]
fn test_eval_bezier_endpoints() {
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(100.0, 0.0);
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    let start = eval_bezier(p0, p1, p2, p3, 0.0);
    let end = eval_bezier(p0, p1, p2, p3, 1.0);
    assert!((start - p0).length() < 0.01);
    assert!((end - p3).length() < 0.01);
}

#[test]
fn test_bezier_tangent_straight_line() {
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(100.0, 0.0);
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    let tang = bezier_tangent(p0, p1, p2, p3, 0.5);
    let normalised = tang.normalize();
    assert!((normalised.x - 1.0).abs() < 0.01);
    assert!(normalised.y.abs() < 0.01);
}

#[test]
fn test_build_avenue_marking_mesh_not_empty() {
    let grid = test_grid();
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(100.0, 0.0);
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    let mesh = build_lane_marking_mesh(
        p0,
        p1,
        p2,
        p3,
        RoadType::Avenue,
        road_half_width(RoadType::Avenue),
        100.0,
        0.0,
        0.0,
        &grid,
    );

    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    assert!(
        positions.len() > 0,
        "avenue marking mesh should have vertices"
    );
}

#[test]
fn test_build_boulevard_marking_mesh_not_empty() {
    let grid = test_grid();
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(200.0, 0.0);
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    let mesh = build_lane_marking_mesh(
        p0,
        p1,
        p2,
        p3,
        RoadType::Boulevard,
        road_half_width(RoadType::Boulevard),
        200.0,
        0.0,
        0.0,
        &grid,
    );

    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    assert!(
        positions.len() > 0,
        "boulevard marking mesh should have vertices"
    );
}

#[test]
fn test_build_highway_marking_mesh_not_empty() {
    let grid = test_grid();
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(300.0, 0.0);
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    let mesh = build_lane_marking_mesh(
        p0,
        p1,
        p2,
        p3,
        RoadType::Highway,
        road_half_width(RoadType::Highway),
        300.0,
        0.0,
        0.0,
        &grid,
    );

    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    assert!(
        positions.len() > 0,
        "highway marking mesh should have vertices"
    );
}

#[test]
fn test_local_road_produces_dashed_center_line() {
    let grid = test_grid();
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(100.0, 0.0);
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    let mesh = build_lane_marking_mesh(
        p0,
        p1,
        p2,
        p3,
        RoadType::Local,
        road_half_width(RoadType::Local),
        100.0,
        0.0,
        0.0,
        &grid,
    );

    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    assert!(
        positions.len() > 0,
        "local road should have dashed center line vertices"
    );
}

#[test]
fn test_oneway_road_produces_dashed_center_line() {
    let grid = test_grid();
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(80.0, 0.0);
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    let mesh = build_lane_marking_mesh(
        p0,
        p1,
        p2,
        p3,
        RoadType::OneWay,
        road_half_width(RoadType::OneWay),
        80.0,
        0.0,
        0.0,
        &grid,
    );

    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    assert!(
        positions.len() > 0,
        "one-way road should have dashed center line vertices"
    );
}

#[test]
fn test_path_produces_no_marking_mesh() {
    let grid = test_grid();
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(50.0, 0.0);
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    let mesh = build_lane_marking_mesh(
        p0,
        p1,
        p2,
        p3,
        RoadType::Path,
        road_half_width(RoadType::Path),
        50.0,
        0.0,
        0.0,
        &grid,
    );

    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    assert_eq!(positions.len(), 0, "path marking mesh should be empty");
}

#[test]
fn test_trim_reduces_vertex_count() {
    let grid = test_grid();
    let p0 = Vec2::new(0.0, 0.0);
    let p3 = Vec2::new(200.0, 0.0);
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;
    let hw = road_half_width(RoadType::Highway);

    let mesh_no_trim = build_lane_marking_mesh(
        p0,
        p1,
        p2,
        p3,
        RoadType::Highway,
        hw,
        200.0,
        0.0,
        0.0,
        &grid,
    );
    let mesh_trimmed = build_lane_marking_mesh(
        p0,
        p1,
        p2,
        p3,
        RoadType::Highway,
        hw,
        200.0,
        hw * 1.2,
        hw * 1.2,
        &grid,
    );

    let count_no_trim = mesh_no_trim
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .map(|a| a.len())
        .unwrap_or(0);
    let count_trimmed = mesh_trimmed
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .map(|a| a.len())
        .unwrap_or(0);

    assert!(
        count_trimmed <= count_no_trim,
        "trimmed mesh should have <= vertices ({count_trimmed} vs {count_no_trim})"
    );
}
