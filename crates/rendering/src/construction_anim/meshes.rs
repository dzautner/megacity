use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use super::types::ConstructionAssets;

// ---------------------------------------------------------------------------
// Mesh builders
// ---------------------------------------------------------------------------

/// Build a wireframe-style scaffolding cuboid from thin boxes (poles + cross
/// braces).  The mesh is centred at the origin and spans `[-0.5, 0.5]` in all
/// axes so it can be scaled per-building later.
pub(crate) fn build_scaffold_mesh() -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let pole_radius = 0.02; // thin poles in normalised space

    // Helper: add an axis-aligned box between two corners.
    let mut add_box = |min: [f32; 3], max: [f32; 3]| {
        let base = positions.len() as u32;
        let (x0, y0, z0) = (min[0], min[1], min[2]);
        let (x1, y1, z1) = (max[0], max[1], max[2]);

        // 8 corners
        #[rustfmt::skip]
        let verts: [[f32; 3]; 8] = [
            [x0, y0, z0], [x1, y0, z0], [x1, y1, z0], [x0, y1, z0],
            [x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1],
        ];

        // 6 faces x 2 triangles = 12 tris = 36 indices
        #[rustfmt::skip]
        let face_indices: [u32; 36] = [
            // front  (z0)
            0, 1, 2, 0, 2, 3,
            // back   (z1)
            4, 6, 5, 4, 7, 6,
            // left   (x0)
            0, 3, 7, 0, 7, 4,
            // right  (x1)
            1, 5, 6, 1, 6, 2,
            // bottom (y0)
            0, 4, 5, 0, 5, 1,
            // top    (y1)
            3, 2, 6, 3, 6, 7,
        ];

        #[rustfmt::skip]
        let face_normals: [[f32; 3]; 8] = [
            [-0.577, -0.577, -0.577], [ 0.577, -0.577, -0.577],
            [ 0.577,  0.577, -0.577], [-0.577,  0.577, -0.577],
            [-0.577, -0.577,  0.577], [ 0.577, -0.577,  0.577],
            [ 0.577,  0.577,  0.577], [-0.577,  0.577,  0.577],
        ];

        positions.extend_from_slice(&verts);
        normals.extend_from_slice(&face_normals);
        for idx in &face_indices {
            indices.push(base + idx);
        }
    };

    let r = pole_radius;

    // 4 vertical corner poles
    for &x in &[-0.5_f32, 0.5] {
        for &z in &[-0.5_f32, 0.5] {
            add_box([x - r, 0.0, z - r], [x + r, 1.0, z + r]);
        }
    }

    // Horizontal rails at 3 heights: 0.25, 0.5, 0.75
    for &y in &[0.25_f32, 0.5, 0.75] {
        // Along X (front and back)
        for &z in &[-0.5_f32, 0.5] {
            add_box([-0.5, y - r, z - r], [0.5, y + r, z + r]);
        }
        // Along Z (left and right)
        for &x in &[-0.5_f32, 0.5] {
            add_box([x - r, y - r, -0.5], [x + r, y + r, 0.5]);
        }
    }

    // Diagonal cross braces on two faces (front z=-0.5 and right x=0.5)
    // We approximate diagonals with small axis-aligned strips (good enough
    // for a low-detail construction prop).
    let steps = 6u32;
    for i in 0..steps {
        let t0 = i as f32 / steps as f32;
        let t1 = (i + 1) as f32 / steps as f32;
        let y0 = t0;
        let y1 = t1;
        // Front face diagonal (z = -0.5)
        let x0 = -0.5 + t0;
        let x1 = -0.5 + t1;
        add_box([x0, y0, -0.5 - r], [x1, y1, -0.5 + r]);
        // Right face diagonal (x = 0.5)
        let z0 = -0.5 + t0;
        let z1 = -0.5 + t1;
        add_box([0.5 - r, y0, z0], [0.5 + r, y1, z1]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Build a simple crane mesh: a tall vertical mast with a horizontal jib arm.
/// The mast is a thin box from origin upward; the jib extends from near the
/// top.  Normalised to [0, 1] height.
pub(crate) fn build_crane_base_mesh() -> Mesh {
    // Thin vertical mast
    let mast_r = 0.03;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut add_box = |min: [f32; 3], max: [f32; 3]| {
        let base = positions.len() as u32;
        let (x0, y0, z0) = (min[0], min[1], min[2]);
        let (x1, y1, z1) = (max[0], max[1], max[2]);
        #[rustfmt::skip]
        let verts: [[f32; 3]; 8] = [
            [x0, y0, z0], [x1, y0, z0], [x1, y1, z0], [x0, y1, z0],
            [x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1],
        ];
        #[rustfmt::skip]
        let face_indices: [u32; 36] = [
            0, 1, 2, 0, 2, 3,
            4, 6, 5, 4, 7, 6,
            0, 3, 7, 0, 7, 4,
            1, 5, 6, 1, 6, 2,
            0, 4, 5, 0, 5, 1,
            3, 2, 6, 3, 6, 7,
        ];
        #[rustfmt::skip]
        let norms: [[f32; 3]; 8] = [
            [-0.577, -0.577, -0.577], [ 0.577, -0.577, -0.577],
            [ 0.577,  0.577, -0.577], [-0.577,  0.577, -0.577],
            [-0.577, -0.577,  0.577], [ 0.577, -0.577,  0.577],
            [ 0.577,  0.577,  0.577], [-0.577,  0.577,  0.577],
        ];
        positions.extend_from_slice(&verts);
        normals.extend_from_slice(&norms);
        for idx in &face_indices {
            indices.push(base + idx);
        }
    };

    // Vertical mast
    add_box([-mast_r, 0.0, -mast_r], [mast_r, 1.0, mast_r]);

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Build the horizontal jib arm of the crane.  Normalised: extends from
/// origin along +X with length 1.0.
pub(crate) fn build_crane_arm_mesh() -> Mesh {
    let arm_r = 0.025;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut add_box = |min: [f32; 3], max: [f32; 3]| {
        let base = positions.len() as u32;
        let (x0, y0, z0) = (min[0], min[1], min[2]);
        let (x1, y1, z1) = (max[0], max[1], max[2]);
        #[rustfmt::skip]
        let verts: [[f32; 3]; 8] = [
            [x0, y0, z0], [x1, y0, z0], [x1, y1, z0], [x0, y1, z0],
            [x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1],
        ];
        #[rustfmt::skip]
        let face_indices: [u32; 36] = [
            0, 1, 2, 0, 2, 3,
            4, 6, 5, 4, 7, 6,
            0, 3, 7, 0, 7, 4,
            1, 5, 6, 1, 6, 2,
            0, 4, 5, 0, 5, 1,
            3, 2, 6, 3, 6, 7,
        ];
        #[rustfmt::skip]
        let norms: [[f32; 3]; 8] = [
            [-0.577, -0.577, -0.577], [ 0.577, -0.577, -0.577],
            [ 0.577,  0.577, -0.577], [-0.577,  0.577, -0.577],
            [-0.577, -0.577,  0.577], [ 0.577, -0.577,  0.577],
            [ 0.577,  0.577,  0.577], [-0.577,  0.577,  0.577],
        ];
        positions.extend_from_slice(&verts);
        normals.extend_from_slice(&norms);
        for idx in &face_indices {
            indices.push(base + idx);
        }
    };

    // Horizontal arm along +X
    add_box([0.0, -arm_r, -arm_r], [1.0, arm_r, arm_r]);

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

// ---------------------------------------------------------------------------
// Asset initialisation
// ---------------------------------------------------------------------------

/// Lazily initialise the shared construction assets on first need.
pub(crate) fn ensure_assets(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    existing: &Option<Res<ConstructionAssets>>,
) -> ConstructionAssets {
    if let Some(ref a) = existing {
        return (**a).clone();
    }

    let scaffold_mesh = meshes.add(build_scaffold_mesh());
    let crane_base_mesh = meshes.add(build_crane_base_mesh());
    let crane_arm_mesh = meshes.add(build_crane_arm_mesh());

    // Semi-transparent orange for scaffolding
    let scaffold_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.85, 0.55, 0.15, 0.55),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    // Yellow for crane
    let crane_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.85, 0.1),
        perceptual_roughness: 0.6,
        ..default()
    });

    let assets = ConstructionAssets {
        scaffold_mesh,
        scaffold_material,
        crane_base_mesh,
        crane_arm_mesh,
        crane_material,
    };

    commands.insert_resource(assets.clone());
    assets
}
