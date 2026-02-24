use bevy::prelude::*;

use simulation::buildings::{Building, UnderConstruction};
use simulation::config::CELL_SIZE;
use simulation::grid::WorldGrid;

use crate::building_meshes::building_scale;

use super::meshes::ensure_assets;
use super::types::{ConstructionAssets, CraneProp, ScaffoldingMesh};

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawn scaffolding + crane meshes for buildings that have an
/// `UnderConstruction` component but no `ScaffoldingMesh` yet.
#[allow(clippy::too_many_arguments)]
pub fn spawn_construction_props(
    mut commands: Commands,
    buildings_uc: Query<(Entity, &Building, &UnderConstruction), Without<ScaffoldingMesh>>,
    existing_scaffolds: Query<&ScaffoldingMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid: Res<WorldGrid>,
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
            Transform::from_xyz(wx, grid.elevation_y(building.grid_x, building.grid_y), wz).with_scale(Vec3::new(
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
                Transform::from_xyz(crane_x, grid.elevation_y(building.grid_x, building.grid_y), crane_z).with_scale(Vec3::new(
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
