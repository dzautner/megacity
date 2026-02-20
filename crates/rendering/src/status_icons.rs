use bevy::prelude::*;

use simulation::buildings::Building;
use simulation::colorblind::ColorblindSettings;
use simulation::config::CELL_SIZE;
use simulation::grid::WorldGrid;

use crate::colorblind_palette::{self, UtilityIconKind};

/// Marker component for status-icon entities floating above buildings.
#[derive(Component)]
pub struct BuildingStatusIcon {
    /// The simulation `Building` entity this icon belongs to.
    pub tracked_entity: Entity,
}

/// Tracks the last-known utility status so we only mutate the world when something changes.
#[derive(Component)]
pub struct LastUtilityStatus {
    has_power: bool,
    has_water: bool,
}

/// Cached status per building, used to detect changes between ticks.
#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(clippy::enum_variant_names)]
enum IconKind {
    NoPower,
    NoWater,
    NoPowerNoWater,
}

fn icon_color(kind: IconKind, settings: &ColorblindSettings) -> Color {
    let cb_kind = match kind {
        IconKind::NoPower => UtilityIconKind::NoPower,
        IconKind::NoWater => UtilityIconKind::NoWater,
        IconKind::NoPowerNoWater => UtilityIconKind::NoPowerNoWater,
    };
    colorblind_palette::utility_icon_color(cb_kind, settings.mode)
}

fn classify(has_power: bool, has_water: bool) -> Option<IconKind> {
    match (has_power, has_water) {
        (false, false) => Some(IconKind::NoPowerNoWater),
        (false, true) => Some(IconKind::NoPower),
        (true, false) => Some(IconKind::NoWater),
        (true, true) => None, // everything OK
    }
}

/// The icon cube is 2x2x2 world units and floats at y=25.
const ICON_Y: f32 = 25.0;
const ICON_HALF_SIZE: f32 = 1.0;

/// System that keeps floating warning cubes in sync with building utility status.
///
/// Designed to run on a timer (every 2 s) rather than every frame.  It:
///   1. Reads every `Building` entity and looks up `has_power`/`has_water` on the grid.
///   2. Compares against a `LastUtilityStatus` stored on existing icon entities.
///   3. Spawns / despawns / recolors icons only when the status has actually changed.
#[allow(clippy::too_many_arguments)]
pub fn update_building_status_icons(
    mut commands: Commands,
    buildings: Query<(Entity, &Building)>,
    grid: Res<WorldGrid>,
    cb_settings: Res<ColorblindSettings>,
    existing_icons: Query<(Entity, &BuildingStatusIcon, &LastUtilityStatus)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Build a lookup: building entity -> existing icon entity + cached status
    let mut icon_map: std::collections::HashMap<Entity, (Entity, bool, bool)> =
        std::collections::HashMap::with_capacity(existing_icons.iter().len());
    for (icon_entity, icon, last) in &existing_icons {
        icon_map.insert(
            icon.tracked_entity,
            (icon_entity, last.has_power, last.has_water),
        );
    }

    for (building_entity, building) in &buildings {
        let gx = building.grid_x;
        let gy = building.grid_y;

        // Bounds check (grid_to_world does not validate)
        if gx >= grid.width || gy >= grid.height {
            continue;
        }

        let cell = grid.get(gx, gy);
        let has_power = cell.has_power;
        let has_water = cell.has_water;
        let kind = classify(has_power, has_water);

        if let Some((_icon_entity, prev_power, prev_water)) = icon_map.remove(&building_entity) {
            // An icon already exists for this building.
            if prev_power == has_power && prev_water == has_water {
                // No change -- skip.
                continue;
            }

            // Status changed. Despawn old icon and potentially spawn a new one.
            commands.entity(_icon_entity).despawn();

            if let Some(k) = kind {
                spawn_icon(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &cb_settings,
                    building_entity,
                    gx,
                    gy,
                    has_power,
                    has_water,
                    k,
                );
            }
        } else {
            // No existing icon for this building.
            if let Some(k) = kind {
                spawn_icon(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &cb_settings,
                    building_entity,
                    gx,
                    gy,
                    has_power,
                    has_water,
                    k,
                );
            }
        }
    }

    // Any icons remaining in the map belong to buildings that no longer exist -- clean up.
    for (_building_entity, (icon_entity, _, _)) in icon_map {
        commands.entity(icon_entity).despawn();
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_icon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    cb_settings: &ColorblindSettings,
    building_entity: Entity,
    gx: usize,
    gy: usize,
    has_power: bool,
    has_water: bool,
    kind: IconKind,
) {
    let (wx, _wy) = WorldGrid::grid_to_world(gx, gy);
    let wz = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;

    let color = icon_color(kind, cb_settings);
    let mesh = meshes.add(Cuboid::new(
        ICON_HALF_SIZE * 2.0,
        ICON_HALF_SIZE * 2.0,
        ICON_HALF_SIZE * 2.0,
    ));
    let material = materials.add(StandardMaterial {
        base_color: color,
        emissive: color.to_linear() * 2.0,
        unlit: true,
        ..default()
    });

    commands.spawn((
        BuildingStatusIcon {
            tracked_entity: building_entity,
        },
        LastUtilityStatus {
            has_power,
            has_water,
        },
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(wx, ICON_Y, wz),
        Visibility::default(),
    ));
}
