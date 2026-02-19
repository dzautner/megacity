use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::grid::{CellType, WorldGrid, ZoneType};
use simulation::services::ServiceBuilding;

use crate::input::{ActiveTool, CursorGridPos, DrawPhase, RoadDrawState};

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

/// Draw a Bezier curve preview while in freeform road drawing mode.
pub fn draw_bezier_preview(
    draw_state: Res<RoadDrawState>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    mut gizmos: Gizmos,
) {
    if draw_state.phase != DrawPhase::PlacedStart || !cursor.valid {
        return;
    }

    // Only draw for road tools
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

    let start = draw_state.start_pos;
    let end = cursor.world_pos;

    // Draw preview as a series of line segments
    let segments = 32;
    let p0 = start;
    let p3 = end;
    let p1 = p0 + (p3 - p0) / 3.0;
    let p2 = p0 + (p3 - p0) * 2.0 / 3.0;
    let y = 0.5; // slightly above ground

    let color = Color::srgba(1.0, 1.0, 0.3, 0.8);
    let mut prev = Vec3::new(p0.x, y, p0.y);

    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;
        let pt = p0 * mt3 + p1 * 3.0 * mt2 * t + p2 * 3.0 * mt * t2 + p3 * t3;
        let curr = Vec3::new(pt.x, y, pt.y);
        gizmos.line(prev, curr, color);
        prev = curr;
    }

    // Draw start marker
    let start_3d = Vec3::new(start.x, y, start.y);
    gizmos.circle(
        Isometry3d::new(start_3d, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        8.0,
        Color::srgba(0.2, 1.0, 0.2, 0.9),
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
