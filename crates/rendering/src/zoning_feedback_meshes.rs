//! Procedural meshes for zone utility badge icons.
//!
//! Each mesh is a flat silhouette in the XZ plane (Y = 0), suitable for
//! standing upright in 3D space.  The meshes use `TriangleList` topology
//! and are double-sided (rendered without backface culling).

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Fan-triangulate a convex or simple polygon given as ordered vertices in XZ.
fn fan_triangulate(positions: &[[f32; 3]]) -> Vec<u32> {
    let mut indices = Vec::new();
    for i in 1..(positions.len() as u32 - 1) {
        indices.push(0);
        indices.push(i);
        indices.push(i + 1);
    }
    indices
}

fn build_flat_mesh(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> Mesh {
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    let uvs = vec![[0.0, 0.0]; positions.len()];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

// ---------------------------------------------------------------------------
// Lightning bolt (NoPower)
// ---------------------------------------------------------------------------

/// Zigzag lightning bolt silhouette, ~8 vertices.
/// Extends roughly from (-1, 0, -2) to (1, 0, 2) in XZ.
pub fn build_lightning_bolt_mesh() -> Mesh {
    // Bolt shape: a stylized zigzag
    #[rustfmt::skip]
    let positions: Vec<[f32; 3]> = vec![
        // Upper part of bolt
        [ 0.0, 0.0, -2.0],  // 0  top point
        [ 0.8, 0.0, -0.6],  // 1  right notch upper
        [ 0.0, 0.0, -0.2],  // 2  center notch
        [ 1.0, 0.0,  0.0],  // 3  right bulge
        [ 0.0, 0.0,  2.0],  // 4  bottom point
        [-0.8, 0.0,  0.6],  // 5  left notch lower
        [ 0.0, 0.0,  0.2],  // 6  center notch
        [-1.0, 0.0,  0.0],  // 7  left bulge
    ];

    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        0, 1, 2,
        2, 1, 3,
        2, 3, 4,
        4, 5, 6,
        6, 5, 7,
        6, 7, 0,
        0, 2, 6,
    ];

    build_flat_mesh(positions, indices)
}

// ---------------------------------------------------------------------------
// Water drop (NoWater)
// ---------------------------------------------------------------------------

/// Teardrop silhouette, ~12 vertices around the outline.
/// Extends roughly from (-1, 0, -2) to (1, 0, 1.5) in XZ.
pub fn build_water_drop_mesh() -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();

    // Top point of the teardrop
    positions.push([0.0, 0.0, -2.0]); // 0: tip

    // Rounded bottom (semicircle), 10 points from left to right
    let segments = 10;
    let radius = 1.0_f32;
    let center_z = 0.5_f32;
    for i in 0..=segments {
        let angle =
            std::f32::consts::PI * (i as f32 / segments as f32) + std::f32::consts::FRAC_PI_2;
        let x = radius * angle.cos();
        let z = center_z + radius * angle.sin();
        positions.push([x, 0.0, z]);
    }

    // Fan triangulate from tip
    let indices = fan_triangulate(&positions);

    build_flat_mesh(positions, indices)
}

// ---------------------------------------------------------------------------
// Warning triangle (NoPowerNoWater)
// ---------------------------------------------------------------------------

/// Equilateral-ish warning triangle.
/// Extends roughly from (-1.2, 0, -2) to (1.2, 0, 1) in XZ.
pub fn build_warning_mesh() -> Mesh {
    #[rustfmt::skip]
    let positions: Vec<[f32; 3]> = vec![
        [ 0.0, 0.0, -2.0],   // 0  top
        [ 1.2, 0.0,  1.0],   // 1  bottom-right
        [-1.2, 0.0,  1.0],   // 2  bottom-left
    ];

    let indices = vec![0u32, 1, 2];

    build_flat_mesh(positions, indices)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lightning_bolt_mesh_valid() {
        let mesh = build_lightning_bolt_mesh();
        assert!(mesh.count_vertices() >= 8);
    }

    #[test]
    fn test_water_drop_mesh_valid() {
        let mesh = build_water_drop_mesh();
        assert!(mesh.count_vertices() >= 10);
    }

    #[test]
    fn test_warning_mesh_valid() {
        let mesh = build_warning_mesh();
        assert_eq!(mesh.count_vertices(), 3);
    }
}
