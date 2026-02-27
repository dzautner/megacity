//! Integration tests for ASCII map rendering (#1902).
//!
//! Tests the overview and detail map generators against a headless
//! TestCity with various grid configurations.

use crate::ascii_map::{build_detail_map, build_overview_map, cell_to_char};
use crate::grid::{Cell, CellType, RoadType, WorldGrid, ZoneType};
use crate::test_harness::TestCity;

// -----------------------------------------------------------------------
// cell_to_char mapping
// -----------------------------------------------------------------------

#[test]
fn test_cell_to_char_mapping() {
    // Grass (default)
    let cell = Cell::default();
    assert_eq!(cell_to_char(&cell), '.');

    // Water
    let mut cell = Cell::default();
    cell.cell_type = CellType::Water;
    assert_eq!(cell_to_char(&cell), '~');

    // Roads
    let mut cell = Cell::default();
    cell.cell_type = CellType::Road;
    cell.road_type = RoadType::Local;
    assert_eq!(cell_to_char(&cell), '#');

    cell.road_type = RoadType::Avenue;
    assert_eq!(cell_to_char(&cell), '=');

    cell.road_type = RoadType::Boulevard;
    assert_eq!(cell_to_char(&cell), 'H');

    cell.road_type = RoadType::Highway;
    assert_eq!(cell_to_char(&cell), '%');

    cell.road_type = RoadType::OneWay;
    assert_eq!(cell_to_char(&cell), '#');

    cell.road_type = RoadType::Path;
    assert_eq!(cell_to_char(&cell), '#');

    // Zones without buildings
    let mut cell = Cell::default();
    cell.zone = ZoneType::ResidentialLow;
    assert_eq!(cell_to_char(&cell), 'r');

    cell.zone = ZoneType::ResidentialMedium;
    assert_eq!(cell_to_char(&cell), 'm');

    cell.zone = ZoneType::ResidentialHigh;
    assert_eq!(cell_to_char(&cell), 'R');

    cell.zone = ZoneType::CommercialLow;
    assert_eq!(cell_to_char(&cell), 'c');

    cell.zone = ZoneType::CommercialHigh;
    assert_eq!(cell_to_char(&cell), 'C');

    cell.zone = ZoneType::Industrial;
    assert_eq!(cell_to_char(&cell), 'I');

    cell.zone = ZoneType::Office;
    assert_eq!(cell_to_char(&cell), 'O');

    cell.zone = ZoneType::MixedUse;
    assert_eq!(cell_to_char(&cell), 'M');

    // Building with zone → uppercase zone char
    let mut cell = Cell::default();
    cell.building_id = Some(bevy::prelude::Entity::from_raw(1));
    cell.zone = ZoneType::ResidentialLow;
    assert_eq!(cell_to_char(&cell), 'R');

    cell.zone = ZoneType::CommercialLow;
    assert_eq!(cell_to_char(&cell), 'C');

    // Building without zone → generic B
    cell.zone = ZoneType::None;
    assert_eq!(cell_to_char(&cell), 'B');
}

// -----------------------------------------------------------------------
// Overview map
// -----------------------------------------------------------------------

#[test]
fn test_empty_grid_overview_all_dots() {
    let city = TestCity::new();
    let grid = city.grid();
    let overview = build_overview_map(grid);

    // The overview should have 64 content lines (plus header + legend).
    // Content lines start after the column header.
    let lines: Vec<&str> = overview.lines().collect();

    // First line is column header, then 64 rows of content.
    // Count how many content rows have ONLY dots (plus the row label).
    let content_lines = &lines[1..65];
    for line in content_lines {
        // After the "XXXX | " prefix (7 chars), all chars should be '.'
        let payload = &line[7..];
        assert!(
            payload.chars().all(|c| c == '.'),
            "Expected all dots in empty grid overview, got: {payload}"
        );
    }
}

#[test]
fn test_overview_dimensions() {
    let city = TestCity::new();
    let grid = city.grid();
    let overview = build_overview_map(grid);
    let lines: Vec<&str> = overview.lines().collect();

    // 1 header line + 64 content lines + 1 blank + 6 legend lines = 72
    // Content lines: exactly 64
    let content_lines = &lines[1..65];
    assert_eq!(content_lines.len(), 64);

    // Each content line should have 7 chars prefix + 64 map chars = 71
    for line in content_lines {
        // The line may be "     | " or "XXXX | " followed by 64 chars
        let after_prefix = &line[7..];
        assert_eq!(
            after_prefix.len(),
            64,
            "Expected 64 map chars, got {}",
            after_prefix.len()
        );
    }
}

#[test]
fn test_overview_priority() {
    // Create a small WorldGrid (just 4x4 so it maps to one overview cell)
    // with mixed types to test priority.
    let mut grid = WorldGrid::new(256, 256);

    // Place water in cell (0,0) and grass elsewhere in the 4x4 block.
    // Water should win (priority 5 > 0).
    grid.get_mut(0, 0).cell_type = CellType::Water;

    let overview = build_overview_map(&grid);
    let lines: Vec<&str> = overview.lines().collect();
    // First content line (line index 1), first map char at position 7
    let first_char = lines[1].chars().nth(7).unwrap();
    assert_eq!(first_char, '~', "Water should dominate the 4x4 block");

    // Now add a road — road priority (4) < water (5), water still wins
    grid.get_mut(1, 0).cell_type = CellType::Road;
    grid.get_mut(1, 0).road_type = RoadType::Local;
    let overview = build_overview_map(&grid);
    let lines: Vec<&str> = overview.lines().collect();
    let first_char = lines[1].chars().nth(7).unwrap();
    assert_eq!(first_char, '~', "Water should still dominate over road");

    // Clear water, road should now dominate over grass/zone
    grid.get_mut(0, 0).cell_type = CellType::Grass;
    grid.get_mut(2, 0).zone = ZoneType::ResidentialLow;
    let overview = build_overview_map(&grid);
    let lines: Vec<&str> = overview.lines().collect();
    let first_char = lines[1].chars().nth(7).unwrap();
    assert_eq!(first_char, '#', "Road should dominate over zone");
}

// -----------------------------------------------------------------------
// Detail map
// -----------------------------------------------------------------------

#[test]
fn test_detail_map_empty_grid() {
    let city = TestCity::new();
    let grid = city.grid();
    let detail = build_detail_map(grid, 2);
    assert!(
        detail.contains("empty grid"),
        "Empty grid should return a message, got: {detail}"
    );
}

#[test]
fn test_road_appears_in_detail_map() {
    let city = TestCity::new()
        .with_road(50, 50, 55, 50, RoadType::Local);
    let grid = city.grid();
    let detail = build_detail_map(grid, 2);

    // The detail map should contain '#' characters for the road.
    assert!(
        detail.contains('#'),
        "Detail map should contain road chars '#'"
    );
}

#[test]
fn test_zone_appears_in_detail_map() {
    let city = TestCity::new()
        .with_zone(100, 100, ZoneType::ResidentialLow);
    let grid = city.grid();
    let detail = build_detail_map(grid, 1);

    assert!(
        detail.contains('r'),
        "Detail map should contain zone char 'r' for ResidentialLow"
    );
}

#[test]
fn test_water_cells_render_tilde() {
    let mut city = TestCity::new();
    // Manually set some cells to water
    {
        let world = city.world_mut();
        let mut grid = world.resource_mut::<WorldGrid>();
        for x in 10..15 {
            grid.get_mut(x, 10).cell_type = CellType::Water;
        }
    }
    let grid = city.grid();
    let detail = build_detail_map(grid, 1);
    assert!(
        detail.contains('~'),
        "Detail map should contain water char '~'"
    );
}

#[test]
fn test_detail_map_crops_to_content() {
    // Place a single zone cell at (100, 100). The detail map with margin=2
    // should be roughly 5x5 (100-2..100+2), NOT the full 256x256.
    let city = TestCity::new()
        .with_zone(100, 100, ZoneType::Industrial);
    let grid = city.grid();
    let detail = build_detail_map(grid, 2);
    let lines: Vec<&str> = detail.lines().collect();

    // Should have: 1 header + 5 content rows + 1 blank + 6 legend = 13
    // Content rows: from y=98 to y=102 (inclusive) = 5 rows
    // (header, then 5 content rows before the blank/legend)
    let content_count = lines
        .iter()
        .filter(|l| l.contains(" | "))
        .count();
    assert_eq!(
        content_count, 5,
        "Expected 5 content rows (margin=2 around single cell), got {content_count}"
    );

    // Each content row should have 5 map chars (100-2..100+2)
    for line in lines.iter().filter(|l| l.contains(" | ")) {
        let payload = line.split(" | ").nth(1).unwrap();
        assert_eq!(
            payload.len(),
            5,
            "Expected 5 map chars per row, got {}",
            payload.len()
        );
    }
}
