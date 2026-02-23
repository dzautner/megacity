//! `MeshData` helper for building procedural meshes from cuboids, cylinders,
//! and roof prisms.

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
// MeshData
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct MeshData {
    pub(crate) positions: Vec<[f32; 3]>,
    pub(crate) normals: Vec<[f32; 3]>,
    pub(crate) colors: Vec<[f32; 4]>,
    pub(crate) indices: Vec<u32>,
}

impl MeshData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_mesh(self) -> Mesh {
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

    #[allow(clippy::too_many_arguments)]
    pub fn add_cuboid(
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

        let front_color = darken(color, 0.85);
        self.positions
            .extend_from_slice(&[[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]]);
        self.normals.extend_from_slice(&[[0.0, 0.0, 1.0]; 4]);
        self.colors.extend_from_slice(&[front_color; 4]);
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        let b = base + 4;
        let back_color = darken(color, 0.75);
        self.positions
            .extend_from_slice(&[[x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]]);
        self.normals.extend_from_slice(&[[0.0, 0.0, -1.0]; 4]);
        self.colors.extend_from_slice(&[back_color; 4]);
        self.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);

        let b = base + 8;
        let top_color = lighten(color, 1.3);
        self.positions
            .extend_from_slice(&[[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]]);
        self.normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
        self.colors.extend_from_slice(&[top_color; 4]);
        self.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);

        let b = base + 12;
        let bot_color = darken(color, 0.5);
        self.positions
            .extend_from_slice(&[[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]]);
        self.normals.extend_from_slice(&[[0.0, -1.0, 0.0]; 4]);
        self.colors.extend_from_slice(&[bot_color; 4]);
        self.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);

        let b = base + 16;
        let right_color = darken(color, 0.7);
        self.positions
            .extend_from_slice(&[[x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]]);
        self.normals.extend_from_slice(&[[1.0, 0.0, 0.0]; 4]);
        self.colors.extend_from_slice(&[right_color; 4]);
        self.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);

        let b = base + 20;
        let left_color = darken(color, 0.65);
        self.positions
            .extend_from_slice(&[[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]]);
        self.normals.extend_from_slice(&[[-1.0, 0.0, 0.0]; 4]);
        self.colors.extend_from_slice(&[left_color; 4]);
        self.indices
            .extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_cylinder(
        &mut self,
        cx: f32,
        cy: f32,
        cz: f32,
        radius: f32,
        height: f32,
        segments: u32,
        color: [f32; 4],
    ) {
        let base = self.positions.len() as u32;
        let half_h = height * 0.5;

        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = cx + angle.cos() * radius;
            let z = cz + angle.sin() * radius;
            let nx = angle.cos();
            let nz = angle.sin();

            self.positions.push([x, cy - half_h, z]);
            self.normals.push([nx, 0.0, nz]);
            self.colors.push(darken(color, 0.9));

            self.positions.push([x, cy + half_h, z]);
            self.normals.push([nx, 0.0, nz]);
            self.colors.push(color);
        }

        for i in 0..segments {
            let i0 = base + i * 2;
            let i1 = base + i * 2 + 1;
            let i2 = base + (i + 1) * 2;
            let i3 = base + (i + 1) * 2 + 1;
            self.indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }

        let top_center = self.positions.len() as u32;
        self.positions.push([cx, cy + half_h, cz]);
        self.normals.push([0.0, 1.0, 0.0]);
        self.colors.push(lighten(color, 1.1));

        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = cx + angle.cos() * radius;
            let z = cz + angle.sin() * radius;
            self.positions.push([x, cy + half_h, z]);
            self.normals.push([0.0, 1.0, 0.0]);
            self.colors.push(lighten(color, 1.1));
        }

        for i in 0..segments {
            let v1 = top_center + 1 + i;
            let v2 = top_center + 1 + (i + 1) % segments;
            self.indices.extend_from_slice(&[top_center, v1, v2]);
        }
    }

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

        self.positions
            .extend_from_slice(&[[x0, cy, z1], [x1, cy, z1], [cx, peak_y, z1]]);
        self.normals.extend_from_slice(&[[0.0, 0.0, 1.0]; 3]);
        self.colors.extend_from_slice(&[color; 3]);
        self.indices.extend_from_slice(&[base, base + 1, base + 2]);

        let b = base + 3;
        self.positions
            .extend_from_slice(&[[x1, cy, z0], [x0, cy, z0], [cx, peak_y, z0]]);
        self.normals.extend_from_slice(&[[0.0, 0.0, -1.0]; 3]);
        self.colors.extend_from_slice(&[color; 3]);
        self.indices.extend_from_slice(&[b, b + 1, b + 2]);

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
