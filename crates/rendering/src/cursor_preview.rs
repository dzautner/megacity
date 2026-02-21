use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::grid::{CellType, RoadType, WorldGrid, ZoneType};
use simulation::services::ServiceBuilding;

use crate::angle_snap::AngleSnapState;
use crate::building_preview_mesh::BuildingPreviewMeshes;
use crate::input::{ActiveTool, CursorGridPos, DrawPhase, IntersectionSnap, RoadDrawState};

/// Marker for the cursor ghost preview entity
#[derive(Component)]
pub struct CursorPreview;

/// Tracks which zone type the preview mesh currently represents so we only
/// swap meshes when the active tool changes zone category.
#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub struct PreviewMeshZone(pub Option<ZoneType>);

pub fn spawn_cursor_preview(
    mut commands: Commands,
    preview_meshes: Res<BuildingPreviewMeshes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = preview_meshes.flat_cuboid.clone();
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.4),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        CursorPreview,
        PreviewMeshZone(None),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Visibility::Hidden,
    ));
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update_cursor_preview(
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    grid: Res<WorldGrid>,
    preview_meshes: Res<BuildingPreviewMeshes>,
    mut query: Query<
        (
            &mut Transform,
            &mut Visibility,
            &MeshMaterial3d<StandardMaterial>,
            &mut Mesh3d,
            &mut PreviewMeshZone,
        ),
        With<CursorPreview>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((mut transform, mut vis, mat_handle, mut mesh3d, mut preview_zone)) =
        query.get_single_mut()
    else {
        return;
    };

    if !cursor.valid {
        *vis = Visibility::Hidden;
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;
    let cell = grid.get(gx, gy);

    // Determine footprint size
    let (fw, fh) = if let Some(st) = tool.service_type() {
        ServiceBuilding::footprint(st)
    } else {
        (1, 1)
    };

    // Determine the zone type the current tool targets (if any)
    let target_zone = tool.zone_type();
    let is_zone_tool = target_zone.is_some();

    // Swap the preview mesh if the zone type changed
    if preview_zone.0 != target_zone {
        let new_mesh = match target_zone {
            Some(zone) => preview_meshes.get(zone),
            None => preview_meshes.flat_cuboid.clone(),
        };
        mesh3d.0 = new_mesh;
        preview_zone.0 = target_zone;
    }

    // Determine preview color and validity based on tool
    let (preview_color, valid) = match *tool {
        ActiveTool::Road
        | ActiveTool::RoadAvenue
        | ActiveTool::RoadBoulevard
        | ActiveTool::RoadHighway
        | ActiveTool::RoadOneWay
        | ActiveTool::RoadPath => {
            let ok = cell.cell_type != CellType::Water && cell.cell_type != CellType::Road;
            (Color::srgba(0.4, 0.4, 0.4, 0.5), ok)
        }
        ActiveTool::Bulldoze => {
            let ok = cell.building_id.is_some()
                || cell.zone != ZoneType::None
                || cell.cell_type == CellType::Road;
            (Color::srgba(0.8, 0.2, 0.1, 0.5), ok)
        }
        ActiveTool::Inspect => (Color::srgba(0.3, 0.6, 0.9, 0.4), true),
        ActiveTool::ZoneResidentialLow
        | ActiveTool::ZoneResidentialMedium
        | ActiveTool::ZoneResidentialHigh => {
            let ok = cell.cell_type == CellType::Grass;
            (Color::srgba(0.2, 0.7, 0.2, 0.5), ok)
        }
        ActiveTool::ZoneCommercialLow | ActiveTool::ZoneCommercialHigh => {
            let ok = cell.cell_type == CellType::Grass;
            (Color::srgba(0.2, 0.3, 0.8, 0.5), ok)
        }
        ActiveTool::ZoneIndustrial => {
            let ok = cell.cell_type == CellType::Grass;
            (Color::srgba(0.8, 0.7, 0.1, 0.5), ok)
        }
        ActiveTool::ZoneOffice => {
            let ok = cell.cell_type == CellType::Grass;
            (Color::srgba(0.6, 0.5, 0.85, 0.5), ok)
        }
        ActiveTool::ZoneMixedUse => {
            let ok = cell.cell_type == CellType::Grass;
            (Color::srgba(0.65, 0.55, 0.3, 0.5), ok)
        }
        _ => {
            // Service / utility placement: check all footprint cells
            let ok = check_footprint_valid(&grid, gx, gy, fw, fh);
            (Color::srgba(0.6, 0.6, 0.6, 0.5), ok)
        }
    };

    // Position on XZ plane
    let (wx, _wy) = WorldGrid::grid_to_world(gx, gy);
    let wz = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;
    let offset_x = (fw as f32 - 1.0) * CELL_SIZE * 0.5;
    let offset_z = (fh as f32 - 1.0) * CELL_SIZE * 0.5;
    transform.translation.x = wx + offset_x;
    transform.translation.z = wz + offset_z;

    // For zone tools with 3D preview meshes, position at ground level.
    // For other tools, keep the flat cuboid slightly above ground.
    if is_zone_tool {
        transform.translation.y = 0.0;
        transform.scale = Vec3::ONE;
    } else {
        transform.translation.y = 0.5;
        transform.scale = Vec3::new(fw as f32, 1.0, fh as f32);
    }

    // Update material color
    let final_color = if valid {
        preview_color
    } else {
        Color::srgba(0.9, 0.15, 0.1, 0.4)
    };

    if let Some(mat) = materials.get_mut(mat_handle.id()) {
        mat.base_color = final_color;
    }

    *vis = Visibility::Visible;
}

/// Returns the visual half-width of a road type for preview rendering.
fn road_half_width(road_type: RoadType) -> f32 {
    match road_type {
        RoadType::Path => 1.5,
        RoadType::OneWay => 3.0,
        RoadType::Local => 4.0,
        RoadType::Avenue => 6.0,
        RoadType::Boulevard => 8.0,
        RoadType::Highway => 10.0,
    }
}

/// Evaluate cubic Bezier at parameter t given four control points (in 2D).
fn bezier_eval(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    let uu = u * u;
    let tt = t * t;
    u * uu * p0 + 3.0 * uu * t * p1 + 3.0 * u * tt * p2 + t * tt * p3
}

/// Tangent (first derivative) of cubic Bezier at parameter t.
fn bezier_tangent(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    3.0 * u * u * (p1 - p0) + 6.0 * u * t * (p2 - p1) + 3.0 * t * t * (p3 - p2)
}

/// Normal vector (perpendicular to tangent, pointing left) in 2D.
fn bezier_normal(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let tan = bezier_tangent(p0, p1, p2, p3, t);
    let len = tan.length();
    if len < 1e-6 {
        return Vec2::ZERO;
    }
    // Perpendicular in 2D: rotate 90 degrees counter-clockwise
    Vec2::new(-tan.y, tan.x) / len
}

/// Draw a full-width Bezier curve preview while in freeform road drawing mode.
/// Intersection markers are handled by the `intersection_preview` module (UX-023).
pub fn draw_bezier_preview(
    draw_state: Res<RoadDrawState>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    angle_snap: Res<AngleSnapState>,
    snap: Res<IntersectionSnap>,
    mut gizmos: Gizmos,
) {
    if draw_state.phase != DrawPhase::PlacedStart || !cursor.valid {
        return;
    }

    // Only draw for road tools
    let road_type = match tool.road_type() {
        Some(rt) => rt,
        None => return,
    };

    let start = draw_state.start_pos;
    // Intersection snap takes precedence over angle snap
    let end = if let Some(snapped) = snap.snapped_pos {
        snapped
    } else if angle_snap.active {
        angle_snap.snapped_pos
    } else {
        cursor.world_pos
    };

    // Bezier control points (straight-line approximation)
    let p0 = start;
    let p3 = end;
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;

    let half_w = road_half_width(road_type);
    let y = 0.5; // slightly above ground
    let segments = 48;

    // Colors
    let center_color = Color::srgba(1.0, 1.0, 0.3, 0.5);
    let edge_color = Color::srgba(1.0, 1.0, 0.3, 0.8);
    let fill_color = Color::srgba(0.5, 0.5, 0.5, 0.2);

    // Sample curve points, normals, and left/right edge points
    let mut centers: Vec<Vec3> = Vec::with_capacity(segments + 1);
    let mut lefts: Vec<Vec3> = Vec::with_capacity(segments + 1);
    let mut rights: Vec<Vec3> = Vec::with_capacity(segments + 1);

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let pt = bezier_eval(p0, p1, p2, p3, t);
        let n = bezier_normal(p0, p1, p2, p3, t);
        let left = pt + n * half_w;
        let right = pt - n * half_w;
        centers.push(Vec3::new(pt.x, y, pt.y));
        lefts.push(Vec3::new(left.x, y, left.y));
        rights.push(Vec3::new(right.x, y, right.y));
    }

    // Draw center line (dimmer)
    for i in 0..segments {
        gizmos.line(centers[i], centers[i + 1], center_color);
    }

    // Draw left and right edge lines
    for i in 0..segments {
        gizmos.line(lefts[i], lefts[i + 1], edge_color);
        gizmos.line(rights[i], rights[i + 1], edge_color);
    }

    // Draw cross-hatching lines to fill the road surface
    let fill_step = 4; // every Nth segment sample
    for i in (0..=segments).step_by(fill_step) {
        gizmos.line(lefts[i], rights[i], fill_color);
    }

    // Draw start marker (green circle)
    let start_3d = Vec3::new(start.x, y, start.y);
    gizmos.circle(
        Isometry3d::new(start_3d, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        half_w,
        Color::srgba(0.2, 1.0, 0.2, 0.9),
    );

    // Draw end marker (yellow circle)
    let end_3d = Vec3::new(end.x, y, end.y);
    gizmos.circle(
        Isometry3d::new(end_3d, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        half_w,
        Color::srgba(1.0, 1.0, 0.3, 0.9),
    );
}

fn check_footprint_valid(grid: &WorldGrid, gx: usize, gy: usize, fw: usize, fh: usize) -> bool {
    for dy in 0..fh {
        for dx in 0..fw {
            let cx = gx + dx;
            let cy = gy + dy;
            if !grid.in_bounds(cx, cy) {
                return false;
            }
            let c = grid.get(cx, cy);
            if c.cell_type != CellType::Grass || c.building_id.is_some() {
                return false;
            }
        }
    }
    true
}

/// Draw a highlight dot when the cursor is snapped to an existing intersection.
pub fn draw_intersection_snap_indicator(
    snap: Res<IntersectionSnap>,
    tool: Res<ActiveTool>,
    mut gizmos: Gizmos,
) {
    // Only show for road tools
    let is_road_tool = matches!(
        *tool,
        ActiveTool::Road
            | ActiveTool::RoadAvenue
            | ActiveTool::RoadBoulevard
            | ActiveTool::RoadHighway
            | ActiveTool::RoadOneWay
            | ActiveTool::RoadPath
    );
    if !is_road_tool {
        return;
    }

    if let Some(pos) = snap.snapped_pos {
        let y = 0.8; // slightly above road preview

        // Outer ring (cyan)
        let center = Vec3::new(pos.x, y, pos.y);
        gizmos.circle(
            Isometry3d::new(center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            12.0,
            Color::srgba(0.0, 1.0, 1.0, 0.9),
        );

        // Inner filled dot (smaller ring to simulate a dot)
        gizmos.circle(
            Isometry3d::new(center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            4.0,
            Color::srgba(0.0, 1.0, 1.0, 1.0),
        );
    }
}
