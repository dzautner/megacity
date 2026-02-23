use bevy::prelude::*;
use bevy::render::mesh::Indices;

/// Build a disc mesh for an intersection: sidewalk ring + asphalt center.
pub fn build_intersection_disc(
    center: Vec2,
    outer_radius: f32,
    inner_radius: f32,
    sidewalk_color: [f32; 4],
    asphalt_color: [f32; 4],
) -> Mesh {
    let disc_segments = 24;
    let y_sidewalk = 0.02;
    let y_road = 0.04;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Sidewalk disc (larger, lower)
    let base = positions.len() as u32;
    positions.push([center.x, y_sidewalk, center.y]);
    normals.push([0.0, 1.0, 0.0]);
    colors.push(sidewalk_color);
    uvs.push([0.5, 0.5]);

    for i in 0..=disc_segments {
        let angle = (i as f32 / disc_segments as f32) * std::f32::consts::TAU;
        let x = center.x + outer_radius * angle.cos();
        let z = center.y + outer_radius * angle.sin();
        positions.push([x, y_sidewalk, z]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(sidewalk_color);
        uvs.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);

        if i > 0 {
            let vi = base + 1 + i as u32;
            indices.push(base);
            indices.push(vi - 1);
            indices.push(vi);
        }
    }

    // Asphalt disc (smaller, higher)
    let base2 = positions.len() as u32;
    positions.push([center.x, y_road, center.y]);
    normals.push([0.0, 1.0, 0.0]);
    colors.push(asphalt_color);
    uvs.push([0.5, 0.5]);

    for i in 0..=disc_segments {
        let angle = (i as f32 / disc_segments as f32) * std::f32::consts::TAU;
        let x = center.x + inner_radius * angle.cos();
        let z = center.y + inner_radius * angle.sin();
        positions.push([x, y_road, z]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(asphalt_color);
        uvs.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);

        if i > 0 {
            let vi = base2 + 1 + i as u32;
            indices.push(base2);
            indices.push(vi - 1);
            indices.push(vi);
        }
    }

    Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD
            | bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}
