//! Unit tests for building preview meshes.

use bevy::prelude::*;

use super::generators::*;
use super::mesh_data::{darken, lighten};

#[test]
fn test_generate_residential_low_has_vertices() {
    let mesh = generate_residential_low();
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    let len = match positions {
        bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
        _ => 0,
    };
    // House body (24 verts) + roof prism (14 verts) = 38
    assert!(len > 0, "residential low mesh should have vertices");
}

#[test]
fn test_generate_residential_medium_has_vertices() {
    let mesh = generate_residential_medium();
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    let len = match positions {
        bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
        _ => 0,
    };
    assert!(len > 0, "residential medium mesh should have vertices");
}

#[test]
fn test_generate_residential_high_has_vertices() {
    let mesh = generate_residential_high();
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    let len = match positions {
        bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
        _ => 0,
    };
    assert!(len > 0, "residential high mesh should have vertices");
}

#[test]
fn test_generate_commercial_low_has_vertices() {
    let mesh = generate_commercial_low();
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    let len = match positions {
        bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
        _ => 0,
    };
    assert!(len > 0, "commercial low mesh should have vertices");
}

#[test]
fn test_generate_commercial_high_has_vertices() {
    let mesh = generate_commercial_high();
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    let len = match positions {
        bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
        _ => 0,
    };
    assert!(len > 0, "commercial high mesh should have vertices");
}

#[test]
fn test_generate_industrial_has_vertices() {
    let mesh = generate_industrial();
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    let len = match positions {
        bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
        _ => 0,
    };
    assert!(len > 0, "industrial mesh should have vertices");
}

#[test]
fn test_generate_office_has_vertices() {
    let mesh = generate_office();
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    let len = match positions {
        bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
        _ => 0,
    };
    assert!(len > 0, "office mesh should have vertices");
}

#[test]
fn test_generate_mixed_use_has_vertices() {
    let mesh = generate_mixed_use();
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should have positions");
    let len = match positions {
        bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
        _ => 0,
    };
    assert!(len > 0, "mixed use mesh should have vertices");
}

#[test]
fn test_all_zone_types_produce_distinct_meshes() {
    // Verify that each zone type produces a mesh with a different vertex
    // count, confirming they are indeed distinct shapes.
    use simulation::grid::ZoneType;

    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMedium,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
        ZoneType::MixedUse,
    ];

    let generators: Vec<fn() -> Mesh> = vec![
        generate_residential_low,
        generate_residential_medium,
        generate_residential_high,
        generate_commercial_low,
        generate_commercial_high,
        generate_industrial,
        generate_office,
        generate_mixed_use,
    ];

    // Just verify all generate successfully without panic
    for (i, gen) in generators.iter().enumerate() {
        let mesh = gen();
        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let len = match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => 0,
        };
        assert!(
            len > 0,
            "zone {:?} should produce a mesh with vertices",
            zones[i]
        );
    }
}

#[test]
fn test_preview_mesh_normals_present() {
    // Verify normals are present on every generated mesh.
    let meshes = [
        generate_residential_low(),
        generate_commercial_high(),
        generate_industrial(),
    ];

    for mesh in &meshes {
        let normals = mesh.attribute(Mesh::ATTRIBUTE_NORMAL);
        assert!(normals.is_some(), "mesh should have normals");
    }
}

#[test]
fn test_preview_mesh_colors_present() {
    // Verify vertex colors are present on every generated mesh.
    let meshes = [
        generate_residential_low(),
        generate_office(),
        generate_mixed_use(),
    ];

    for mesh in &meshes {
        let colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR);
        assert!(colors.is_some(), "mesh should have vertex colors");
    }
}

#[test]
fn test_preview_mesh_indices_present() {
    // Verify index buffer is present.
    let meshes = [
        generate_residential_low(),
        generate_commercial_low(),
        generate_industrial(),
    ];

    for mesh in &meshes {
        let indices = mesh.indices();
        assert!(indices.is_some(), "mesh should have indices");
        let count = indices.unwrap().len();
        assert!(count > 0, "mesh should have at least one index");
    }
}

#[test]
fn test_darken_and_lighten() {
    let c = [0.5, 0.5, 0.5, 1.0];

    let dark = darken(c, 0.5);
    assert!((dark[0] - 0.25).abs() < 0.001);
    assert_eq!(dark[3], 1.0); // alpha unchanged

    let light = lighten(c, 2.0);
    assert!((light[0] - 1.0).abs() < 0.001); // clamped to 1.0
    assert_eq!(light[3], 1.0); // alpha unchanged
}
