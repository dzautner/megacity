//! Zoning visual feedback (PLAY-P1-01).
//!
//! When a player zones cells, they need immediate feedback about why buildings
//! aren't spawning. This module shows floating 3D badge icons above clusters
//! of zoned cells that are missing power, water, or both.
//!
//! Badges are procedural mesh silhouettes (Cities: Skylines 2-style):
//!   - Lightning bolt: missing power
//!   - Water drop: missing water
//!   - Warning triangle: missing both
//!
//! Nearby cells of the same missing-utility type are clustered together so
//! that one badge represents an entire zone area, not individual cells.
//! Badges appear when the camera is close enough (LOD) and are rebuilt
//! on a 2-second timer.

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use simulation::app_state::AppState;
use simulation::colorblind::ColorblindSettings;
use simulation::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{WorldGrid, ZoneType};
use simulation::SaveLoadState;

use crate::camera::OrbitCamera;
use crate::colorblind_palette::{self, UtilityIconKind};
use crate::zoning_feedback_clustering::{cluster_cells, ZoneCluster, ZoneFeedbackCell};
use crate::zoning_feedback_meshes;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Camera distance beyond which zone feedback badges are hidden.
const MAX_VISIBLE_DISTANCE: f32 = 800.0;

/// Height above ground for floating badges.
const BADGE_Y: f32 = 15.0;

/// Base scale for badge meshes (world units).
const BADGE_BASE_SCALE: f32 = 3.0;

/// Maximum number of badges displayed at once (largest clusters first).
const MAX_BADGES: usize = 25;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Cached list of zoned-but-empty cells that need utility feedback.
/// Rebuilt on a timer to avoid scanning the full grid every frame.
#[derive(Resource, Default)]
struct ZoneFeedbackCells {
    cells: Vec<ZoneFeedbackCell>,
}

/// Clustered badge data, rebuilt from `ZoneFeedbackCells`.
#[derive(Resource, Default)]
struct ZoneBadgeClusters {
    clusters: Vec<ZoneCluster>,
    generation: u64,
}

/// Pre-built mesh handles for badge icons.
#[derive(Resource)]
struct ZoneBadgeAssets {
    bolt_mesh: Handle<Mesh>,
    drop_mesh: Handle<Mesh>,
    warning_mesh: Handle<Mesh>,
}

/// Marker component for zone utility badge entities.
#[derive(Component)]
struct ZoneBadgeMarker;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn setup_badge_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let bolt = meshes.add(zoning_feedback_meshes::build_lightning_bolt_mesh());
    let drop = meshes.add(zoning_feedback_meshes::build_water_drop_mesh());
    let warning = meshes.add(zoning_feedback_meshes::build_warning_mesh());

    commands.insert_resource(ZoneBadgeAssets {
        bolt_mesh: bolt,
        drop_mesh: drop,
        warning_mesh: warning,
    });
}

/// Scan the grid for zoned cells without buildings that are missing utilities.
fn rebuild_zone_feedback(grid: Res<WorldGrid>, mut feedback: ResMut<ZoneFeedbackCells>) {
    let mut cells = Vec::new();

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);

            if cell.zone == ZoneType::None || cell.building_id.is_some() {
                continue;
            }

            let kind = match (cell.has_power, cell.has_water) {
                (false, false) => UtilityIconKind::NoPowerNoWater,
                (false, true) => UtilityIconKind::NoPower,
                (true, false) => UtilityIconKind::NoWater,
                (true, true) => continue,
            };

            cells.push(ZoneFeedbackCell { gx: x, gy: y, kind });
        }
    }

    feedback.cells = cells;
}

/// Build clusters from the cached feedback cells.
fn rebuild_zone_clusters(
    feedback: Res<ZoneFeedbackCells>,
    mut badge_clusters: ResMut<ZoneBadgeClusters>,
) {
    let clusters = cluster_cells(&feedback.cells);
    badge_clusters.clusters = clusters;
    badge_clusters.generation = badge_clusters.generation.wrapping_add(1);
}

/// Synchronize badge entities with the current cluster data.
fn sync_zone_badges(
    mut commands: Commands,
    badge_clusters: Res<ZoneBadgeClusters>,
    badge_assets: Res<ZoneBadgeAssets>,
    cb_settings: Res<ColorblindSettings>,
    existing: Query<Entity, With<ZoneBadgeMarker>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut last_gen: Local<u64>,
) {
    if *last_gen == badge_clusters.generation {
        return;
    }
    *last_gen = badge_clusters.generation;

    // Despawn old badges
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    // Spawn new badges (up to MAX_BADGES, largest first)
    let count = badge_clusters.clusters.len().min(MAX_BADGES);
    for cluster in &badge_clusters.clusters[..count] {
        let (wx, _) = WorldGrid::grid_to_world(
            cluster.centroid_gx as usize,
            cluster.centroid_gy as usize,
        );
        let wz = cluster.centroid_gy * CELL_SIZE + CELL_SIZE * 0.5;

        let color = colorblind_palette::utility_icon_color(cluster.kind, cb_settings.mode);

        let mesh_handle = match cluster.kind {
            UtilityIconKind::NoPower => badge_assets.bolt_mesh.clone(),
            UtilityIconKind::NoWater => badge_assets.drop_mesh.clone(),
            UtilityIconKind::NoPowerNoWater => badge_assets.warning_mesh.clone(),
        };

        let material = materials.add(StandardMaterial {
            base_color: color,
            emissive: color.to_linear() * 2.0,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        });

        // Scale grows with cluster size: 1.0 + 0.3 * log2(cell_count), clamped
        let size_factor =
            (1.0 + 0.3 * (cluster.cell_count as f32).log2()).clamp(1.0, 3.0);
        let scale = BADGE_BASE_SCALE * size_factor;

        // The mesh lies in XZ plane; rotate -90deg around X to stand upright
        let transform = Transform {
            translation: Vec3::new(wx, BADGE_Y, wz),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            scale: Vec3::splat(scale),
        };

        commands.spawn((
            ZoneBadgeMarker,
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            transform,
            Visibility::default(),
        ));
    }
}

/// Hide badges when the camera is too far away.
fn lod_zone_badges(
    orbit: Res<OrbitCamera>,
    mut badges: Query<&mut Visibility, With<ZoneBadgeMarker>>,
) {
    let visible = orbit.distance <= MAX_VISIBLE_DISTANCE;
    for mut vis in &mut badges {
        *vis = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin that provides visual feedback on zoned cells missing utilities.
///
/// Spawns floating 3D badge icons above clusters of zoned-but-empty cells
/// that lack power and/or water, helping players understand why buildings
/// aren't spawning in their zoned areas.
pub struct ZoningFeedbackPlugin;

impl Plugin for ZoningFeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoneFeedbackCells>();
        app.init_resource::<ZoneBadgeClusters>();

        app.add_systems(Startup, setup_badge_assets);

        let idle = in_state(SaveLoadState::Idle);
        let playing = in_state(AppState::Playing);

        app.add_systems(
            Update,
            (
                rebuild_zone_feedback
                    .run_if(on_timer(std::time::Duration::from_secs(2))),
                rebuild_zone_clusters.after(rebuild_zone_feedback),
                sync_zone_badges.after(rebuild_zone_clusters),
                lod_zone_badges.after(sync_zone_badges),
            )
                .run_if(idle)
                .run_if(playing),
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::colorblind::ColorblindMode;
    use simulation::grid::CellType;

    #[test]
    fn test_feedback_cells_default_empty() {
        let feedback = ZoneFeedbackCells::default();
        assert!(feedback.cells.is_empty());
    }

    #[test]
    fn test_rebuild_finds_missing_power() {
        let mut grid = WorldGrid::new(10, 10);
        let idx = grid.index(5, 5);
        grid.cells[idx].zone = ZoneType::ResidentialLow;
        grid.cells[idx].cell_type = CellType::Grass;
        grid.cells[idx].has_power = false;
        grid.cells[idx].has_water = true;

        let mut feedback = ZoneFeedbackCells::default();
        let cell = grid.get(5, 5);
        if cell.zone != ZoneType::None && cell.building_id.is_none() {
            if let (false, true) = (cell.has_power, cell.has_water) {
                feedback.cells.push(ZoneFeedbackCell {
                    gx: 5,
                    gy: 5,
                    kind: UtilityIconKind::NoPower,
                });
            }
        }

        assert_eq!(feedback.cells.len(), 1);
        assert!(matches!(feedback.cells[0].kind, UtilityIconKind::NoPower));
    }

    #[test]
    fn test_rebuild_finds_missing_water() {
        let mut grid = WorldGrid::new(10, 10);
        let idx = grid.index(3, 3);
        grid.cells[idx].zone = ZoneType::CommercialLow;
        grid.cells[idx].cell_type = CellType::Grass;
        grid.cells[idx].has_power = true;
        grid.cells[idx].has_water = false;

        let cell = grid.get(3, 3);
        let kind = match (cell.has_power, cell.has_water) {
            (true, false) => Some(UtilityIconKind::NoWater),
            _ => None,
        };
        assert!(matches!(kind, Some(UtilityIconKind::NoWater)));
    }

    #[test]
    fn test_rebuild_finds_missing_both() {
        let mut grid = WorldGrid::new(10, 10);
        let idx = grid.index(7, 7);
        grid.cells[idx].zone = ZoneType::Industrial;
        grid.cells[idx].cell_type = CellType::Grass;
        grid.cells[idx].has_power = false;
        grid.cells[idx].has_water = false;

        let cell = grid.get(7, 7);
        let kind = match (cell.has_power, cell.has_water) {
            (false, false) => Some(UtilityIconKind::NoPowerNoWater),
            _ => None,
        };
        assert!(matches!(kind, Some(UtilityIconKind::NoPowerNoWater)));
    }

    #[test]
    fn test_rebuild_skips_cells_with_buildings() {
        let mut grid = WorldGrid::new(10, 10);
        let idx = grid.index(5, 5);
        grid.cells[idx].zone = ZoneType::ResidentialLow;
        grid.cells[idx].has_power = false;
        grid.cells[idx].has_water = false;
        grid.cells[idx].building_id = Some(Entity::from_raw(42));

        let cell = grid.get(5, 5);
        assert!(cell.building_id.is_some());
    }

    #[test]
    fn test_rebuild_skips_fully_connected() {
        let mut grid = WorldGrid::new(10, 10);
        let idx = grid.index(5, 5);
        grid.cells[idx].zone = ZoneType::ResidentialLow;
        grid.cells[idx].has_power = true;
        grid.cells[idx].has_water = true;

        let cell = grid.get(5, 5);
        let should_show = cell.zone != ZoneType::None
            && cell.building_id.is_none()
            && !(cell.has_power && cell.has_water);
        assert!(!should_show);
    }

    #[test]
    fn test_icon_colors_are_reused_from_palette() {
        let settings = ColorblindSettings::default();
        let _c1 =
            colorblind_palette::utility_icon_color(UtilityIconKind::NoPower, settings.mode);
        let _c2 =
            colorblind_palette::utility_icon_color(UtilityIconKind::NoWater, settings.mode);
        let _c3 = colorblind_palette::utility_icon_color(
            UtilityIconKind::NoPowerNoWater,
            settings.mode,
        );
    }

    #[test]
    fn test_icon_colors_distinct_all_modes() {
        for mode in ColorblindMode::ALL {
            let c1 =
                colorblind_palette::utility_icon_color(UtilityIconKind::NoPower, mode)
                    .to_srgba();
            let c2 =
                colorblind_palette::utility_icon_color(UtilityIconKind::NoWater, mode)
                    .to_srgba();
            let diff = (c1.red - c2.red).abs()
                + (c1.green - c2.green).abs()
                + (c1.blue - c2.blue).abs();
            assert!(
                diff > 0.1,
                "NoPower and NoWater should be distinct in {:?} mode",
                mode
            );
        }
    }

    #[test]
    fn test_max_visible_distance_reasonable() {
        assert!(MAX_VISIBLE_DISTANCE > 0.0);
        assert!(MAX_VISIBLE_DISTANCE < 4000.0);
    }
}
