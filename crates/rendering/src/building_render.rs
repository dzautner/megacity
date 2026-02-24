use bevy::prelude::*;

use simulation::buildings::{Building, UnderConstruction};
use simulation::config::CELL_SIZE;
use simulation::grid::{CellType, WorldGrid};
use simulation::services::ServiceBuilding;
use simulation::trees::PlantedTree;
use simulation::utilities::UtilitySource;

use crate::building_mesh_variants::BuildingVariant;
use crate::building_meshes::{building_scale, BuildingModelCache};
use crate::building_variant_proportions;

/// Marker for 3D building entities (both GLB scenes and procedural meshes)
#[derive(Component)]
pub struct BuildingMesh3d {
    pub tracked_entity: Entity,
}

/// Marker to distinguish zone buildings (GLB SceneRoot) from procedural service/utility meshes
#[derive(Component)]
pub struct ZoneBuilding;

/// Determine the yaw rotation so a building faces the nearest road.
/// Returns a yaw angle (0, PI/2, PI, or 3*PI/2) pointing toward the adjacent road.
fn building_facing_road(grid: &WorldGrid, gx: usize, gy: usize, hash: usize) -> f32 {
    let w = grid.width;
    let h = grid.height;
    let south = gy + 1 < h && grid.get(gx, gy + 1).cell_type == CellType::Road;
    let north = gy > 0 && grid.get(gx, gy - 1).cell_type == CellType::Road;
    let east = gx + 1 < w && grid.get(gx + 1, gy).cell_type == CellType::Road;
    let west = gx > 0 && grid.get(gx - 1, gy).cell_type == CellType::Road;

    let mut options = Vec::new();
    if south {
        options.push(0.0);
    }
    if north {
        options.push(std::f32::consts::PI);
    }
    if east {
        options.push(std::f32::consts::FRAC_PI_2);
    }
    if west {
        options.push(-std::f32::consts::FRAC_PI_2);
    }

    if options.is_empty() {
        (hash % 4) as f32 * std::f32::consts::FRAC_PI_2
    } else {
        options[hash % options.len()]
    }
}

/// Compute the position hash used for minor per-building variation.
fn position_hash(grid_x: usize, grid_y: usize) -> usize {
    grid_x
        .wrapping_mul(7)
        .wrapping_add(grid_y.wrapping_mul(13))
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_building_meshes(
    mut commands: Commands,
    buildings: Query<(Entity, &Building), Without<BuildingMesh3d>>,
    under_construction: Query<&UnderConstruction>,
    services: Query<(Entity, &ServiceBuilding), Without<BuildingMesh3d>>,
    utilities: Query<(Entity, &UtilitySource), Without<BuildingMesh3d>>,
    existing: Query<&BuildingMesh3d>,
    grid: Res<WorldGrid>,
    mut model_cache: ResMut<BuildingModelCache>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if buildings.is_empty() && services.is_empty() && utilities.is_empty() {
        return;
    }

    let tracked: std::collections::HashSet<Entity> =
        existing.iter().map(|b| b.tracked_entity).collect();

    // Zone buildings -> spawn as GLTF SceneRoot
    for (entity, building) in &buildings {
        if tracked.contains(&entity) {
            continue;
        }

        let hash = position_hash(building.grid_x, building.grid_y);
        let scene_handle = model_cache.get_zone_scene(building.zone_type, building.level, hash);
        let scale = building_scale(building.zone_type, building.level);

        let (wx, _wy) = WorldGrid::grid_to_world(building.grid_x, building.grid_y);
        let wz = building.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;

        let yaw = building_facing_road(&grid, building.grid_x, building.grid_y, hash);
        let scale_var = 0.98 + (hash % 5) as f32 / 100.0;

        // If under construction, start at 30% y-scale for a "being built" look
        let base_scale = scale * scale_var;
        let build_scale = if let Ok(uc) = under_construction.get(entity) {
            let progress = if uc.total_ticks > 0 {
                1.0 - (uc.ticks_remaining as f32 / uc.total_ticks as f32)
            } else {
                1.0
            };
            let y_factor = 0.3 + progress * 0.7;
            Vec3::new(base_scale, base_scale * y_factor, base_scale)
        } else {
            Vec3::splat(base_scale)
        };

        commands.spawn((
            BuildingMesh3d {
                tracked_entity: entity,
            },
            ZoneBuilding,
            SceneRoot(scene_handle),
            Transform::from_xyz(wx, 0.0, wz)
                .with_rotation(Quat::from_rotation_y(yaw))
                .with_scale(build_scale),
            Visibility::default(),
        ));
    }

    // Service buildings -> procedural meshes (kept as Mesh3d)
    for (entity, service) in &services {
        if tracked.contains(&entity) {
            continue;
        }
        let mesh_handle = model_cache.get_or_create_service_mesh(service.service_type, &mut meshes);
        let mat_handle = model_cache.fallback_material.clone();

        let (fw, fh) = ServiceBuilding::footprint(service.service_type);
        let (wx, _wy) = WorldGrid::grid_to_world(service.grid_x, service.grid_y);
        let wz = service.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        let offset_x = (fw as f32 - 1.0) * CELL_SIZE * 0.5;
        let offset_z = (fh as f32 - 1.0) * CELL_SIZE * 0.5;

        commands.spawn((
            BuildingMesh3d {
                tracked_entity: entity,
            },
            Mesh3d(mesh_handle),
            MeshMaterial3d(mat_handle),
            Transform::from_xyz(wx + offset_x, 0.0, wz + offset_z),
            Visibility::default(),
        ));
    }

    // Utility buildings -> procedural meshes
    for (entity, utility) in &utilities {
        if tracked.contains(&entity) {
            continue;
        }
        let mesh_handle = model_cache.get_or_create_utility_mesh(utility.utility_type, &mut meshes);
        let mat_handle = model_cache.fallback_material.clone();

        let (wx, _wy) = WorldGrid::grid_to_world(utility.grid_x, utility.grid_y);
        let wz = utility.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;

        commands.spawn((
            BuildingMesh3d {
                tracked_entity: entity,
            },
            Mesh3d(mesh_handle),
            MeshMaterial3d(mat_handle),
            Transform::from_xyz(wx, 0.0, wz),
            Visibility::default(),
        ));
    }
}

pub fn update_building_meshes(
    buildings: Query<(Entity, &Building), Changed<Building>>,
    mut commands: Commands,
    mesh_sprites: Query<(Entity, &BuildingMesh3d, Option<&ZoneBuilding>)>,
    grid: Res<WorldGrid>,
    model_cache: Res<BuildingModelCache>,
) {
    if buildings.is_empty() {
        return;
    }

    let sprite_lookup: std::collections::HashMap<Entity, (Entity, bool)> = mesh_sprites
        .iter()
        .map(|(sprite_e, bm, zone)| (bm.tracked_entity, (sprite_e, zone.is_some())))
        .collect();

    for (entity, building) in &buildings {
        if let Some(&(sprite_entity, is_zone)) = sprite_lookup.get(&entity) {
            if is_zone {
                let hash = position_hash(building.grid_x, building.grid_y);
                let scene_handle =
                    model_cache.get_zone_scene(building.zone_type, building.level, hash);
                let scale = building_scale(building.zone_type, building.level);

                let (wx, _wy) = WorldGrid::grid_to_world(building.grid_x, building.grid_y);
                let wz = building.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                let yaw = building_facing_road(&grid, building.grid_x, building.grid_y, hash);
                let scale_var = 0.98 + (hash % 5) as f32 / 100.0;

                commands.entity(sprite_entity).despawn();
                commands.spawn((
                    BuildingMesh3d {
                        tracked_entity: entity,
                    },
                    ZoneBuilding,
                    SceneRoot(scene_handle),
                    Transform::from_xyz(wx, 0.0, wz)
                        .with_rotation(Quat::from_rotation_y(yaw))
                        .with_scale(Vec3::splat(scale * scale_var)),
                    Visibility::default(),
                ));
            }
        }
    }
}

/// Gradually increases the y-scale of buildings under construction as they
/// progress toward completion. When construction finishes (UnderConstruction
/// removed), snaps scale to the full target.
///
/// Respects per-variant proportions from `BuildingVariant` so construction
/// animation is consistent with the final variant shape.
pub fn update_construction_visuals(
    sim_buildings: Query<(&Building, Option<&UnderConstruction>)>,
    mut mesh_query: Query<
        (&BuildingMesh3d, &mut Transform, Option<&BuildingVariant>),
        With<ZoneBuilding>,
    >,
) {
    for (bm, mut transform, maybe_variant) in &mut mesh_query {
        let Ok((building, maybe_uc)) = sim_buildings.get(bm.tracked_entity) else {
            continue;
        };

        let hash = position_hash(building.grid_x, building.grid_y);
        let base_scale = building_scale(building.zone_type, building.level);
        let scale_var = 0.98 + (hash % 5) as f32 / 100.0;
        let s = base_scale * scale_var;

        // Apply per-variant proportions if a variant has been assigned
        let props = maybe_variant.map_or_else(
            || building_variant_proportions::VariantProportion { x: 1.0, y: 1.0, z: 1.0 },
            |v| {
                building_variant_proportions::proportions_for(
                    building.zone_type,
                    building.level,
                )[v.variant_index]
            },
        );

        if let Some(uc) = maybe_uc {
            let progress = if uc.total_ticks > 0 {
                1.0 - (uc.ticks_remaining as f32 / uc.total_ticks as f32)
            } else {
                1.0
            };
            let y_factor = 0.3 + progress * 0.7;
            transform.scale = Vec3::new(
                s * props.x,
                s * props.y * y_factor,
                s * props.z,
            );
        } else {
            let target = Vec3::new(s * props.x, s * props.y, s * props.z);
            if (transform.scale - target).length() > 0.01 {
                transform.scale = target;
            }
        }
    }
}

pub fn cleanup_orphan_building_meshes(
    mut commands: Commands,
    sprites: Query<(Entity, &BuildingMesh3d)>,
    buildings: Query<Entity, With<Building>>,
    services: Query<Entity, With<ServiceBuilding>>,
    utilities: Query<Entity, With<UtilitySource>>,
) {
    for (sprite_entity, bm) in &sprites {
        let exists = buildings.get(bm.tracked_entity).is_ok()
            || services.get(bm.tracked_entity).is_ok()
            || utilities.get(bm.tracked_entity).is_ok();

        if !exists {
            commands.entity(sprite_entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Planted tree mesh rendering
// ---------------------------------------------------------------------------

/// Marker for the 3D mesh of a player-planted tree.
#[derive(Component)]
pub struct PlantedTreeMesh {
    pub tracked_entity: Entity,
}

/// Resource that caches the procedural tree mesh + material so we only create them once.
#[derive(Resource)]
pub struct PlantedTreeAssets {
    pub trunk_mesh: Handle<Mesh>,
    pub canopy_mesh: Handle<Mesh>,
    pub trunk_material: Handle<StandardMaterial>,
    pub canopy_material: Handle<StandardMaterial>,
}

/// Spawn 3D meshes for newly planted trees (PlantedTree entities without a
/// corresponding PlantedTreeMesh).
pub fn spawn_planted_tree_meshes(
    mut commands: Commands,
    new_trees: Query<(Entity, &PlantedTree), Without<PlantedTreeMesh>>,
    existing_meshes: Query<&PlantedTreeMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tree_assets: Option<Res<PlantedTreeAssets>>,
) {
    if new_trees.is_empty() {
        return;
    }

    let tracked: std::collections::HashSet<Entity> =
        existing_meshes.iter().map(|m| m.tracked_entity).collect();

    let assets = if let Some(a) = tree_assets {
        a.clone()
    } else {
        let trunk = meshes.add(Cylinder::new(0.8, 6.0));
        let canopy = meshes.add(Sphere::new(3.5));

        let trunk_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.28, 0.12),
            perceptual_roughness: 0.9,
            ..default()
        });
        let canopy_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.15, 0.55, 0.15),
            perceptual_roughness: 0.8,
            ..default()
        });

        let a = PlantedTreeAssets {
            trunk_mesh: trunk,
            canopy_mesh: canopy,
            trunk_material: trunk_mat,
            canopy_material: canopy_mat,
        };
        commands.insert_resource(a.clone());
        a
    };

    for (entity, tree) in &new_trees {
        if tracked.contains(&entity) {
            continue;
        }

        let (wx, _wy) = WorldGrid::grid_to_world(tree.grid_x, tree.grid_y);
        let wz = tree.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;

        let hash = tree
            .grid_x
            .wrapping_mul(41)
            .wrapping_add(tree.grid_y.wrapping_mul(53));
        let scale_var = 0.85 + (hash % 30) as f32 / 100.0;

        let trunk_entity = commands
            .spawn((
                PlantedTreeMesh {
                    tracked_entity: entity,
                },
                Mesh3d(assets.trunk_mesh.clone()),
                MeshMaterial3d(assets.trunk_material.clone()),
                Transform::from_xyz(wx, 3.0 * scale_var, wz).with_scale(Vec3::splat(scale_var)),
                Visibility::default(),
            ))
            .id();

        commands
            .spawn((
                Mesh3d(assets.canopy_mesh.clone()),
                MeshMaterial3d(assets.canopy_material.clone()),
                Transform::from_xyz(0.0, 4.5, 0.0),
                Visibility::default(),
            ))
            .set_parent(trunk_entity);
    }
}

/// Clean up planted tree meshes whose PlantedTree entity was despawned.
pub fn cleanup_planted_tree_meshes(
    mut commands: Commands,
    mesh_entities: Query<(Entity, &PlantedTreeMesh)>,
    trees: Query<Entity, With<PlantedTree>>,
) {
    for (mesh_entity, ptm) in &mesh_entities {
        if trees.get(ptm.tracked_entity).is_err() {
            commands.entity(mesh_entity).despawn();
        }
    }
}

impl Clone for PlantedTreeAssets {
    fn clone(&self) -> Self {
        Self {
            trunk_mesh: self.trunk_mesh.clone(),
            canopy_mesh: self.canopy_mesh.clone(),
            trunk_material: self.trunk_material.clone(),
            canopy_material: self.canopy_material.clone(),
        }
    }
}
