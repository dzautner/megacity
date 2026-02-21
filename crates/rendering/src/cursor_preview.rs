use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::grid::{CellType, RoadType, WorldGrid, ZoneType};
use simulation::road_segments::RoadSegmentStore;
use simulation::services::ServiceBuilding;

use crate::angle_snap::AngleSnapState;
use crate::input::{ActiveTool, CursorGridPos, DrawPhase, IntersectionSnap, RoadDrawState};

/// Marker for the cursor ghost preview entity
#[derive(Component)]
pub struct CursorPreview;

pub fn spawn_cursor_preview(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(CELL_SIZE, 1.0, CELL_SIZE));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.4),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        CursorPreview,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Visibility::Hidden,
    ));
}

pub fn update_cursor_preview(
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    grid: Res<WorldGrid>,
    mut query: Query<
        (
            &mut Transform,
            &mut Visibility,
            &MeshMaterial3d<StandardMaterial>,
        ),
        With<CursorPreview>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((mut transform, mut vis, mat_handle)) = query.get_single_mut() else {
        return;
    };

    if !cursor.valid {
        *vis = Visibility::Hidden;
        return;
    }

    // Hide cursor preview when zone brush gizmos take over
    if tool.zone_type().is_some() {
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
    transform.translation.y = 0.5;
    transform.translation.z = wz + offset_z;

    // Scale mesh to footprint size
    transform.scale = Vec3::new(fw as f32, 1.0, fh as f32);

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

/// Find approximate intersection points between a preview Bezier curve and
/// existing road segments. Returns world-space 2D positions of intersections.
fn find_preview_intersections(
    p0: Vec2,
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    store: &RoadSegmentStore,
) -> Vec<Vec2> {
    let mut intersections = Vec::new();
    let preview_samples = 48;

    // Pre-sample the preview curve
    let mut preview_points: Vec<Vec2> = Vec::with_capacity(preview_samples + 1);
    for i in 0..=preview_samples {
        let t = i as f32 / preview_samples as f32;
        preview_points.push(bezier_eval(p0, p1, p2, p3, t));
    }

    for segment in &store.segments {
        let seg_samples = 32;
        let mut seg_points: Vec<Vec2> = Vec::with_capacity(seg_samples + 1);
        for i in 0..=seg_samples {
            let t = i as f32 / seg_samples as f32;
            seg_points.push(segment.evaluate(t));
        }

        // Check for line-segment intersections between consecutive sample pairs
        for i in 0..preview_samples {
            let a1 = preview_points[i];
            let a2 = preview_points[i + 1];
            for j in 0..seg_samples {
                let b1 = seg_points[j];
                let b2 = seg_points[j + 1];
                if let Some(pt) = segment_intersection(a1, a2, b1, b2) {
                    // Avoid duplicate markers that are too close together
                    let dominated = intersections
                        .iter()
                        .any(|&p: &Vec2| (p - pt).length() < 8.0);
                    if !dominated {
                        intersections.push(pt);
                    }
                }
            }
        }
    }

    intersections
}

/// 2D line-segment intersection test. Returns the intersection point if
/// the two segments (a1-a2) and (b1-b2) cross each other.
fn segment_intersection(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<Vec2> {
    let d1 = a2 - a1;
    let d2 = b2 - b1;
    let cross = d1.x * d2.y - d1.y * d2.x;
    if cross.abs() < 1e-6 {
        return None; // parallel
    }
    let d = b1 - a1;
    let t = (d.x * d2.y - d.y * d2.x) / cross;
    let u = (d.x * d1.y - d.y * d1.x) / cross;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some(a1 + d1 * t)
    } else {
        None
    }
}

/// Draw a full-width Bezier curve preview while in freeform road drawing mode,
/// with intersection markers where the preview crosses existing roads.
#[allow(clippy::too_many_arguments)]
pub fn draw_bezier_preview(
    draw_state: Res<RoadDrawState>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    angle_snap: Res<AngleSnapState>,
    snap: Res<IntersectionSnap>,
    segment_store: Res<RoadSegmentStore>,
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

    // Find and draw intersection markers
    let isections = find_preview_intersections(p0, p1, p2, p3, &segment_store);
    let marker_color = Color::srgba(1.0, 0.4, 0.1, 0.95);
    for pt in &isections {
        let pos = Vec3::new(pt.x, y + 0.1, pt.y);
        let marker_size = half_w * 0.8;
        gizmos.circle(
            Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            marker_size,
            marker_color,
        );
        // Draw an inner cross for visibility
        let cross_size = marker_size * 0.7;
        gizmos.line(
            pos + Vec3::new(-cross_size, 0.0, -cross_size),
            pos + Vec3::new(cross_size, 0.0, cross_size),
            marker_color,
        );
        gizmos.line(
            pos + Vec3::new(-cross_size, 0.0, cross_size),
            pos + Vec3::new(cross_size, 0.0, -cross_size),
            marker_color,
        );
    }
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
