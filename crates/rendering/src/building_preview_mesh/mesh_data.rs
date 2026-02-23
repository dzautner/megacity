//! Low-level mesh builder for building preview meshes.
//!
//! Provides [`PreviewMeshData`], a simple vertex+index accumulator with
//! helpers for common primitives (cuboid, roof prism), plus color utilities
//! ([`lighten`] / [`darken`]).

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

pub(crate) fn lighten(c: [f32; 4], factor: f32) -> [f32; 4] {
    [
        (c[0] * factor).min(1.0),
        (c[1] * factor).min(1.0),
        (c[2] * factor).min(1.0),
        c[3],
    ]
}

pub(crate) fn darken(c: [f32; 4], factor: f32) -> [f32; 4] {
    [c[0] * factor, c[1] * factor, c[2] * factor, c[3]]
}

// ---------------------------------------------------------------------------
// PreviewMeshData
// ---------------------------------------------------------------------------

/// Simple mesh builder that accumulates vertices with position, normal, and color.
#[derive(Default)]
pub(crate) struct PreviewMeshData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl PreviewMeshData {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn into_mesh(self) -> Mesh {
        let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; self.positions.len()];
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(self.indices))
    }

    /// Add a cuboid (box) centered at (cx, cy, cz) with half-extents (hw, hh, hd).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_cuboid(
        &mut self,
        cx: f32,
        cy: f32,
        cz: f32,
        hw: f32,
        hh: f32,
        hd: f32,
        color: [f32; 4],
    ) {
        let base = self.positions.len() as u32;
        let x0 = cx - hw;
        let x1 = cx + hw;
        let y0 = cy - hh;
        let y1 = cy + hh;
        let z0 = cz - hd;
        let z1 = cz + hd;

        // Front face (+Z)
        self.positions
            .extend_from_slice(&[[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]]);
        self.normals.extend_from_slice(&[[0.0, 0.0, 1.0]; 4]);
        self.colors.extend_from_slice(&[darken(color, 0.85); 4]);
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        // Back face (-Z)
        let b = base + 4;
        self.positions
            .extend_from_slice(&[[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]]);
        self.normals.extend_from_slice(&[[0.0, 0.0, -1.0]; 4]);
        self.colors.extend_from_slice(&[darken(color, 0.75); 4]);
        self.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);

        // Top face (+Y)
        let b = base + 8;
        self.positions
            .extend_from_slice(&[[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]]);
        self.normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
        self.colors.extend_from_slice(&[lighten(color, 1.15); 4]);
        self.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);

        // Bottom face (-Y)
        let b = base + 12;
        self.positions
            .extend_from_slice(&[[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]]);
        self.normals.extend_from_slice(&[[0.0, -1.0, 0.0]; 4]);
        self.colors.extend_from_slice(&[darken(color, 0.5); 4]);
        self.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);

        // Right face (+X)
        let b = base + 16;
        self.positions
            .extend_from_slice(&[[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]]);
        self.normals.extend_from_slice(&[[1.0, 0.0, 0.0]; 4]);
        self.colors.extend_from_slice(&[darken(color, 0.7); 4]);
        self.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);

        // Left face (-X)
        let b = base + 20;
        self.positions
            .extend_from_slice(&[[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]]);
        self.normals.extend_from_slice(&[[-1.0, 0.0, 0.0]; 4]);
        self.colors.extend_from_slice(&[darken(color, 0.65); 4]);
        self.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }

    /// Add a triangular roof prism centered at (cx, cy, cz) with the ridge
    /// running along the Z axis.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_roof_prism(
        &mut self,
        cx: f32,
        cy: f32,
        cz: f32,
        hw: f32,
        hh: f32,
        hd: f32,
        color: [f32; 4],
    ) {
        let base = self.positions.len() as u32;
        let x0 = cx - hw;
        let x1 = cx + hw;
        let z0 = cz - hd;
        let z1 = cz + hd;
        let peak_y = cy + hh;

        // Front gable (+Z)
        self.positions
            .extend_from_slice(&[[x0, cy, z1], [x1, cy, z1], [cx, peak_y, z1]]);
        self.normals.extend_from_slice(&[[0.0, 0.0, 1.0]; 3]);
        self.colors.extend_from_slice(&[color; 3]);
        self.indices.extend_from_slice(&[base, base + 1, base + 2]);

        // Back gable (-Z)
        let b = base + 3;
        self.positions
            .extend_from_slice(&[[x1, cy, z0], [x0, cy, z0], [cx, peak_y, z0]]);
        self.normals.extend_from_slice(&[[0.0, 0.0, -1.0]; 3]);
        self.colors.extend_from_slice(&[color; 3]);
        self.indices.extend_from_slice(&[b, b + 1, b + 2]);

        // Left slope
        let b = base + 6;
        let n_left = Vec3::new(-hh, hw, 0.0).normalize();
        let nl = [n_left.x, n_left.y, n_left.z];
        self.positions.extend_from_slice(&[
            [x0, cy, z0],
            [x0, cy, z1],
            [cx, peak_y, z1],
            [cx, peak_y, z0],
        ]);
        self.normals.extend_from_slice(&[nl; 4]);
        self.colors.extend_from_slice(&[darken(color, 0.85); 4]);
        self.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);

        // Right slope
        let b = base + 10;
        let n_right = Vec3::new(hh, hw, 0.0).normalize();
        let nr = [n_right.x, n_right.y, n_right.z];
        self.positions.extend_from_slice(&[
            [x1, cy, z1],
            [x1, cy, z0],
            [cx, peak_y, z0],
            [cx, peak_y, z1],
        ]);
        self.normals.extend_from_slice(&[nr; 4]);
        self.colors.extend_from_slice(&[darken(color, 0.9); 4]);
        self.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }
}
