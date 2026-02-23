//! Tests for district inspection helpers and resource defaults.

use super::*;
use simulation::config::CELL_SIZE;
use simulation::districts::{DistrictMap, Districts, DISTRICT_SIZE};

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
