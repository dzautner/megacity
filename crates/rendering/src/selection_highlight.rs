use bevy::prelude::*;

use simulation::buildings::Building;
use simulation::config::CELL_SIZE;
use simulation::grid::WorldGrid;
use simulation::services::ServiceBuilding;
use simulation::utilities::UtilitySource;

use crate::building_render::BuildingMesh3d;
use crate::input::SelectedBuilding;

/// Highlight color used for selected building outlines.
const HIGHLIGHT_COLOR: Color = Color::srgba(0.2, 0.7, 1.0, 0.45);

/// Coverage circle color for service buildings.
const COVERAGE_COLOR: Color = Color::srgba(0.2, 0.7, 1.0, 0.15);

/// Marker component on the pulsing highlight overlay mesh.
#[derive(Component)]
pub struct SelectionHighlight {
    /// The simulation entity this highlight tracks.
    pub tracked: Entity,
}

/// Marker component on the coverage radius circle mesh.
#[derive(Component)]
pub struct CoverageCircle {
    pub tracked: Entity,
}

/// Cached handles for the highlight material so we can mutate alpha each frame.
#[derive(Resource)]
pub struct HighlightAssets {
    pub material: Handle<StandardMaterial>,
    pub coverage_material: Handle<StandardMaterial>,
}

/// Spawn / despawn highlight overlays when the selected building changes.
#[allow(clippy::too_many_arguments)]
pub fn manage_selection_highlights(
    mut commands: Commands,
    selected: Res<SelectedBuilding>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
    utilities: Query<&UtilitySource>,
    mesh_sprites: Query<(&BuildingMesh3d, &Transform)>,
    existing_highlights: Query<(Entity, &SelectionHighlight)>,
    existing_circles: Query<(Entity, &CoverageCircle)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    highlight_assets: Option<Res<HighlightAssets>>,
) {
    let selected_entity = selected.0;

    // Despawn highlights that no longer match the selection
    for (hl_entity, hl) in &existing_highlights {
        if selected_entity != Some(hl.tracked) {
            commands.entity(hl_entity).despawn();
        }
    }
    for (circle_entity, cc) in &existing_circles {
        if selected_entity != Some(cc.tracked) {
            commands.entity(circle_entity).despawn();
        }
    }

    let Some(sel) = selected_entity else {
        return;
    };

    // Check if highlight already exists for this entity
    let already_highlighted = existing_highlights.iter().any(|(_, hl)| hl.tracked == sel);
    if already_highlighted {
        return;
    }

    // Find the render mesh transform for the selected building
    let Some((_, render_transform)) = mesh_sprites.iter().find(|(bm, _)| bm.tracked_entity == sel)
    else {
        return;
    };

    // Lazily create shared materials
    let (mat_handle, coverage_mat_handle) = if let Some(assets) = &highlight_assets {
        (assets.material.clone(), assets.coverage_material.clone())
    } else {
        let mat = materials.add(StandardMaterial {
            base_color: HIGHLIGHT_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let cov_mat = materials.add(StandardMaterial {
            base_color: COVERAGE_COLOR,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        commands.insert_resource(HighlightAssets {
            material: mat.clone(),
            coverage_material: cov_mat.clone(),
        });
        (mat, cov_mat)
    };

    // Determine highlight mesh size based on entity type
    let (size_x, size_y, size_z) = if let Ok(building) = buildings.get(sel) {
        // Zone building: use a box around the cell, scaled up 5%
        let _ = building;
        let s = render_transform.scale;
        (
            CELL_SIZE * s.x * 1.05,
            CELL_SIZE * s.y.max(1.0) * 1.05,
            CELL_SIZE * s.z * 1.05,
        )
    } else if let Ok(service) = services.get(sel) {
        let (fw, fh) = ServiceBuilding::footprint(service.service_type);
        (
            fw as f32 * CELL_SIZE * 1.05,
            CELL_SIZE * 1.05,
            fh as f32 * CELL_SIZE * 1.05,
        )
    } else if utilities.get(sel).is_ok() {
        (CELL_SIZE * 1.05, CELL_SIZE * 1.05, CELL_SIZE * 1.05)
    } else {
        return;
    };

    // Spawn highlight overlay mesh (slightly larger box around the building)
    let highlight_mesh = meshes.add(Cuboid::new(size_x, size_y, size_z));
    let pos = render_transform.translation;

    commands.spawn((
        SelectionHighlight { tracked: sel },
        Mesh3d(highlight_mesh),
        MeshMaterial3d(mat_handle),
        Transform::from_translation(Vec3::new(pos.x, size_y * 0.5, pos.z)),
        Visibility::default(),
    ));

    // For service buildings, also spawn a coverage radius circle
    if let Ok(service) = services.get(sel) {
        let radius = service.radius;
        let (fw, fh) = ServiceBuilding::footprint(service.service_type);
        let (wx, _) = WorldGrid::grid_to_world(service.grid_x, service.grid_y);
        let wz = service.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        let offset_x = (fw as f32 - 1.0) * CELL_SIZE * 0.5;
        let offset_z = (fh as f32 - 1.0) * CELL_SIZE * 0.5;
        let center = Vec3::new(wx + offset_x, 0.2, wz + offset_z);

        // Flat circle mesh on the ground (XZ plane)
        let circle_mesh = meshes.add(Circle::new(radius));
        commands.spawn((
            CoverageCircle { tracked: sel },
            Mesh3d(circle_mesh),
            MeshMaterial3d(coverage_mat_handle),
            Transform::from_translation(center)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            Visibility::default(),
        ));
    }
}

/// Animate the highlight material alpha between 0.3 and 0.6 at 2 Hz.
pub fn animate_selection_highlights(
    time: Res<Time>,
    highlight_assets: Option<Res<HighlightAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    highlights: Query<&SelectionHighlight>,
) {
    // Only animate if there are active highlights
    if highlights.is_empty() {
        return;
    }

    let Some(assets) = highlight_assets else {
        return;
    };

    // Sine wave: 2 Hz means 2 full cycles per second => angular freq = 2 * 2 * PI
    let t = time.elapsed_secs();
    let sine = (t * 2.0 * std::f32::consts::TAU).sin(); // -1 to 1
    let alpha = 0.45 + sine * 0.15; // 0.3 to 0.6

    // Update highlight material
    if let Some(mat) = materials.get_mut(assets.material.id()) {
        let Srgba {
            red, green, blue, ..
        } = mat.base_color.to_srgba();
        mat.base_color = Color::srgba(red, green, blue, alpha);
    }

    // Pulse coverage circle at lower intensity
    let cov_alpha = 0.10 + sine * 0.05; // 0.05 to 0.15
    if let Some(mat) = materials.get_mut(assets.coverage_material.id()) {
        let Srgba {
            red, green, blue, ..
        } = mat.base_color.to_srgba();
        mat.base_color = Color::srgba(red, green, blue, cov_alpha);
    }
}

/// Draw gizmo highlights for connected entities (residents' homes/workplaces).
pub fn draw_connected_highlights(
    selected: Res<SelectedBuilding>,
    buildings: Query<&Building>,
    services: Query<&ServiceBuilding>,
    citizens: Query<(
        &simulation::citizen::HomeLocation,
        Option<&simulation::citizen::WorkLocation>,
    )>,
    mut gizmos: Gizmos,
) {
    let Some(sel) = selected.0 else {
        return;
    };

    // Check if the selected entity is a building with residents
    if let Ok(building) = buildings.get(sel) {
        let (wx, _) = WorldGrid::grid_to_world(building.grid_x, building.grid_y);
        let wz = building.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        let building_pos = Vec3::new(wx, 1.0, wz);

        // Find citizens connected to this building (home or work)
        for (home, work) in &citizens {
            let is_home = home.building == sel;
            let is_work = work.as_ref().map_or(false, |w| w.building == sel);

            if is_home {
                // Draw line to workplace
                if let Some(w) = work {
                    let (wwx, _) = WorldGrid::grid_to_world(w.grid_x, w.grid_y);
                    let wwz = w.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                    let work_pos = Vec3::new(wwx, 1.0, wwz);
                    gizmos.line(building_pos, work_pos, Color::srgba(0.3, 0.9, 0.3, 0.3));
                }
            } else if is_work {
                // Draw line to home
                let (hwx, _) = WorldGrid::grid_to_world(home.grid_x, home.grid_y);
                let hwz = home.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
                let home_pos = Vec3::new(hwx, 1.0, hwz);
                gizmos.line(building_pos, home_pos, Color::srgba(0.9, 0.6, 0.2, 0.3));
            }
        }
    }

    // For service buildings, draw coverage circle gizmo as secondary feedback
    if let Ok(service) = services.get(sel) {
        let (fw, fh) = ServiceBuilding::footprint(service.service_type);
        let (wx, _) = WorldGrid::grid_to_world(service.grid_x, service.grid_y);
        let wz = service.grid_y as f32 * CELL_SIZE + CELL_SIZE * 0.5;
        let offset_x = (fw as f32 - 1.0) * CELL_SIZE * 0.5;
        let offset_z = (fh as f32 - 1.0) * CELL_SIZE * 0.5;
        let center = Vec3::new(wx + offset_x, 0.5, wz + offset_z);

        gizmos.circle(
            Isometry3d::new(center, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            service.radius,
            Color::srgba(0.2, 0.7, 1.0, 0.5),
        );
    }
}
