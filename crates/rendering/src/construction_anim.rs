use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use simulation::buildings::{Building, UnderConstruction};
use simulation::config::CELL_SIZE;
use simulation::grid::WorldGrid;

use crate::building_meshes::building_scale;
use crate::building_render::{BuildingMesh3d, ZoneBuilding};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker for scaffolding mesh entities attached to buildings under construction.
#[derive(Component)]
pub struct ScaffoldingMesh {
    /// The simulation entity (Building) this scaffolding tracks.
    pub tracked_entity: Entity,
}

/// Marker for crane prop entities attached to buildings under construction.
#[derive(Component)]
pub struct CraneProp {
    /// The simulation entity (Building) this crane tracks.
    pub tracked_entity: Entity,
}

// ---------------------------------------------------------------------------
// Shared assets (lazy-initialised)
// ---------------------------------------------------------------------------

/// Cached mesh and material handles for construction visuals, created once on
/// first use and reused for all construction sites.
#[derive(Resource, Clone)]
pub struct ConstructionAssets {
    pub scaffold_mesh: Handle<Mesh>,
    pub scaffold_material: Handle<StandardMaterial>,
    pub crane_base_mesh: Handle<Mesh>,
    pub crane_arm_mesh: Handle<Mesh>,
    pub crane_material: Handle<StandardMaterial>,
}

// ---------------------------------------------------------------------------
// Mesh builders
// ---------------------------------------------------------------------------

/// Build a wireframe-style scaffolding cuboid from thin boxes (poles + cross
/// braces).  The mesh is centred at the origin and spans `[-0.5, 0.5]` in all
/// axes so it can be scaled per-building later.
fn build_scaffold_mesh() -> Mesh {
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
fn build_crane_base_mesh() -> Mesh {
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
fn build_crane_arm_mesh() -> Mesh {
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
// Systems
// ---------------------------------------------------------------------------

/// Lazily initialise the shared construction assets on first need.
fn ensure_assets(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    existing: &Option<Res<ConstructionAssets>>,
) -> ConstructionAssets {
    if let Some(a) = existing {
        return a.clone();
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

/// Spawn scaffolding + crane meshes for buildings that have an
/// `UnderConstruction` component but no `ScaffoldingMesh` yet.
#[allow(clippy::too_many_arguments)]
pub fn spawn_construction_props(
    mut commands: Commands,
    buildings_uc: Query<(Entity, &Building, &UnderConstruction), Without<ScaffoldingMesh>>,
    existing_scaffolds: Query<&ScaffoldingMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Option<Res<ConstructionAssets>>,
) {
    if buildings_uc.is_empty() {
        return;
    }

    let tracked: std::collections::HashSet<Entity> = existing_scaffolds
        .iter()
        .map(|s| s.tracked_entity)
        .collect();

    let ca = ensure_assets(&mut commands, &mut meshes, &mut materials, &assets);

    for (entity, building, uc) in &buildings_uc {
        if tracked.contains(&entity) {
            continue;
        }

        let progress = if uc.total_ticks > 0 {
            1.0 - (uc.ticks_remaining as f32 / uc.total_ticks as f32)
        } else {
            1.0
        };

        // World position of building
        let (wx, _) = WorldGrid::grid_to_world(building.grid_x, building.grid_y);
        let wz = building.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;

        // Use building scale to size scaffolding appropriately
        let hash = building
            .grid_x
            .wrapping_mul(7)
            .wrapping_add(building.grid_y.wrapping_mul(13));
        let base_scale = building_scale(building.zone_type, building.level);
        let scale_var = 0.98 + (hash % 5) as f32 / 100.0;
        let full_scale = base_scale * scale_var;

        // Scaffolding dimensions (slightly larger than building footprint)
        let scaffold_width = CELL_SIZE * 1.05;
        let scaffold_height = CELL_SIZE * full_scale * 0.12; // proportional to building height
        let y_factor = 0.3 + progress * 0.7;
        let visible_height = scaffold_height * y_factor;

        // Spawn scaffolding mesh
        commands.spawn((
            ScaffoldingMesh {
                tracked_entity: entity,
            },
            Mesh3d(ca.scaffold_mesh.clone()),
            MeshMaterial3d(ca.scaffold_material.clone()),
            Transform::from_xyz(wx, 0.0, wz).with_scale(Vec3::new(
                scaffold_width,
                visible_height,
                scaffold_width,
            )),
            Visibility::default(),
        ));

        // Spawn crane prop (only for buildings with enough height to warrant it --
        // level >= 1 always gets one since we want the visual).
        // Crane mast sits on the building edge, extends above the scaffold.
        let crane_mast_height = scaffold_height * 1.3; // taller than the final building
        let crane_arm_length = scaffold_width * 0.8;

        // Offset crane to one corner of the building
        let crane_x = wx + scaffold_width * 0.4;
        let crane_z = wz + scaffold_width * 0.4;

        let crane_mast_entity = commands
            .spawn((
                CraneProp {
                    tracked_entity: entity,
                },
                Mesh3d(ca.crane_base_mesh.clone()),
                MeshMaterial3d(ca.crane_material.clone()),
                Transform::from_xyz(crane_x, 0.0, crane_z).with_scale(Vec3::new(
                    CELL_SIZE * 0.15,
                    crane_mast_height,
                    CELL_SIZE * 0.15,
                )),
                Visibility::default(),
            ))
            .id();

        // Jib arm as child of mast (positioned near the top)
        let arm_local_y = 0.92; // near top of mast in local space
        commands
            .spawn((
                Mesh3d(ca.crane_arm_mesh.clone()),
                MeshMaterial3d(ca.crane_material.clone()),
                Transform::from_xyz(-crane_arm_length * 0.5, arm_local_y, 0.0).with_scale(
                    Vec3::new(
                        crane_arm_length / (CELL_SIZE * 0.15), // undo parent scale on X
                        0.8 / crane_mast_height,               // thin arm
                        1.0,
                    ),
                ),
                Visibility::default(),
            ))
            .set_parent(crane_mast_entity);
    }
}

/// Update scaffolding scale to match construction progress -- the scaffolding
/// grows upward in sync with the building's y-scale animation from
/// `building_render::update_construction_visuals`.
pub fn update_construction_anim(
    sim_buildings: Query<(&Building, Option<&UnderConstruction>)>,
    mut scaffolds: Query<(&ScaffoldingMesh, &mut Transform)>,
) {
    for (scaffold, mut transform) in &mut scaffolds {
        let Ok((building, maybe_uc)) = sim_buildings.get(scaffold.tracked_entity) else {
            continue;
        };

        let Some(uc) = maybe_uc else {
            continue;
        };

        let progress = if uc.total_ticks > 0 {
            1.0 - (uc.ticks_remaining as f32 / uc.total_ticks as f32)
        } else {
            1.0
        };

        let hash = building
            .grid_x
            .wrapping_mul(7)
            .wrapping_add(building.grid_y.wrapping_mul(13));
        let base_scale = building_scale(building.zone_type, building.level);
        let scale_var = 0.98 + (hash % 5) as f32 / 100.0;
        let full_scale = base_scale * scale_var;

        let scaffold_width = CELL_SIZE * 1.05;
        let scaffold_height = CELL_SIZE * full_scale * 0.12;
        let y_factor = 0.3 + progress * 0.7;

        transform.scale = Vec3::new(scaffold_width, scaffold_height * y_factor, scaffold_width);
    }
}

/// Slowly rotate crane jib arms for visual interest.
pub fn animate_crane_rotation(
    time: Res<Time>,
    cranes: Query<(Entity, &CraneProp)>,
    mut transforms: Query<&mut Transform>,
) {
    for (crane_entity, _crane) in &cranes {
        let Ok(mut t) = transforms.get_mut(crane_entity) else {
            continue;
        };
        // Rotate at ~6 degrees per second (leisurely crane swing)
        let rot_speed = 0.1; // radians per second
        let angle = time.elapsed_secs() * rot_speed;
        t.rotation = Quat::from_rotation_y(angle);
    }
}

/// Despawn scaffolding and crane meshes once the building's
/// `UnderConstruction` component is removed (construction complete).
pub fn cleanup_construction_props(
    mut commands: Commands,
    scaffolds: Query<(Entity, &ScaffoldingMesh)>,
    cranes: Query<(Entity, &CraneProp)>,
    under_construction: Query<&UnderConstruction>,
) {
    for (scaffold_entity, scaffold) in &scaffolds {
        if under_construction.get(scaffold.tracked_entity).is_err() {
            commands.entity(scaffold_entity).despawn();
        }
    }
    for (crane_entity, crane) in &cranes {
        if under_construction.get(crane.tracked_entity).is_err() {
            commands.entity(crane_entity).despawn();
        }
    }
}

/// Despawn construction props whose tracked building entity no longer exists
/// (e.g. bulldozed during construction).
pub fn cleanup_orphan_construction_props(
    mut commands: Commands,
    scaffolds: Query<(Entity, &ScaffoldingMesh)>,
    cranes: Query<(Entity, &CraneProp)>,
    buildings: Query<Entity, With<Building>>,
) {
    for (scaffold_entity, scaffold) in &scaffolds {
        if buildings.get(scaffold.tracked_entity).is_err() {
            commands.entity(scaffold_entity).despawn();
        }
    }
    for (crane_entity, crane) in &cranes {
        if buildings.get(crane.tracked_entity).is_err() {
            commands.entity(crane_entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaffold_mesh_has_geometry() {
        let mesh = build_scaffold_mesh();
        // The mesh should have positions, normals, and indices
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("scaffold mesh should have positions");
        match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => {
                assert!(!v.is_empty(), "scaffold mesh should have vertices");
            }
            _ => panic!("unexpected vertex attribute type"),
        }
    }

    #[test]
    fn test_crane_base_mesh_has_geometry() {
        let mesh = build_crane_base_mesh();
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("crane base mesh should have positions");
        match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => {
                assert!(!v.is_empty(), "crane base mesh should have vertices");
            }
            _ => panic!("unexpected vertex attribute type"),
        }
    }

    #[test]
    fn test_crane_arm_mesh_has_geometry() {
        let mesh = build_crane_arm_mesh();
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("crane arm mesh should have positions");
        match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => {
                assert!(!v.is_empty(), "crane arm mesh should have vertices");
            }
            _ => panic!("unexpected vertex attribute type"),
        }
    }

    #[test]
    fn test_scaffold_mesh_index_count() {
        let mesh = build_scaffold_mesh();
        if let Some(Indices::U32(idx)) = mesh.indices() {
            // Each box = 36 indices (12 triangles).
            // 4 poles + 3*4 horizontal rails + 2*6 diagonal segments = 4+12+12 = 28 boxes
            // 28 * 36 = 1008 indices
            assert_eq!(
                idx.len(),
                28 * 36,
                "scaffold should have correct index count"
            );
        } else {
            panic!("scaffold mesh should have u32 indices");
        }
    }

    #[test]
    fn test_crane_base_mesh_index_count() {
        let mesh = build_crane_base_mesh();
        if let Some(Indices::U32(idx)) = mesh.indices() {
            // 1 box = 36 indices
            assert_eq!(idx.len(), 36, "crane base should have one box");
        } else {
            panic!("crane base mesh should have u32 indices");
        }
    }

    #[test]
    fn test_crane_arm_mesh_index_count() {
        let mesh = build_crane_arm_mesh();
        if let Some(Indices::U32(idx)) = mesh.indices() {
            assert_eq!(idx.len(), 36, "crane arm should have one box");
        } else {
            panic!("crane arm mesh should have u32 indices");
        }
    }

    #[test]
    fn test_progress_calculation() {
        // Progress should be 0.0 at start (ticks_remaining == total_ticks)
        let total = 100u32;
        let remaining = 100u32;
        let progress = 1.0 - (remaining as f32 / total as f32);
        assert!((progress - 0.0).abs() < f32::EPSILON);

        // Progress should be 1.0 when done
        let remaining = 0u32;
        let progress = 1.0 - (remaining as f32 / total as f32);
        assert!((progress - 1.0).abs() < f32::EPSILON);

        // Progress should be 0.5 at midpoint
        let remaining = 50u32;
        let progress = 1.0 - (remaining as f32 / total as f32);
        assert!((progress - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_y_factor_range() {
        // y_factor should range from 0.3 (progress=0) to 1.0 (progress=1)
        let y_at_start = 0.3 + 0.0 * 0.7;
        assert!((y_at_start - 0.3).abs() < f32::EPSILON);

        let y_at_end = 0.3 + 1.0 * 0.7;
        assert!((y_at_end - 1.0).abs() < f32::EPSILON);

        let y_at_mid = 0.3 + 0.5 * 0.7;
        assert!((y_at_mid - 0.65).abs() < f32::EPSILON);
    }
}
