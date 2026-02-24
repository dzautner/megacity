use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::grid::{CellType, RoadType, WorldGrid, ZoneType};

use crate::building_meshes::BuildingModelCache;

/// Marker component for prop entities (trees, lamps, benches, etc.)
#[derive(Component)]
pub struct PropEntity;

/// Marker for street lamp props
#[derive(Component)]
pub struct StreetLamp;

/// Marker for tree props
#[derive(Component)]
pub struct TreeProp;

/// Marker for parked car props
#[derive(Component)]
pub struct ParkedCar;

/// Resource to track whether props have been spawned
#[derive(Resource, Default)]
pub struct PropsSpawned {
    pub trees_spawned: bool,
    pub lamps_spawned: bool,
    pub parked_cars_spawned: bool,
}

/// Scale for tree GLB models (Kenney nature/prop trees are ~2-4 units tall natively)
const TREE_SCALE: f32 = 2.0;

/// Scale for street lamp models (Kenney lamps are ~3 units tall natively)
const LAMP_SCALE: f32 = 1.5;

/// Spawn 3D tree entities: street trees along roads + nature trees on grass
pub fn spawn_tree_props(
    mut commands: Commands,
    model_cache: Res<BuildingModelCache>,
    grid: Res<WorldGrid>,
    mut props_spawned: ResMut<PropsSpawned>,
) {
    if props_spawned.trees_spawned || model_cache.trees.is_empty() {
        return;
    }
    props_spawned.trees_spawned = true;

    let width = grid.width;
    let height = grid.height;

    for gy in 0..height {
        for gx in 0..width {
            let cell = grid.get(gx, gy);

            // --- Street trees: spawn along road edges ---
            if cell.cell_type == CellType::Road
                && cell.road_type != RoadType::Highway
                && cell.road_type != RoadType::Path
            {
                // Only on road-edge cells (adjacent to non-road)
                let has_non_road = [
                    (gx.wrapping_sub(1), gy),
                    (gx + 1, gy),
                    (gx, gy.wrapping_sub(1)),
                    (gx, gy + 1),
                ]
                .iter()
                .any(|&(nx, ny)| {
                    nx < width && ny < height && grid.get(nx, ny).cell_type != CellType::Road
                });

                if has_non_road {
                    // ~75% of road-edge cells get a street tree
                    let tree_hash = gx.wrapping_mul(41).wrapping_add(gy.wrapping_mul(53)) % 100;
                    if tree_hash < 75 {
                        let (wx, _) = WorldGrid::grid_to_world(gx, gy);
                        let wz = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;

                        // Offset toward the sidewalk (non-road side)
                        let mut off_x: f32 = 0.0;
                        let mut off_z: f32 = 0.0;
                        if gx > 0 && grid.get(gx - 1, gy).cell_type != CellType::Road {
                            off_x = -CELL_SIZE * 0.35;
                        } else if gx + 1 < width && grid.get(gx + 1, gy).cell_type != CellType::Road
                        {
                            off_x = CELL_SIZE * 0.35;
                        }
                        if gy > 0 && grid.get(gx, gy - 1).cell_type != CellType::Road {
                            off_z = -CELL_SIZE * 0.35;
                        } else if gy + 1 < height
                            && grid.get(gx, gy + 1).cell_type != CellType::Road
                        {
                            off_z = CELL_SIZE * 0.35;
                        }

                        let variant = gx.wrapping_mul(7).wrapping_add(gy.wrapping_mul(13));
                        let scene_handle = model_cache.get_tree(variant);
                        // Street trees are smaller and more uniform
                        let scale = TREE_SCALE * (0.6 + (tree_hash as f32 % 4.0) / 20.0);

                        commands.spawn((
                            PropEntity,
                            TreeProp,
                            SceneRoot(scene_handle),
                            Transform::from_xyz(wx + off_x, grid.elevation_y(gx, gy), wz + off_z)
                                .with_scale(Vec3::splat(scale)),
                            Visibility::default(),
                        ));
                    }
                }
                continue;
            }

            // --- Nature/urban trees: on grass cells (zoned or unzoned, without buildings) ---
            if cell.cell_type != CellType::Grass {
                continue;
            }
            if cell.building_id.is_some() {
                continue;
            }

            let tree_hash = gx.wrapping_mul(31).wrapping_add(gy.wrapping_mul(37)) % 100;

            // Zoned cells without buildings: ~25% get small urban trees (fill empty blocks)
            // Unzoned cells: ~15% get nature trees
            let threshold = if cell.zone != ZoneType::None { 25 } else { 15 };
            if tree_hash >= threshold {
                continue;
            }

            let offset_x = ((gx * 17 + gy * 23) % 8) as f32 + 2.0;
            let offset_z = ((gx * 11 + gy * 29) % 8) as f32 + 2.0;

            let (wx, _) = WorldGrid::grid_to_world(gx, gy);
            let wz = gy as f32 * CELL_SIZE + offset_z;
            let wx = wx - CELL_SIZE * 0.5 + offset_x;

            let tree_variant = gx.wrapping_mul(7).wrapping_add(gy.wrapping_mul(13));
            let scene_handle = model_cache.get_tree(tree_variant);

            let scale_var = TREE_SCALE * (0.7 + (tree_hash as f32 % 7.0) / 10.0);
            let yaw = (tree_variant % 8) as f32 * std::f32::consts::FRAC_PI_4;

            commands.spawn((
                PropEntity,
                TreeProp,
                SceneRoot(scene_handle),
                Transform::from_xyz(wx, grid.elevation_y(gx, gy), wz)
                    .with_rotation(Quat::from_rotation_y(yaw))
                    .with_scale(Vec3::splat(scale_var)),
                Visibility::default(),
            ));
        }
    }
}

/// Spawn street lamps along road edges and props in parks
pub fn spawn_road_props(
    mut commands: Commands,
    model_cache: Res<BuildingModelCache>,
    grid: Res<WorldGrid>,
    mut props_spawned: ResMut<PropsSpawned>,
) {
    if props_spawned.lamps_spawned || model_cache.props.is_empty() {
        return;
    }
    props_spawned.lamps_spawned = true;

    let width = grid.width;
    let height = grid.height;

    for gy in 1..height.saturating_sub(1) {
        for gx in 1..width.saturating_sub(1) {
            let cell = grid.get(gx, gy);

            if cell.cell_type != CellType::Road {
                continue;
            }

            // Check if this road cell has a non-road neighbor (edge of road)
            let has_non_road_neighbor = [
                (gx.wrapping_sub(1), gy),
                (gx + 1, gy),
                (gx, gy.wrapping_sub(1)),
                (gx, gy + 1),
            ]
            .iter()
            .any(|&(nx, ny)| {
                if nx < width && ny < height {
                    grid.get(nx, ny).cell_type != CellType::Road
                } else {
                    true
                }
            });

            if !has_non_road_neighbor {
                continue;
            }

            // ~50% of road-edge cells get a street lamp
            let lamp_hash = gx.wrapping_mul(43).wrapping_add(gy.wrapping_mul(59)) % 100;
            if lamp_hash >= 50 {
                continue;
            }

            let (wx, _) = WorldGrid::grid_to_world(gx, gy);
            let wz = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;

            // Offset lamp toward the non-road neighbor side
            let mut offset_x: f32 = 0.0;
            let mut offset_z: f32 = 0.0;
            if gx > 0 && grid.get(gx - 1, gy).cell_type != CellType::Road {
                offset_x = -CELL_SIZE * 0.4;
            } else if gx + 1 < width && grid.get(gx + 1, gy).cell_type != CellType::Road {
                offset_x = CELL_SIZE * 0.4;
            }
            if gy > 0 && grid.get(gx, gy - 1).cell_type != CellType::Road {
                offset_z = -CELL_SIZE * 0.4;
            } else if gy + 1 < height && grid.get(gx, gy + 1).cell_type != CellType::Road {
                offset_z = CELL_SIZE * 0.4;
            }

            // Use street lamp model (detail-light-single or detail-light-double)
            let lamp_variant = (lamp_hash / 10) % 2; // 0 = single, 1 = double
            let scene_handle = if lamp_variant == 0 && model_cache.props.len() > 1 {
                model_cache.props[1].clone() // detail-light-single
            } else if model_cache.props.len() > 2 {
                model_cache.props[2].clone() // detail-light-double
            } else {
                model_cache.get_prop(lamp_hash)
            };

            commands.spawn((
                PropEntity,
                StreetLamp,
                SceneRoot(scene_handle),
                Transform::from_xyz(wx + offset_x, grid.elevation_y(gx, gy), wz + offset_z)
                    .with_scale(Vec3::splat(LAMP_SCALE)),
                Visibility::default(),
            ));
        }
    }
}

/// Spawn static parked cars along residential and commercial streets
pub fn spawn_parked_cars(
    mut commands: Commands,
    model_cache: Res<BuildingModelCache>,
    grid: Res<WorldGrid>,
    mut props_spawned: ResMut<PropsSpawned>,
) {
    if props_spawned.parked_cars_spawned || model_cache.vehicles.is_empty() {
        return;
    }
    props_spawned.parked_cars_spawned = true;

    let width = grid.width;
    let height = grid.height;

    for gy in 1..height.saturating_sub(1) {
        for gx in 1..width.saturating_sub(1) {
            let cell = grid.get(gx, gy);
            if cell.cell_type != CellType::Road {
                continue;
            }
            // Only local and avenue roads get parked cars (not highways/paths)
            if !matches!(
                cell.road_type,
                RoadType::Local | RoadType::Avenue | RoadType::OneWay
            ) {
                continue;
            }

            // Check if this road is adjacent to a zoned/built area
            let adj_zoned = [
                (gx.wrapping_sub(1), gy),
                (gx + 1, gy),
                (gx, gy.wrapping_sub(1)),
                (gx, gy + 1),
            ]
            .iter()
            .any(|&(nx, ny)| {
                if nx < width && ny < height {
                    let nc = grid.get(nx, ny);
                    nc.zone != ZoneType::None && nc.cell_type != CellType::Road
                } else {
                    false
                }
            });
            if !adj_zoned {
                continue;
            }

            // ~20% of eligible road cells get a parked car
            let car_hash = gx.wrapping_mul(67).wrapping_add(gy.wrapping_mul(71)) % 100;
            if car_hash >= 20 {
                continue;
            }

            let (wx, _) = WorldGrid::grid_to_world(gx, gy);
            let wz = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;

            // Determine road direction and park car on the shoulder
            let has_left = gx > 0 && grid.get(gx - 1, gy).cell_type == CellType::Road;
            let has_right = gx + 1 < width && grid.get(gx + 1, gy).cell_type == CellType::Road;
            let has_up = gy + 1 < height && grid.get(gx, gy + 1).cell_type == CellType::Road;
            let has_down = gy > 0 && grid.get(gx, gy - 1).cell_type == CellType::Road;

            let is_horizontal = has_left || has_right;
            let is_vertical = has_up || has_down;

            let (off_x, off_z, yaw) = if is_horizontal && !is_vertical {
                // Horizontal road: park on north or south shoulder
                let side = if car_hash % 2 == 0 { 1.0 } else { -1.0 };
                (0.0, side * CELL_SIZE * 0.28, 0.0)
            } else if is_vertical && !is_horizontal {
                // Vertical road: park on east or west shoulder
                let side = if car_hash % 2 == 0 { 1.0 } else { -1.0 };
                (side * CELL_SIZE * 0.28, 0.0, std::f32::consts::FRAC_PI_2)
            } else {
                continue; // Skip intersections
            };

            // Pick a civilian vehicle (skip emergency vehicles — indices 8-12)
            let car_idx = car_hash % 8; // sedan, sedan-sports, hatchback, suv, suv-luxury, van, truck, taxi
            let scene_handle = model_cache.vehicles[car_idx % model_cache.vehicles.len()].clone();

            commands.spawn((
                PropEntity,
                ParkedCar,
                SceneRoot(scene_handle),
                Transform::from_xyz(wx + off_x, grid.elevation_y(gx, gy), wz + off_z)
                    .with_rotation(Quat::from_rotation_y(yaw))
                    .with_scale(Vec3::splat(1.0)),
                Visibility::default(),
            ));
        }
    }
}
