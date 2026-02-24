//! ECS systems for district selection and cache refresh, plus the plugin definition.

use bevy::prelude::*;

use rendering::input::{ActiveTool, CursorGridPos};
use simulation::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use simulation::districts::{DistrictMap, Districts};
use simulation::services::ServiceBuilding;
use simulation::SaveLoadState;

use super::helpers::{resolve_district_index, service_covers_cell};
use super::resources::{DistrictInspectCache, DistrictPanelOpen, SelectedDistrict};
use super::ui_panel::district_inspect_ui;

/// System that detects cell clicks in Inspect mode and selects the corresponding
/// player-defined district. If no player district is assigned, clears the selection.
pub fn detect_district_selection(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    district_map: Res<DistrictMap>,
    mut selected: ResMut<SelectedDistrict>,
    mut panel_open: ResMut<DistrictPanelOpen>,
) {
    if !buttons.just_pressed(MouseButton::Left) || !cursor.valid {
        return;
    }

    // Only detect in Inspect mode
    if *tool != ActiveTool::Inspect {
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    if gx >= GRID_WIDTH || gy >= GRID_HEIGHT {
        return;
    }

    if let Some(di) = resolve_district_index(&district_map, gx, gy) {
        selected.0 = Some(di);
        panel_open.0 = true;
    } else {
        // Clicked on a cell not in any player district -- clear selection
        selected.0 = None;
        panel_open.0 = false;
    }
}

/// System that refreshes the district inspect cache based on the selected district.
///
/// Reads from the player-defined `DistrictMap` (for district name, color, cells) and
/// the automatic `Districts` resource (for per-district aggregate statistics).
/// Also computes service coverage by checking which service buildings have coverage
/// radii overlapping cells in the selected district.
pub fn refresh_district_inspect(
    selected: Res<SelectedDistrict>,
    district_map: Res<DistrictMap>,
    districts: Res<Districts>,
    services: Query<&ServiceBuilding>,
    mut cache: ResMut<DistrictInspectCache>,
    mut panel_open: ResMut<DistrictPanelOpen>,
) {
    let Some(di) = selected.0 else {
        cache.valid = false;
        panel_open.0 = false;
        return;
    };

    if di >= district_map.districts.len() {
        cache.valid = false;
        panel_open.0 = false;
        return;
    }

    let district = &district_map.districts[di];
    cache.name = district.name.clone();
    cache.color = district.color;
    cache.cell_count = district.cells.len();

    // Use the PlayerDistrictStats computed by district_stats system
    cache.population = district.stats.population;
    cache.avg_happiness = district.stats.avg_happiness;

    // Aggregate jobs from the automatic statistical districts that overlap
    // with this player-defined district's cells.
    let mut commercial_jobs = 0u32;
    let mut industrial_jobs = 0u32;
    let mut office_jobs = 0u32;

    // Collect which automatic districts overlap with this player district
    let mut seen_auto_districts = std::collections::HashSet::new();
    for &(cx, cy) in &district.cells {
        let (adx, ady) = Districts::district_for_grid(cx, cy);
        seen_auto_districts.insert((adx, ady));
    }

    for (adx, ady) in &seen_auto_districts {
        let auto_d = districts.get(*adx, *ady);
        commercial_jobs += auto_d.commercial_jobs;
        industrial_jobs += auto_d.industrial_jobs;
        office_jobs += auto_d.office_jobs;
    }

    cache.commercial_jobs = commercial_jobs;
    cache.industrial_jobs = industrial_jobs;
    cache.office_jobs = office_jobs;

    // Service coverage: count services whose radius overlaps at least one cell
    // in this district. We check a representative sample of cells for efficiency.
    let mut fire = 0u32;
    let mut police = 0u32;
    let mut health = 0u32;
    let mut education = 0u32;
    let mut parks = 0u32;
    let mut transport = 0u32;

    // Precompute a bounding box of the district cells to quickly reject far services
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (GRID_WIDTH, GRID_HEIGHT, 0usize, 0usize);
    for &(cx, cy) in &district.cells {
        min_x = min_x.min(cx);
        min_y = min_y.min(cy);
        max_x = max_x.max(cx);
        max_y = max_y.max(cy);
    }

    for service in &services {
        let stype = service.service_type;
        let radius = service.radius;

        // Quick bounding box rejection
        let service_world_x = service.grid_x as f32 * CELL_SIZE;
        let service_world_y = service.grid_y as f32 * CELL_SIZE;
        let bb_min_x = min_x as f32 * CELL_SIZE;
        let bb_min_y = min_y as f32 * CELL_SIZE;
        let bb_max_x = (max_x + 1) as f32 * CELL_SIZE;
        let bb_max_y = (max_y + 1) as f32 * CELL_SIZE;

        // If the service is too far from the bounding box, skip
        if service_world_x + radius < bb_min_x
            || service_world_x - radius > bb_max_x
            || service_world_y + radius < bb_min_y
            || service_world_y - radius > bb_max_y
        {
            continue;
        }

        // Check if the service covers at least one cell in the district
        let covers = district
            .cells
            .iter()
            .any(|&(cx, cy)| service_covers_cell(service.grid_x, service.grid_y, radius, cx, cy));

        if covers {
            if ServiceBuilding::is_fire(stype) {
                fire += 1;
            } else if ServiceBuilding::is_police(stype) {
                police += 1;
            } else if ServiceBuilding::is_health(stype) {
                health += 1;
            } else if ServiceBuilding::is_education(stype) {
                education += 1;
            } else if ServiceBuilding::is_park(stype) {
                parks += 1;
            } else if ServiceBuilding::is_transport(stype) {
                transport += 1;
            }
        }
    }

    cache.fire_services = fire;
    cache.police_services = police;
    cache.health_services = health;
    cache.education_services = education;
    cache.park_services = parks;
    cache.transport_services = transport;
    cache.valid = true;
}

// =============================================================================
// Plugin
// =============================================================================

pub struct DistrictInspectPlugin;

impl Plugin for DistrictInspectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedDistrict>()
            .init_resource::<DistrictPanelOpen>()
            .init_resource::<DistrictInspectCache>()
            .add_systems(
                Update,
                (
                    detect_district_selection,
                    refresh_district_inspect,
                    district_inspect_ui,
                )
                    .chain()
                    .run_if(in_state(SaveLoadState::Idle)),
            );
    }
}
