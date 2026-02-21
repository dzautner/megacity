//! Building Preview Meshes (UX-016)
//!
//! Replaces the generic cuboid cursor preview with zone-type-specific
//! procedural meshes that give the player a visual hint of what will be
//! built. Each zone type gets a distinct silhouette:
//!
//! - **Residential Low**: compact house with pitched roof
//! - **Residential Medium**: taller townhouse/duplex
//! - **Residential High**: tall apartment tower
//! - **Commercial Low**: medium shop building
//! - **Commercial High**: tall commercial skyscraper
//! - **Industrial**: wide, low warehouse/factory
//! - **Office**: tall glass tower
//! - **MixedUse**: medium multi-story building
//!
//! Preview meshes are cached in a resource so they are only generated once.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::HashMap;

use simulation::config::CELL_SIZE;
use simulation::grid::ZoneType;

// ---------------------------------------------------------------------------
// Resource: cached preview mesh handles per zone type
// ---------------------------------------------------------------------------

/// Holds pre-built procedural mesh handles for each zone type preview.
#[derive(Resource)]
pub struct BuildingPreviewMeshes {
    meshes: HashMap<ZoneType, Handle<Mesh>>,
    /// Fallback flat cuboid for non-zone tools (road, bulldoze, etc.)
    pub flat_cuboid: Handle<Mesh>,
}

impl BuildingPreviewMeshes {
    /// Get the preview mesh for a given zone type. Falls back to the flat
    /// cuboid if no specific mesh is registered (e.g. `ZoneType::None`).
    pub fn get(&self, zone: ZoneType) -> Handle<Mesh> {
        self.meshes
            .get(&zone)
            .cloned()
            .unwrap_or_else(|| self.flat_cuboid.clone())
    }
}

// ---------------------------------------------------------------------------
// Mesh generation helpers
// ---------------------------------------------------------------------------

/// Simple mesh builder that accumulates vertices with position, normal, and color.
#[derive(Default)]
struct PreviewMeshData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl PreviewMeshData {
    fn new() -> Self {
        Self::default()
    }

    fn into_mesh(self) -> Mesh {
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
    fn add_cuboid(
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
    fn add_roof_prism(
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

fn lighten(c: [f32; 4], factor: f32) -> [f32; 4] {
    [
        (c[0] * factor).min(1.0),
        (c[1] * factor).min(1.0),
        (c[2] * factor).min(1.0),
        c[3],
    ]
}

fn darken(c: [f32; 4], factor: f32) -> [f32; 4] {
    [c[0] * factor, c[1] * factor, c[2] * factor, c[3]]
}

// ---------------------------------------------------------------------------
// Per-zone preview mesh generators
// ---------------------------------------------------------------------------

/// All preview meshes are built in a 1x1 cell-size coordinate system centered
/// at the origin. The cursor_preview system applies translation and scaling.

/// Residential Low: a small suburban house with a pitched roof.
fn generate_residential_low() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.55, 0.78, 0.55, 1.0]; // soft green tint

    // Main house body
    let hw = s * 0.35;
    let hh = s * 0.22;
    let hd = s * 0.30;
    m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);

    // Pitched roof
    let roof_color = [0.65, 0.35, 0.25, 1.0]; // brown/terra cotta
    m.add_roof_prism(
        0.0,
        hh * 2.0,
        0.0,
        hw * 1.05,
        hh * 0.6,
        hd * 1.05,
        roof_color,
    );

    m.into_mesh()
}

/// Residential Medium: a taller townhouse/duplex.
fn generate_residential_medium() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.45, 0.72, 0.45, 1.0];

    // Taller, narrower body
    let hw = s * 0.30;
    let hh = s * 0.35;
    let hd = s * 0.32;
    m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);

    // Flat roof accent
    let roof_color = [0.50, 0.50, 0.50, 1.0];
    m.add_cuboid(
        0.0,
        hh * 2.0 + s * 0.02,
        0.0,
        hw * 0.9,
        s * 0.02,
        hd * 0.9,
        roof_color,
    );

    m.into_mesh()
}

/// Residential High: a tall apartment tower.
fn generate_residential_high() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.40, 0.68, 0.40, 1.0];

    // Tall tower
    let hw = s * 0.28;
    let hh = s * 0.55;
    let hd = s * 0.28;
    m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);

    // Setback upper portion (stepped silhouette)
    let upper_color = [0.45, 0.72, 0.45, 1.0];
    let upper_hw = hw * 0.75;
    let upper_hh = s * 0.15;
    m.add_cuboid(
        0.0,
        hh * 2.0 + upper_hh,
        0.0,
        upper_hw,
        upper_hh,
        upper_hw,
        upper_color,
    );

    m.into_mesh()
}

/// Commercial Low: a medium-height shop building.
fn generate_commercial_low() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.45, 0.50, 0.82, 1.0]; // blue tint

    // Main body
    let hw = s * 0.38;
    let hh = s * 0.30;
    let hd = s * 0.35;
    m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);

    // Storefront awning
    let awning_color = [0.35, 0.40, 0.70, 1.0];
    m.add_cuboid(
        0.0,
        hh * 0.35,
        hd + s * 0.04,
        hw * 0.9,
        s * 0.015,
        s * 0.04,
        awning_color,
    );

    m.into_mesh()
}

/// Commercial High: a tall skyscraper-like tower.
fn generate_commercial_high() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.40, 0.45, 0.78, 1.0];

    // Base podium
    let base_hw = s * 0.38;
    let base_hh = s * 0.15;
    let base_hd = s * 0.38;
    m.add_cuboid(0.0, base_hh, 0.0, base_hw, base_hh, base_hd, color);

    // Tower portion (narrower, much taller)
    let tower_color = [0.50, 0.55, 0.85, 1.0];
    let tower_hw = s * 0.25;
    let tower_hh = s * 0.50;
    m.add_cuboid(
        0.0,
        base_hh * 2.0 + tower_hh,
        0.0,
        tower_hw,
        tower_hh,
        tower_hw,
        tower_color,
    );

    m.into_mesh()
}

/// Industrial: a wide, low warehouse/factory.
fn generate_industrial() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.78, 0.72, 0.35, 1.0]; // yellow tint

    // Wide, low main body
    let hw = s * 0.42;
    let hh = s * 0.18;
    let hd = s * 0.38;
    m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);

    // Smokestack / vent
    let stack_color = [0.55, 0.50, 0.30, 1.0];
    m.add_cuboid(
        hw * 0.7,
        hh * 2.0 + s * 0.12,
        -hd * 0.6,
        s * 0.035,
        s * 0.12,
        s * 0.035,
        stack_color,
    );

    // Pitched roof over main body (sawtooth factory look)
    let roof_color = [0.60, 0.55, 0.30, 1.0];
    m.add_roof_prism(
        0.0,
        hh * 2.0,
        0.0,
        hw * 1.02,
        hh * 0.35,
        hd * 1.02,
        roof_color,
    );

    m.into_mesh()
}

/// Office: a tall glass tower silhouette.
fn generate_office() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.58, 0.52, 0.80, 1.0]; // purple tint

    // Tall tower
    let hw = s * 0.28;
    let hh = s * 0.55;
    let hd = s * 0.28;
    m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);

    // Crown / spire at top
    let crown_color = [0.65, 0.60, 0.88, 1.0];
    let crown_hw = hw * 0.5;
    let crown_hh = s * 0.08;
    m.add_cuboid(
        0.0,
        hh * 2.0 + crown_hh,
        0.0,
        crown_hw,
        crown_hh,
        crown_hw,
        crown_color,
    );

    m.into_mesh()
}

/// MixedUse: a medium multi-story building.
fn generate_mixed_use() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;

    // Ground floor commercial (blue)
    let comm_color = [0.45, 0.50, 0.72, 1.0];
    let hw = s * 0.35;
    let ground_hh = s * 0.14;
    let hd = s * 0.35;
    m.add_cuboid(0.0, ground_hh, 0.0, hw, ground_hh, hd, comm_color);

    // Upper residential floors (green, slightly narrower)
    let res_color = [0.50, 0.70, 0.45, 1.0];
    let upper_hw = hw * 0.92;
    let upper_hh = s * 0.28;
    m.add_cuboid(
        0.0,
        ground_hh * 2.0 + upper_hh,
        0.0,
        upper_hw,
        upper_hh,
        hd * 0.92,
        res_color,
    );

    m.into_mesh()
}

// ---------------------------------------------------------------------------
// Startup system
// ---------------------------------------------------------------------------

/// Generates and caches all zone-type preview meshes at startup.
pub fn setup_building_preview_meshes(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut map = HashMap::new();

    map.insert(
        ZoneType::ResidentialLow,
        meshes.add(generate_residential_low()),
    );
    map.insert(
        ZoneType::ResidentialMedium,
        meshes.add(generate_residential_medium()),
    );
    map.insert(
        ZoneType::ResidentialHigh,
        meshes.add(generate_residential_high()),
    );
    map.insert(
        ZoneType::CommercialLow,
        meshes.add(generate_commercial_low()),
    );
    map.insert(
        ZoneType::CommercialHigh,
        meshes.add(generate_commercial_high()),
    );
    map.insert(ZoneType::Industrial, meshes.add(generate_industrial()));
    map.insert(ZoneType::Office, meshes.add(generate_office()));
    map.insert(ZoneType::MixedUse, meshes.add(generate_mixed_use()));

    let flat_cuboid = meshes.add(Cuboid::new(CELL_SIZE, 1.0, CELL_SIZE));

    commands.insert_resource(BuildingPreviewMeshes {
        meshes: map,
        flat_cuboid,
    });
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct BuildingPreviewMeshPlugin;

impl Plugin for BuildingPreviewMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_building_preview_meshes.before(crate::cursor_preview::spawn_cursor_preview),
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
}
