//! District Selection and Inspection Panel (UX-062).
//!
//! Clicking a cell in Inspect mode shows its district assignment. When a district
//! is selected, an info panel displays:
//! - District name and color swatch
//! - Population and jobs (commercial, industrial, office)
//! - Average happiness
//! - Service coverage: counts of fire, police, health, education, parks, and
//!   transport services whose radius overlaps cells in the district
//! - District boundary highlighting via an optional overlay flag
//!
//! The panel appears as an egui window anchored to the left side of the screen.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::input::{ActiveTool, CursorGridPos};
use simulation::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use simulation::districts::{DistrictMap, Districts, DISTRICT_SIZE};
use simulation::services::ServiceBuilding;

// =============================================================================
// Resources
// =============================================================================

/// Resource tracking the currently selected district index.
#[derive(Resource, Default)]
pub struct SelectedDistrict(pub Option<usize>);

/// Whether the district inspection panel is open (controls boundary highlighting).
#[derive(Resource, Default)]
pub struct DistrictPanelOpen(pub bool);

/// Cached district statistics for the selected district, refreshed each frame.
#[derive(Resource, Default)]
pub struct DistrictInspectCache {
    pub name: String,
    pub color: [f32; 4],
    pub population: u32,
    pub commercial_jobs: u32,
    pub industrial_jobs: u32,
    pub office_jobs: u32,
    pub avg_happiness: f32,
    pub cell_count: usize,
    // Service coverage counts
    pub fire_services: u32,
    pub police_services: u32,
    pub health_services: u32,
    pub education_services: u32,
    pub park_services: u32,
    pub transport_services: u32,
    pub valid: bool,
}

// =============================================================================
// Pure helper functions (testable without ECS)
// =============================================================================

/// Convert a grid cell (gx, gy) to a world-space center coordinate.
fn grid_to_world_center(gx: usize, gy: usize) -> (f32, f32) {
    let wx = gx as f32 * CELL_SIZE + CELL_SIZE * 0.5;
    let wy = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;
    (wx, wy)
}

/// Check if a service building's coverage radius overlaps with a grid cell.
fn service_covers_cell(
    service_x: usize,
    service_y: usize,
    radius: f32,
    cell_x: usize,
    cell_y: usize,
) -> bool {
    let (swx, swy) = grid_to_world_center(service_x, service_y);
    let (cwx, cwy) = grid_to_world_center(cell_x, cell_y);
    let dx = swx - cwx;
    let dy = swy - cwy;
    dx * dx + dy * dy <= radius * radius
}

/// Format a happiness value as a colored label descriptor.
fn happiness_label(happiness: f32) -> &'static str {
    if happiness >= 80.0 {
        "Excellent"
    } else if happiness >= 60.0 {
        "Good"
    } else if happiness >= 40.0 {
        "Fair"
    } else if happiness >= 20.0 {
        "Poor"
    } else {
        "Critical"
    }
}

/// Color for a happiness value.
fn happiness_color(happiness: f32) -> egui::Color32 {
    if happiness >= 80.0 {
        egui::Color32::from_rgb(50, 200, 50) // green
    } else if happiness >= 60.0 {
        egui::Color32::from_rgb(120, 200, 50) // light green
    } else if happiness >= 40.0 {
        egui::Color32::from_rgb(220, 220, 50) // yellow
    } else if happiness >= 20.0 {
        egui::Color32::from_rgb(220, 150, 50) // orange
    } else {
        egui::Color32::from_rgb(220, 50, 50) // red
    }
}

/// Get district index from a player-defined district map for a grid cell,
/// or fall back to the automatic statistical district.
fn resolve_district_index(district_map: &DistrictMap, gx: usize, gy: usize) -> Option<usize> {
    district_map.get_district_index_at(gx, gy)
}

// =============================================================================
// Systems
// =============================================================================

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
            use simulation::services::ServiceType;
            if ServiceType::is_fire(stype) {
                fire += 1;
            } else if ServiceType::is_police(stype) {
                police += 1;
            } else if ServiceType::is_health(stype) {
                health += 1;
            } else if ServiceType::is_education(stype) {
                education += 1;
            } else if ServiceType::is_park(stype) {
                parks += 1;
            } else if ServiceType::is_transport(stype) {
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

/// System that renders the District Inspection Panel using egui.
pub fn district_inspect_ui(mut contexts: EguiContexts, cache: Res<DistrictInspectCache>) {
    if !cache.valid {
        return;
    }

    egui::Window::new("District Info")
        .default_width(260.0)
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 80.0))
        .show(contexts.ctx_mut(), |ui| {
            // District name with color swatch
            ui.horizontal(|ui| {
                let c = cache.color;
                let swatch_color = egui::Color32::from_rgba_unmultiplied(
                    (c[0] * 255.0) as u8,
                    (c[1] * 255.0) as u8,
                    (c[2] * 255.0) as u8,
                    255,
                );
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, egui::CornerRadius::same(3), swatch_color);
                ui.heading(&cache.name);
            });
            ui.separator();

            // Overview stats
            egui::Grid::new("district_overview")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Population:");
                    ui.label(format!("{}", cache.population));
                    ui.end_row();

                    ui.label("Happiness:");
                    let h_color = happiness_color(cache.avg_happiness);
                    ui.colored_label(
                        h_color,
                        format!(
                            "{:.0}% ({})",
                            cache.avg_happiness,
                            happiness_label(cache.avg_happiness)
                        ),
                    );
                    ui.end_row();

                    ui.label("Cells:");
                    ui.label(format!("{}", cache.cell_count));
                    ui.end_row();
                });

            ui.separator();
            ui.heading("Jobs");
            egui::Grid::new("district_jobs")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Commercial:");
                    ui.label(format!("{}", cache.commercial_jobs));
                    ui.end_row();

                    ui.label("Industrial:");
                    ui.label(format!("{}", cache.industrial_jobs));
                    ui.end_row();

                    ui.label("Office:");
                    ui.label(format!("{}", cache.office_jobs));
                    ui.end_row();
                });

            ui.separator();
            ui.heading("Service Coverage");
            egui::Grid::new("district_services")
                .num_columns(2)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    service_row(ui, "Fire", cache.fire_services);
                    service_row(ui, "Police", cache.police_services);
                    service_row(ui, "Health", cache.health_services);
                    service_row(ui, "Education", cache.education_services);
                    service_row(ui, "Parks", cache.park_services);
                    service_row(ui, "Transport", cache.transport_services);
                });
        });
}

/// Helper to render a service coverage row with color indicator.
fn service_row(ui: &mut egui::Ui, label: &str, count: u32) {
    ui.label(format!("{}:", label));
    let color = if count >= 2 {
        egui::Color32::from_rgb(50, 200, 50) // well covered
    } else if count == 1 {
        egui::Color32::from_rgb(220, 220, 50) // minimal
    } else {
        egui::Color32::from_rgb(220, 50, 50) // none
    };
    ui.colored_label(color, format!("{}", count));
    ui.end_row();
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
                    .chain(),
            );
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_to_world_center() {
        // CELL_SIZE = 16.0
        let (wx, wy) = grid_to_world_center(0, 0);
        assert!((wx - 8.0).abs() < f32::EPSILON);
        assert!((wy - 8.0).abs() < f32::EPSILON);

        let (wx, wy) = grid_to_world_center(3, 2);
        assert!((wx - 56.0).abs() < f32::EPSILON);
        assert!((wy - 40.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_service_covers_cell_same_cell() {
        // Service at (5, 5), radius = 1 cell width, cell at (5, 5) -> covered
        assert!(service_covers_cell(5, 5, CELL_SIZE, 5, 5));
    }

    #[test]
    fn test_service_covers_cell_adjacent() {
        // Adjacent cell should be covered with radius = 2 * CELL_SIZE
        assert!(service_covers_cell(5, 5, 2.0 * CELL_SIZE, 6, 5));
    }

    #[test]
    fn test_service_covers_cell_far_away() {
        // Far cell should not be covered with small radius
        assert!(!service_covers_cell(5, 5, CELL_SIZE, 100, 100));
    }

    #[test]
    fn test_service_covers_cell_zero_radius() {
        // Zero radius covers nothing (distance > 0)
        assert!(!service_covers_cell(5, 5, 0.0, 6, 5));
        // But same cell has distance 0
        assert!(service_covers_cell(5, 5, 0.0, 5, 5));
    }

    #[test]
    fn test_happiness_label_levels() {
        assert_eq!(happiness_label(90.0), "Excellent");
        assert_eq!(happiness_label(80.0), "Excellent");
        assert_eq!(happiness_label(70.0), "Good");
        assert_eq!(happiness_label(60.0), "Good");
        assert_eq!(happiness_label(50.0), "Fair");
        assert_eq!(happiness_label(40.0), "Fair");
        assert_eq!(happiness_label(30.0), "Poor");
        assert_eq!(happiness_label(20.0), "Poor");
        assert_eq!(happiness_label(10.0), "Critical");
        assert_eq!(happiness_label(0.0), "Critical");
    }

    #[test]
    fn test_happiness_color_levels() {
        // Each level should produce a different color
        let excellent = happiness_color(90.0);
        let good = happiness_color(70.0);
        let fair = happiness_color(50.0);
        let poor = happiness_color(30.0);
        let critical = happiness_color(10.0);

        // At minimum, green and red extremes should differ
        assert_ne!(excellent, critical);
        assert_ne!(good, poor);
        assert_ne!(fair, critical);
    }

    #[test]
    fn test_resolve_district_index_no_assignment() {
        let map = DistrictMap::default();
        // No cells assigned, should return None
        assert!(resolve_district_index(&map, 10, 10).is_none());
    }

    #[test]
    fn test_resolve_district_index_with_assignment() {
        let mut map = DistrictMap::default();
        map.assign_cell_to_district(10, 10, 2);
        assert_eq!(resolve_district_index(&map, 10, 10), Some(2));
    }

    #[test]
    fn test_resolve_district_index_out_of_bounds() {
        let map = DistrictMap::default();
        assert!(resolve_district_index(&map, 999, 999).is_none());
    }

    #[test]
    fn test_selected_district_default() {
        let selected = SelectedDistrict::default();
        assert!(selected.0.is_none());
    }

    #[test]
    fn test_district_panel_open_default() {
        let panel = DistrictPanelOpen::default();
        assert!(!panel.0);
    }

    #[test]
    fn test_district_inspect_cache_default() {
        let cache = DistrictInspectCache::default();
        assert!(!cache.valid);
        assert_eq!(cache.population, 0);
        assert_eq!(cache.avg_happiness, 0.0);
        assert_eq!(cache.commercial_jobs, 0);
        assert_eq!(cache.industrial_jobs, 0);
        assert_eq!(cache.office_jobs, 0);
        assert_eq!(cache.fire_services, 0);
        assert_eq!(cache.police_services, 0);
        assert_eq!(cache.health_services, 0);
        assert_eq!(cache.education_services, 0);
        assert_eq!(cache.park_services, 0);
        assert_eq!(cache.transport_services, 0);
    }

    #[test]
    fn test_district_auto_mapping() {
        // Verify that district_for_grid works for the standard 16x16 districts
        let (dx, dy) = Districts::district_for_grid(0, 0);
        assert_eq!((dx, dy), (0, 0));

        let (dx, dy) = Districts::district_for_grid(DISTRICT_SIZE - 1, DISTRICT_SIZE - 1);
        assert_eq!((dx, dy), (0, 0));

        let (dx, dy) = Districts::district_for_grid(DISTRICT_SIZE, 0);
        assert_eq!((dx, dy), (1, 0));
    }

    #[test]
    fn test_service_covers_diagonal_cell() {
        // Diagonal cell at (6, 6) from service at (5, 5)
        // Distance = sqrt(2) * CELL_SIZE ~= 22.6
        // With radius = 2 * CELL_SIZE = 32.0, should be covered
        assert!(service_covers_cell(5, 5, 2.0 * CELL_SIZE, 6, 6));

        // With radius = CELL_SIZE = 16.0, diagonal distance > radius
        // distance = sqrt(16^2 + 16^2) = sqrt(512) ~= 22.6 > 16
        assert!(!service_covers_cell(5, 5, CELL_SIZE, 6, 6));
    }

    #[test]
    fn test_district_map_default_has_districts() {
        let map = DistrictMap::default();
        assert!(!map.districts.is_empty());
        // Default districts from DEFAULT_DISTRICTS constant
        assert_eq!(map.districts[0].name, "Downtown");
        assert_eq!(map.districts[1].name, "Suburbs");
    }
}
